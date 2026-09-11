import * as Dialog from "@radix-ui/react-dialog";

type Props = {
  abierto: boolean;
  titulo: string;
  /** Cada string es un párrafo. Conviene decir qué se va a modificar, no solo "¿estás seguro?". */
  mensaje: string | string[];
  textoConfirmar?: string;
  textoCancelar?: string;
  /** true para acciones destructivas (anular, restaurar): pinta el botón en rojo. */
  peligrosa?: boolean;
  confirmando?: boolean;
  onConfirmar: () => void;
  onCancelar: () => void;
};

/**
 * Reemplaza a `window.confirm`, que en Windows abre el cartel gris del
 * sistema: otra tipografía, otros botones y sin forma de explicar bien qué
 * está por pasar. Este vive dentro de la app y usa el mismo Dialog de
 * Radix que el resto de las pantallas.
 */
export function ConfirmDialog({
  abierto,
  titulo,
  mensaje,
  textoConfirmar = "Confirmar",
  textoCancelar = "Cancelar",
  peligrosa = false,
  confirmando = false,
  onConfirmar,
  onCancelar,
}: Props) {
  const parrafos = Array.isArray(mensaje) ? mensaje : [mensaje];

  return (
    <Dialog.Root open={abierto} onOpenChange={(open) => !open && onCancelar()}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/40" />
        <Dialog.Content className="fixed left-1/2 top-1/2 w-[90vw] max-w-md -translate-x-1/2 -translate-y-1/2 rounded-lg border border-line bg-surface p-6 shadow-lg">
          <Dialog.Title className="text-base font-semibold text-ink">{titulo}</Dialog.Title>
          <div className="mt-2 flex flex-col gap-2">
            {parrafos.map((parrafo, i) => (
              <Dialog.Description key={i} className="text-sm text-ink-muted">
                {parrafo}
              </Dialog.Description>
            ))}
          </div>

          <div className="mt-5 flex justify-end gap-2">
            <button
              type="button"
              onClick={onCancelar}
              className="rounded-md border border-line px-4 py-1.5 text-sm text-ink-muted hover:bg-surface-2"
            >
              {textoCancelar}
            </button>
            <button
              type="button"
              onClick={onConfirmar}
              disabled={confirmando}
              autoFocus
              className={`rounded-md px-4 py-1.5 text-sm font-medium text-white disabled:opacity-60 ${
                peligrosa ? "bg-danger hover:bg-danger/90" : "bg-accent hover:bg-accent/90"
              }`}
            >
              {confirmando ? "Un momento..." : textoConfirmar}
            </button>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
