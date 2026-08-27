import { invoke } from "./client";

export type TipoComprobante = "ticket" | "a4";
export type TipoEventoComprobante = "impreso" | "pdf_generado";

export type Comprobante = {
  id: number;
  ventaId: number;
  numero: string;
  tipo: TipoComprobante;
  creadoEn: string;
};

export type ComprobanteEvento = {
  id: number;
  tipoEvento: TipoEventoComprobante;
  fecha: string;
};

/** Idempotente: si ya existe un comprobante de ese tipo para la venta, lo devuelve tal cual. */
export function obtenerOCrearComprobante(ventaId: number, tipo: TipoComprobante) {
  return invoke<Comprobante>("comprobantes_obtener_o_crear", { datos: { ventaId, tipo } });
}

/** Solo lectura: nunca genera un comprobante. */
export function listarComprobantesPorVenta(ventaId: number) {
  return invoke<Comprobante[]>("comprobantes_listar_por_venta", { ventaId });
}

export function registrarEventoComprobante(comprobanteId: number, tipoEvento: TipoEventoComprobante) {
  return invoke<void>("comprobantes_registrar_evento", { comprobanteId, tipoEvento });
}

export function listarEventosComprobante(comprobanteId: number) {
  return invoke<ComprobanteEvento[]>("comprobantes_listar_eventos", { comprobanteId });
}
