use sqlx::SqlitePool;

use crate::error::AppResult;
use crate::models::caja::{MontoPorMetodo, ResumenCaja};

const NOMBRE_CUENTA_CORRIENTE: &str = "cuenta_corriente";

/// Resumen de caja de un día puntual (`fecha` en formato `YYYY-MM-DD`).
/// Se recalcula siempre desde `venta_pagos`, nunca se cachea (ver
/// ESQUEMA_BD.md, punto B): así "lo que dice caja" nunca se puede
/// desincronizar de lo que dicen las ventas.
pub async fn resumen(pool: &SqlitePool, fecha: &str) -> AppResult<ResumenCaja> {
    let por_metodo: Vec<MontoPorMetodo> = sqlx::query_as(
        "SELECT mp.id AS metodo_pago_id, mp.nombre AS metodo_pago_nombre, SUM(vp.monto) AS monto
         FROM venta_pagos vp
         JOIN ventas v ON v.id = vp.venta_id
         JOIN metodos_pago mp ON mp.id = vp.metodo_pago_id
         WHERE v.estado = 'confirmada' AND date(vp.fecha) = date(?)
         GROUP BY mp.id
         ORDER BY mp.orden",
    )
    .bind(fecha)
    .fetch_all(pool)
    .await?;

    let total_cobrado: i64 = por_metodo
        .iter()
        .filter(|m| m.metodo_pago_nombre != NOMBRE_CUENTA_CORRIENTE)
        .map(|m| m.monto)
        .sum();
    let total_fiado: i64 = por_metodo
        .iter()
        .filter(|m| m.metodo_pago_nombre == NOMBRE_CUENTA_CORRIENTE)
        .map(|m| m.monto)
        .sum();

    let (cantidad_ventas,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM ventas WHERE estado = 'confirmada' AND date(fecha) = date(?)",
    )
    .bind(fecha)
    .fetch_one(pool)
    .await?;

    Ok(ResumenCaja {
        fecha: fecha.to_string(),
        por_metodo,
        total_cobrado,
        total_fiado,
        cantidad_ventas,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::producto::GuardarProducto;
    use crate::models::stock::AjusteStock;
    use crate::models::venta::{CrearVenta, ItemCarrito, PagoInput};
    use crate::services::{productos, stock, ventas};

    async fn pool_de_prueba() -> SqlitePool {
        let dir = tempfile::tempdir().expect("tempdir");
        let dir = Box::leak(Box::new(dir));
        crate::db::init_pool(dir.path()).await.expect("init_pool")
    }

    async fn crear_producto_con_stock(
        pool: &SqlitePool,
        nombre: &str,
        precio: i64,
        stock_inicial: i64,
    ) -> i64 {
        let detalle = productos::crear(
            pool,
            GuardarProducto {
                nombre: nombre.to_string(),
                marca_id: None,
                categoria_id: None,
                descripcion: None,
                observaciones: None,
                costo_actual: Some(precio / 2),
                precio_venta_actual: Some(precio),
                precio_publico_referencia: None,
                estado: "activo".to_string(),
                codigos_fabricante: vec![],
            },
        )
        .await
        .expect("crear producto");
        let id = detalle.producto.id;
        stock::ajustar(
            pool,
            id,
            AjusteStock {
                nueva_cantidad: stock_inicial,
                motivo: "carga inicial".into(),
            },
        )
        .await
        .expect("cargar stock");
        id
    }

    fn hoy() -> String {
        chrono::Utc::now().format("%Y-%m-%d").to_string()
    }

    #[tokio::test]
    async fn agrupa_pagos_por_metodo_y_excluye_cuenta_corriente_del_cobrado() {
        let pool = pool_de_prueba().await;
        let producto_id = crear_producto_con_stock(&pool, "Bujía", 1_000_000, 10).await;

        // Venta pagada en efectivo.
        ventas::crear(
            &pool,
            CrearVenta {
                cliente_id: None,
                items: vec![ItemCarrito {
                    producto_id,
                    cantidad: 1,
                    precio_unitario: 1_000_000,
                    descuento: 0,
                }],
                pagos: vec![PagoInput {
                    metodo_pago_id: 1, // efectivo
                    monto: 1_000_000,
                }],
            },
        )
        .await
        .expect("venta en efectivo");

        // Venta pagada por transferencia.
        ventas::crear(
            &pool,
            CrearVenta {
                cliente_id: None,
                items: vec![ItemCarrito {
                    producto_id,
                    cantidad: 1,
                    precio_unitario: 1_000_000,
                    descuento: 0,
                }],
                pagos: vec![PagoInput {
                    metodo_pago_id: 2, // transferencia
                    monto: 1_000_000,
                }],
            },
        )
        .await
        .expect("venta por transferencia");

        // Venta fiada (cuenta corriente) -- necesita un cliente real.
        let cliente_id = crate::services::clientes::crear(
            &pool,
            crate::models::cliente::GuardarCliente {
                nombre: "Cliente fiado".into(),
                telefono: None,
                patente: None,
                direccion: None,
                observaciones: None,
            },
        )
        .await
        .unwrap()
        .id;
        ventas::crear(
            &pool,
            CrearVenta {
                cliente_id: Some(cliente_id),
                items: vec![ItemCarrito {
                    producto_id,
                    cantidad: 1,
                    precio_unitario: 1_000_000,
                    descuento: 0,
                }],
                pagos: vec![PagoInput {
                    metodo_pago_id: 4, // cuenta_corriente
                    monto: 1_000_000,
                }],
            },
        )
        .await
        .expect("venta fiada");

        let hoy = hoy();
        let resumen = resumen(&pool, &hoy).await.unwrap();

        assert_eq!(resumen.cantidad_ventas, 3);
        assert_eq!(resumen.por_metodo.len(), 3);
        assert_eq!(resumen.total_cobrado, 2_000_000); // efectivo + transferencia, sin el fiado
        assert_eq!(resumen.total_fiado, 1_000_000);

        let efectivo = resumen
            .por_metodo
            .iter()
            .find(|m| m.metodo_pago_nombre == "efectivo")
            .unwrap();
        assert_eq!(efectivo.monto, 1_000_000);
    }

    #[tokio::test]
    async fn dia_sin_ventas_devuelve_todo_en_cero() {
        let pool = pool_de_prueba().await;
        let resumen = resumen(&pool, "2000-01-01").await.unwrap();
        assert_eq!(resumen.cantidad_ventas, 0);
        assert!(resumen.por_metodo.is_empty());
        assert_eq!(resumen.total_cobrado, 0);
        assert_eq!(resumen.total_fiado, 0);
    }

    #[tokio::test]
    async fn venta_anulada_no_deberia_contar_pero_todavia_no_hay_anulacion() {
        // Sanity check: el filtro estado = 'confirmada' ya está en la
        // consulta aunque la Fase 11 (anulaciones) todavía no exista --
        // así cuando llegue esa fase, la caja ya la respeta sin tocar
        // este servicio.
        let pool = pool_de_prueba().await;
        let producto_id = crear_producto_con_stock(&pool, "Filtro", 500_000, 5).await;
        ventas::crear(
            &pool,
            CrearVenta {
                cliente_id: None,
                items: vec![ItemCarrito {
                    producto_id,
                    cantidad: 1,
                    precio_unitario: 500_000,
                    descuento: 0,
                }],
                pagos: vec![PagoInput {
                    metodo_pago_id: 1,
                    monto: 500_000,
                }],
            },
        )
        .await
        .unwrap();

        let (estado,): (String,) = sqlx::query_as("SELECT estado FROM ventas LIMIT 1")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(estado, "confirmada");
    }
}
