use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Venta {
    pub id: i64,
    pub numero: i64,
    pub cliente_id: i64,
    pub cliente_nombre: String,
    pub fecha: String,
    pub estado: String,
    /// Centavos.
    pub subtotal: i64,
    pub descuento_total: i64,
    pub total: i64,
    pub motivo_anulacion: Option<String>,
    pub fecha_anulacion: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct DetalleVenta {
    pub id: i64,
    pub producto_id: i64,
    pub producto_nombre: String,
    pub codigo_interno: String,
    pub cantidad: i64,
    pub precio_unitario: i64,
    pub descuento: i64,
    pub subtotal: i64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PagoVenta {
    pub id: i64,
    pub metodo_pago_id: i64,
    pub metodo_pago_nombre: String,
    pub monto: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VentaDetalle {
    #[serde(flatten)]
    pub venta: Venta,
    pub detalles: Vec<DetalleVenta>,
    pub pagos: Vec<PagoVenta>,
}

/// Una línea del carrito. El precio y el descuento son siempre los que
/// carga el vendedor -- nunca se recalculan solos (precio de venta siempre
/// editable a mano, confirmado desde el análisis inicial).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemCarrito {
    pub producto_id: i64,
    pub cantidad: i64,
    pub precio_unitario: i64,
    pub descuento: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PagoInput {
    pub metodo_pago_id: i64,
    pub monto: i64,
}

/// subtotal/descuento_total/total NO se reciben del cliente: se calculan
/// siempre del lado del servidor a partir de los items, para que no se
/// pueda mandar un total manipulado.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrearVenta {
    pub cliente_id: Option<i64>,
    pub items: Vec<ItemCarrito>,
    pub pagos: Vec<PagoInput>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnularVenta {
    pub motivo: String,
}
