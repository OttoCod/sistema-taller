use tauri::State;

use crate::db::AppState;
use crate::error::AppResult;
use crate::models::marca::{Marca, NuevaMarca};
use crate::services::marcas;

#[tauri::command]
pub async fn marcas_listar(state: State<'_, AppState>) -> AppResult<Vec<Marca>> {
    marcas::listar(&state.pool).await
}

#[tauri::command]
pub async fn marcas_crear(state: State<'_, AppState>, datos: NuevaMarca) -> AppResult<Marca> {
    marcas::crear(&state.pool, datos).await
}
