import { invoke } from "./client";

export type EstadoStock = "sin_stock" | "bajo" | "ok";

export type ProductoStock = {
  id: number;
  codigoInterno: string;
  nombre: string;
  marcaNombre: string | null;
  categoriaNombre: string | null;
  stockActual: number;
  stockMinimo: number;
  estadoStock: EstadoStock;
};

export function listarStock() {
  return invoke<ProductoStock[]>("stock_listar");
}

export function listarReposicion() {
  return invoke<ProductoStock[]>("stock_listar_reposicion");
}

/** Único movimiento posible en la Fase 4: ajuste manual con motivo obligatorio. */
export function ajustarStock(productoId: number, nuevaCantidad: number, motivo: string) {
  return invoke<void>("stock_ajustar", {
    productoId,
    datos: { nuevaCantidad, motivo },
  });
}

export function actualizarStockMinimo(productoId: number, stockMinimo: number) {
  return invoke<void>("stock_actualizar_minimo", { productoId, stockMinimo });
}
