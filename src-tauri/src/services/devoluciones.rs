use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};
use crate::models::cliente::ID_CONSUMIDOR_FINAL;
use crate::models::devolucion::{
    CrearDevolucion, Devolucion, DevolucionConDetalles, DevolucionDetalleFila,
};

const METODOS_VALIDOS: [&str; 4] = [
    "reembolso_efectivo",
    "nota_credito",
    "cambio_producto",
    "reduccion_deuda",
];
const ESTADOS_PRODUCTO_VALIDOS: [&str; 4] =
    ["vuelve_a_stock", "en_revision", "defectuoso", "dañado"];

const SELECT_DEVOLUCION: &str = "
    SELECT id, venta_id, fecha, motivo, metodo_devolucion, total_devuelto
    FROM devoluciones
";

async fn obtener_devolucion(pool: &SqlitePool, id: i64) -> AppResult<Devolucion> {
    sqlx::query_as::<_, Devolucion>(&format!("{SELECT_DEVOLUCION} WHERE id = ?"))
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("No existe la devolución {id}.")))
}

async fn obtener_detalles(
    pool: &SqlitePool,
    devolucion_id: i64,
) -> AppResult<Vec<DevolucionDetalleFila>> {
    let detalles = sqlx::query_as::<_, DevolucionDetalleFila>(
        "SELECT dd.id, dd.venta_detalle_id, vd.producto_id, p.nombre AS producto_nombre,
                dd.cantidad, dd.monto, dd.estado_producto, dd.observacion
         FROM devolucion_detalles dd
         JOIN venta_detalles vd ON vd.id = dd.venta_detalle_id
         JOIN productos p ON p.id = vd.producto_id
         WHERE dd.devolucion_id = ?
         ORDER BY dd.id",
    )
    .bind(devolucion_id)
    .fetch_all(pool)
    .await?;
    Ok(detalles)
}

pub async fn obtener(pool: &SqlitePool, id: i64) -> AppResult<DevolucionConDetalles> {
    let devolucion = obtener_devolucion(pool, id).await?;
    let detalles = obtener_detalles(pool, id).await?;
    Ok(DevolucionConDetalles {
        devolucion,
        detalles,
    })
}

pub async fn listar_por_venta(
    pool: &SqlitePool,
    venta_id: i64,
) -> AppResult<Vec<DevolucionConDetalles>> {
    let devoluciones = sqlx::query_as::<_, Devolucion>(&format!(
        "{SELECT_DEVOLUCION} WHERE venta_id = ? ORDER BY id"
    ))
    .bind(venta_id)
    .fetch_all(pool)
    .await?;

    let mut resultado = Vec::with_capacity(devoluciones.len());
    for devolucion in devoluciones {
        let detalles = obtener_detalles(pool, devolucion.id).await?;
        resultado.push(DevolucionConDetalles {
            devolucion,
            detalles,
        });
    }
    Ok(resultado)
}

struct LineaAResolver {
    venta_detalle_id: i64,
    producto_id: i64,
    cantidad: i64,
    monto: i64,
    estado_producto: String,
    observacion: Option<String>,
}

