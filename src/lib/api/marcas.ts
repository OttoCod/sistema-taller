import { invoke } from "./client";

export type Marca = {
  id: number;
  nombre: string;
};

export function listarMarcas() {
  return invoke<Marca[]>("marcas_listar");
}

/** Si ya existe una marca con ese nombre, el backend devuelve esa misma en vez de duplicarla. */
export function crearMarca(nombre: string) {
  return invoke<Marca>("marcas_crear", { datos: { nombre } });
}
