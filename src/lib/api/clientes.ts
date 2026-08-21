import { invoke } from "./client";

export const ID_CONSUMIDOR_FINAL = 1;

export type Cliente = {
  id: number;
  nombre: string;
  telefono: string | null;
  dniCuit: string | null;
  direccion: string | null;
  observaciones: string | null;
  /** Centavos. Positivo = debe; negativo = tiene a favor. */
  saldoCuentaCorriente: number;
};

export type GuardarCliente = {
  nombre: string;
  telefono: string | null;
  dniCuit: string | null;
  direccion: string | null;
  observaciones: string | null;
};

export type ClienteConDeuda = {
  id: number;
  nombre: string;
  telefono: string | null;
  saldoCuentaCorriente: number;
  fechaUltimoMovimiento: string | null;
};

export function listarClientes() {
  return invoke<Cliente[]>("clientes_listar");
}

export function buscarClientes(consulta: string) {
  return invoke<Cliente[]>("clientes_buscar", { consulta });
}

export function obtenerCliente(id: number) {
  return invoke<Cliente>("clientes_obtener", { id });
}

export function crearCliente(datos: GuardarCliente) {
  return invoke<Cliente>("clientes_crear", { datos });
}

export function actualizarCliente(id: number, datos: GuardarCliente) {
  return invoke<Cliente>("clientes_actualizar", { id, datos });
}

export function listarCuentasPendientes() {
  return invoke<ClienteConDeuda[]>("clientes_listar_cuentas_pendientes");
}
