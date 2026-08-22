use sqlx::{Sqlite, SqlitePool, Transaction};

use crate::error::{AppError, AppResult};
use crate::models::producto::{CodigoFabricante, GuardarProducto, Producto, ProductoDetalle};

const SELECT_PRODUCTO: &str = "
    SELECT
        p.id, p.codigo_interno, p.nombre,
        p.marca_id, m.nombre AS marca_nombre,
        p.categoria_id, c.nombre AS categoria_nombre,
        p.descripcion, p.observaciones,
        p.costo_actual, p.precio_venta_actual, p.precio_publico_referencia,
        p.precio_actualizado_en, p.estado, p.stock_actual,
        cf.codigos AS codigos_fabricante_resumen, p.codigo_legado
    FROM productos p
    LEFT JOIN marcas m ON m.id = p.marca_id
    LEFT JOIN categorias c ON c.id = p.categoria_id
    LEFT JOIN (
        SELECT producto_id, GROUP_CONCAT(codigo, ', ') AS codigos
        FROM producto_codigos_fabricante
        GROUP BY producto_id
    ) cf ON cf.producto_id = p.id
";

pub async fn listar(pool: &SqlitePool) -> AppResult<Vec<Producto>> {
    let productos = sqlx::query_as::<_, Producto>(&format!(
        "{SELECT_PRODUCTO} WHERE p.estado != 'fusionado' ORDER BY p.nombre LIMIT 500"
    ))
    .fetch_all(pool)
    .await?;
    Ok(productos)
}

pub async fn obtener(pool: &SqlitePool, id: i64) -> AppResult<ProductoDetalle> {
    let producto = obtener_producto(pool, id).await?;
    let codigos_fabricante = obtener_codigos(pool, id).await?;
    Ok(ProductoDetalle {
        producto,
        codigos_fabricante,
    })
}

async fn obtener_producto(pool: &SqlitePool, id: i64) -> AppResult<Producto> {
    sqlx::query_as::<_, Producto>(&format!("{SELECT_PRODUCTO} WHERE p.id = ?"))
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("No existe el producto {id}.")))
}

async fn obtener_codigos(pool: &SqlitePool, producto_id: i64) -> AppResult<Vec<CodigoFabricante>> {
    let codigos = sqlx::query_as::<_, CodigoFabricante>(
        "SELECT codigo, fabricante_nombre, observacion
         FROM producto_codigos_fabricante WHERE producto_id = ? ORDER BY id",
    )
    .bind(producto_id)
    .fetch_all(pool)
    .await?;
    Ok(codigos)
}

fn validar(datos: &GuardarProducto) -> AppResult<()> {
    if datos.nombre.trim().is_empty() {
        return Err(AppError::Validation(
            "El nombre del producto no puede estar vacío.".into(),
        ));
    }
    if !matches!(datos.estado.as_str(), "activo" | "inactivo") {
        return Err(AppError::Validation(format!(
            "Estado inválido: {}",
            datos.estado
        )));
    }
    Ok(())
}

pub async fn crear(pool: &SqlitePool, datos: GuardarProducto) -> AppResult<ProductoDetalle> {
    validar(&datos)?;

    let tiene_precio = datos.costo_actual.is_some()
        || datos.precio_venta_actual.is_some()
        || datos.precio_publico_referencia.is_some();

    let mut tx = pool.begin().await?;

    let id = sqlx::query(
        "INSERT INTO productos (
            codigo_interno, nombre, marca_id, categoria_id, descripcion, observaciones,
            costo_actual, precio_venta_actual, precio_publico_referencia,
            precio_actualizado_en, estado
        ) VALUES ('', ?, ?, ?, ?, ?, ?, ?, ?, CASE WHEN ? THEN strftime('%Y-%m-%dT%H:%M:%fZ','now') END, ?)",
    )
    .bind(datos.nombre.trim())
    .bind(datos.marca_id)
    .bind(datos.categoria_id)
    .bind(&datos.descripcion)
    .bind(&datos.observaciones)
    .bind(datos.costo_actual)
    .bind(datos.precio_venta_actual)
    .bind(datos.precio_publico_referencia)
    .bind(tiene_precio)
    .bind(&datos.estado)
    .execute(&mut *tx)
    .await?
    .last_insert_rowid();

    // El código interno se deriva del id autoincremental: simple,
    // determinístico y único por construcción (ver docs/ARQUITECTURA.md).
    let codigo_interno = format!("P-{id:06}");
    sqlx::query("UPDATE productos SET codigo_interno = ? WHERE id = ?")
        .bind(&codigo_interno)
        .bind(id)
        .execute(&mut *tx)
        .await?;

    for (tipo, valor) in precios_iniciales(&datos) {
        registrar_precio(&mut tx, id, tipo, valor).await?;
    }

    guardar_codigos(&mut tx, id, &datos.codigos_fabricante).await?;
    sincronizar_fts(&mut tx, id).await?;

    tx.commit().await?;

    obtener(pool, id).await
}

