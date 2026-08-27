import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { guardarConfiguracionNegocio, obtenerConfiguracionNegocio } from "../../lib/api/configuracion";
import { AppError } from "../../lib/api/client";

export function ConfiguracionPage() {
  const queryClient = useQueryClient();
  const { data: negocio, isLoading } = useQuery({
    queryKey: ["configuracion", "negocio"],
    queryFn: obtenerConfiguracionNegocio,
  });

  const [nombre, setNombre] = useState("");
  const [direccion, setDireccion] = useState("");
  const [telefono, setTelefono] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [guardadoOk, setGuardadoOk] = useState(false);

  useEffect(() => {
    if (negocio) {
      setNombre(negocio.nombre);
      setDireccion(negocio.direccion);
      setTelefono(negocio.telefono);
    }
  }, [negocio]);

  const guardarMutation = useMutation({
    mutationFn: async () => {
      if (nombre.trim() === "") {
        throw new AppError("validation", "El nombre del negocio no puede estar vacío.");
      }
      return guardarConfiguracionNegocio({ nombre: nombre.trim(), direccion: direccion.trim(), telefono: telefono.trim() });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["configuracion"] });
      setError(null);
      setGuardadoOk(true);
      setTimeout(() => setGuardadoOk(false), 2000);
    },
    onError: (e) => {
      setGuardadoOk(false);
      setError(e instanceof AppError ? e.userMessage : "No se pudo guardar la configuración.");
    },
  });

  return (
    <div className="flex flex-col gap-4">
      <div>
        <h1 className="text-xl font-semibold text-ink">Configuración</h1>
        <p className="text-sm text-ink-muted">
          Estos datos aparecen en el encabezado de los comprobantes que se imprimen desde una venta.
        </p>
      </div>

      {isLoading && <p className="text-sm text-ink-muted">Cargando...</p>}

      {!isLoading && (
        <div className="flex max-w-md flex-col gap-3 rounded-lg border border-line bg-surface p-4">
          <p className="text-sm font-medium text-ink">Datos del negocio</p>
          <label className="flex flex-col gap-1 text-sm">
            <span className="text-ink-muted">Nombre</span>
            <input
              className="rounded-md border border-line bg-surface px-2 py-1.5 text-sm focus:border-accent focus:outline-none"
              value={nombre}
              onChange={(e) => {
                const valor = e.currentTarget.value;
                setNombre(valor);
              }}
            />
          </label>
          <label className="flex flex-col gap-1 text-sm">
            <span className="text-ink-muted">Dirección</span>
            <input
              className="rounded-md border border-line bg-surface px-2 py-1.5 text-sm focus:border-accent focus:outline-none"
              value={direccion}
              onChange={(e) => {
                const valor = e.currentTarget.value;
                setDireccion(valor);
              }}
            />
          </label>
          <label className="flex flex-col gap-1 text-sm">
            <span className="text-ink-muted">Teléfono</span>
            <input
              className="rounded-md border border-line bg-surface px-2 py-1.5 text-sm focus:border-accent focus:outline-none"
              value={telefono}
              onChange={(e) => {
                const valor = e.currentTarget.value;
                setTelefono(valor);
              }}
            />
          </label>

          {error && <p className="text-sm text-danger">{error}</p>}
          {guardadoOk && <p className="text-sm text-good">Guardado.</p>}

          <button
            type="button"
            onClick={() => guardarMutation.mutate()}
            disabled={guardarMutation.isPending}
            className="self-start rounded-md bg-accent px-3 py-1.5 text-sm font-medium text-accent-ink disabled:opacity-60"
          >
            {guardarMutation.isPending ? "Guardando..." : "Guardar"}
          </button>
        </div>
      )}
    </div>
  );
}
