use serde::{Deserialize, Serialize};

pub const ID_CONSUMIDOR_FINAL: i64 = 1;

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Cliente {
    pub id: i64,
    pub nombre: String,
    pub telefono: Option<String>,
    pub dni_cuit: Option<String>,
    pub direccion: Option<String>,
    pub observaciones: Option<String>,
    /// Centavos. Positivo = el cliente debe; negativo = tiene a favor.
    pub saldo_cuenta_corriente: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuardarCliente {
    pub nombre: String,
    pub telefono: Option<String>,
    pub dni_cuit: Option<String>,
    pub direccion: Option<String>,
    pub observaciones: Option<String>,
}

/// Fila para la pantalla de Cuentas pendientes.
#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ClienteConDeuda {
    pub id: i64,
    pub nombre: String,
    pub telefono: Option<String>,
    pub saldo_cuenta_corriente: i64,
    pub fecha_ultimo_movimiento: Option<String>,
}
