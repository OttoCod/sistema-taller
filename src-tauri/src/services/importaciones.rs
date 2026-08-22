use std::collections::HashMap;
use std::path::Path;

use sha2::{Digest, Sha256};
use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};
use crate::models::importacion::{
    FilaCruda, Importacion, ImportacionFila, ResolverFila, ResumenImportacion,
};
use crate::models::producto::GuardarProducto;
use crate::services::lector_excel;
use crate::services::productos;

const DECISION_CREAR: &str = "crear_nuevo";
const DECISION_VINCULAR: &str = "vincular_existente";
const DECISION_OMITIR: &str = "omitir";

const CLASIF_VALIDO: &str = "producto_valido";
const CLASIF_SECCION: &str = "seccion";
const CLASIF_IGNORADA: &str = "ignorada";
const CLASIF_REVISION: &str = "requiere_revision";
const CLASIF_ERROR: &str = "error";

fn normalizar_para_comparar(texto: &str) -> String {
    let sin_acentos: String = texto
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| match c {
            'á' => 'a',
            'é' => 'e',
            'í' => 'i',
            'ó' => 'o',
            'ú' => 'u',
            'ü' => 'u',
            'ñ' => 'n',
            other => other,
        })
        .collect();
    sin_acentos.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn normalizar_codigo(codigo: &str) -> String {
    codigo.trim().to_lowercase()
}

/// Clasifica una fila cruda según las reglas acordadas (ver diseño de la
/// Fase 3): nunca fusiona ni asume -- una fila "problemática" siempre
/// queda para revisión manual, jamás se descarta en silencio.
fn clasificar(cruda: &FilaCruda) -> (&'static str, Option<String>) {
    let todo_vacio = cruda.codigo.is_none()
        && cruda.nombre.is_none()
        && cruda.precio_centavos.is_none()
        && cruda.precio_texto_invalido.is_none();
    if todo_vacio {
        return (CLASIF_IGNORADA, None);
    }
    if let Some(texto) = &cruda.precio_texto_invalido {
        return (
            CLASIF_ERROR,
            Some(format!("El precio no es un número válido: \"{texto}\".")),
        );
    }
    // Sin código y sin precio, pero con un nombre: es un título de
    // sección (ej. "CUBIERTAS ECONOMICAS"), no un producto.
    if cruda.codigo.is_none() && cruda.precio_centavos.is_none() && cruda.nombre.is_some() {
        return (CLASIF_SECCION, None);
    }
    // Sin nombre no hay producto posible (nombre es NOT NULL en el
    // esquema) -- pero tampoco se descarta, va a revisión.
    if cruda.nombre.is_none() {
        return (CLASIF_REVISION, None);
    }
    (CLASIF_VALIDO, None)
}

struct FilaClasificada {
    cruda: FilaCruda,
    categoria_excel_texto: Option<String>,
    clasificacion: &'static str,
    motivo_error: Option<String>,
    es_duplicado_codigo: bool,
    es_posible_duplicado_nombre: bool,
}

/// Clasifica todas las filas en orden, arrastrando la categoría vigente
/// desde el último título de sección visto (si el Excel no tenía
/// ninguno todavía, queda sin categoría -- nunca se inventa una).
fn clasificar_todas(filas_crudas: Vec<FilaCruda>) -> Vec<FilaClasificada> {
    let mut categoria_vigente: Option<String> = None;
    filas_crudas
        .into_iter()
        .map(|cruda| {
            let (clasificacion, motivo_error) = clasificar(&cruda);
            if clasificacion == CLASIF_SECCION {
                categoria_vigente = cruda.nombre.clone();
            }
            let categoria_excel_texto = if clasificacion == CLASIF_VALIDO {
                categoria_vigente.clone()
            } else {
                None
            };
            FilaClasificada {
                cruda,
                categoria_excel_texto,
                clasificacion,
                motivo_error,
                es_duplicado_codigo: false,
                es_posible_duplicado_nombre: false,
            }
        })
        .collect()
}

/// Marca duplicados DENTRO del mismo archivo -- nunca fusiona, solo deja
/// la marca para que la revisión los muestre agrupados (ver diseño,
/// punto 4). Solo mira filas `producto_valido`: una fila de sección o en
/// revisión no participa de esta comparación.
fn marcar_duplicados_internos(filas: &mut [FilaClasificada]) {
    let mut por_codigo: HashMap<String, Vec<usize>> = HashMap::new();
    let mut por_nombre: HashMap<String, Vec<usize>> = HashMap::new();

    for (indice, fila) in filas.iter().enumerate() {
        if fila.clasificacion != CLASIF_VALIDO {
            continue;
        }
        if let Some(codigo) = &fila.cruda.codigo {
            por_codigo
                .entry(normalizar_codigo(codigo))
                .or_default()
                .push(indice);
        }
        if let Some(nombre) = &fila.cruda.nombre {
            por_nombre
                .entry(normalizar_para_comparar(nombre))
                .or_default()
                .push(indice);
        }
    }

    for indices in por_codigo.values().filter(|v| v.len() > 1) {
        for &indice in indices {
            filas[indice].es_duplicado_codigo = true;
        }
    }
    for indices in por_nombre.values().filter(|v| v.len() > 1) {
        for &indice in indices {
            filas[indice].es_posible_duplicado_nombre = true;
        }
    }
}

