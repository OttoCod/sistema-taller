use tauri::State;

use crate::db::AppState;
use crate::error::AppResult;
use crate::models::cuenta_corriente::{
    AjusteCuentaCorriente, MovimientoCuentaCorriente, RegistrarPago,
};
use crate::services::cuenta_corriente;

#[tauri::command]
pub async fn cuenta_corriente_listar_movimientos(
    state: State<'_, AppState>,
    cliente_id: i64,
) -> AppResult<Vec<MovimientoCuentaCorriente>> {
    cuenta_corriente::listar_movimientos(&state.pool, cliente_id).await
}

#[tauri::command]
pub async fn cuenta_corriente_registrar_pago(
    state: State<'_, AppState>,
    cliente_id: i64,
    datos: RegistrarPago,
) -> AppResult<()> {
    cuenta_corriente::registrar_pago(&state.pool, cliente_id, datos).await
}

#[tauri::command]
pub async fn cuenta_corriente_ajustar(
    state: State<'_, AppState>,
    cliente_id: i64,
    datos: AjusteCuentaCorriente,
) -> AppResult<()> {
    cuenta_corriente::ajustar(&state.pool, cliente_id, datos).await
}
