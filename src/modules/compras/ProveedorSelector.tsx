import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { buscarProveedores } from "../../lib/api/proveedores";
import { ProveedorFormDialog } from "../proveedores/ProveedorFormDialog";

type Props = {
  proveedorId: number | null;
  proveedorNombre: string;
  onSeleccionar: (proveedor: { id: number; nombre: string }) => void;
};

export function ProveedorSelector({ proveedorId, proveedorNombre, onSeleccionar }: Props) {
  const [consulta, setConsulta] = useState("");
  const [abierto, setAbierto] = useState(false);
  const [dialogoAbierto, setDialogoAbierto] = useState(false);

  const { data: resultados = [] } = useQuery({
    queryKey: ["proveedores", "buscar-compra", consulta],
    queryFn: () => buscarProveedores(consulta),
    enabled: abierto,
  });

  return (
    <div className="relative">
      <label className="flex flex-col gap-1 text-sm">
        <span className="text-ink-muted">Proveedor</span>
        <input
          className="rounded-md border border-line bg-surface px-2 py-1.5 text-sm focus:border-accent focus:outline-none"
          value={abierto ? consulta : proveedorNombre}
          placeholder="Buscar proveedor..."
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
      </label>
      {abierto && (
        <div className="absolute z-10 mt-1 max-h-60 w-full overflow-y-auto rounded-md border border-line bg-surface shadow-lg">
          {resultados.map((p) => (
            <button
              key={p.id}
              type="button"
              onMouseDown={() => onSeleccionar({ id: p.id, nombre: p.nombre })}
              className="block w-full border-b border-line px-3 py-1.5 text-left text-sm last:border-b-0 hover:bg-surface-2"
            >
              {p.nombre}
              {p.telefono && <span className="ml-2 text-xs text-ink-muted">{p.telefono}</span>}
            </button>
          ))}
          {resultados.length === 0 && (
            <p className="px-3 py-2 text-xs text-ink-muted">Sin resultados.</p>
          )}
          <button
            type="button"
            onMouseDown={() => setDialogoAbierto(true)}
            className="block w-full border-t border-line px-3 py-1.5 text-left text-sm font-medium text-accent hover:bg-surface-2"
          >
            + Crear proveedor nuevo
          </button>
        </div>
      )}
      {proveedorId !== null && (
        <p className="mt-1 text-xs text-ink-muted">Seleccionado: {proveedorNombre}</p>
      )}

      <ProveedorFormDialog
        open={dialogoAbierto}
        onOpenChange={setDialogoAbierto}
        proveedor={null}
        onGuardado={(proveedor) => {
          onSeleccionar({ id: proveedor.id, nombre: proveedor.nombre });
          setAbierto(false);
        }}
      />
    </div>
  );
}
