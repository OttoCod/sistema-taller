use serde::{Deserialize, Serialize};

/// Un vínculo producto-proveedor, con el nombre/código del producto ya
/// resuelto por join (se lista siempre desde la pantalla de un
/// proveedor puntual).
#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ProductoProveedor {
    pub id: i64,
    pub producto_id: i64,
    pub producto_nombre: String,
    pub codigo_interno: String,
    pub proveedor_id: i64,
    pub codigo_proveedor: Option<String>,
    pub url_producto: Option<String>,
    pub url_busqueda: Option<String>,
    pub es_principal: bool,
    pub activo: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuardarProductoProveedor {
    pub producto_id: i64,
    pub codigo_proveedor: Option<String>,
    pub url_producto: Option<String>,
    pub url_busqueda: Option<String>,
    pub es_principal: bool,
}
