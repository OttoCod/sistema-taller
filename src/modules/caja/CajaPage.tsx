import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { obtenerResumenCaja } from "../../lib/api/caja";
import { AppError } from "../../lib/api/client";
import { formatearCentavos } from "../../lib/money";

function fechaLocalHoy(): string {
  const ahora = new Date();
  const offsetMs = ahora.getTimezoneOffset() * 60_000;
  return new Date(ahora.getTime() - offsetMs).toISOString().slice(0, 10);
}

export function CajaPage() {
  const [fecha, setFecha] = useState(fechaLocalHoy());

  const { data: resumen, error, isLoading } = useQuery({
    queryKey: ["caja", fecha],
    queryFn: () => obtenerResumenCaja(fecha),
  });

  const porMetodoCobrado = (resumen?.porMetodo ?? []).filter((m) => m.metodoPagoNombre !== "cuenta_corriente");

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-semibold text-ink">Caja</h1>
          <p className="text-sm text-ink-muted">
            Se calcula al momento a partir de las ventas confirmadas de ese día -- no es una planilla aparte,
            así nunca se desincroniza de lo que dicen las ventas.
          </p>
        </div>
        <label className="flex flex-col gap-1 text-sm">
          <span className="text-ink-muted">Fecha</span>
          <input
            type="date"
            value={fecha}
            onChange={(e) => {
              const valor = e.currentTarget.value;
              setFecha(valor);
            }}
            className="rounded-md border border-line bg-surface px-2 py-1.5 text-sm focus:border-accent focus:outline-none"
          />
        </label>
      </div>

      {error && (
        <p className="text-sm text-danger">
          {error instanceof AppError ? error.userMessage : "No se pudo cargar la caja."}
        </p>
      )}

      {isLoading && <p className="text-sm text-ink-muted">Cargando...</p>}

      {resumen && (
        <>
          <div className="rounded-lg border border-line bg-surface p-6">
            <p className="text-sm text-ink-muted">Total cobrado</p>
            <p className="text-3xl font-semibold text-ink">{formatearCentavos(resumen.totalCobrado)}</p>
            <p className="mt-1 text-sm text-ink-muted">{resumen.cantidadVentas} venta(s) confirmada(s) ese día</p>
          </div>

          <div className="overflow-x-auto rounded-lg border border-line">
            <table className="w-full min-w-[420px] text-sm">
              <thead>
                <tr className="border-b border-line bg-surface-2 text-left text-xs uppercase tracking-wide text-ink-muted">
                  <th className="px-3 py-2">Método de pago</th>
                  <th className="px-3 py-2 text-right">Monto</th>
                </tr>
              </thead>
              <tbody>
                {porMetodoCobrado.length === 0 && (
                  <tr>
                    <td colSpan={2} className="px-3 py-4 text-center text-ink-muted">
                      No hay cobros registrados ese día.
                    </td>
                  </tr>
                )}
                {porMetodoCobrado.map((m) => (
                  <tr key={m.metodoPagoId} className="border-b border-line last:border-b-0">
                    <td className="px-3 py-2 capitalize text-ink">{m.metodoPagoNombre}</td>
                    <td className="px-3 py-2 text-right font-mono">{formatearCentavos(m.monto)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          {resumen.totalFiado > 0 && (
            <p className="text-sm text-warn">
              Además, ese día quedaron <strong>{formatearCentavos(resumen.totalFiado)}</strong> fiados (cuenta
              corriente) -- no es plata que entró a la caja.
            </p>
          )}
        </>
      )}
    </div>
  );
}