/// Registra una devolución parcial o total de una venta ya confirmada.
/// Nunca se registra sobre una venta anulada (esa venta ya se consideró
/// un error entero, no algo que el cliente "trajo de vuelta" -- punto E).
///
/// El monto de cada línea es proporcional al importe efectivo que quedó
/// en `venta_detalles.subtotal` (ya con su descuento aplicado), no un
/// precio recalculado de cero. La cantidad devuelta nunca puede superar
/// lo que quedaba de esa línea sin devolver, sumando devoluciones previas.
pub async fn crear(pool: &SqlitePool, datos: CrearDevolucion) -> AppResult<DevolucionConDetalles> {
    let motivo = datos.motivo.trim();
    if motivo.is_empty() {
        return Err(AppError::Validation(
            "Tenés que indicar un motivo para la devolución.".into(),
        ));
    }
    if !METODOS_VALIDOS.contains(&datos.metodo_devolucion.as_str()) {
        return Err(AppError::Validation(
            "Método de devolución inválido.".into(),
        ));
    }
    if datos.items.is_empty() {
        return Err(AppError::Validation(
            "Tenés que indicar al menos un producto a devolver.".into(),
        ));
    }
    for item in &datos.items {
        if item.cantidad <= 0 {
            return Err(AppError::Validation(
                "La cantidad a devolver tiene que ser mayor a 0.".into(),
            ));
        }
        if !ESTADOS_PRODUCTO_VALIDOS.contains(&item.estado_producto.as_str()) {
            return Err(AppError::Validation(
                "Estado del producto devuelto inválido.".into(),
            ));
        }
    }

    let mut tx = pool.begin().await?;

    let venta: Option<(String, i64)> =
        sqlx::query_as("SELECT estado, cliente_id FROM ventas WHERE id = ?")
            .bind(datos.venta_id)
            .fetch_optional(&mut *tx)
            .await?;
    let (estado_venta, cliente_id) = venta
        .ok_or_else(|| AppError::NotFound(format!("No existe la venta {}.", datos.venta_id)))?;
    if estado_venta != "confirmada" {
        return Err(AppError::Validation(
            "No se puede registrar una devolución sobre una venta que no está confirmada.".into(),
        ));
    }

    if datos.metodo_devolucion == "reduccion_deuda" && cliente_id == ID_CONSUMIDOR_FINAL {
        return Err(AppError::Validation(
            "\"Consumidor final\" no tiene cuenta corriente: elegí otro método de devolución."
                .into(),
        ));
    }

    let mut lineas = Vec::with_capacity(datos.items.len());
    let mut total_devuelto: i64 = 0;
    for item in &datos.items {
        let detalle: Option<(i64, i64, i64, i64)> = sqlx::query_as(
            "SELECT producto_id, cantidad, subtotal, venta_id FROM venta_detalles WHERE id = ?",
        )
        .bind(item.venta_detalle_id)
        .fetch_optional(&mut *tx)
        .await?;
        let (producto_id, cantidad_original, subtotal, venta_id_de_la_linea) =
            detalle.ok_or_else(|| {
                AppError::NotFound(format!(
                    "No existe la línea de venta {}.",
                    item.venta_detalle_id
                ))
            })?;
        if venta_id_de_la_linea != datos.venta_id {
            return Err(AppError::Validation(
                "Esa línea no pertenece a la venta indicada.".into(),
            ));
        }

        let (ya_devuelta,): (Option<i64>,) = sqlx::query_as(
            "SELECT SUM(cantidad) FROM devolucion_detalles WHERE venta_detalle_id = ?",
        )
        .bind(item.venta_detalle_id)
        .fetch_one(&mut *tx)
        .await?;
        let ya_devuelta = ya_devuelta.unwrap_or(0);
        let disponible = cantidad_original - ya_devuelta;
        if item.cantidad > disponible {
            return Err(AppError::Validation(format!(
                "No se pueden devolver {} unidades: solo quedan {} sin devolver de esa línea.",
                item.cantidad, disponible
            )));
        }

        let monto_linea = subtotal * item.cantidad / cantidad_original;
        total_devuelto += monto_linea;
        lineas.push(LineaAResolver {
            venta_detalle_id: item.venta_detalle_id,
            producto_id,
            cantidad: item.cantidad,
            monto: monto_linea,
            estado_producto: item.estado_producto.clone(),
            observacion: item.observacion.clone(),
        });
    }

    let devolucion_id = sqlx::query(
        "INSERT INTO devoluciones (venta_id, motivo, metodo_devolucion, total_devuelto)
         VALUES (?, ?, ?, ?)",
    )
    .bind(datos.venta_id)
    .bind(motivo)
    .bind(&datos.metodo_devolucion)
    .bind(total_devuelto)
    .execute(&mut *tx)
    .await?
    .last_insert_rowid();

    for linea in &lineas {
        sqlx::query(
            "INSERT INTO devolucion_detalles
                (devolucion_id, venta_detalle_id, cantidad, monto, estado_producto, observacion)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(devolucion_id)
        .bind(linea.venta_detalle_id)
        .bind(linea.cantidad)
        .bind(linea.monto)
        .bind(&linea.estado_producto)
        .bind(&linea.observacion)
        .execute(&mut *tx)
        .await?;

        if linea.estado_producto == "vuelve_a_stock" {
            let (stock_actual,): (i64,) =
                sqlx::query_as("SELECT stock_actual FROM productos WHERE id = ?")
                    .bind(linea.producto_id)
                    .fetch_one(&mut *tx)
                    .await?;
            let stock_nuevo = stock_actual + linea.cantidad;
            sqlx::query("UPDATE productos SET stock_actual = ? WHERE id = ?")
                .bind(stock_nuevo)
                .bind(linea.producto_id)
                .execute(&mut *tx)
                .await?;
            sqlx::query(
                "INSERT INTO stock_movimientos
                    (producto_id, tipo, cantidad, stock_resultante, referencia_tipo, referencia_id)
                 VALUES (?, 'devolucion', ?, ?, 'devolucion', ?)",
            )
            .bind(linea.producto_id)
            .bind(linea.cantidad)
            .bind(stock_nuevo)
            .bind(devolucion_id)
            .execute(&mut *tx)
            .await?;
        }
    }

    if datos.metodo_devolucion == "reduccion_deuda" && total_devuelto > 0 {
        let (saldo_actual,): (i64,) =
            sqlx::query_as("SELECT saldo_cuenta_corriente FROM clientes WHERE id = ?")
                .bind(cliente_id)
                .fetch_one(&mut *tx)
                .await?;
        let saldo_nuevo = saldo_actual - total_devuelto;
        sqlx::query("UPDATE clientes SET saldo_cuenta_corriente = ? WHERE id = ?")
            .bind(saldo_nuevo)
            .bind(cliente_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query(
            "INSERT INTO cuenta_corriente_movimientos
                (cliente_id, tipo, monto, saldo_resultante, referencia_tipo, referencia_id, observacion)
             VALUES (?, 'devolucion', ?, ?, 'devolucion', ?, ?)",
        )
        .bind(cliente_id)
        .bind(-total_devuelto)
        .bind(saldo_nuevo)
        .bind(devolucion_id)
        .bind(motivo)
        .execute(&mut *tx)
        .await?;
    }

    sqlx::query(
        "INSERT INTO auditoria (entidad_tipo, entidad_id, accion, detalle_json, usuario_id)
         VALUES ('devolucion', ?, 'devolucion_creada', ?, 1)",
    )
    .bind(devolucion_id)
    .bind(
        serde_json::json!({
            "ventaId": datos.venta_id,
            "totalDevuelto": total_devuelto,
            "metodoDevolucion": datos.metodo_devolucion,
        })
        .to_string(),
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    obtener(pool, devolucion_id).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::cliente::GuardarCliente;
    use crate::models::devolucion::ItemDevolucion;
    use crate::models::producto::GuardarProducto;
    use crate::models::stock::AjusteStock;
    use crate::models::venta::{CrearVenta, ItemCarrito, PagoInput};
    use crate::services::{clientes, productos, stock, ventas};

    async fn pool_de_prueba() -> SqlitePool {
        let dir = tempfile::tempdir().expect("tempdir");
        let dir = Box::leak(Box::new(dir));
        crate::db::init_pool(dir.path()).await.expect("init_pool")
    }

    async fn crear_producto_con_stock(
        pool: &SqlitePool,
        nombre: &str,
        precio: i64,
        stock: i64,
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
                nueva_cantidad: stock,
                motivo: "carga inicial".into(),
            },
        )
        .await
        .expect("cargar stock");
        id
    }

    async fn crear_venta_simple(
        pool: &SqlitePool,
        producto_id: i64,
        cantidad: i64,
        precio: i64,
    ) -> i64 {
        ventas::crear(
            pool,
            CrearVenta {
                cliente_id: None,
                items: vec![ItemCarrito {
                    producto_id,
                    cantidad,
                    precio_unitario: precio,
                    descuento: 0,
                }],
                pagos: vec![PagoInput {
                    metodo_pago_id: 1,
                    monto: precio * cantidad,
                }],
            },
        )
        .await
        .unwrap()
        .venta
        .id
    }

    #[tokio::test]
    async fn devolucion_que_vuelve_a_stock_repone_stock() {
        let pool = pool_de_prueba().await;
        let producto_id = crear_producto_con_stock(&pool, "Casco", 1_000_000, 10).await;
        let venta_id = crear_venta_simple(&pool, producto_id, 3, 1_000_000).await;
        let venta = ventas::obtener(&pool, venta_id).await.unwrap();
        let venta_detalle_id = venta.detalles[0].id;

        let devolucion = crear(
            &pool,
            CrearDevolucion {
                venta_id,
                motivo: "El cliente no lo quería".into(),
                metodo_devolucion: "reembolso_efectivo".into(),
                items: vec![ItemDevolucion {
                    venta_detalle_id,
                    cantidad: 2,
                    estado_producto: "vuelve_a_stock".into(),
                    observacion: None,
                }],
            },
        )
        .await
        .expect("crear devolución");

        assert_eq!(devolucion.devolucion.total_devuelto, 2_000_000);
        assert_eq!(devolucion.detalles.len(), 1);

        let (stock_actual,): (i64,) =
            sqlx::query_as("SELECT stock_actual FROM productos WHERE id = ?")
                .bind(producto_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(stock_actual, 9); // 10 - 3 (venta) + 2 (devolución)
    }

    #[tokio::test]
    async fn devolucion_defectuosa_no_repone_stock() {
        let pool = pool_de_prueba().await;
        let producto_id = crear_producto_con_stock(&pool, "Filtro", 500_000, 10).await;
        let venta_id = crear_venta_simple(&pool, producto_id, 1, 500_000).await;
        let venta = ventas::obtener(&pool, venta_id).await.unwrap();
        let venta_detalle_id = venta.detalles[0].id;

        crear(
            &pool,
            CrearDevolucion {
                venta_id,
                motivo: "Venía roto".into(),
                metodo_devolucion: "reembolso_efectivo".into(),
                items: vec![ItemDevolucion {
                    venta_detalle_id,
                    cantidad: 1,
                    estado_producto: "defectuoso".into(),
                    observacion: Some("Roto de fábrica".into()),
                }],
            },
        )
        .await
        .unwrap();

        let (stock_actual,): (i64,) =
            sqlx::query_as("SELECT stock_actual FROM productos WHERE id = ?")
                .bind(producto_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(stock_actual, 9); // 10 - 1, nunca vuelve
    }

    #[tokio::test]
    async fn no_se_puede_devolver_mas_de_lo_vendido() {
        let pool = pool_de_prueba().await;
        let producto_id = crear_producto_con_stock(&pool, "Bujía", 100_000, 10).await;
        let venta_id = crear_venta_simple(&pool, producto_id, 2, 100_000).await;
        let venta = ventas::obtener(&pool, venta_id).await.unwrap();
        let venta_detalle_id = venta.detalles[0].id;

        let resultado = crear(
            &pool,
            CrearDevolucion {
                venta_id,
                motivo: "prueba".into(),
                metodo_devolucion: "reembolso_efectivo".into(),
                items: vec![ItemDevolucion {
                    venta_detalle_id,
                    cantidad: 3,
                    estado_producto: "vuelve_a_stock".into(),
                    observacion: None,
                }],
            },
        )
        .await;
        assert!(matches!(resultado, Err(AppError::Validation(_))));
    }

    #[tokio::test]
    async fn dos_devoluciones_parciales_respetan_el_acumulado() {
        let pool = pool_de_prueba().await;
        let producto_id = crear_producto_con_stock(&pool, "Cubierta", 300_000, 10).await;
        let venta_id = crear_venta_simple(&pool, producto_id, 5, 300_000).await;
        let venta = ventas::obtener(&pool, venta_id).await.unwrap();
        let venta_detalle_id = venta.detalles[0].id;

        crear(
            &pool,
            CrearDevolucion {
                venta_id,
                motivo: "primera".into(),
                metodo_devolucion: "reembolso_efectivo".into(),
                items: vec![ItemDevolucion {
                    venta_detalle_id,
                    cantidad: 3,
                    estado_producto: "vuelve_a_stock".into(),
                    observacion: None,
                }],
            },
        )
        .await
        .unwrap();

        // Ya se devolvieron 3 de 5: pedir 3 más debe fallar (solo quedan 2).
        let resultado = crear(
            &pool,
            CrearDevolucion {
                venta_id,
                motivo: "segunda".into(),
                metodo_devolucion: "reembolso_efectivo".into(),
                items: vec![ItemDevolucion {
                    venta_detalle_id,
                    cantidad: 3,
                    estado_producto: "vuelve_a_stock".into(),
                    observacion: None,
                }],
            },
        )
        .await;
        assert!(matches!(resultado, Err(AppError::Validation(_))));

        // Pero devolver las 2 restantes sí funciona.
        crear(
            &pool,
            CrearDevolucion {
                venta_id,
                motivo: "segunda, correcta".into(),
                metodo_devolucion: "reembolso_efectivo".into(),
                items: vec![ItemDevolucion {
                    venta_detalle_id,
                    cantidad: 2,
                    estado_producto: "vuelve_a_stock".into(),
                    observacion: None,
                }],
            },
        )
        .await
        .expect("devolver el resto");

        let devoluciones = listar_por_venta(&pool, venta_id).await.unwrap();
        assert_eq!(devoluciones.len(), 2);
    }

    #[tokio::test]
    async fn reduccion_deuda_baja_el_saldo_del_cliente() {
        let pool = pool_de_prueba().await;
        let producto_id = crear_producto_con_stock(&pool, "Batería", 2_000_000, 5).await;
        let cliente_id = clientes::crear(
            &pool,
            GuardarCliente {
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
        let metodo_cc: (i64,) =
            sqlx::query_as("SELECT id FROM metodos_pago WHERE nombre = 'cuenta_corriente'")
                .fetch_one(&pool)
                .await
                .unwrap();
        let venta = ventas::crear(
            &pool,
            CrearVenta {
                cliente_id: Some(cliente_id),
                items: vec![ItemCarrito {
                    producto_id,
                    cantidad: 1,
                    precio_unitario: 2_000_000,
                    descuento: 0,
                }],
                pagos: vec![PagoInput {
                    metodo_pago_id: metodo_cc.0,
                    monto: 2_000_000,
                }],
            },
        )
        .await
        .unwrap();

        crear(
            &pool,
            CrearDevolucion {
                venta_id: venta.venta.id,
                motivo: "No andaba".into(),
                metodo_devolucion: "reduccion_deuda".into(),
                items: vec![ItemDevolucion {
                    venta_detalle_id: venta.detalles[0].id,
                    cantidad: 1,
                    estado_producto: "defectuoso".into(),
                    observacion: None,
                }],
            },
        )
        .await
        .expect("devolución con reducción de deuda");

        let cliente = clientes::obtener(&pool, cliente_id).await.unwrap();
        assert_eq!(cliente.saldo_cuenta_corriente, 0);
    }

    #[tokio::test]
    async fn no_se_puede_devolver_sobre_venta_anulada() {
        let pool = pool_de_prueba().await;
        let producto_id = crear_producto_con_stock(&pool, "Espejo", 400_000, 5).await;
        let venta_id = crear_venta_simple(&pool, producto_id, 1, 400_000).await;
        let venta = ventas::obtener(&pool, venta_id).await.unwrap();
        let venta_detalle_id = venta.detalles[0].id;

        ventas::anular(
            &pool,
            venta_id,
            crate::models::venta::AnularVenta {
                motivo: "Error".into(),
            },
        )
        .await
        .unwrap();

        let resultado = crear(
            &pool,
            CrearDevolucion {
                venta_id,
                motivo: "prueba".into(),
                metodo_devolucion: "reembolso_efectivo".into(),
                items: vec![ItemDevolucion {
                    venta_detalle_id,
                    cantidad: 1,
                    estado_producto: "vuelve_a_stock".into(),
                    observacion: None,
                }],
            },
        )
        .await;
        assert!(matches!(resultado, Err(AppError::Validation(_))));
    }

    #[tokio::test]
    async fn no_se_puede_anular_una_venta_con_devoluciones() {
        let pool = pool_de_prueba().await;
        let producto_id = crear_producto_con_stock(&pool, "Espejo", 400_000, 5).await;
        let venta_id = crear_venta_simple(&pool, producto_id, 2, 400_000).await;
        let venta = ventas::obtener(&pool, venta_id).await.unwrap();
        let venta_detalle_id = venta.detalles[0].id;

        crear(
            &pool,
            CrearDevolucion {
                venta_id,
                motivo: "Devolvió uno".into(),
                metodo_devolucion: "reembolso_efectivo".into(),
                items: vec![ItemDevolucion {
                    venta_detalle_id,
                    cantidad: 1,
                    estado_producto: "vuelve_a_stock".into(),
                    observacion: None,
                }],
            },
        )
        .await
        .unwrap();

        let resultado = ventas::anular(
            &pool,
            venta_id,
            crate::models::venta::AnularVenta {
                motivo: "Cambio de opinión".into(),
            },
        )
        .await;
        assert!(matches!(resultado, Err(AppError::Validation(_))));
    }
}
