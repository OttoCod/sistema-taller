//! Lectura de archivos Excel para la Fase 3 (importación). Este módulo es
//! deliberadamente puro: no toca la base de datos ni clasifica filas, solo
//! sabe abrir un archivo EN MODO LECTURA (calamine nunca escribe) y
//! devolver hechos crudos. La clasificación/normalización vive en
//! `services::importaciones`.
//!
//! Nada acá asume una hoja, fila de encabezado o columna fija: todo se
//! detecta por el texto de los encabezados, porque un archivo real puede
//! cambiar de estructura de una entrega a la siguiente.

use std::path::Path;

use calamine::{open_workbook_auto, Data, DataType, Range, Reader};

use crate::error::{AppError, AppResult};
use crate::models::importacion::{
    ColumnaDetectada, ColumnasDetectadas, EstructuraDetectada, FilaCruda,
};

const PALABRAS_CODIGO: &[&str] = &["codigo"];
// En orden de especificidad: la primera que matchee gana.
const PALABRAS_NOMBRE: &[&str] = &[
    "mercaderia",
    "descripcion",
    "nombre",
    "producto",
    "detalle",
    "articulo",
];
const PALABRAS_PRECIO_FUERTE: &[&str] = &["precio lista", "precio base"];
const PALABRAS_PRECIO_DEBIL: &[&str] = &["costo", "precio"];

fn normalizar_texto(s: &str) -> String {
    s.trim()
        .to_lowercase()
        .chars()
        .map(|c| match c {
            'á' => 'a',
            'é' => 'e',
            'í' => 'i',
            'ó' => 'o',
            'ú' => 'u',
            'ü' => 'u',
            'ñ' => 'n',
            other => other,
        })
        .collect()
}

fn es_columna_derivada(encabezado_normalizado: &str) -> bool {
    encabezado_normalizado.contains('%') || encabezado_normalizado.contains("pp+")
}

fn texto_celda(celda: &Data) -> Option<String> {
    if celda.is_empty() {
        return None;
    }
    let texto = celda.as_string().unwrap_or_default();
    let limpio = texto.trim();
    if limpio.is_empty() {
        None
    } else {
        Some(limpio.to_string())
    }
}

/// Intenta interpretar una fila como encabezado: busca, entre sus celdas,
/// una columna de nombre y una de precio base (obligatorias) y una de
/// código (opcional). Devuelve `None` si no encuentra ambas obligatorias.
fn intentar_detectar_columnas(fila: &[Data]) -> Option<ColumnasDetectadas> {
    let mut codigo: Option<ColumnaDetectada> = None;
    let mut nombre: Option<(usize, ColumnaDetectada)> = None; // (prioridad, columna)
    let mut precio: Option<(usize, ColumnaDetectada)> = None; // (prioridad, columna)
    let mut derivadas = Vec::new();

    for (indice, celda) in fila.iter().enumerate() {
        let Some(encabezado) = texto_celda(celda) else {
            continue;
        };
        let normalizado = normalizar_texto(&encabezado);

        if es_columna_derivada(&normalizado) {
            derivadas.push(ColumnaDetectada { encabezado, indice });
            continue;
        }

        if codigo.is_none() && PALABRAS_CODIGO.iter().any(|p| normalizado.contains(p)) {
            codigo = Some(ColumnaDetectada {
                encabezado: encabezado.clone(),
                indice,
            });
        }

        if let Some(prioridad) = PALABRAS_NOMBRE.iter().position(|p| normalizado.contains(p)) {
            if nombre.as_ref().is_none_or(|(mejor, _)| prioridad < *mejor) {
                nombre = Some((
                    prioridad,
                    ColumnaDetectada {
                        encabezado: encabezado.clone(),
                        indice,
                    },
                ));
            }
        }

        let prioridad_precio = if PALABRAS_PRECIO_FUERTE
            .iter()
            .any(|p| normalizado.contains(p))
        {
            Some(0)
        } else if PALABRAS_PRECIO_DEBIL
            .iter()
            .any(|p| normalizado.contains(p))
        {
            Some(1)
        } else {
            None
        };
        if let Some(prioridad) = prioridad_precio {
            if precio.as_ref().is_none_or(|(mejor, _)| prioridad < *mejor) {
                precio = Some((prioridad, ColumnaDetectada { encabezado, indice }));
            }
        }
    }

    Some(ColumnasDetectadas {
        codigo,
        nombre: nombre?.1,
        precio: precio?.1,
        derivadas,
    })
}

