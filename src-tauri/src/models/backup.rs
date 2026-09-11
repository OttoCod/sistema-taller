use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Backup {
    pub id: i64,
    pub archivo_path: String,
    pub fecha: String,
    pub tamano_bytes: i64,
    pub tipo: String,
    pub version_esquema: i64,
    /// Se calcula al listar: un backup registrado cuyo archivo ya no está
    /// (lo borraron a mano, se desconectó el pendrive) no se puede
    /// restaurar, y conviene que se vea antes de intentarlo.
    #[sqlx(default)]
    pub archivo_existe: bool,
}

/// Lo que se guarda en el `.json` que acompaña a cada copia. Sirve para
/// saber qué es ese archivo aunque se lo encuentre suelto, años después,
/// sin la app al lado.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifiestoBackup {
    pub archivo: String,
    pub fecha: String,
    pub tipo: String,
    pub version_esquema: i64,
    pub version_app: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ConfiguracionBackups {
    /// Carpeta donde se guardan las copias. Vacío = la carpeta por
    /// defecto dentro de los datos de la app.
    pub carpeta: String,
    /// Cuántas copias se conservan antes de empezar a rotar las viejas.
    pub retencion: i64,
    /// Carpeta que se usa efectivamente (la elegida, o la por defecto ya
    /// resuelta). Solo lectura, para mostrarla en pantalla.
    pub carpeta_efectiva: String,
}

/// Resultado de preparar una restauración: todavía no se tocó la base en
/// uso -- eso pasa recién al reiniciar la app.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestauracionPreparada {
    pub backup_restaurado: String,
    pub copia_de_seguridad: String,
}
