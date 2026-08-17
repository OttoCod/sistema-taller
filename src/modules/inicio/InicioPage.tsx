import { useQuery } from "@tanstack/react-query";
import { getHealthCheck } from "../../lib/api/system";
import { AppError } from "../../lib/api/client";

function StatusRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between border-b border-line py-2 text-sm last:border-b-0">
      <span className="text-ink-muted">{label}</span>
      <span className="font-mono text-ink">{value}</span>
    </div>
  );
}

export function InicioPage() {
  const { data, error, isLoading } = useQuery({
    queryKey: ["system", "health-check"],
    queryFn: getHealthCheck,
  });

  return (
    <div className="flex flex-col gap-6">
      <div>
        <h1 className="text-xl font-semibold text-ink">Inicio</h1>
        <p className="text-sm text-ink-muted">
          Los módulos funcionales (ventas, stock, clientes, etc.) se agregan en las próximas
          fases. Esta pantalla solo confirma que la arquitectura base está funcionando.
        </p>
      </div>

      <div className="max-w-md rounded-lg border border-line bg-surface p-4">
        <p className="mb-2 font-mono text-xs uppercase tracking-wider text-ink-muted">
          Estado del sistema
        </p>
        {isLoading && <p className="text-sm text-ink-muted">Consultando base de datos...</p>}
        {error && (
          <p className="text-sm text-danger">
            {error instanceof AppError ? error.userMessage : "No se pudo conectar."}
          </p>
        )}
        {data && (
          <div>
            <StatusRow label="Base de datos" value="Conectada" />
            <StatusRow label="Versión de esquema" value={String(data.schemaVersion)} />
            <StatusRow label="Motor" value={data.sqliteVersion} />
            <StatusRow label="Versión de la app" value={data.appVersion} />
          </div>
        )}
      </div>
    </div>
  );
}
