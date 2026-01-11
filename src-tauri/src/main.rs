// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod state;
mod websocket;

use state::AppState;
use tauri::Manager;
use tracing::{info, error};
use tracing_subscriber;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("hybrid_llm=debug,info")
        .init();

    info!("🚀 Starting Hybrid LLM Platform Tauri v2 app...");

    tauri::Builder::default()
        .setup(|app| {
            // Initialize app state
            let state = AppState::new();
            app.manage(state);

            // Start WebSocket server for real-time updates
            // Clone the handle for the async task
            let app_handle = app.handle().clone();
            tokio::spawn(async move {
                if let Err(e) = websocket::start_server(app_handle).await {
                    error!("WebSocket server error: {}", e);
                }
            });

            info!("✅ Tauri v2 app initialized successfully");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // System commands
            commands::get_system_state,
            commands::trigger_lockdown,
            commands::release_lockdown,

            // LLM commands
            commands::get_llms,
            commands::load_llm,
            commands::unload_llm,
            commands::send_message,

            // Document commands
            commands::upload_document,
            commands::get_documents,
            commands::delete_document,

            // Permission commands
            commands::get_permissions,
            commands::update_permissions,

            // Audit log commands
            commands::get_audit_log,

            // Sandbox commands
            commands::create_sandbox,
            commands::execute_in_sandbox,
            commands::get_sandbox_files,
            commands::approve_transfer,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
