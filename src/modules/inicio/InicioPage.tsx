import { useQuery } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { obtenerResumenCaja } from "../../lib/api/caja";
import { listarCuentasPendientes } from "../../lib/api/clientes";
import { listarReposicion } from "../../lib/api/stock";
import { formatearCentavos } from "../../lib/money";

function fechaLocalHoy(): string {
  const ahora = new Date();
  return new Date(ahora.getTime() - ahora.getTimezoneOffset() * 60_000).toISOString().slice(0, 10);
}

type TarjetaProps = {
  a: string;
  titulo: string;
  valor: string;
  detalle: string;
  alerta?: boolean;
};

function Tarjeta({ a, titulo, valor, detalle, alerta = false }: TarjetaProps) {
  return (
    <Link
      to={a}
      className="flex flex-col gap-1 rounded-lg border border-line bg-surface p-5 transition-colors hover:border-accent"
    >
      <p className="text-sm text-ink-muted">{titulo}</p>
      <p className={`text-2xl font-semibold ${alerta ? "text-warn" : "text-ink"}`}>{valor}</p>
      <p className="text-sm text-ink-muted">{detalle}</p>
    </Link>
  );
}

export function InicioPage() {
  const hoy = fechaLocalHoy();

  const cajaQuery = useQuery({ queryKey: ["caja", hoy], queryFn: () => obtenerResumenCaja(hoy) });
  const pendientesQuery = useQuery({
    queryKey: ["clientes", "cuentasPendientes"],
    queryFn: listarCuentasPendientes,
  });
  const reposicionQuery = useQuery({ queryKey: ["stock", "reposicion"], queryFn: listarReposicion });

  const deuda = (pendientesQuery.data ?? []).reduce((s, c) => s + c.saldoCuentaCorriente, 0);
  const clientesConDeuda = (pendientesQuery.data ?? []).length;
  const aReponer = (reposicionQuery.data ?? []).length;

  return (
    <div className="flex flex-col gap-6">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h1 className="text-xl font-semibold text-ink">Inicio</h1>
          <p className="text-sm text-ink-muted">Cómo viene el día.</p>
        </div>
        <Link
          to="/ventas/nueva"
          className="rounded-md bg-accent px-4 py-2 text-sm font-medium text-accent-ink hover:bg-accent/90"
        >
          + Nueva venta
        </Link>
      </div>

      <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
        <Tarjeta
          a="/caja"
          titulo="Cobrado hoy"
          valor={cajaQuery.data ? formatearCentavos(cajaQuery.data.totalCobrado) : "—"}
          detalle={
            cajaQuery.data
              ? `${cajaQuery.data.cantidadVentas} venta(s)${
                  cajaQuery.data.totalFiado > 0
                    ? ` · ${formatearCentavos(cajaQuery.data.totalFiado)} fiado`
                    : ""
                }`
              : "Cargando..."
          }
        />
        <Tarjeta
          a="/clientes/cuentas-pendientes"
          titulo="Te deben"
          valor={pendientesQuery.data ? formatearCentavos(deuda) : "—"}
          detalle={
            pendientesQuery.data
              ? clientesConDeuda === 0
                ? "Nadie con saldo pendiente"
                : `${clientesConDeuda} cliente(s) con saldo`
              : "Cargando..."
          }
          alerta={clientesConDeuda > 0}
        />
        <Tarjeta
          a="/productos/reposicion"
          titulo="Para reponer"
          valor={reposicionQuery.data ? String(aReponer) : "—"}
          detalle={
            reposicionQuery.data
              ? aReponer === 0
                ? "Todo por encima del mínimo"
                : "Productos en o bajo el mínimo"
              : "Cargando..."
          }
          alerta={aReponer > 0}
        />
      </div>
    </div>
  );
}
