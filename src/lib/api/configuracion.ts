import { invoke } from "./client";

export type ConfiguracionNegocio = {
  nombre: string;
  direccion: string;
  telefono: string;
};

export function obtenerConfiguracionNegocio() {
  return invoke<ConfiguracionNegocio>("configuracion_obtener_negocio");
}

export function guardarConfiguracionNegocio(datos: ConfiguracionNegocio) {
  return invoke<void>("configuracion_guardar_negocio", { datos });
}
