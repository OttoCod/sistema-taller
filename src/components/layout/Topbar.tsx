import { useQuery } from "@tanstack/react-query";
import { obtenerConfiguracionNegocio } from "../../lib/api/configuracion";

function fechaDeHoy(): string {
  return new Date().toLocaleDateString("es-AR", {
    weekday: "long",
    day: "numeric",
    month: "long",
    year: "numeric",
  });
}

export function Topbar() {
  // Hasta la Fase 10 acá había un buscador global deshabilitado, de adorno.
  // Se sacó: lo que sirve arriba de todo es saber en qué negocio y en qué
  // día se está parado (la fecha manda en Caja y en los comprobantes).
  const { data: negocio } = useQuery({
    queryKey: ["configuracion", "negocio"],
    queryFn: obtenerConfiguracionNegocio,
  });

  return (
    <header className="flex h-14 shrink-0 items-center justify-between gap-4 border-b border-line bg-surface px-6">
      <p className="truncate text-sm font-medium text-ink">
        {negocio?.nombre || "Sin nombre configurado"}
      </p>
      {/* first-letter y no capitalize: este último pondría en mayúscula
          cada palabra ("Viernes, 11 De Septiembre De 2026"). */}
      <p className="shrink-0 text-sm text-ink-muted first-letter:uppercase">{fechaDeHoy()}</p>
    </header>
  );
}
