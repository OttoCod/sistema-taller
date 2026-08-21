use tauri::State;

use crate::db::AppState;
use crate::error::AppResult;
use crate::models::producto::{GuardarProducto, Producto, ProductoDetalle};
use crate::services::productos;

#[tauri::command]
pub async fn productos_listar(state: State<'_, AppState>) -> AppResult<Vec<Producto>> {
    productos::listar(&state.pool).await
}

#[tauri::command]
pub async fn productos_obtener(state: State<'_, AppState>, id: i64) -> AppResult<ProductoDetalle> {
    productos::obtener(&state.pool, id).await
}

#[tauri::command]
pub async fn productos_crear(
    state: State<'_, AppState>,
    datos: GuardarProducto,
) -> AppResult<ProductoDetalle> {
    productos::crear(&state.pool, datos).await
}

#[tauri::command]
pub async fn productos_actualizar(
    state: State<'_, AppState>,
    id: i64,
    datos: GuardarProducto,
) -> AppResult<ProductoDetalle> {
    productos::actualizar(&state.pool, id, datos).await
}

#[tauri::command]
pub async fn productos_buscar(
    state: State<'_, AppState>,
    consulta: String,
) -> AppResult<Vec<Producto>> {
    productos::buscar(&state.pool, &consulta).await
}
