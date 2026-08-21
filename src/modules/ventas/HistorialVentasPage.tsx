import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { listarVentas } from "../../lib/api/ventas";
import { AppError } from "../../lib/api/client";
import { formatearCentavos } from "../../lib/money";
import { VentaDetalleDialog } from "./VentaDetalleDialog";

export function HistorialVentasPage() {
  const {
    data: ventas = [],
    error,
    isLoading,
  } = useQuery({ queryKey: ["ventas", "historial"], queryFn: listarVentas });
  const [ventaAbierta, setVentaAbierta] = useState<number | null>(null);

  return (
    <div className="flex flex-col gap-4">
      <div>
        <h1 className="text-xl font-semibold text-ink">Historial de ventas</h1>
        <p className="text-sm text-ink-muted">{ventas.length} venta(s)</p>
      </div>

      {error && (
        <p className="text-sm text-danger">
          {error instanceof AppError ? error.userMessage : "No se pudo cargar el historial."}
        </p>
      )}

      <div className="overflow-x-auto rounded-lg border border-line">
        <table className="w-full min-w-[640px] text-sm">
          <thead>
            <tr className="border-b border-line bg-surface-2 text-left text-xs uppercase tracking-wide text-ink-muted">
              <th className="px-3 py-2">N°</th>
              <th className="px-3 py-2">Fecha</th>
              <th className="px-3 py-2">Cliente</th>
              <th className="px-3 py-2 text-right">Total</th>
              <th className="px-3 py-2">Estado</th>
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
            {!isLoading && ventas.length === 0 && (
              <tr>
                <td colSpan={5} className="px-3 py-4 text-center text-ink-muted">
                  Todavía no se registró ninguna venta.
                </td>
              </tr>
            )}
            {ventas.map((venta) => (
              <tr
                key={venta.id}
                onClick={() => setVentaAbierta(venta.id)}
                className="cursor-pointer border-b border-line last:border-b-0 hover:bg-surface-2"
              >
                <td className="px-3 py-2 font-mono text-xs text-ink-muted">
                  V-{String(venta.numero).padStart(6, "0")}
                </td>
                <td className="px-3 py-2 text-ink-muted">{venta.fecha.slice(0, 10)}</td>
                <td className="px-3 py-2 text-ink">{venta.clienteNombre}</td>
                <td className="px-3 py-2 text-right font-mono">{formatearCentavos(venta.total)}</td>
                <td className="px-3 py-2">
                  <span
                    className={`rounded-full px-2 py-0.5 text-xs font-medium ${
                      venta.estado === "confirmada" ? "bg-good/15 text-good" : "bg-danger/15 text-danger"
                    }`}
                  >
                    {venta.estado === "confirmada" ? "Confirmada" : "Anulada"}
                  </span>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <VentaDetalleDialog ventaId={ventaAbierta} onOpenChange={(open) => !open && setVentaAbierta(null)} />
    </div>
  );
}
