// Todo monto viaja como centavos (INTEGER) entre la UI y la base de datos
// -- nunca de punto flotante (ver docs/ESQUEMA_BD.md). Estas son las únicas
// funciones que deberían cruzar esa frontera.

export function pesosACentavos(pesos: number): number {
  return Math.round(pesos * 100);
}

export function centavosAPesos(centavos: number): number {
  return centavos / 100;
}

// Sin decimales por defecto para Argentina (punto G, confirmado). Cuando
// exista la pantalla de Configuración esto pasa a leer de `configuracion`.
const formateador = new Intl.NumberFormat("es-AR", {
  style: "currency",
  currency: "ARS",
  maximumFractionDigits: 0,
});

export function formatearCentavos(centavos: number | null | undefined): string {
  if (centavos === null || centavos === undefined) {
    return "—";
  }
  return formateador.format(centavosAPesos(centavos));
}
