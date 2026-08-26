use sqlx::{Sqlite, SqlitePool, Transaction};

use crate::error::{AppError, AppResult};
use crate::models::cliente::ID_CONSUMIDOR_FINAL;
use crate::models::venta::{AnularVenta, CrearVenta, DetalleVenta, PagoVenta, Venta, VentaDetalle};

const SELECT_VENTA: &str = "
    SELECT v.id, v.numero, v.cliente_id, c.nombre AS cliente_nombre,
           v.fecha, v.estado, v.subtotal, v.descuento_total, v.total,
           v.motivo_anulacion, v.fecha_anulacion
    FROM ventas v
    JOIN clientes c ON c.id = v.cliente_id
";

pub async fn listar(pool: &SqlitePool) -> AppResult<Vec<Venta>> {
    let ventas = sqlx::query_as::<_, Venta>(&format!(
        "{SELECT_VENTA} ORDER BY v.fecha DESC, v.id DESC LIMIT 200"
    ))
    .fetch_all(pool)
    .await?;
    Ok(ventas)
}

async fn obtener_venta(pool: &SqlitePool, id: i64) -> AppResult<Venta> {
    sqlx::query_as::<_, Venta>(&format!("{SELECT_VENTA} WHERE v.id = ?"))
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("No existe la venta {id}.")))
}

async fn obtener_detalles(pool: &SqlitePool, venta_id: i64) -> AppResult<Vec<DetalleVenta>> {
    let detalles = sqlx::query_as::<_, DetalleVenta>(
        "SELECT vd.id, vd.producto_id, p.nombre AS producto_nombre, p.codigo_interno,
                vd.cantidad, vd.precio_unitario, vd.descuento, vd.subtotal
         FROM venta_detalles vd
         JOIN productos p ON p.id = vd.producto_id
         WHERE vd.venta_id = ?
         ORDER BY vd.id",
    )
    .bind(venta_id)
    .fetch_all(pool)
    .await?;
    Ok(detalles)
}

async fn obtener_pagos(pool: &SqlitePool, venta_id: i64) -> AppResult<Vec<PagoVenta>> {
    let pagos = sqlx::query_as::<_, PagoVenta>(
        "SELECT vp.id, vp.metodo_pago_id, mp.nombre AS metodo_pago_nombre, vp.monto
         FROM venta_pagos vp
         JOIN metodos_pago mp ON mp.id = vp.metodo_pago_id
         WHERE vp.venta_id = ?
         ORDER BY vp.id",
    )
    .bind(venta_id)
    .fetch_all(pool)
    .await?;
    Ok(pagos)
}

pub async fn obtener(pool: &SqlitePool, id: i64) -> AppResult<VentaDetalle> {
    let venta = obtener_venta(pool, id).await?;
    let detalles = obtener_detalles(pool, id).await?;
    let pagos = obtener_pagos(pool, id).await?;
    Ok(VentaDetalle {
        venta,
        detalles,
        pagos,
    })
}

struct ProductoParaVenta {
    estado: String,
    stock_actual: i64,
    costo_actual: Option<i64>,
}

async fn producto_para_venta(
    tx: &mut Transaction<'_, Sqlite>,
    producto_id: i64,
) -> AppResult<ProductoParaVenta> {
    let fila: Option<(String, i64, Option<i64>)> =
        sqlx::query_as("SELECT estado, stock_actual, costo_actual FROM productos WHERE id = ?")
            .bind(producto_id)
            .fetch_optional(&mut **tx)
            .await?;
    let (estado, stock_actual, costo_actual) =
        fila.ok_or_else(|| AppError::NotFound(format!("No existe el producto {producto_id}.")))?;
    Ok(ProductoParaVenta {
        estado,
        stock_actual,
        costo_actual,
    })
}

