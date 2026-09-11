use std::path::PathBuf;

use tauri::{AppHandle, Manager, State};

use crate::db::AppState;
use crate::error::{AppError, AppResult};
use crate::models::backup::{Backup, ConfiguracionBackups, RestauracionPreparada};
use crate::services::backups;

/// Carpeta de datos de la app: es donde vive la base y, si no se eligió
/// otra, también los backups.
fn app_data_dir(app: &AppHandle) -> AppResult<PathBuf> {
    app.path().app_data_dir().map_err(|err| {
        AppError::Unexpected(format!("no se pudo resolver la carpeta de datos: {err}"))
    })
}

#[tauri::command]
pub async fn backups_listar(state: State<'_, AppState>) -> AppResult<Vec<Backup>> {
    backups::listar(&state.pool).await
}

#[tauri::command]
pub async fn backups_crear(app: AppHandle, state: State<'_, AppState>) -> AppResult<Backup> {
    let dir = app_data_dir(&app)?;
    backups::crear(&state.pool, &dir, "manual").await
}

#[tauri::command]
pub async fn backups_obtener_configuracion(
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<ConfiguracionBackups> {
    let dir = app_data_dir(&app)?;
    backups::obtener_configuracion(&state.pool, &dir).await
}

#[tauri::command]
pub async fn backups_guardar_configuracion(
    state: State<'_, AppState>,
    carpeta: String,
    retencion: i64,
) -> AppResult<()> {
    backups::guardar_configuracion(&state.pool, &carpeta, retencion).await
}

/// Prepara la restauración (copia de seguridad + marcador). El reemplazo
/// real ocurre al reiniciar: el frontend avisa y ofrece `app_reiniciar`.
#[tauri::command]
pub async fn backups_preparar_restauracion(
    app: AppHandle,
    state: State<'_, AppState>,
    id: i64,
) -> AppResult<RestauracionPreparada> {
    let dir = app_data_dir(&app)?;
    backups::preparar_restauracion(&state.pool, &dir, id).await
}

/// Reinicio explícito, pedido desde la pantalla después de preparar una
/// restauración. Nunca se dispara solo.
#[tauri::command]
pub fn app_reiniciar(app: AppHandle) {
    app.restart();
}
