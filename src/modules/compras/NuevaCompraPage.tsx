import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import Fuse from "fuse.js";
import { useMemo, useState } from "react";
import { buscarProductos, listarProductos, type Producto } from "../../lib/api/productos";
import { crearCompra, type CompraDetalle } from "../../lib/api/compras";
import { AppError } from "../../lib/api/client";
import { centavosAPesos, formatearCentavos, pesosACentavos } from "../../lib/money";
import { ProveedorSelector } from "./ProveedorSelector";

type ItemCompraUI = {
  producto: Producto;
  cantidad: number;
  costoUnitarioTexto: string;
};

function centavosDeTexto(texto: string): number {
  const numero = Number(texto);
  return Number.isFinite(numero) ? pesosACentavos(numero) : 0;
}

export function NuevaCompraPage() {
  const queryClient = useQueryClient();

  const [consultaProducto, setConsultaProducto] = useState("");
  const [items, setItems] = useState<ItemCompraUI[]>([]);
  const [proveedor, setProveedor] = useState<{ id: number; nombre: string } | null>(null);
  const [numeroFactura, setNumeroFactura] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [ultimaCompra, setUltimaCompra] = useState<CompraDetalle | null>(null);

  const consultaLimpia = consultaProducto.trim();
  const { data: candidatos = [] } = useQuery({
    queryKey: ["productos", "buscar-compra", consultaLimpia],
    queryFn: () => (consultaLimpia ? buscarProductos(consultaLimpia) : listarProductos()),
    enabled: consultaLimpia.length > 0,
  });

  const resultadosBusqueda = useMemo(() => {
    if (!consultaLimpia) return [];
    const fuse = new Fuse(candidatos, {
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
      items.map((item) => {
        const costoUnitario = centavosDeTexto(item.costoUnitarioTexto);
        return { ...item, costoUnitario, subtotal: costoUnitario * item.cantidad };
      }),
    [items],
  );

  const total = itemsCalculados.reduce((acc, i) => acc + i.subtotal, 0);

  function agregarProducto(producto: Producto) {
    setItems((c) => {
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
          costoUnitarioTexto: String(centavosAPesos(producto.costoActual ?? 0)),
        },
      ];
    });
    setConsultaProducto("");
  }

  function quitarItem(indice: number) {
    setItems((c) => c.filter((_, i) => i !== indice));
  }

  function actualizarItem(indice: number, cambios: Partial<ItemCompraUI>) {
    setItems((c) => c.map((item, i) => (i === indice ? { ...item, ...cambios } : item)));
  }

  function limpiarCompra() {
    setItems([]);
    setProveedor(null);
    setNumeroFactura("");
    setError(null);
  }

  const confirmarMutation = useMutation({
    mutationFn: () => {
      if (!proveedor) throw new Error("Falta seleccionar un proveedor.");
      return crearCompra({
        proveedorId: proveedor.id,
        numeroFactura: numeroFactura.trim() === "" ? null : numeroFactura.trim(),
        items: itemsCalculados.map((i) => ({
          productoId: i.producto.id,
          cantidad: i.cantidad,
          costoUnitario: i.costoUnitario,
        })),
      });
    },
    onSuccess: (compra) => {
      queryClient.invalidateQueries({ queryKey: ["productos"] });
      queryClient.invalidateQueries({ queryKey: ["stock"] });
      queryClient.invalidateQueries({ queryKey: ["compras"] });
      setUltimaCompra(compra);
      limpiarCompra();
    },
    onError: (e) => setError(e instanceof AppError ? e.userMessage : "No se pudo confirmar la recepción."),
  });

  const puedeConfirmar = proveedor !== null && items.length > 0;

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
                    {p.estado === "inactivo" && (
                      <span className="ml-2 text-xs text-ink-muted">(inactivo)</span>
                    )}
                  </span>
                  <span className="flex items-center gap-3">
                    <span className="text-xs text-ink-muted">stock: {p.stockActual}</span>
                    <span className="font-mono">{formatearCentavos(p.costoActual ?? 0)}</span>
                  </span>
                </button>
              ))}
            </div>
          )}
        </div>

        <div className="overflow-x-auto rounded-lg border border-line">
          <table className="w-full min-w-[560px] text-sm">
            <thead>
              <tr className="border-b border-line bg-surface-2 text-left text-xs uppercase tracking-wide text-ink-muted">
                <th className="px-3 py-2">Producto</th>
                <th className="px-3 py-2 text-right">Cant.</th>
                <th className="px-3 py-2 text-right">Costo unit.</th>
                <th className="px-3 py-2 text-right">Subtotal</th>
                <th className="px-3 py-2" />
              </tr>
            </thead>
            <tbody>
              {itemsCalculados.length === 0 && (
                <tr>
                  <td colSpan={5} className="px-3 py-6 text-center text-ink-muted">
                    Recepción vacía. Buscá un producto para empezar.
                  </td>
                </tr>
              )}
              {itemsCalculados.map((item, indice) => (
                <tr key={indice} className="border-b border-line last:border-b-0">
                  <td className="px-3 py-2">
                    <div className="font-medium text-ink">{item.producto.nombre}</div>
                    <div className="font-mono text-xs text-ink-muted">{item.producto.codigoInterno}</div>
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
                      value={item.costoUnitarioTexto}
                      onChange={(e) => {
                        const valor = e.currentTarget.value;
                        actualizarItem(indice, { costoUnitarioTexto: valor });
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
              ))}
            </tbody>
          </table>
        </div>

        {ultimaCompra && (
          <div className="rounded-lg border border-good/30 bg-good/10 p-4">
            <p className="font-medium text-good">
              Recepción C-{String(ultimaCompra.id).padStart(6, "0")} confirmada —{" "}
              {formatearCentavos(ultimaCompra.total)}
            </p>
            <p className="text-sm text-ink-muted">Proveedor: {ultimaCompra.proveedorNombre}</p>
          </div>
        )}
      </div>

      <div className="flex flex-col gap-4 rounded-lg border border-line bg-surface p-4">
        <ProveedorSelector
          proveedorId={proveedor?.id ?? null}
          proveedorNombre={proveedor?.nombre ?? ""}
          onSeleccionar={setProveedor}
        />

        <label className="flex flex-col gap-1 text-sm">
          <span className="text-ink-muted">N° de factura (opcional)</span>
          <input
            className="rounded-md border border-line bg-surface px-2 py-1.5 text-sm focus:border-accent focus:outline-none"
            value={numeroFactura}
            onChange={(e) => {
              const valor = e.currentTarget.value;
              setNumeroFactura(valor);
            }}
          />
        </label>

        <div className="border-t border-line pt-3">
          <div className="flex justify-between text-base font-semibold text-ink">
            <span>Total</span>
            <span className="font-mono">{formatearCentavos(total)}</span>
          </div>
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
          {confirmarMutation.isPending ? "Confirmando..." : "Confirmar recepción"}
        </button>
      </div>
    </div>
  );
}
