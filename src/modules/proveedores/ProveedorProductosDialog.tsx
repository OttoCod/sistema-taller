import * as Dialog from "@radix-ui/react-dialog";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import Fuse from "fuse.js";
import { useEffect, useMemo, useState } from "react";
import { buscarProductos, listarProductos, type Producto } from "../../lib/api/productos";
import {
  agregarProductoProveedor,
  listarProductoProveedores,
  quitarProductoProveedor,
  type ProductoProveedor,
} from "../../lib/api/productoProveedores";
import { AppError } from "../../lib/api/client";
import type { Proveedor } from "../../lib/api/proveedores";

type Props = {
  proveedor: Proveedor | null;
  onOpenChange: (open: boolean) => void;
};

type FormularioVinculo = {
  productoId: number;
  productoNombre: string;
  codigoProveedor: string;
  urlProducto: string;
  urlBusqueda: string;
  esPrincipal: boolean;
};

function formularioVacio(): FormularioVinculo | null {
  return null;
}

function opcional(valor: string): string | null {
  const limpio = valor.trim();
  return limpio === "" ? null : limpio;
}

export function ProveedorProductosDialog({ proveedor, onOpenChange }: Props) {
  const open = proveedor !== null;
  const queryClient = useQueryClient();

  const vinculosQuery = useQuery({
    queryKey: ["productoProveedores", proveedor?.id],
    queryFn: () => listarProductoProveedores(proveedor?.id as number),
    enabled: open,
  });

  const [consultaProducto, setConsultaProducto] = useState("");
  const [formulario, setFormulario] = useState<FormularioVinculo | null>(formularioVacio());
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!open) return;
    setConsultaProducto("");
    setFormulario(formularioVacio());
    setError(null);
  }, [open, proveedor?.id]);

  const consultaLimpia = consultaProducto.trim();
  const { data: candidatos = [] } = useQuery({
    queryKey: ["productos", "buscar-proveedor", consultaLimpia],
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

  function elegirProducto(producto: Producto) {
    setFormulario({
      productoId: producto.id,
      productoNombre: producto.nombre,
      codigoProveedor: "",
      urlProducto: "",
      urlBusqueda: "",
      esPrincipal: false,
    });
    setConsultaProducto("");
  }

  function editarVinculo(vinculo: ProductoProveedor) {
    setFormulario({
      productoId: vinculo.productoId,
      productoNombre: vinculo.productoNombre,
      codigoProveedor: vinculo.codigoProveedor ?? "",
      urlProducto: vinculo.urlProducto ?? "",
      urlBusqueda: vinculo.urlBusqueda ?? "",
      esPrincipal: vinculo.esPrincipal,
    });
  }

  const guardarMutation = useMutation({
    mutationFn: async () => {
      if (!formulario) throw new AppError("validation", "Elegí un producto primero.");
      return agregarProductoProveedor(proveedor?.id as number, {
        productoId: formulario.productoId,
        codigoProveedor: opcional(formulario.codigoProveedor),
        urlProducto: opcional(formulario.urlProducto),
        urlBusqueda: opcional(formulario.urlBusqueda),
        esPrincipal: formulario.esPrincipal,
      });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["productoProveedores", proveedor?.id] });
      setFormulario(formularioVacio());
      setError(null);
    },
    onError: (e) => setError(e instanceof AppError ? e.userMessage : "No se pudo guardar el vínculo."),
  });

  const quitarMutation = useMutation({
    mutationFn: (id: number) => quitarProductoProveedor(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["productoProveedores", proveedor?.id] });
    },
    onError: (e) => setError(e instanceof AppError ? e.userMessage : "No se pudo quitar el vínculo."),
  });

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/40" />
        <Dialog.Content className="fixed left-1/2 top-1/2 max-h-[85vh] w-[90vw] max-w-2xl -translate-x-1/2 -translate-y-1/2 overflow-y-auto rounded-lg border border-line bg-surface p-6 shadow-lg">
          <Dialog.Title className="text-lg font-semibold text-ink">
            Productos de {proveedor?.nombre}
          </Dialog.Title>

          <div className="mt-4 border-b border-line pb-4">
            {!formulario ? (
              <div className="relative">
                <input
                  type="search"
                  value={consultaProducto}
                  onChange={(e) => {
                    const valor = e.currentTarget.value;
                    setConsultaProducto(valor);
                  }}
                  placeholder="Buscar producto para agregar..."
                  className="w-full rounded-md border border-line bg-surface px-3 py-2 text-sm focus:border-accent focus:outline-none"
                />
                {resultadosBusqueda.length > 0 && (
                  <div className="absolute z-10 mt-1 w-full rounded-md border border-line bg-surface shadow-lg">
                    {resultadosBusqueda.map((p) => (
                      <button
                        key={p.id}
                        type="button"
                        onClick={() => elegirProducto(p)}
                        className="flex w-full items-center justify-between border-b border-line px-3 py-2 text-left text-sm last:border-b-0 hover:bg-surface-2"
                      >
                        <span className="font-medium text-ink">{p.nombre}</span>
                        <span className="font-mono text-xs text-ink-muted">{p.codigoInterno}</span>
                      </button>
                    ))}
                  </div>
                )}
              </div>
            ) : (
              <div className="flex flex-col gap-3">
                <p className="text-sm font-medium text-ink">{formulario.productoNombre}</p>
                <div className="grid grid-cols-2 gap-3">
                  <label className="flex flex-col gap-1 text-sm">
                    <span className="text-ink-muted">Código del proveedor</span>
                    <input
                      className="rounded-md border border-line bg-surface px-2 py-1.5 text-sm focus:border-accent focus:outline-none"
                      value={formulario.codigoProveedor}
                      onChange={(e) => {
                        const valor = e.currentTarget.value;
                        setFormulario((f) => (f ? { ...f, codigoProveedor: valor } : f));
                      }}
                    />
                  </label>
                  <label className="flex items-center gap-2 pt-5 text-sm text-ink">
                    <input
                      type="checkbox"
                      checked={formulario.esPrincipal}
                      onChange={(e) => {
                        const valor = e.currentTarget.checked;
                        setFormulario((f) => (f ? { ...f, esPrincipal: valor } : f));
                      }}
                    />
                    Proveedor principal para este producto
                  </label>
                </div>
                <label className="flex flex-col gap-1 text-sm">
                  <span className="text-ink-muted">URL del producto</span>
                  <input
                    className="rounded-md border border-line bg-surface px-2 py-1.5 text-sm focus:border-accent focus:outline-none"
                    value={formulario.urlProducto}
                    onChange={(e) => {
                      const valor = e.currentTarget.value;
                      setFormulario((f) => (f ? { ...f, urlProducto: valor } : f));
                    }}
                  />
                </label>
                <label className="flex flex-col gap-1 text-sm">
                  <span className="text-ink-muted">URL de búsqueda</span>
                  <input
                    className="rounded-md border border-line bg-surface px-2 py-1.5 text-sm focus:border-accent focus:outline-none"
                    value={formulario.urlBusqueda}
                    onChange={(e) => {
                      const valor = e.currentTarget.value;
                      setFormulario((f) => (f ? { ...f, urlBusqueda: valor } : f));
                    }}
                  />
                </label>
                <div className="flex gap-2">
                  <button
                    type="button"
                    onClick={() => guardarMutation.mutate()}
                    disabled={guardarMutation.isPending}
                    className="rounded-md bg-accent px-3 py-1.5 text-sm font-medium text-accent-ink disabled:opacity-60"
                  >
                    {guardarMutation.isPending ? "Guardando..." : "Guardar vínculo"}
                  </button>
                  <button
                    type="button"
                    onClick={() => setFormulario(formularioVacio())}
                    className="rounded-md border border-line px-3 py-1.5 text-sm text-ink-muted hover:bg-surface-2"
                  >
                    Cancelar
                  </button>
                </div>
              </div>
            )}
          </div>

          {error && <p className="mt-3 text-sm text-danger">{error}</p>}

          <div className="mt-4">
            <div className="overflow-x-auto rounded-lg border border-line">
              <table className="w-full min-w-[560px] text-sm">
                <thead>
                  <tr className="border-b border-line bg-surface-2 text-left text-xs uppercase tracking-wide text-ink-muted">
                    <th className="px-3 py-2">Producto</th>
                    <th className="px-3 py-2">Código prov.</th>
                    <th className="px-3 py-2">Principal</th>
                    <th className="px-3 py-2" />
                  </tr>
                </thead>
                <tbody>
                  {(vinculosQuery.data ?? []).length === 0 && (
                    <tr>
                      <td colSpan={4} className="px-3 py-4 text-center text-ink-muted">
                        Todavía no hay productos vinculados a este proveedor.
                      </td>
                    </tr>
                  )}
                  {(vinculosQuery.data ?? []).map((vinculo) => (
                    <tr
                      key={vinculo.id}
                      className="cursor-pointer border-b border-line last:border-b-0 hover:bg-surface-2"
                      onClick={() => editarVinculo(vinculo)}
                    >
                      <td className="px-3 py-2">
                        <div className="font-medium text-ink">{vinculo.productoNombre}</div>
                        <div className="font-mono text-xs text-ink-muted">{vinculo.codigoInterno}</div>
                      </td>
                      <td className="px-3 py-2 text-ink-muted">{vinculo.codigoProveedor ?? "—"}</td>
                      <td className="px-3 py-2 text-ink-muted">{vinculo.esPrincipal ? "Sí" : "—"}</td>
                      <td className="px-3 py-2 text-right">
                        <button
                          type="button"
                          onClick={(e) => {
                            e.stopPropagation();
                            quitarMutation.mutate(vinculo.id);
                          }}
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
