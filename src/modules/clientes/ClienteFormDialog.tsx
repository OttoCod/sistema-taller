import * as Dialog from "@radix-ui/react-dialog";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import {
  ID_CONSUMIDOR_FINAL,
  actualizarCliente,
  crearCliente,
  obtenerCliente,
} from "../../lib/api/clientes";
import { AppError } from "../../lib/api/client";
import {
  clienteAFormValues,
  clienteFormSchema,
  clienteFormVacio,
  formValuesAGuardarCliente,
  type ClienteFormValues,
} from "./clienteSchema";

type Props = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  clienteId: number | null;
};

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

export function ClienteFormDialog({ open, onOpenChange, clienteId }: Props) {
  const queryClient = useQueryClient();
  const esEdicion = clienteId !== null;
  const esConsumidorFinal = clienteId === ID_CONSUMIDOR_FINAL;

  const clienteQuery = useQuery({
    queryKey: ["clientes", clienteId],
    queryFn: () => obtenerCliente(clienteId as number),
    enabled: open && esEdicion,
  });

  const [valores, setValores] = useState<ClienteFormValues>(clienteFormVacio);
  const [errores, setErrores] = useState<Partial<Record<string, string>>>({});
  const [errorGeneral, setErrorGeneral] = useState<string | null>(null);

  useEffect(() => {
    if (!open) return;
    if (esEdicion && clienteQuery.data) {
      setValores(clienteAFormValues(clienteQuery.data));
    } else if (!esEdicion) {
      setValores(clienteFormVacio);
    }
    setErrores({});
    setErrorGeneral(null);
  }, [open, esEdicion, clienteQuery.data]);

  const guardarMutation = useMutation({
    mutationFn: async (datos: ClienteFormValues) => {
      const guardar = formValuesAGuardarCliente(datos);
      return esEdicion ? actualizarCliente(clienteId as number, guardar) : crearCliente(guardar);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["clientes"] });
      onOpenChange(false);
    },
    onError: (error) => {
      setErrorGeneral(error instanceof AppError ? error.userMessage : "No se pudo guardar.");
    },
  });

  function onSubmit(e: React.FormEvent) {
    e.preventDefault();
    setErrorGeneral(null);
    const resultado = clienteFormSchema.safeParse(valores);
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
        <Dialog.Content className="fixed left-1/2 top-1/2 w-[90vw] max-w-lg -translate-x-1/2 -translate-y-1/2 rounded-lg border border-line bg-surface p-6 shadow-lg">
          <Dialog.Title className="text-lg font-semibold text-ink">
            {esEdicion ? "Editar cliente" : "Nuevo cliente"}
          </Dialog.Title>

          {esConsumidorFinal ? (
            <p className="mt-4 text-sm text-ink-muted">
              "Consumidor final" es un cliente reservado del sistema y no se puede editar.
            </p>
          ) : esEdicion && clienteQuery.isLoading ? (
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
                <Campo label="Teléfono">
                  <input
                    className={inputClass}
                    value={valores.telefono}
                    onChange={(e) => {
                      const telefono = e.currentTarget.value;
                      setValores((v) => ({ ...v, telefono }));
                    }}
                  />
                </Campo>
                <Campo label="DNI / CUIT">
                  <input
                    className={inputClass}
                    value={valores.dniCuit}
                    onChange={(e) => {
                      const dniCuit = e.currentTarget.value;
                      setValores((v) => ({ ...v, dniCuit }));
                    }}
                  />
                </Campo>
              </div>

              <Campo label="Dirección">
                <input
                  className={inputClass}
                  value={valores.direccion}
                  onChange={(e) => {
                    const direccion = e.currentTarget.value;
                    setValores((v) => ({ ...v, direccion }));
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
