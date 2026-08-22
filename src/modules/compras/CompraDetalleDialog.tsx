import * as Dialog from "@radix-ui/react-dialog";
import { useQuery } from "@tanstack/react-query";
import { obtenerCompra } from "../../lib/api/compras";
import { formatearCentavos } from "../../lib/money";

type Props = {
  compraId: number | null;
  onOpenChange: (open: boolean) => void;
};

export function CompraDetalleDialog({ compraId, onOpenChange }: Props) {
  const open = compraId !== null;
  const { data: compra, isLoading } = useQuery({
    queryKey: ["compras", compraId],
    queryFn: () => obtenerCompra(compraId as number),
    enabled: open,
  });

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/40" />
        <Dialog.Content className="fixed left-1/2 top-1/2 max-h-[85vh] w-[90vw] max-w-xl -translate-x-1/2 -translate-y-1/2 overflow-y-auto rounded-lg border border-line bg-surface p-6 shadow-lg">
          {isLoading || !compra ? (
            <p className="text-sm text-ink-muted">Cargando...</p>
          ) : (
            <>
              <Dialog.Title className="text-lg font-semibold text-ink">
                Recepción C-{String(compra.id).padStart(6, "0")}
              </Dialog.Title>
              <p className="mt-1 text-sm text-ink-muted">
                {compra.fecha.slice(0, 10)} · {compra.proveedorNombre}
                {compra.numeroFactura && <> · Factura {compra.numeroFactura}</>}
              </p>

              <div className="mt-4 overflow-x-auto rounded-lg border border-line">
                <table className="w-full min-w-[420px] text-sm">
                  <thead>
                    <tr className="border-b border-line bg-surface-2 text-left text-xs uppercase tracking-wide text-ink-muted">
                      <th className="px-3 py-2">Producto</th>
                      <th className="px-3 py-2 text-right">Cant.</th>
                      <th className="px-3 py-2 text-right">Costo</th>
                      <th className="px-3 py-2 text-right">Subtotal</th>
                    </tr>
                  </thead>
                  <tbody>
                    {compra.detalles.map((d) => (
                      <tr key={d.id} className="border-b border-line last:border-b-0">
                        <td className="px-3 py-2">{d.productoNombre}</td>
                        <td className="px-3 py-2 text-right font-mono">{d.cantidad}</td>
                        <td className="px-3 py-2 text-right font-mono">{formatearCentavos(d.costoUnitario)}</td>
                        <td className="px-3 py-2 text-right font-mono">{formatearCentavos(d.subtotal)}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>

              <div className="mt-4 flex justify-between text-base font-semibold text-ink">
                <span>Total</span>
                <span className="font-mono">{formatearCentavos(compra.total)}</span>
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
