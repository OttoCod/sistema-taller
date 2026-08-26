import * as Dialog from "@radix-ui/react-dialog";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { anularVenta, obtenerVenta } from "../../lib/api/ventas";
import {
  crearDevolucion,
  listarDevolucionesPorVenta,
  type EstadoProductoDevuelto,
  type MetodoDevolucion,
} from "../../lib/api/devoluciones";
import { ID_CONSUMIDOR_FINAL } from "../../lib/api/clientes";
import { AppError } from "../../lib/api/client";
import { formatearCentavos } from "../../lib/money";

type Props = {
  ventaId: number | null;
  onOpenChange: (open: boolean) => void;
};

const METODOS_DEVOLUCION: { valor: MetodoDevolucion; texto: string }[] = [
  { valor: "reembolso_efectivo", texto: "Reembolso en efectivo" },
  { valor: "nota_credito", texto: "Nota de crédito" },
  { valor: "cambio_producto", texto: "Cambio por otro producto" },
  { valor: "reduccion_deuda", texto: "Reducción de deuda (cuenta corriente)" },
];

const ESTADOS_PRODUCTO: { valor: EstadoProductoDevuelto; texto: string }[] = [
  { valor: "vuelve_a_stock", texto: "Vuelve a stock" },
  { valor: "en_revision", texto: "En revisión" },
  { valor: "defectuoso", texto: "Defectuoso" },
  { valor: "dañado", texto: "Dañado" },
];

type LineaDevolucion = {
  seleccionada: boolean;
  cantidad: string;
  estadoProducto: EstadoProductoDevuelto;
  observacion: string;
};

