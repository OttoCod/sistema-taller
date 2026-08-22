use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Proveedor {
    pub id: i64,
    pub nombre: String,
    pub telefono: Option<String>,
    pub whatsapp: Option<String>,
    pub email: Option<String>,
    pub sitio_web: Option<String>,
    pub observaciones: Option<String>,
    pub activo: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuardarProveedor {
    pub nombre: String,
    pub telefono: Option<String>,
    pub whatsapp: Option<String>,
    pub email: Option<String>,
    pub sitio_web: Option<String>,
    pub observaciones: Option<String>,
    pub activo: bool,
}
