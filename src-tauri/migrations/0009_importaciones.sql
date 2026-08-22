-- Fase 3: importación de listas de precios desde Excel.
--
-- codigo_legado guarda el código que traía el archivo original, solo para
-- trazabilidad -- nunca es el ID interno ni tiene UNIQUE, porque el Excel
-- real tiene códigos repetidos apuntando a productos distintos. Si en el
-- futuro hace falta guardar varios códigos históricos/equivalentes por
-- producto, se generaliza con una tabla propia (mismo patrón que
-- producto_codigos_fabricante); por ahora un solo campo alcanza.
ALTER TABLE productos ADD COLUMN codigo_legado TEXT;
CREATE INDEX idx_productos_codigo_legado ON productos (codigo_legado);

CREATE TABLE importaciones (
    id INTEGER PRIMARY KEY,
    archivo_nombre TEXT NOT NULL,
    -- sha256 hex del archivo leído, para avisar si ya se importó antes.
    archivo_hash TEXT NOT NULL,
    -- La hoja/fila/columnas se detectan dinámicamente (nunca se asume una
    -- posición fija) y quedan documentadas acá para que el usuario pueda
    -- ver qué se detectó.
    hoja_detectada TEXT NOT NULL,
    fila_encabezado_detectada INTEGER NOT NULL,
    columnas_detectadas_json TEXT NOT NULL,
    estado TEXT NOT NULL DEFAULT 'en_revision' CHECK (estado IN ('en_revision', 'confirmada', 'descartada')),
    total_filas INTEGER NOT NULL,
    creada_en TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    cerrada_en TEXT
) STRICT;

CREATE TABLE importacion_filas (
    id INTEGER PRIMARY KEY,
    importacion_id INTEGER NOT NULL REFERENCES importaciones (id),
    -- Número de fila real en el Excel, para poder volver a ubicarla ahí.
    fila_excel INTEGER NOT NULL,
    codigo_excel TEXT,
    nombre_excel TEXT,
    precio_lista_centavos INTEGER,
    -- Título de sección vigente para esta fila (si el Excel tenía uno
    -- funcionando como agrupador justo antes). Se usa para crear/asociar
    -- una categoría al confirmar, nunca se inventa una si no había.
    categoria_excel_texto TEXT,
    -- Snapshot de las columnas de porcentaje (pp+50%, etc.), solo como
    -- referencia histórica de auditoría -- nunca se vuelca a producto.
    precios_calculados_json TEXT,
    clasificacion TEXT NOT NULL CHECK (
        clasificacion IN ('producto_valido', 'seccion', 'ignorada', 'requiere_revision', 'error')
    ),
    motivo_error TEXT,
    -- Duplicado DENTRO de este mismo archivo (no contra el catálogo).
    es_duplicado_codigo INTEGER NOT NULL DEFAULT 0,
    es_posible_duplicado_nombre INTEGER NOT NULL DEFAULT 0,
    -- Coincidencia contra un producto YA existente (por codigo_legado),
    -- de otra importación anterior o carga manual. Solo sugerencia.
    coincide_producto_existente_id INTEGER REFERENCES productos (id),
    -- NULL = todavía sin resolver. Nunca se aplica nada solo.
    decision TEXT CHECK (decision IN ('crear_nuevo', 'vincular_existente', 'omitir')),
    producto_vinculado_id INTEGER REFERENCES productos (id),
    -- Solo aplica si decision = vincular_existente: si el usuario pidió
    -- explícitamente pisar el costo_actual del producto con el valor del
    -- Excel (nunca es automático -- ver services/importaciones.rs).
    actualizar_costo_en_vinculo INTEGER NOT NULL DEFAULT 0,
    -- Resultado final una vez aplicada la decisión.
    producto_id INTEGER REFERENCES productos (id),
    resuelta_en TEXT
) STRICT;

CREATE INDEX idx_importacion_filas_importacion ON importacion_filas (importacion_id);
CREATE INDEX idx_importacion_filas_codigo ON importacion_filas (importacion_id, codigo_excel);
