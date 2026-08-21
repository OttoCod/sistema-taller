use tauri::State;

use crate::db::AppState;
use crate::error::AppResult;
use crate::models::venta::{CrearVenta, Venta, VentaDetalle};
use crate::services::ventas;

#[tauri::command]
pub async fn ventas_listar(state: State<'_, AppState>) -> AppResult<Vec<Venta>> {
    ventas::listar(&state.pool).await
}

#[tauri::command]
pub async fn ventas_obtener(state: State<'_, AppState>, id: i64) -> AppResult<VentaDetalle> {
    ventas::obtener(&state.pool, id).await
}

#[tauri::command]
pub async fn ventas_crear(
    state: State<'_, AppState>,
    datos: CrearVenta,
) -> AppResult<VentaDetalle> {
    ventas::crear(&state.pool, datos).await
}
