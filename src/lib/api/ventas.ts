import { invoke } from "./client";

export type EstadoVenta = "confirmada" | "anulada";

export type Venta = {
  id: number;
  numero: number;
  clienteId: number;
  clienteNombre: string;
  fecha: string;
  estado: EstadoVenta;
  /** Centavos. */
  subtotal: number;
  descuentoTotal: number;
  total: number;
};

export type DetalleVenta = {
  id: number;
  productoId: number;
  productoNombre: string;
  codigoInterno: string;
  cantidad: number;
  precioUnitario: number;
  descuento: number;
  subtotal: number;
};

export type PagoVenta = {
  id: number;
  metodoPagoId: number;
  metodoPagoNombre: string;
  monto: number;
};

export type VentaDetalle = Venta & {
  detalles: DetalleVenta[];
  pagos: PagoVenta[];
};

export type ItemCarrito = {
  productoId: number;
  cantidad: number;
  precioUnitario: number;
  descuento: number;
};

export type PagoInput = {
  metodoPagoId: number;
  monto: number;
};

export type CrearVenta = {
  clienteId: number | null;
  items: ItemCarrito[];
  pagos: PagoInput[];
};

export function listarVentas() {
  return invoke<Venta[]>("ventas_listar");
}

export function obtenerVenta(id: number) {
  return invoke<VentaDetalle>("ventas_obtener", { id });
}

export function crearVenta(datos: CrearVenta) {
  return invoke<VentaDetalle>("ventas_crear", { datos });
}
