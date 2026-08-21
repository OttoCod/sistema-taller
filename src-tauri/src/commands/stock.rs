use tauri::State;

use crate::db::AppState;
use crate::error::AppResult;
use crate::models::stock::{AjusteStock, ProductoStock};
use crate::services::stock;

#[tauri::command]
pub async fn stock_listar(state: State<'_, AppState>) -> AppResult<Vec<ProductoStock>> {
    stock::listar(&state.pool).await
}

#[tauri::command]
pub async fn stock_listar_reposicion(state: State<'_, AppState>) -> AppResult<Vec<ProductoStock>> {
    stock::listar_reposicion(&state.pool).await
}

#[tauri::command]
pub async fn stock_ajustar(
    state: State<'_, AppState>,
    producto_id: i64,
    datos: AjusteStock,
) -> AppResult<()> {
    stock::ajustar(&state.pool, producto_id, datos).await
}

#[tauri::command]
pub async fn stock_actualizar_minimo(
    state: State<'_, AppState>,
    producto_id: i64,
    stock_minimo: i64,
) -> AppResult<()> {
    stock::actualizar_minimo(&state.pool, producto_id, stock_minimo).await
}
