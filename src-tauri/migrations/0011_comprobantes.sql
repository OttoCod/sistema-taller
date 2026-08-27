-- Fase 10: comprobantes e impresión.
--
-- El enfoque de impresión es vía WebView2 (window.print() sobre una vista
-- HTML pensada para imprimir), no un generador de PDF en Rust -- por eso
-- esta migración no necesita nada más que registrar qué se generó y
-- cuándo se imprimió, para poder mostrar "ya se imprimió antes" y evitar
-- confusiones. `configuracion` (creada en la Fase 1) ya alcanza para los
-- datos del negocio que van en el encabezado del comprobante -- no hace
-- falta ninguna tabla nueva para eso.
--
-- UNIQUE (venta_id, tipo): un solo comprobante por venta y por tipo
-- (ticket / a4). Volver a "generarlo" reutiliza el mismo número; cada
-- impresión deja su propia fila en comprobante_eventos.
CREATE TABLE comprobantes (
    id INTEGER PRIMARY KEY,
    venta_id INTEGER NOT NULL REFERENCES ventas (id),
    numero TEXT NOT NULL,
    tipo TEXT NOT NULL CHECK (tipo IN ('ticket', 'a4')),
    creado_en TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE (venta_id, tipo)
) STRICT;

CREATE INDEX idx_comprobantes_venta ON comprobantes (venta_id);

CREATE TABLE comprobante_eventos (
    id INTEGER PRIMARY KEY,
    comprobante_id INTEGER NOT NULL REFERENCES comprobantes (id),
    tipo_evento TEXT NOT NULL CHECK (tipo_evento IN ('impreso', 'pdf_generado')),
    fecha TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) STRICT;

CREATE INDEX idx_comprobante_eventos_comprobante ON comprobante_eventos (comprobante_id);
