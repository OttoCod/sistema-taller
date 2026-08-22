use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};
use crate::models::producto_proveedor::{GuardarProductoProveedor, ProductoProveedor};

const SELECT_PRODUCTO_PROVEEDOR: &str = "
    SELECT pp.id, pp.producto_id, p.nombre AS producto_nombre, p.codigo_interno,
           pp.proveedor_id, pp.codigo_proveedor, pp.url_producto, pp.url_busqueda,
           pp.es_principal, pp.activo
    FROM producto_proveedores pp
    JOIN productos p ON p.id = pp.producto_id
";

pub async fn listar_por_proveedor(
    pool: &SqlitePool,
    proveedor_id: i64,
) -> AppResult<Vec<ProductoProveedor>> {
    let vinculos = sqlx::query_as::<_, ProductoProveedor>(&format!(
        "{SELECT_PRODUCTO_PROVEEDOR}
         WHERE pp.proveedor_id = ? AND pp.activo = 1
         ORDER BY p.nombre"
    ))
    .bind(proveedor_id)
    .fetch_all(pool)
    .await?;
    Ok(vinculos)
}

async fn obtener(pool: &SqlitePool, id: i64) -> AppResult<ProductoProveedor> {
    sqlx::query_as::<_, ProductoProveedor>(&format!("{SELECT_PRODUCTO_PROVEEDOR} WHERE pp.id = ?"))
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("No existe el vínculo {id}.")))
}

/// Crea o actualiza el vínculo entre un producto y un proveedor. Como
/// `(producto_id, proveedor_id)` es UNIQUE, "agregar" un producto que ya
/// estaba vinculado (incluso si se lo había quitado antes, lo que solo
/// pone `activo = 0`) actualiza esa misma fila en vez de chocar contra la
/// restricción -- así reactivar un vínculo no crea duplicados.
pub async fn agregar(
    pool: &SqlitePool,
    proveedor_id: i64,
    datos: GuardarProductoProveedor,
) -> AppResult<ProductoProveedor> {
    let producto_existe: Option<(i64,)> = sqlx::query_as("SELECT id FROM productos WHERE id = ?")
        .bind(datos.producto_id)
        .fetch_optional(pool)
        .await?;
    if producto_existe.is_none() {
        return Err(AppError::NotFound(format!(
            "No existe el producto {}.",
            datos.producto_id
        )));
    }

    let existente: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM producto_proveedores WHERE producto_id = ? AND proveedor_id = ?",
    )
    .bind(datos.producto_id)
    .bind(proveedor_id)
    .fetch_optional(pool)
    .await?;

    let id = if let Some((id,)) = existente {
        sqlx::query(
            "UPDATE producto_proveedores
             SET codigo_proveedor = ?, url_producto = ?, url_busqueda = ?, es_principal = ?, activo = 1
             WHERE id = ?",
        )
        .bind(&datos.codigo_proveedor)
        .bind(&datos.url_producto)
        .bind(&datos.url_busqueda)
        .bind(datos.es_principal)
        .bind(id)
        .execute(pool)
        .await?;
        id
    } else {
        sqlx::query(
            "INSERT INTO producto_proveedores
                (producto_id, proveedor_id, codigo_proveedor, url_producto, url_busqueda, es_principal)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(datos.producto_id)
        .bind(proveedor_id)
        .bind(&datos.codigo_proveedor)
        .bind(&datos.url_producto)
        .bind(&datos.url_busqueda)
        .bind(datos.es_principal)
        .execute(pool)
        .await?
        .last_insert_rowid()
    };

    obtener(pool, id).await
}

