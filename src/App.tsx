import { HashRouter, Routes, Route } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { AppShell } from "./components/layout/AppShell";
import { ErrorBoundary } from "./components/layout/ErrorBoundary";
import { InicioPage } from "./modules/inicio/InicioPage";
import { CatalogoPage } from "./modules/productos/CatalogoPage";
import { PlaceholderPage } from "./modules/placeholder/PlaceholderPage";
import { NAV_SECTIONS, flattenNav } from "./lib/nav";

const queryClient = new QueryClient();

// Fase en la que cada ruta obtiene su implementación real (plan de fases,
// sección 35 del documento de arquitectura). Se usa solo para el texto del
// placeholder; no afecta el ruteo.
const FASE_POR_RUTA: Record<string, string> = {
  "/ventas/nueva": "Fase 5 — Ventas",
  "/productos/stock": "Fase 4 — Stock",
  "/productos/reposicion": "Fase 4 — Stock",
  "/compras/nueva": "Fase 7 — Compras y recepción",
  "/compras/historial": "Fase 7 — Compras y recepción",
  "/proveedores": "Fase 8 — Proveedores",
  "/clientes": "Fase 6 — Clientes y cuentas corrientes",
  "/clientes/cuentas-pendientes": "Fase 6 — Clientes y cuentas corrientes",
  "/caja": "Fase 9 — Caja",
  "/ventas/historial": "Fase 5 — Ventas",
  "/configuracion": "Fase 1+ — se completa a medida que cada módulo lo necesita",
};

const RUTAS_IMPLEMENTADAS = new Set(["/", "/productos/catalogo"]);
const placeholderRoutes = flattenNav(NAV_SECTIONS).filter(
  (item) => !RUTAS_IMPLEMENTADAS.has(item.path),
);

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <ErrorBoundary>
        <HashRouter>
          <Routes>
            <Route element={<AppShell />}>
              <Route path="/" element={<InicioPage />} />
              <Route path="/productos/catalogo" element={<CatalogoPage />} />
              {placeholderRoutes.map((item) => (
                <Route
                  key={item.path}
                  path={item.path}
                  element={
                    <PlaceholderPage
                      title={item.label}
                      fase={FASE_POR_RUTA[item.path] ?? "próxima fase"}
                    />
                  }
                />
              ))}
            </Route>
          </Routes>
        </HashRouter>
      </ErrorBoundary>
    </QueryClientProvider>
  );
}

export default App;
