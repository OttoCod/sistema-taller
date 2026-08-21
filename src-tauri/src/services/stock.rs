use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};
use crate::models::stock::{AjusteStock, ProductoStock};

const SELECT_PRODUCTO_STOCK: &str = "
    SELECT
        p.id, p.codigo_interno, p.nombre,
        m.nombre AS marca_nombre, c.nombre AS categoria_nombre,
        p.stock_actual, p.stock_minimo,
        CASE
            WHEN p.stock_actual <= 0 THEN 'sin_stock'
            WHEN p.stock_actual <= p.stock_minimo THEN 'bajo'
            ELSE 'ok'
        END AS estado_stock
    FROM productos p
    LEFT JOIN marcas m ON m.id = p.marca_id
    LEFT JOIN categorias c ON c.id = p.categoria_id
    WHERE p.estado = 'activo'
";

pub async fn listar(pool: &SqlitePool) -> AppResult<Vec<ProductoStock>> {
    let productos = sqlx::query_as::<_, ProductoStock>(&format!(
        "{SELECT_PRODUCTO_STOCK} ORDER BY p.nombre LIMIT 500"
    ))
    .fetch_all(pool)
    .await?;
    Ok(productos)
}

pub async fn listar_reposicion(pool: &SqlitePool) -> AppResult<Vec<ProductoStock>> {
    let productos = sqlx::query_as::<_, ProductoStock>(&format!(
        "{SELECT_PRODUCTO_STOCK} AND p.stock_actual <= p.stock_minimo
         ORDER BY (p.stock_actual - p.stock_minimo) ASC, p.nombre
         LIMIT 500"
    ))
    .fetch_all(pool)
    .await?;
    Ok(productos)
}

/// Único movimiento de stock posible en la Fase 4: un ajuste manual con
/// motivo obligatorio. Deja un registro en stock_movimientos (el ledger,
/// fuente de verdad) y en auditoria (sección 23) en la misma transacción
/// que actualiza el cacheado productos.stock_actual.
pub async fn ajustar(pool: &SqlitePool, producto_id: i64, datos: AjusteStock) -> AppResult<()> {
    let motivo = datos.motivo.trim();
    if motivo.is_empty() {
        return Err(AppError::Validation(
            "Tenés que indicar un motivo para el ajuste.".into(),
        ));
    }
    if datos.nueva_cantidad < 0 {
        return Err(AppError::Validation(
            "El stock no puede ser negativo.".into(),
        ));
    }

    let (stock_anterior,): (i64,) =
        sqlx::query_as("SELECT stock_actual FROM productos WHERE id = ?")
            .bind(producto_id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("No existe el producto {producto_id}.")))?;

    let delta = datos.nueva_cantidad - stock_anterior;

    let mut tx = pool.begin().await?;

    sqlx::query("UPDATE productos SET stock_actual = ? WHERE id = ?")
        .bind(datos.nueva_cantidad)
        .bind(producto_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query(
        "INSERT INTO stock_movimientos (producto_id, tipo, cantidad, stock_resultante, observacion)
         VALUES (?, 'ajuste', ?, ?, ?)",
    )
    .bind(producto_id)
    .bind(delta)
    .bind(datos.nueva_cantidad)
    .bind(motivo)
    .execute(&mut *tx)
    .await?;

    let detalle = serde_json::json!({
        "stockAnterior": stock_anterior,
        "stockNuevo": datos.nueva_cantidad,
        "delta": delta,
        "motivo": motivo,
    })
    .to_string();

    sqlx::query(
        "INSERT INTO auditoria (entidad_tipo, entidad_id, accion, detalle_json, usuario_id)
         VALUES ('producto', ?, 'ajuste_stock', ?, 1)",
    )
    .bind(producto_id)
    .bind(detalle)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}

pub async fn actualizar_minimo(
    pool: &SqlitePool,
    producto_id: i64,
    stock_minimo: i64,
) -> AppResult<()> {
    if stock_minimo < 0 {
        return Err(AppError::Validation(
            "El stock mínimo no puede ser negativo.".into(),
        ));
    }

    let resultado = sqlx::query("UPDATE productos SET stock_minimo = ? WHERE id = ?")
        .bind(stock_minimo)
        .bind(producto_id)
        .execute(pool)
        .await?;

    if resultado.rows_affected() == 0 {
        return Err(AppError::NotFound(format!(
            "No existe el producto {producto_id}."
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::producto::GuardarProducto;
    use crate::services::productos;

    async fn pool_de_prueba() -> SqlitePool {
        let dir = tempfile::tempdir().expect("tempdir");
        let dir = Box::leak(Box::new(dir));
        crate::db::init_pool(dir.path()).await.expect("init_pool")
    }

    async fn crear_producto(pool: &SqlitePool, nombre: &str, stock_minimo: i64) -> i64 {
        let detalle = productos::crear(
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
        .expect("crear producto");
        let id = detalle.producto.id;
        actualizar_minimo(pool, id, stock_minimo)
            .await
            .expect("stock minimo");
        id
    }

    #[tokio::test]
    async fn ajustar_actualiza_cacheado_y_deja_ledger_y_auditoria() {
        let pool = pool_de_prueba().await;
        let id = crear_producto(&pool, "Filtro de aire", 5).await;

        ajustar(
            &pool,
            id,
            AjusteStock {
                nueva_cantidad: 20,
                motivo: "Conteo físico inicial".into(),
            },
        )
        .await
        .expect("ajustar");

        let (stock_actual,): (i64,) =
            sqlx::query_as("SELECT stock_actual FROM productos WHERE id = ?")
                .bind(id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(stock_actual, 20);

        let (cantidad_movimiento, stock_resultante): (i64, i64) = sqlx::query_as(
            "SELECT cantidad, stock_resultante FROM stock_movimientos WHERE producto_id = ?",
        )
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(cantidad_movimiento, 20); // 20 - 0
        assert_eq!(stock_resultante, 20);

        let (auditoria,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM auditoria WHERE accion = 'ajuste_stock'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(auditoria, 1);
    }

    #[tokio::test]
    async fn ajustar_exige_motivo() {
        let pool = pool_de_prueba().await;
        let id = crear_producto(&pool, "Cadena", 0).await;

        let resultado = ajustar(
            &pool,
            id,
            AjusteStock {
                nueva_cantidad: 5,
                motivo: "   ".into(),
            },
        )
        .await;
        assert!(matches!(resultado, Err(AppError::Validation(_))));
    }

    #[tokio::test]
    async fn reposicion_solo_incluye_productos_bajo_el_minimo() {
        let pool = pool_de_prueba().await;
        let bajo = crear_producto(&pool, "Pastilla de freno", 10).await;
        let ok = crear_producto(&pool, "Casco", 2).await;

        ajustar(
            &pool,
            bajo,
            AjusteStock {
                nueva_cantidad: 3,
                motivo: "Carga inicial".into(),
            },
        )
        .await
        .unwrap();
        ajustar(
            &pool,
            ok,
            AjusteStock {
                nueva_cantidad: 8,
                motivo: "Carga inicial".into(),
            },
        )
        .await
        .unwrap();

        let reposicion = listar_reposicion(&pool).await.unwrap();
        assert_eq!(reposicion.len(), 1);
        assert_eq!(reposicion[0].nombre, "Pastilla de freno");
        assert_eq!(reposicion[0].estado_stock, "bajo");
    }
}
