import { HashRouter, Routes, Route } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { AppShell } from "./components/layout/AppShell";
import { ErrorBoundary } from "./components/layout/ErrorBoundary";
import { InicioPage } from "./modules/inicio/InicioPage";
import { CatalogoPage } from "./modules/productos/CatalogoPage";
import { StockPage } from "./modules/productos/StockPage";
import { ReposicionPage } from "./modules/productos/ReposicionPage";
import { ClientesPage } from "./modules/clientes/ClientesPage";
import { CuentasPendientesPage } from "./modules/clientes/CuentasPendientesPage";
import { NuevaVentaPage } from "./modules/ventas/NuevaVentaPage";
import { HistorialVentasPage } from "./modules/ventas/HistorialVentasPage";
import { NuevaCompraPage } from "./modules/compras/NuevaCompraPage";
import { HistorialComprasPage } from "./modules/compras/HistorialComprasPage";
import { PlaceholderPage } from "./modules/placeholder/PlaceholderPage";
import { NAV_SECTIONS, flattenNav } from "./lib/nav";

const queryClient = new QueryClient();

// Fase en la que cada ruta obtiene su implementación real (plan de fases,
// sección 35 del documento de arquitectura). Se usa solo para el texto del
// placeholder; no afecta el ruteo.
const FASE_POR_RUTA: Record<string, string> = {
  "/proveedores": "Fase 8 — Proveedores",
  "/caja": "Fase 9 — Caja",
  "/configuracion": "Fase 1+ — se completa a medida que cada módulo lo necesita",
};

const RUTAS_IMPLEMENTADAS = new Set([
  "/",
  "/productos/catalogo",
  "/productos/stock",
  "/productos/reposicion",
  "/clientes",
  "/clientes/cuentas-pendientes",
  "/ventas/nueva",
  "/ventas/historial",
  "/compras/nueva",
  "/compras/historial",
]);
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
              <Route path="/productos/stock" element={<StockPage />} />
              <Route path="/productos/reposicion" element={<ReposicionPage />} />
              <Route path="/clientes" element={<ClientesPage />} />
              <Route path="/clientes/cuentas-pendientes" element={<CuentasPendientesPage />} />
              <Route path="/ventas/nueva" element={<NuevaVentaPage />} />
              <Route path="/ventas/historial" element={<HistorialVentasPage />} />
              <Route path="/compras/nueva" element={<NuevaCompraPage />} />
              <Route path="/compras/historial" element={<HistorialComprasPage />} />
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
