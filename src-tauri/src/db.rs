use std::path::{Path, PathBuf};

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};

/// Estado compartido entre todos los comandos de Tauri.
pub struct AppState {
    pub pool: SqlitePool,
    pub db_path: String,
}

pub fn db_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("espinola.db")
}

/// Abre (o crea) la base SQLite, activa las claves foráneas y el modo WAL,
/// y aplica las migraciones pendientes. Se llama una sola vez, al arrancar
/// la app, antes de que cualquier comando pueda ejecutarse.
pub async fn init_pool(app_data_dir: &Path) -> AppResult<SqlitePool> {
    std::fs::create_dir_all(app_data_dir)?;

    let options = SqliteConnectOptions::new()
        .filename(db_path(app_data_dir))
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal);

    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .connect_with(options)
        .await?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|err| {
            AppError::Unexpected(format!("no se pudieron aplicar las migraciones: {err}"))
        })?;

    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Prueba de extremo a extremo de la Fase 1: crear la base, aplicar la
    /// migración de arranque, confirmar el usuario sembrado, y llamar al
    /// mismo health check que expone el comando de Tauri.
    #[tokio::test]
    async fn init_pool_aplica_migraciones_y_siembra_usuario_principal() {
        let dir = tempfile::tempdir().expect("tempdir");
        let pool = init_pool(dir.path()).await.expect("init_pool");

        let (usuarios,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM usuarios")
            .fetch_one(&pool)
            .await
            .expect("query usuarios");
        assert_eq!(usuarios, 1);

        let (nombre,): (String,) = sqlx::query_as("SELECT nombre FROM usuarios WHERE id = 1")
            .fetch_one(&pool)
            .await
            .expect("query nombre");
        assert_eq!(nombre, "Usuario principal");

        let health = crate::services::system::health_check(&pool, "test.db")
            .await
            .expect("health_check");
        assert_eq!(health.schema_version, 1);
        assert!(!health.sqlite_version.is_empty());
    }
}
