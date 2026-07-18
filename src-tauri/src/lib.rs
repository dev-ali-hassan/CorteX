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
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            if !args.iter().any(|argument| argument == "--background") {
                let app_handle = app.clone();
                let _ = app.run_on_main_thread(move || {
                    let _ = desktop::show_main_window(&app_handle);
                });
            }
        }))
        .setup(|app| {
            let state = state::AppState::new(app.handle())?;
            let launch_at_startup = state
                .db
                .get_settings()
                .map(|settings| settings.launch_at_startup)
                .unwrap_or(false);
            app.manage(state);
            let _ = desktop::prepare_popup_window(app.handle());
            tray::create(app.handle())?;
            shortcuts::register(app)?;
            let _ = desktop::sync_launch_at_startup(launch_at_startup);
            if !std::env::args().any(|argument| argument == "--background") {
                let _ = desktop::show_main_window(app.handle());
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    let minimize_to_tray = window
                        .app_handle()
                        .state::<state::AppState>()
                        .db
                        .get_settings()
                        .map(|settings| settings.minimize_to_tray)
                        .unwrap_or(true);

                    if minimize_to_tray {
                        api.prevent_close();
                        let _ = window.unmaximize();
                        let _ = window.hide();
                    }
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
            commands::test_provider_connection,
            commands::open_provider_guide,
            commands::set_shortcuts_paused,
            commands::get_shortcuts_paused,
            commands::hide_main_window,
            commands::close_main_window,
            commands::hide_popup_window,
            commands::show_main_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running CorteX");
}