/// No borra la fila -- la desactiva, mismo criterio que el resto de la
/// app (nunca DELETE). Si se vuelve a agregar el mismo producto para el
/// mismo proveedor, `agregar` reactiva esta misma fila.
pub async fn quitar(pool: &SqlitePool, id: i64) -> AppResult<()> {
    obtener(pool, id).await?;
    sqlx::query("UPDATE producto_proveedores SET activo = 0 WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::producto::GuardarProducto;
    use crate::models::proveedor::GuardarProveedor;
    use crate::services::{productos, proveedores};

    async fn pool_de_prueba() -> SqlitePool {
        let dir = tempfile::tempdir().expect("tempdir");
        let dir = Box::leak(Box::new(dir));
        crate::db::init_pool(dir.path()).await.expect("init_pool")
    }

    async fn crear_proveedor(pool: &SqlitePool, nombre: &str) -> i64 {
        proveedores::crear(
            pool,
            GuardarProveedor {
                nombre: nombre.to_string(),
                telefono: None,
                whatsapp: None,
                email: None,
                sitio_web: None,
                observaciones: None,
                activo: true,
            },
        )
        .await
        .expect("crear proveedor")
        .id
    }

    async fn crear_producto(pool: &SqlitePool, nombre: &str) -> i64 {
        productos::crear(
            pool,
            GuardarProducto {
                nombre: nombre.to_string(),
                marca_id: None,
                categoria_id: None,
                descripcion: None,
                observaciones: None,
                costo_actual: None,
                precio_venta_actual: None,
                precio_publico_referencia: None,
                estado: "activo".to_string(),
                codigos_fabricante: vec![],
            },
        )
        .await
        .expect("crear producto")
        .producto
        .id
    }

    fn vinculo_minimo(producto_id: i64) -> GuardarProductoProveedor {
        GuardarProductoProveedor {
            producto_id,
            codigo_proveedor: None,
            url_producto: None,
            url_busqueda: None,
            es_principal: false,
        }
    }

    #[tokio::test]
    async fn agregar_y_listar_vinculo() {
        let pool = pool_de_prueba().await;
        let proveedor_id = crear_proveedor(&pool, "Repuestos del Sur").await;
        let producto_id = crear_producto(&pool, "Bujía").await;

        let mut datos = vinculo_minimo(producto_id);
        datos.codigo_proveedor = Some("BJ-100".into());
        agregar(&pool, proveedor_id, datos).await.expect("agregar");

        let vinculos = listar_por_proveedor(&pool, proveedor_id).await.unwrap();
        assert_eq!(vinculos.len(), 1);
        assert_eq!(vinculos[0].codigo_proveedor.as_deref(), Some("BJ-100"));
        assert_eq!(vinculos[0].producto_nombre, "Bujía");
    }

    #[tokio::test]
    async fn agregar_de_nuevo_actualiza_en_vez_de_duplicar() {
        let pool = pool_de_prueba().await;
        let proveedor_id = crear_proveedor(&pool, "Proveedor X").await;
        let producto_id = crear_producto(&pool, "Filtro").await;

        agregar(&pool, proveedor_id, vinculo_minimo(producto_id))
            .await
            .unwrap();

        let mut datos = vinculo_minimo(producto_id);
        datos.codigo_proveedor = Some("FL-9".into());
        agregar(&pool, proveedor_id, datos).await.unwrap();

        let vinculos = listar_por_proveedor(&pool, proveedor_id).await.unwrap();
        assert_eq!(vinculos.len(), 1);
        assert_eq!(vinculos[0].codigo_proveedor.as_deref(), Some("FL-9"));
    }

    #[tokio::test]
    async fn quitar_lo_saca_del_listado_y_volver_a_agregar_lo_reactiva() {
        let pool = pool_de_prueba().await;
        let proveedor_id = crear_proveedor(&pool, "Proveedor Y").await;
        let producto_id = crear_producto(&pool, "Cadena").await;

        let vinculo = agregar(&pool, proveedor_id, vinculo_minimo(producto_id))
            .await
            .unwrap();
        quitar(&pool, vinculo.id).await.unwrap();

        let vinculos = listar_por_proveedor(&pool, proveedor_id).await.unwrap();
        assert!(vinculos.is_empty());

        agregar(&pool, proveedor_id, vinculo_minimo(producto_id))
            .await
            .unwrap();
        let vinculos = listar_por_proveedor(&pool, proveedor_id).await.unwrap();
        assert_eq!(vinculos.len(), 1);
    }

    #[tokio::test]
    async fn agregar_con_producto_inexistente_falla() {
        let pool = pool_de_prueba().await;
        let proveedor_id = crear_proveedor(&pool, "Proveedor Z").await;

        let resultado = agregar(&pool, proveedor_id, vinculo_minimo(999)).await;
        assert!(matches!(resultado, Err(AppError::NotFound(_))));
    }
}
