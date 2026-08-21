import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { buscarClientes } from "../../lib/api/clientes";

type Props = {
  clienteId: number | null;
  clienteNombre: string;
  onSeleccionar: (cliente: { id: number; nombre: string } | null) => void;
};

export function ClienteSelector({ clienteId, clienteNombre, onSeleccionar }: Props) {
  const [consulta, setConsulta] = useState("");
  const [abierto, setAbierto] = useState(false);

  const { data: resultados = [] } = useQuery({
    queryKey: ["clientes", "buscar-venta", consulta],
    queryFn: () => buscarClientes(consulta),
    enabled: abierto,
  });

  return (
    <div className="relative">
      <label className="flex flex-col gap-1 text-sm">
        <span className="text-ink-muted">Cliente</span>
        <div className="flex gap-2">
          <input
            className="flex-1 rounded-md border border-line bg-surface px-2 py-1.5 text-sm focus:border-accent focus:outline-none"
            value={abierto ? consulta : clienteNombre}
            placeholder="Buscar cliente..."
            onFocus={() => {
              setAbierto(true);
              setConsulta("");
            }}
            onChange={(e) => {
              const valor = e.currentTarget.value;
              setConsulta(valor);
            }}
            onBlur={() => {
              window.setTimeout(() => setAbierto(false), 150);
            }}
          />
          {clienteId !== null && (
            <button
              type="button"
              onMouseDown={() => onSeleccionar(null)}
              className="shrink-0 rounded-md border border-line px-2 text-xs text-ink-muted hover:bg-surface-2"
            >
              Quitar
            </button>
          )}
        </div>
      </label>
      {abierto && (
        <div className="absolute z-10 mt-1 max-h-60 w-full overflow-y-auto rounded-md border border-line bg-surface shadow-lg">
          <button
            type="button"
            onMouseDown={() => onSeleccionar(null)}
            className="block w-full px-3 py-1.5 text-left text-sm text-ink-muted hover:bg-surface-2"
          >
            Consumidor final
          </button>
          {resultados.map((c) => (
            <button
              key={c.id}
              type="button"
              onMouseDown={() => onSeleccionar({ id: c.id, nombre: c.nombre })}
              className="block w-full border-t border-line px-3 py-1.5 text-left text-sm hover:bg-surface-2"
            >
              {c.nombre}
              {c.telefono && <span className="ml-2 text-xs text-ink-muted">{c.telefono}</span>}
            </button>
          ))}
          {resultados.length === 0 && consulta.trim() !== "" && (
            <p className="border-t border-line px-3 py-2 text-xs text-ink-muted">Sin resultados.</p>
          )}
        </div>
      )}
    </div>
  );
}
