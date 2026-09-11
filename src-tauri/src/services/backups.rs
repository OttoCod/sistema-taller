use std::path::{Path, PathBuf};

use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};
use crate::models::backup::{
    Backup, ConfiguracionBackups, ManifiestoBackup, RestauracionPreparada,
};
use crate::services::configuracion;

pub const RETENCION_POR_DEFECTO: i64 = 10;
/// Cada cuánto se saca una copia sola al abrir la app.
const HORAS_ENTRE_AUTOMATICOS: i64 = 24;
/// Marcador que deja `preparar_restauracion` y consume `db::init_pool` en
/// el arranque siguiente. Ver el comentario de `aplicar_pendiente`.
const ARCHIVO_MARCADOR: &str = "restauracion_pendiente.txt";

const SELECT_BACKUP: &str = "
    SELECT id, archivo_path, fecha, tamano_bytes, tipo, version_esquema, 0 AS archivo_existe
    FROM backups
";

/// Carpeta por defecto: adentro de los datos de la app. Se usa mientras el
/// negocio no elija una propia (idealmente en otro disco o un pendrive --
/// un backup en el mismo disco no protege contra que se rompa el disco).
pub fn carpeta_por_defecto(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("backups")
}

pub async fn obtener_configuracion(
    pool: &SqlitePool,
    app_data_dir: &Path,
) -> AppResult<ConfiguracionBackups> {
    let carpeta = configuracion::obtener_backups_carpeta(pool).await?;
    let retencion = configuracion::obtener_backups_retencion(pool).await?;
    let carpeta_efectiva = if carpeta.trim().is_empty() {
        carpeta_por_defecto(app_data_dir).display().to_string()
    } else {
        carpeta.clone()
    };
    Ok(ConfiguracionBackups {
        carpeta,
        retencion,
        carpeta_efectiva,
    })
}

pub async fn guardar_configuracion(
    pool: &SqlitePool,
    carpeta: &str,
    retencion: i64,
) -> AppResult<()> {
    if retencion < 1 {
        return Err(AppError::Validation(
            "Hay que conservar al menos 1 copia.".into(),
        ));
    }
    let carpeta = carpeta.trim();
    if !carpeta.is_empty() && !Path::new(carpeta).is_dir() {
        return Err(AppError::Validation(
            "Esa carpeta no existe o no se puede abrir.".into(),
        ));
    }
    configuracion::guardar_backups(pool, carpeta, retencion).await
}

async fn version_esquema(pool: &SqlitePool) -> AppResult<i64> {
    let fila: Option<(i64,)> = sqlx::query_as("SELECT MAX(version) FROM _sqlx_migrations")
        .fetch_optional(pool)
        .await?;
    Ok(fila.map(|(v,)| v).unwrap_or(0))
}

fn nombre_de_archivo(tipo: &str, fecha: &chrono::DateTime<chrono::Utc>) -> String {
    format!("espinola-{}-{}.db", fecha.format("%Y%m%d-%H%M%S"), tipo)
}

