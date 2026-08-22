import { invoke } from "./client";

export type EstadoCompra = "registrada" | "anulada";

export type Compra = {
  id: number;
  proveedorId: number;
  proveedorNombre: string;
  numeroFactura: string | null;
  fecha: string;
  estado: EstadoCompra;
  /** Centavos. */
  subtotal: number;
  total: number;
};

export type DetalleCompra = {
  id: number;
  productoId: number;
  productoNombre: string;
  codigoInterno: string;
  cantidad: number;
  costoUnitario: number;
  subtotal: number;
};

export type CompraDetalle = Compra & {
  detalles: DetalleCompra[];
};

export type ItemCompra = {
  productoId: number;
  cantidad: number;
  costoUnitario: number;
};

export type CrearCompra = {
  proveedorId: number;
  numeroFactura: string | null;
  items: ItemCompra[];
};

export function listarCompras() {
  return invoke<Compra[]>("compras_listar");
}

export function obtenerCompra(id: number) {
  return invoke<CompraDetalle>("compras_obtener", { id });
}

export function crearCompra(datos: CrearCompra) {
  return invoke<CompraDetalle>("compras_crear", { datos });
}
