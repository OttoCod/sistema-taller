import { invoke } from "./client";

export type MovimientoCuentaCorriente = {
  id: number;
  tipo: "venta_fiada" | "pago" | "ajuste" | "devolucion";
  monto: number;
  saldoResultante: number;
  metodoPagoNombre: string | null;
  fecha: string;
  observacion: string | null;
};

export function listarMovimientos(clienteId: number) {
  return invoke<MovimientoCuentaCorriente[]>("cuenta_corriente_listar_movimientos", { clienteId });
}

export function registrarPago(
  clienteId: number,
  monto: number,
  metodoPagoId: number,
  observacion: string | null,
) {
  return invoke<void>("cuenta_corriente_registrar_pago", {
    clienteId,
    datos: { monto, metodoPagoId, observacion },
  });
}

export function ajustarCuentaCorriente(clienteId: number, monto: number, motivo: string) {
  return invoke<void>("cuenta_corriente_ajustar", {
    clienteId,
    datos: { monto, motivo },
  });
}
