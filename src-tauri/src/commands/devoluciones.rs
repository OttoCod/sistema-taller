use tauri::State;

use crate::db::AppState;
use crate::error::AppResult;
use crate::models::devolucion::{CrearDevolucion, DevolucionConDetalles};
use crate::services::devoluciones;

#[tauri::command]
pub async fn devoluciones_crear(
    state: State<'_, AppState>,
    datos: CrearDevolucion,
) -> AppResult<DevolucionConDetalles> {
    devoluciones::crear(&state.pool, datos).await
}

#[tauri::command]
pub async fn devoluciones_listar_por_venta(
    state: State<'_, AppState>,
    venta_id: i64,
) -> AppResult<Vec<DevolucionConDetalles>> {
    devoluciones::listar_por_venta(&state.pool, venta_id).await
}
