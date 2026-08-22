import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { listarCompras } from "../../lib/api/compras";
import { AppError } from "../../lib/api/client";
import { formatearCentavos } from "../../lib/money";
import { CompraDetalleDialog } from "./CompraDetalleDialog";

export function HistorialComprasPage() {
  const {
    data: compras = [],
    error,
    isLoading,
  } = useQuery({ queryKey: ["compras", "historial"], queryFn: listarCompras });
  const [compraAbierta, setCompraAbierta] = useState<number | null>(null);

  return (
    <div className="flex flex-col gap-4">
      <div>
        <h1 className="text-xl font-semibold text-ink">Historial de compras</h1>
        <p className="text-sm text-ink-muted">{compras.length} recepción(es)</p>
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
              <th className="px-3 py-2">Proveedor</th>
              <th className="px-3 py-2">Factura</th>
              <th className="px-3 py-2 text-right">Total</th>
              <th className="px-3 py-2">Estado</th>
            </tr>
          </thead>
          <tbody>
            {isLoading && (
              <tr>
                <td colSpan={6} className="px-3 py-4 text-center text-ink-muted">
                  Cargando...
                </td>
              </tr>
            )}
            {!isLoading && compras.length === 0 && (
              <tr>
                <td colSpan={6} className="px-3 py-4 text-center text-ink-muted">
                  Todavía no se registró ninguna recepción.
                </td>
              </tr>
            )}
            {compras.map((compra) => (
              <tr
                key={compra.id}
                onClick={() => setCompraAbierta(compra.id)}
                className="cursor-pointer border-b border-line last:border-b-0 hover:bg-surface-2"
              >
                <td className="px-3 py-2 font-mono text-xs text-ink-muted">
                  C-{String(compra.id).padStart(6, "0")}
                </td>
                <td className="px-3 py-2 text-ink-muted">{compra.fecha.slice(0, 10)}</td>
                <td className="px-3 py-2 text-ink">{compra.proveedorNombre}</td>
                <td className="px-3 py-2 text-ink-muted">{compra.numeroFactura ?? "—"}</td>
                <td className="px-3 py-2 text-right font-mono">{formatearCentavos(compra.total)}</td>
                <td className="px-3 py-2">
                  <span
                    className={`rounded-full px-2 py-0.5 text-xs font-medium ${
                      compra.estado === "registrada" ? "bg-good/15 text-good" : "bg-danger/15 text-danger"
                    }`}
                  >
                    {compra.estado === "registrada" ? "Registrada" : "Anulada"}
                  </span>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <CompraDetalleDialog compraId={compraAbierta} onOpenChange={(open) => !open && setCompraAbierta(null)} />
    </div>
  );
}
