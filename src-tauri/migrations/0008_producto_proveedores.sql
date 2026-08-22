-- Fase 8: gestión completa de proveedores y vínculo producto-proveedor.

CREATE TABLE producto_proveedores (
    id INTEGER PRIMARY KEY,
    producto_id INTEGER NOT NULL REFERENCES productos (id),
    proveedor_id INTEGER NOT NULL REFERENCES proveedores (id),
    codigo_proveedor TEXT,
    url_producto TEXT,
    url_busqueda TEXT,
    es_principal INTEGER NOT NULL DEFAULT 0,
    activo INTEGER NOT NULL DEFAULT 1,
    UNIQUE (producto_id, proveedor_id)
) STRICT;

CREATE INDEX idx_producto_proveedores_proveedor ON producto_proveedores (proveedor_id);
CREATE INDEX idx_producto_proveedores_producto ON producto_proveedores (producto_id);
