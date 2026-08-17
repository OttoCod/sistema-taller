use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

/// Error único de toda la aplicación. Todo comando de Tauri devuelve
/// `AppResult<T>`, así el frontend siempre recibe el mismo shape
/// `{ kind, message }` (ver src/lib/api/client.ts) sin importar qué
/// falló internamente.
// Validation/NotFound/Conflict se usan a partir de la Fase 2, cuando
// aparecen los primeros servicios de negocio; hasta entonces el compilador
// los marcaría como código muerto.
#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Error de base de datos: {0}")]
    Database(#[from] sqlx::Error),

    #[error("{0}")]
    Validation(String),

    #[error("No se encontró: {0}")]
    NotFound(String),

    #[error("{0}")]
    Conflict(String),

    #[error("Error de E/S: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Unexpected(String),
}

impl AppError {
    fn kind(&self) -> &'static str {
        match self {
            AppError::Database(_) => "database",
            AppError::Validation(_) => "validation",
            AppError::NotFound(_) => "not_found",
            AppError::Conflict(_) => "conflict",
            AppError::Io(_) => "io",
            AppError::Unexpected(_) => "unexpected",
        }
    }

    /// Errores de negocio (validación, no encontrado, conflicto) son
    /// esperables y ya tienen un mensaje pensado para el usuario. Los
    /// demás son fallas del sistema: se registran en el log para poder
    /// diagnosticarlas sin depender de herramientas de desarrollador.
    fn is_business_error(&self) -> bool {
        matches!(
            self,
            AppError::Validation(_) | AppError::NotFound(_) | AppError::Conflict(_)
        )
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if !self.is_business_error() {
            tracing::error!(error = %self, "comando falló");
        }
        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("kind", self.kind())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}

pub type AppResult<T> = Result<T, AppError>;