fn precios_derivados_a_json(pares: &[(String, f64)]) -> Option<String> {
    if pares.is_empty() {
        return None;
    }
    let mapa: serde_json::Map<String, serde_json::Value> = pares
        .iter()
        .map(|(clave, valor)| (clave.clone(), serde_json::json!(valor)))
        .collect();
    Some(serde_json::Value::Object(mapa).to_string())
}

/// Lee el archivo (solo lectura, nunca lo modifica), detecta su
/// estructura, clasifica y marca duplicados, y guarda todo en
/// `importaciones`/`importacion_filas` como borrador -- nada de esto toca
/// todavía la tabla `productos`. Eso pasa recién cuando se resuelve y
/// aplica cada fila (ver `resolver`/`aplicar`).
pub async fn procesar_archivo(
    pool: &SqlitePool,
    ruta: &Path,
    archivo_nombre: String,
) -> AppResult<Importacion> {
    let bytes = std::fs::read(ruta)?;
    let hash = {
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        format!("{:x}", hasher.finalize())
    };

    let (estructura, filas_crudas) = lector_excel::leer_archivo(ruta)?;
    if filas_crudas.is_empty() {
        return Err(AppError::Validation(
            "El archivo no tiene ninguna fila de datos debajo del encabezado detectado.".into(),
        ));
    }

    let mut filas = clasificar_todas(filas_crudas);
    marcar_duplicados_internos(&mut filas);

    // Coincidencias contra el catálogo ya existente, por codigo_legado
    // (de una importación anterior o carga manual). Solo se guarda como
    // sugerencia -- nunca se aplica sola (ver diseño, punto 6).
    let existentes: Vec<(i64, Option<String>)> =
        sqlx::query_as("SELECT id, codigo_legado FROM productos WHERE codigo_legado IS NOT NULL")
            .fetch_all(pool)
            .await?;
    let mapa_existentes: HashMap<String, i64> = existentes
        .into_iter()
        .filter_map(|(id, codigo)| codigo.map(|c| (normalizar_codigo(&c), id)))
        .collect();

    let columnas_json = serde_json::to_string(&estructura.columnas)
        .expect("serializar columnas detectadas no debería fallar");

    let mut tx = pool.begin().await?;

    let importacion_id = sqlx::query(
        "INSERT INTO importaciones
            (archivo_nombre, archivo_hash, hoja_detectada, fila_encabezado_detectada, columnas_detectadas_json, total_filas)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&archivo_nombre)
    .bind(&hash)
    .bind(&estructura.hoja)
    .bind(estructura.fila_encabezado)
    .bind(&columnas_json)
    .bind(filas.len() as i64)
    .execute(&mut *tx)
    .await?
    .last_insert_rowid();

    for fila in &filas {
        let coincide_producto_existente_id = if fila.clasificacion == CLASIF_VALIDO {
            fila.cruda
                .codigo
                .as_deref()
                .map(normalizar_codigo)
                .and_then(|c| mapa_existentes.get(&c))
                .copied()
        } else {
            None
        };

        // Las filas limpias (sin ningún flag de duplicado ni coincidencia
        // con el catálogo) arrancan con la decisión ya tomada, para no
        // obligar a aprobar manualmente miles de filas sin problemas.
        let decision_automatica = (fila.clasificacion == CLASIF_VALIDO
            && !fila.es_duplicado_codigo
            && !fila.es_posible_duplicado_nombre
            && coincide_producto_existente_id.is_none())
        .then_some("crear_nuevo");

        let precios_json = precios_derivados_a_json(&fila.cruda.precios_derivados);

        sqlx::query(
            "INSERT INTO importacion_filas
                (importacion_id, fila_excel, codigo_excel, nombre_excel, precio_lista_centavos,
                 categoria_excel_texto, precios_calculados_json, clasificacion, motivo_error,
                 es_duplicado_codigo, es_posible_duplicado_nombre, coincide_producto_existente_id, decision)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(importacion_id)
        .bind(fila.cruda.fila_excel)
        .bind(&fila.cruda.codigo)
        .bind(&fila.cruda.nombre)
        .bind(fila.cruda.precio_centavos)
        .bind(&fila.categoria_excel_texto)
        .bind(&precios_json)
        .bind(fila.clasificacion)
        .bind(&fila.motivo_error)
        .bind(fila.es_duplicado_codigo)
        .bind(fila.es_posible_duplicado_nombre)
        .bind(coincide_producto_existente_id)
        .bind(decision_automatica)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    obtener(pool, importacion_id).await
}

const SELECT_IMPORTACION: &str = "
    SELECT id, archivo_nombre, archivo_hash, hoja_detectada, fila_encabezado_detectada,
           columnas_detectadas_json, estado, total_filas, creada_en, cerrada_en
    FROM importaciones
";

pub async fn listar(pool: &SqlitePool) -> AppResult<Vec<Importacion>> {
    let importaciones = sqlx::query_as::<_, Importacion>(&format!(
        "{SELECT_IMPORTACION} ORDER BY creada_en DESC, id DESC LIMIT 100"
    ))
    .fetch_all(pool)
    .await?;
    Ok(importaciones)
}

pub async fn obtener(pool: &SqlitePool, id: i64) -> AppResult<Importacion> {
    sqlx::query_as::<_, Importacion>(&format!("{SELECT_IMPORTACION} WHERE id = ?"))
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("No existe la importación {id}.")))
}

