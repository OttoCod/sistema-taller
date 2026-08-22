import { invoke } from "./client";

export type CodigoFabricante = {
  codigo: string;
  fabricanteNombre: string | null;
  observacion: string | null;
};

export type EstadoProducto = "activo" | "inactivo";

export type Producto = {
  id: number;
  codigoInterno: string;
  nombre: string;
  marcaId: number | null;
  marcaNombre: string | null;
  categoriaId: number | null;
  categoriaNombre: string | null;
  descripcion: string | null;
  observaciones: string | null;
  /** Centavos. Ver src/lib/money.ts para convertir a pesos. */
  costoActual: number | null;
  precioVentaActual: number | null;
  precioPublicoReferencia: number | null;
  precioActualizadoEn: string | null;
  estado: EstadoProducto;
  /** Agregado en la Fase 5: el carrito de venta necesita mostrar disponibilidad. */
  stockActual: number;
  /** "1566, 00034420" -- resumen para el listado, ver ProductoDetalle.codigosFabricante para el detalle completo. */
  codigosFabricanteResumen: string | null;
  /** Agregado en la Fase 3: código traído por una importación de Excel, solo trazabilidad. */
  codigoLegado: string | null;
};

export type ProductoDetalle = Producto & {
  codigosFabricante: CodigoFabricante[];
};

export type GuardarProducto = {
  nombre: string;
  marcaId: number | null;
  categoriaId: number | null;
  descripcion: string | null;
  observaciones: string | null;
  costoActual: number | null;
  precioVentaActual: number | null;
  precioPublicoReferencia: number | null;
  estado: EstadoProducto;
  codigosFabricante: CodigoFabricante[];
};

export function listarProductos() {
  return invoke<Producto[]>("productos_listar");
}

export function obtenerProducto(id: number) {
  return invoke<ProductoDetalle>("productos_obtener", { id });
}

export function crearProducto(datos: GuardarProducto) {
  return invoke<ProductoDetalle>("productos_crear", { datos });
}

export function actualizarProducto(id: number, datos: GuardarProducto) {
  return invoke<ProductoDetalle>("productos_actualizar", { id, datos });
}

/** Filtrado por FTS5 en el backend; el llamador se encarga del reordenamiento difuso (ver CatalogoPage). */
export function buscarProductos(consulta: string) {
  return invoke<Producto[]>("productos_buscar", { consulta });
}
