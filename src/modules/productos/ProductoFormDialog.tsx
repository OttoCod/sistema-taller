import * as Dialog from "@radix-ui/react-dialog";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { crearMarca, listarMarcas } from "../../lib/api/marcas";
import { crearCategoria, listarCategorias } from "../../lib/api/categorias";
import {
  actualizarProducto,
  crearProducto,
  obtenerProducto,
} from "../../lib/api/productos";
import { AppError } from "../../lib/api/client";
import {
  formValuesAGuardarProducto,
  productoDetalleAFormValues,
  productoFormSchema,
  productoFormVacio,
  type ProductoFormValues,
} from "./productoSchema";

type Props = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  productoId: number | null;
};

function QuickAdd({
  placeholder,
  onAgregar,
}: {
  placeholder: string;
  onAgregar: (nombre: string) => void;
}) {
  const [valor, setValor] = useState("");
  return (
    <div className="mt-1 flex gap-1">
      <input
        value={valor}
        onChange={(e) => {
          const nuevoValor = e.currentTarget.value;
          setValor(nuevoValor);
        }}
        placeholder={placeholder}
        className="w-full rounded-md border border-line bg-surface px-2 py-1 text-xs"
      />
      <button
        type="button"
        onClick={() => {
          if (valor.trim()) {
            onAgregar(valor.trim());
            setValor("");
          }
        }}
        className="shrink-0 rounded-md border border-line px-2 text-xs text-ink-muted hover:bg-surface-2"
      >
        + agregar
      </button>
    </div>
  );
}

function Campo({ label, error, children }: { label: string; error?: string; children: React.ReactNode }) {
  return (
    <label className="flex flex-col gap-1 text-sm">
      <span className="text-ink-muted">{label}</span>
      {children}
      {error && <span className="text-xs text-danger">{error}</span>}
    </label>
  );
}

const inputClass =
  "rounded-md border border-line bg-surface px-2 py-1.5 text-sm text-ink focus:border-accent focus:outline-none";

