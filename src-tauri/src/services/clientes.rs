use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};
use crate::models::cliente::{Cliente, ClienteConDeuda, GuardarCliente, ID_CONSUMIDOR_FINAL};

const SELECT_CLIENTE: &str = "
    SELECT id, nombre, telefono, dni_cuit, direccion, observaciones, saldo_cuenta_corriente
    FROM clientes
";

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
         WHERE nombre LIKE ? OR telefono LIKE ? OR dni_cuit LIKE ?
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
        "INSERT INTO clientes (nombre, telefono, dni_cuit, direccion, observaciones)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(nombre)
    .bind(&datos.telefono)
    .bind(&datos.dni_cuit)
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
        "UPDATE clientes SET nombre = ?, telefono = ?, dni_cuit = ?, direccion = ?, observaciones = ?
         WHERE id = ?",
    )
    .bind(nombre)
    .bind(&datos.telefono)
    .bind(&datos.dni_cuit)
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
