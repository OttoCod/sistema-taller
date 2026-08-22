use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Compra {
    pub id: i64,
    pub proveedor_id: i64,
    pub proveedor_nombre: String,
    pub numero_factura: Option<String>,
    pub fecha: String,
    pub estado: String,
    /// Centavos.
    pub subtotal: i64,
    pub total: i64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct DetalleCompra {
    pub id: i64,
    pub producto_id: i64,
    pub producto_nombre: String,
    pub codigo_interno: String,
    pub cantidad: i64,
    pub costo_unitario: i64,
    pub subtotal: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompraDetalle {
    #[serde(flatten)]
    pub compra: Compra,
    pub detalles: Vec<DetalleCompra>,
}

/// Una línea de la recepción. El costo es siempre el que carga quien
/// recibe la mercadería -- nunca se propone solo, a diferencia del carrito
/// de venta donde sí se sugiere el precio de referencia (acá no hay
/// "costo de referencia" del producto que tenga sentido proponer).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemCompra {
    pub producto_id: i64,
    pub cantidad: i64,
    pub costo_unitario: i64,
}

/// subtotal/total NO se reciben del cliente: se calculan siempre del lado
/// del servidor a partir de los items, mismo criterio que Ventas.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrearCompra {
    pub proveedor_id: i64,
    pub numero_factura: Option<String>,
    pub items: Vec<ItemCompra>,
}
