import { invoke } from "./client";

export type ProductoProveedor = {
  id: number;
  productoId: number;
  productoNombre: string;
  codigoInterno: string;
  proveedorId: number;
  codigoProveedor: string | null;
  urlProducto: string | null;
  urlBusqueda: string | null;
  esPrincipal: boolean;
  activo: boolean;
};

export type GuardarProductoProveedor = {
  productoId: number;
  codigoProveedor: string | null;
  urlProducto: string | null;
  urlBusqueda: string | null;
  esPrincipal: boolean;
};

export function listarProductoProveedores(proveedorId: number) {
  return invoke<ProductoProveedor[]>("producto_proveedores_listar", { proveedorId });
}

export function agregarProductoProveedor(proveedorId: number, datos: GuardarProductoProveedor) {
  return invoke<ProductoProveedor>("producto_proveedores_agregar", { proveedorId, datos });
}

export function quitarProductoProveedor(id: number) {
  return invoke<void>("producto_proveedores_quitar", { id });
}
