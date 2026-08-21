import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { listarCuentasPendientes } from "../../lib/api/clientes";
import { AppError } from "../../lib/api/client";
import { formatearCentavos } from "../../lib/money";
import { CuentaCorrienteDialog } from "./CuentaCorrienteDialog";

function fechaCorta(fecha: string | null): string {
  return fecha ? fecha.slice(0, 10) : "—";
}

export function CuentasPendientesPage() {
  const {
    data: clientes = [],
    error,
    isLoading,
  } = useQuery({
    queryKey: ["clientes", "cuentasPendientes"],
    queryFn: listarCuentasPendientes,
  });
  const [clienteAbierto, setClienteAbierto] = useState<number | null>(null);

  return (
    <div className="flex flex-col gap-4">
      <div>
        <h1 className="text-xl font-semibold text-ink">Cuentas pendientes</h1>
        <p className="text-sm text-ink-muted">{clientes.length} cliente(s) con saldo pendiente</p>
      </div>

      {error && (
        <p className="text-sm text-danger">
          {error instanceof AppError ? error.userMessage : "No se pudo cargar las cuentas pendientes."}
        </p>
      )}

      <div className="overflow-x-auto rounded-lg border border-line">
        <table className="w-full min-w-[640px] text-sm">
          <thead>
            <tr className="border-b border-line bg-surface-2 text-left text-xs uppercase tracking-wide text-ink-muted">
              <th className="px-3 py-2">Cliente</th>
              <th className="px-3 py-2">Teléfono</th>
              <th className="px-3 py-2">Último movimiento</th>
              <th className="px-3 py-2 text-right">Deuda</th>
              <th className="px-3 py-2" />
            </tr>
          </thead>
          <tbody>
            {isLoading && (
              <tr>
                <td colSpan={5} className="px-3 py-4 text-center text-ink-muted">
                  Cargando...
                </td>
              </tr>
            )}
            {!isLoading && clientes.length === 0 && (
              <tr>
                <td colSpan={5} className="px-3 py-4 text-center text-ink-muted">
                  No hay cuentas pendientes.
                </td>
              </tr>
            )}
            {clientes.map((cliente) => (
              <tr key={cliente.id} className="border-b border-line last:border-b-0 hover:bg-surface-2">
                <td className="px-3 py-2 font-medium text-ink">{cliente.nombre}</td>
                <td className="px-3 py-2 text-ink-muted">{cliente.telefono ?? "—"}</td>
                <td className="px-3 py-2 text-ink-muted">{fechaCorta(cliente.fechaUltimoMovimiento)}</td>
                <td className="px-3 py-2 text-right font-mono font-medium text-danger">
                  {formatearCentavos(cliente.saldoCuentaCorriente)}
                </td>
                <td className="px-3 py-2 text-right">
                  <button
                    type="button"
                    onClick={() => setClienteAbierto(cliente.id)}
                    className="rounded-md border border-line px-2 py-1 text-xs text-ink-muted hover:bg-surface-2 hover:text-ink"
                  >
                    Registrar pago
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <CuentaCorrienteDialog
        clienteId={clienteAbierto}
        onOpenChange={(open) => !open && setClienteAbierto(null)}
      />
    </div>
  );
}
