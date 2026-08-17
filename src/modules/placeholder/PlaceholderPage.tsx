type Props = {
  title: string;
  fase: string;
};

/** Marcador para secciones de navegación cuyo módulo funcional todavía no se implementó. */
export function PlaceholderPage({ title, fase }: Props) {
  return (
    <div className="flex h-full flex-col items-start justify-center gap-2 text-ink-muted">
      <h1 className="text-xl font-semibold text-ink">{title}</h1>
      <p className="font-mono text-sm">Módulo pendiente — {fase}</p>
    </div>
  );
}
