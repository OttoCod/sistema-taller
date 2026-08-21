import type { ProductoStock } from "../../lib/api/stock";

function EstadoStockBadge({ estado }: { estado: ProductoStock["estadoStock"] }) {
  const estilos = {
    sin_stock: { texto: "Sin stock", clase: "bg-danger/15 text-danger" },
    bajo: { texto: "Bajo", clase: "bg-warn/15 text-warn" },
    ok: { texto: "OK", clase: "bg-good/15 text-good" },
  }[estado];
  return (
    <span className={`rounded-full px-2 py-0.5 text-xs font-medium ${estilos.clase}`}>
      {estilos.texto}
    </span>
  );
}

type Props = {
  productos: ProductoStock[];
  cargando: boolean;
  mensajeVacio: string;
  onAjustar: (producto: ProductoStock) => void;
};

export function StockTable({ productos, cargando, mensajeVacio, onAjustar }: Props) {
  return (
    <div className="overflow-x-auto rounded-lg border border-line">
      <table className="w-full min-w-[680px] text-sm">
        <thead>
          <tr className="border-b border-line bg-surface-2 text-left text-xs uppercase tracking-wide text-ink-muted">
            <th className="px-3 py-2">Nombre</th>
            <th className="px-3 py-2">Código</th>
            <th className="px-3 py-2">Marca</th>
            <th className="px-3 py-2 text-right">Stock actual</th>
            <th className="px-3 py-2 text-right">Stock mínimo</th>
            <th className="px-3 py-2">Estado</th>
            <th className="px-3 py-2" />
          </tr>
        </thead>
        <tbody>
          {cargando && (
            <tr>
              <td colSpan={7} className="px-3 py-4 text-center text-ink-muted">
                Cargando...
              </td>
            </tr>
          )}
          {!cargando && productos.length === 0 && (
            <tr>
              <td colSpan={7} className="px-3 py-4 text-center text-ink-muted">
                {mensajeVacio}
              </td>
            </tr>
          )}
          {productos.map((producto) => (
            <tr key={producto.id} className="border-b border-line last:border-b-0 hover:bg-surface-2">
              <td className="px-3 py-2 font-medium text-ink">{producto.nombre}</td>
              <td className="px-3 py-2 font-mono text-xs text-ink-muted">{producto.codigoInterno}</td>
              <td className="px-3 py-2 text-ink-muted">{producto.marcaNombre ?? "—"}</td>
              <td className="px-3 py-2 text-right font-mono">{producto.stockActual}</td>
              <td className="px-3 py-2 text-right font-mono text-ink-muted">{producto.stockMinimo}</td>
              <td className="px-3 py-2">
                <EstadoStockBadge estado={producto.estadoStock} />
              </td>
              <td className="px-3 py-2 text-right">
                <button
                  type="button"
                  onClick={() => onAjustar(producto)}
                  className="rounded-md border border-line px-2 py-1 text-xs text-ink-muted hover:bg-surface-2 hover:text-ink"
                >
                  Ajustar
                </button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