/// Otra importación ya confirmada con exactamente el mismo archivo (mismo
/// hash) -- se usa para avisar, nunca para bloquear (el usuario puede
/// querer reimportar a propósito).
pub async fn buscar_confirmada_con_mismo_hash(
    pool: &SqlitePool,
    hash: &str,
    excluyendo_id: i64,
) -> AppResult<Option<Importacion>> {
    let importacion = sqlx::query_as::<_, Importacion>(&format!(
        "{SELECT_IMPORTACION} WHERE archivo_hash = ? AND estado = 'confirmada' AND id != ? ORDER BY creada_en DESC LIMIT 1"
    ))
    .bind(hash)
    .bind(excluyendo_id)
    .fetch_optional(pool)
    .await?;
    Ok(importacion)
}

const SELECT_FILA: &str = "
    SELECT id, importacion_id, fila_excel, codigo_excel, nombre_excel, precio_lista_centavos,
           categoria_excel_texto, precios_calculados_json, clasificacion, motivo_error,
           es_duplicado_codigo, es_posible_duplicado_nombre, coincide_producto_existente_id,
           decision, producto_vinculado_id, actualizar_costo_en_vinculo, producto_id, resuelta_en
    FROM importacion_filas
";

pub async fn listar_filas(
    pool: &SqlitePool,
    importacion_id: i64,
) -> AppResult<Vec<ImportacionFila>> {
    let filas = sqlx::query_as::<_, ImportacionFila>(&format!(
        "{SELECT_FILA} WHERE importacion_id = ? ORDER BY fila_excel"
    ))
    .bind(importacion_id)
    .fetch_all(pool)
    .await?;
    Ok(filas)
}

pub async fn resumen(pool: &SqlitePool, importacion_id: i64) -> AppResult<ResumenImportacion> {
    let resumen = sqlx::query_as::<_, ResumenImportacion>(
        "SELECT
            COUNT(*) AS total,
            SUM(CASE WHEN clasificacion = 'producto_valido' THEN 1 ELSE 0 END) AS validos,
            SUM(CASE WHEN clasificacion = 'producto_valido' AND codigo_excel IS NULL THEN 1 ELSE 0 END) AS sin_codigo,
            SUM(CASE WHEN clasificacion = 'requiere_revision' THEN 1 ELSE 0 END) AS sin_nombre,
            SUM(CASE WHEN es_duplicado_codigo THEN 1 ELSE 0 END) AS duplicados_codigo,
            SUM(CASE WHEN es_posible_duplicado_nombre THEN 1 ELSE 0 END) AS duplicados_nombre,
            SUM(CASE WHEN coincide_producto_existente_id IS NOT NULL THEN 1 ELSE 0 END) AS coincide_existente,
            SUM(CASE WHEN clasificacion = 'seccion' THEN 1 ELSE 0 END) AS secciones,
            SUM(CASE WHEN clasificacion = 'ignorada' THEN 1 ELSE 0 END) AS ignoradas,
            SUM(CASE WHEN clasificacion = 'error' THEN 1 ELSE 0 END) AS errores,
            SUM(CASE WHEN clasificacion IN ('producto_valido', 'requiere_revision') AND resuelta_en IS NULL THEN 1 ELSE 0 END) AS pendientes,
            SUM(CASE WHEN resuelta_en IS NOT NULL THEN 1 ELSE 0 END) AS resueltas
         FROM importacion_filas
         WHERE importacion_id = ?",
    )
    .bind(importacion_id)
    .fetch_one(pool)
    .await?;
    Ok(resumen)
}

async fn obtener_fila(pool: &SqlitePool, id: i64) -> AppResult<ImportacionFila> {
    sqlx::query_as::<_, ImportacionFila>(&format!("{SELECT_FILA} WHERE id = ?"))
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("No existe la fila de importación {id}.")))
}

/// Crea el producto para una fila `crear_nuevo`: reutiliza
/// `productos::crear` (así el código interno, precios_historial y FTS5
/// quedan exactamente igual que si se hubiera cargado a mano desde el
/// Catálogo). `codigo_legado` se completa aparte porque `GuardarProducto`
/// todavía no lo conoce -- es un campo propio de esta fase.
///
/// Nunca asigna categoría sola: el título de sección del Excel
/// (`categoria_excel_texto`) queda como dato histórico en la fila, pero
/// crear categorías o asociarlas automáticamente quedó descartado --
/// varios títulos "cierran" recién en la última fila del archivo, así
/// que "hasta el próximo título" terminaba categorizando miles de
/// productos que no tenían nada que ver. Eso se resuelve más adelante con
/// una herramienta de categorización aparte, con confirmación humana.
async fn crear_producto_desde_fila(pool: &SqlitePool, fila: &ImportacionFila) -> AppResult<i64> {
    let nombre = fila.nombre_excel.clone().ok_or_else(|| {
        AppError::Validation(
            "No se puede crear el producto sin nombre. Corregí el nombre antes de aplicar.".into(),
        )
    })?;

    let detalle = productos::crear(
        pool,
        GuardarProducto {
            nombre,
            marca_id: None,
            categoria_id: None,
            descripcion: None,
            observaciones: None,
            costo_actual: fila.precio_lista_centavos,
            precio_venta_actual: None,
            precio_publico_referencia: None,
            estado: "activo".to_string(),
            codigos_fabricante: vec![],
        },
    )
    .await?;

    if let Some(codigo) = &fila.codigo_excel {
        sqlx::query("UPDATE productos SET codigo_legado = ? WHERE id = ?")
            .bind(codigo)
            .bind(detalle.producto.id)
            .execute(pool)
            .await?;
    }

    Ok(detalle.producto.id)
}

