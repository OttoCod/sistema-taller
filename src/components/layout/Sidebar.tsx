import { NavLink } from "react-router-dom";
import { NAV_SECTIONS } from "../../lib/nav";

export function Sidebar() {
  return (
    <aside className="w-60 shrink-0 border-r border-line bg-surface px-3 py-4 overflow-y-auto">
      <div className="px-2 pb-4">
        <p className="font-mono text-[11px] uppercase tracking-wider text-ink-muted">
          Espínola
        </p>
        <p className="font-semibold text-ink">Motorepuestos</p>
      </div>
      <nav className="flex flex-col gap-1">
        {NAV_SECTIONS.map((item) => (
          <div key={item.path}>
            <NavLink
              to={item.path}
              end={item.path === "/"}
              className={({ isActive }) =>
                `block rounded-md px-3 py-2 text-sm font-medium transition-colors ${
                  isActive
                    ? "bg-accent text-accent-ink"
                    : "text-ink hover:bg-surface-2"
                }`
              }
            >
              {item.label}
            </NavLink>
            {item.children && item.children.length > 1 && (
              <div className="ml-3 mt-1 flex flex-col gap-1 border-l border-line pl-3">
                {item.children.map((child) => (
                  <NavLink
                    key={child.path}
                    to={child.path}
                    className={({ isActive }) =>
                      `rounded-md px-3 py-1.5 text-sm transition-colors ${
                        isActive
                          ? "bg-accent text-accent-ink"
                          : "text-ink-muted hover:bg-surface-2 hover:text-ink"
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