/// Devuelve `true` si, con las columnas ya detectadas, esta fila parece
/// una fila de datos real (nombre o precio con algo cargado) -- se usa
/// solo para contar cuántas filas de datos hay debajo de un candidato a
/// encabezado, y así desempatar entre hojas/filas candidatas.
fn parece_fila_de_datos(fila: &[Data], columnas: &ColumnasDetectadas) -> bool {
    let tiene_nombre = fila
        .get(columnas.nombre.indice)
        .is_some_and(|c| !c.is_empty());
    let tiene_precio = fila
        .get(columnas.precio.indice)
        .is_some_and(|c| !c.is_empty());
    tiene_nombre || tiene_precio
}

/// Recorre todas las hojas del archivo buscando, entre las primeras filas
/// de cada una, una que sirva de encabezado (con columnas de nombre y
/// precio reconocibles). Entre todos los candidatos se queda con el que
/// tenga más filas de datos reales debajo -- así una coincidencia
/// accidental de palabras en una hoja vacía no le gana a la hoja real.
///
/// Si ningún candidato tiene datos reales debajo, no asume nada: devuelve
/// un error para que el usuario revise el archivo.
fn detectar_estructura(
    workbook: &mut calamine::Sheets<std::io::BufReader<std::fs::File>>,
) -> AppResult<(EstructuraDetectada, Range<Data>)> {
    const FILAS_A_REVISAR: usize = 30;

    let mut mejor: Option<(String, u32, ColumnasDetectadas, usize, Range<Data>)> = None;

    for nombre_hoja in workbook.sheet_names().to_owned() {
        let Ok(rango) = workbook.worksheet_range(&nombre_hoja) else {
            continue; // hojas de gráficos u otras no legibles como tabla
        };
        let Some((fila_inicio, _)) = rango.start() else {
            continue; // hoja vacía
        };

        let filas: Vec<&[Data]> = rango.rows().collect();
        for (indice_relativo, fila) in filas.iter().enumerate().take(FILAS_A_REVISAR) {
            let Some(columnas) = intentar_detectar_columnas(fila) else {
                continue;
            };
            let filas_de_datos = filas
                .iter()
                .skip(indice_relativo + 1)
                .filter(|f| parece_fila_de_datos(f, &columnas))
                .count();
            if filas_de_datos == 0 {
                continue;
            }

            let es_mejor = mejor
                .as_ref()
                .is_none_or(|(_, _, _, mejor_cantidad, _)| filas_de_datos > *mejor_cantidad);
            if es_mejor {
                let fila_encabezado_real = fila_inicio + indice_relativo as u32;
                mejor = Some((
                    nombre_hoja.clone(),
                    fila_encabezado_real,
                    columnas,
                    filas_de_datos,
                    rango.clone(),
                ));
            }
        }
    }

    let (hoja, fila_encabezado_0based, columnas, _, rango) = mejor.ok_or_else(|| {
        AppError::Validation(
            "No se pudo detectar la estructura del archivo: ninguna hoja tiene columnas reconocibles de nombre/precio con datos debajo. Revisá que el Excel tenga un encabezado con esas columnas.".into(),
        )
    })?;

    Ok((
        EstructuraDetectada {
            hoja,
            fila_encabezado: fila_encabezado_0based as i64 + 1,
            columnas,
        },
        rango,
    ))
}

fn extraer_filas(rango: &Range<Data>, estructura: &EstructuraDetectada) -> Vec<FilaCruda> {
    let fila_inicio_absoluta = rango.start().map(|(r, _)| r).unwrap_or(0);
    let columnas = &estructura.columnas;

    rango
        .rows()
        .enumerate()
        // La fila de encabezado (y todo lo anterior) no es un dato.
        .filter(|(indice, _)| {
            fila_inicio_absoluta as i64 + *indice as i64 + 1 > estructura.fila_encabezado
        })
        .map(|(indice, fila)| {
            let fila_excel = fila_inicio_absoluta as i64 + indice as i64 + 1;

            let codigo = columnas
                .codigo
                .as_ref()
                .and_then(|c| fila.get(c.indice))
                .and_then(texto_celda);

            let nombre = fila.get(columnas.nombre.indice).and_then(texto_celda);

            let celda_precio = fila.get(columnas.precio.indice);
            let (precio_centavos, precio_texto_invalido) = match celda_precio {
                None => (None, None),
                Some(c) if c.is_empty() => (None, None),
                Some(c) => match c.as_f64() {
                    Some(valor) => (Some((valor * 100.0).round() as i64), None),
                    None => (None, texto_celda(c)),
                },
            };

            let precios_derivados = columnas
                .derivadas
                .iter()
                .filter_map(|d| {
                    let celda = fila.get(d.indice)?;
                    let valor = celda.as_f64()?;
                    Some((d.encabezado.clone(), valor))
                })
                .collect();

            FilaCruda {
                fila_excel,
                codigo,
                nombre,
                precio_centavos,
                precio_texto_invalido,
                precios_derivados,
            }
        })
        .collect()
}

