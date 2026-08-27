use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};
use crate::models::configuracion::ConfiguracionNegocio;

const CLAVE_NOMBRE: &str = "negocio.nombre";
const CLAVE_DIRECCION: &str = "negocio.direccion";
const CLAVE_TELEFONO: &str = "negocio.telefono";

async fn obtener_valor(pool: &SqlitePool, clave: &str) -> AppResult<Option<String>> {
    let fila: Option<(Option<String>,)> =
        sqlx::query_as("SELECT valor FROM configuracion WHERE clave = ?")
            .bind(clave)
            .fetch_optional(pool)
            .await?;
    Ok(fila.and_then(|(v,)| v))
}

/// Si nunca se cargó, devuelve todo vacío -- no es un error: el negocio
/// simplemente todavía no llenó su ficha en Configuración.
pub async fn obtener_negocio(pool: &SqlitePool) -> AppResult<ConfiguracionNegocio> {
    Ok(ConfiguracionNegocio {
        nombre: obtener_valor(pool, CLAVE_NOMBRE).await?.unwrap_or_default(),
        direccion: obtener_valor(pool, CLAVE_DIRECCION)
            .await?
            .unwrap_or_default(),
        telefono: obtener_valor(pool, CLAVE_TELEFONO)
            .await?
            .unwrap_or_default(),
    })
}

async fn upsert(pool: &SqlitePool, clave: &str, valor: &str) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO configuracion (clave, valor) VALUES (?, ?)
         ON CONFLICT (clave) DO UPDATE SET
            valor = excluded.valor,
            actualizado_en = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
    )
    .bind(clave)
    .bind(valor)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn guardar_negocio(pool: &SqlitePool, datos: ConfiguracionNegocio) -> AppResult<()> {
    let nombre = datos.nombre.trim();
    if nombre.is_empty() {
        return Err(AppError::Validation(
            "El nombre del negocio no puede estar vacío: aparece en el encabezado de los comprobantes.".into(),
        ));
    }

    upsert(pool, CLAVE_NOMBRE, nombre).await?;
    upsert(pool, CLAVE_DIRECCION, datos.direccion.trim()).await?;
    upsert(pool, CLAVE_TELEFONO, datos.telefono.trim()).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn pool_de_prueba() -> SqlitePool {
        let dir = tempfile::tempdir().expect("tempdir");
        let dir = Box::leak(Box::new(dir));
        crate::db::init_pool(dir.path()).await.expect("init_pool")
    }

    #[tokio::test]
    async fn sin_configurar_devuelve_todo_vacio() {
        let pool = pool_de_prueba().await;
        let negocio = obtener_negocio(&pool).await.unwrap();
        assert_eq!(negocio.nombre, "");
        assert_eq!(negocio.direccion, "");
        assert_eq!(negocio.telefono, "");
    }

    #[tokio::test]
    async fn guardar_y_releer_persiste_los_valores() {
        let pool = pool_de_prueba().await;
        guardar_negocio(
            &pool,
            ConfiguracionNegocio {
                nombre: "Espínola Motorepuestos".into(),
                direccion: "Av. Siempre Viva 123".into(),
                telefono: "3794-000000".into(),
            },
        )
        .await
        .expect("guardar");

        let negocio = obtener_negocio(&pool).await.unwrap();
        assert_eq!(negocio.nombre, "Espínola Motorepuestos");
        assert_eq!(negocio.direccion, "Av. Siempre Viva 123");
        assert_eq!(negocio.telefono, "3794-000000");
    }

    #[tokio::test]
    async fn guardar_dos_veces_actualiza_en_vez_de_duplicar() {
        let pool = pool_de_prueba().await;
        guardar_negocio(
            &pool,
            ConfiguracionNegocio {
                nombre: "Nombre viejo".into(),
                direccion: "".into(),
                telefono: "".into(),
            },
        )
        .await
        .unwrap();
        guardar_negocio(
            &pool,
            ConfiguracionNegocio {
                nombre: "Nombre nuevo".into(),
                direccion: "".into(),
                telefono: "".into(),
            },
        )
        .await
        .unwrap();

        let negocio = obtener_negocio(&pool).await.unwrap();
        assert_eq!(negocio.nombre, "Nombre nuevo");

        let (cantidad,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM configuracion WHERE clave = 'negocio.nombre'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(cantidad, 1);
    }

    #[tokio::test]
    async fn nombre_vacio_falla() {
        let pool = pool_de_prueba().await;
        let resultado = guardar_negocio(
            &pool,
            ConfiguracionNegocio {
                nombre: "   ".into(),
                direccion: "".into(),
                telefono: "".into(),
            },
        )
        .await;
        assert!(matches!(resultado, Err(AppError::Validation(_))));
    }
}
