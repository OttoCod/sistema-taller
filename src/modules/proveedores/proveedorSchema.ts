import { z } from "zod";
import type { GuardarProveedor, Proveedor } from "../../lib/api/proveedores";

export const proveedorFormSchema = z.object({
  nombre: z.string().trim().min(1, "El nombre es obligatorio"),
  telefono: z.string(),
  whatsapp: z.string(),
  email: z.string(),
  sitioWeb: z.string(),
  observaciones: z.string(),
});

export type ProveedorFormValues = z.infer<typeof proveedorFormSchema>;

export const proveedorFormVacio: ProveedorFormValues = {
  nombre: "",
  telefono: "",
  whatsapp: "",
  email: "",
  sitioWeb: "",
  observaciones: "",
};

function opcional(valor: string): string | null {
  const limpio = valor.trim();
  return limpio === "" ? null : limpio;
}

export function formValuesAGuardarProveedor(valores: ProveedorFormValues): GuardarProveedor {
  return {
    nombre: valores.nombre.trim(),
    telefono: opcional(valores.telefono),
    whatsapp: opcional(valores.whatsapp),
    email: opcional(valores.email),
    sitioWeb: opcional(valores.sitioWeb),
    observaciones: opcional(valores.observaciones),
  };
}

export function proveedorAFormValues(proveedor: Proveedor): ProveedorFormValues {
  return {
    nombre: proveedor.nombre,
    telefono: proveedor.telefono ?? "",
    whatsapp: proveedor.whatsapp ?? "",
    email: proveedor.email ?? "",
    sitioWeb: proveedor.sitioWeb ?? "",
    observaciones: proveedor.observaciones ?? "",
  };
}
