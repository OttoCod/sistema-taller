use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};
use crate::models::marca::{Marca, NuevaMarca};

pub async fn listar(pool: &SqlitePool) -> AppResult<Vec<Marca>> {
    let marcas = sqlx::query_as::<_, Marca>("SELECT id, nombre FROM marcas ORDER BY nombre")
        .fetch_all(pool)
        .await?;
    Ok(marcas)
}

pub async fn crear(pool: &SqlitePool, datos: NuevaMarca) -> AppResult<Marca> {
    let nombre = datos.nombre.trim();
    if nombre.is_empty() {
        return Err(AppError::Validation(
            "El nombre de la marca no puede estar vacío.".into(),
        ));
    }

    let existente = sqlx::query_as::<_, Marca>("SELECT id, nombre FROM marcas WHERE nombre = ?")
        .bind(nombre)
        .fetch_optional(pool)
        .await?;
    if let Some(marca) = existente {
        return Ok(marca);
    }

    let id = sqlx::query("INSERT INTO marcas (nombre) VALUES (?)")
        .bind(nombre)
        .execute(pool)
        .await?
        .last_insert_rowid();

    Ok(Marca {
        id,
        nombre: nombre.to_string(),
    })
}
