use serde::{Deserialize, Serialize};

/// Los únicos datos de `configuracion` que necesita esta fase: el
/// encabezado del comprobante impreso. El resto de las claves que
/// contempla el esquema original (formato de comprobante, numeración,
/// decimales de moneda) se agregan cuando algún módulo los necesite -- la
/// tabla clave-valor no exige una migración para eso.
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ConfiguracionNegocio {
    pub nombre: String,
    pub direccion: String,
    pub telefono: String,
}
