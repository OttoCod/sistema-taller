import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { listarReposicion, type ProductoStock } from "../../lib/api/stock";
import { AppError } from "../../lib/api/client";
import { StockTable } from "./StockTable";
import { AjusteStockDialog } from "./AjusteStockDialog";

export function ReposicionPage() {
  const { data: productos = [], error, isLoading } = useQuery({
    queryKey: ["stock", "reposicion"],
    queryFn: listarReposicion,
  });
  const [productoAjustando, setProductoAjustando] = useState<ProductoStock | null>(null);

  return (
    <div className="flex flex-col gap-4">
      <div>
        <h1 className="text-xl font-semibold text-ink">Reposición</h1>
        <p className="text-sm text-ink-muted">
          {productos.length} producto(s) en o por debajo del stock mínimo
        </p>
      </div>

      {error && (
        <p className="text-sm text-danger">
          {error instanceof AppError ? error.userMessage : "No se pudo cargar la reposición."}
        </p>
      )}

      <StockTable
        productos={productos}
        cargando={isLoading}
        mensajeVacio="No hay productos por debajo del stock mínimo."
        onAjustar={setProductoAjustando}
      />

      <AjusteStockDialog
        producto={productoAjustando}
        onOpenChange={(open) => !open && setProductoAjustando(null)}
      />
    </div>
  );
}
