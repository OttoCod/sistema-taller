import * as Dialog from "@radix-ui/react-dialog";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { actualizarProveedor, crearProveedor, type Proveedor } from "../../lib/api/proveedores";
import { AppError } from "../../lib/api/client";
import {
  formValuesAGuardarProveedor,
  proveedorAFormValues,
  proveedorFormSchema,
  proveedorFormVacio,
  type ProveedorFormValues,
} from "./proveedorSchema";

type Props = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  proveedor: Proveedor | null;
  onGuardado?: (proveedor: Proveedor) => void;
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

export function ProveedorFormDialog({ open, onOpenChange, proveedor, onGuardado }: Props) {
  const queryClient = useQueryClient();
  const esEdicion = proveedor !== null;

  const [valores, setValores] = useState<ProveedorFormValues>(proveedorFormVacio);
  const [errores, setErrores] = useState<Partial<Record<string, string>>>({});
  const [errorGeneral, setErrorGeneral] = useState<string | null>(null);

  useEffect(() => {
    if (!open) return;
    setValores(proveedor ? proveedorAFormValues(proveedor) : proveedorFormVacio);
    setErrores({});
    setErrorGeneral(null);
  }, [open, proveedor]);

  const guardarMutation = useMutation({
    mutationFn: async (datos: ProveedorFormValues) => {
      const guardar = formValuesAGuardarProveedor(datos);
      return esEdicion ? actualizarProveedor(proveedor.id, guardar) : crearProveedor(guardar);
    },
    onSuccess: (proveedorGuardado) => {
      queryClient.invalidateQueries({ queryKey: ["proveedores"] });
      onOpenChange(false);
      onGuardado?.(proveedorGuardado);
    },
    onError: (error) => {
      setErrorGeneral(error instanceof AppError ? error.userMessage : "No se pudo guardar.");
    },
  });

  function onSubmit(e: React.FormEvent) {
    e.preventDefault();
    setErrorGeneral(null);
    const resultado = proveedorFormSchema.safeParse(valores);
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
            {esEdicion ? "Editar proveedor" : "Nuevo proveedor"}
          </Dialog.Title>

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
              <Campo label="WhatsApp">
                <input
                  className={inputClass}
                  value={valores.whatsapp}
                  onChange={(e) => {
                    const whatsapp = e.currentTarget.value;
                    setValores((v) => ({ ...v, whatsapp }));
                  }}
                />
              </Campo>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <Campo label="Email">
                <input
                  className={inputClass}
                  value={valores.email}
                  onChange={(e) => {
                    const email = e.currentTarget.value;
                    setValores((v) => ({ ...v, email }));
                  }}
                />
              </Campo>
              <Campo label="Sitio web">
                <input
                  className={inputClass}
                  value={valores.sitioWeb}
                  onChange={(e) => {
                    const sitioWeb = e.currentTarget.value;
                    setValores((v) => ({ ...v, sitioWeb }));
                  }}
                />
              </Campo>
            </div>

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
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
