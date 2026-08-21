use tauri::State;

use crate::db::AppState;
use crate::error::AppResult;
use crate::models::cliente::{Cliente, ClienteConDeuda, GuardarCliente};
use crate::services::clientes;

#[tauri::command]
pub async fn clientes_listar(state: State<'_, AppState>) -> AppResult<Vec<Cliente>> {
    clientes::listar(&state.pool).await
}

#[tauri::command]
pub async fn clientes_buscar(
    state: State<'_, AppState>,
    consulta: String,
) -> AppResult<Vec<Cliente>> {
    if consulta.trim().is_empty() {
        return clientes::listar(&state.pool).await;
    }
    clientes::buscar(&state.pool, &consulta).await
}

#[tauri::command]
pub async fn clientes_obtener(state: State<'_, AppState>, id: i64) -> AppResult<Cliente> {
    clientes::obtener(&state.pool, id).await
}

#[tauri::command]
pub async fn clientes_crear(
    state: State<'_, AppState>,
    datos: GuardarCliente,
) -> AppResult<Cliente> {
    clientes::crear(&state.pool, datos).await
}

#[tauri::command]
pub async fn clientes_actualizar(
    state: State<'_, AppState>,
    id: i64,
    datos: GuardarCliente,
) -> AppResult<Cliente> {
    clientes::actualizar(&state.pool, id, datos).await
}

#[tauri::command]
pub async fn clientes_listar_cuentas_pendientes(
    state: State<'_, AppState>,
) -> AppResult<Vec<ClienteConDeuda>> {
    clientes::listar_cuentas_pendientes(&state.pool).await
}