/// Saca una copia con `VACUUM INTO`: SQLite escribe una base nueva,
/// completa y consistente, sin importar qué haya a medio escribir en el
/// WAL en ese momento. Copiar el `.db` a mano mientras la app lo usa
/// puede dar un archivo corrupto -- por eso nunca se hace así.
pub async fn crear(pool: &SqlitePool, app_data_dir: &Path, tipo: &str) -> AppResult<Backup> {
    if !["manual", "automatico", "previo_restauracion"].contains(&tipo) {
        return Err(AppError::Validation("Tipo de backup inválido.".into()));
    }

    let config = obtener_configuracion(pool, app_data_dir).await?;
    let carpeta = PathBuf::from(&config.carpeta_efectiva);
    std::fs::create_dir_all(&carpeta)?;

    let ahora = chrono::Utc::now();
    let destino = carpeta.join(nombre_de_archivo(tipo, &ahora));
    if destino.exists() {
        return Err(AppError::Conflict(
            "Ya existe un backup con ese nombre; probá de nuevo en unos segundos.".into(),
        ));
    }

    // El path va interpolado porque VACUUM INTO no admite parámetros. Se
    // escapan las comillas simples para que un nombre de carpeta raro no
    // rompa la sentencia.
    let destino_sql = destino.display().to_string().replace('\'', "''");
    sqlx::query(&format!("VACUUM INTO '{destino_sql}'"))
        .execute(pool)
        .await
        .map_err(|err| {
            AppError::Unexpected(format!("no se pudo generar la copia de la base: {err}"))
        })?;

    let tamano_bytes = std::fs::metadata(&destino)?.len() as i64;
    let version = version_esquema(pool).await?;

    let manifiesto = ManifiestoBackup {
        archivo: destino
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default(),
        fecha: ahora.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
        tipo: tipo.to_string(),
        version_esquema: version,
        version_app: env!("CARGO_PKG_VERSION").to_string(),
    };
    std::fs::write(
        destino.with_extension("json"),
        serde_json::to_string_pretty(&manifiesto).map_err(|err| {
            AppError::Unexpected(format!("no se pudo armar el manifiesto: {err}"))
        })?,
    )?;

    let id = sqlx::query(
        "INSERT INTO backups (archivo_path, fecha, tamano_bytes, tipo, version_esquema)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(destino.display().to_string())
    .bind(&manifiesto.fecha)
    .bind(tamano_bytes)
    .bind(tipo)
    .bind(version)
    .execute(pool)
    .await?
    .last_insert_rowid();

    sqlx::query(
        "INSERT INTO auditoria (entidad_tipo, entidad_id, accion, detalle_json, usuario_id)
         VALUES ('backup', ?, 'backup_creado', ?, 1)",
    )
    .bind(id)
    .bind(serde_json::json!({ "tipo": tipo, "tamanoBytes": tamano_bytes }).to_string())
    .execute(pool)
    .await?;

    rotar(pool, config.retencion).await?;

    obtener(pool, id).await
}

/// Conserva las últimas N copias y borra las más viejas, archivo y
/// manifiesto incluidos. Las copias previas a una restauración quedan
/// siempre afuera de la rotación: son la red de contención de una
/// operación destructiva, no una copia rutinaria más.
async fn rotar(pool: &SqlitePool, retencion: i64) -> AppResult<()> {
    let sobrantes: Vec<(i64, String)> = sqlx::query_as(
        "SELECT id, archivo_path FROM backups
         WHERE tipo != 'previo_restauracion'
         ORDER BY fecha DESC, id DESC
         LIMIT -1 OFFSET ?",
    )
    .bind(retencion)
    .fetch_all(pool)
    .await?;

    for (id, path) in sobrantes {
        let archivo = PathBuf::from(&path);
        // Si el archivo ya no está (lo borraron a mano), igual se saca de
        // la tabla: el objetivo es que el listado refleje la realidad.
        let _ = std::fs::remove_file(&archivo);
        let _ = std::fs::remove_file(archivo.with_extension("json"));
        sqlx::query("DELETE FROM backups WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
    }
    Ok(())
}

async fn obtener(pool: &SqlitePool, id: i64) -> AppResult<Backup> {
    let mut backup = sqlx::query_as::<_, Backup>(&format!("{SELECT_BACKUP} WHERE id = ?"))
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("No existe el backup {id}.")))?;
    backup.archivo_existe = Path::new(&backup.archivo_path).is_file();
    Ok(backup)
}

pub async fn listar(pool: &SqlitePool) -> AppResult<Vec<Backup>> {
    let mut backups =
        sqlx::query_as::<_, Backup>(&format!("{SELECT_BACKUP} ORDER BY fecha DESC, id DESC"))
            .fetch_all(pool)
            .await?;
    for backup in &mut backups {
        backup.archivo_existe = Path::new(&backup.archivo_path).is_file();
    }
    Ok(backups)
}

/// True si nunca se hizo un backup o si el último ya tiene más de 24 h.
pub async fn necesita_automatico(pool: &SqlitePool) -> AppResult<bool> {
    let fila: Option<(String,)> = sqlx::query_as("SELECT MAX(fecha) FROM backups")
        .fetch_optional(pool)
        .await?;
    let Some((ultima,)) = fila.filter(|(f,)| !f.is_empty()) else {
        return Ok(true);
    };
    let Ok(ultima) = chrono::DateTime::parse_from_rfc3339(&ultima) else {
        return Ok(true);
    };
    let horas = chrono::Utc::now()
        .signed_duration_since(ultima.with_timezone(&chrono::Utc))
        .num_hours();
    Ok(horas >= HORAS_ENTRE_AUTOMATICOS)
}

fn marcador_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(ARCHIVO_MARCADOR)
}

