use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct MovimientoCuentaCorriente {
    pub id: i64,
    /// venta_fiada | pago | ajuste | devolucion
    pub tipo: String,
    pub monto: i64,
    pub saldo_resultante: i64,
    pub metodo_pago_nombre: Option<String>,
    pub fecha: String,
    pub observacion: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrarPago {
    /// Centavos, siempre positivo -- lo que pagó el cliente.
    pub monto: i64,
    pub metodo_pago_id: i64,
    pub observacion: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AjusteCuentaCorriente {
    /// Centavos. Positivo aumenta la deuda (ej. migrar saldo previo desde
    /// papel), negativo la reduce (ej. corrección, condonación).
    pub monto: i64,
    pub motivo: String,
}
