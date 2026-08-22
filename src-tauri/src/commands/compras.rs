use tauri::State;

use crate::db::AppState;
use crate::error::AppResult;
use crate::models::compra::{Compra, CompraDetalle, CrearCompra};
use crate::services::compras;

#[tauri::command]
pub async fn compras_listar(state: State<'_, AppState>) -> AppResult<Vec<Compra>> {
    compras::listar(&state.pool).await
}

#[tauri::command]
pub async fn compras_obtener(state: State<'_, AppState>, id: i64) -> AppResult<CompraDetalle> {
    compras::obtener(&state.pool, id).await
}

#[tauri::command]
pub async fn compras_crear(
    state: State<'_, AppState>,
    datos: CrearCompra,
) -> AppResult<CompraDetalle> {
    compras::crear(&state.pool, datos).await
}
