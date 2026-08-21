use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};
use crate::models::cliente::ID_CONSUMIDOR_FINAL;
use crate::models::cuenta_corriente::{
    AjusteCuentaCorriente, MovimientoCuentaCorriente, RegistrarPago,
};

pub async fn listar_movimientos(
    pool: &SqlitePool,
    cliente_id: i64,
) -> AppResult<Vec<MovimientoCuentaCorriente>> {
    let movimientos = sqlx::query_as::<_, MovimientoCuentaCorriente>(
        "SELECT
            cc.id, cc.tipo, cc.monto, cc.saldo_resultante,
            mp.nombre AS metodo_pago_nombre, cc.fecha, cc.observacion
         FROM cuenta_corriente_movimientos cc
         LEFT JOIN metodos_pago mp ON mp.id = cc.metodo_pago_id
         WHERE cc.cliente_id = ?
         ORDER BY cc.fecha DESC, cc.id DESC",
    )
    .bind(cliente_id)
    .fetch_all(pool)
    .await?;
    Ok(movimientos)
}

/// Ambas operaciones (pago y ajuste) comparten esta validación: el cliente
/// tiene que existir y no puede ser "Consumidor final" (punto D -- una
/// cuenta corriente siempre necesita un cliente identificado).
async fn saldo_actual_de_cliente_valido(pool: &SqlitePool, cliente_id: i64) -> AppResult<i64> {
    if cliente_id == ID_CONSUMIDOR_FINAL {
        return Err(AppError::Validation(
            "\"Consumidor final\" no puede tener cuenta corriente.".into(),
        ));
    }
    let saldo: Option<(i64,)> =
        sqlx::query_as("SELECT saldo_cuenta_corriente FROM clientes WHERE id = ?")
            .bind(cliente_id)
            .fetch_optional(pool)
            .await?;
    saldo
        .map(|(s,)| s)
        .ok_or_else(|| AppError::NotFound(format!("No existe el cliente {cliente_id}.")))
}

/// Agrupa los argumentos de `aplicar_movimiento` en vez de pasarlos
/// sueltos (clippy::too_many_arguments).
struct DatosMovimiento<'a> {
    cliente_id: i64,
    tipo: &'a str,
    monto: i64,
    saldo_nuevo: i64,
    metodo_pago_id: Option<i64>,
    observacion: &'a str,
    accion_auditoria: &'a str,
}

