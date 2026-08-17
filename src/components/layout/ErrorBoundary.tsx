import { Component, type ErrorInfo, type ReactNode } from "react";

type Props = { children: ReactNode };
type State = { error: Error | null };

/**
 * Red de contención para errores de render inesperados. El objetivo es que
 * la app nunca muestre una pantalla en blanco frente al cliente: siempre
 * hay algo legible en la pantalla, aunque sea este mensaje.
 */
export class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error("Error no controlado en la interfaz:", error, info.componentStack);
  }

  render() {
    if (this.state.error) {
      return (
        <div className="flex h-screen w-screen items-center justify-center bg-bg p-8">
          <div className="max-w-md rounded-lg border border-line bg-surface p-6 text-center">
            <p className="font-semibold text-ink">Ocurrió un error inesperado</p>
            <p className="mt-2 text-sm text-ink-muted">
              Cerrá y volvé a abrir la aplicación. Si el problema continúa, avisá para
              revisar el log.
            </p>
          </div>
        </div>
      );
    }
    return this.props.children;
  }
}
