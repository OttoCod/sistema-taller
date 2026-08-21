import { invoke } from "./client";

export type MetodoPago = {
  id: number;
  nombre: string;
};

export function listarMetodosPago() {
  return invoke<MetodoPago[]>("metodos_pago_listar");
}
