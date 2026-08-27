use tauri::State;

use crate::db::AppState;
use crate::error::AppResult;
use crate::models::comprobante::{Comprobante, ComprobanteEvento, ObtenerOCrearComprobante};
use crate::services::comprobantes;

#[tauri::command]
pub async fn comprobantes_obtener_o_crear(
    state: State<'_, AppState>,
    datos: ObtenerOCrearComprobante,
) -> AppResult<Comprobante> {
    comprobantes::obtener_o_crear(&state.pool, datos.venta_id, &datos.tipo).await
}

#[tauri::command]
pub async fn comprobantes_listar_por_venta(
    state: State<'_, AppState>,
    venta_id: i64,
) -> AppResult<Vec<Comprobante>> {
    comprobantes::listar_por_venta(&state.pool, venta_id).await
}

#[tauri::command]
pub async fn comprobantes_registrar_evento(
    state: State<'_, AppState>,
    comprobante_id: i64,
    tipo_evento: String,
) -> AppResult<()> {
    comprobantes::registrar_evento(&state.pool, comprobante_id, &tipo_evento).await
}

#[tauri::command]
pub async fn comprobantes_listar_eventos(
    state: State<'_, AppState>,
    comprobante_id: i64,
) -> AppResult<Vec<ComprobanteEvento>> {
    comprobantes::listar_eventos(&state.pool, comprobante_id).await
}
