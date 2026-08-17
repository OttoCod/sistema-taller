import { invoke } from "./client";

export type HealthCheck = {
  appVersion: string;
  schemaVersion: number;
  dbPath: string;
  sqliteVersion: string;
};

/** Vertical slice de prueba de la Fase 1: React -> Tauri -> SQLite y de vuelta. */
export function getHealthCheck() {
  return invoke<HealthCheck>("system_health_check");
}
