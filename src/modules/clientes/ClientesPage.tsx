import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { ID_CONSUMIDOR_FINAL, buscarClientes, listarClientes, type Cliente } from "../../lib/api/clientes";
import { AppError } from "../../lib/api/client";
import { formatearCentavos } from "../../lib/money";
import { ClienteFormDialog } from "./ClienteFormDialog";
import { CuentaCorrienteDialog } from "./CuentaCorrienteDialog";

function SaldoTexto({ saldo }: { saldo: number }) {
  if (saldo === 0) return <span className="font-mono text-ink-muted">{formatearCentavos(0)}</span>;
  const clase = saldo > 0 ? "text-danger" : "text-good";
  return <span className={`font-mono font-medium ${clase}`}>{formatearCentavos(saldo)}</span>;
}

export function ClientesPage() {
  const [consulta, setConsulta] = useState("");
  const [dialogoAbierto, setDialogoAbierto] = useState(false);
  const [clienteEditando, setClienteEditando] = useState<number | null>(null);
  const [clienteCuentaCorriente, setClienteCuentaCorriente] = useState<number | null>(null);

  const consultaLimpia = consulta.trim();
  const {
    data: clientes = [],
    error,
    isLoading,
  } = useQuery({
    queryKey: ["clientes", "buscar", consultaLimpia],
    queryFn: () => (consultaLimpia ? buscarClientes(consultaLimpia) : listarClientes()),
  });

  function abrirNuevo() {
    setClienteEditando(null);
    setDialogoAbierto(true);
  }

  function abrirEdicion(cliente: Cliente) {
    setClienteEditando(cliente.id);
    setDialogoAbierto(true);
  }

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-semibold text-ink">Clientes</h1>
          <p className="text-sm text-ink-muted">{clientes.length} cliente(s)</p>
        </div>
        <button
          onClick={abrirNuevo}
          className="rounded-md bg-accent px-4 py-2 text-sm font-medium text-accent-ink"
        >
          + Nuevo cliente
        </button>
      </div>

      <input
        type="search"
        value={consulta}
        onChange={(e) => setConsulta(e.currentTarget.value)}
        placeholder="Buscar por nombre, teléfono o patente..."
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
              <th className="px-3 py-2">Patente</th>
              <th className="px-3 py-2 text-right">Saldo</th>
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
            {!isLoading && clientes.length === 0 && (
              <tr>
                <td colSpan={5} className="px-3 py-4 text-center text-ink-muted">
                  {consultaLimpia ? "Sin resultados para esa búsqueda." : "Todavía no hay clientes cargados."}
                </td>
              </tr>
            )}
            {clientes.map((cliente) => (
              <tr
                key={cliente.id}
                className="cursor-pointer border-b border-line last:border-b-0 hover:bg-surface-2"
                onClick={() => abrirEdicion(cliente)}
              >
                <td className="px-3 py-2 font-medium text-ink">{cliente.nombre}</td>
                <td className="px-3 py-2 text-ink-muted">{cliente.telefono ?? "—"}</td>
                <td className="px-3 py-2 font-mono text-ink-muted">{cliente.patente ?? "—"}</td>
                <td className="px-3 py-2 text-right">
                  <SaldoTexto saldo={cliente.saldoCuentaCorriente} />
                </td>
                <td className="px-3 py-2 text-right">
                  {cliente.id !== ID_CONSUMIDOR_FINAL && (
                    <button
                      type="button"
                      onClick={(e) => {
                        e.stopPropagation();
                        setClienteCuentaCorriente(cliente.id);
                      }}
                      className="rounded-md border border-line px-2 py-1 text-xs text-ink-muted hover:bg-surface-2 hover:text-ink"
                    >
                      Cuenta corriente
                    </button>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <ClienteFormDialog
        open={dialogoAbierto}
        onOpenChange={setDialogoAbierto}
        clienteId={clienteEditando}
      />
      <CuentaCorrienteDialog
        clienteId={clienteCuentaCorriente}
        onOpenChange={(open) => !open && setClienteCuentaCorriente(null)}
      />
    </div>
  );
}
