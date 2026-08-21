-- Fase 6: clientes y cuenta corriente/fiado.
--
-- metodos_pago se crea acá (aunque conceptualmente es parte de Ventas/Caja,
-- Fases 5 y 9) porque un pago de cuenta corriente ya necesita saber cómo
-- se pagó -- efectivo, transferencia o tarjeta -- para que cuando lleguen
-- esas fases el dato ya esté siendo capturado de forma consistente.

CREATE TABLE metodos_pago (
    id INTEGER PRIMARY KEY,
    nombre TEXT NOT NULL UNIQUE,
    activo INTEGER NOT NULL DEFAULT 1,
    orden INTEGER NOT NULL DEFAULT 0
) STRICT;

INSERT INTO metodos_pago (nombre, orden) VALUES
    ('efectivo', 1),
    ('transferencia', 2),
    ('tarjeta', 3),
    ('cuenta_corriente', 4);

CREATE TABLE clientes (
    id INTEGER PRIMARY KEY,
    nombre TEXT NOT NULL,
    telefono TEXT,
    dni_cuit TEXT,
    direccion TEXT,
    observaciones TEXT,
    saldo_cuenta_corriente INTEGER NOT NULL DEFAULT 0,
    creado_en TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) STRICT;

-- Fila reservada: destino por defecto de toda venta sin cliente
-- identificado (Fase 5). Nunca se edita ni se le abre cuenta corriente
-- (punto D: una venta fiada siempre exige un cliente real).
INSERT INTO clientes (id, nombre) VALUES (1, 'Consumidor final');

CREATE TABLE cuenta_corriente_movimientos (
    id INTEGER PRIMARY KEY,
    cliente_id INTEGER NOT NULL REFERENCES clientes (id),
    tipo TEXT NOT NULL CHECK (tipo IN ('venta_fiada', 'pago', 'ajuste', 'devolucion')),
    -- Centavos. Positivo aumenta la deuda, negativo la reduce.
    monto INTEGER NOT NULL,
    saldo_resultante INTEGER NOT NULL,
    metodo_pago_id INTEGER REFERENCES metodos_pago (id),
    referencia_tipo TEXT,
    referencia_id INTEGER,
    fecha TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    observacion TEXT
) STRICT;

CREATE INDEX idx_cc_movimientos_cliente ON cuenta_corriente_movimientos (cliente_id, fecha);
