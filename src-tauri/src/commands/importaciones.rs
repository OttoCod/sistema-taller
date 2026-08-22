use tauri::State;

use crate::db::AppState;
use crate::error::AppResult;
use crate::models::importacion::{Importacion, ImportacionFila, ResolverFila, ResumenImportacion};
use crate::services::importaciones;

#[tauri::command]
pub async fn importaciones_procesar_archivo(
    state: State<'_, AppState>,
    ruta: String,
    archivo_nombre: String,
) -> AppResult<Importacion> {
    importaciones::procesar_archivo(&state.pool, std::path::Path::new(&ruta), archivo_nombre).await
}

#[tauri::command]
pub async fn importaciones_listar(state: State<'_, AppState>) -> AppResult<Vec<Importacion>> {
    importaciones::listar(&state.pool).await
}

#[tauri::command]
pub async fn importaciones_obtener(state: State<'_, AppState>, id: i64) -> AppResult<Importacion> {
    importaciones::obtener(&state.pool, id).await
}

#[tauri::command]
pub async fn importaciones_resumen(
    state: State<'_, AppState>,
    id: i64,
) -> AppResult<ResumenImportacion> {
    importaciones::resumen(&state.pool, id).await
}

#[tauri::command]
pub async fn importaciones_listar_filas(
    state: State<'_, AppState>,
    id: i64,
) -> AppResult<Vec<ImportacionFila>> {
    importaciones::listar_filas(&state.pool, id).await
}

#[tauri::command]
pub async fn importaciones_buscar_confirmada_con_mismo_hash(
    state: State<'_, AppState>,
    hash: String,
    excluyendo_id: i64,
) -> AppResult<Option<Importacion>> {
    importaciones::buscar_confirmada_con_mismo_hash(&state.pool, &hash, excluyendo_id).await
}

#[tauri::command]
pub async fn importaciones_resolver_fila(
    state: State<'_, AppState>,
    fila_id: i64,
    datos: ResolverFila,
) -> AppResult<ImportacionFila> {
    importaciones::resolver_fila(&state.pool, fila_id, datos).await
}

#[tauri::command]
pub async fn importaciones_aplicar_pendientes(
    state: State<'_, AppState>,
    id: i64,
) -> AppResult<i64> {
    importaciones::aplicar_pendientes_con_decision(&state.pool, id).await
}

#[tauri::command]
pub async fn importaciones_descartar(
    state: State<'_, AppState>,
    id: i64,
) -> AppResult<Importacion> {
    importaciones::descartar(&state.pool, id).await
}
