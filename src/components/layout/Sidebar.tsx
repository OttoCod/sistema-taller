import { useState } from "react";
import { NavLink } from "react-router-dom";
import { NAV_SECTIONS } from "../../lib/nav";
import { Icono } from "../ui/Icono";

/**
 * Muestra el logo del negocio si el archivo `public/logo.png` existe. Si
 * no está, cae en una versión tipográfica que imita el logo (ESPÍNDOLA en
 * blanco sobre negro, MOTORESPUESTOS en amarillo debajo), así la pantalla
 * nunca queda con un hueco ni con el ícono de imagen rota.
 *
 * Es a propósito que dependa de que el archivo exista y no de un `import`:
 * un import de un archivo ausente rompe la compilación, y la idea es que
 * el negocio pueda poner o cambiar su logo sin tocar código ni necesitar
 * a un programador.
 *
 * Ojo con el nombre: es Espíndola (con d) Motorespuestos (con s), tal
 * como está en el logo y en el Instagram del local. Los identificadores
 * internos (`espinola.db`, el nombre del paquete, el identifier del
 * bundle) quedaron con la grafía vieja a propósito: cambiarlos haría que
 * la app instalada busque otro archivo y otra carpeta de datos, y el
 * negocio vería su base vacía.
 */
function Marca() {
  // Si existe public/logo.png, se muestra ese archivo; si no, queda la
  // versión tipográfica. Así el negocio puede poner su logo real sin que
  // haya que tocar una línea de código: alcanza con dejar el archivo ahí.
  const [logo, setLogo] = useState<"probando" | "si" | "no">("probando");

  return (
    <div className="px-2 pb-5 pt-1">
      <img
        src="/logo.png"
        alt="Espíndola Motorespuestos"
        hidden={logo !== "si"}
        onLoad={() => setLogo("si")}
        onError={() => setLogo("no")}
        className="max-h-24 w-full object-contain object-left"
      />
      {logo !== "si" && (
        <>
          <p className="text-lg font-extrabold italic leading-none tracking-tight text-marca-ink">
            ESPÍNDOLA
          </p>
          <p className="mt-0.5 text-[11px] font-semibold uppercase leading-none tracking-[0.18em] text-accent">
            Motorespuestos
          </p>
        </>
      )}
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
                const base =
                  "flex items-center gap-2.5 rounded-md px-3 py-2 text-sm font-medium transition-colors";
                if (!isActive) return `${base} text-marca-ink hover:bg-marca-2`;
                return tieneHijos
                  ? `${base} text-accent`
                  : `${base} bg-accent text-accent-ink`;
              }}
            >
              {item.icono && <Icono nombre={item.icono} className="h-[18px] w-[18px] shrink-0" />}
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
