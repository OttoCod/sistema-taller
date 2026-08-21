import { invoke } from "./client";

export type Categoria = {
  id: number;
  nombre: string;
  categoriaPadreId: number | null;
  categoriaPadreNombre: string | null;
};

export function listarCategorias() {
  return invoke<Categoria[]>("categorias_listar");
}

export function crearCategoria(nombre: string, categoriaPadreId: number | null = null) {
  return invoke<Categoria>("categorias_crear", { datos: { nombre, categoriaPadreId } });
}
