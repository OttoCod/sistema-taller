mod commands;
mod db;
mod error;
mod logging;
mod services;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("no se pudo resolver el directorio de datos de la app");

            logging::init(&app_data_dir);

            let db_path = db::db_path(&app_data_dir);
            let pool = tauri::async_runtime::block_on(db::init_pool(&app_data_dir))
                .expect("no se pudo inicializar la base de datos");

            app.manage(db::AppState {
                pool,
                db_path: db_path.display().to_string(),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::system::system_health_check
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
