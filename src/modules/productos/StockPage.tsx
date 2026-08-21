import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { listarStock, type ProductoStock } from "../../lib/api/stock";
import { AppError } from "../../lib/api/client";
import { StockTable } from "./StockTable";
import { AjusteStockDialog } from "./AjusteStockDialog";

export function StockPage() {
  const { data: productos = [], error, isLoading } = useQuery({
    queryKey: ["stock", "todos"],
    queryFn: listarStock,
  });
  const [productoAjustando, setProductoAjustando] = useState<ProductoStock | null>(null);

  return (
    <div className="flex flex-col gap-4">
      <div>
        <h1 className="text-xl font-semibold text-ink">Stock</h1>
        <p className="text-sm text-ink-muted">{productos.length} producto(s) activo(s)</p>
      </div>

      {error && (
        <p className="text-sm text-danger">
          {error instanceof AppError ? error.userMessage : "No se pudo cargar el stock."}
        </p>
      )}

      <StockTable
        productos={productos}
        cargando={isLoading}
        mensajeVacio="Todavía no hay productos activos con stock."
        onAjustar={setProductoAjustando}
      />

      <AjusteStockDialog
        producto={productoAjustando}
        onOpenChange={(open) => !open && setProductoAjustando(null)}
      />
    </div>
  );
}
