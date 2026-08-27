use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};
use crate::models::comprobante::{Comprobante, ComprobanteEvento};

const TIPOS_VALIDOS: [&str; 2] = ["ticket", "a4"];
const EVENTOS_VALIDOS: [&str; 2] = ["impreso", "pdf_generado"];

fn prefijo(tipo: &str) -> &'static str {
    match tipo {
        "ticket" => "TCK",
        _ => "A4",
    }
}

const SELECT_COMPROBANTE: &str = "
    SELECT id, venta_id, numero, tipo, creado_en
    FROM comprobantes
";

async fn obtener(pool: &SqlitePool, id: i64) -> AppResult<Comprobante> {
    sqlx::query_as::<_, Comprobante>(&format!("{SELECT_COMPROBANTE} WHERE id = ?"))
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("No existe el comprobante {id}.")))
}

/// Solo lectura: nunca crea nada. Se usa para mostrar "ya se imprimió
/// antes" sin generar un comprobante solo por abrir el detalle de la
/// venta.
pub async fn listar_por_venta(pool: &SqlitePool, venta_id: i64) -> AppResult<Vec<Comprobante>> {
    let comprobantes = sqlx::query_as::<_, Comprobante>(&format!(
        "{SELECT_COMPROBANTE} WHERE venta_id = ? ORDER BY tipo"
    ))
    .bind(venta_id)
    .fetch_all(pool)
    .await?;
    Ok(comprobantes)
}

/// Un solo comprobante por (venta, tipo) -- ver migración 0011. Si ya
/// existe, lo devuelve tal cual (mismo número); si no, lo crea. El número
/// se deriva del id, mismo mecanismo que `ventas.numero` y
/// `productos.codigo_interno`: nunca se reutiliza ni se recalcula.
pub async fn obtener_o_crear(
    pool: &SqlitePool,
    venta_id: i64,
    tipo: &str,
) -> AppResult<Comprobante> {
    if !TIPOS_VALIDOS.contains(&tipo) {
        return Err(AppError::Validation("Tipo de comprobante inválido.".into()));
    }

    let mut tx = pool.begin().await?;

    let venta: Option<(String,)> = sqlx::query_as("SELECT estado FROM ventas WHERE id = ?")
        .bind(venta_id)
        .fetch_optional(&mut *tx)
        .await?;
    let (estado,) =
        venta.ok_or_else(|| AppError::NotFound(format!("No existe la venta {venta_id}.")))?;
    if estado != "confirmada" {
        return Err(AppError::Validation(
            "No se puede generar un comprobante de una venta que no está confirmada.".into(),
        ));
    }

    let existente: Option<(i64,)> =
        sqlx::query_as("SELECT id FROM comprobantes WHERE venta_id = ? AND tipo = ?")
            .bind(venta_id)
            .bind(tipo)
            .fetch_optional(&mut *tx)
            .await?;
    let id = if let Some((id,)) = existente {
        id
    } else {
        let id = sqlx::query("INSERT INTO comprobantes (venta_id, numero, tipo) VALUES (?, '', ?)")
            .bind(venta_id)
            .bind(tipo)
            .execute(&mut *tx)
            .await?
            .last_insert_rowid();
        let numero = format!("{}-{:06}", prefijo(tipo), id);
        sqlx::query("UPDATE comprobantes SET numero = ? WHERE id = ?")
            .bind(&numero)
            .bind(id)
            .execute(&mut *tx)
            .await?;
        sqlx::query(
            "INSERT INTO auditoria (entidad_tipo, entidad_id, accion, detalle_json, usuario_id)
             VALUES ('comprobante', ?, 'comprobante_generado', ?, 1)",
        )
        .bind(id)
        .bind(
            serde_json::json!({ "ventaId": venta_id, "tipo": tipo, "numero": numero }).to_string(),
        )
        .execute(&mut *tx)
        .await?;
        id
    };

    tx.commit().await?;

    obtener(pool, id).await
}

