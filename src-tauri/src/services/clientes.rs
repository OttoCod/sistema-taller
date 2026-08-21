use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};
use crate::models::cliente::{Cliente, ClienteConDeuda, GuardarCliente, ID_CONSUMIDOR_FINAL};

const SELECT_CLIENTE: &str = "
    SELECT id, nombre, telefono, patente, direccion, observaciones, saldo_cuenta_corriente
    FROM clientes
";

/// Las patentes se guardan siempre en mayúsculas, sin importar cómo las
/// haya tipeado el vendedor -- así "abc123" y "ABC123" son la misma
/// búsqueda y no aparecen como dos formatos distintos en el listado.
fn normalizar_patente(patente: &Option<String>) -> Option<String> {
    patente
        .as_deref()
        .map(|p| p.trim().to_uppercase())
        .filter(|p| !p.is_empty())
}

pub async fn listar(pool: &SqlitePool) -> AppResult<Vec<Cliente>> {
    let clientes =
        sqlx::query_as::<_, Cliente>(&format!("{SELECT_CLIENTE} ORDER BY nombre LIMIT 500"))
            .fetch_all(pool)
            .await?;
    Ok(clientes)
}

/// Búsqueda simple por substring (no hace falta FTS5: la lista de clientes
/// de un negocio de este tamaño no justifica esa infraestructura).
pub async fn buscar(pool: &SqlitePool, consulta: &str) -> AppResult<Vec<Cliente>> {
    let patron = format!("%{}%", consulta.trim());
    let clientes = sqlx::query_as::<_, Cliente>(&format!(
        "{SELECT_CLIENTE}
         WHERE nombre LIKE ? OR telefono LIKE ? OR patente LIKE ?
         ORDER BY nombre LIMIT 100"
    ))
    .bind(&patron)
    .bind(&patron)
    .bind(&patron)
    .fetch_all(pool)
    .await?;
    Ok(clientes)
}

pub async fn obtener(pool: &SqlitePool, id: i64) -> AppResult<Cliente> {
    sqlx::query_as::<_, Cliente>(&format!("{SELECT_CLIENTE} WHERE id = ?"))
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("No existe el cliente {id}.")))
}