export function VentaDetalleDialog({ ventaId, onOpenChange }: Props) {
  const open = ventaId !== null;
  const queryClient = useQueryClient();

  const ventaQuery = useQuery({
    queryKey: ["ventas", ventaId],
    queryFn: () => obtenerVenta(ventaId as number),
    enabled: open,
  });
  const devolucionesQuery = useQuery({
    queryKey: ["devoluciones", "porVenta", ventaId],
    queryFn: () => listarDevolucionesPorVenta(ventaId as number),
    enabled: open,
  });
  const venta = ventaQuery.data;
  const devoluciones = devolucionesQuery.data ?? [];

  const [mostrarAnular, setMostrarAnular] = useState(false);
  const [motivoAnulacion, setMotivoAnulacion] = useState("");
  const [mostrarDevolucion, setMostrarDevolucion] = useState(false);
  const [metodoDevolucion, setMetodoDevolucion] = useState<MetodoDevolucion>("reembolso_efectivo");
  const [motivoDevolucion, setMotivoDevolucion] = useState("");
  const [lineas, setLineas] = useState<Record<number, LineaDevolucion>>({});
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (open) {
      setMostrarAnular(false);
      setMotivoAnulacion("");
      setMostrarDevolucion(false);
      setMetodoDevolucion("reembolso_efectivo");
      setMotivoDevolucion("");
      setLineas({});
      setError(null);
    }
  }, [open, ventaId]);

  function invalidar() {
    queryClient.invalidateQueries({ queryKey: ["ventas"] });
    queryClient.invalidateQueries({ queryKey: ["devoluciones"] });
    queryClient.invalidateQueries({ queryKey: ["productos"] });
    queryClient.invalidateQueries({ queryKey: ["stock"] });
    queryClient.invalidateQueries({ queryKey: ["clientes"] });
    queryClient.invalidateQueries({ queryKey: ["cuentaCorriente"] });
    queryClient.invalidateQueries({ queryKey: ["caja"] });
  }

  const anularMutation = useMutation({
    mutationFn: async () => {
      const motivo = motivoAnulacion.trim();
      if (motivo === "") {
        throw new AppError("validation", "Indicá un motivo para anular la venta.");
      }
      return anularVenta(ventaId!, motivo);
    },
    onSuccess: () => {
      invalidar();
      setMostrarAnular(false);
      setMotivoAnulacion("");
      setError(null);
    },
    onError: (e) => setError(e instanceof AppError ? e.userMessage : "No se pudo anular la venta."),
  });

  const devolucionMutation = useMutation({
    mutationFn: async () => {
      const motivo = motivoDevolucion.trim();
      if (motivo === "") {
        throw new AppError("validation", "Indicá un motivo para la devolución.");
      }
      const items = Object.entries(lineas)
        .filter(([, l]) => l.seleccionada)
        .map(([ventaDetalleId, l]) => ({
          ventaDetalleId: Number(ventaDetalleId),
          cantidad: Number(l.cantidad),
          estadoProducto: l.estadoProducto,
          observacion: l.observacion.trim() === "" ? null : l.observacion.trim(),
        }));
      if (items.length === 0) {
        throw new AppError("validation", "Seleccioná al menos un producto a devolver.");
      }
      if (items.some((i) => !Number.isFinite(i.cantidad) || i.cantidad <= 0)) {
        throw new AppError("validation", "Cada cantidad a devolver tiene que ser mayor a 0.");
      }
      return crearDevolucion({
        ventaId: ventaId!,
        motivo,
        metodoDevolucion,
        items,
      });
    },
    onSuccess: () => {
      invalidar();
      setMostrarDevolucion(false);
      setMotivoDevolucion("");
      setLineas({});
      setError(null);
    },
    onError: (e) => setError(e instanceof AppError ? e.userMessage : "No se pudo registrar la devolución."),
  });

  function disponiblePorLinea(ventaDetalleId: number, cantidadOriginal: number): number {
    const yaDevuelta = devoluciones
      .flatMap((d) => d.detalles)
      .filter((d) => d.ventaDetalleId === ventaDetalleId)
      .reduce((suma, d) => suma + d.cantidad, 0);
    return cantidadOriginal - yaDevuelta;
  }

  function actualizarLinea(ventaDetalleId: number, cambios: Partial<LineaDevolucion>) {
    setLineas((actual) => ({
      ...actual,
      [ventaDetalleId]: {
        seleccionada: actual[ventaDetalleId]?.seleccionada ?? false,
        cantidad: actual[ventaDetalleId]?.cantidad ?? "1",
        estadoProducto: actual[ventaDetalleId]?.estadoProducto ?? "vuelve_a_stock",
        observacion: actual[ventaDetalleId]?.observacion ?? "",
        ...cambios,
      },
    }));
  }

  const hayLineasDisponibles =
    venta?.detalles.some((d) => disponiblePorLinea(d.id, d.cantidad) > 0) ?? false;
  const puedeAnular = venta?.estado === "confirmada" && devoluciones.length === 0;

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/40" />
        <Dialog.Content className="fixed left-1/2 top-1/2 max-h-[85vh] w-[90vw] max-w-2xl -translate-x-1/2 -translate-y-1/2 overflow-y-auto rounded-lg border border-line bg-surface p-6 shadow-lg">
          {ventaQuery.isLoading || !venta ? (
            <p className="text-sm text-ink-muted">Cargando...</p>
          ) : (
            <>
              <div className="flex items-start justify-between">
                <div>
                  <Dialog.Title className="text-lg font-semibold text-ink">
                    Venta V-{String(venta.numero).padStart(6, "0")}
                  </Dialog.Title>
                  <p className="mt-1 text-sm text-ink-muted">
                    {venta.fecha.slice(0, 10)} · {venta.clienteNombre}
                  </p>
                </div>
                <span
                  className={`rounded-full px-2 py-0.5 text-xs font-medium ${
                    venta.estado === "confirmada" ? "bg-good/15 text-good" : "bg-danger/15 text-danger"
                  }`}
                >
                  {venta.estado === "confirmada" ? "Confirmada" : "Anulada"}
                </span>
              </div>

              {venta.estado === "anulada" && (
                <p className="mt-2 rounded-md bg-danger/10 px-3 py-2 text-sm text-danger">
                  Anulada el {venta.fechaAnulacion?.slice(0, 10)} — {venta.motivoAnulacion}
                </p>
              )}

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

              {devoluciones.length > 0 && (
                <div className="mt-4">
                  <p className="mb-2 text-sm font-medium text-ink">Devoluciones registradas</p>
                  <div className="flex flex-col gap-2">
                    {devoluciones.map((dev) => (
                      <div key={dev.id} className="rounded-md border border-line px-3 py-2 text-sm">
                        <div className="flex justify-between text-ink-muted">
                          <span>
                            {dev.fecha.slice(0, 10)} ·{" "}
                            {METODOS_DEVOLUCION.find((m) => m.valor === dev.metodoDevolucion)?.texto}
                          </span>
                          <span className="font-mono text-ink">{formatearCentavos(dev.totalDevuelto)}</span>
                        </div>
                        <p className="text-ink-muted">{dev.motivo}</p>
                        <ul className="mt-1 list-inside list-disc text-xs text-ink-muted">
                          {dev.detalles.map((det) => (
                            <li key={det.id}>
                              {det.cantidad}× {det.productoNombre} —{" "}
                              {ESTADOS_PRODUCTO.find((e) => e.valor === det.estadoProducto)?.texto}
                            </li>
                          ))}
                        </ul>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {error && <p className="mt-4 text-sm text-danger">{error}</p>}

              {venta.estado === "confirmada" && (
                <div className="mt-5 flex flex-col gap-4 border-t border-line pt-4">
                  {!mostrarDevolucion && !mostrarAnular && (
                    <div className="flex gap-2">
                      {hayLineasDisponibles && (
                        <button
                          type="button"
                          onClick={() => setMostrarDevolucion(true)}
                          className="rounded-md border border-line px-3 py-1.5 text-sm text-ink hover:bg-surface-2"
                        >
                          Registrar devolución
                        </button>
                      )}
                      {puedeAnular && (
                        <button
                          type="button"
                          onClick={() => setMostrarAnular(true)}
                          className="rounded-md border border-danger/40 px-3 py-1.5 text-sm text-danger hover:bg-danger/10"
                        >
                          Anular venta
                        </button>
                      )}
                    </div>
                  )}

                  {!puedeAnular && !mostrarDevolucion && devoluciones.length > 0 && (
                    <p className="text-xs text-ink-muted">
                      No se puede anular: esta venta ya tiene devoluciones registradas.
                    </p>
                  )}

                  {mostrarAnular && (
                    <div className="flex flex-col gap-2 rounded-md border border-danger/40 p-3">
                      <p className="text-sm font-medium text-ink">
                        Anular esta venta completa (repone stock y revierte cuenta corriente si corresponde)
                      </p>
                      <label className="flex flex-col gap-1 text-sm">
                        <span className="text-ink-muted">Motivo</span>
                        <textarea
                          className="rounded-md border border-line bg-surface px-2 py-1.5 text-sm focus:border-accent focus:outline-none"
                          rows={2}
                          value={motivoAnulacion}
                          onChange={(e) => {
                            const valor = e.currentTarget.value;
                            setMotivoAnulacion(valor);
                          }}
                        />
                      </label>
                      <div className="flex gap-2">
                        <button
                          type="button"
                          onClick={() => {
                            if (
                              window.confirm(
                                "¿Confirmás que anulás esta venta completa? Esta acción no se puede deshacer.",
                              )
                            ) {
                              anularMutation.mutate();
                            }
                          }}
                          disabled={anularMutation.isPending}
                          className="self-start rounded-md bg-danger px-3 py-1.5 text-sm font-medium text-white disabled:opacity-60"
                        >
                          {anularMutation.isPending ? "Anulando..." : "Confirmar anulación"}
                        </button>
                        <button
                          type="button"
                          onClick={() => {
                            setMostrarAnular(false);
                            setError(null);
                          }}
                          className="rounded-md border border-line px-3 py-1.5 text-sm text-ink-muted hover:bg-surface-2"
                        >
                          Cancelar
                        </button>
                      </div>
                    </div>
                  )}

                  {mostrarDevolucion && (
                    <div className="flex flex-col gap-3 rounded-md border border-line p-3">
                      <p className="text-sm font-medium text-ink">Qué productos vuelven</p>
                      {venta.detalles.map((d) => {
                        const disponible = disponiblePorLinea(d.id, d.cantidad);
                        if (disponible <= 0) return null;
                        const linea = lineas[d.id];
                        return (
                          <div key={d.id} className="flex flex-col gap-2 border-b border-line pb-2 last:border-b-0">
                            <label className="flex items-center gap-2 text-sm">
                              <input
                                type="checkbox"
                                checked={linea?.seleccionada ?? false}
                                onChange={(e) => {
                                  const marcado = e.currentTarget.checked;
                                  actualizarLinea(d.id, { seleccionada: marcado });
                                }}
                              />
                              <span className="text-ink">
                                {d.productoNombre} (quedan {disponible} sin devolver de {d.cantidad})
                              </span>
                            </label>
                            {linea?.seleccionada && (
                              <div className="ml-6 flex flex-wrap gap-3">
                                <label className="flex flex-col gap-1 text-sm">
                                  <span className="text-ink-muted">Cantidad</span>
                                  <input
                                    type="number"
                                    min={1}
                                    max={disponible}
                                    className="w-20 rounded-md border border-line bg-surface px-2 py-1 text-sm focus:border-accent focus:outline-none"
                                    value={linea.cantidad}
                                    onChange={(e) => {
                                      const valor = e.currentTarget.value;
                                      actualizarLinea(d.id, { cantidad: valor });
                                    }}
                                  />
                                </label>
                                <label className="flex flex-col gap-1 text-sm">
                                  <span className="text-ink-muted">Estado del producto</span>
                                  <select
                                    className="rounded-md border border-line bg-surface px-2 py-1 text-sm focus:border-accent focus:outline-none"
                                    value={linea.estadoProducto}
                                    onChange={(e) => {
                                      const valor = e.currentTarget.value as EstadoProductoDevuelto;
                                      actualizarLinea(d.id, { estadoProducto: valor });
                                    }}
                                  >
                                    {ESTADOS_PRODUCTO.map((es) => (
                                      <option key={es.valor} value={es.valor}>
                                        {es.texto}
                                      </option>
                                    ))}
                                  </select>
                                </label>
                                <label className="flex flex-1 flex-col gap-1 text-sm">
                                  <span className="text-ink-muted">Observación (opcional)</span>
                                  <input
                                    className="rounded-md border border-line bg-surface px-2 py-1 text-sm focus:border-accent focus:outline-none"
                                    value={linea.observacion}
                                    onChange={(e) => {
                                      const valor = e.currentTarget.value;
                                      actualizarLinea(d.id, { observacion: valor });
                                    }}
                                  />
                                </label>
                              </div>
                            )}
                          </div>
                        );
                      })}

                      <label className="flex flex-col gap-1 text-sm">
                        <span className="text-ink-muted">Método de devolución</span>
                        <select
                          className="rounded-md border border-line bg-surface px-2 py-1.5 text-sm focus:border-accent focus:outline-none"
                          value={metodoDevolucion}
                          onChange={(e) => {
                            const valor = e.currentTarget.value as MetodoDevolucion;
                            setMetodoDevolucion(valor);
                          }}
                        >
                          {METODOS_DEVOLUCION.filter(
                            (m) => m.valor !== "reduccion_deuda" || venta.clienteId !== ID_CONSUMIDOR_FINAL,
                          ).map((m) => (
                            <option key={m.valor} value={m.valor}>
                              {m.texto}
                            </option>
                          ))}
                        </select>
                      </label>

                      <label className="flex flex-col gap-1 text-sm">
                        <span className="text-ink-muted">Motivo</span>
                        <textarea
                          className="rounded-md border border-line bg-surface px-2 py-1.5 text-sm focus:border-accent focus:outline-none"
                          rows={2}
                          value={motivoDevolucion}
                          onChange={(e) => {
                            const valor = e.currentTarget.value;
                            setMotivoDevolucion(valor);
                          }}
                        />
                      </label>

                      <div className="flex gap-2">
                        <button
                          type="button"
                          onClick={() => devolucionMutation.mutate()}
                          disabled={devolucionMutation.isPending}
                          className="self-start rounded-md bg-accent px-3 py-1.5 text-sm font-medium text-accent-ink disabled:opacity-60"
                        >
                          {devolucionMutation.isPending ? "Guardando..." : "Confirmar devolución"}
                        </button>
                        <button
                          type="button"
                          onClick={() => {
                            setMostrarDevolucion(false);
                            setLineas({});
                            setError(null);
                          }}
                          className="rounded-md border border-line px-3 py-1.5 text-sm text-ink-muted hover:bg-surface-2"
                        >
                          Cancelar
                        </button>
                      </div>
                    </div>
                  )}
                </div>
              )}

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
