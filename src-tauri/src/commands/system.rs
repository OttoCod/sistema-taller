use tauri::State;

use crate::db::AppState;
use crate::error::AppResult;
use crate::services::system::{self, HealthCheck};

#[tauri::command]
pub async fn system_health_check(state: State<'_, AppState>) -> AppResult<HealthCheck> {
    system::health_check(&state.pool, &state.db_path).await
}
