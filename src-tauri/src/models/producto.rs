use serde::{Deserialize, Serialize};

/// Fila de la tabla productos, con el nombre de marca/categoría ya resuelto
/// por join (evita que cada pantalla tenga que resolverlo por separado).
#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Producto {
    pub id: i64,
    pub codigo_interno: String,
    pub nombre: String,
    pub marca_id: Option<i64>,
    pub marca_nombre: Option<String>,
    pub categoria_id: Option<i64>,
    pub categoria_nombre: Option<String>,
    pub descripcion: Option<String>,
    pub observaciones: Option<String>,
    /// Centavos. Nunca de punto flotante (ver docs/ESQUEMA_BD.md).
    pub costo_actual: Option<i64>,
    pub precio_venta_actual: Option<i64>,
    pub precio_publico_referencia: Option<i64>,
    pub precio_actualizado_en: Option<String>,
    pub estado: String,
    /// Se agregó en la Fase 5: el carrito de venta necesita mostrar
    /// disponibilidad. No se editaba desde el catálogo (eso sigue siendo
    /// solo la pantalla de Stock, Fase 4).
    pub stock_actual: i64,
    /// Resumen ("1566, 00034420") para el listado. Nombre distinto del
    /// codigos_fabricante de ProductoDetalle (ahí sí van completos, con
    /// fabricante y observación) para que el flatten no choque.
    pub codigos_fabricante_resumen: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CodigoFabricante {
    pub codigo: String,
    pub fabricante_nombre: Option<String>,
    pub observacion: Option<String>,
}

/// Datos que llegan del formulario de producto tanto para crear como para
/// editar. codigos_fabricante reemplaza la lista completa en cada guardado
/// (no hay diffing fila por fila: es más simple y suficiente acá).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuardarProducto {
    pub nombre: String,
    pub marca_id: Option<i64>,
    pub categoria_id: Option<i64>,
    pub descripcion: Option<String>,
    pub observaciones: Option<String>,
    pub costo_actual: Option<i64>,
    pub precio_venta_actual: Option<i64>,
    pub precio_publico_referencia: Option<i64>,
    pub estado: String,
    pub codigos_fabricante: Vec<CodigoFabricante>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductoDetalle {
    #[serde(flatten)]
    pub producto: Producto,
    pub codigos_fabricante: Vec<CodigoFabricante>,
}
