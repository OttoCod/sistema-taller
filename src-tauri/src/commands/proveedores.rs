use tauri::State;

use crate::db::AppState;
use crate::error::AppResult;
use crate::models::proveedor::{GuardarProveedor, Proveedor};
use crate::services::proveedores;

#[tauri::command]
pub async fn proveedores_listar(state: State<'_, AppState>) -> AppResult<Vec<Proveedor>> {
    proveedores::listar(&state.pool).await
}

#[tauri::command]
pub async fn proveedores_buscar(
    state: State<'_, AppState>,
    consulta: String,
) -> AppResult<Vec<Proveedor>> {
    if consulta.trim().is_empty() {
        return proveedores::listar(&state.pool).await;
    }
    proveedores::buscar(&state.pool, &consulta).await
}

#[tauri::command]
pub async fn proveedores_crear(
    state: State<'_, AppState>,
    datos: GuardarProveedor,
) -> AppResult<Proveedor> {
    proveedores::crear(&state.pool, datos).await
}

#[tauri::command]
pub async fn proveedores_actualizar(
    state: State<'_, AppState>,
    id: i64,
    datos: GuardarProveedor,
) -> AppResult<Proveedor> {
    proveedores::actualizar(&state.pool, id, datos).await
}
