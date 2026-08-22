-- Fase 7: compras y recepción de mercadería.
--
-- proveedores se crea acá (aunque conceptualmente la gestión completa --
-- listado, edición, vínculo producto-proveedor, "consultar proveedor" --
-- es Fase 8) porque una compra necesita saber a quién se le compró desde
-- el día uno. Mismo criterio que metodos_pago en la migración 0004.

CREATE TABLE proveedores (
    id INTEGER PRIMARY KEY,
    nombre TEXT NOT NULL,
    telefono TEXT,
    whatsapp TEXT,
    email TEXT,
    sitio_web TEXT,
    observaciones TEXT,
    activo INTEGER NOT NULL DEFAULT 1
) STRICT;

CREATE TABLE compras (
    id INTEGER PRIMARY KEY,
    proveedor_id INTEGER NOT NULL REFERENCES proveedores (id),
    numero_factura TEXT,
    fecha TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    -- Centavos.
    subtotal INTEGER NOT NULL,
    total INTEGER NOT NULL,
    estado TEXT NOT NULL DEFAULT 'registrada' CHECK (estado IN ('registrada', 'anulada'))
) STRICT;

CREATE TABLE compra_detalles (
    id INTEGER PRIMARY KEY,
    compra_id INTEGER NOT NULL REFERENCES compras (id),
    producto_id INTEGER NOT NULL REFERENCES productos (id),
    cantidad INTEGER NOT NULL,
    costo_unitario INTEGER NOT NULL,
    subtotal INTEGER NOT NULL
) STRICT;

CREATE INDEX idx_compras_proveedor ON compras (proveedor_id, fecha);
CREATE INDEX idx_compra_detalles_compra ON compra_detalles (compra_id);
