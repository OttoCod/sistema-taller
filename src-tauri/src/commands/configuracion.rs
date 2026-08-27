use tauri::State;

use crate::db::AppState;
use crate::error::AppResult;
use crate::models::configuracion::ConfiguracionNegocio;
use crate::services::configuracion;

#[tauri::command]
pub async fn configuracion_obtener_negocio(
    state: State<'_, AppState>,
) -> AppResult<ConfiguracionNegocio> {
    configuracion::obtener_negocio(&state.pool).await
}

#[tauri::command]
pub async fn configuracion_guardar_negocio(
    state: State<'_, AppState>,
    datos: ConfiguracionNegocio,
) -> AppResult<()> {
    configuracion::guardar_negocio(&state.pool, datos).await
}
