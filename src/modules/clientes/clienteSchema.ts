import { z } from "zod";
import type { Cliente, GuardarCliente } from "../../lib/api/clientes";

export const clienteFormSchema = z.object({
  nombre: z.string().trim().min(1, "El nombre es obligatorio"),
  telefono: z.string(),
  dniCuit: z.string(),
  direccion: z.string(),
  observaciones: z.string(),
});

export type ClienteFormValues = z.infer<typeof clienteFormSchema>;

export const clienteFormVacio: ClienteFormValues = {
  nombre: "",
  telefono: "",
  dniCuit: "",
  direccion: "",
  observaciones: "",
};

function opcional(valor: string): string | null {
  const limpio = valor.trim();
  return limpio === "" ? null : limpio;
}

export function formValuesAGuardarCliente(valores: ClienteFormValues): GuardarCliente {
  return {
    nombre: valores.nombre.trim(),
    telefono: opcional(valores.telefono),
    dniCuit: opcional(valores.dniCuit),
    direccion: opcional(valores.direccion),
    observaciones: opcional(valores.observaciones),
  };
}

export function clienteAFormValues(cliente: Cliente): ClienteFormValues {
  return {
    nombre: cliente.nombre,
    telefono: cliente.telefono ?? "",
    dniCuit: cliente.dniCuit ?? "",
    direccion: cliente.direccion ?? "",
    observaciones: cliente.observaciones ?? "",
  };
}
