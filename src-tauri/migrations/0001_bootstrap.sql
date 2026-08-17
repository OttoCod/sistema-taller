-- Fase 1: solo infraestructura transversal, ningún módulo funcional
-- (productos, ventas, etc. llegan en la Fase 2 en adelante).
--
-- STRICT: SQLite exige que cada columna respete su tipo declarado en vez
-- de aceptar cualquier cosa (comportamiento por defecto de SQLite). Se usa
-- en todas las tablas del proyecto desde acá en adelante.

CREATE TABLE usuarios (
    id INTEGER PRIMARY KEY,
    nombre TEXT NOT NULL,
    activo INTEGER NOT NULL DEFAULT 1,
    creado_en TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) STRICT;

-- Única fila en V1: no hay login ni usuarios múltiples todavía (sección 32
-- del documento de arquitectura), pero toda tabla futura que necesite
-- "quién hizo esto" ya puede referenciar esta fila desde el día uno.
INSERT INTO usuarios (id, nombre) VALUES (1, 'Usuario principal');

CREATE TABLE configuracion (
    clave TEXT PRIMARY KEY,
    valor TEXT,
    actualizado_en TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) STRICT;

CREATE TABLE auditoria (
    id INTEGER PRIMARY KEY,
    entidad_tipo TEXT NOT NULL,
    entidad_id INTEGER,
    accion TEXT NOT NULL,
    detalle_json TEXT,
    fecha TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    usuario_id INTEGER NOT NULL REFERENCES usuarios (id)
) STRICT;

CREATE INDEX idx_auditoria_entidad ON auditoria (entidad_tipo, entidad_id);