/// Crea la venta completa en una única transacción: detalles, descuento de
/// stock (con su propio movimiento en el ledger), pagos, y si alguno de
/// los pagos es "cuenta_corriente", el aumento de deuda del cliente. Nada
/// de esto se recibe calculado del frontend -- subtotal/descuento/total se
/// recalculan acá para que no se pueda mandar un total manipulado.
pub async fn crear(pool: &SqlitePool, datos: CrearVenta) -> AppResult<VentaDetalle> {
    if datos.items.is_empty() {
        return Err(AppError::Validation("El carrito está vacío.".into()));
    }
    if datos.pagos.is_empty() {
        return Err(AppError::Validation(
            "Tenés que indicar al menos un método de pago.".into(),
        ));
    }
    for item in &datos.items {
        if item.cantidad <= 0 {
            return Err(AppError::Validation(
                "La cantidad tiene que ser mayor a 0.".into(),
            ));
        }
        if item.precio_unitario < 0 || item.descuento < 0 {
            return Err(AppError::Validation(
                "El precio y el descuento no pueden ser negativos.".into(),
            ));
        }
        if item.descuento > item.precio_unitario * item.cantidad {
            return Err(AppError::Validation(
                "El descuento no puede ser mayor al importe de la línea.".into(),
            ));
        }
    }
    for pago in &datos.pagos {
        if pago.monto <= 0 {
            return Err(AppError::Validation(
                "Cada pago tiene que ser mayor a 0.".into(),
            ));
        }
    }

    let cliente_id = datos.cliente_id.unwrap_or(ID_CONSUMIDOR_FINAL);

    let mut tx = pool.begin().await?;

    let saldo_cliente: Option<(i64,)> =
        sqlx::query_as("SELECT saldo_cuenta_corriente FROM clientes WHERE id = ?")
            .bind(cliente_id)
            .fetch_optional(&mut *tx)
            .await?;
    let saldo_cliente = saldo_cliente
        .map(|(s,)| s)
        .ok_or_else(|| AppError::NotFound(format!("No existe el cliente {cliente_id}.")))?;

    // Primera pasada: valida cada producto y arma los totales sin escribir
    // nada todavía.
    let mut productos = Vec::with_capacity(datos.items.len());
    let mut subtotal_general: i64 = 0;
    let mut descuento_general: i64 = 0;
    for item in &datos.items {
        let producto = producto_para_venta(&mut tx, item.producto_id).await?;
        if producto.estado != "activo" {
            return Err(AppError::Validation(format!(
                "El producto {} no está activo.",
                item.producto_id
            )));
        }
        subtotal_general += item.precio_unitario * item.cantidad;
        descuento_general += item.descuento;
        productos.push(producto);
    }
    let total = subtotal_general - descuento_general;

    let mut suma_pagos: i64 = 0;
    let mut monto_cuenta_corriente: i64 = 0;
    for pago in &datos.pagos {
        suma_pagos += pago.monto;
        let metodo: Option<(String,)> =
            sqlx::query_as("SELECT nombre FROM metodos_pago WHERE id = ? AND activo = 1")
                .bind(pago.metodo_pago_id)
                .fetch_optional(&mut *tx)
                .await?;
        let Some((nombre,)) = metodo else {
            return Err(AppError::Validation("Método de pago inválido.".into()));
        };
        if nombre == "cuenta_corriente" {
            monto_cuenta_corriente += pago.monto;
        }
    }
    if suma_pagos != total {
        return Err(AppError::Validation(
            "La suma de los pagos no coincide con el total de la venta.".into(),
        ));
    }
    if monto_cuenta_corriente > 0 && cliente_id == ID_CONSUMIDOR_FINAL {
        return Err(AppError::Validation(
            "Una venta fiada necesita un cliente identificado: \"Consumidor final\" no puede quedar a cuenta corriente.".into(),
        ));
    }

    let id = sqlx::query(
        "INSERT INTO ventas (numero, cliente_id, subtotal, descuento_total, total)
         VALUES (0, ?, ?, ?, ?)",
    )
    .bind(cliente_id)
    .bind(subtotal_general)
    .bind(descuento_general)
    .bind(total)
    .execute(&mut *tx)
    .await?
    .last_insert_rowid();

    // Mismo mecanismo que productos.codigo_interno: el número se deriva
    // del id, nunca se reutiliza.
    sqlx::query("UPDATE ventas SET numero = ? WHERE id = ?")
        .bind(id)
        .bind(id)
        .execute(&mut *tx)
        .await?;

    for (item, producto) in datos.items.iter().zip(productos.iter()) {
        let subtotal_linea = item.precio_unitario * item.cantidad - item.descuento;
        sqlx::query(
            "INSERT INTO venta_detalles
                (venta_id, producto_id, cantidad, precio_unitario, descuento, subtotal, costo_unitario_snapshot)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(item.producto_id)
        .bind(item.cantidad)
        .bind(item.precio_unitario)
        .bind(item.descuento)
        .bind(subtotal_linea)
        .bind(producto.costo_actual)
        .execute(&mut *tx)
        .await?;

        let stock_nuevo = producto.stock_actual - item.cantidad;
        sqlx::query("UPDATE productos SET stock_actual = ? WHERE id = ?")
            .bind(stock_nuevo)
            .bind(item.producto_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query(
            "INSERT INTO stock_movimientos
                (producto_id, tipo, cantidad, stock_resultante, referencia_tipo, referencia_id)
             VALUES (?, 'venta', ?, ?, 'venta', ?)",
        )
        .bind(item.producto_id)
        .bind(-item.cantidad)
        .bind(stock_nuevo)
        .bind(id)
        .execute(&mut *tx)
        .await?;
    }

    for pago in &datos.pagos {
        sqlx::query("INSERT INTO venta_pagos (venta_id, metodo_pago_id, monto) VALUES (?, ?, ?)")
            .bind(id)
            .bind(pago.metodo_pago_id)
            .bind(pago.monto)
            .execute(&mut *tx)
            .await?;
    }

    if monto_cuenta_corriente > 0 {
        let saldo_nuevo = saldo_cliente + monto_cuenta_corriente;
        sqlx::query("UPDATE clientes SET saldo_cuenta_corriente = ? WHERE id = ?")
            .bind(saldo_nuevo)
            .bind(cliente_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query(
            "INSERT INTO cuenta_corriente_movimientos
                (cliente_id, tipo, monto, saldo_resultante, referencia_tipo, referencia_id)
             VALUES (?, 'venta_fiada', ?, ?, 'venta', ?)",
        )
        .bind(cliente_id)
        .bind(monto_cuenta_corriente)
        .bind(saldo_nuevo)
        .bind(id)
        .execute(&mut *tx)
        .await?;
    }

    sqlx::query(
        "INSERT INTO auditoria (entidad_tipo, entidad_id, accion, detalle_json, usuario_id)
         VALUES ('venta', ?, 'venta_creada', ?, 1)",
    )
    .bind(id)
    .bind(serde_json::json!({ "total": total, "clienteId": cliente_id }).to_string())
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    obtener(pool, id).await
}

/// Anula una venta completa: revierte el stock descontado, revierte la
/// deuda de cuenta corriente que haya generado (si pagó con
/// cuenta_corriente) y marca la venta como 'anulada'. Nunca borra la
/// venta ni sus filas -- ventas.numero nunca se reutiliza (punto E).
///
/// Se rechaza si la venta ya está anulada o si ya tiene devoluciones
/// registradas (una devolución asume que la venta fue válida; anularla
/// después mezclaría dos reversiones distintas sobre las mismas líneas).
pub async fn anular(pool: &SqlitePool, id: i64, datos: AnularVenta) -> AppResult<VentaDetalle> {
    let motivo = datos.motivo.trim();
    if motivo.is_empty() {
        return Err(AppError::Validation(
            "Tenés que indicar un motivo para anular la venta.".into(),
        ));
    }

    let mut tx = pool.begin().await?;

    let estado: Option<(String,)> = sqlx::query_as("SELECT estado FROM ventas WHERE id = ?")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;
    let (estado,) =
        estado.ok_or_else(|| AppError::NotFound(format!("No existe la venta {id}.")))?;
    if estado == "anulada" {
        return Err(AppError::Validation("Esta venta ya está anulada.".into()));
    }

    let (tiene_devoluciones,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM devoluciones WHERE venta_id = ?")
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
    if tiene_devoluciones > 0 {
        return Err(AppError::Validation(
            "No se puede anular: esta venta ya tiene devoluciones registradas.".into(),
        ));
    }

    // Revierte el stock de cada línea.
    let detalles: Vec<(i64, i64)> =
        sqlx::query_as("SELECT producto_id, cantidad FROM venta_detalles WHERE venta_id = ?")
            .bind(id)
            .fetch_all(&mut *tx)
            .await?;
    for (producto_id, cantidad) in detalles {
        let (stock_actual,): (i64,) =
            sqlx::query_as("SELECT stock_actual FROM productos WHERE id = ?")
                .bind(producto_id)
                .fetch_one(&mut *tx)
                .await?;
        let stock_nuevo = stock_actual + cantidad;
        sqlx::query("UPDATE productos SET stock_actual = ? WHERE id = ?")
            .bind(stock_nuevo)
            .bind(producto_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query(
            "INSERT INTO stock_movimientos
                (producto_id, tipo, cantidad, stock_resultante, referencia_tipo, referencia_id)
             VALUES (?, 'anulacion', ?, ?, 'venta', ?)",
        )
        .bind(producto_id)
        .bind(cantidad)
        .bind(stock_nuevo)
        .bind(id)
        .execute(&mut *tx)
        .await?;
    }

    // Revierte la deuda de cuenta corriente que haya generado esta venta.
    let monto_cuenta_corriente: (Option<i64>,) = sqlx::query_as(
        "SELECT SUM(vp.monto) FROM venta_pagos vp
         JOIN metodos_pago mp ON mp.id = vp.metodo_pago_id
         WHERE vp.venta_id = ? AND mp.nombre = 'cuenta_corriente'",
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;
    if let Some(monto) = monto_cuenta_corriente.0.filter(|m| *m > 0) {
        let cliente_id: (i64,) = sqlx::query_as("SELECT cliente_id FROM ventas WHERE id = ?")
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
        let (saldo_actual,): (i64,) =
            sqlx::query_as("SELECT saldo_cuenta_corriente FROM clientes WHERE id = ?")
                .bind(cliente_id.0)
                .fetch_one(&mut *tx)
                .await?;
        let saldo_nuevo = saldo_actual - monto;
        sqlx::query("UPDATE clientes SET saldo_cuenta_corriente = ? WHERE id = ?")
            .bind(saldo_nuevo)
            .bind(cliente_id.0)
            .execute(&mut *tx)
            .await?;
        sqlx::query(
            "INSERT INTO cuenta_corriente_movimientos
                (cliente_id, tipo, monto, saldo_resultante, referencia_tipo, referencia_id, observacion)
             VALUES (?, 'devolucion', ?, ?, 'venta', ?, 'Anulación de venta')",
        )
        .bind(cliente_id.0)
        .bind(-monto)
        .bind(saldo_nuevo)
        .bind(id)
        .execute(&mut *tx)
        .await?;
    }

    sqlx::query(
        "UPDATE ventas
         SET estado = 'anulada', motivo_anulacion = ?, fecha_anulacion = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
    )
    .bind(motivo)
    .bind(id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO auditoria (entidad_tipo, entidad_id, accion, detalle_json, usuario_id)
         VALUES ('venta', ?, 'venta_anulada', ?, 1)",
    )
    .bind(id)
    .bind(serde_json::json!({ "motivo": motivo }).to_string())
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    obtener(pool, id).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::cliente::GuardarCliente;
    use crate::models::producto::GuardarProducto;
    use crate::models::stock::AjusteStock;
    use crate::models::venta::{ItemCarrito, PagoInput};
    use crate::services::{clientes, productos, stock};

    async fn pool_de_prueba() -> SqlitePool {
        let dir = tempfile::tempdir().expect("tempdir");
        let dir = Box::leak(Box::new(dir));
        crate::db::init_pool(dir.path()).await.expect("init_pool")
    }

    async fn crear_producto_con_stock(
        pool: &SqlitePool,
        nombre: &str,
        precio_venta: i64,
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
                costo_actual: Some(precio_venta / 2),
                precio_venta_actual: Some(precio_venta),
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

    #[tokio::test]
    async fn crear_venta_simple_actualiza_stock_y_total() {
        let pool = pool_de_prueba().await;
        let producto_id = crear_producto_con_stock(&pool, "Bujía", 800_000, 10).await;

        let venta = crear(
            &pool,
            CrearVenta {
                cliente_id: None,
                items: vec![ItemCarrito {
                    producto_id,
                    cantidad: 2,
                    precio_unitario: 800_000,
                    descuento: 0,
                }],
                pagos: vec![PagoInput {
                    metodo_pago_id: 1,
                    monto: 1_600_000,
                }],
            },
        )
        .await
        .expect("crear venta");

        assert_eq!(venta.venta.total, 1_600_000);
        assert_eq!(venta.venta.numero, venta.venta.id); // mismo mecanismo que codigo_interno
        assert_eq!(venta.venta.cliente_nombre, "Consumidor final");
        assert_eq!(venta.detalles.len(), 1);
        assert_eq!(venta.pagos.len(), 1);

        let (stock_actual,): (i64,) =
            sqlx::query_as("SELECT stock_actual FROM productos WHERE id = ?")
                .bind(producto_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(stock_actual, 8); // 10 - 2

        let (tipo, cantidad): (String, i64) = sqlx::query_as(
            "SELECT tipo, cantidad FROM stock_movimientos WHERE producto_id = ? ORDER BY id DESC LIMIT 1",
        )
        .bind(producto_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(tipo, "venta");
        assert_eq!(cantidad, -2);
    }

    #[tokio::test]
    async fn descuento_por_linea_se_resta_del_total() {
        let pool = pool_de_prueba().await;
        let producto_id = crear_producto_con_stock(&pool, "Casco", 5_000_000, 5).await;

        let venta = crear(
            &pool,
            CrearVenta {
                cliente_id: None,
                items: vec![ItemCarrito {
                    producto_id,
                    cantidad: 1,
                    precio_unitario: 5_000_000,
                    descuento: 500_000,
                }],
                pagos: vec![PagoInput {
                    metodo_pago_id: 1,
                    monto: 4_500_000,
                }],
            },
        )
        .await
        .expect("crear venta");

        assert_eq!(venta.venta.subtotal, 5_000_000);
        assert_eq!(venta.venta.descuento_total, 500_000);
        assert_eq!(venta.venta.total, 4_500_000);
    }

    #[tokio::test]
    async fn pago_dividido_en_varios_metodos() {
        let pool = pool_de_prueba().await;
        let producto_id = crear_producto_con_stock(&pool, "Cubierta", 3_000_000, 5).await;

        let venta = crear(
            &pool,
            CrearVenta {
                cliente_id: None,
                items: vec![ItemCarrito {
                    producto_id,
                    cantidad: 1,
                    precio_unitario: 3_000_000,
                    descuento: 0,
                }],
                pagos: vec![
                    PagoInput {
                        metodo_pago_id: 1,
                        monto: 1_000_000,
                    },
                    PagoInput {
                        metodo_pago_id: 2,
                        monto: 2_000_000,
                    },
                ],
            },
        )
        .await
        .expect("crear venta con pago dividido");

        assert_eq!(venta.pagos.len(), 2);
        let suma: i64 = venta.pagos.iter().map(|p| p.monto).sum();
        assert_eq!(suma, 3_000_000);
    }

    #[tokio::test]
    async fn pagos_que_no_suman_el_total_fallan() {
        let pool = pool_de_prueba().await;
        let producto_id = crear_producto_con_stock(&pool, "Espejo", 1_000_000, 5).await;

        let resultado = crear(
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
                    metodo_pago_id: 1,
                    monto: 900_000,
                }],
            },
        )
        .await;
        assert!(matches!(resultado, Err(AppError::Validation(_))));

        // Y no debe haber quedado ningún rastro (transacción completa).
        let (cantidad_ventas,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM ventas")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(cantidad_ventas, 0);
    }

    #[tokio::test]
    async fn venta_fiada_aumenta_deuda_y_exige_cliente_real() {
        let pool = pool_de_prueba().await;
        let producto_id = crear_producto_con_stock(&pool, "Batería", 4_000_000, 5).await;
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

        // Con Consumidor Final (None) debe fallar.
        let metodo_cc: (i64,) =
            sqlx::query_as("SELECT id FROM metodos_pago WHERE nombre = 'cuenta_corriente'")
                .fetch_one(&pool)
                .await
                .unwrap();
        let resultado = crear(
            &pool,
            CrearVenta {
                cliente_id: None,
                items: vec![ItemCarrito {
                    producto_id,
                    cantidad: 1,
                    precio_unitario: 4_000_000,
                    descuento: 0,
                }],
                pagos: vec![PagoInput {
                    metodo_pago_id: metodo_cc.0,
                    monto: 4_000_000,
                }],
            },
        )
        .await;
        assert!(matches!(resultado, Err(AppError::Validation(_))));

        // Con un cliente real, sí funciona y le aumenta la deuda.
        let venta = crear(
            &pool,
            CrearVenta {
                cliente_id: Some(cliente_id),
                items: vec![ItemCarrito {
                    producto_id,
                    cantidad: 1,
                    precio_unitario: 4_000_000,
                    descuento: 0,
                }],
                pagos: vec![PagoInput {
                    metodo_pago_id: metodo_cc.0,
                    monto: 4_000_000,
                }],
            },
        )
        .await
        .expect("venta fiada");

        assert_eq!(venta.venta.cliente_id, cliente_id);
        let cliente = clientes::obtener(&pool, cliente_id).await.unwrap();
        assert_eq!(cliente.saldo_cuenta_corriente, 4_000_000);

        let movimientos = crate::services::cuenta_corriente::listar_movimientos(&pool, cliente_id)
            .await
            .unwrap();
        assert_eq!(movimientos.len(), 1);
        assert_eq!(movimientos[0].tipo, "venta_fiada");
    }

    #[tokio::test]
    async fn carrito_vacio_falla() {
        let pool = pool_de_prueba().await;
        let resultado = crear(
            &pool,
            CrearVenta {
                cliente_id: None,
                items: vec![],
                pagos: vec![PagoInput {
                    metodo_pago_id: 1,
                    monto: 100,
                }],
            },
        )
        .await;
        assert!(matches!(resultado, Err(AppError::Validation(_))));
    }

    #[tokio::test]
    async fn vender_producto_inactivo_falla() {
        let pool = pool_de_prueba().await;
        let producto_id = crear_producto_con_stock(&pool, "Descontinuado", 100_000, 5).await;
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
                precio_venta_actual: Some(100_000),
                precio_publico_referencia: None,
                estado: "inactivo".into(),
                codigos_fabricante: vec![],
            },
        )
        .await
        .unwrap();

        let resultado = crear(
            &pool,
            CrearVenta {
                cliente_id: None,
                items: vec![ItemCarrito {
                    producto_id,
                    cantidad: 1,
                    precio_unitario: 100_000,
                    descuento: 0,
                }],
                pagos: vec![PagoInput {
                    metodo_pago_id: 1,
                    monto: 100_000,
                }],
            },
        )
        .await;
        assert!(matches!(resultado, Err(AppError::Validation(_))));
    }

    #[tokio::test]
    async fn anular_venta_revierte_stock_y_deja_registro() {
        let pool = pool_de_prueba().await;
        let producto_id = crear_producto_con_stock(&pool, "Casco", 1_000_000, 10).await;

        let venta = crear(
            &pool,
            CrearVenta {
                cliente_id: None,
                items: vec![ItemCarrito {
                    producto_id,
                    cantidad: 3,
                    precio_unitario: 1_000_000,
                    descuento: 0,
                }],
                pagos: vec![PagoInput {
                    metodo_pago_id: 1,
                    monto: 3_000_000,
                }],
            },
        )
        .await
        .unwrap();

        let anulada = anular(
            &pool,
            venta.venta.id,
            AnularVenta {
                motivo: "Cliente se arrepintió".into(),
            },
        )
        .await
        .expect("anular venta");

        assert_eq!(anulada.venta.estado, "anulada");
        assert_eq!(
            anulada.venta.motivo_anulacion.as_deref(),
            Some("Cliente se arrepintió")
        );
        assert!(anulada.venta.fecha_anulacion.is_some());

        let (stock_actual,): (i64,) =
            sqlx::query_as("SELECT stock_actual FROM productos WHERE id = ?")
                .bind(producto_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(stock_actual, 10); // volvió al valor original

        let (tipo, cantidad): (String, i64) = sqlx::query_as(
            "SELECT tipo, cantidad FROM stock_movimientos WHERE producto_id = ? ORDER BY id DESC LIMIT 1",
        )
        .bind(producto_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(tipo, "anulacion");
        assert_eq!(cantidad, 3);
    }

    #[tokio::test]
    async fn anular_venta_fiada_revierte_la_deuda() {
        let pool = pool_de_prueba().await;
        let producto_id = crear_producto_con_stock(&pool, "Batería", 4_000_000, 5).await;
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

        let venta = crear(
            &pool,
            CrearVenta {
                cliente_id: Some(cliente_id),
                items: vec![ItemCarrito {
                    producto_id,
                    cantidad: 1,
                    precio_unitario: 4_000_000,
                    descuento: 0,
                }],
                pagos: vec![PagoInput {
                    metodo_pago_id: metodo_cc.0,
                    monto: 4_000_000,
                }],
            },
        )
        .await
        .unwrap();

        let cliente = clientes::obtener(&pool, cliente_id).await.unwrap();
        assert_eq!(cliente.saldo_cuenta_corriente, 4_000_000);

        anular(
            &pool,
            venta.venta.id,
            AnularVenta {
                motivo: "Error de carga".into(),
            },
        )
        .await
        .expect("anular venta fiada");

        let cliente = clientes::obtener(&pool, cliente_id).await.unwrap();
        assert_eq!(cliente.saldo_cuenta_corriente, 0);
    }

    #[tokio::test]
    async fn no_se_puede_anular_dos_veces() {
        let pool = pool_de_prueba().await;
        let producto_id = crear_producto_con_stock(&pool, "Filtro", 100_000, 5).await;
        let venta = crear(
            &pool,
            CrearVenta {
                cliente_id: None,
                items: vec![ItemCarrito {
                    producto_id,
                    cantidad: 1,
                    precio_unitario: 100_000,
                    descuento: 0,
                }],
                pagos: vec![PagoInput {
                    metodo_pago_id: 1,
                    monto: 100_000,
                }],
            },
        )
        .await
        .unwrap();

        anular(
            &pool,
            venta.venta.id,
            AnularVenta {
                motivo: "primera anulación".into(),
            },
        )
        .await
        .unwrap();

        let resultado = anular(
            &pool,
            venta.venta.id,
            AnularVenta {
                motivo: "segunda anulación".into(),
            },
        )
        .await;
        assert!(matches!(resultado, Err(AppError::Validation(_))));
    }

    #[tokio::test]
    async fn anular_sin_motivo_falla() {
        let pool = pool_de_prueba().await;
        let producto_id = crear_producto_con_stock(&pool, "Bujía", 100_000, 5).await;
        let venta = crear(
            &pool,
            CrearVenta {
                cliente_id: None,
                items: vec![ItemCarrito {
                    producto_id,
                    cantidad: 1,
                    precio_unitario: 100_000,
                    descuento: 0,
                }],
                pagos: vec![PagoInput {
                    metodo_pago_id: 1,
                    monto: 100_000,
                }],
            },
        )
        .await
        .unwrap();

        let resultado = anular(
            &pool,
            venta.venta.id,
            AnularVenta {
                motivo: "   ".into(),
            },
        )
        .await;
        assert!(matches!(resultado, Err(AppError::Validation(_))));
    }
}
