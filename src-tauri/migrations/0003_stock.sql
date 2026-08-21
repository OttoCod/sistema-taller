-- Fase 4: ledger de stock. stock_actual en productos es el cacheado;
-- esta tabla es la fuente de verdad y siempre se puede reconstruir sumando
-- sus filas (ver docs/ESQUEMA_BD.md, "Reglas transversales").
--
-- Hoy el único tipo que se genera es 'ajuste' (Fase 4). 'venta', 'compra',
-- 'devolucion', 'fusion' y 'anulacion' se habilitan cuando lleguen esas
-- fases -- ya están en el CHECK para no tener que migrar de nuevo.

CREATE TABLE stock_movimientos (
    id INTEGER PRIMARY KEY,
    producto_id INTEGER NOT NULL REFERENCES productos (id),
    tipo TEXT NOT NULL CHECK (
        tipo IN ('venta', 'compra', 'devolucion', 'ajuste', 'fusion', 'anulacion')
    ),
    cantidad INTEGER NOT NULL,
    stock_resultante INTEGER NOT NULL,
    referencia_tipo TEXT,
    referencia_id INTEGER,
    fecha TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    observacion TEXT
) STRICT;

CREATE INDEX idx_stock_movimientos_producto ON stock_movimientos (producto_id, fecha);