/// Paso 1 y 2 de la restauración (ver ARQUITECTURA.md, sección 5): saca
/// una copia de seguridad de la base actual y deja anotado qué archivo hay
/// que restaurar. **No toca la base en uso**: reemplazar el `.db` mientras
/// la app lo tiene abierto es justamente la forma de corromperlo. El
/// reemplazo real ocurre en el arranque siguiente (`aplicar_pendiente`).
pub async fn preparar_restauracion(
    pool: &SqlitePool,
    app_data_dir: &Path,
    id: i64,
) -> AppResult<RestauracionPreparada> {
    let backup = obtener(pool, id).await?;
    let origen = PathBuf::from(&backup.archivo_path);
    if !origen.is_file() {
        return Err(AppError::Validation(
            "El archivo de ese backup ya no está en su carpeta: no se puede restaurar.".into(),
        ));
    }

    let version_actual = version_esquema(pool).await?;
    if backup.version_esquema > version_actual {
        return Err(AppError::Validation(format!(
            "Ese backup es de una versión más nueva del sistema (esquema {} contra {}). \
             Actualizá la aplicación antes de restaurarlo.",
            backup.version_esquema, version_actual
        )));
    }

    verificar_archivo_sqlite(&origen).await?;

    let copia = crear(pool, app_data_dir, "previo_restauracion").await?;

    std::fs::write(marcador_path(app_data_dir), origen.display().to_string())?;

    sqlx::query(
        "INSERT INTO auditoria (entidad_tipo, entidad_id, accion, detalle_json, usuario_id)
         VALUES ('backup', ?, 'restauracion_preparada', ?, 1)",
    )
    .bind(id)
    .bind(
        serde_json::json!({
            "backup": backup.archivo_path,
            "copiaDeSeguridad": copia.archivo_path,
        })
        .to_string(),
    )
    .execute(pool)
    .await?;

    Ok(RestauracionPreparada {
        backup_restaurado: backup.archivo_path,
        copia_de_seguridad: copia.archivo_path,
    })
}

/// Abre el archivo como base SQLite y corre `integrity_check` antes de
/// dejar que reemplace a la base real. Restaurar un archivo corrupto o que
/// no es una base dejaría la app sin datos y sin forma de arrancar.
async fn verificar_archivo_sqlite(archivo: &Path) -> AppResult<()> {
    use sqlx::sqlite::SqliteConnectOptions;
    use sqlx::ConnectOptions;

    let mut conexion = SqliteConnectOptions::new()
        .filename(archivo)
        .read_only(true)
        .connect()
        .await
        .map_err(|_| {
            AppError::Validation(
                "Ese archivo no se puede abrir como base de datos: puede estar dañado.".into(),
            )
        })?;

    let (resultado,): (String,) = sqlx::query_as("PRAGMA integrity_check")
        .fetch_one(&mut conexion)
        .await
        .map_err(|_| {
            AppError::Validation("No se pudo verificar la integridad de ese backup.".into())
        })?;
    if resultado != "ok" {
        return Err(AppError::Validation(
            "Ese backup está dañado: SQLite no lo da por íntegro.".into(),
        ));
    }

    let (tablas,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'ventas'",
    )
    .fetch_one(&mut conexion)
    .await
    .map_err(|_| AppError::Validation("No se pudo leer el contenido de ese backup.".into()))?;
    if tablas == 0 {
        return Err(AppError::Validation(
            "Ese archivo no parece una base de este sistema.".into(),
        ));
    }

    Ok(())
}

