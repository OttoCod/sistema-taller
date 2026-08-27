import * as Dialog from "@radix-ui/react-dialog";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  listarEventosComprobante,
  obtenerOCrearComprobante,
  registrarEventoComprobante,
  type TipoComprobante,
} from "../../lib/api/comprobantes";
import { obtenerConfiguracionNegocio } from "../../lib/api/configuracion";
import type { VentaDetalle } from "../../lib/api/ventas";
import { formatearCentavos } from "../../lib/money";

type Props = {
  venta: VentaDetalle | null;
  tipo: TipoComprobante;
  onOpenChange: (open: boolean) => void;
};

const TEXTO_TIPO: Record<TipoComprobante, string> = {
  ticket: "Ticket",
  a4: "Comprobante A4",
};

export function ComprobanteDialog({ venta, tipo, onOpenChange }: Props) {
  const open = venta !== null;
  const queryClient = useQueryClient();

  const negocioQuery = useQuery({
    queryKey: ["configuracion", "negocio"],
    queryFn: obtenerConfiguracionNegocio,
    enabled: open,
  });
  const comprobanteQuery = useQuery({
    queryKey: ["comprobante", venta?.id, tipo],
    queryFn: () => obtenerOCrearComprobante(venta!.id, tipo),
    enabled: open,
  });
  const eventosQuery = useQuery({
    queryKey: ["comprobanteEventos", comprobanteQuery.data?.id],
    queryFn: () => listarEventosComprobante(comprobanteQuery.data!.id),
    enabled: open && comprobanteQuery.data !== undefined,
  });

  const imprimirMutation = useMutation({
    mutationFn: async () => {
      await registrarEventoComprobante(comprobanteQuery.data!.id, "impreso");
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["comprobanteEventos", comprobanteQuery.data?.id] });
      window.print();
    },
  });

  const cargando = negocioQuery.isLoading || comprobanteQuery.isLoading || !venta;
  const yaImpreso = (eventosQuery.data ?? []).filter((e) => e.tipoEvento === "impreso").length;

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/40 print:hidden" />
        <Dialog.Content className="fixed left-1/2 top-1/2 max-h-[85vh] w-[90vw] max-w-lg -translate-x-1/2 -translate-y-1/2 overflow-y-auto rounded-lg border border-line bg-surface p-6 shadow-lg print:static print:h-auto print:max-h-none print:w-auto print:max-w-none print:translate-x-0 print:translate-y-0 print:border-none print:p-0 print:shadow-none">
          {cargando ? (
            <p className="text-sm text-ink-muted print:hidden">Cargando...</p>
          ) : (
            <>
              <div className="flex items-center justify-between print:hidden">
                <Dialog.Title className="text-lg font-semibold text-ink">
                  {TEXTO_TIPO[tipo]} — {comprobanteQuery.data?.numero}
                </Dialog.Title>
              </div>
              {yaImpreso > 0 && (
                <p className="mt-1 text-xs text-ink-muted print:hidden">
                  Ya se imprimió {yaImpreso} {yaImpreso === 1 ? "vez" : "veces"}.
                </p>
              )}

              <div
                id="comprobante-imprimible"
                className={`mt-4 flex flex-col gap-2 border border-line p-4 text-sm ${
                  tipo === "ticket" ? "mx-auto max-w-[300px] font-mono text-xs" : ""
                }`}
              >
                <div className="text-center">
                  <p className="font-semibold text-ink">{negocioQuery.data?.nombre || "(sin nombre configurado)"}</p>
                  {negocioQuery.data?.direccion && <p className="text-ink-muted">{negocioQuery.data.direccion}</p>}
                  {negocioQuery.data?.telefono && <p className="text-ink-muted">{negocioQuery.data.telefono}</p>}
                </div>

                <div className="border-t border-line pt-2 text-ink-muted">
                  <p>
                    {TEXTO_TIPO[tipo]} {comprobanteQuery.data?.numero}
                  </p>
                  <p>
                    Venta V-{String(venta.numero).padStart(6, "0")} · {venta.fecha.slice(0, 10)}
                  </p>
                  <p>Cliente: {venta.clienteNombre}</p>
                </div>

                <table className="w-full border-t border-line pt-2 text-left">
                  <thead>
                    <tr className="text-ink-muted">
                      <th className="py-1">Producto</th>
                      <th className="py-1 text-right">Cant.</th>
                      <th className="py-1 text-right">Precio</th>
                      <th className="py-1 text-right">Subtotal</th>
                    </tr>
                  </thead>
                  <tbody>
                    {venta.detalles.map((d) => (
                      <tr key={d.id}>
                        <td className="py-0.5">{d.productoNombre}</td>
                        <td className="py-0.5 text-right font-mono">{d.cantidad}</td>
                        <td className="py-0.5 text-right font-mono">{formatearCentavos(d.precioUnitario)}</td>
                        <td className="py-0.5 text-right font-mono">{formatearCentavos(d.subtotal)}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>

                <div className="flex flex-col gap-0.5 border-t border-line pt-2">
                  <div className="flex justify-between text-ink-muted">
                    <span>Subtotal</span>
                    <span className="font-mono">{formatearCentavos(venta.subtotal)}</span>
                  </div>
                  <div className="flex justify-between text-ink-muted">
                    <span>Descuento</span>
                    <span className="font-mono">{formatearCentavos(venta.descuentoTotal)}</span>
                  </div>
                  <div className="flex justify-between font-semibold text-ink">
                    <span>Total</span>
                    <span className="font-mono">{formatearCentavos(venta.total)}</span>
                  </div>
                </div>

                <div className="border-t border-line pt-2">
                  {venta.pagos.map((p) => (
                    <div key={p.id} className="flex justify-between text-ink-muted">
                      <span>{p.metodoPagoNombre}</span>
                      <span className="font-mono">{formatearCentavos(p.monto)}</span>
                    </div>
                  ))}
                </div>

                <p className="border-t border-line pt-2 text-center text-xs text-ink-muted">
                  Comprobante interno, no válido como factura.
                </p>
              </div>

              <div className="mt-5 flex justify-end gap-2 print:hidden">
                <Dialog.Close asChild>
                  <button
                    type="button"
                    className="rounded-md border border-line px-4 py-1.5 text-sm text-ink-muted hover:bg-surface-2"
                  >
                    Cerrar
                  </button>
                </Dialog.Close>
                <button
                  type="button"
                  onClick={() => imprimirMutation.mutate()}
                  disabled={imprimirMutation.isPending}
                  className="rounded-md bg-accent px-4 py-1.5 text-sm font-medium text-accent-ink disabled:opacity-60"
                >
                  Imprimir
                </button>
              </div>
            </>
          )}
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
