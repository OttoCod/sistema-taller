use tauri::State;

use crate::db::AppState;
use crate::error::AppResult;
use crate::models::metodo_pago::MetodoPago;
use crate::services::metodos_pago;

#[tauri::command]
pub async fn metodos_pago_listar(state: State<'_, AppState>) -> AppResult<Vec<MetodoPago>> {
    metodos_pago::listar(&state.pool).await
}
