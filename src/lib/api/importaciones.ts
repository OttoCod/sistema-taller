import { invoke } from "./client";

export type EstadoImportacion = "en_revision" | "confirmada" | "descartada";

export type Importacion = {
  id: number;
  archivoNombre: string;
  archivoHash: string;
  hojaDetectada: string;
  filaEncabezadoDetectada: number;
  columnasDetectadasJson: string;
  estado: EstadoImportacion;
  totalFilas: number;
  creadaEn: string;
  cerradaEn: string | null;
};

export type ClasificacionFila =
  | "producto_valido"
  | "seccion"
  | "ignorada"
  | "requiere_revision"
  | "error";

export type DecisionFila = "crear_nuevo" | "vincular_existente" | "omitir";

export type ImportacionFila = {
  id: number;
  importacionId: number;
  filaExcel: number;
  codigoExcel: string | null;
  nombreExcel: string | null;
  /** Centavos. */
  precioListaCentavos: number | null;
  categoriaExcelTexto: string | null;
  preciosCalculadosJson: string | null;
  clasificacion: ClasificacionFila;
  motivoError: string | null;
  esDuplicadoCodigo: boolean;
  esPosibleDuplicadoNombre: boolean;
  coincideProductoExistenteId: number | null;
  decision: DecisionFila | null;
  productoVinculadoId: number | null;
  actualizarCostoEnVinculo: boolean;
  productoId: number | null;
  resueltaEn: string | null;
};

export type ResumenImportacion = {
  total: number;
  validos: number;
  sinCodigo: number;
  sinNombre: number;
  duplicadosCodigo: number;
  duplicadosNombre: number;
  coincideExistente: number;
  secciones: number;
  ignoradas: number;
  errores: number;
  pendientes: number;
  resueltas: number;
};

export type ColumnaDetectada = {
  encabezado: string;
  indice: number;
};

export type ColumnasDetectadas = {
  codigo: ColumnaDetectada | null;
  nombre: ColumnaDetectada;
  precio: ColumnaDetectada;
  derivadas: ColumnaDetectada[];
};

export type ResolverFila = {
  decision: DecisionFila;
  productoVinculadoId: number | null;
  actualizarCostoEnVinculo: boolean;
  nombreCorregido: string | null;
  codigoCorregido: string | null;
};

export function procesarArchivo(ruta: string, archivoNombre: string) {
  return invoke<Importacion>("importaciones_procesar_archivo", { ruta, archivoNombre });
}

export function listarImportaciones() {
  return invoke<Importacion[]>("importaciones_listar");
}

export function obtenerImportacion(id: number) {
  return invoke<Importacion>("importaciones_obtener", { id });
}

export function obtenerResumen(id: number) {
  return invoke<ResumenImportacion>("importaciones_resumen", { id });
}

export function listarFilas(id: number) {
  return invoke<ImportacionFila[]>("importaciones_listar_filas", { id });
}

export function buscarConfirmadaConMismoHash(hash: string, excluyendoId: number) {
  return invoke<Importacion | null>("importaciones_buscar_confirmada_con_mismo_hash", {
    hash,
    excluyendoId,
  });
}

export function resolverFila(filaId: number, datos: ResolverFila) {
  return invoke<ImportacionFila>("importaciones_resolver_fila", { filaId, datos });
}

export function aplicarPendientes(id: number) {
  return invoke<number>("importaciones_aplicar_pendientes", { id });
}

export function descartarImportacion(id: number) {
  return invoke<Importacion>("importaciones_descartar", { id });
}

export function parsearColumnasDetectadas(json: string): ColumnasDetectadas {
  return JSON.parse(json) as ColumnasDetectadas;
}
