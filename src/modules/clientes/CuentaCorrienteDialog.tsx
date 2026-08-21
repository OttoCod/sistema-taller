import * as Dialog from "@radix-ui/react-dialog";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { obtenerCliente } from "../../lib/api/clientes";
import { ajustarCuentaCorriente, listarMovimientos, registrarPago } from "../../lib/api/cuentaCorriente";
import { listarMetodosPago } from "../../lib/api/metodosPago";
import { AppError } from "../../lib/api/client";
import { formatearCentavos, pesosACentavos } from "../../lib/money";

type Props = {
  clienteId: number | null;
  onOpenChange: (open: boolean) => void;
};

export function CuentaCorrienteDialog({ clienteId, onOpenChange }: Props) {
  const open = clienteId !== null;
  const queryClient = useQueryClient();

  const clienteQuery = useQuery({
    queryKey: ["clientes", clienteId],
    queryFn: () => obtenerCliente(clienteId as number),
    enabled: open,
  });
  const movimientosQuery = useQuery({
    queryKey: ["cuentaCorriente", "movimientos", clienteId],
    queryFn: () => listarMovimientos(clienteId as number),
    enabled: open,
  });
  const metodosPagoQuery = useQuery({
    queryKey: ["metodosPago"],
    queryFn: listarMetodosPago,
    enabled: open,
  });
  // "cuenta_corriente" es el método de pago que se elige al VENDER a
  // crédito; no tiene sentido como forma de pagar la deuda en sí.
  const metodosParaPago = (metodosPagoQuery.data ?? []).filter((m) => m.nombre !== "cuenta_corriente");

  const [montoPago, setMontoPago] = useState("");
  const [metodoPagoId, setMetodoPagoId] = useState<number | null>(null);
  const [montoAjuste, setMontoAjuste] = useState("");
  const [motivoAjuste, setMotivoAjuste] = useState("");
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (open) {
      setMontoPago("");
      setMontoAjuste("");
      setMotivoAjuste("");
      setError(null);
    }
  }, [open, clienteId]);

  useEffect(() => {
    if (metodosParaPago.length > 0 && metodoPagoId === null) {
      setMetodoPagoId(metodosParaPago[0].id);
    }
  }, [metodosParaPago, metodoPagoId]);

  function invalidar() {
    queryClient.invalidateQueries({ queryKey: ["clientes"] });
    queryClient.invalidateQueries({ queryKey: ["cuentaCorriente"] });
  }

  const pagoMutation = useMutation({
    mutationFn: async () => {
      const monto = pesosACentavos(Number(montoPago));
      if (!Number.isFinite(monto) || monto <= 0) {
        throw new AppError("validation", "Ingresá un monto de pago mayor a 0.");
      }
      if (metodoPagoId === null) {
        throw new AppError("validation", "Seleccioná un método de pago.");
      }
      return registrarPago(clienteId!, monto, metodoPagoId, null);
    },
    onSuccess: () => {
      invalidar();
      setMontoPago("");
      setError(null);
    },
    onError: (e) => setError(e instanceof AppError ? e.userMessage : "No se pudo registrar el pago."),
  });

  const ajusteMutation = useMutation({
    mutationFn: async () => {
      const monto = pesosACentavos(Number(montoAjuste));
      if (!Number.isFinite(monto) || monto === 0) {
        throw new AppError(
          "validation",
          "Ingresá un monto distinto de 0 (positivo suma deuda, negativo la reduce).",
        );
      }
      if (motivoAjuste.trim() === "") {
        throw new AppError("validation", "Indicá un motivo para el ajuste.");
      }
      return ajustarCuentaCorriente(clienteId!, monto, motivoAjuste.trim());
    },
    onSuccess: () => {
      invalidar();
      setMontoAjuste("");
      setMotivoAjuste("");
      setError(null);
    },
    onError: (e) => setError(e instanceof AppError ? e.userMessage : "No se pudo ajustar la cuenta."),
  });

  const TIPO_TEXTO: Record<string, string> = {
    venta_fiada: "Venta fiada",
    pago: "Pago",
    ajuste: "Ajuste",
    devolucion: "Devolución",
  };

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/40" />
        <Dialog.Content className="fixed left-1/2 top-1/2 max-h-[85vh] w-[90vw] max-w-xl -translate-x-1/2 -translate-y-1/2 overflow-y-auto rounded-lg border border-line bg-surface p-6 shadow-lg">
          <Dialog.Title className="text-lg font-semibold text-ink">
            Cuenta corriente — {clienteQuery.data?.nombre}
          </Dialog.Title>
          <p className="mt-1 text-sm text-ink-muted">
            Saldo actual:{" "}
            <span className="font-mono font-medium text-ink">
              {formatearCentavos(clienteQuery.data?.saldoCuentaCorriente ?? 0)}
            </span>
          </p>

          <div className="mt-5 grid grid-cols-2 gap-4 border-b border-line pb-5">
            <div className="flex flex-col gap-2">
              <p className="text-sm font-medium text-ink">Registrar pago</p>
              <label className="flex flex-col gap-1 text-sm">
                <span className="text-ink-muted">Monto ($)</span>
                <input
                  className="rounded-md border border-line bg-surface px-2 py-1.5 text-sm focus:border-accent focus:outline-none"
                  inputMode="decimal"
                  value={montoPago}
                  onChange={(e) => {
                    const valor = e.currentTarget.value;
                    setMontoPago(valor);
                  }}
                />
              </label>
              <label className="flex flex-col gap-1 text-sm">
                <span className="text-ink-muted">Método</span>
                <select
                  className="rounded-md border border-line bg-surface px-2 py-1.5 text-sm focus:border-accent focus:outline-none"
                  value={metodoPagoId ?? ""}
                  onChange={(e) => {
                    const valor = e.currentTarget.value ? Number(e.currentTarget.value) : null;
                    setMetodoPagoId(valor);
                  }}
                >
                  {metodosParaPago.map((m) => (
                    <option key={m.id} value={m.id}>
                      {m.nombre}
                    </option>
                  ))}
                </select>
              </label>
              <button
                type="button"
                onClick={() => pagoMutation.mutate()}
                disabled={pagoMutation.isPending}
                className="self-start rounded-md bg-accent px-3 py-1.5 text-sm font-medium text-accent-ink disabled:opacity-60"
              >
                {pagoMutation.isPending ? "Guardando..." : "Registrar pago"}
              </button>
            </div>

            <div className="flex flex-col gap-2">
              <p className="text-sm font-medium text-ink">Ajuste manual</p>
              <label className="flex flex-col gap-1 text-sm">
                <span className="text-ink-muted">Monto ($, +suma / -resta)</span>
                <input
                  className="rounded-md border border-line bg-surface px-2 py-1.5 text-sm focus:border-accent focus:outline-none"
                  inputMode="decimal"
                  value={montoAjuste}
                  onChange={(e) => {
                    const valor = e.currentTarget.value;
                    setMontoAjuste(valor);
                  }}
                />
              </label>
              <label className="flex flex-col gap-1 text-sm">
                <span className="text-ink-muted">Motivo</span>
                <input
                  className="rounded-md border border-line bg-surface px-2 py-1.5 text-sm focus:border-accent focus:outline-none"
                  placeholder="Ej: saldo migrado desde libreta"
                  value={motivoAjuste}
                  onChange={(e) => {
                    const valor = e.currentTarget.value;
                    setMotivoAjuste(valor);
                  }}
                />
              </label>
              <button
                type="button"
                onClick={() => ajusteMutation.mutate()}
                disabled={ajusteMutation.isPending}
                className="self-start rounded-md border border-line px-3 py-1.5 text-sm text-ink hover:bg-surface-2 disabled:opacity-60"
              >
                {ajusteMutation.isPending ? "Guardando..." : "Guardar ajuste"}
              </button>
            </div>
          </div>

          {error && <p className="mt-4 text-sm text-danger">{error}</p>}

          <div className="mt-5">
            <p className="mb-2 text-sm font-medium text-ink">Historial</p>
            <div className="overflow-x-auto rounded-lg border border-line">
              <table className="w-full min-w-[480px] text-sm">
                <thead>
                  <tr className="border-b border-line bg-surface-2 text-left text-xs uppercase tracking-wide text-ink-muted">
                    <th className="px-3 py-2">Fecha</th>
                    <th className="px-3 py-2">Tipo</th>
                    <th className="px-3 py-2 text-right">Monto</th>
                    <th className="px-3 py-2 text-right">Saldo</th>
                    <th className="px-3 py-2">Detalle</th>
                  </tr>
                </thead>
                <tbody>
                  {(movimientosQuery.data ?? []).length === 0 && (
                    <tr>
                      <td colSpan={5} className="px-3 py-4 text-center text-ink-muted">
                        Todavía no hay movimientos.
                      </td>
                    </tr>
                  )}
                  {(movimientosQuery.data ?? []).map((m) => (
                    <tr key={m.id} className="border-b border-line last:border-b-0">
                      <td className="px-3 py-2 text-xs text-ink-muted">{m.fecha.slice(0, 10)}</td>
                      <td className="px-3 py-2">{TIPO_TEXTO[m.tipo] ?? m.tipo}</td>
                      <td className="px-3 py-2 text-right font-mono">{formatearCentavos(m.monto)}</td>
                      <td className="px-3 py-2 text-right font-mono">
                        {formatearCentavos(m.saldoResultante)}
                      </td>
                      <td className="px-3 py-2 text-xs text-ink-muted">
                        {m.metodoPagoNombre ?? m.observacion ?? "—"}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>

          <div className="mt-5 flex justify-end">
            <Dialog.Close asChild>
              <button
                type="button"
                className="rounded-md border border-line px-4 py-1.5 text-sm text-ink-muted hover:bg-surface-2"
              >
                Cerrar
              </button>
            </Dialog.Close>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
