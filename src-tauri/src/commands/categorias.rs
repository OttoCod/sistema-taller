use tauri::State;

use crate::db::AppState;
use crate::error::AppResult;
use crate::models::categoria::{Categoria, NuevaCategoria};
use crate::services::categorias;

#[tauri::command]
pub async fn categorias_listar(state: State<'_, AppState>) -> AppResult<Vec<Categoria>> {
    categorias::listar(&state.pool).await
}

#[tauri::command]
pub async fn categorias_crear(
    state: State<'_, AppState>,
    datos: NuevaCategoria,
) -> AppResult<Categoria> {
    categorias::crear(&state.pool, datos).await
}
