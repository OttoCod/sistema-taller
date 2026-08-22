use tauri::State;

use crate::db::AppState;
use crate::error::AppResult;
use crate::models::producto_proveedor::{GuardarProductoProveedor, ProductoProveedor};
use crate::services::producto_proveedores;

#[tauri::command]
pub async fn producto_proveedores_listar(
    state: State<'_, AppState>,
    proveedor_id: i64,
) -> AppResult<Vec<ProductoProveedor>> {
    producto_proveedores::listar_por_proveedor(&state.pool, proveedor_id).await
}

#[tauri::command]
pub async fn producto_proveedores_agregar(
    state: State<'_, AppState>,
    proveedor_id: i64,
    datos: GuardarProductoProveedor,
) -> AppResult<ProductoProveedor> {
    producto_proveedores::agregar(&state.pool, proveedor_id, datos).await
}

#[tauri::command]
pub async fn producto_proveedores_quitar(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    producto_proveedores::quitar(&state.pool, id).await
}