pub async fn registrar_evento(
    pool: &SqlitePool,
    comprobante_id: i64,
    tipo_evento: &str,
) -> AppResult<()> {
    if !EVENTOS_VALIDOS.contains(&tipo_evento) {
        return Err(AppError::Validation(
            "Tipo de evento de comprobante inválido.".into(),
        ));
    }
    // Confirma que existe antes de insertar (mensaje más claro que un
    // error de foreign key crudo).
    obtener(pool, comprobante_id).await?;

    sqlx::query("INSERT INTO comprobante_eventos (comprobante_id, tipo_evento) VALUES (?, ?)")
        .bind(comprobante_id)
        .bind(tipo_evento)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn listar_eventos(
    pool: &SqlitePool,
    comprobante_id: i64,
) -> AppResult<Vec<ComprobanteEvento>> {
    let eventos = sqlx::query_as::<_, ComprobanteEvento>(
        "SELECT id, tipo_evento, fecha FROM comprobante_eventos
         WHERE comprobante_id = ? ORDER BY fecha DESC, id DESC",
    )
    .bind(comprobante_id)
    .fetch_all(pool)
    .await?;
    Ok(eventos)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::producto::GuardarProducto;
    use crate::models::stock::AjusteStock;
    use crate::models::venta::{AnularVenta, CrearVenta, ItemCarrito, PagoInput};
    use crate::services::{productos, stock, ventas};

    async fn pool_de_prueba() -> SqlitePool {
        let dir = tempfile::tempdir().expect("tempdir");
        let dir = Box::leak(Box::new(dir));
        crate::db::init_pool(dir.path()).await.expect("init_pool")
    }

    async fn crear_venta_de_prueba(pool: &SqlitePool) -> i64 {
        let detalle = productos::crear(
            pool,
            GuardarProducto {
                nombre: "Casco".to_string(),
                marca_id: None,
                categoria_id: None,
                descripcion: None,
                observaciones: None,
                costo_actual: Some(500_000),
                precio_venta_actual: Some(1_000_000),
                precio_publico_referencia: None,
                estado: "activo".to_string(),
                codigos_fabricante: vec![],
            },
        )
        .await
        .expect("crear producto");
        let producto_id = detalle.producto.id;
        stock::ajustar(
            pool,
            producto_id,
            AjusteStock {
                nueva_cantidad: 5,
                motivo: "carga inicial".into(),
            },
        )
        .await
        .expect("cargar stock");

        ventas::crear(
            pool,
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
                    monto: 1_000_000,
                }],
            },
        )
        .await
        .unwrap()
        .venta
        .id
    }

    #[tokio::test]
    async fn crear_genera_numero_con_prefijo_segun_tipo() {
        let pool = pool_de_prueba().await;
        let venta_id = crear_venta_de_prueba(&pool).await;

        let ticket = obtener_o_crear(&pool, venta_id, "ticket").await.unwrap();
        assert!(ticket.numero.starts_with("TCK-"));

        let a4 = obtener_o_crear(&pool, venta_id, "a4").await.unwrap();
        assert!(a4.numero.starts_with("A4-"));
        assert_ne!(ticket.id, a4.id);
    }

    #[tokio::test]
    async fn pedirlo_dos_veces_devuelve_el_mismo_comprobante() {
        let pool = pool_de_prueba().await;
        let venta_id = crear_venta_de_prueba(&pool).await;

        let primero = obtener_o_crear(&pool, venta_id, "ticket").await.unwrap();
        let segundo = obtener_o_crear(&pool, venta_id, "ticket").await.unwrap();

        assert_eq!(primero.id, segundo.id);
        assert_eq!(primero.numero, segundo.numero);
    }

    #[tokio::test]
    async fn listar_por_venta_no_crea_nada() {
        let pool = pool_de_prueba().await;
        let venta_id = crear_venta_de_prueba(&pool).await;

        let comprobantes = listar_por_venta(&pool, venta_id).await.unwrap();
        assert!(comprobantes.is_empty());
    }

    #[tokio::test]
    async fn no_se_puede_generar_comprobante_de_venta_anulada() {
        let pool = pool_de_prueba().await;
        let venta_id = crear_venta_de_prueba(&pool).await;
        ventas::anular(
            &pool,
            venta_id,
            AnularVenta {
                motivo: "Error de carga".into(),
            },
        )
        .await
        .unwrap();

        let resultado = obtener_o_crear(&pool, venta_id, "ticket").await;
        assert!(matches!(resultado, Err(AppError::Validation(_))));
    }

    #[tokio::test]
    async fn registrar_evento_y_listar_eventos() {
        let pool = pool_de_prueba().await;
        let venta_id = crear_venta_de_prueba(&pool).await;
        let comprobante = obtener_o_crear(&pool, venta_id, "ticket").await.unwrap();

        registrar_evento(&pool, comprobante.id, "impreso")
            .await
            .unwrap();
        registrar_evento(&pool, comprobante.id, "impreso")
            .await
            .unwrap();

        let eventos = listar_eventos(&pool, comprobante.id).await.unwrap();
        assert_eq!(eventos.len(), 2);
        assert!(eventos.iter().all(|e| e.tipo_evento == "impreso"));
    }

    #[tokio::test]
    async fn tipo_invalido_falla() {
        let pool = pool_de_prueba().await;
        let venta_id = crear_venta_de_prueba(&pool).await;
        let resultado = obtener_o_crear(&pool, venta_id, "pdf").await;
        assert!(matches!(resultado, Err(AppError::Validation(_))));
    }
}