pub async fn actualizar(
    pool: &SqlitePool,
    id: i64,
    datos: GuardarProducto,
) -> AppResult<ProductoDetalle> {
    validar(&datos)?;

    let anterior = obtener_producto(pool, id).await?;

    let cambia_precio = anterior.costo_actual != datos.costo_actual
        || anterior.precio_venta_actual != datos.precio_venta_actual
        || anterior.precio_publico_referencia != datos.precio_publico_referencia;

    let mut tx = pool.begin().await?;

    sqlx::query(
        "UPDATE productos SET
            nombre = ?, marca_id = ?, categoria_id = ?, descripcion = ?, observaciones = ?,
            costo_actual = ?, precio_venta_actual = ?, precio_publico_referencia = ?,
            precio_actualizado_en = CASE WHEN ? THEN strftime('%Y-%m-%dT%H:%M:%fZ','now')
                                          ELSE precio_actualizado_en END,
            estado = ?
        WHERE id = ?",
    )
    .bind(datos.nombre.trim())
    .bind(datos.marca_id)
    .bind(datos.categoria_id)
    .bind(&datos.descripcion)
    .bind(&datos.observaciones)
    .bind(datos.costo_actual)
    .bind(datos.precio_venta_actual)
    .bind(datos.precio_publico_referencia)
    .bind(cambia_precio)
    .bind(&datos.estado)
    .bind(id)
    .execute(&mut *tx)
    .await?;

    // Solo queda historial de los campos que realmente cambiaron, no de
    // todo el formulario en cada guardado.
    if anterior.costo_actual != datos.costo_actual {
        if let Some(valor) = datos.costo_actual {
            registrar_precio(&mut tx, id, "costo", valor).await?;
        }
    }
    if anterior.precio_venta_actual != datos.precio_venta_actual {
        if let Some(valor) = datos.precio_venta_actual {
            registrar_precio(&mut tx, id, "venta", valor).await?;
        }
    }
    if anterior.precio_publico_referencia != datos.precio_publico_referencia {
        if let Some(valor) = datos.precio_publico_referencia {
            registrar_precio(&mut tx, id, "publico_referencia", valor).await?;
        }
    }

    sqlx::query("DELETE FROM producto_codigos_fabricante WHERE producto_id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    guardar_codigos(&mut tx, id, &datos.codigos_fabricante).await?;
    sincronizar_fts(&mut tx, id).await?;

    tx.commit().await?;

    obtener(pool, id).await
}

pub async fn buscar(pool: &SqlitePool, consulta: &str) -> AppResult<Vec<Producto>> {
    let Some(consulta_fts) = construir_consulta_fts(consulta) else {
        return listar(pool).await;
    };

    let productos = sqlx::query_as::<_, Producto>(&format!(
        "{SELECT_PRODUCTO}
         JOIN (SELECT rowid, rank FROM productos_fts WHERE productos_fts MATCH ?) f
            ON f.rowid = p.id
         WHERE p.estado != 'fusionado'
         ORDER BY f.rank
         LIMIT 50"
    ))
    .bind(consulta_fts)
    .fetch_all(pool)
    .await?;

    Ok(productos)
}

/// Cada palabra se busca como prefijo, entre comillas para que caracteres
/// especiales de la sintaxis de FTS5 (:, -, (, ) ...) no rompan la
/// consulta. El AND implícito entre términos separados por espacio hace
/// que el orden de las palabras no importe.
fn construir_consulta_fts(consulta: &str) -> Option<String> {
    let terminos: Vec<String> = consulta
        .split_whitespace()
        .map(|palabra| palabra.replace('"', ""))
        .filter(|palabra| !palabra.is_empty())
        .map(|palabra| format!("\"{palabra}\"*"))
        .collect();
    if terminos.is_empty() {
        None
    } else {
        Some(terminos.join(" "))
    }
}

fn precios_iniciales(datos: &GuardarProducto) -> Vec<(&'static str, i64)> {
    let mut precios = Vec::new();
    if let Some(valor) = datos.costo_actual {
        precios.push(("costo", valor));
    }
    if let Some(valor) = datos.precio_venta_actual {
        precios.push(("venta", valor));
    }
    if let Some(valor) = datos.precio_publico_referencia {
        precios.push(("publico_referencia", valor));
    }
    precios
}

pub(crate) async fn registrar_precio(
    tx: &mut Transaction<'_, Sqlite>,
    producto_id: i64,
    tipo: &str,
    valor: i64,
) -> AppResult<()> {
    sqlx::query("INSERT INTO precios_historial (producto_id, tipo, valor) VALUES (?, ?, ?)")
        .bind(producto_id)
        .bind(tipo)
        .bind(valor)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

async fn guardar_codigos(
    tx: &mut Transaction<'_, Sqlite>,
    producto_id: i64,
    codigos: &[CodigoFabricante],
) -> AppResult<()> {
    for codigo in codigos {
        let texto = codigo.codigo.trim();
        if texto.is_empty() {
            continue;
        }
        sqlx::query(
            "INSERT INTO producto_codigos_fabricante (producto_id, codigo, fabricante_nombre, observacion)
             VALUES (?, ?, ?, ?)",
        )
        .bind(producto_id)
        .bind(texto)
        .bind(&codigo.fabricante_nombre)
        .bind(&codigo.observacion)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

/// Reconstruye la fila de búsqueda del producto a partir del estado actual
/// de la base (nombre, código, marca, categoría, códigos de fabricante) y
/// la reemplaza en productos_fts. Se llama siempre dentro de la misma
/// transacción que crea/edita el producto -- ver la decisión en
/// docs/ARQUITECTURA.md sobre por qué no se usan triggers de SQLite.
async fn sincronizar_fts(tx: &mut Transaction<'_, Sqlite>, producto_id: i64) -> AppResult<()> {
    let (nombre, codigo_interno, marca, categoria): (
        String,
        String,
        Option<String>,
        Option<String>,
    ) = sqlx::query_as(
        "SELECT p.nombre, p.codigo_interno, m.nombre, c.nombre
         FROM productos p
         LEFT JOIN marcas m ON m.id = p.marca_id
         LEFT JOIN categorias c ON c.id = p.categoria_id
         WHERE p.id = ?",
    )
    .bind(producto_id)
    .fetch_one(&mut **tx)
    .await?;

    let codigos: Vec<String> =
        sqlx::query_scalar("SELECT codigo FROM producto_codigos_fabricante WHERE producto_id = ?")
            .bind(producto_id)
            .fetch_all(&mut **tx)
            .await?;

    sqlx::query("DELETE FROM productos_fts WHERE rowid = ?")
        .bind(producto_id)
        .execute(&mut **tx)
        .await?;

    sqlx::query(
        "INSERT INTO productos_fts (rowid, nombre, codigo_interno, marca, categoria, codigos_fabricante)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(producto_id)
    .bind(nombre)
    .bind(codigo_interno)
    .bind(marca.unwrap_or_default())
    .bind(categoria.unwrap_or_default())
    .bind(codigos.join(" "))
    .execute(&mut **tx)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::marca::NuevaMarca;
    use crate::models::producto::CodigoFabricante;
    use crate::services::marcas;

    async fn pool_de_prueba() -> SqlitePool {
        let dir = tempfile::tempdir().expect("tempdir");
        // Se filtra a propósito: el pool vive lo que dura el test, y
        // borrar el directorio temporal antes de tiempo lo invalidaría.
        let dir = Box::leak(Box::new(dir));
        crate::db::init_pool(dir.path()).await.expect("init_pool")
    }

    fn producto_minimo(nombre: &str) -> GuardarProducto {
        GuardarProducto {
            nombre: nombre.to_string(),
            marca_id: None,
            categoria_id: None,
            descripcion: None,
            observaciones: None,
            costo_actual: None,
            precio_venta_actual: None,
            precio_publico_referencia: None,
            estado: "activo".to_string(),
            codigos_fabricante: vec![],
        }
    }

    #[tokio::test]
    async fn crear_genera_codigo_interno_y_registra_precio_inicial() {
        let pool = pool_de_prueba().await;

        let mut datos = producto_minimo("Bujía NGK C7HSA");
        datos.costo_actual = Some(500_000); // $5.000
        datos.precio_venta_actual = Some(800_000); // $8.000

        let detalle = crear(&pool, datos).await.expect("crear producto");

        assert_eq!(
            detalle.producto.codigo_interno,
            format!("P-{:06}", detalle.producto.id)
        );
        assert_eq!(detalle.producto.costo_actual, Some(500_000));

        let (historial,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM precios_historial WHERE producto_id = ?")
                .bind(detalle.producto.id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(
            historial, 2,
            "debe registrar costo y precio de venta iniciales"
        );
    }

    #[tokio::test]
    async fn actualizar_solo_registra_precio_si_realmente_cambio() {
        let pool = pool_de_prueba().await;

        let mut datos = producto_minimo("Filtro de aceite");
        datos.precio_venta_actual = Some(300_000);
        let creado = crear(&pool, datos).await.expect("crear");

        // Editar sin tocar el precio: no debe agregar historial nuevo.
        let mut sin_cambios = producto_minimo("Filtro de aceite");
        sin_cambios.precio_venta_actual = Some(300_000);
        actualizar(&pool, creado.producto.id, sin_cambios)
            .await
            .expect("actualizar sin cambios");

        let (historial_sin_cambio,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM precios_historial WHERE producto_id = ?")
                .bind(creado.producto.id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(historial_sin_cambio, 1);

        // Ahora sí cambia el precio: debe sumar una fila más.
        let mut con_cambio = producto_minimo("Filtro de aceite");
        con_cambio.precio_venta_actual = Some(350_000);
        actualizar(&pool, creado.producto.id, con_cambio)
            .await
            .expect("actualizar con cambio");

        let (historial_con_cambio,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM precios_historial WHERE producto_id = ?")
                .bind(creado.producto.id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(historial_con_cambio, 2);
    }

    #[tokio::test]
    async fn buscar_tolera_mayusculas_acentos_y_orden_de_palabras() {
        let pool = pool_de_prueba().await;
        let ngk = marcas::crear(
            &pool,
            NuevaMarca {
                nombre: "NGK".into(),
            },
        )
        .await
        .unwrap();

        let mut datos = producto_minimo("Bujía NGK C7HSA");
        datos.marca_id = Some(ngk.id);
        datos.codigos_fabricante = vec![CodigoFabricante {
            codigo: "C7HSA".into(),
            fabricante_nombre: Some("NGK".into()),
            observacion: None,
        }];
        crear(&pool, datos).await.expect("crear producto");

        // Sin acento, minúsculas, orden de palabras invertido.
        let resultados = buscar(&pool, "c7hsa bujia").await.expect("buscar");
        assert_eq!(resultados.len(), 1);
        assert_eq!(resultados[0].nombre, "Bujía NGK C7HSA");
        // El listado tiene que mostrar el código de fabricante, no solo el
        // código interno autogenerado (bug reportado tras probar en Windows).
        assert_eq!(
            resultados[0].codigos_fabricante_resumen.as_deref(),
            Some("C7HSA")
        );

        // Por marca.
        let por_marca = buscar(&pool, "ngk").await.expect("buscar por marca");
        assert_eq!(por_marca.len(), 1);

        // Sin resultados razonables.
        let sin_resultados = buscar(&pool, "cadena de transmision")
            .await
            .expect("buscar sin resultados");
        assert!(sin_resultados.is_empty());
    }

    #[tokio::test]
    async fn buscar_no_devuelve_productos_fusionados() {
        let pool = pool_de_prueba().await;
        let creado = crear(&pool, producto_minimo("Cubierta Pirelli 90/90-18"))
            .await
            .unwrap();

        sqlx::query("UPDATE productos SET estado = 'fusionado' WHERE id = ?")
            .bind(creado.producto.id)
            .execute(&pool)
            .await
            .unwrap();

        let resultados = buscar(&pool, "pirelli").await.expect("buscar");
        assert!(resultados.is_empty());
    }
}
