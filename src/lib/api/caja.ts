import { invoke } from "./client";

export type MontoPorMetodo = {
  metodoPagoId: number;
  metodoPagoNombre: string;
  /** Centavos. */
  monto: number;
};

export type ResumenCaja = {
  fecha: string;
  porMetodo: MontoPorMetodo[];
  /** Centavos. Suma de porMetodo excluyendo cuenta_corriente. */
  totalCobrado: number;
  /** Centavos. Lo que quedó fiado ese día (informativo, no es caja). */
  totalFiado: number;
  cantidadVentas: number;
};

export function obtenerResumenCaja(fecha: string) {
  return invoke<ResumenCaja>("caja_resumen", { fecha });
}
