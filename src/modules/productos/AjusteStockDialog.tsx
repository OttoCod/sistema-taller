import * as Dialog from "@radix-ui/react-dialog";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { ajustarStock, actualizarStockMinimo, type ProductoStock } from "../../lib/api/stock";
import { AppError } from "../../lib/api/client";

type Props = {
  producto: ProductoStock | null;
  onOpenChange: (open: boolean) => void;
};

export function AjusteStockDialog({ producto, onOpenChange }: Props) {
  const queryClient = useQueryClient();
  const open = producto !== null;

  const [nuevaCantidad, setNuevaCantidad] = useState("");
  const [motivo, setMotivo] = useState("");
  const [nuevoMinimo, setNuevoMinimo] = useState("");
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (producto) {
      setNuevaCantidad(String(producto.stockActual));
      setMotivo("");
      setNuevoMinimo(String(producto.stockMinimo));
      setError(null);
    }
  }, [producto]);

  function invalidar() {
    queryClient.invalidateQueries({ queryKey: ["stock"] });
  }

  const ajusteMutation = useMutation({
    mutationFn: async () => {
      const cantidad = Number(nuevaCantidad);
      if (!Number.isFinite(cantidad) || cantidad < 0) {
        throw new AppError("validation", "Ingresá una cantidad válida (0 o más).");
      }
      if (motivo.trim() === "") {
        throw new AppError("validation", "Indicá un motivo para el ajuste.");
      }
      return ajustarStock(producto!.id, cantidad, motivo.trim());
    },
    onSuccess: () => {
      invalidar();
      setMotivo("");
      setError(null);
    },
    onError: (e) => setError(e instanceof AppError ? e.userMessage : "No se pudo ajustar el stock."),
  });

  const minimoMutation = useMutation({
    mutationFn: async () => {
      const valor = Number(nuevoMinimo);
      if (!Number.isFinite(valor) || valor < 0) {
        throw new AppError("validation", "Ingresá un stock mínimo válido (0 o más).");
      }
      return actualizarStockMinimo(producto!.id, valor);
    },
    onSuccess: () => {
      invalidar();
      setError(null);
    },
    onError: (e) => setError(e instanceof AppError ? e.userMessage : "No se pudo guardar el mínimo."),
  });

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/40" />
        <Dialog.Content className="fixed left-1/2 top-1/2 w-[90vw] max-w-md -translate-x-1/2 -translate-y-1/2 rounded-lg border border-line bg-surface p-6 shadow-lg">
          <Dialog.Title className="text-lg font-semibold text-ink">
            Stock — {producto?.nombre}
          </Dialog.Title>
          <p className="mt-1 font-mono text-xs text-ink-muted">{producto?.codigoInterno}</p>

          <div className="mt-5 flex flex-col gap-2 border-b border-line pb-5">
            <p className="text-sm font-medium text-ink">Ajustar stock actual</p>
            <p className="text-xs text-ink-muted">
              Stock actual: <span className="font-mono">{producto?.stockActual}</span>
            </p>
            <label className="flex flex-col gap-1 text-sm">
              <span className="text-ink-muted">Nueva cantidad</span>
              <input
                className="rounded-md border border-line bg-surface px-2 py-1.5 text-sm focus:border-accent focus:outline-none"
                inputMode="numeric"
                value={nuevaCantidad}
                onChange={(e) => {
                  const valor = e.currentTarget.value;
                  setNuevaCantidad(valor);
                }}
              />
            </label>
            <label className="flex flex-col gap-1 text-sm">
              <span className="text-ink-muted">Motivo</span>
              <textarea
                className="rounded-md border border-line bg-surface px-2 py-1.5 text-sm focus:border-accent focus:outline-none"
                rows={2}
                placeholder="Ej: conteo físico, recepción sin factura, rotura..."
                value={motivo}
                onChange={(e) => {
                  const valor = e.currentTarget.value;
                  setMotivo(valor);
                }}
              />
            </label>
            <button
              type="button"
              onClick={() => ajusteMutation.mutate()}
              disabled={ajusteMutation.isPending}
              className="mt-1 self-start rounded-md bg-accent px-3 py-1.5 text-sm font-medium text-accent-ink disabled:opacity-60"
            >
              {ajusteMutation.isPending ? "Guardando..." : "Guardar ajuste"}
            </button>
          </div>

          <div className="mt-5 flex flex-col gap-2">
            <p className="text-sm font-medium text-ink">Stock mínimo</p>
            <p className="text-xs text-ink-muted">
              Umbral para que aparezca en "Reposición". No queda en el historial de movimientos.
            </p>
            <div className="flex gap-2">
              <input
                className="w-24 rounded-md border border-line bg-surface px-2 py-1.5 text-sm focus:border-accent focus:outline-none"
                inputMode="numeric"
                value={nuevoMinimo}
                onChange={(e) => {
                  const valor = e.currentTarget.value;
                  setNuevoMinimo(valor);
                }}
              />
              <button
                type="button"
                onClick={() => minimoMutation.mutate()}
                disabled={minimoMutation.isPending}
                className="rounded-md border border-line px-3 py-1.5 text-sm text-ink hover:bg-surface-2 disabled:opacity-60"
              >
                Guardar mínimo
              </button>
            </div>
          </div>

          {error && <p className="mt-4 text-sm text-danger">{error}</p>}

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