/// Punto de entrada del módulo: abre el archivo (solo lectura), detecta su
/// estructura y devuelve todas las filas de datos ya extraídas (sin
/// clasificar todavía).
pub fn leer_archivo(ruta: &Path) -> AppResult<(EstructuraDetectada, Vec<FilaCruda>)> {
    let mut workbook = open_workbook_auto(ruta)
        .map_err(|e| AppError::Validation(format!("No se pudo abrir el archivo: {e}")))?;

    let (estructura, rango) = detectar_estructura(&mut workbook)?;
    let filas = extraer_filas(&rango, &estructura);

    Ok((estructura, filas))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture(nombre: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(nombre)
    }

    #[test]
    fn detecta_estructura_dinamica_y_lee_filas() {
        let (estructura, filas) =
            leer_archivo(&fixture("lista_precios_ok.xlsx")).expect("leer archivo");

        assert_eq!(estructura.hoja, "Hoja3");
        assert_eq!(estructura.fila_encabezado, 6);
        assert_eq!(
            estructura.columnas.codigo.as_ref().unwrap().encabezado,
            "codigo"
        );
        assert_eq!(estructura.columnas.nombre.encabezado, "mercaderia");
        assert_eq!(estructura.columnas.precio.encabezado, "precio lista");
        assert!(estructura.columnas.derivadas.len() >= 2);

        // La primera fila de datos real.
        let primera = filas
            .iter()
            .find(|f| f.codigo.as_deref() == Some("54"))
            .unwrap();
        assert_eq!(primera.nombre.as_deref(), Some("disco 5 pieza embrague"));
        assert_eq!(primera.precio_centavos, Some(174_200));
    }

    #[test]
    fn ignora_hojas_de_grafico_y_hojas_vacias() {
        // El fixture tiene, además de la hoja con datos, una hoja vacía y
        // no debería explotar ni elegir la vacía.
        let (estructura, _filas) =
            leer_archivo(&fixture("lista_precios_ok.xlsx")).expect("leer archivo");
        assert_eq!(estructura.hoja, "Hoja3");
    }

    #[test]
    fn archivo_sin_columnas_reconocibles_falla_en_vez_de_asumir() {
        let resultado = leer_archivo(&fixture("sin_estructura_reconocible.xlsx"));
        assert!(matches!(resultado, Err(AppError::Validation(_))));
    }

    #[test]
    fn codigo_alfanumerico_y_precio_con_decimales_se_leen_bien() {
        let (_estructura, filas) =
            leer_archivo(&fixture("lista_precios_ok.xlsx")).expect("leer archivo");
        let fila = filas
            .iter()
            .find(|f| f.codigo.as_deref() == Some("b201"))
            .unwrap();
        assert_eq!(fila.nombre.as_deref(), Some("separador de disco"));

        let con_decimales = filas
            .iter()
            .find(|f| f.nombre.as_deref() == Some("con decimales"))
            .unwrap();
        assert_eq!(con_decimales.precio_centavos, Some(29521));
    }

    #[test]
    fn precio_con_texto_invalido_se_marca_sin_romper_la_lectura() {
        let (_estructura, filas) =
            leer_archivo(&fixture("lista_precios_ok.xlsx")).expect("leer archivo");
        let fila = filas
            .iter()
            .find(|f| f.nombre.as_deref() == Some("precio roto"))
            .unwrap();
        assert_eq!(fila.precio_centavos, None);
        assert_eq!(
            fila.precio_texto_invalido.as_deref(),
            Some("no es un precio")
        );
    }
}
