pub mod commands;
pub mod state;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::connect_to_server,
            commands::disconnect,
            commands::health_check,
            commands::list_components,
            commands::get_component,
            commands::start_flash,
            commands::list_jobs,
            commands::get_job,
            commands::cancel_job,
            commands::read_dtcs,
            commands::clear_dtcs,
            commands::read_data,
            commands::write_data,
            commands::get_live_data,
            commands::get_logs,
            commands::list_plugins,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