async fn aplicar_movimiento(pool: &SqlitePool, datos: DatosMovimiento<'_>) -> AppResult<()> {
    let mut tx = pool.begin().await?;

    sqlx::query("UPDATE clientes SET saldo_cuenta_corriente = ? WHERE id = ?")
        .bind(datos.saldo_nuevo)
        .bind(datos.cliente_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query(
        "INSERT INTO cuenta_corriente_movimientos
            (cliente_id, tipo, monto, saldo_resultante, metodo_pago_id, observacion)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(datos.cliente_id)
    .bind(datos.tipo)
    .bind(datos.monto)
    .bind(datos.saldo_nuevo)
    .bind(datos.metodo_pago_id)
    .bind(datos.observacion)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO auditoria (entidad_tipo, entidad_id, accion, detalle_json, usuario_id)
         VALUES ('cliente', ?, ?, ?, 1)",
    )
    .bind(datos.cliente_id)
    .bind(datos.accion_auditoria)
    .bind(
        serde_json::json!({
            "monto": datos.monto,
            "saldoResultante": datos.saldo_nuevo,
            "observacion": datos.observacion,
        })
        .to_string(),
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

pub async fn registrar_pago(
    pool: &SqlitePool,
    cliente_id: i64,
    datos: RegistrarPago,
) -> AppResult<()> {
    if datos.monto <= 0 {
        return Err(AppError::Validation(
            "El monto del pago tiene que ser mayor a 0.".into(),
        ));
    }

    let metodo_valido: Option<(i64,)> =
        sqlx::query_as("SELECT id FROM metodos_pago WHERE id = ? AND activo = 1")
            .bind(datos.metodo_pago_id)
            .fetch_optional(pool)
            .await?;
    if metodo_valido.is_none() {
        return Err(AppError::Validation("Método de pago inválido.".into()));
    }

    let saldo_anterior = saldo_actual_de_cliente_valido(pool, cliente_id).await?;
    let saldo_nuevo = saldo_anterior - datos.monto;

    aplicar_movimiento(
        pool,
        DatosMovimiento {
            cliente_id,
            tipo: "pago",
            monto: -datos.monto,
            saldo_nuevo,
            metodo_pago_id: Some(datos.metodo_pago_id),
            observacion: datos.observacion.as_deref().unwrap_or(""),
            accion_auditoria: "pago_cuenta_corriente",
        },
    )
    .await
}

pub async fn ajustar(
    pool: &SqlitePool,
    cliente_id: i64,
    datos: AjusteCuentaCorriente,
) -> AppResult<()> {
    let motivo = datos.motivo.trim();
    if motivo.is_empty() {
        return Err(AppError::Validation(
            "Tenés que indicar un motivo para el ajuste.".into(),
        ));
    }
    if datos.monto == 0 {
        return Err(AppError::Validation(
            "El ajuste tiene que ser distinto de 0.".into(),
        ));
    }

    let saldo_anterior = saldo_actual_de_cliente_valido(pool, cliente_id).await?;
    let saldo_nuevo = saldo_anterior + datos.monto;

    aplicar_movimiento(
        pool,
        DatosMovimiento {
            cliente_id,
            tipo: "ajuste",
            monto: datos.monto,
            saldo_nuevo,
            metodo_pago_id: None,
            observacion: motivo,
            accion_auditoria: "ajuste_cuenta_corriente",
        },
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::cliente::GuardarCliente;
    use crate::services::clientes;

    async fn pool_de_prueba() -> SqlitePool {
        let dir = tempfile::tempdir().expect("tempdir");
        let dir = Box::leak(Box::new(dir));
        crate::db::init_pool(dir.path()).await.expect("init_pool")
    }

    async fn crear_cliente(pool: &SqlitePool, nombre: &str) -> i64 {
        clientes::crear(
            pool,
            GuardarCliente {
                nombre: nombre.to_string(),
                telefono: None,
                dni_cuit: None,
                direccion: None,
                observaciones: None,
            },
        )
        .await
        .expect("crear cliente")
        .id
    }

    #[tokio::test]
    async fn ajuste_y_pago_actualizan_saldo_en_secuencia() {
        let pool = pool_de_prueba().await;
        let id = crear_cliente(&pool, "Juan Pérez").await;

        // Carga inicial de deuda migrada desde papel.
        ajustar(
            &pool,
            id,
            AjusteCuentaCorriente {
                monto: 3_500_000,
                motivo: "Saldo migrado desde libreta".into(),
            },
        )
        .await
        .expect("ajuste inicial");

        registrar_pago(
            &pool,
            id,
            RegistrarPago {
                monto: 2_000_000,
                metodo_pago_id: 1,
                observacion: None,
            },
        )
        .await
        .expect("pago");

        let cliente = clientes::obtener(&pool, id).await.unwrap();
        assert_eq!(cliente.saldo_cuenta_corriente, 1_500_000); // $35.000 - $20.000 = $15.000

        let movimientos = listar_movimientos(&pool, id).await.unwrap();
        assert_eq!(movimientos.len(), 2);
        assert_eq!(movimientos[0].tipo, "pago"); // el más reciente primero
        assert_eq!(
            movimientos[0].metodo_pago_nombre.as_deref(),
            Some("efectivo")
        );

        let (auditoria,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM auditoria WHERE entidad_tipo = 'cliente'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(auditoria, 2);
    }

    #[tokio::test]
    async fn consumidor_final_no_puede_tener_cuenta_corriente() {
        let pool = pool_de_prueba().await;
        let resultado = ajustar(
            &pool,
            ID_CONSUMIDOR_FINAL,
            AjusteCuentaCorriente {
                monto: 1000,
                motivo: "prueba".into(),
            },
        )
        .await;
        assert!(matches!(resultado, Err(AppError::Validation(_))));
    }

    #[tokio::test]
    async fn pago_exige_metodo_valido_y_monto_positivo() {
        let pool = pool_de_prueba().await;
        let id = crear_cliente(&pool, "María López").await;

        let sin_monto = registrar_pago(
            &pool,
            id,
            RegistrarPago {
                monto: 0,
                metodo_pago_id: 1,
                observacion: None,
            },
        )
        .await;
        assert!(matches!(sin_monto, Err(AppError::Validation(_))));

        let metodo_invalido = registrar_pago(
            &pool,
            id,
            RegistrarPago {
                monto: 1000,
                metodo_pago_id: 999,
                observacion: None,
            },
        )
        .await;
        assert!(matches!(metodo_invalido, Err(AppError::Validation(_))));
    }

    #[tokio::test]
    async fn cuentas_pendientes_solo_incluye_saldo_positivo() {
        let pool = pool_de_prueba().await;
        let con_deuda = crear_cliente(&pool, "Con deuda").await;
        let _sin_deuda = crear_cliente(&pool, "Sin deuda").await;

        ajustar(
            &pool,
            con_deuda,
            AjusteCuentaCorriente {
                monto: 500_000,
                motivo: "prueba".into(),
            },
        )
        .await
        .unwrap();

        let pendientes = clientes::listar_cuentas_pendientes(&pool).await.unwrap();
        assert_eq!(pendientes.len(), 1);
        assert_eq!(pendientes[0].nombre, "Con deuda");
        assert!(pendientes[0].fecha_ultimo_movimiento.is_some());
    }
}
