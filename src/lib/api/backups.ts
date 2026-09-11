import { invoke } from "./client";

export type TipoBackup = "manual" | "automatico" | "previo_restauracion";

export type Backup = {
  id: number;
  archivoPath: string;
  fecha: string;
  tamanoBytes: number;
  tipo: TipoBackup;
  versionEsquema: number;
  /** False si el archivo ya no está en su carpeta (lo borraron, se desconectó el pendrive). */
  archivoExiste: boolean;
};

export type ConfiguracionBackups = {
  /** Vacío = se usa la carpeta por defecto dentro de los datos de la app. */
  carpeta: string;
  retencion: number;
  /** La carpeta que se usa realmente, ya resuelta. Solo lectura. */
  carpetaEfectiva: string;
};

export type RestauracionPreparada = {
  backupRestaurado: string;
  copiaDeSeguridad: string;
};

export function listarBackups() {
  return invoke<Backup[]>("backups_listar");
}

export function crearBackup() {
  return invoke<Backup>("backups_crear");
}

export function obtenerConfiguracionBackups() {
  return invoke<ConfiguracionBackups>("backups_obtener_configuracion");
}

export function guardarConfiguracionBackups(carpeta: string, retencion: number) {
  return invoke<void>("backups_guardar_configuracion", { carpeta, retencion });
}

/**
 * Deja todo listo para restaurar (copia de seguridad + marcador) pero no
 * toca la base en uso: el reemplazo ocurre al reiniciar la app.
 */
export function prepararRestauracion(id: number) {
  return invoke<RestauracionPreparada>("backups_preparar_restauracion", { id });
}

export function reiniciarApp() {
  return invoke<void>("app_reiniciar");
}

export function formatearTamano(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}
