import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { resolverFila, type ImportacionFila } from "../../lib/api/importaciones";
import type { Producto } from "../../lib/api/productos";
import { AppError } from "../../lib/api/client";
import { formatearCentavos } from "../../lib/money";
import { ProductoVincularSelector } from "./ProductoVincularSelector";

type Props = {
  fila: ImportacionFila;
};

export function FilaRevisionRow({ fila }: Props) {
  const queryClient = useQueryClient();
  const [modo, setModo] = useState<"ninguno" | "vincular">("ninguno");
  const [nombreCorregido, setNombreCorregido] = useState("");
  const [productoElegido, setProductoElegido] = useState<Producto | null>(null);
  const [actualizarCosto, setActualizarCosto] = useState(false);
  const [error, setError] = useState<string | null>(null);

  function invalidar() {
    queryClient.invalidateQueries({ queryKey: ["importaciones", fila.importacionId] });
    queryClient.invalidateQueries({ queryKey: ["importacionResumen", fila.importacionId] });
    queryClient.invalidateQueries({ queryKey: ["importacionFilas", fila.importacionId] });
    queryClient.invalidateQueries({ queryKey: ["productos"] });
  }

  const mutation = useMutation({
    mutationFn: (
      datos: Parameters<typeof resolverFila>[1],
    ) => resolverFila(fila.id, datos),
    onSuccess: () => {
      setError(null);
      invalidar();
    },
    onError: (e) => setError(e instanceof AppError ? e.userMessage : "No se pudo resolver la fila."),
  });

  const necesitaNombre = fila.nombreExcel === null;

  function crearNuevo() {
    setError(null);
    if (necesitaNombre && nombreCorregido.trim() === "") {
      setError("Escribí un nombre antes de crear el producto.");
      return;
    }
    mutation.mutate({
      decision: "crear_nuevo",
      productoVinculadoId: null,
      actualizarCostoEnVinculo: false,
      nombreCorregido: necesitaNombre ? nombreCorregido.trim() : null,
      codigoCorregido: null,
    });
  }

  function confirmarVinculo() {
    setError(null);
    if (!productoElegido) {
      setError("Elegí a qué producto vincular esta fila.");
      return;
    }
    mutation.mutate({
      decision: "vincular_existente",
      productoVinculadoId: productoElegido.id,
      actualizarCostoEnVinculo: actualizarCosto,
      nombreCorregido: null,
      codigoCorregido: null,
    });
  }

  function omitir() {
    setError(null);
    mutation.mutate({
      decision: "omitir",
      productoVinculadoId: null,
      actualizarCostoEnVinculo: false,
      nombreCorregido: null,
      codigoCorregido: null,
    });
  }

  return (
    <div className="flex flex-col gap-2 border-b border-line px-3 py-3 last:border-b-0">
      <div className="flex flex-wrap items-center gap-2 text-sm">
        <span className="font-mono text-xs text-ink-muted">fila {fila.filaExcel}</span>
        {fila.codigoExcel && (
          <span className="font-mono text-xs text-ink-muted">cód. {fila.codigoExcel}</span>
        )}
        <span className="font-medium text-ink">{fila.nombreExcel ?? "(sin nombre)"}</span>
        <span className="ml-auto font-mono text-xs text-ink-muted">
          {formatearCentavos(fila.precioListaCentavos ?? 0)}
        </span>
      </div>

      <div className="flex flex-wrap gap-1">
        {fila.esDuplicadoCodigo && (
          <span className="rounded-full bg-warn/15 px-2 py-0.5 text-xs text-warn">Código duplicado</span>
        )}
        {fila.esPosibleDuplicadoNombre && (
          <span className="rounded-full bg-warn/15 px-2 py-0.5 text-xs text-warn">Nombre duplicado</span>
        )}
        {fila.coincideProductoExistenteId !== null && (
          <span className="rounded-full bg-warn/15 px-2 py-0.5 text-xs text-warn">Ya existe en el catálogo</span>
        )}
      </div>

      {necesitaNombre && (
        <input
          value={nombreCorregido}
          onChange={(e) => {
            const valor = e.currentTarget.value;
            setNombreCorregido(valor);
          }}
          placeholder="Completá el nombre para poder crear el producto..."
          className="rounded-md border border-line bg-surface px-2 py-1.5 text-sm focus:border-accent focus:outline-none"
        />
      )}

      {modo === "vincular" && (
        <div className="rounded-md border border-line bg-surface-2 p-3">
          {!productoElegido ? (
            <ProductoVincularSelector onSeleccionar={setProductoElegido} />
          ) : (
            <div className="flex flex-col gap-2 text-sm">
              <div className="flex items-center justify-between">
                <span className="text-ink">{productoElegido.nombre}</span>
                <button
                  type="button"
                  onClick={() => setProductoElegido(null)}
                  className="text-xs text-ink-muted hover:underline"
                >
                  cambiar
                </button>
              </div>
              <div className="flex justify-between text-ink-muted">
                <span>Costo actual del producto</span>
                <span className="font-mono">{formatearCentavos(productoElegido.costoActual ?? 0)}</span>
              </div>
              <div className="flex justify-between text-ink-muted">
                <span>Valor de esta fila del Excel</span>
                <span className="font-mono">{formatearCentavos(fila.precioListaCentavos ?? 0)}</span>
              </div>
              <label className="flex items-center gap-2 text-ink">
                <input
                  type="checkbox"
                  checked={actualizarCosto}
                  onChange={(e) => {
                    const marcado = e.currentTarget.checked;
                    setActualizarCosto(marcado);
                  }}
                />
                Actualizar el costo del producto con el valor del Excel
              </label>
              <p className="text-xs text-ink-muted">
                Nombre, marca, categoría y precio de venta del producto no se tocan nunca.
              </p>
              <button
                type="button"
                onClick={confirmarVinculo}
                disabled={mutation.isPending}
                className="self-start rounded-md bg-accent px-3 py-1.5 text-sm font-medium text-accent-ink disabled:opacity-60"
              >
                {mutation.isPending ? "Vinculando..." : "Confirmar vínculo"}
              </button>
            </div>
          )}
        </div>
      )}

      {error && <p className="text-sm text-danger">{error}</p>}

      {modo === "ninguno" && (
        <div className="flex gap-2">
          <button
            type="button"
            onClick={crearNuevo}
            disabled={mutation.isPending}
            className="rounded-md bg-accent px-3 py-1.5 text-sm font-medium text-accent-ink disabled:opacity-60"
          >
            Crear como nuevo
          </button>
          <button
            type="button"
            onClick={() => setModo("vincular")}
            disabled={mutation.isPending}
            className="rounded-md border border-line px-3 py-1.5 text-sm text-ink hover:bg-surface-2"
          >
            Vincular a producto existente...
          </button>
          <button
            type="button"
            onClick={omitir}
            disabled={mutation.isPending}
            className="rounded-md border border-line px-3 py-1.5 text-sm text-ink-muted hover:bg-surface-2"
          >
            Omitir
          </button>
        </div>
      )}
      {modo === "vincular" && (
        <button
          type="button"
          onClick={() => {
            setModo("ninguno");
            setProductoElegido(null);
          }}
          className="self-start text-xs text-ink-muted hover:underline"
        >
          cancelar vínculo
        </button>
      )}
    </div>
  );
}
