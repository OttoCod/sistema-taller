use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};
use crate::models::compra::{Compra, CompraDetalle, CrearCompra, DetalleCompra};
use crate::services::productos::registrar_precio;

const SELECT_COMPRA: &str = "
    SELECT co.id, co.proveedor_id, pr.nombre AS proveedor_nombre,
           co.numero_factura, co.fecha, co.estado, co.subtotal, co.total
    FROM compras co
    JOIN proveedores pr ON pr.id = co.proveedor_id
";

pub async fn listar(pool: &SqlitePool) -> AppResult<Vec<Compra>> {
    let compras = sqlx::query_as::<_, Compra>(&format!(
        "{SELECT_COMPRA} ORDER BY co.fecha DESC, co.id DESC LIMIT 200"
    ))
    .fetch_all(pool)
    .await?;
    Ok(compras)
}

async fn obtener_compra(pool: &SqlitePool, id: i64) -> AppResult<Compra> {
    sqlx::query_as::<_, Compra>(&format!("{SELECT_COMPRA} WHERE co.id = ?"))
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("No existe la compra {id}.")))
}

async fn obtener_detalles(pool: &SqlitePool, compra_id: i64) -> AppResult<Vec<DetalleCompra>> {
    let detalles = sqlx::query_as::<_, DetalleCompra>(
        "SELECT cd.id, cd.producto_id, p.nombre AS producto_nombre, p.codigo_interno,
                cd.cantidad, cd.costo_unitario, cd.subtotal
         FROM compra_detalles cd
         JOIN productos p ON p.id = cd.producto_id
         WHERE cd.compra_id = ?
         ORDER BY cd.id",
    )
    .bind(compra_id)
    .fetch_all(pool)
    .await?;
    Ok(detalles)
}

pub async fn obtener(pool: &SqlitePool, id: i64) -> AppResult<CompraDetalle> {
    let compra = obtener_compra(pool, id).await?;
    let detalles = obtener_detalles(pool, id).await?;
    Ok(CompraDetalle { compra, detalles })
}

