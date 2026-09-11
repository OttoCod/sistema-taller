mod commands;
mod db;
mod error;
mod logging;
mod models;
mod services;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("no se pudo resolver el directorio de datos de la app");

            logging::init(&app_data_dir);

            let db_path = db::db_path(&app_data_dir);
            let pool = tauri::async_runtime::block_on(db::init_pool(&app_data_dir))
                .expect("no se pudo inicializar la base de datos");

            // Fase 12: si el último backup ya tiene más de 24 h (o nunca
            // hubo uno), se saca uno solo, en segundo plano, para no
            // demorar el arranque. Si falla, queda en el log y la app
            // abre igual: un backup que no salió no es motivo para dejar
            // al negocio sin sistema.
            let pool_backup = pool.clone();
            let dir_backup = app_data_dir.clone();
            tauri::async_runtime::spawn(async move {
                match services::backups::necesita_automatico(&pool_backup).await {
                    Ok(true) => {
                        if let Err(err) =
                            services::backups::crear(&pool_backup, &dir_backup, "automatico").await
                        {
                            tracing::error!(error = %err, "no se pudo generar el backup automático");
                        } else {
                            tracing::info!("backup automático generado");
                        }
                    }
                    Ok(false) => {}
                    Err(err) => {
                        tracing::error!(error = %err, "no se pudo evaluar si hacía falta un backup")
                    }
                }
            });

            app.manage(db::AppState {
                pool,
                db_path: db_path.display().to_string(),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::system::system_health_check,
            commands::marcas::marcas_listar,
            commands::marcas::marcas_crear,
            commands::categorias::categorias_listar,
            commands::categorias::categorias_crear,
            commands::productos::productos_listar,
            commands::productos::productos_obtener,
            commands::productos::productos_crear,
            commands::productos::productos_actualizar,
            commands::productos::productos_buscar,
            commands::stock::stock_listar,
            commands::stock::stock_listar_reposicion,
            commands::stock::stock_ajustar,
            commands::stock::stock_actualizar_minimo,
            commands::clientes::clientes_listar,
            commands::clientes::clientes_buscar,
            commands::clientes::clientes_obtener,
            commands::clientes::clientes_crear,
            commands::clientes::clientes_actualizar,
            commands::clientes::clientes_listar_cuentas_pendientes,
            commands::cuenta_corriente::cuenta_corriente_listar_movimientos,
            commands::cuenta_corriente::cuenta_corriente_registrar_pago,
            commands::cuenta_corriente::cuenta_corriente_ajustar,
            commands::metodos_pago::metodos_pago_listar,
            commands::ventas::ventas_listar,
            commands::ventas::ventas_obtener,
            commands::ventas::ventas_crear,
            commands::ventas::ventas_anular,
            commands::devoluciones::devoluciones_crear,
            commands::devoluciones::devoluciones_listar_por_venta,
            commands::proveedores::proveedores_listar,
            commands::proveedores::proveedores_buscar,
            commands::proveedores::proveedores_crear,
            commands::proveedores::proveedores_actualizar,
            commands::compras::compras_listar,
            commands::compras::compras_obtener,
            commands::compras::compras_crear,
            commands::producto_proveedores::producto_proveedores_listar,
            commands::producto_proveedores::producto_proveedores_agregar,
            commands::producto_proveedores::producto_proveedores_quitar,
            commands::importaciones::importaciones_procesar_archivo,
            commands::importaciones::importaciones_listar,
            commands::importaciones::importaciones_obtener,
            commands::importaciones::importaciones_resumen,
            commands::importaciones::importaciones_listar_filas,
            commands::importaciones::importaciones_buscar_confirmada_con_mismo_hash,
            commands::importaciones::importaciones_resolver_fila,
            commands::importaciones::importaciones_aplicar_pendientes,
            commands::importaciones::importaciones_descartar,
            commands::caja::caja_resumen,
            commands::configuracion::configuracion_obtener_negocio,
            commands::configuracion::configuracion_guardar_negocio,
            commands::comprobantes::comprobantes_obtener_o_crear,
            commands::comprobantes::comprobantes_listar_por_venta,
            commands::comprobantes::comprobantes_registrar_evento,
            commands::comprobantes::comprobantes_listar_eventos,
            commands::backups::backups_listar,
            commands::backups::backups_crear,
            commands::backups::backups_obtener_configuracion,
            commands::backups::backups_guardar_configuracion,
            commands::backups::backups_preparar_restauracion,
            commands::backups::app_reiniciar,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
