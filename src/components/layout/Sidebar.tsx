import { NavLink } from "react-router-dom";
import { NAV_SECTIONS } from "../../lib/nav";

/**
 * Reproduce el bloque de texto del logo (ESPÍNDOLA en blanco sobre negro,
 * MOTORESPUESTOS en amarillo debajo). Es una versión tipográfica, no el
 * logo real: si más adelante se suma el archivo de imagen, se reemplaza
 * solo este componente.
 *
 * Ojo con el nombre: es Espíndola (con d) Motorespuestos (con s), tal
 * como está en el logo y en el Instagram del local. Los identificadores
 * internos (`espinola.db`, el nombre del paquete, el identifier del
 * bundle) quedaron con la grafía vieja a propósito: cambiarlos haría que
 * la app instalada busque otro archivo y otra carpeta de datos, y el
 * negocio vería su base vacía.
 */
function Marca() {
  return (
    <div className="px-2 pb-5 pt-1">
      <p className="text-lg font-extrabold italic leading-none tracking-tight text-marca-ink">
        ESPÍNDOLA
      </p>
      <p className="mt-0.5 text-[11px] font-semibold uppercase leading-none tracking-[0.18em] text-accent">
        Motorespuestos
      </p>
    </div>
  );
}

export function Sidebar() {
  return (
    <aside className="w-60 shrink-0 overflow-y-auto bg-marca px-3 py-4">
      <Marca />
      <nav className="flex flex-col gap-1">
        {NAV_SECTIONS.map((item) => (
          <div key={item.path}>
            <NavLink
              to={item.path}
              end={item.path === "/"}
              className={({ isActive }) => {
                // Una sección con hijos comparte la ruta con el primero de
                // ellos, así que se marcarían las dos a la vez. El bloque
                // amarillo queda para la pantalla concreta; la sección solo
                // se resalta en color.
                const tieneHijos = (item.children?.length ?? 0) > 1;
                const base = "block rounded-md px-3 py-2 text-sm font-medium transition-colors";
                if (!isActive) return `${base} text-marca-ink hover:bg-marca-2`;
                return tieneHijos
                  ? `${base} text-accent`
                  : `${base} bg-accent text-accent-ink`;
              }}
            >
              {item.label}
            </NavLink>
            {item.children && item.children.length > 1 && (
              <div className="ml-3 mt-1 flex flex-col gap-1 border-l border-marca-2 pl-3">
                {item.children.map((child) => (
                  <NavLink
                    key={child.path}
                    to={child.path}
                    className={({ isActive }) =>
                      `rounded-md px-3 py-1.5 text-sm transition-colors ${
                        isActive
                          ? "bg-accent text-accent-ink"
                          : "text-marca-ink-muted hover:bg-marca-2 hover:text-marca-ink"
                      }`
                    }
                  >
                    {child.label}
                  </NavLink>
                ))}
              </div>
            )}
          </div>
        ))}
      </nav>
    </aside>
  );
}
