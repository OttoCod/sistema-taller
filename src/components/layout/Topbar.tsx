export function Topbar() {
  return (
    <header className="flex h-14 shrink-0 items-center gap-4 border-b border-line bg-surface px-4">
      <input
        type="search"
        placeholder="Buscar producto por nombre, código o marca... (Fase 2)"
        disabled
        className="w-full max-w-xl rounded-md border border-line bg-surface-2 px-3 py-1.5 text-sm text-ink-muted placeholder:text-ink-muted disabled:cursor-not-allowed"
      />
    </header>
  );
}
