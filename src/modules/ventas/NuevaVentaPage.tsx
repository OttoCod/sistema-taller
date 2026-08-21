import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import Fuse from "fuse.js";
import { useMemo, useState } from "react";
import { buscarProductos, listarProductos, type Producto } from "../../lib/api/productos";
import { listarMetodosPago } from "../../lib/api/metodosPago";
import { crearVenta, type VentaDetalle } from "../../lib/api/ventas";
import { ID_CONSUMIDOR_FINAL } from "../../lib/api/clientes";
import { AppError } from "../../lib/api/client";
import { centavosAPesos, formatearCentavos, pesosACentavos } from "../../lib/money";
import { ClienteSelector } from "./ClienteSelector";

type ItemCarritoUI = {
  producto: Producto;
  cantidad: number;
  precioUnitarioTexto: string;
  descuentoTexto: string;
};

type PagoUI = {
  metodoPagoId: number;
  montoTexto: string;
};

function centavosDeTexto(texto: string): number {
  const numero = Number(texto);
  return Number.isFinite(numero) ? pesosACentavos(numero) : 0;
}

export function NuevaVentaPage() {
  const queryClient = useQueryClient();

  const [consultaProducto, setConsultaProducto] = useState("");
  const [carrito, setCarrito] = useState<ItemCarritoUI[]>([]);
  const [cliente, setCliente] = useState<{ id: number; nombre: string } | null>(null);
  const [pagos, setPagos] = useState<PagoUI[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [ultimaVenta, setUltimaVenta] = useState<VentaDetalle | null>(null);

  const consultaLimpia = consultaProducto.trim();
  const { data: candidatos = [] } = useQuery({
    queryKey: ["productos", "buscar-venta", consultaLimpia],
    queryFn: () => (consultaLimpia ? buscarProductos(consultaLimpia) : listarProductos()),
    enabled: consultaLimpia.length > 0,
  });
  const metodosPagoQuery = useQuery({ queryKey: ["metodosPago"], queryFn: listarMetodosPago });

  const resultadosBusqueda = useMemo(() => {
    if (!consultaLimpia) return [];
    const activos = candidatos.filter((p) => p.estado === "activo");
    const fuse = new Fuse(activos, {
      keys: ["nombre", "codigoInterno", "marcaNombre"],
      threshold: 0.4,
      ignoreLocation: true,
    });
    return fuse
      .search(consultaLimpia)
      .map((r) => r.item)
      .slice(0, 8);
  }, [candidatos, consultaLimpia]);

  const itemsCalculados = useMemo(
    () =>
      carrito.map((item) => {
        const precioUnitario = centavosDeTexto(item.precioUnitarioTexto);
        const descuento = centavosDeTexto(item.descuentoTexto);
        const bruto = precioUnitario * item.cantidad;
        return { ...item, precioUnitario, descuento, subtotal: bruto - descuento };
      }),
    [carrito],
  );

  const subtotalGeneral = itemsCalculados.reduce((acc, i) => acc + i.precioUnitario * i.cantidad, 0);
  const descuentoGeneral = itemsCalculados.reduce((acc, i) => acc + i.descuento, 0);
  const total = subtotalGeneral - descuentoGeneral;

  const pagosCalculados = pagos.map((p) => ({ ...p, monto: centavosDeTexto(p.montoTexto) }));
  const totalPagado = pagosCalculados.reduce((acc, p) => acc + p.monto, 0);
  const falta = total - totalPagado;

  const usaCuentaCorriente = pagosCalculados.some((p) => {
    const metodo = metodosPagoQuery.data?.find((m) => m.id === p.metodoPagoId);
    return metodo?.nombre === "cuenta_corriente";
  });
  const clienteEsConsumidorFinal = (cliente?.id ?? ID_CONSUMIDOR_FINAL) === ID_CONSUMIDOR_FINAL;

  function agregarProducto(producto: Producto) {
    setCarrito((c) => {
      const existente = c.findIndex((i) => i.producto.id === producto.id);
      if (existente >= 0) {
        const copia = [...c];
        copia[existente] = { ...copia[existente], cantidad: copia[existente].cantidad + 1 };
        return copia;
      }
      return [
        ...c,
        {
          producto,
          cantidad: 1,
          precioUnitarioTexto: String(centavosAPesos(producto.precioVentaActual ?? 0)),
          descuentoTexto: "0",
        },
      ];
    });
    setConsultaProducto("");
  }

  function quitarItem(indice: number) {
    setCarrito((c) => c.filter((_, i) => i !== indice));
  }

  function actualizarItem(indice: number, cambios: Partial<ItemCarritoUI>) {
    setCarrito((c) => c.map((item, i) => (i === indice ? { ...item, ...cambios } : item)));
  }

  function agregarPago() {
    const primerMetodo = metodosPagoQuery.data?.[0];
    if (!primerMetodo) return;
    setPagos((p) => [
      ...p,
      { metodoPagoId: primerMetodo.id, montoTexto: String(centavosAPesos(Math.max(falta, 0))) },
    ]);
  }

  function quitarPago(indice: number) {
    setPagos((p) => p.filter((_, i) => i !== indice));
  }

  function actualizarPago(indice: number, cambios: Partial<PagoUI>) {
    setPagos((p) => p.map((pago, i) => (i === indice ? { ...pago, ...cambios } : pago)));
  }

  function limpiarVenta() {
    setCarrito([]);
    setCliente(null);
    setPagos([]);
    setError(null);
  }

  const confirmarMutation = useMutation({
    mutationFn: () =>
      crearVenta({
        clienteId: cliente?.id ?? null,
        items: itemsCalculados.map((i) => ({
          productoId: i.producto.id,
          cantidad: i.cantidad,
          precioUnitario: i.precioUnitario,
          descuento: i.descuento,
        })),
        pagos: pagosCalculados.map((p) => ({ metodoPagoId: p.metodoPagoId, monto: p.monto })),
      }),
    onSuccess: (venta) => {
      queryClient.invalidateQueries({ queryKey: ["productos"] });
      queryClient.invalidateQueries({ queryKey: ["stock"] });
      queryClient.invalidateQueries({ queryKey: ["clientes"] });
      queryClient.invalidateQueries({ queryKey: ["ventas"] });
      setUltimaVenta(venta);
      limpiarVenta();
    },
    onError: (e) => setError(e instanceof AppError ? e.userMessage : "No se pudo confirmar la venta."),
  });

  const puedeConfirmar =
    carrito.length > 0 &&
    pagos.length > 0 &&
    falta === 0 &&
    pagosCalculados.every((p) => p.monto > 0) &&
    !(usaCuentaCorriente && clienteEsConsumidorFinal);

  return (
    <div className="grid grid-cols-[1fr_360px] gap-6">
      <div className="flex flex-col gap-4">
        <div className="relative">
          <input
            autoFocus
            type="search"
            value={consultaProducto}
            onChange={(e) => {
              const valor = e.currentTarget.value;
              setConsultaProducto(valor);
            }}
            onKeyDown={(e) => {
              if (e.key === "Enter" && resultadosBusqueda.length > 0) {
                e.preventDefault();
                agregarProducto(resultadosBusqueda[0]);
              }
            }}
            placeholder="Buscar producto por nombre, código o marca... (Enter agrega el primero)"
            className="w-full rounded-md border border-line bg-surface px-3 py-2 text-sm focus:border-accent focus:outline-none"
          />
          {resultadosBusqueda.length > 0 && (
            <div className="absolute z-10 mt-1 w-full rounded-md border border-line bg-surface shadow-lg">
              {resultadosBusqueda.map((p) => (
                <button
                  key={p.id}
                  type="button"
                  onClick={() => agregarProducto(p)}
                  className="flex w-full items-center justify-between border-b border-line px-3 py-2 text-left text-sm last:border-b-0 hover:bg-surface-2"
                >
                  <span>
                    <span className="font-medium text-ink">{p.nombre}</span>{" "}
                    <span className="font-mono text-xs text-ink-muted">{p.codigoInterno}</span>
                  </span>
                  <span className="flex items-center gap-3">
                    <span className="text-xs text-ink-muted">stock: {p.stockActual}</span>
                    <span className="font-mono">{formatearCentavos(p.precioVentaActual)}</span>
                  </span>
                </button>
              ))}
            </div>
          )}
        </div>

        <div className="overflow-x-auto rounded-lg border border-line">
          <table className="w-full min-w-[640px] text-sm">
            <thead>
              <tr className="border-b border-line bg-surface-2 text-left text-xs uppercase tracking-wide text-ink-muted">
                <th className="px-3 py-2">Producto</th>
                <th className="px-3 py-2 text-right">Cant.</th>
                <th className="px-3 py-2 text-right">Precio</th>
                <th className="px-3 py-2 text-right">Desc.</th>
                <th className="px-3 py-2 text-right">Subtotal</th>
                <th className="px-3 py-2" />
              </tr>
            </thead>
            <tbody>
              {itemsCalculados.length === 0 && (
                <tr>
                  <td colSpan={6} className="px-3 py-6 text-center text-ink-muted">
                    Carrito vacío. Buscá un producto para empezar.
                  </td>
                </tr>
              )}
              {itemsCalculados.map((item, indice) => {
                const superaStock = item.cantidad > item.producto.stockActual;
                return (
                  <tr key={indice} className="border-b border-line last:border-b-0">
                    <td className="px-3 py-2">
                      <div className="font-medium text-ink">{item.producto.nombre}</div>
                      <div className="font-mono text-xs text-ink-muted">{item.producto.codigoInterno}</div>
                      {superaStock && (
                        <div className="text-xs text-warn">
                          Stock disponible: {item.producto.stockActual} (igual se puede vender)
                        </div>
                      )}
                    </td>
                    <td className="px-3 py-2">
                      <input
                        className="w-16 rounded-md border border-line bg-surface px-2 py-1 text-right text-sm"
                        inputMode="numeric"
                        value={item.cantidad}
                        onChange={(e) => {
                          const valor = Math.max(1, Number(e.currentTarget.value) || 1);
                          actualizarItem(indice, { cantidad: valor });
                        }}
                      />
                    </td>
                    <td className="px-3 py-2">
                      <input
                        className="w-24 rounded-md border border-line bg-surface px-2 py-1 text-right text-sm"
                        inputMode="decimal"
                        value={item.precioUnitarioTexto}
                        onChange={(e) => {
                          const valor = e.currentTarget.value;
                          actualizarItem(indice, { precioUnitarioTexto: valor });
                        }}
                      />
                    </td>
                    <td className="px-3 py-2">
                      <input
                        className="w-20 rounded-md border border-line bg-surface px-2 py-1 text-right text-sm"
                        inputMode="decimal"
                        value={item.descuentoTexto}
                        onChange={(e) => {
                          const valor = e.currentTarget.value;
                          actualizarItem(indice, { descuentoTexto: valor });
                        }}
                      />
                    </td>
                    <td className="px-3 py-2 text-right font-mono">{formatearCentavos(item.subtotal)}</td>
                    <td className="px-3 py-2 text-right">
                      <button
                        type="button"
                        onClick={() => quitarItem(indice)}
                        className="rounded-md border border-line px-2 py-1 text-xs text-ink-muted hover:bg-surface-2"
                      >
                        Quitar
                      </button>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>

        {ultimaVenta && (
          <div className="rounded-lg border border-good/30 bg-good/10 p-4">
            <p className="font-medium text-good">
              Venta V-{String(ultimaVenta.numero).padStart(6, "0")} confirmada —{" "}
              {formatearCentavos(ultimaVenta.total)}
            </p>
            <p className="text-sm text-ink-muted">Cliente: {ultimaVenta.clienteNombre}</p>
          </div>
        )}
      </div>

      <div className="flex flex-col gap-4 rounded-lg border border-line bg-surface p-4">
        <ClienteSelector
          clienteId={cliente?.id ?? null}
          clienteNombre={cliente?.nombre ?? "Consumidor final"}
          onSeleccionar={setCliente}
        />

        <div className="border-t border-line pt-3">
          <div className="flex justify-between text-sm text-ink-muted">
            <span>Subtotal</span>
            <span className="font-mono">{formatearCentavos(subtotalGeneral)}</span>
          </div>
          <div className="flex justify-between text-sm text-ink-muted">
            <span>Descuento</span>
            <span className="font-mono">{formatearCentavos(descuentoGeneral)}</span>
          </div>
          <div className="mt-1 flex justify-between text-base font-semibold text-ink">
            <span>Total</span>
            <span className="font-mono">{formatearCentavos(total)}</span>
          </div>
        </div>

        <div className="border-t border-line pt-3">
          <div className="mb-2 flex items-center justify-between">
            <span className="text-sm font-medium text-ink">Pagos</span>
            <button
              type="button"
              onClick={agregarPago}
              disabled={!metodosPagoQuery.data?.length}
              className="text-xs text-accent hover:underline"
            >
              + agregar pago
            </button>
          </div>
          <div className="flex flex-col gap-2">
            {pagos.map((pago, indice) => (
              <div key={indice} className="flex gap-2">
                <select
                  className="flex-1 rounded-md border border-line bg-surface px-2 py-1 text-sm"
                  value={pago.metodoPagoId}
                  onChange={(e) => {
                    const valor = Number(e.currentTarget.value);
                    actualizarPago(indice, { metodoPagoId: valor });
                  }}
                >
                  {metodosPagoQuery.data?.map((m) => (
                    <option key={m.id} value={m.id}>
                      {m.nombre}
                    </option>
                  ))}
                </select>
                <input
                  className="w-24 rounded-md border border-line bg-surface px-2 py-1 text-right text-sm"
                  inputMode="decimal"
                  value={pago.montoTexto}
                  onChange={(e) => {
                    const valor = e.currentTarget.value;
                    actualizarPago(indice, { montoTexto: valor });
                  }}
                />
                <button
                  type="button"
                  onClick={() => quitarPago(indice)}
                  className="rounded-md border border-line px-2 text-xs text-ink-muted hover:bg-surface-2"
                >
                  ×
                </button>
              </div>
            ))}
          </div>
          {pagos.length > 0 && (
            <p className={`mt-2 text-right text-sm ${falta === 0 ? "text-good" : "text-warn"}`}>
              {falta === 0 ? "Pago completo" : falta > 0 ? `Falta ${formatearCentavos(falta)}` : `Sobra ${formatearCentavos(-falta)}`}
            </p>
          )}
          {usaCuentaCorriente && clienteEsConsumidorFinal && (
            <p className="mt-2 text-sm text-danger">
              Una venta fiada necesita un cliente identificado, no "Consumidor final".
            </p>
          )}
        </div>

        {error && <p className="text-sm text-danger">{error}</p>}

        <button
          type="button"
          onClick={() => {
            setError(null);
            confirmarMutation.mutate();
          }}
          disabled={!puedeConfirmar || confirmarMutation.isPending}
          className="mt-auto rounded-md bg-accent px-4 py-2 text-sm font-medium text-accent-ink disabled:opacity-50"
        >
          {confirmarMutation.isPending ? "Confirmando..." : "Confirmar venta"}
        </button>
      </div>
    </div>
  );
}
