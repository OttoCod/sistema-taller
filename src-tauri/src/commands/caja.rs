use tauri::State;

use crate::db::AppState;
use crate::error::AppResult;
use crate::models::caja::ResumenCaja;
use crate::services::caja;

#[tauri::command]
pub async fn caja_resumen(state: State<'_, AppState>, fecha: String) -> AppResult<ResumenCaja> {
    caja::resumen(&state.pool, &fecha).await
}
