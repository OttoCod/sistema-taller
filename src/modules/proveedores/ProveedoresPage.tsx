import { useQuery } from "@tanstack/react-query";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useState } from "react";
import { buscarProveedores, listarProveedores, type Proveedor } from "../../lib/api/proveedores";
import { AppError } from "../../lib/api/client";
import { ProveedorFormDialog } from "./ProveedorFormDialog";
import { ProveedorProductosDialog } from "./ProveedorProductosDialog";

function sitioAUrl(sitio: string): string {
  return /^https?:\/\//i.test(sitio) ? sitio : `https://${sitio}`;
}

export function ProveedoresPage() {
  const [consulta, setConsulta] = useState("");
  const [dialogoAbierto, setDialogoAbierto] = useState(false);
  const [proveedorEditando, setProveedorEditando] = useState<Proveedor | null>(null);
  const [proveedorProductos, setProveedorProductos] = useState<Proveedor | null>(null);

  const consultaLimpia = consulta.trim();
  const {
    data: proveedores = [],
    error,
    isLoading,
  } = useQuery({
    queryKey: ["proveedores", "buscar", consultaLimpia],
    queryFn: () => (consultaLimpia ? buscarProveedores(consultaLimpia) : listarProveedores()),
  });

  function abrirNuevo() {
    setProveedorEditando(null);
    setDialogoAbierto(true);
  }

  function abrirEdicion(proveedor: Proveedor) {
    setProveedorEditando(proveedor);
    setDialogoAbierto(true);
  }

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-semibold text-ink">Proveedores</h1>
          <p className="text-sm text-ink-muted">{proveedores.length} proveedor(es)</p>
        </div>
        <button
          onClick={abrirNuevo}
          className="rounded-md bg-accent px-4 py-2 text-sm font-medium text-accent-ink"
        >
          + Nuevo proveedor
        </button>
      </div>

      <input
        type="search"
        value={consulta}
        onChange={(e) => {
          const valor = e.currentTarget.value;
          setConsulta(valor);
        }}
        placeholder="Buscar por nombre..."
        className="rounded-md border border-line bg-surface px-3 py-2 text-sm focus:border-accent focus:outline-none"
      />

      {error && (
        <p className="text-sm text-danger">
          {error instanceof AppError ? error.userMessage : "No se pudo buscar."}
        </p>
      )}

      <div className="overflow-x-auto rounded-lg border border-line">
        <table className="w-full min-w-[700px] text-sm">
          <thead>
            <tr className="border-b border-line bg-surface-2 text-left text-xs uppercase tracking-wide text-ink-muted">
              <th className="px-3 py-2">Nombre</th>
              <th className="px-3 py-2">Teléfono</th>
              <th className="px-3 py-2">Email</th>
              <th className="px-3 py-2">Estado</th>
              <th className="px-3 py-2" />
            </tr>
          </thead>
          <tbody>
            {isLoading && (
              <tr>
                <td colSpan={5} className="px-3 py-4 text-center text-ink-muted">
                  Buscando...
                </td>
              </tr>
            )}
            {!isLoading && proveedores.length === 0 && (
              <tr>
                <td colSpan={5} className="px-3 py-4 text-center text-ink-muted">
                  {consultaLimpia ? "Sin resultados para esa búsqueda." : "Todavía no hay proveedores cargados."}
                </td>
              </tr>
            )}
            {proveedores.map((proveedor) => (
              <tr
                key={proveedor.id}
                className="cursor-pointer border-b border-line last:border-b-0 hover:bg-surface-2"
                onClick={() => abrirEdicion(proveedor)}
              >
                <td className="px-3 py-2 font-medium text-ink">{proveedor.nombre}</td>
                <td className="px-3 py-2 text-ink-muted">{proveedor.telefono ?? "—"}</td>
                <td className="px-3 py-2 text-ink-muted">{proveedor.email ?? "—"}</td>
                <td className="px-3 py-2">
                  <span
                    className={`rounded-full px-2 py-0.5 text-xs font-medium ${
                      proveedor.activo ? "bg-good/15 text-good" : "bg-danger/15 text-danger"
                    }`}
                  >
                    {proveedor.activo ? "Activo" : "Inactivo"}
                  </span>
                </td>
                <td className="px-3 py-2 text-right">
                  <div className="flex justify-end gap-2">
                    {proveedor.sitioWeb && (
                      <button
                        type="button"
                        onClick={(e) => {
                          e.stopPropagation();
                          void openUrl(sitioAUrl(proveedor.sitioWeb as string));
                        }}
                        className="rounded-md border border-line px-2 py-1 text-xs text-ink-muted hover:bg-surface-2 hover:text-ink"
                      >
                        Consultar proveedor
                      </button>
                    )}
                    <button
                      type="button"
                      onClick={(e) => {
                        e.stopPropagation();
                        setProveedorProductos(proveedor);
                      }}
                      className="rounded-md border border-line px-2 py-1 text-xs text-ink-muted hover:bg-surface-2 hover:text-ink"
                    >
                      Productos
                    </button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <ProveedorFormDialog
        open={dialogoAbierto}
        onOpenChange={setDialogoAbierto}
        proveedor={proveedorEditando}
      />
      <ProveedorProductosDialog
        proveedor={proveedorProductos}
        onOpenChange={(open) => !open && setProveedorProductos(null)}
      />
    </div>
  );
}
