import { invoke } from "./client";

export type Proveedor = {
  id: number;
  nombre: string;
  telefono: string | null;
  whatsapp: string | null;
  email: string | null;
  sitioWeb: string | null;
  observaciones: string | null;
  activo: boolean;
};

export type GuardarProveedor = {
  nombre: string;
  telefono: string | null;
  whatsapp: string | null;
  email: string | null;
  sitioWeb: string | null;
  observaciones: string | null;
};

export function listarProveedores() {
  return invoke<Proveedor[]>("proveedores_listar");
}

export function buscarProveedores(consulta: string) {
  return invoke<Proveedor[]>("proveedores_buscar", { consulta });
}

export function crearProveedor(datos: GuardarProveedor) {
  return invoke<Proveedor>("proveedores_crear", { datos });
}

export function actualizarProveedor(id: number, datos: GuardarProveedor) {
  return invoke<Proveedor>("proveedores_actualizar", { id, datos });
}
