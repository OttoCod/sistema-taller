import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { open } from "@tauri-apps/plugin-dialog";
import { useState } from "react";
import { useNavigate } from "react-router-dom";
import {
  buscarConfirmadaConMismoHash,
  listarImportaciones,
  procesarArchivo,
  type Importacion,
} from "../../lib/api/importaciones";
import { AppError } from "../../lib/api/client";

function nombreDeRuta(ruta: string): string {
  const partes = ruta.split(/[/\\]/);
  return partes[partes.length - 1] || ruta;
}

const ESTADO_TEXTO: Record<string, string> = {
  en_revision: "En revisión",
  confirmada: "Confirmada",
  descartada: "Descartada",
};

export function ImportarExcelPage() {
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const [error, setError] = useState<string | null>(null);
  const [avisoReimportacion, setAvisoReimportacion] = useState<Importacion | null>(null);

  const historialQuery = useQuery({ queryKey: ["importaciones"], queryFn: listarImportaciones });

  const procesarMutation = useMutation({
    mutationFn: async () => {
      const ruta = await open({
        multiple: false,
        filters: [{ name: "Excel", extensions: ["xlsx", "xls", "xlsm"] }],
      });
      if (!ruta) return null;
      const archivoNombre = nombreDeRuta(ruta);
      const importacion = await procesarArchivo(ruta, archivoNombre);

      const previa = await buscarConfirmadaConMismoHash(importacion.archivoHash, importacion.id);
      if (previa) setAvisoReimportacion(previa);

      return importacion;
    },
    onSuccess: (importacion) => {
      queryClient.invalidateQueries({ queryKey: ["importaciones"] });
      if (importacion) navigate(`/productos/importar/${importacion.id}`);
    },
    onError: (e) => setError(e instanceof AppError ? e.userMessage : "No se pudo procesar el archivo."),
  });

  return (
    <div className="flex flex-col gap-6">
      <div>
        <h1 className="text-xl font-semibold text-ink">Importar Excel</h1>
        <p className="text-sm text-ink-muted">
          Solo lectura: el archivo original nunca se modifica. Nada se toca en el catálogo hasta que
          confirmes la revisión, fila por fila o en bloque para las que no tienen ningún problema.
        </p>
      </div>

      <div className="rounded-lg border border-line bg-surface p-6">
        <button
          type="button"
          onClick={() => {
            setError(null);
            setAvisoReimportacion(null);
            procesarMutation.mutate();
          }}
          disabled={procesarMutation.isPending}
          className="rounded-md bg-accent px-4 py-2 text-sm font-medium text-accent-ink disabled:opacity-60"
        >
          {procesarMutation.isPending ? "Analizando archivo..." : "Elegir archivo Excel..."}
        </button>

        {error && <p className="mt-3 text-sm text-danger">{error}</p>}

        {avisoReimportacion && (
          <p className="mt-3 text-sm text-warn">
            Este archivo ya se importó antes ({new Date(avisoReimportacion.creadaEn).toLocaleString("es-AR")}
            ). Podés seguir igual si es a propósito.
          </p>
        )}
      </div>

      <div>
        <h2 className="mb-2 text-sm font-medium text-ink">Importaciones anteriores</h2>
        <div className="overflow-x-auto rounded-lg border border-line">
          <table className="w-full min-w-[560px] text-sm">
            <thead>
              <tr className="border-b border-line bg-surface-2 text-left text-xs uppercase tracking-wide text-ink-muted">
                <th className="px-3 py-2">Archivo</th>
                <th className="px-3 py-2">Fecha</th>
                <th className="px-3 py-2 text-right">Filas</th>
                <th className="px-3 py-2">Estado</th>
              </tr>
            </thead>
            <tbody>
              {(historialQuery.data ?? []).length === 0 && (
                <tr>
                  <td colSpan={4} className="px-3 py-4 text-center text-ink-muted">
                    Todavía no se importó ningún archivo.
                  </td>
                </tr>
              )}
              {(historialQuery.data ?? []).map((imp) => (
                <tr
                  key={imp.id}
                  onClick={() => navigate(`/productos/importar/${imp.id}`)}
                  className="cursor-pointer border-b border-line last:border-b-0 hover:bg-surface-2"
                >
                  <td className="px-3 py-2 text-ink">{imp.archivoNombre}</td>
                  <td className="px-3 py-2 text-ink-muted">{imp.creadaEn.slice(0, 16).replace("T", " ")}</td>
                  <td className="px-3 py-2 text-right font-mono">{imp.totalFilas}</td>
                  <td className="px-3 py-2">
                    <span
                      className={`rounded-full px-2 py-0.5 text-xs font-medium ${
                        imp.estado === "confirmada"
                          ? "bg-good/15 text-good"
                          : imp.estado === "descartada"
                            ? "bg-danger/15 text-danger"
                            : "bg-warn/15 text-warn"
                      }`}
                    >
                      {ESTADO_TEXTO[imp.estado] ?? imp.estado}
                    </span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
