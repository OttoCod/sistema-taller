/**
 * Íconos de la navegación, dibujados acá en vez de instalar una librería:
 * son diez, no cambian seguido, y evitan sumar una dependencia (y un
 * `npm install` más) por unos pocos kilobytes de trazos.
 *
 * Todos comparten la misma caja de 24x24, el mismo grosor de línea y
 * `currentColor`, así que heredan el color del texto que los rodea y
 * quedan parejos entre sí.
 */
export type NombreIcono =
  | "inicio"
  | "venta"
  | "productos"
  | "compras"
  | "proveedores"
  | "clientes"
  | "caja"
  | "ventas"
  | "configuracion";

const TRAZOS: Record<NombreIcono, React.ReactNode> = {
  inicio: (
    <>
      <path d="M3 10.4 12 3.2l9 7.2" />
      <path d="M5.6 9.6V19.6a1.2 1.2 0 0 0 1.2 1.2h10.4a1.2 1.2 0 0 0 1.2-1.2V9.6" />
    </>
  ),
  venta: (
    <>
      <path d="M2.6 3.4h2.3l2.4 11.3a1.5 1.5 0 0 0 1.5 1.2h8.1a1.5 1.5 0 0 0 1.5-1.2l1.4-7H6.2" />
      <circle cx="9.2" cy="19.8" r="1.4" />
      <circle cx="17.6" cy="19.8" r="1.4" />
    </>
  ),
  productos: (
    <>
      <path d="M12 2.9 20.4 7v10L12 21.1 3.6 17V7z" />
      <path d="M3.6 7 12 11.2 20.4 7" />
      <path d="M12 11.2v9.9" />
    </>
  ),
  compras: (
    <>
      <path d="M2.8 16.4V6.9a.9.9 0 0 1 .9-.9h9.2a.9.9 0 0 1 .9.9v9.5" />
      <path d="M13.8 9.7h3.4l3 3.2v3.5h-6.4" />
      <circle cx="7.3" cy="17.6" r="1.6" />
      <circle cx="17.1" cy="17.6" r="1.6" />
      <path d="M8.9 17.6h6.6" />
    </>
  ),
  proveedores: (
    <>
      <path d="M3.8 9.8h16.4v10a1 1 0 0 1-1 1H4.8a1 1 0 0 1-1-1z" />
      <path d="M3.2 9.8 5 3.9h14l1.8 5.9" />
      <path d="M9.6 20.8v-6h4.8v6" />
    </>
  ),
  clientes: (
    <>
      <circle cx="9.2" cy="8.2" r="3.3" />
      <path d="M3 20.4c0-3.5 2.8-5.7 6.2-5.7s6.2 2.2 6.2 5.7" />
      <path d="M16.4 5.3a3.3 3.3 0 0 1 0 5.9" />
      <path d="M17.6 15c2.1.7 3.4 2.5 3.4 5.4" />
    </>
  ),
  caja: (
    <>
      <rect x="2.7" y="6.3" width="18.6" height="11.4" rx="1.4" />
      <circle cx="12" cy="12" r="2.5" />
      <path d="M6.2 12h.01" />
      <path d="M17.8 12h.01" />
    </>
  ),
  ventas: (
    <>
      <path d="M6.2 2.9h11.6v18.2l-2.9-1.9-2.9 1.9-2.9-1.9-2.9 1.9z" />
      <path d="M9.3 8.1h5.4" />
      <path d="M9.3 12.2h5.4" />
    </>
  ),
  configuracion: (
    <>
      <path d="M4 7.2h16" />
      <path d="M4 12h16" />
      <path d="M4 16.8h16" />
      <path d="M9.4 5.1v4.2" />
      <path d="M15.2 9.9v4.2" />
      <path d="M7.6 14.7v4.2" />
    </>
  ),
};

export function Icono({ nombre, className = "" }: { nombre: NombreIcono; className?: string }) {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={1.7}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      className={className}
    >
      {TRAZOS[nombre]}
    </svg>
  );
}
