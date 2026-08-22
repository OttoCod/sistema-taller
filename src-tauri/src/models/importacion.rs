use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Importacion {
    pub id: i64,
    pub archivo_nombre: String,
    pub archivo_hash: String,
    pub hoja_detectada: String,
    pub fila_encabezado_detectada: i64,
    pub columnas_detectadas_json: String,
    pub estado: String,
    pub total_filas: i64,
    pub creada_en: String,
    pub cerrada_en: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ImportacionFila {
    pub id: i64,
    pub importacion_id: i64,
    pub fila_excel: i64,
    pub codigo_excel: Option<String>,
    pub nombre_excel: Option<String>,
    /// Centavos.
    pub precio_lista_centavos: Option<i64>,
    pub categoria_excel_texto: Option<String>,
    /// JSON `{"pp+50%": 2613.0, ...}`, solo referencia histórica -- nunca
    /// se convierte en campos de producto.
    pub precios_calculados_json: Option<String>,
    pub clasificacion: String,
    pub motivo_error: Option<String>,
    pub es_duplicado_codigo: bool,
    pub es_posible_duplicado_nombre: bool,
    pub coincide_producto_existente_id: Option<i64>,
    pub decision: Option<String>,
    pub producto_vinculado_id: Option<i64>,
    pub actualizar_costo_en_vinculo: bool,
    pub producto_id: Option<i64>,
    pub resuelta_en: Option<String>,
}

/// Resumen agregado para la vista previa (sección 5 del diseño): se
/// calcula con una consulta, nunca se cachea en `importaciones`.
#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ResumenImportacion {
    pub total: i64,
    pub validos: i64,
    pub sin_codigo: i64,
    pub sin_nombre: i64,
    pub duplicados_codigo: i64,
    pub duplicados_nombre: i64,
    pub coincide_existente: i64,
    pub secciones: i64,
    pub ignoradas: i64,
    pub errores: i64,
    pub pendientes: i64,
    pub resueltas: i64,
}

/// Qué columna de la hoja detectada cumple cada rol: se guarda el texto
/// del encabezado tal cual estaba en el archivo (para mostrárselo al
/// usuario) junto con el índice relativo dentro de la fila usado para
/// leer los datos.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnaDetectada {
    pub encabezado: String,
    pub indice: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnasDetectadas {
    pub codigo: Option<ColumnaDetectada>,
    pub nombre: ColumnaDetectada,
    pub precio: ColumnaDetectada,
    /// Columnas de porcentaje (pp+50%, etc.) que se snapshotean para
    /// auditoría pero nunca se importan como precios propios del
    /// producto.
    pub derivadas: Vec<ColumnaDetectada>,
}

/// Resultado de detectar dinámicamente dónde están los datos en el
/// archivo: nunca se asume una hoja/fila/columna fija (ver services::
/// lector_excel).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EstructuraDetectada {
    pub hoja: String,
    /// Número de fila real del Excel (1-based, tal cual lo ve el
    /// usuario al abrirlo).
    pub fila_encabezado: i64,
    pub columnas: ColumnasDetectadas,
}

/// Una fila cruda ya leída del Excel, antes de clasificar/normalizar.
#[derive(Debug, Clone)]
pub struct FilaCruda {
    /// Número de fila real en el Excel (1-based, tal cual lo vería el
    /// usuario abriéndolo), para trazabilidad.
    pub fila_excel: i64,
    pub codigo: Option<String>,
    pub nombre: Option<String>,
    /// Centavos, ya convertido. `None` si la celda estaba vacía.
    pub precio_centavos: Option<i64>,
    /// Presente solo si la celda de precio tenía algo no convertible a
    /// número -- dispara clasificación `error`.
    pub precio_texto_invalido: Option<String>,
    pub precios_derivados: Vec<(String, f64)>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolverFila {
    /// crear_nuevo | vincular_existente | omitir
    pub decision: String,
    pub producto_vinculado_id: Option<i64>,
    #[serde(default)]
    pub actualizar_costo_en_vinculo: bool,
    /// Si el usuario corrigió el nombre/código durante la revisión (por
    /// ejemplo, una fila que llegó sin nombre) -- `None` deja el valor
    /// que ya traía la fila tal cual se leyó del Excel.
    pub nombre_corregido: Option<String>,
    pub codigo_corregido: Option<String>,
}
