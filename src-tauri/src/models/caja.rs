use serde::Serialize;

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct MontoPorMetodo {
    pub metodo_pago_id: i64,
    pub metodo_pago_nombre: String,
    /// Centavos.
    pub monto: i64,
}

/// La caja no es una tabla propia (ver ESQUEMA_BD.md, punto B): se
/// calcula agrupando `venta_pagos` de ventas confirmadas de un día,
/// para que nunca pueda desincronizarse de lo que dicen las ventas.
/// `total_cobrado` excluye "cuenta_corriente" -- fiado no es plata que
/// entró a la caja. Arqueo y egresos quedan fuera de esta fase.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResumenCaja {
    pub fecha: String,
    pub por_metodo: Vec<MontoPorMetodo>,
    /// Centavos. Suma de por_metodo excluyendo cuenta_corriente.
    pub total_cobrado: i64,
    /// Centavos. Lo que quedó fiado ese día (informativo, no es caja).
    pub total_fiado: i64,
    pub cantidad_ventas: i64,
}