/// Crea la recepción completa en una única transacción: detalles, suma de
/// stock (con su propio movimiento en el ledger) y, si el costo pagado
/// difiere del costo_actual del producto, la actualización de ese
/// cacheado más su fila en precios_historial. subtotal/total se
/// recalculan acá, nunca se confían del frontend (mismo criterio que
/// Ventas).
pub async fn crear(pool: &SqlitePool, datos: CrearCompra) -> AppResult<CompraDetalle> {
    if datos.items.is_empty() {
        return Err(AppError::Validation(
            "La recepción no tiene productos cargados.".into(),
        ));
    }
    for item in &datos.items {
        if item.cantidad <= 0 {
            return Err(AppError::Validation(
                "La cantidad tiene que ser mayor a 0.".into(),
            ));
        }
        if item.costo_unitario < 0 {
            return Err(AppError::Validation(
                "El costo no puede ser negativo.".into(),
            ));
        }
    }

    let mut tx = pool.begin().await?;

    let proveedor: Option<(i64,)> =
        sqlx::query_as("SELECT id FROM proveedores WHERE id = ? AND activo = 1")
            .bind(datos.proveedor_id)
            .fetch_optional(&mut *tx)
            .await?;
    if proveedor.is_none() {
        return Err(AppError::Validation(
            "El proveedor seleccionado no existe o está inactivo.".into(),
        ));
    }

    // Valida que todos los productos existan antes de escribir nada.
    for item in &datos.items {
        let existe: Option<(i64,)> = sqlx::query_as("SELECT id FROM productos WHERE id = ?")
            .bind(item.producto_id)
            .fetch_optional(&mut *tx)
            .await?;
        if existe.is_none() {
            return Err(AppError::NotFound(format!(
                "No existe el producto {}.",
                item.producto_id
            )));
        }
    }

    let total: i64 = datos
        .items
        .iter()
        .map(|item| item.costo_unitario * item.cantidad)
        .sum();

    let id = sqlx::query(
        "INSERT INTO compras (proveedor_id, numero_factura, subtotal, total)
         VALUES (?, ?, ?, ?)",
    )
    .bind(datos.proveedor_id)
    .bind(&datos.numero_factura)
    .bind(total)
    .bind(total)
    .execute(&mut *tx)
    .await?
    .last_insert_rowid();

    for item in &datos.items {
        let subtotal_linea = item.costo_unitario * item.cantidad;
        sqlx::query(
            "INSERT INTO compra_detalles (compra_id, producto_id, cantidad, costo_unitario, subtotal)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(item.producto_id)
        .bind(item.cantidad)
        .bind(item.costo_unitario)
        .bind(subtotal_linea)
        .execute(&mut *tx)
        .await?;

        let (stock_actual, costo_actual): (i64, Option<i64>) =
            sqlx::query_as("SELECT stock_actual, costo_actual FROM productos WHERE id = ?")
                .bind(item.producto_id)
                .fetch_one(&mut *tx)
                .await?;

        let stock_nuevo = stock_actual + item.cantidad;
        sqlx::query("UPDATE productos SET stock_actual = ? WHERE id = ?")
            .bind(stock_nuevo)
            .bind(item.producto_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query(
            "INSERT INTO stock_movimientos
                (producto_id, tipo, cantidad, stock_resultante, referencia_tipo, referencia_id)
             VALUES (?, 'compra', ?, ?, 'compra', ?)",
        )
        .bind(item.producto_id)
        .bind(item.cantidad)
        .bind(stock_nuevo)
        .bind(id)
        .execute(&mut *tx)
        .await?;

        if costo_actual != Some(item.costo_unitario) {
            sqlx::query(
                "UPDATE productos SET costo_actual = ?, precio_actualizado_en = strftime('%Y-%m-%dT%H:%M:%fZ','now')
                 WHERE id = ?",
            )
            .bind(item.costo_unitario)
            .bind(item.producto_id)
            .execute(&mut *tx)
            .await?;
            registrar_precio(&mut tx, item.producto_id, "costo", item.costo_unitario).await?;
        }
    }

    sqlx::query(
        "INSERT INTO auditoria (entidad_tipo, entidad_id, accion, detalle_json, usuario_id)
         VALUES ('compra', ?, 'compra_creada', ?, 1)",
    )
    .bind(id)
    .bind(serde_json::json!({ "total": total, "proveedorId": datos.proveedor_id }).to_string())
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    obtener(pool, id).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::compra::ItemCompra;
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

    async fn crear_producto(pool: &SqlitePool, nombre: &str, costo: Option<i64>) -> i64 {
        productos::crear(
            pool,
            GuardarProducto {
                nombre: nombre.to_string(),
                marca_id: None,
                categoria_id: None,
                descripcion: None,
                observaciones: None,
                costo_actual: costo,
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

    #[tokio::test]
    async fn crear_compra_simple_suma_stock_y_total() {
        let pool = pool_de_prueba().await;
        let proveedor_id = crear_proveedor(&pool, "Repuestos del Sur").await;
        let producto_id = crear_producto(&pool, "Bujía", None).await;

        let compra = crear(
            &pool,
            CrearCompra {
                proveedor_id,
                numero_factura: Some("A-0001".into()),
                items: vec![ItemCompra {
                    producto_id,
                    cantidad: 10,
                    costo_unitario: 300_000,
                }],
            },
        )
        .await
        .expect("crear compra");

        assert_eq!(compra.compra.total, 3_000_000);
        assert_eq!(compra.compra.proveedor_nombre, "Repuestos del Sur");
        assert_eq!(compra.detalles.len(), 1);

        let (stock_actual,): (i64,) =
            sqlx::query_as("SELECT stock_actual FROM productos WHERE id = ?")
                .bind(producto_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(stock_actual, 10);

        let (tipo, cantidad): (String, i64) = sqlx::query_as(
            "SELECT tipo, cantidad FROM stock_movimientos WHERE producto_id = ? ORDER BY id DESC LIMIT 1",
        )
        .bind(producto_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(tipo, "compra");
        assert_eq!(cantidad, 10);
    }

    #[tokio::test]
    async fn costo_distinto_actualiza_producto_y_deja_historial() {
        let pool = pool_de_prueba().await;
        let proveedor_id = crear_proveedor(&pool, "Motorepuestos Norte").await;
        let producto_id = crear_producto(&pool, "Filtro de aceite", Some(200_000)).await;

        crear(
            &pool,
            CrearCompra {
                proveedor_id,
                numero_factura: None,
                items: vec![ItemCompra {
                    producto_id,
                    cantidad: 5,
                    costo_unitario: 250_000,
                }],
            },
        )
        .await
        .expect("crear compra");

        let producto = productos::obtener(&pool, producto_id).await.unwrap();
        assert_eq!(producto.producto.costo_actual, Some(250_000));

        let (cantidad_historial,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM precios_historial WHERE producto_id = ? AND tipo = 'costo'",
        )
        .bind(producto_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        // Una fila del alta inicial (costo 200_000) + una de la compra.
        assert_eq!(cantidad_historial, 2);
    }

    #[tokio::test]
    async fn costo_igual_no_agrega_historial() {
        let pool = pool_de_prueba().await;
        let proveedor_id = crear_proveedor(&pool, "Proveedor X").await;
        let producto_id = crear_producto(&pool, "Cadena", Some(400_000)).await;

        crear(
            &pool,
            CrearCompra {
                proveedor_id,
                numero_factura: None,
                items: vec![ItemCompra {
                    producto_id,
                    cantidad: 3,
                    costo_unitario: 400_000,
                }],
            },
        )
        .await
        .expect("crear compra");

        let (cantidad_historial,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM precios_historial WHERE producto_id = ? AND tipo = 'costo'",
        )
        .bind(producto_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(cantidad_historial, 1); // solo la del alta
    }

    #[tokio::test]
    async fn items_vacios_falla() {
        let pool = pool_de_prueba().await;
        let proveedor_id = crear_proveedor(&pool, "Proveedor Y").await;

        let resultado = crear(
            &pool,
            CrearCompra {
                proveedor_id,
                numero_factura: None,
                items: vec![],
            },
        )
        .await;
        assert!(matches!(resultado, Err(AppError::Validation(_))));
    }

    #[tokio::test]
    async fn proveedor_inexistente_falla_y_no_deja_rastro() {
        let pool = pool_de_prueba().await;
        let producto_id = crear_producto(&pool, "Casco", None).await;

        let resultado = crear(
            &pool,
            CrearCompra {
                proveedor_id: 999,
                numero_factura: None,
                items: vec![ItemCompra {
                    producto_id,
                    cantidad: 1,
                    costo_unitario: 100_000,
                }],
            },
        )
        .await;
        assert!(matches!(resultado, Err(AppError::Validation(_))));

        let (cantidad_compras,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM compras")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(cantidad_compras, 0);
    }

    #[tokio::test]
    async fn recibir_stock_de_producto_inactivo_funciona() {
        let pool = pool_de_prueba().await;
        let proveedor_id = crear_proveedor(&pool, "Proveedor Z").await;
        let producto_id = crear_producto(&pool, "Descontinuado", None).await;
        productos::actualizar(
            &pool,
            producto_id,
            GuardarProducto {
                nombre: "Descontinuado".into(),
                marca_id: None,
                categoria_id: None,
                descripcion: None,
                observaciones: None,
                costo_actual: None,
                precio_venta_actual: None,
                precio_publico_referencia: None,
                estado: "inactivo".into(),
                codigos_fabricante: vec![],
            },
        )
        .await
        .unwrap();

        let compra = crear(
            &pool,
            CrearCompra {
                proveedor_id,
                numero_factura: None,
                items: vec![ItemCompra {
                    producto_id,
                    cantidad: 4,
                    costo_unitario: 150_000,
                }],
            },
        )
        .await
        .expect("recibir stock de producto inactivo debe funcionar");
        assert_eq!(compra.compra.total, 600_000);
    }
}
