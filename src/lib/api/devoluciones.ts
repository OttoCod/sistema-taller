import { invoke } from "./client";

export type MetodoDevolucion = "reembolso_efectivo" | "nota_credito" | "cambio_producto" | "reduccion_deuda";

export type EstadoProductoDevuelto = "vuelve_a_stock" | "en_revision" | "defectuoso" | "dañado";

export type Devolucion = {
  id: number;
  ventaId: number;
  fecha: string;
  motivo: string;
  metodoDevolucion: MetodoDevolucion;
  /** Centavos. */
  totalDevuelto: number;
};

export type DevolucionDetalleFila = {
  id: number;
  ventaDetalleId: number;
  productoId: number;
  productoNombre: string;
  cantidad: number;
  /** Centavos. */
  monto: number;
  estadoProducto: EstadoProductoDevuelto;
  observacion: string | null;
};

export type DevolucionConDetalles = Devolucion & {
  detalles: DevolucionDetalleFila[];
};

export type ItemDevolucion = {
  ventaDetalleId: number;
  cantidad: number;
  estadoProducto: EstadoProductoDevuelto;
  observacion: string | null;
};

export type CrearDevolucion = {
  ventaId: number;
  motivo: string;
  metodoDevolucion: MetodoDevolucion;
  items: ItemDevolucion[];
};

export function crearDevolucion(datos: CrearDevolucion) {
  return invoke<DevolucionConDetalles>("devoluciones_crear", { datos });
}

export function listarDevolucionesPorVenta(ventaId: number) {
  return invoke<DevolucionConDetalles[]>("devoluciones_listar_por_venta", { ventaId });
}
