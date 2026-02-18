//! BullMQ TUI Desktop - Tauri Application
//!
//! This provides a native desktop application with direct Redis access
//! (no proxy needed) using Tauri's IPC commands.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

use commands::AppState;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::connect,
            commands::disconnect,
            commands::discover_queues,
            commands::get_queue_info,
            commands::get_queue_counts,
            commands::get_jobs,
            commands::get_job,
            commands::retry_job,
            commands::remove_job,
            commands::pause_queue,
            commands::resume_queue,
        ])
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
