use serde::Serialize;

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct MetodoPago {
    pub id: i64,
    pub nombre: String,
}
