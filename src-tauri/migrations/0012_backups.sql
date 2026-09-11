-- Fase 12: backups y restauración.
--
-- Registro de cada copia generada. El archivo en sí vive afuera (en la
-- carpeta que elija el negocio, o en <datos de la app>/backups por
-- defecto); acá solo queda la referencia para poder listarlos, rotarlos y
-- restaurarlos sin salir a escanear el disco.
--
-- Dos desvíos respecto del boceto original de ESQUEMA_BD.md:
--   * `tamano_bytes` sin eñe: el nombre viaja hasta un struct de Rust y un
--     tipo de TypeScript, y no vale la pena arrastrar un identificador no
--     ASCII por toda la cadena solo por el nombre de una columna.
--   * `tipo` admite 'previo_restauracion' además de manual/automatico: la
--     copia de seguridad que se saca sola justo antes de restaurar es
--     importante poder distinguirla de las otras (es la red de contención
--     si la restauración fue un error).
CREATE TABLE backups (
    id INTEGER PRIMARY KEY,
    archivo_path TEXT NOT NULL,
    fecha TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    tamano_bytes INTEGER NOT NULL,
    tipo TEXT NOT NULL CHECK (tipo IN ('manual', 'automatico', 'previo_restauracion')),
    -- Versión de migración de la base al momento de sacar la copia, para
    -- poder avisar si un backup es de un esquema más nuevo que el binario
    -- que lo quiere restaurar.
    version_esquema INTEGER NOT NULL
) STRICT;

CREATE INDEX idx_backups_fecha ON backups (fecha DESC);
