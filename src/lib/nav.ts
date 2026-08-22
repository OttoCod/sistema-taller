// Fuente unica de la navegacion (seccion 29 del documento de arquitectura).
// Sidebar y rutas se generan a partir de esta lista: agregar una pantalla
// nueva en una fase futura es agregar una entrada aca, no tocar el router
// ni el sidebar por separado.

export type NavLeaf = {
  label: string;
  path: string;
};

export type NavItem = NavLeaf & {
  children?: NavLeaf[];
};

export const NAV_SECTIONS: NavItem[] = [
  { label: "Inicio", path: "/" },
  { label: "Nueva venta", path: "/ventas/nueva" },
  {
    label: "Productos",
    path: "/productos/catalogo",
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
    children: [
      { label: "Nueva recepción", path: "/compras/nueva" },
      { label: "Historial", path: "/compras/historial" },
    ],
  },
  { label: "Proveedores", path: "/proveedores" },
  {
    label: "Clientes",
    path: "/clientes",
    children: [
      { label: "Clientes", path: "/clientes" },
      { label: "Cuentas pendientes", path: "/clientes/cuentas-pendientes" },
    ],
  },
  { label: "Caja", path: "/caja" },
  {
    label: "Ventas",
    path: "/ventas/historial",
    children: [{ label: "Historial", path: "/ventas/historial" }],
  },
  { label: "Configuración", path: "/configuracion" },
];

export function flattenNav(items: NavItem[]): NavLeaf[] {
  return items.flatMap((item) => [
    { label: item.label, path: item.path },
    ...(item.children ?? []),
  ]);
}
