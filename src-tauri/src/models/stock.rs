use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ProductoStock {
    pub id: i64,
    pub codigo_interno: String,
    pub nombre: String,
    pub marca_nombre: Option<String>,
    pub categoria_nombre: Option<String>,
    pub stock_actual: i64,
    pub stock_minimo: i64,
    /// "sin_stock" | "bajo" | "ok" -- calculado en la consulta, no guardado.
    pub estado_stock: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AjusteStock {
    pub nueva_cantidad: i64,
    pub motivo: String,
}