pub async fn crear(pool: &SqlitePool, datos: GuardarCliente) -> AppResult<Cliente> {
    let nombre = datos.nombre.trim();
    if nombre.is_empty() {
        return Err(AppError::Validation(
            "El nombre del cliente no puede estar vacío.".into(),
        ));
    }

    let id = sqlx::query(
        "INSERT INTO clientes (nombre, telefono, patente, direccion, observaciones)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(nombre)
    .bind(&datos.telefono)
    .bind(normalizar_patente(&datos.patente))
    .bind(&datos.direccion)
    .bind(&datos.observaciones)
    .execute(pool)
    .await?
    .last_insert_rowid();

    obtener(pool, id).await
}

pub async fn actualizar(pool: &SqlitePool, id: i64, datos: GuardarCliente) -> AppResult<Cliente> {
    if id == ID_CONSUMIDOR_FINAL {
        return Err(AppError::Validation(
            "\"Consumidor final\" es un cliente reservado del sistema y no se puede editar.".into(),
        ));
    }
    let nombre = datos.nombre.trim();
    if nombre.is_empty() {
        return Err(AppError::Validation(
            "El nombre del cliente no puede estar vacío.".into(),
        ));
    }

    // Confirma que existe antes de intentar el update (mensaje más claro
    // que un "0 filas afectadas" silencioso).
    obtener(pool, id).await?;

    sqlx::query(
        "UPDATE clientes SET nombre = ?, telefono = ?, patente = ?, direccion = ?, observaciones = ?
         WHERE id = ?",
    )
    .bind(nombre)
    .bind(&datos.telefono)
    .bind(normalizar_patente(&datos.patente))
    .bind(&datos.direccion)
    .bind(&datos.observaciones)
    .bind(id)
    .execute(pool)
    .await?;

    obtener(pool, id).await
}

pub async fn listar_cuentas_pendientes(pool: &SqlitePool) -> AppResult<Vec<ClienteConDeuda>> {
    let clientes = sqlx::query_as::<_, ClienteConDeuda>(
        "SELECT
            c.id, c.nombre, c.telefono, c.saldo_cuenta_corriente,
            (SELECT MAX(fecha) FROM cuenta_corriente_movimientos m WHERE m.cliente_id = c.id)
                AS fecha_ultimo_movimiento
         FROM clientes c
         WHERE c.saldo_cuenta_corriente > 0
         ORDER BY c.saldo_cuenta_corriente DESC",
    )
    .fetch_all(pool)
    .await?;
    Ok(clientes)
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
    async fn la_patente_se_guarda_en_mayusculas_y_sin_espacios() {
        let pool = pool_de_prueba().await;

        let cliente = crear(
            &pool,
            GuardarCliente {
                nombre: "Carlos Gómez".into(),
                telefono: None,
                patente: Some("  ab123cd  ".into()),
                direccion: None,
                observaciones: None,
            },
        )
        .await
        .expect("crear cliente");

        assert_eq!(cliente.patente.as_deref(), Some("AB123CD"));

        let encontrado = buscar(&pool, "ab123").await.expect("buscar");
        assert_eq!(encontrado.len(), 1);
    }

    #[tokio::test]
    async fn patente_vacia_se_guarda_como_nulo() {
        let pool = pool_de_prueba().await;

        let cliente = crear(
            &pool,
            GuardarCliente {
                nombre: "Sin patente".into(),
                telefono: None,
                patente: Some("   ".into()),
                direccion: None,
                observaciones: None,
            },
        )
        .await
        .expect("crear cliente");

        assert_eq!(cliente.patente, None);
    }

    #[tokio::test]
    async fn actualizar_persiste_los_cambios() {
        let pool = pool_de_prueba().await;
        let creado = crear(
            &pool,
            GuardarCliente {
                nombre: "Ana Díaz".into(),
                telefono: Some("3794111111".into()),
                patente: None,
                direccion: None,
                observaciones: None,
            },
        )
        .await
        .expect("crear");

        let actualizado = actualizar(
            &pool,
            creado.id,
            GuardarCliente {
                nombre: "Ana Díaz".into(),
                telefono: Some("3794222222".into()),
                patente: Some("xyz789".into()),
                direccion: Some("San Martín 123".into()),
                observaciones: None,
            },
        )
        .await
        .expect("actualizar");

        assert_eq!(actualizado.telefono.as_deref(), Some("3794222222"));
        assert_eq!(actualizado.patente.as_deref(), Some("XYZ789"));
        assert_eq!(actualizado.direccion.as_deref(), Some("San Martín 123"));

        // Se releyó de la base, no es solo lo que se mandó -- confirma que
        // el UPDATE realmente pegó.
        let releido = obtener(&pool, creado.id).await.unwrap();
        assert_eq!(releido.telefono.as_deref(), Some("3794222222"));
    }

    #[tokio::test]
    async fn no_se_puede_editar_consumidor_final() {
        let pool = pool_de_prueba().await;
        let resultado = actualizar(
            &pool,
            ID_CONSUMIDOR_FINAL,
            GuardarCliente {
                nombre: "Otro nombre".into(),
                telefono: None,
                patente: None,
                direccion: None,
                observaciones: None,
            },
        )
        .await;
        assert!(matches!(resultado, Err(AppError::Validation(_))));

        // Y la fila reservada sigue intacta.
        let consumidor_final = obtener(&pool, ID_CONSUMIDOR_FINAL).await.unwrap();
        assert_eq!(consumidor_final.nombre, "Consumidor final");
    }

    #[tokio::test]
    async fn buscar_encuentra_por_telefono() {
        let pool = pool_de_prueba().await;
        crear(
            &pool,
            GuardarCliente {
                nombre: "Roberto Silva".into(),
                telefono: Some("3794555555".into()),
                patente: None,
                direccion: None,
                observaciones: None,
            },
        )
        .await
        .expect("crear");

        let resultado = buscar(&pool, "555555").await.expect("buscar");
        assert_eq!(resultado.len(), 1);
        assert_eq!(resultado[0].nombre, "Roberto Silva");

        let sin_resultado = buscar(&pool, "999999").await.expect("buscar");
        assert!(sin_resultado.is_empty());
    }
}
