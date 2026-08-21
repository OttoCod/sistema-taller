import * as Dialog from "@radix-ui/react-dialog";
import { useQuery } from "@tanstack/react-query";
import { obtenerVenta } from "../../lib/api/ventas";
import { formatearCentavos } from "../../lib/money";

type Props = {
  ventaId: number | null;
  onOpenChange: (open: boolean) => void;
};

export function VentaDetalleDialog({ ventaId, onOpenChange }: Props) {
  const open = ventaId !== null;
  const { data: venta, isLoading } = useQuery({
    queryKey: ["ventas", ventaId],
    queryFn: () => obtenerVenta(ventaId as number),
    enabled: open,
  });

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/40" />
        <Dialog.Content className="fixed left-1/2 top-1/2 max-h-[85vh] w-[90vw] max-w-xl -translate-x-1/2 -translate-y-1/2 overflow-y-auto rounded-lg border border-line bg-surface p-6 shadow-lg">
          {isLoading || !venta ? (
            <p className="text-sm text-ink-muted">Cargando...</p>
          ) : (
            <>
              <Dialog.Title className="text-lg font-semibold text-ink">
                Venta V-{String(venta.numero).padStart(6, "0")}
              </Dialog.Title>
              <p className="mt-1 text-sm text-ink-muted">
                {venta.fecha.slice(0, 10)} · {venta.clienteNombre}
              </p>

              <div className="mt-4 overflow-x-auto rounded-lg border border-line">
                <table className="w-full min-w-[420px] text-sm">
                  <thead>
                    <tr className="border-b border-line bg-surface-2 text-left text-xs uppercase tracking-wide text-ink-muted">
                      <th className="px-3 py-2">Producto</th>
                      <th className="px-3 py-2 text-right">Cant.</th>
                      <th className="px-3 py-2 text-right">Precio</th>
                      <th className="px-3 py-2 text-right">Subtotal</th>
                    </tr>
                  </thead>
                  <tbody>
                    {venta.detalles.map((d) => (
                      <tr key={d.id} className="border-b border-line last:border-b-0">
                        <td className="px-3 py-2">{d.productoNombre}</td>
                        <td className="px-3 py-2 text-right font-mono">{d.cantidad}</td>
                        <td className="px-3 py-2 text-right font-mono">{formatearCentavos(d.precioUnitario)}</td>
                        <td className="px-3 py-2 text-right font-mono">{formatearCentavos(d.subtotal)}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>

              <div className="mt-4 flex flex-col gap-1 text-sm">
                <div className="flex justify-between text-ink-muted">
                  <span>Subtotal</span>
                  <span className="font-mono">{formatearCentavos(venta.subtotal)}</span>
                </div>
                <div className="flex justify-between text-ink-muted">
                  <span>Descuento</span>
                  <span className="font-mono">{formatearCentavos(venta.descuentoTotal)}</span>
                </div>
                <div className="flex justify-between text-base font-semibold text-ink">
                  <span>Total</span>
                  <span className="font-mono">{formatearCentavos(venta.total)}</span>
                </div>
              </div>

              <div className="mt-4">
                <p className="mb-1 text-sm font-medium text-ink">Pagos</p>
                {venta.pagos.map((p) => (
                  <div key={p.id} className="flex justify-between text-sm text-ink-muted">
                    <span>{p.metodoPagoNombre}</span>
                    <span className="font-mono">{formatearCentavos(p.monto)}</span>
                  </div>
                ))}
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
            </>
          )}
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