/// Paso 3: se llama al arrancar, **antes** de abrir la base. Si hay una
/// restauración preparada, reemplaza el `.db` por el backup elegido y
/// borra el marcador. Devuelve el archivo restaurado, si hubo alguno.
///
/// Hacerlo acá y no en caliente es a propósito: en este punto nadie tiene
/// la base abierta todavía, así que el reemplazo es un simple copy sin
/// riesgo de dejar el archivo a medio escribir.
pub fn aplicar_pendiente(app_data_dir: &Path, db_path: &Path) -> AppResult<Option<String>> {
    let marcador = marcador_path(app_data_dir);
    if !marcador.is_file() {
        return Ok(None);
    }

    let origen = std::fs::read_to_string(&marcador)?.trim().to_string();
    // Pase lo que pase con la copia, el marcador se borra: si quedara,
    // la app entraría en un bucle de restaurar lo mismo en cada arranque.
    let _ = std::fs::remove_file(&marcador);

    let origen = PathBuf::from(origen);
    if !origen.is_file() {
        return Err(AppError::Unexpected(format!(
            "quedó pendiente restaurar {} pero ese archivo ya no está",
            origen.display()
        )));
    }

    // El WAL y el shm que haya son del archivo viejo: si sobreviven,
    // SQLite podría aplicarlos encima de la base recién restaurada. Se
    // borran antes (para que nadie los replique durante la copia) y
    // después (por si el copiado los recreó). Perder ese WAL es
    // intencional: su contenido ya quedó guardado en la copia de
    // seguridad que sacó `preparar_restauracion`.
    let wal = db_path.with_extension("db-wal");
    let shm = db_path.with_extension("db-shm");
    let _ = std::fs::remove_file(&wal);
    let _ = std::fs::remove_file(&shm);

    std::fs::copy(&origen, db_path)?;

    let _ = std::fs::remove_file(&wal);
    let _ = std::fs::remove_file(&shm);

    Ok(Some(origen.display().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::producto::GuardarProducto;
    use crate::services::productos;

    async fn pool_en(dir: &Path) -> SqlitePool {
        crate::db::init_pool(dir).await.expect("init_pool")
    }

    async fn crear_producto(pool: &SqlitePool, nombre: &str) {
        productos::crear(
            pool,
            GuardarProducto {
                nombre: nombre.to_string(),
                marca_id: None,
                categoria_id: None,
                descripcion: None,
                observaciones: None,
                costo_actual: None,
                precio_venta_actual: Some(100_000),
                precio_publico_referencia: None,
                estado: "activo".to_string(),
                codigos_fabricante: vec![],
            },
        )
        .await
        .expect("crear producto");
    }

    #[tokio::test]
    async fn crear_backup_genera_archivo_manifiesto_y_registro() {
        let dir = tempfile::tempdir().expect("tempdir");
        let pool = pool_en(dir.path()).await;
        crear_producto(&pool, "Casco").await;

        let backup = crear(&pool, dir.path(), "manual")
            .await
            .expect("crear backup");

        assert_eq!(backup.tipo, "manual");
        assert!(backup.tamano_bytes > 0);
        assert!(backup.archivo_existe);
        assert!(Path::new(&backup.archivo_path).is_file());

        let manifiesto = PathBuf::from(&backup.archivo_path).with_extension("json");
        assert!(manifiesto.is_file());
        let contenido = std::fs::read_to_string(manifiesto).unwrap();
        assert!(contenido.contains("versionEsquema"));

        let listado = listar(&pool).await.unwrap();
        assert_eq!(listado.len(), 1);
    }

    #[tokio::test]
    async fn el_backup_es_una_base_valida_con_los_datos() {
        let dir = tempfile::tempdir().expect("tempdir");
        let pool = pool_en(dir.path()).await;
        crear_producto(&pool, "Casco integral").await;

        let backup = crear(&pool, dir.path(), "manual").await.unwrap();

        // Se abre la copia como base propia y se busca el producto adentro.
        let copia = SqlitePool::connect(&format!("sqlite:{}", backup.archivo_path))
            .await
            .expect("abrir la copia");
        let (nombre,): (String,) = sqlx::query_as("SELECT nombre FROM productos LIMIT 1")
            .fetch_one(&copia)
            .await
            .expect("leer el producto de la copia");
        assert_eq!(nombre, "Casco integral");
    }

    #[tokio::test]
    async fn la_retencion_rota_las_copias_mas_viejas() {
        let dir = tempfile::tempdir().expect("tempdir");
        let pool = pool_en(dir.path()).await;
        guardar_configuracion(&pool, "", 2).await.unwrap();

        let mut paths = Vec::new();
        for _ in 0..3 {
            let backup = crear(&pool, dir.path(), "manual").await.unwrap();
            paths.push(backup.archivo_path);
            // VACUUM INTO falla si el destino existe y el nombre lleva
            // segundos: se espera para que no colisionen.
            tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
        }

        let listado = listar(&pool).await.unwrap();
        assert_eq!(listado.len(), 2, "solo quedan las 2 más nuevas");
        assert!(
            !Path::new(&paths[0]).exists(),
            "la más vieja se borró del disco"
        );
        assert!(Path::new(&paths[2]).exists(), "la más nueva sigue ahí");
    }

    #[tokio::test]
    async fn restaurar_deja_marcador_y_copia_de_seguridad_sin_tocar_la_base() {
        let dir = tempfile::tempdir().expect("tempdir");
        let pool = pool_en(dir.path()).await;
        crear_producto(&pool, "Producto original").await;

        let backup = crear(&pool, dir.path(), "manual").await.unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(1100)).await;

        // Después de la copia se agrega otro producto: si la restauración
        // se aplicara sola acá, este producto ya no estaría.
        crear_producto(&pool, "Producto posterior").await;

        let preparada = preparar_restauracion(&pool, dir.path(), backup.id)
            .await
            .expect("preparar restauración");

        assert!(Path::new(&preparada.copia_de_seguridad).is_file());
        assert!(marcador_path(dir.path()).is_file());

        // La base en uso sigue intacta: la restauración todavía no pasó.
        let (cantidad,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM productos")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(cantidad, 2);
    }

    #[tokio::test]
    async fn aplicar_pendiente_reemplaza_la_base_y_borra_el_marcador() {
        let dir = tempfile::tempdir().expect("tempdir");
        let pool = pool_en(dir.path()).await;
        crear_producto(&pool, "Producto original").await;

        let backup = crear(&pool, dir.path(), "manual").await.unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
        crear_producto(&pool, "Producto posterior").await;

        preparar_restauracion(&pool, dir.path(), backup.id)
            .await
            .unwrap();

        // Simula el reinicio de la app: cerrar el pool y soltarlo. El
        // cierre real del archivo lo termina sqlx en un hilo de fondo, así
        // que hay que esperarlo -- si no, ese cierre tardío escribe las
        // páginas viejas encima de la base recién restaurada. En
        // producción esto no aplica: aplicar_pendiente corre al arrancar,
        // antes de que exista ningún pool.
        pool.close().await;
        drop(pool);
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;

        let restaurado = aplicar_pendiente(dir.path(), &crate::db::db_path(dir.path()))
            .expect("aplicar pendiente");
        assert!(restaurado.is_some());
        assert!(
            !marcador_path(dir.path()).is_file(),
            "el marcador se consume"
        );

        // Al reabrir, solo está el producto que existía cuando se sacó la copia.
        let pool = pool_en(dir.path()).await;
        let (cantidad,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM productos")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(cantidad, 1);
        let (nombre,): (String,) = sqlx::query_as("SELECT nombre FROM productos")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(nombre, "Producto original");
    }

    #[tokio::test]
    async fn sin_marcador_no_hay_nada_que_aplicar() {
        let dir = tempfile::tempdir().expect("tempdir");
        let resultado = aplicar_pendiente(dir.path(), &crate::db::db_path(dir.path())).unwrap();
        assert!(resultado.is_none());
    }

    #[tokio::test]
    async fn no_se_restaura_un_archivo_que_no_es_una_base() {
        let dir = tempfile::tempdir().expect("tempdir");
        let pool = pool_en(dir.path()).await;
        let backup = crear(&pool, dir.path(), "manual").await.unwrap();

        // Se pisa el archivo del backup con basura.
        std::fs::write(&backup.archivo_path, b"esto no es una base de datos").unwrap();

        let resultado = preparar_restauracion(&pool, dir.path(), backup.id).await;
        assert!(matches!(resultado, Err(AppError::Validation(_))));
        assert!(
            !marcador_path(dir.path()).is_file(),
            "no queda ninguna restauración pendiente"
        );
    }

    #[tokio::test]
    async fn el_primer_arranque_siempre_necesita_backup_automatico() {
        let dir = tempfile::tempdir().expect("tempdir");
        let pool = pool_en(dir.path()).await;

        assert!(necesita_automatico(&pool).await.unwrap());

        crear(&pool, dir.path(), "automatico").await.unwrap();
        assert!(
            !necesita_automatico(&pool).await.unwrap(),
            "recién hecho, no hace falta otro"
        );
    }

    #[tokio::test]
    async fn la_retencion_no_puede_ser_cero() {
        let dir = tempfile::tempdir().expect("tempdir");
        let pool = pool_en(dir.path()).await;
        let resultado = guardar_configuracion(&pool, "", 0).await;
        assert!(matches!(resultado, Err(AppError::Validation(_))));
    }

    #[tokio::test]
    async fn una_carpeta_inexistente_no_se_guarda() {
        let dir = tempfile::tempdir().expect("tempdir");
        let pool = pool_en(dir.path()).await;
        let resultado = guardar_configuracion(&pool, "/no/existe/esta/carpeta", 5).await;
        assert!(matches!(resultado, Err(AppError::Validation(_))));
    }
}
