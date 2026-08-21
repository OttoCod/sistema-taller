-- Fase 2: catálogo de productos. Los campos de stock existen en la tabla
-- pero no se editan todavía (llegan con stock_movimientos en la Fase 4);
-- la relación con proveedores llega en la Fase 8.

CREATE TABLE marcas (
    id INTEGER PRIMARY KEY,
    nombre TEXT NOT NULL UNIQUE
) STRICT;

CREATE TABLE categorias (
    id INTEGER PRIMARY KEY,
    nombre TEXT NOT NULL,
    categoria_padre_id INTEGER REFERENCES categorias (id)
) STRICT;

CREATE TABLE productos (
    id INTEGER PRIMARY KEY,
    codigo_interno TEXT UNIQUE NOT NULL,
    nombre TEXT NOT NULL,
    marca_id INTEGER REFERENCES marcas (id),
    categoria_id INTEGER REFERENCES categorias (id),
    descripcion TEXT,
    observaciones TEXT,
    imagen_path TEXT,
    stock_actual INTEGER NOT NULL DEFAULT 0,
    stock_minimo INTEGER NOT NULL DEFAULT 0,
    costo_actual INTEGER,
    precio_venta_actual INTEGER,
    precio_publico_referencia INTEGER,
    precio_actualizado_en TEXT,
    estado TEXT NOT NULL DEFAULT 'activo',
    fusionado_en_id INTEGER REFERENCES productos (id),
    creado_en TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) STRICT;

CREATE INDEX idx_productos_marca ON productos (marca_id);
CREATE INDEX idx_productos_categoria ON productos (categoria_id);
CREATE INDEX idx_productos_estado ON productos (estado);

CREATE TABLE producto_codigos_fabricante (
    id INTEGER PRIMARY KEY,
    producto_id INTEGER NOT NULL REFERENCES productos (id),
    codigo TEXT NOT NULL,
    fabricante_nombre TEXT,
    observacion TEXT
) STRICT;

CREATE INDEX idx_codigos_fabricante_producto ON producto_codigos_fabricante (producto_id);

CREATE TABLE precios_historial (
    id INTEGER PRIMARY KEY,
    producto_id INTEGER NOT NULL REFERENCES productos (id),
    tipo TEXT NOT NULL,
    valor INTEGER NOT NULL,
    fecha TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    observacion TEXT
) STRICT;

CREATE INDEX idx_precios_historial_producto ON precios_historial (producto_id, fecha);

-- Búsqueda: tabla FTS5 independiente (no "external content"), sincronizada
-- explícitamente desde services/productos.rs en la misma transacción que
-- crea/edita un producto -- no con triggers de SQLite. El tokenizador
-- unicode61 con remove_diacritics resuelve mayúsculas/acentos solo.
CREATE VIRTUAL TABLE productos_fts USING fts5 (
    nombre,
    codigo_interno,
    marca,
    categoria,
    codigos_fabricante,
    tokenize = "unicode61 remove_diacritics 2"
);
