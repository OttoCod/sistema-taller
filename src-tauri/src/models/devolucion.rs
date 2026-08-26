use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Devolucion {
    pub id: i64,
    pub venta_id: i64,
    pub fecha: String,
    pub motivo: String,
    pub metodo_devolucion: String,
    /// Centavos.
    pub total_devuelto: i64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct DevolucionDetalleFila {
    pub id: i64,
    pub venta_detalle_id: i64,
    pub producto_id: i64,
    pub producto_nombre: String,
    pub cantidad: i64,
    /// Centavos.
    pub monto: i64,
    pub estado_producto: String,
    pub observacion: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DevolucionConDetalles {
    #[serde(flatten)]
    pub devolucion: Devolucion,
    pub detalles: Vec<DevolucionDetalleFila>,
}

/// Una línea a devolver. `venta_detalle_id` identifica la línea original
/// de la venta; `cantidad` no puede superar lo que quedaba de esa línea
/// sin devolver todavía (se valida contra devoluciones previas).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemDevolucion {
    pub venta_detalle_id: i64,
    pub cantidad: i64,
    pub estado_producto: String,
    pub observacion: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrearDevolucion {
    pub venta_id: i64,
    pub motivo: String,
    pub metodo_devolucion: String,
    pub items: Vec<ItemDevolucion>,
}
