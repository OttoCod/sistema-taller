use std::path::Path;

use tracing_subscriber::EnvFilter;

/// Logging a archivo, no a consola: la app se usa desde un ejecutable
/// instalado, no desde una terminal, así que la consola no sirve para
/// diagnosticar nada en el local. Un archivo por día dentro del directorio
/// de datos de la app permite revisar qué pasó después del hecho.
pub fn init(app_data_dir: &Path) {
    let log_dir = app_data_dir.join("logs");
    if let Err(err) = std::fs::create_dir_all(&log_dir) {
        eprintln!("no se pudo crear el directorio de logs: {err}");
        return;
    }

    let file_appender = tracing_appender::rolling::daily(&log_dir, "espinola.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    // El guard vive el resto del proceso: se filtra a propósito en vez de
    // pasarlo por todas las capas hasta el cierre de la app.
    Box::leak(Box::new(guard));

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(non_blocking)
        .with_ansi(false)
        .init();

    tracing::info!(dir = %log_dir.display(), "logging inicializado");
}