/// Vincula la fila a un producto ya existente. Nunca toca nombre, marca,
/// categoría, descripción, precio de venta ni stock -- son datos que
/// pueden haber sido corregidos a mano después de cualquier importación
/// anterior. Solo actualiza `costo_actual` (con su historial) si el
/// usuario lo pidió explícitamente, y completa `codigo_legado` solo si
/// el producto todavía no tenía uno.
async fn vincular_producto_existente(
    pool: &SqlitePool,
    fila: &ImportacionFila,
    producto_id: i64,
    actualizar_costo: bool,
) -> AppResult<()> {
    let actual = productos::obtener(pool, producto_id).await?;

    if actualizar_costo {
        if let Some(nuevo_costo) = fila.precio_lista_centavos {
            let datos = GuardarProducto {
                nombre: actual.producto.nombre.clone(),
                marca_id: actual.producto.marca_id,
                categoria_id: actual.producto.categoria_id,
                descripcion: actual.producto.descripcion.clone(),
                observaciones: actual.producto.observaciones.clone(),
                costo_actual: Some(nuevo_costo),
                precio_venta_actual: actual.producto.precio_venta_actual,
                precio_publico_referencia: actual.producto.precio_publico_referencia,
                estado: actual.producto.estado.clone(),
                codigos_fabricante: actual.codigos_fabricante.clone(),
            };
            productos::actualizar(pool, producto_id, datos).await?;
        }
    }

    sqlx::query("UPDATE productos SET codigo_legado = COALESCE(codigo_legado, ?) WHERE id = ?")
        .bind(&fila.codigo_excel)
        .bind(producto_id)
        .execute(pool)
        .await?;

    Ok(())
}

