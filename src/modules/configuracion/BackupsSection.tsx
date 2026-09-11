import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import {
  crearBackup,
  formatearTamano,
  guardarConfiguracionBackups,
  listarBackups,
  obtenerConfiguracionBackups,
  prepararRestauracion,
  reiniciarApp,
  type Backup,
  type RestauracionPreparada,
  type TipoBackup,
} from "../../lib/api/backups";
import { AppError } from "../../lib/api/client";

const TEXTO_TIPO: Record<TipoBackup, string> = {
  manual: "Manual",
  automatico: "Automático",
  previo_restauracion: "Previo a restaurar",
};

function fechaLegible(iso: string): string {
  const fecha = new Date(iso);
  if (Number.isNaN(fecha.getTime())) return iso.slice(0, 16).replace("T", " ");
  return fecha.toLocaleString("es-AR", { dateStyle: "short", timeStyle: "short" });
}

export function BackupsSection() {
  const queryClient = useQueryClient();
  const configQuery = useQuery({
    queryKey: ["backups", "configuracion"],
    queryFn: obtenerConfiguracionBackups,
  });
  const backupsQuery = useQuery({ queryKey: ["backups", "listado"], queryFn: listarBackups });

  const [carpeta, setCarpeta] = useState("");
  const [retencion, setRetencion] = useState("10");
  const [error, setError] = useState<string | null>(null);
  const [aviso, setAviso] = useState<string | null>(null);
  const [restauracion, setRestauracion] = useState<RestauracionPreparada | null>(null);

  useEffect(() => {
    if (configQuery.data) {
      setCarpeta(configQuery.data.carpeta);
      setRetencion(String(configQuery.data.retencion));
    }
  }, [configQuery.data]);

  function invalidar() {
    queryClient.invalidateQueries({ queryKey: ["backups"] });
  }

  const guardarConfigMutation = useMutation({
    mutationFn: async () => {
      const valor = Number(retencion);
      if (!Number.isInteger(valor) || valor < 1) {
        throw new AppError("validation", "La cantidad de copias a conservar tiene que ser 1 o más.");
      }
      return guardarConfiguracionBackups(carpeta.trim(), valor);
    },
    onSuccess: () => {
      invalidar();
      setError(null);
      setAviso("Configuración guardada.");
    },
    onError: (e) => {
      setAviso(null);
      setError(e instanceof AppError ? e.userMessage : "No se pudo guardar la configuración.");
    },
  });

  const elegirCarpetaMutation = useMutation({
    mutationFn: async () => {
      const elegida = await open({ directory: true, multiple: false });
      return typeof elegida === "string" ? elegida : null;
    },
    onSuccess: (elegida) => {
      if (elegida) {
        setCarpeta(elegida);
        setAviso('Carpeta elegida. Tocá "Guardar configuración" para aplicarla.');
      }
    },
    onError: () => setError("No se pudo abrir el selector de carpetas."),
  });

  const crearMutation = useMutation({
    mutationFn: crearBackup,
    onSuccess: (backup) => {
      invalidar();
      setError(null);
      setAviso(`Copia creada (${formatearTamano(backup.tamanoBytes)}).`);
    },
    onError: (e) => {
      setAviso(null);
      setError(e instanceof AppError ? e.userMessage : "No se pudo crear la copia.");
    },
  });

  const restaurarMutation = useMutation({
    mutationFn: (id: number) => prepararRestauracion(id),
    onSuccess: (preparada) => {
      invalidar();
      setError(null);
      setAviso(null);
      setRestauracion(preparada);
    },
    onError: (e) => {
      setAviso(null);
      setError(e instanceof AppError ? e.userMessage : "No se pudo preparar la restauración.");
    },
  });

  function pedirRestaurar(backup: Backup) {
    const confirmado = window.confirm(
      `¿Restaurar la copia del ${fechaLegible(backup.fecha)}?\n\n` +
        "Todos los datos actuales van a ser reemplazados por los de esa copia: " +
        "las ventas, compras y cambios posteriores a esa fecha se pierden.\n\n" +
        "Antes de reemplazar nada se guarda una copia de seguridad del estado actual, " +
        "y el cambio recién se aplica cuando reinicies la aplicación.",
    );
    if (confirmado) restaurarMutation.mutate(backup.id);
  }

  const backups = backupsQuery.data ?? [];

  return (
    <div className="flex flex-col gap-3 rounded-lg border border-line bg-surface p-4">
      <div>
        <p className="text-sm font-medium text-ink">Backups</p>
        <p className="text-sm text-ink-muted">
          Copias de seguridad de toda la base. Se saca una sola automáticamente al abrir la
          aplicación si la última ya tiene más de un día.
        </p>
      </div>

      {restauracion && (
        <div className="flex flex-col gap-2 rounded-md border border-warn/50 bg-warn/10 p-3 text-sm">
          <p className="font-medium text-ink">Restauración lista, falta reiniciar</p>
          <p className="text-ink-muted">
            Al reiniciar, la base va a quedar como estaba en la copia elegida. El estado actual quedó
            guardado en <span className="font-mono text-xs">{restauracion.copiaDeSeguridad}</span>, por
            si hiciera falta volver atrás.
          </p>
          <button
            type="button"
            onClick={() => reiniciarApp()}
            className="self-start rounded-md bg-accent px-3 py-1.5 text-sm font-medium text-accent-ink"
          >
            Reiniciar ahora
          </button>
        </div>
      )}

      <div className="flex flex-col gap-2 border-t border-line pt-3">
        <label className="flex flex-col gap-1 text-sm">
          <span className="text-ink-muted">Carpeta donde guardar las copias</span>
          <div className="flex gap-2">
            <input
              readOnly
              value={carpeta || configQuery.data?.carpetaEfectiva || ""}
              className="flex-1 rounded-md border border-line bg-surface-2 px-2 py-1.5 font-mono text-xs text-ink-muted"
            />
            <button
              type="button"
              onClick={() => elegirCarpetaMutation.mutate()}
              className="rounded-md border border-line px-3 py-1.5 text-sm text-ink hover:bg-surface-2"
            >
              Elegir...
            </button>
            {carpeta !== "" && (
              <button
                type="button"
                onClick={() => {
                  setCarpeta("");
                  setAviso('Se va a usar la carpeta por defecto. Tocá "Guardar configuración".');
                }}
                className="rounded-md border border-line px-3 py-1.5 text-sm text-ink-muted hover:bg-surface-2"
              >
                Usar la de siempre
              </button>
            )}
          </div>
          <span className="text-xs text-ink-muted">
            Conviene elegir un pendrive o un disco externo: una copia en el mismo disco no sirve si
            justamente se rompe ese disco.
          </span>
        </label>

        <label className="flex flex-col gap-1 text-sm">
          <span className="text-ink-muted">Copias a conservar</span>
          <input
            type="number"
            min={1}
            value={retencion}
            onChange={(e) => {
              const valor = e.currentTarget.value;
              setRetencion(valor);
            }}
            className="w-24 rounded-md border border-line bg-surface px-2 py-1.5 text-sm focus:border-accent focus:outline-none"
          />
          <span className="text-xs text-ink-muted">
            Las más viejas se borran solas. Las copias previas a una restauración nunca se rotan.
          </span>
        </label>

        <div className="flex gap-2">
          <button
            type="button"
            onClick={() => guardarConfigMutation.mutate()}
            disabled={guardarConfigMutation.isPending}
            className="rounded-md border border-line px-3 py-1.5 text-sm text-ink hover:bg-surface-2 disabled:opacity-60"
          >
            Guardar configuración
          </button>
          <button
            type="button"
            onClick={() => crearMutation.mutate()}
            disabled={crearMutation.isPending}
            className="rounded-md bg-accent px-3 py-1.5 text-sm font-medium text-accent-ink disabled:opacity-60"
          >
            {crearMutation.isPending ? "Creando copia..." : "Crear copia ahora"}
          </button>
        </div>
      </div>

      {error && <p className="text-sm text-danger">{error}</p>}
      {aviso && <p className="text-sm text-good">{aviso}</p>}

      <div className="overflow-x-auto rounded-lg border border-line">
        <table className="w-full min-w-[520px] text-sm">
          <thead>
            <tr className="border-b border-line bg-surface-2 text-left text-xs uppercase tracking-wide text-ink-muted">
              <th className="px-3 py-2">Fecha</th>
              <th className="px-3 py-2">Tipo</th>
              <th className="px-3 py-2 text-right">Tamaño</th>
              <th className="px-3 py-2"></th>
            </tr>
          </thead>
          <tbody>
            {backups.length === 0 && (
              <tr>
                <td colSpan={4} className="px-3 py-4 text-center text-ink-muted">
                  Todavía no hay ninguna copia.
                </td>
              </tr>
            )}
            {backups.map((backup) => (
              <tr key={backup.id} className="border-b border-line last:border-b-0">
                <td className="px-3 py-2 text-ink">{fechaLegible(backup.fecha)}</td>
                <td className="px-3 py-2 text-ink-muted">{TEXTO_TIPO[backup.tipo]}</td>
                <td className="px-3 py-2 text-right font-mono text-ink-muted">
                  {formatearTamano(backup.tamanoBytes)}
                </td>
                <td className="px-3 py-2 text-right">
                  {backup.archivoExiste ? (
                    <button
                      type="button"
                      onClick={() => pedirRestaurar(backup)}
                      disabled={restaurarMutation.isPending}
                      className="rounded-md border border-line px-3 py-1 text-xs text-ink hover:bg-surface-2 disabled:opacity-60"
                    >
                      Restaurar
                    </button>
                  ) : (
                    <span className="text-xs text-danger">Archivo no encontrado</span>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
