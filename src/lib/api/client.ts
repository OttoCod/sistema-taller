import { invoke as tauriInvoke } from "@tauri-apps/api/core";

// Todo comando de Rust que puede fallar devuelve este mismo shape
// (ver src-tauri/src/error.rs). El resto de la app nunca maneja el
// error crudo de Tauri: siempre pasa por este wrapper.
export type AppErrorKind =
  | "database"
  | "validation"
  | "not_found"
  | "conflict"
  | "io"
  | "unexpected";

export class AppError extends Error {
  readonly kind: AppErrorKind;

  constructor(kind: AppErrorKind, message: string) {
    super(message);
    this.name = "AppError";
    this.kind = kind;
  }

  /** Mensaje seguro para mostrarle al vendedor en el mostrador. */
  get userMessage(): string {
    if (this.kind === "unexpected" || this.kind === "database" || this.kind === "io") {
      return "Ocurrió un error inesperado. Quedó registrado en el log de la aplicación.";
    }
    return this.message;
  }
}

function isAppErrorShape(value: unknown): value is { kind: AppErrorKind; message: string } {
  return (
    typeof value === "object" &&
    value !== null &&
    "kind" in value &&
    "message" in value &&
    typeof (value as { kind: unknown }).kind === "string" &&
    typeof (value as { message: unknown }).message === "string"
  );
}

export async function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await tauriInvoke<T>(command, args);
  } catch (raw) {
    if (isAppErrorShape(raw)) {
      throw new AppError(raw.kind, raw.message);
    }
    throw new AppError(
      "unexpected",
      typeof raw === "string" ? raw : "Ocurrió un error inesperado.",
    );
  }
}
