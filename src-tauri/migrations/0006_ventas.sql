-- Fase 5: ventas. numero se deriva del id (mismo mecanismo que
-- productos.codigo_interno): nunca se reutiliza, es simple y determinístico.
--
-- Todavía no hay anulación (Fase 11) ni comprobantes/impresión (Fase 10) --
-- 'estado' ya contempla 'anulada' para no tener que migrar de nuevo.

CREATE TABLE ventas (
    id INTEGER PRIMARY KEY,
    numero INTEGER NOT NULL UNIQUE,
    cliente_id INTEGER NOT NULL DEFAULT 1 REFERENCES clientes (id),
    fecha TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    estado TEXT NOT NULL DEFAULT 'confirmada' CHECK (estado IN ('confirmada', 'anulada')),
    -- Centavos. subtotal = suma de (precio_unitario * cantidad) sin
    -- descuentos; descuento_total = suma de los descuentos por línea;
    -- total = subtotal - descuento_total.
    subtotal INTEGER NOT NULL,
    descuento_total INTEGER NOT NULL DEFAULT 0,
    total INTEGER NOT NULL,
    motivo_anulacion TEXT,
    fecha_anulacion TEXT
) STRICT;

CREATE INDEX idx_ventas_cliente ON ventas (cliente_id);
CREATE INDEX idx_ventas_fecha ON ventas (fecha);

CREATE TABLE venta_detalles (
    id INTEGER PRIMARY KEY,
    venta_id INTEGER NOT NULL REFERENCES ventas (id),
    producto_id INTEGER NOT NULL REFERENCES productos (id),
    cantidad INTEGER NOT NULL,
    -- El precio aplicado en esta venta -- puede diferir del de referencia
    -- del producto porque siempre es editable a mano.
    precio_unitario INTEGER NOT NULL,
    descuento INTEGER NOT NULL DEFAULT 0,
    subtotal INTEGER NOT NULL,
    -- Copia del costo al momento de vender, para poder calcular el margen
    -- histórico aunque el costo del producto cambie después.
    costo_unitario_snapshot INTEGER
) STRICT;

CREATE INDEX idx_venta_detalles_venta ON venta_detalles (venta_id);

CREATE TABLE venta_pagos (
    id INTEGER PRIMARY KEY,
    venta_id INTEGER NOT NULL REFERENCES ventas (id),
    metodo_pago_id INTEGER NOT NULL REFERENCES metodos_pago (id),
    monto INTEGER NOT NULL,
    fecha TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) STRICT;

CREATE INDEX idx_venta_pagos_venta ON venta_pagos (venta_id);
