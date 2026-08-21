use sqlx::SqlitePool;

use crate::error::AppResult;
use crate::models::metodo_pago::MetodoPago;

pub async fn listar(pool: &SqlitePool) -> AppResult<Vec<MetodoPago>> {
    let metodos = sqlx::query_as::<_, MetodoPago>(
        "SELECT id, nombre FROM metodos_pago WHERE activo = 1 ORDER BY orden",
    )
    .fetch_all(pool)
    .await?;
    Ok(metodos)
}