async fn registrar_auditoria(
    pool: &SqlitePool,
    fila: &ImportacionFila,
    accion: &str,
    producto_id: Option<i64>,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO auditoria (entidad_tipo, entidad_id, accion, detalle_json, usuario_id)
         VALUES ('importacion_fila', ?, ?, ?, 1)",
    )
    .bind(fila.id)
    .bind(accion)
    .bind(
        serde_json::json!({
            "importacionId": fila.importacion_id,
            "filaExcel": fila.fila_excel,
            "productoId": producto_id,
        })
        .to_string(),
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Aplica una fila que YA tiene `decision` cargada (sea porque arrancó
/// con `crear_nuevo` automático, o porque el usuario la resolvió a
/// mano): ejecuta la acción, dejando auditoría, y devuelve el producto
/// resultante (si corresponde).
async fn aplicar_fila_ya_decidida(
    pool: &SqlitePool,
    fila: &ImportacionFila,
) -> AppResult<Option<i64>> {
    let decision = fila
        .decision
        .as_deref()
        .ok_or_else(|| AppError::Validation("Esta fila todavía no tiene una decisión.".into()))?;

    let producto_id = match decision {
        DECISION_CREAR => {
            let id = crear_producto_desde_fila(pool, fila).await?;
            registrar_auditoria(pool, fila, "producto_importado_excel", Some(id)).await?;
            Some(id)
        }
        DECISION_VINCULAR => {
            let destino = fila.producto_vinculado_id.ok_or_else(|| {
                AppError::Validation("Elegí a qué producto vincular esta fila.".into())
            })?;
            vincular_producto_existente(pool, fila, destino, fila.actualizar_costo_en_vinculo)
                .await?;
            registrar_auditoria(pool, fila, "producto_vinculado_excel", Some(destino)).await?;
            Some(destino)
        }
        DECISION_OMITIR => {
            registrar_auditoria(pool, fila, "fila_excel_omitida", None).await?;
            None
        }
        otra => {
            return Err(AppError::Validation(format!(
                "Decisión inválida: \"{otra}\"."
            )));
        }
    };

    sqlx::query(
        "UPDATE importacion_filas
         SET producto_id = ?, resuelta_en = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
    )
    .bind(producto_id)
    .bind(fila.id)
    .execute(pool)
    .await?;

    cerrar_si_no_quedan_pendientes(pool, fila.importacion_id).await?;

    Ok(producto_id)
}

async fn cerrar_si_no_quedan_pendientes(pool: &SqlitePool, importacion_id: i64) -> AppResult<()> {
    let (pendientes,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM importacion_filas
         WHERE importacion_id = ?
           AND clasificacion IN ('producto_valido', 'requiere_revision')
           AND resuelta_en IS NULL",
    )
    .bind(importacion_id)
    .fetch_one(pool)
    .await?;

    if pendientes == 0 {
        sqlx::query(
            "UPDATE importaciones
             SET estado = 'confirmada', cerrada_en = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ? AND estado = 'en_revision'",
        )
        .bind(importacion_id)
        .execute(pool)
        .await?;
    }

    Ok(())
}

/// Punto de entrada para resolver una fila a mano desde la revisión:
/// guarda la decisión (y las correcciones de nombre/código, si vinieron)
/// y la aplica de inmediato. Nunca se puede resolver una fila dos veces
/// ni pisar lo que ya se aplicó.
pub async fn resolver_fila(
    pool: &SqlitePool,
    fila_id: i64,
    datos: ResolverFila,
) -> AppResult<ImportacionFila> {
    if ![DECISION_CREAR, DECISION_VINCULAR, DECISION_OMITIR].contains(&datos.decision.as_str()) {
        return Err(AppError::Validation("Decisión inválida.".into()));
    }

    let fila = obtener_fila(pool, fila_id).await?;
    if fila.resuelta_en.is_some() {
        return Err(AppError::Validation("Esta fila ya fue resuelta.".into()));
    }

    if let Some(nombre) = &datos.nombre_corregido {
        let limpio = nombre.trim();
        if limpio.is_empty() {
            return Err(AppError::Validation(
                "El nombre no puede quedar vacío.".into(),
            ));
        }
        sqlx::query("UPDATE importacion_filas SET nombre_excel = ? WHERE id = ?")
            .bind(limpio)
            .bind(fila_id)
            .execute(pool)
            .await?;
    }
    if let Some(codigo) = &datos.codigo_corregido {
        let limpio = codigo.trim();
        let valor: Option<&str> = if limpio.is_empty() {
            None
        } else {
            Some(limpio)
        };
        sqlx::query("UPDATE importacion_filas SET codigo_excel = ? WHERE id = ?")
            .bind(valor)
            .bind(fila_id)
            .execute(pool)
            .await?;
    }

    sqlx::query(
        "UPDATE importacion_filas
         SET decision = ?, producto_vinculado_id = ?, actualizar_costo_en_vinculo = ?
         WHERE id = ?",
    )
    .bind(&datos.decision)
    .bind(datos.producto_vinculado_id)
    .bind(datos.actualizar_costo_en_vinculo)
    .bind(fila_id)
    .execute(pool)
    .await?;

    let fila = obtener_fila(pool, fila_id).await?;
    aplicar_fila_ya_decidida(pool, &fila).await?;

    obtener_fila(pool, fila_id).await
}

/// Aplica en bloque todas las filas que ya tienen una decisión cargada
/// pero todavía no se aplicaron -- típicamente las miles de filas
/// "limpias" que arrancaron con `crear_nuevo` automático. Devuelve
/// cuántas se aplicaron.
pub async fn aplicar_pendientes_con_decision(
    pool: &SqlitePool,
    importacion_id: i64,
) -> AppResult<i64> {
    let filas = sqlx::query_as::<_, ImportacionFila>(&format!(
        "{SELECT_FILA} WHERE importacion_id = ? AND decision IS NOT NULL AND resuelta_en IS NULL"
    ))
    .bind(importacion_id)
    .fetch_all(pool)
    .await?;

    let mut aplicadas = 0i64;
    for fila in &filas {
        aplicar_fila_ya_decidida(pool, fila).await?;
        aplicadas += 1;
    }

    Ok(aplicadas)
}

/// Cancela el intento de importación sin aplicar nada (las filas ya
/// resueltas -- si las hubiera -- no se deshacen, ver diseño: nunca se
/// pierde lo ya aplicado).
pub async fn descartar(pool: &SqlitePool, importacion_id: i64) -> AppResult<Importacion> {
    sqlx::query(
        "UPDATE importaciones
         SET estado = 'descartada', cerrada_en = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ? AND estado = 'en_revision'",
    )
    .bind(importacion_id)
    .execute(pool)
    .await?;
    obtener(pool, importacion_id).await
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn pool_de_prueba() -> SqlitePool {
        let dir = tempfile::tempdir().expect("tempdir");
        let dir = Box::leak(Box::new(dir));
        crate::db::init_pool(dir.path()).await.expect("init_pool")
    }

    fn fixture(nombre: &str) -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(nombre)
    }

    #[tokio::test]
    async fn procesa_archivo_y_clasifica_todo_correctamente() {
        let pool = pool_de_prueba().await;
        let importacion = procesar_archivo(
            &pool,
            &fixture("lista_precios_ok.xlsx"),
            "lista_precios_ok.xlsx".into(),
        )
        .await
        .expect("procesar archivo");

        assert_eq!(importacion.estado, "en_revision");
        assert_eq!(importacion.hoja_detectada, "Hoja3");

        let filas = listar_filas(&pool, importacion.id).await.unwrap();
        let por_nombre = |n: &str| filas.iter().find(|f| f.nombre_excel.as_deref() == Some(n));

        // Producto normal, decisión automática.
        let disco = por_nombre("disco 5 pieza embrague").unwrap();
        assert_eq!(disco.clasificacion, "producto_valido");
        assert_eq!(disco.decision.as_deref(), Some("crear_nuevo"));
        assert_eq!(disco.precio_lista_centavos, Some(174_200));

        // Sin nombre -> revisión, sin decisión automática.
        let sin_nombre = filas
            .iter()
            .find(|f| f.codigo_excel.as_deref() == Some("E2001"))
            .unwrap();
        assert_eq!(sin_nombre.clasificacion, "requiere_revision");
        assert_eq!(sin_nombre.decision, None);

        // Sección: no es producto, no tiene decisión.
        let seccion = por_nombre("SECCION ACEITES").unwrap();
        assert_eq!(seccion.clasificacion, "seccion");
        assert_eq!(seccion.decision, None);

        // La fila de la sección siguiente hereda la categoría del título.
        let aceite = por_nombre("aceite 20w50").unwrap();
        assert_eq!(
            aceite.categoria_excel_texto.as_deref(),
            Some("SECCION ACEITES")
        );

        // Fila totalmente vacía -> ignorada.
        assert!(filas.iter().any(|f| f.clasificacion == "ignorada"));

        // Precio con texto inválido -> error, con motivo.
        let error = por_nombre("precio roto").unwrap();
        assert_eq!(error.clasificacion, "error");
        assert!(error
            .motivo_error
            .as_deref()
            .unwrap()
            .contains("no es un precio"));

        // Códigos duplicados dentro del archivo: mismo código, nombres
        // distintos -- NUNCA se fusionan, ambas quedan marcadas.
        let guino_titan = por_nombre("guiño titan").unwrap();
        let guino_xr200 = por_nombre("guiño xr200").unwrap();
        assert!(guino_titan.es_duplicado_codigo);
        assert!(guino_xr200.es_duplicado_codigo);
        assert_eq!(guino_titan.decision, None); // no arranca con decisión automática
        assert_eq!(guino_xr200.decision, None);

        // Nombres duplicados con códigos distintos.
        let cadena1 = filas
            .iter()
            .find(|f| f.codigo_excel.as_deref() == Some("E3000"))
            .unwrap();
        let cadena2 = filas
            .iter()
            .find(|f| f.codigo_excel.as_deref() == Some("E3001"))
            .unwrap();
        assert!(cadena1.es_posible_duplicado_nombre);
        assert!(cadena2.es_posible_duplicado_nombre);

        // El resumen agregado coincide con lo anterior.
        let resumen = resumen(&pool, importacion.id).await.unwrap();
        assert_eq!(resumen.errores, 1);
        assert_eq!(resumen.secciones, 1);
        assert_eq!(resumen.sin_nombre, 1);
        assert!(resumen.ignoradas >= 1);
        assert_eq!(resumen.duplicados_codigo, 2);
        assert_eq!(resumen.duplicados_nombre, 2);
    }

    #[tokio::test]
    async fn detecta_coincidencia_con_producto_existente_por_codigo_legado() {
        let pool = pool_de_prueba().await;

        // Simula un producto ya cargado con codigo_legado "54" (por
        // ejemplo, de una importación anterior).
        crate::services::productos::crear(
            &pool,
            crate::models::producto::GuardarProducto {
                nombre: "Disco de embrague ya cargado".into(),
                marca_id: None,
                categoria_id: None,
                descripcion: None,
                observaciones: None,
                costo_actual: Some(900_000),
                precio_venta_actual: None,
                precio_publico_referencia: None,
                estado: "activo".into(),
                codigos_fabricante: vec![],
            },
        )
        .await
        .unwrap();
        sqlx::query("UPDATE productos SET codigo_legado = '54' WHERE nombre = 'Disco de embrague ya cargado'")
            .execute(&pool)
            .await
            .unwrap();

        let importacion = procesar_archivo(
            &pool,
            &fixture("lista_precios_ok.xlsx"),
            "lista_precios_ok.xlsx".into(),
        )
        .await
        .unwrap();
        let filas = listar_filas(&pool, importacion.id).await.unwrap();
        let fila = filas
            .iter()
            .find(|f| f.codigo_excel.as_deref() == Some("54"))
            .unwrap();

        assert!(fila.coincide_producto_existente_id.is_some());
        // No arranca con decisión automática: coincidir con el catálogo
        // exige revisión manual, nunca se vincula solo.
        assert_eq!(fila.decision, None);
    }

    #[tokio::test]
    async fn reimportar_el_mismo_archivo_se_puede_detectar_por_hash() {
        let pool = pool_de_prueba().await;
        let primera = procesar_archivo(
            &pool,
            &fixture("lista_precios_ok.xlsx"),
            "lista_precios_ok.xlsx".into(),
        )
        .await
        .unwrap();

        // Todavía no está confirmada: no debería aparecer como aviso.
        let aviso = buscar_confirmada_con_mismo_hash(&pool, &primera.archivo_hash, -1)
            .await
            .unwrap();
        assert!(aviso.is_none());

        sqlx::query("UPDATE importaciones SET estado = 'confirmada' WHERE id = ?")
            .bind(primera.id)
            .execute(&pool)
            .await
            .unwrap();

        let segunda = procesar_archivo(
            &pool,
            &fixture("lista_precios_ok.xlsx"),
            "lista_precios_ok.xlsx".into(),
        )
        .await
        .unwrap();
        let aviso = buscar_confirmada_con_mismo_hash(&pool, &segunda.archivo_hash, segunda.id)
            .await
            .unwrap();
        assert_eq!(aviso.unwrap().id, primera.id);
    }

    #[tokio::test]
    async fn archivo_sin_estructura_reconocible_falla() {
        let pool = pool_de_prueba().await;
        let resultado = procesar_archivo(
            &pool,
            &fixture("sin_estructura_reconocible.xlsx"),
            "sin_estructura_reconocible.xlsx".into(),
        )
        .await;
        assert!(matches!(resultado, Err(AppError::Validation(_))));

        let importaciones = listar(&pool).await.unwrap();
        assert!(importaciones.is_empty());
    }

    #[tokio::test]
    async fn aplicar_pendientes_crea_productos_para_las_filas_limpias_sin_asignar_categoria() {
        let pool = pool_de_prueba().await;
        let importacion = procesar_archivo(
            &pool,
            &fixture("lista_precios_ok.xlsx"),
            "lista_precios_ok.xlsx".into(),
        )
        .await
        .unwrap();

        // Del fixture: disco, b201, con_decimales y aceite son las únicas
        // producto_valido sin ningún flag de duplicado ni coincidencia.
        let aplicadas = aplicar_pendientes_con_decision(&pool, importacion.id)
            .await
            .unwrap();
        assert_eq!(aplicadas, 4);

        let filas = listar_filas(&pool, importacion.id).await.unwrap();
        let aceite = filas
            .iter()
            .find(|f| f.nombre_excel.as_deref() == Some("aceite 20w50"))
            .unwrap();
        assert!(aceite.producto_id.is_some());
        // El título de sección se conserva en la fila como dato histórico...
        assert_eq!(
            aceite.categoria_excel_texto.as_deref(),
            Some("SECCION ACEITES")
        );

        let producto = productos::obtener(&pool, aceite.producto_id.unwrap())
            .await
            .unwrap();
        assert_eq!(producto.producto.costo_actual, Some(50_000));
        assert_eq!(producto.producto.codigo_legado.as_deref(), Some("E1000"));
        // ...pero nunca se crea ni asigna una categoría sola (ver diseño: un
        // título sin otro que lo cierre puede terminar categorizando miles
        // de productos que no corresponden).
        assert_eq!(producto.producto.categoria_id, None);
        let (cantidad_categorias,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM categorias")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(cantidad_categorias, 0);

        // Las filas problemáticas (duplicados, sin nombre) siguen pendientes.
        let resumen_tras_bulk = resumen(&pool, importacion.id).await.unwrap();
        assert!(resumen_tras_bulk.pendientes > 0);
    }

    #[tokio::test]
    async fn vincular_existente_no_toca_nombre_y_solo_actualiza_costo_si_se_pide() {
        let pool = pool_de_prueba().await;
        let existente = productos::crear(
            &pool,
            GuardarProducto {
                nombre: "Nombre corregido a mano, no se debe tocar".into(),
                marca_id: None,
                categoria_id: None,
                descripcion: None,
                observaciones: None,
                costo_actual: Some(999_999),
                precio_venta_actual: Some(1_500_000),
                precio_publico_referencia: None,
                estado: "activo".into(),
                codigos_fabricante: vec![],
            },
        )
        .await
        .unwrap()
        .producto;

        let importacion = procesar_archivo(
            &pool,
            &fixture("lista_precios_ok.xlsx"),
            "lista_precios_ok.xlsx".into(),
        )
        .await
        .unwrap();
        let filas = listar_filas(&pool, importacion.id).await.unwrap();
        let disco = filas
            .iter()
            .find(|f| f.nombre_excel.as_deref() == Some("disco 5 pieza embrague"))
            .unwrap();

        // Sin pedir actualizar costo: no debe tocar nombre ni costo_actual.
        resolver_fila(
            &pool,
            disco.id,
            ResolverFila {
                decision: DECISION_VINCULAR.into(),
                producto_vinculado_id: Some(existente.id),
                actualizar_costo_en_vinculo: false,
                nombre_corregido: None,
                codigo_corregido: None,
            },
        )
        .await
        .unwrap();

        let releido = productos::obtener(&pool, existente.id).await.unwrap();
        assert_eq!(
            releido.producto.nombre,
            "Nombre corregido a mano, no se debe tocar"
        );
        assert_eq!(releido.producto.costo_actual, Some(999_999));
        // codigo_legado sí se completa porque estaba vacío.
        assert_eq!(releido.producto.codigo_legado.as_deref(), Some("54"));

        let (cantidad_historial,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM precios_historial WHERE producto_id = ? AND tipo = 'costo'",
        )
        .bind(existente.id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(cantidad_historial, 1); // solo la del alta inicial

        // Otra fila, ahora SÍ pidiendo actualizar el costo.
        let b201 = filas
            .iter()
            .find(|f| f.codigo_excel.as_deref() == Some("b201"))
            .unwrap();
        resolver_fila(
            &pool,
            b201.id,
            ResolverFila {
                decision: DECISION_VINCULAR.into(),
                producto_vinculado_id: Some(existente.id),
                actualizar_costo_en_vinculo: true,
                nombre_corregido: None,
                codigo_corregido: None,
            },
        )
        .await
        .unwrap();

        let releido = productos::obtener(&pool, existente.id).await.unwrap();
        assert_eq!(
            releido.producto.nombre,
            "Nombre corregido a mano, no se debe tocar"
        ); // sigue intacto
        assert_eq!(releido.producto.costo_actual, Some(100_700)); // precio de b201
                                                                  // codigo_legado NO se pisa: ya tenía "54" de la vinculación anterior.
        assert_eq!(releido.producto.codigo_legado.as_deref(), Some("54"));

        let (cantidad_historial,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM precios_historial WHERE producto_id = ? AND tipo = 'costo'",
        )
        .bind(existente.id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(cantidad_historial, 2);
    }

    #[tokio::test]
    async fn requiere_revision_se_puede_resolver_corrigiendo_el_nombre() {
        let pool = pool_de_prueba().await;
        let importacion = procesar_archivo(
            &pool,
            &fixture("lista_precios_ok.xlsx"),
            "lista_precios_ok.xlsx".into(),
        )
        .await
        .unwrap();
        let filas = listar_filas(&pool, importacion.id).await.unwrap();
        let sin_nombre = filas
            .iter()
            .find(|f| f.codigo_excel.as_deref() == Some("E2001"))
            .unwrap();
        assert_eq!(sin_nombre.clasificacion, "requiere_revision");

        // Sin corregir el nombre, crear_nuevo debe fallar (no hay con qué).
        let resultado = resolver_fila(
            &pool,
            sin_nombre.id,
            ResolverFila {
                decision: DECISION_CREAR.into(),
                producto_vinculado_id: None,
                actualizar_costo_en_vinculo: false,
                nombre_corregido: None,
                codigo_corregido: None,
            },
        )
        .await;
        assert!(matches!(resultado, Err(AppError::Validation(_))));

        let resuelta = resolver_fila(
            &pool,
            sin_nombre.id,
            ResolverFila {
                decision: DECISION_CREAR.into(),
                producto_vinculado_id: None,
                actualizar_costo_en_vinculo: false,
                nombre_corregido: Some("nombre completado a mano".into()),
                codigo_corregido: None,
            },
        )
        .await
        .expect("resolver con nombre corregido");

        assert_eq!(
            resuelta.nombre_excel.as_deref(),
            Some("nombre completado a mano")
        );
        assert!(resuelta.producto_id.is_some());
        let producto = productos::obtener(&pool, resuelta.producto_id.unwrap())
            .await
            .unwrap();
        assert_eq!(producto.producto.nombre, "nombre completado a mano");
    }

    #[tokio::test]
    async fn omitir_no_crea_producto_y_no_se_puede_resolver_dos_veces() {
        let pool = pool_de_prueba().await;
        let importacion = procesar_archivo(
            &pool,
            &fixture("lista_precios_ok.xlsx"),
            "lista_precios_ok.xlsx".into(),
        )
        .await
        .unwrap();
        let filas = listar_filas(&pool, importacion.id).await.unwrap();
        let guino = filas
            .iter()
            .find(|f| f.nombre_excel.as_deref() == Some("guiño titan"))
            .unwrap();

        let resuelta = resolver_fila(
            &pool,
            guino.id,
            ResolverFila {
                decision: DECISION_OMITIR.into(),
                producto_vinculado_id: None,
                actualizar_costo_en_vinculo: false,
                nombre_corregido: None,
                codigo_corregido: None,
            },
        )
        .await
        .unwrap();
        assert_eq!(resuelta.producto_id, None);
        assert!(resuelta.resuelta_en.is_some());

        let resultado = resolver_fila(
            &pool,
            guino.id,
            ResolverFila {
                decision: DECISION_OMITIR.into(),
                producto_vinculado_id: None,
                actualizar_costo_en_vinculo: false,
                nombre_corregido: None,
                codigo_corregido: None,
            },
        )
        .await;
        assert!(matches!(resultado, Err(AppError::Validation(_))));
    }

    #[tokio::test]
    async fn importacion_pasa_a_confirmada_cuando_se_resuelven_todas_las_pendientes() {
        let pool = pool_de_prueba().await;
        let importacion = procesar_archivo(
            &pool,
            &fixture("lista_precios_ok.xlsx"),
            "lista_precios_ok.xlsx".into(),
        )
        .await
        .unwrap();

        aplicar_pendientes_con_decision(&pool, importacion.id)
            .await
            .unwrap();
        let releida = obtener(&pool, importacion.id).await.unwrap();
        assert_eq!(releida.estado, "en_revision"); // todavía quedan las problemáticas

        let filas = listar_filas(&pool, importacion.id).await.unwrap();
        for fila in filas.iter().filter(|f| {
            f.resuelta_en.is_none()
                && f.clasificacion != "seccion"
                && f.clasificacion != "ignorada"
                && f.clasificacion != "error"
        }) {
            resolver_fila(
                &pool,
                fila.id,
                ResolverFila {
                    decision: DECISION_OMITIR.into(),
                    producto_vinculado_id: None,
                    actualizar_costo_en_vinculo: false,
                    nombre_corregido: None,
                    codigo_corregido: None,
                },
            )
            .await
            .unwrap();
        }

        let releida = obtener(&pool, importacion.id).await.unwrap();
        assert_eq!(releida.estado, "confirmada");
        assert!(releida.cerrada_en.is_some());
    }

    #[tokio::test]
    async fn descartar_marca_la_importacion_sin_tocar_filas() {
        let pool = pool_de_prueba().await;
        let importacion = procesar_archivo(
            &pool,
            &fixture("lista_precios_ok.xlsx"),
            "lista_precios_ok.xlsx".into(),
        )
        .await
        .unwrap();

        let descartada = descartar(&pool, importacion.id).await.unwrap();
        assert_eq!(descartada.estado, "descartada");
        assert!(descartada.cerrada_en.is_some());

        // No se aplicó nada: sigue sin haber productos creados.
        let productos = productos::listar(&pool).await.unwrap();
        assert!(productos.is_empty());
    }
}
