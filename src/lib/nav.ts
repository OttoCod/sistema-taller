// Fuente unica de la navegacion (seccion 29 del documento de arquitectura).
// Sidebar y rutas se generan a partir de esta lista: agregar una pantalla
// nueva en una fase futura es agregar una entrada aca, no tocar el router
// ni el sidebar por separado.

import type { NombreIcono } from "../components/ui/Icono";

export type NavLeaf = {
  label: string;
  path: string;
};

export type NavItem = NavLeaf & {
  children?: NavLeaf[];
  /** Solo las secciones de primer nivel lo llevan: los hijos ya quedan
      acotados por la sangría, y repetir íconos ahí agrega ruido. */
  icono?: NombreIcono;
};

export const NAV_SECTIONS: NavItem[] = [
  { label: "Inicio", path: "/", icono: "inicio" },
  { label: "Nueva venta", path: "/ventas/nueva", icono: "venta" },
  {
    label: "Productos",
    path: "/productos/catalogo",
    icono: "productos",
    children: [
      { label: "Catálogo", path: "/productos/catalogo" },
      { label: "Stock", path: "/productos/stock" },
      { label: "Reposición", path: "/productos/reposicion" },
      { label: "Importar Excel", path: "/productos/importar" },
    ],
  },
  {
    label: "Compras",
    path: "/compras/nueva",
    icono: "compras",
    children: [
      { label: "Nueva recepción", path: "/compras/nueva" },
      { label: "Historial", path: "/compras/historial" },
    ],
  },
  { label: "Proveedores", path: "/proveedores", icono: "proveedores" },
  {
    label: "Clientes",
    path: "/clientes",
    icono: "clientes",
    children: [
      { label: "Clientes", path: "/clientes" },
      { label: "Cuentas pendientes", path: "/clientes/cuentas-pendientes" },
    ],
  },
  { label: "Caja", path: "/caja", icono: "caja" },
  {
    label: "Ventas",
    path: "/ventas/historial",
    icono: "ventas",
    children: [{ label: "Historial", path: "/ventas/historial" }],
  },
  { label: "Configuración", path: "/configuracion", icono: "configuracion" },
];

export function flattenNav(items: NavItem[]): NavLeaf[] {
  return items.flatMap((item) => [
    { label: item.label, path: item.path },
    ...(item.children ?? []),
  ]);
}
