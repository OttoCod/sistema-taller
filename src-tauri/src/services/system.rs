use serde::Serialize;
use sqlx::SqlitePool;

use crate::error::AppResult;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthCheck {
    pub app_version: String,
    pub schema_version: i64,
    pub db_path: String,
    pub sqlite_version: String,
}

/// Vertical slice de la Fase 1: prueba que React puede pedirle algo a Rust,
/// Rust puede consultar SQLite, y la respuesta tipada vuelve a React.
/// No es un módulo funcional del negocio.
pub async fn health_check(pool: &SqlitePool, db_path: &str) -> AppResult<HealthCheck> {
    let (sqlite_version,): (String,) = sqlx::query_as("SELECT sqlite_version()")
        .fetch_one(pool)
        .await?;

    let (schema_version,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM _sqlx_migrations")
        .fetch_one(pool)
        .await?;

    Ok(HealthCheck {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        schema_version,
        db_path: db_path.to_string(),
        sqlite_version,
    })
}
