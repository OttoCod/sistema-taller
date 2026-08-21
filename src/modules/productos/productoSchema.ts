import { z } from "zod";
import type {
  CodigoFabricante,
  EstadoProducto,
  GuardarProducto,
  ProductoDetalle,
} from "../../lib/api/productos";
import { centavosAPesos, pesosACentavos } from "../../lib/money";

const montoOpcional = z.string().trim().refine(
  (valor) => valor === "" || (!Number.isNaN(Number(valor)) && Number(valor) >= 0),
  { message: "Tiene que ser un número mayor o igual a 0" },
);

export const productoFormSchema = z.object({
  nombre: z.string().trim().min(1, "El nombre es obligatorio"),
  marcaId: z.number().nullable(),
  categoriaId: z.number().nullable(),
  descripcion: z.string(),
  observaciones: z.string(),
  costo: montoOpcional,
  precioVenta: montoOpcional,
  precioPublico: montoOpcional,
  estado: z.enum(["activo", "inactivo"]),
  codigosFabricante: z.array(
    z.object({
      codigo: z.string().trim().min(1, "El código no puede estar vacío"),
      fabricanteNombre: z.string(),
      observacion: z.string(),
    }),
  ),
});

export type ProductoFormValues = z.infer<typeof productoFormSchema>;

export const productoFormVacio: ProductoFormValues = {
  nombre: "",
  marcaId: null,
  categoriaId: null,
  descripcion: "",
  observaciones: "",
  costo: "",
  precioVenta: "",
  precioPublico: "",
  estado: "activo",
  codigosFabricante: [],
};

function textoOpcional(valor: string): string | null {
  const limpio = valor.trim();
  return limpio === "" ? null : limpio;
}

function montoOpcionalACentavos(valor: string): number | null {
  return valor.trim() === "" ? null : pesosACentavos(Number(valor));
}

function codigoFabricanteAGuardar(codigo: ProductoFormValues["codigosFabricante"][number]): CodigoFabricante {
  return {
    codigo: codigo.codigo.trim(),
    fabricanteNombre: textoOpcional(codigo.fabricanteNombre),
    observacion: textoOpcional(codigo.observacion),
  };
}

export function formValuesAGuardarProducto(valores: ProductoFormValues): GuardarProducto {
  return {
    nombre: valores.nombre.trim(),
    marcaId: valores.marcaId,
    categoriaId: valores.categoriaId,
    descripcion: textoOpcional(valores.descripcion),
    observaciones: textoOpcional(valores.observaciones),
    costoActual: montoOpcionalACentavos(valores.costo),
    precioVentaActual: montoOpcionalACentavos(valores.precioVenta),
    precioPublicoReferencia: montoOpcionalACentavos(valores.precioPublico),
    estado: valores.estado as EstadoProducto,
    codigosFabricante: valores.codigosFabricante
      .map(codigoFabricanteAGuardar)
      .filter((c) => c.codigo !== ""),
  };
}

function centavosATexto(valor: number | null): string {
  return valor === null ? "" : String(centavosAPesos(valor));
}

export function productoDetalleAFormValues(detalle: ProductoDetalle): ProductoFormValues {
  return {
    nombre: detalle.nombre,
    marcaId: detalle.marcaId,
    categoriaId: detalle.categoriaId,
    descripcion: detalle.descripcion ?? "",
    observaciones: detalle.observaciones ?? "",
    costo: centavosATexto(detalle.costoActual),
    precioVenta: centavosATexto(detalle.precioVentaActual),
    precioPublico: centavosATexto(detalle.precioPublicoReferencia),
    estado: detalle.estado,
    codigosFabricante: detalle.codigosFabricante.map((c) => ({
      codigo: c.codigo,
      fabricanteNombre: c.fabricanteNombre ?? "",
      observacion: c.observacion ?? "",
    })),
  };
}
