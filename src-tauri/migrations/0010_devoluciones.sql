-- Fase 11: anulaciones y devoluciones de ventas.
--
-- La anulación de venta NO necesita migración: ventas.estado ya admite
-- 'anulada' y motivo_anulacion/fecha_anulacion ya existen desde la
-- migración 0006 (se dejaron previstos a propósito). Acá solo se agregan
-- las tablas de devolución parcial/total, que no existían todavía.
--
-- Criterio (ESQUEMA_BD.md, punto E): anulación = la venta completa fue un
-- error y se cancela entera; devolución = la venta fue válida y el
-- cliente trae de vuelta uno o más productos después. Ninguna borra la
-- venta original.

CREATE TABLE devoluciones (
    id INTEGER PRIMARY KEY,
    venta_id INTEGER NOT NULL REFERENCES ventas (id),
    fecha TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    motivo TEXT NOT NULL,
    metodo_devolucion TEXT NOT NULL CHECK (
        metodo_devolucion IN ('reembolso_efectivo', 'nota_credito', 'cambio_producto', 'reduccion_deuda')
    ),
    -- Centavos. Suma de devolucion_detalles de esta devolución.
    total_devuelto INTEGER NOT NULL
) STRICT;

CREATE INDEX idx_devoluciones_venta ON devoluciones (venta_id);

CREATE TABLE devolucion_detalles (
    id INTEGER PRIMARY KEY,
    devolucion_id INTEGER NOT NULL REFERENCES devoluciones (id),
    venta_detalle_id INTEGER NOT NULL REFERENCES venta_detalles (id),
    cantidad INTEGER NOT NULL,
    -- Centavos. Proporcional al precio efectivo de esa línea (ya con su
    -- descuento aplicado), no un precio recalculado de cero.
    monto INTEGER NOT NULL,
    estado_producto TEXT NOT NULL CHECK (
        estado_producto IN ('vuelve_a_stock', 'en_revision', 'defectuoso', 'dañado')
    ),
    observacion TEXT
) STRICT;

CREATE INDEX idx_devolucion_detalles_devolucion ON devolucion_detalles (devolucion_id);
CREATE INDEX idx_devolucion_detalles_venta_detalle ON devolucion_detalles (venta_detalle_id);