export function ProductoFormDialog({ open, onOpenChange, productoId }: Props) {
  const queryClient = useQueryClient();
  const esEdicion = productoId !== null;

  const marcasQuery = useQuery({ queryKey: ["marcas"], queryFn: listarMarcas, enabled: open });
  const categoriasQuery = useQuery({
    queryKey: ["categorias"],
    queryFn: listarCategorias,
    enabled: open,
  });
  const detalleQuery = useQuery({
    queryKey: ["productos", productoId],
    queryFn: () => obtenerProducto(productoId as number),
    enabled: open && esEdicion,
  });

  const [valores, setValores] = useState<ProductoFormValues>(productoFormVacio);
  const [errores, setErrores] = useState<Partial<Record<string, string>>>({});
  const [errorGeneral, setErrorGeneral] = useState<string | null>(null);

  useEffect(() => {
    if (!open) return;
    if (esEdicion && detalleQuery.data) {
      setValores(productoDetalleAFormValues(detalleQuery.data));
    } else if (!esEdicion) {
      setValores(productoFormVacio);
    }
    setErrores({});
    setErrorGeneral(null);
  }, [open, esEdicion, detalleQuery.data]);

  const marcaMutation = useMutation({
    mutationFn: crearMarca,
    onSuccess: (marca) => {
      queryClient.invalidateQueries({ queryKey: ["marcas"] });
      setValores((v) => ({ ...v, marcaId: marca.id }));
    },
  });

  const categoriaMutation = useMutation({
    mutationFn: (nombre: string) => crearCategoria(nombre),
    onSuccess: (categoria) => {
      queryClient.invalidateQueries({ queryKey: ["categorias"] });
      setValores((v) => ({ ...v, categoriaId: categoria.id }));
    },
  });

  const guardarMutation = useMutation({
    mutationFn: async (datos: ProductoFormValues) => {
      const guardar = formValuesAGuardarProducto(datos);
      return esEdicion ? actualizarProducto(productoId as number, guardar) : crearProducto(guardar);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["productos"] });
      onOpenChange(false);
    },
    onError: (error) => {
      setErrorGeneral(error instanceof AppError ? error.userMessage : "No se pudo guardar.");
    },
  });

  function agregarCodigo() {
    setValores((v) => ({
      ...v,
      codigosFabricante: [...v.codigosFabricante, { codigo: "", fabricanteNombre: "", observacion: "" }],
    }));
  }

  function quitarCodigo(indice: number) {
    setValores((v) => ({
      ...v,
      codigosFabricante: v.codigosFabricante.filter((_, i) => i !== indice),
    }));
  }

  function actualizarCodigo(indice: number, campo: "codigo" | "fabricanteNombre", valor: string) {
    setValores((v) => ({
      ...v,
      codigosFabricante: v.codigosFabricante.map((c, i) =>
        i === indice ? { ...c, [campo]: valor } : c,
      ),
    }));
  }

  function onSubmit(e: React.FormEvent) {
    e.preventDefault();
    setErrorGeneral(null);
    const resultado = productoFormSchema.safeParse(valores);
    if (!resultado.success) {
      const nuevosErrores: Partial<Record<string, string>> = {};
      for (const issue of resultado.error.issues) {
        nuevosErrores[String(issue.path[0])] = issue.message;
      }
      setErrores(nuevosErrores);
      return;
    }
    setErrores({});
    guardarMutation.mutate(resultado.data);
  }

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/40" />
        <Dialog.Content className="fixed left-1/2 top-1/2 max-h-[85vh] w-[90vw] max-w-2xl -translate-x-1/2 -translate-y-1/2 overflow-y-auto rounded-lg border border-line bg-surface p-6 shadow-lg">
          <Dialog.Title className="text-lg font-semibold text-ink">
            {esEdicion ? `Editar producto ${detalleQuery.data?.codigoInterno ?? ""}` : "Nuevo producto"}
          </Dialog.Title>

          {esEdicion && detalleQuery.isLoading ? (
            <p className="mt-4 text-sm text-ink-muted">Cargando...</p>
          ) : (
            <form onSubmit={onSubmit} className="mt-4 flex flex-col gap-4">
              <Campo label="Nombre" error={errores.nombre}>
                <input
                  className={inputClass}
                  value={valores.nombre}
                  onChange={(e) => {
                    const nombre = e.currentTarget.value;
                    setValores((v) => ({ ...v, nombre }));
                  }}
                  autoFocus
                />
              </Campo>

              <div className="grid grid-cols-2 gap-4">
                <Campo label="Marca">
                  <select
                    className={inputClass}
                    value={valores.marcaId ?? ""}
                    onChange={(e) => {
                      const marcaId = e.currentTarget.value ? Number(e.currentTarget.value) : null;
                      setValores((v) => ({ ...v, marcaId }));
                    }}
                  >
                    <option value="">Sin marca</option>
                    {marcasQuery.data?.map((m) => (
                      <option key={m.id} value={m.id}>
                        {m.nombre}
                      </option>
                    ))}
                  </select>
                  <QuickAdd placeholder="Nueva marca..." onAgregar={(n) => marcaMutation.mutate(n)} />
                </Campo>

                <Campo label="Categoría">
                  <select
                    className={inputClass}
                    value={valores.categoriaId ?? ""}
                    onChange={(e) => {
                      const categoriaId = e.currentTarget.value ? Number(e.currentTarget.value) : null;
                      setValores((v) => ({ ...v, categoriaId }));
                    }}
                  >
                    <option value="">Sin categoría</option>
                    {categoriasQuery.data?.map((c) => (
                      <option key={c.id} value={c.id}>
                        {c.nombre}
                      </option>
                    ))}
                  </select>
                  <QuickAdd
                    placeholder="Nueva categoría..."
                    onAgregar={(n) => categoriaMutation.mutate(n)}
                  />
                </Campo>
              </div>

              <div className="grid grid-cols-3 gap-4">
                <Campo label="Costo ($)" error={errores.costo}>
                  <input
                    className={inputClass}
                    inputMode="decimal"
                    value={valores.costo}
                    onChange={(e) => {
                      const costo = e.currentTarget.value;
                      setValores((v) => ({ ...v, costo }));
                    }}
                  />
                </Campo>
                <Campo label="Precio de venta ($)" error={errores.precioVenta}>
                  <input
                    className={inputClass}
                    inputMode="decimal"
                    value={valores.precioVenta}
                    onChange={(e) => {
                      const precioVenta = e.currentTarget.value;
                      setValores((v) => ({ ...v, precioVenta }));
                    }}
                  />
                </Campo>
                <Campo label="Precio público ref. ($)" error={errores.precioPublico}>
                  <input
                    className={inputClass}
                    inputMode="decimal"
                    value={valores.precioPublico}
                    onChange={(e) => {
                      const precioPublico = e.currentTarget.value;
                      setValores((v) => ({ ...v, precioPublico }));
                    }}
                  />
                </Campo>
              </div>
              <p className="-mt-2 text-xs text-ink-muted">
                El precio de venta es siempre editable a mano; nada se calcula automáticamente.
              </p>

              <div className="grid grid-cols-2 gap-4">
                <Campo label="Descripción">
                  <textarea
                    className={inputClass}
                    rows={2}
                    value={valores.descripcion}
                    onChange={(e) => {
                      const descripcion = e.currentTarget.value;
                      setValores((v) => ({ ...v, descripcion }));
                    }}
                  />
                </Campo>
                <Campo label="Observaciones">
                  <textarea
                    className={inputClass}
                    rows={2}
                    value={valores.observaciones}
                    onChange={(e) => {
                      const observaciones = e.currentTarget.value;
                      setValores((v) => ({ ...v, observaciones }));
                    }}
                  />
                </Campo>
              </div>

              {esEdicion && (
                <Campo label="Estado">
                  <select
                    className={inputClass}
                    value={valores.estado}
                    onChange={(e) => {
                      const estado = e.currentTarget.value as "activo" | "inactivo";
                      setValores((v) => ({ ...v, estado }));
                    }}
                  >
                    <option value="activo">Activo</option>
                    <option value="inactivo">Inactivo</option>
                  </select>
                </Campo>
              )}

              <div>
                <div className="flex items-center justify-between">
                  <span className="text-sm text-ink-muted">Códigos de fabricante / equivalencias</span>
                  <button
                    type="button"
                    onClick={agregarCodigo}
                    className="text-xs text-accent hover:underline"
                  >
                    + agregar código
                  </button>
                </div>
                <div className="mt-2 flex flex-col gap-2">
                  {valores.codigosFabricante.map((codigo, indice) => (
                    <div key={indice} className="flex gap-2">
                      <input
                        className={`${inputClass} flex-1`}
                        placeholder="Código"
                        value={codigo.codigo}
                        onChange={(e) => actualizarCodigo(indice, "codigo", e.currentTarget.value)}
                      />
                      <input
                        className={`${inputClass} flex-1`}
                        placeholder="Fabricante (opcional)"
                        value={codigo.fabricanteNombre}
                        onChange={(e) =>
                          actualizarCodigo(indice, "fabricanteNombre", e.currentTarget.value)
                        }
                      />
                      <button
                        type="button"
                        onClick={() => quitarCodigo(indice)}
                        className="rounded-md border border-line px-2 text-xs text-ink-muted hover:bg-surface-2"
                      >
                        Quitar
                      </button>
                    </div>
                  ))}
                </div>
              </div>

              {errorGeneral && <p className="text-sm text-danger">{errorGeneral}</p>}

              <div className="mt-2 flex justify-end gap-2">
                <Dialog.Close asChild>
                  <button
                    type="button"
                    className="rounded-md border border-line px-4 py-1.5 text-sm text-ink-muted hover:bg-surface-2"
                  >
                    Cancelar
                  </button>
                </Dialog.Close>
                <button
                  type="submit"
                  disabled={guardarMutation.isPending}
                  className="rounded-md bg-accent px-4 py-1.5 text-sm font-medium text-accent-ink disabled:opacity-60"
                >
                  {guardarMutation.isPending ? "Guardando..." : "Guardar"}
                </button>
              </div>
            </form>
          )}
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
