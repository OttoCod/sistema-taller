use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};
use crate::models::proveedor::{GuardarProveedor, Proveedor};

const SELECT_PROVEEDOR: &str = "
    SELECT id, nombre, telefono, whatsapp, email, sitio_web, observaciones, activo
    FROM proveedores
";

/// Lista todos, activos e inactivos -- mismo criterio que
/// `productos::listar`. Quien necesite solo los activos (por ejemplo el
/// selector de proveedor al cargar una compra) filtra del lado del
/// llamador, igual que se hace con `estado` de producto en el carrito de
/// venta.
pub async fn listar(pool: &SqlitePool) -> AppResult<Vec<Proveedor>> {
    let proveedores =
        sqlx::query_as::<_, Proveedor>(&format!("{SELECT_PROVEEDOR} ORDER BY nombre LIMIT 500"))
            .fetch_all(pool)
            .await?;
    Ok(proveedores)
}

/// Búsqueda simple por substring, igual criterio que clientes: no
/// justifica FTS5 para una lista de este tamaño. Tampoco filtra por
/// activo -- ver comentario de `listar`.
pub async fn buscar(pool: &SqlitePool, consulta: &str) -> AppResult<Vec<Proveedor>> {
    let patron = format!("%{}%", consulta.trim());
    let proveedores = sqlx::query_as::<_, Proveedor>(&format!(
        "{SELECT_PROVEEDOR}
         WHERE nombre LIKE ?
         ORDER BY nombre LIMIT 100"
    ))
    .bind(&patron)
    .fetch_all(pool)
    .await?;
    Ok(proveedores)
}

pub async fn obtener(pool: &SqlitePool, id: i64) -> AppResult<Proveedor> {
    sqlx::query_as::<_, Proveedor>(&format!("{SELECT_PROVEEDOR} WHERE id = ?"))
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("No existe el proveedor {id}.")))
}

pub async fn crear(pool: &SqlitePool, datos: GuardarProveedor) -> AppResult<Proveedor> {
    let nombre = datos.nombre.trim();
    if nombre.is_empty() {
        return Err(AppError::Validation(
            "El nombre del proveedor no puede estar vacío.".into(),
        ));
    }

    let id = sqlx::query(
        "INSERT INTO proveedores (nombre, telefono, whatsapp, email, sitio_web, observaciones, activo)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(nombre)
    .bind(&datos.telefono)
    .bind(&datos.whatsapp)
    .bind(&datos.email)
    .bind(&datos.sitio_web)
    .bind(&datos.observaciones)
    .bind(datos.activo)
    .execute(pool)
    .await?
    .last_insert_rowid();

    obtener(pool, id).await
}

pub async fn actualizar(
    pool: &SqlitePool,
    id: i64,
    datos: GuardarProveedor,
) -> AppResult<Proveedor> {
    let nombre = datos.nombre.trim();
    if nombre.is_empty() {
        return Err(AppError::Validation(
            "El nombre del proveedor no puede estar vacío.".into(),
        ));
    }

    // Confirma que existe antes del update (mensaje más claro que un "0
    // filas afectadas" silencioso).
    obtener(pool, id).await?;

    sqlx::query(
        "UPDATE proveedores SET nombre = ?, telefono = ?, whatsapp = ?, email = ?, sitio_web = ?, observaciones = ?, activo = ?
         WHERE id = ?",
    )
    .bind(nombre)
    .bind(&datos.telefono)
    .bind(&datos.whatsapp)
    .bind(&datos.email)
    .bind(&datos.sitio_web)
    .bind(&datos.observaciones)
    .bind(datos.activo)
    .bind(id)
    .execute(pool)
    .await?;

    obtener(pool, id).await
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn pool_de_prueba() -> SqlitePool {
        let dir = tempfile::tempdir().expect("tempdir");
        let dir = Box::leak(Box::new(dir));
        crate::db::init_pool(dir.path()).await.expect("init_pool")
    }

    fn proveedor_minimo(nombre: &str) -> GuardarProveedor {
        GuardarProveedor {
            nombre: nombre.to_string(),
            telefono: None,
            whatsapp: None,
            email: None,
            sitio_web: None,
            observaciones: None,
            activo: true,
        }
    }

    #[tokio::test]
    async fn crear_y_buscar_proveedor() {
        let pool = pool_de_prueba().await;
        let creado = crear(&pool, proveedor_minimo("Repuestos del Sur"))
            .await
            .expect("crear proveedor");
        assert!(creado.activo);

        let encontrados = buscar(&pool, "sur").await.expect("buscar");
        assert_eq!(encontrados.len(), 1);
        assert_eq!(encontrados[0].id, creado.id);
    }

    #[tokio::test]
    async fn nombre_vacio_falla() {
        let pool = pool_de_prueba().await;
        let resultado = crear(&pool, proveedor_minimo("   ")).await;
        assert!(matches!(resultado, Err(AppError::Validation(_))));
    }

    #[tokio::test]
    async fn actualizar_persiste_los_cambios() {
        let pool = pool_de_prueba().await;
        let creado = crear(&pool, proveedor_minimo("Motorepuestos Norte"))
            .await
            .expect("crear");

        let mut datos = proveedor_minimo("Motorepuestos Norte");
        datos.whatsapp = Some("3794123456".into());
        let actualizado = actualizar(&pool, creado.id, datos)
            .await
            .expect("actualizar");

        assert_eq!(actualizado.whatsapp.as_deref(), Some("3794123456"));
    }

    #[tokio::test]
    async fn desactivar_no_lo_saca_de_listar_ni_buscar() {
        let pool = pool_de_prueba().await;
        let creado = crear(&pool, proveedor_minimo("Proveedor a dar de baja"))
            .await
            .expect("crear");

        let mut datos = proveedor_minimo("Proveedor a dar de baja");
        datos.activo = false;
        let actualizado = actualizar(&pool, creado.id, datos)
            .await
            .expect("desactivar");
        assert!(!actualizado.activo);

        let listado = listar(&pool).await.unwrap();
        assert!(listado.iter().any(|p| p.id == creado.id));

        let encontrado = buscar(&pool, "dar de baja").await.unwrap();
        assert!(encontrado.iter().any(|p| p.id == creado.id));
    }
}
