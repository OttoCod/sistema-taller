import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useMemo, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import {
  aplicarPendientes,
  descartarImportacion,
  listarFilas,
  obtenerImportacion,
  obtenerResumen,
  type ImportacionFila,
} from "../../lib/api/importaciones";
import { AppError } from "../../lib/api/client";
import { FilaRevisionRow } from "./FilaRevisionRow";

type Pestana =
  | "validas"
  | "sin_nombre"
  | "duplicados_codigo"
  | "duplicados_nombre"
  | "coincide_existente"
  | "otras"
  | "resueltas";

function esValidaLimpia(f: ImportacionFila): boolean {
  return (
    f.clasificacion === "producto_valido" &&
    !f.esDuplicadoCodigo &&
    !f.esPosibleDuplicadoNombre &&
    f.coincideProductoExistenteId === null &&
    f.resueltaEn === null
  );
}

export function RevisionImportacionPage() {
  const { id } = useParams<{ id: string }>();
  const importacionId = Number(id);
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const [pestana, setPestana] = useState<Pestana>("validas");
  const [error, setError] = useState<string | null>(null);

  const importacionQuery = useQuery({
    queryKey: ["importaciones", importacionId],
    queryFn: () => obtenerImportacion(importacionId),
  });
  const resumenQuery = useQuery({
    queryKey: ["importacionResumen", importacionId],
    queryFn: () => obtenerResumen(importacionId),
    refetchInterval: false,
  });
  const filasQuery = useQuery({
    queryKey: ["importacionFilas", importacionId],
    queryFn: () => listarFilas(importacionId),
  });

  function invalidarTodo() {
    queryClient.invalidateQueries({ queryKey: ["importaciones", importacionId] });
    queryClient.invalidateQueries({ queryKey: ["importacionResumen", importacionId] });
    queryClient.invalidateQueries({ queryKey: ["importacionFilas", importacionId] });
    queryClient.invalidateQueries({ queryKey: ["importaciones"] });
  }

  const filas = filasQuery.data ?? [];

  const grupos = useMemo(() => {
    return {
      validas: filas.filter(esValidaLimpia),
      sinNombre: filas.filter((f) => f.clasificacion === "requiere_revision" && f.resueltaEn === null),
      duplicadosCodigo: filas.filter((f) => f.esDuplicadoCodigo && f.resueltaEn === null),
      duplicadosNombre: filas.filter((f) => f.esPosibleDuplicadoNombre && f.resueltaEn === null),
      coincideExistente: filas.filter((f) => f.coincideProductoExistenteId !== null && f.resueltaEn === null),
      otras: filas.filter((f) => ["seccion", "ignorada", "error"].includes(f.clasificacion)),
      resueltas: filas.filter((f) => f.resueltaEn !== null),
    };
  }, [filas]);

  const aplicarMutation = useMutation({
    mutationFn: () => aplicarPendientes(importacionId),
    onSuccess: () => {
      setError(null);
      invalidarTodo();
    },
    onError: (e) => setError(e instanceof AppError ? e.userMessage : "No se pudieron aplicar las filas."),
  });

  const descartarMutation = useMutation({
    mutationFn: () => descartarImportacion(importacionId),
    onSuccess: () => {
      invalidarTodo();
      navigate("/productos/importar");
    },
    onError: (e) => setError(e instanceof AppError ? e.userMessage : "No se pudo descartar la importación."),
  });

  const importacion = importacionQuery.data;
  const resumen = resumenQuery.data;
  const soloLectura = importacion?.estado !== "en_revision";

  const TABS: { id: Pestana; etiqueta: string; cantidad: number }[] = [
    { id: "validas", etiqueta: "Listas para crear", cantidad: grupos.validas.length },
    { id: "sin_nombre", etiqueta: "Sin nombre", cantidad: grupos.sinNombre.length },
    { id: "duplicados_codigo", etiqueta: "Códigos duplicados", cantidad: grupos.duplicadosCodigo.length },
    { id: "duplicados_nombre", etiqueta: "Nombres duplicados", cantidad: grupos.duplicadosNombre.length },
    { id: "coincide_existente", etiqueta: "Ya existen en el catálogo", cantidad: grupos.coincideExistente.length },
    { id: "otras", etiqueta: "Secciones / ignoradas / errores", cantidad: grupos.otras.length },
    { id: "resueltas", etiqueta: "Resueltas", cantidad: grupos.resueltas.length },
  ];

  const filasDeLaPestana: ImportacionFila[] =
    pestana === "validas"
      ? grupos.validas
      : pestana === "sin_nombre"
        ? grupos.sinNombre
        : pestana === "duplicados_codigo"
          ? grupos.duplicadosCodigo
          : pestana === "duplicados_nombre"
            ? grupos.duplicadosNombre
            : pestana === "coincide_existente"
              ? grupos.coincideExistente
              : pestana === "otras"
                ? grupos.otras
                : grupos.resueltas;

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-semibold text-ink">Revisión de importación</h1>
          <p className="text-sm text-ink-muted">{importacion?.archivoNombre}</p>
        </div>
        <span
          className={`rounded-full px-3 py-1 text-sm font-medium ${
            importacion?.estado === "confirmada"
              ? "bg-good/15 text-good"
              : importacion?.estado === "descartada"
                ? "bg-danger/15 text-danger"
                : "bg-warn/15 text-warn"
          }`}
        >
          {importacion?.estado === "confirmada"
            ? "Confirmada — ya se aplicó a la base"
            : importacion?.estado === "descartada"
              ? "Descartada — no se aplicó nada"
              : "Vista previa — todavía no se aplicó nada"}
        </span>
      </div>

      {resumen && (
        <div className="grid grid-cols-3 gap-3 rounded-lg border border-line bg-surface p-4 text-sm sm:grid-cols-6">
          <div>
            <p className="text-ink-muted">Total</p>
            <p className="font-mono text-lg text-ink">{resumen.total}</p>
          </div>
          <div>
            <p className="text-ink-muted">Válidos</p>
            <p className="font-mono text-lg text-ink">{resumen.validos}</p>
          </div>
          <div>
            <p className="text-ink-muted">Sin código</p>
            <p className="font-mono text-lg text-ink">{resumen.sinCodigo}</p>
          </div>
          <div>
            <p className="text-ink-muted">Sin nombre</p>
            <p className="font-mono text-lg text-ink">{resumen.sinNombre}</p>
          </div>
          <div>
            <p className="text-ink-muted">Dup. código</p>
            <p className="font-mono text-lg text-ink">{resumen.duplicadosCodigo}</p>
          </div>
          <div>
            <p className="text-ink-muted">Dup. nombre</p>
            <p className="font-mono text-lg text-ink">{resumen.duplicadosNombre}</p>
          </div>
          <div>
            <p className="text-ink-muted">Ya en catálogo</p>
            <p className="font-mono text-lg text-ink">{resumen.coincideExistente}</p>
          </div>
          <div>
            <p className="text-ink-muted">Ignoradas</p>
            <p className="font-mono text-lg text-ink">{resumen.ignoradas}</p>
          </div>
          <div>
            <p className="text-ink-muted">Errores</p>
            <p className="font-mono text-lg text-ink">{resumen.errores}</p>
          </div>
          <div>
            <p className="text-ink-muted">Pendientes</p>
            <p className="font-mono text-lg text-warn">{resumen.pendientes}</p>
          </div>
          <div>
            <p className="text-ink-muted">Resueltas</p>
            <p className="font-mono text-lg text-good">{resumen.resueltas}</p>
          </div>
        </div>
      )}

      {!soloLectura && grupos.validas.length > 0 && (
        <div className="flex items-center justify-between rounded-lg border border-line bg-surface-2 p-4">
          <p className="text-sm text-ink">
            <strong>{grupos.validas.length}</strong> filas están listas para crearse como producto nuevo, sin
            ningún problema detectado.
          </p>
          <button
            type="button"
            onClick={() => aplicarMutation.mutate()}
            disabled={aplicarMutation.isPending}
            className="rounded-md bg-accent px-4 py-2 text-sm font-medium text-accent-ink disabled:opacity-60"
          >
            {aplicarMutation.isPending
              ? "Aplicando..."
              : `Confirmar y crear estas ${grupos.validas.length} filas`}
          </button>
        </div>
      )}

      {error && <p className="text-sm text-danger">{error}</p>}

      <div className="flex flex-wrap gap-1 border-b border-line">
        {TABS.map((t) => (
          <button
            key={t.id}
            type="button"
            onClick={() => setPestana(t.id)}
            className={`rounded-t-md px-3 py-2 text-sm ${
              pestana === t.id
                ? "border-b-2 border-accent font-medium text-ink"
                : "text-ink-muted hover:text-ink"
            }`}
          >
            {t.etiqueta} ({t.cantidad})
          </button>
        ))}
      </div>

      <div className="rounded-lg border border-line bg-surface">
        {filasDeLaPestana.length === 0 && (
          <p className="px-3 py-6 text-center text-sm text-ink-muted">No hay filas en esta categoría.</p>
        )}
        {pestana === "otras" || pestana === "resueltas" || soloLectura
          ? filasDeLaPestana.map((f) => (
              <div key={f.id} className="flex items-center justify-between border-b border-line px-3 py-2 text-sm last:border-b-0">
                <div className="flex flex-wrap items-center gap-2">
                  <span className="font-mono text-xs text-ink-muted">fila {f.filaExcel}</span>
                  <span className="text-ink">{f.nombreExcel ?? "(sin nombre)"}</span>
                  <span className="rounded-full bg-surface-2 px-2 py-0.5 text-xs text-ink-muted">
                    {f.clasificacion}
                  </span>
                  {f.motivoError && <span className="text-xs text-danger">{f.motivoError}</span>}
                </div>
                {f.resueltaEn && (
                  <span className="text-xs text-ink-muted">
                    {f.decision === "crear_nuevo"
                      ? "creado"
                      : f.decision === "vincular_existente"
                        ? "vinculado"
                        : "omitido"}
                  </span>
                )}
              </div>
            ))
          : filasDeLaPestana.map((f) => <FilaRevisionRow key={f.id} fila={f} />)}
      </div>

      {!soloLectura && (
        <div className="flex justify-end">
          <button
            type="button"
            onClick={() => {
              if (window.confirm("¿Descartar esta importación? No se va a aplicar nada de lo que quedó pendiente.")) {
                descartarMutation.mutate();
              }
            }}
            disabled={descartarMutation.isPending}
            className="rounded-md border border-line px-3 py-1.5 text-sm text-ink-muted hover:bg-surface-2"
          >
            Descartar importación
          </button>
        </div>
      )}
    </div>
  );
}
