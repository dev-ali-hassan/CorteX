mod commands;
mod db;
mod desktop;
mod models;
mod providers;
mod shortcuts;
mod state;
mod text;
mod tray;

use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let state = state::AppState::new(app.handle())?;
            app.manage(state);
            tray::create(app.handle())?;
            shortcuts::register(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.unmaximize();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::rewrite_text,
            commands::copy_text,
            commands::replace_selected_text,
            commands::capture_selected_text,
            commands::show_popup,
            commands::get_popup_payload,
            commands::get_settings,
            commands::save_settings,
            commands::set_shortcuts_paused,
            commands::get_shortcuts_paused,
            commands::hide_main_window,
            commands::show_main_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running CorteX");
}
