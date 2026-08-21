use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};
use crate::models::categoria::{Categoria, NuevaCategoria};

const SELECT_CATEGORIA: &str = "
    SELECT c.id, c.nombre, c.categoria_padre_id, p.nombre AS categoria_padre_nombre
    FROM categorias c
    LEFT JOIN categorias p ON p.id = c.categoria_padre_id
";

pub async fn listar(pool: &SqlitePool) -> AppResult<Vec<Categoria>> {
    let categorias =
        sqlx::query_as::<_, Categoria>(&format!("{SELECT_CATEGORIA} ORDER BY c.nombre"))
            .fetch_all(pool)
            .await?;
    Ok(categorias)
}

pub async fn crear(pool: &SqlitePool, datos: NuevaCategoria) -> AppResult<Categoria> {
    let nombre = datos.nombre.trim();
    if nombre.is_empty() {
        return Err(AppError::Validation(
            "El nombre de la categoría no puede estar vacío.".into(),
        ));
    }

    let id = sqlx::query("INSERT INTO categorias (nombre, categoria_padre_id) VALUES (?, ?)")
        .bind(nombre)
        .bind(datos.categoria_padre_id)
        .execute(pool)
        .await?
        .last_insert_rowid();

    let categoria = sqlx::query_as::<_, Categoria>(&format!("{SELECT_CATEGORIA} WHERE c.id = ?"))
        .bind(id)
        .fetch_one(pool)
        .await?;
    Ok(categoria)
}
