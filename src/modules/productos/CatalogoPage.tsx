import { useQuery } from "@tanstack/react-query";
import Fuse from "fuse.js";
import { useMemo, useState } from "react";
import { buscarProductos, listarProductos, type Producto } from "../../lib/api/productos";
import { AppError } from "../../lib/api/client";
import { formatearCentavos } from "../../lib/money";
import { ProductoFormDialog } from "./ProductoFormDialog";

function EstadoBadge({ estado }: { estado: Producto["estado"] }) {
  const esActivo = estado === "activo";
  return (
    <span
      className={`rounded-full px-2 py-0.5 text-xs font-medium ${
        esActivo ? "bg-good/15 text-good" : "bg-ink-muted/15 text-ink-muted"
      }`}
    >
      {esActivo ? "Activo" : "Inactivo"}
    </span>
  );
}

export function CatalogoPage() {
  const [consulta, setConsulta] = useState("");
  const [dialogoAbierto, setDialogoAbierto] = useState(false);
  const [productoEditando, setProductoEditando] = useState<number | null>(null);

  const consultaLimpia = consulta.trim();
  const { data: candidatos = [], error, isLoading } = useQuery({
    queryKey: ["productos", "buscar", consultaLimpia],
    queryFn: () => (consultaLimpia ? buscarProductos(consultaLimpia) : listarProductos()),
  });

  // El backend (FTS5) ya filtró por token; acá se reordena tolerando
  // errores de tipeo que un match exacto de FTS5 no captura (punto H).
  const resultados = useMemo(() => {
    if (!consultaLimpia) return candidatos;
    const fuse = new Fuse(candidatos, {
      keys: ["nombre", "codigoInterno", "marcaNombre"],
      threshold: 0.4,
      ignoreLocation: true,
    });
    return fuse.search(consultaLimpia).map((r) => r.item);
  }, [candidatos, consultaLimpia]);

  function abrirNuevo() {
    setProductoEditando(null);
    setDialogoAbierto(true);
  }

  function abrirEdicion(id: number) {
    setProductoEditando(id);
    setDialogoAbierto(true);
  }

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-semibold text-ink">Catálogo</h1>
          <p className="text-sm text-ink-muted">{resultados.length} producto(s)</p>
        </div>
        <button
          onClick={abrirNuevo}
          className="rounded-md bg-accent px-4 py-2 text-sm font-medium text-accent-ink"
        >
          + Nuevo producto
        </button>
      </div>

      <input
        type="search"
        value={consulta}
        onChange={(e) => setConsulta(e.currentTarget.value)}
        placeholder="Buscar por nombre, código interno, código de fabricante, marca o categoría..."
        className="rounded-md border border-line bg-surface px-3 py-2 text-sm focus:border-accent focus:outline-none"
      />

      {error && (
        <p className="text-sm text-danger">
          {error instanceof AppError ? error.userMessage : "No se pudo buscar."}
        </p>
      )}

      <div className="overflow-x-auto rounded-lg border border-line">
        <table className="w-full min-w-[720px] text-sm">
          <thead>
            <tr className="border-b border-line bg-surface-2 text-left text-xs uppercase tracking-wide text-ink-muted">
              <th className="px-3 py-2">Nombre</th>
              <th className="px-3 py-2">Código</th>
              <th className="px-3 py-2">Marca</th>
              <th className="px-3 py-2">Categoría</th>
              <th className="px-3 py-2 text-right">Costo</th>
              <th className="px-3 py-2 text-right">Precio venta</th>
              <th className="px-3 py-2">Estado</th>
            </tr>
          </thead>
          <tbody>
            {isLoading && (
              <tr>
                <td colSpan={7} className="px-3 py-4 text-center text-ink-muted">
                  Buscando...
                </td>
              </tr>
            )}
            {!isLoading && resultados.length === 0 && (
              <tr>
                <td colSpan={7} className="px-3 py-4 text-center text-ink-muted">
                  {consultaLimpia
                    ? "Sin resultados para esa búsqueda."
                    : "Todavía no hay productos cargados."}
                </td>
              </tr>
            )}
            {resultados.map((producto) => (
              <tr
                key={producto.id}
                onClick={() => abrirEdicion(producto.id)}
                className="cursor-pointer border-b border-line last:border-b-0 hover:bg-surface-2"
              >
                <td className="px-3 py-2 font-medium text-ink">{producto.nombre}</td>
                <td className="px-3 py-2 font-mono text-xs text-ink-muted">{producto.codigoInterno}</td>
                <td className="px-3 py-2 text-ink-muted">{producto.marcaNombre ?? "—"}</td>
                <td className="px-3 py-2 text-ink-muted">{producto.categoriaNombre ?? "—"}</td>
                <td className="px-3 py-2 text-right font-mono">{formatearCentavos(producto.costoActual)}</td>
                <td className="px-3 py-2 text-right font-mono">
                  {formatearCentavos(producto.precioVentaActual)}
                </td>
                <td className="px-3 py-2">
                  <EstadoBadge estado={producto.estado} />
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <ProductoFormDialog
        open={dialogoAbierto}
        onOpenChange={setDialogoAbierto}
        productoId={productoEditando}
      />
    </div>
  );
}
