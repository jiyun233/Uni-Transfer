// Prevents additional console window on Windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod ftp_server;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_all_ipv4,
            ftp_server::start_ftp,
            ftp_server::stop_ftp,
            ftp_server::get_ftp_status,
            ftp_server::drain_ftp_logs,
            ftp_server::get_active_sessions,
            ftp_server::get_transfer_history,
            ftp_server::clear_transfer_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
