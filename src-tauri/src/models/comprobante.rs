use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Comprobante {
    pub id: i64,
    pub venta_id: i64,
    pub numero: String,
    pub tipo: String,
    pub creado_en: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ComprobanteEvento {
    pub id: i64,
    pub tipo_evento: String,
    pub fecha: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObtenerOCrearComprobante {
    pub venta_id: i64,
    pub tipo: String,
}
