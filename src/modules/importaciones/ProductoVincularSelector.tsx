import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { buscarProductos, type Producto } from "../../lib/api/productos";
import { formatearCentavos } from "../../lib/money";

type Props = {
  onSeleccionar: (producto: Producto) => void;
};

export function ProductoVincularSelector({ onSeleccionar }: Props) {
  const [consulta, setConsulta] = useState("");
  const [abierto, setAbierto] = useState(false);

  const consultaLimpia = consulta.trim();
  const { data: resultados = [] } = useQuery({
    queryKey: ["productos", "buscar-vincular", consultaLimpia],
    queryFn: () => buscarProductos(consultaLimpia),
    enabled: abierto && consultaLimpia.length > 0,
  });

  return (
    <div className="relative">
      <input
        type="search"
        value={consulta}
        onChange={(e) => {
          const valor = e.currentTarget.value;
          setConsulta(valor);
        }}
        onFocus={() => setAbierto(true)}
        onBlur={() => {
          window.setTimeout(() => setAbierto(false), 150);
        }}
        placeholder="Buscar producto por nombre o código..."
        className="w-full rounded-md border border-line bg-surface px-2 py-1.5 text-sm focus:border-accent focus:outline-none"
      />
      {abierto && consultaLimpia.length > 0 && (
        <div className="absolute z-10 mt-1 w-full max-h-60 overflow-y-auto rounded-md border border-line bg-surface shadow-lg">
          {resultados.map((p) => (
            <button
              key={p.id}
              type="button"
              onMouseDown={() => onSeleccionar(p)}
              className="flex w-full items-center justify-between border-b border-line px-3 py-1.5 text-left text-sm last:border-b-0 hover:bg-surface-2"
            >
              <span>
                <span className="text-ink">{p.nombre}</span>{" "}
                <span className="font-mono text-xs text-ink-muted">{p.codigoInterno}</span>
              </span>
              <span className="font-mono text-xs text-ink-muted">{formatearCentavos(p.costoActual ?? 0)}</span>
            </button>
          ))}
          {resultados.length === 0 && (
            <p className="px-3 py-2 text-xs text-ink-muted">Sin resultados.</p>
          )}
        </div>
      )}
    </div>
  );
}
