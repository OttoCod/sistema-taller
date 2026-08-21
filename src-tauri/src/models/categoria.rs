use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Categoria {
    pub id: i64,
    pub nombre: String,
    pub categoria_padre_id: Option<i64>,
    pub categoria_padre_nombre: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NuevaCategoria {
    pub nombre: String,
    pub categoria_padre_id: Option<i64>,
}
