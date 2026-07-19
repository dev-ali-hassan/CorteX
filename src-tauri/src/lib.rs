mod commands;
mod db;
mod desktop;
mod models;
mod providers;
mod shortcuts;
mod state;
mod text;
mod tray;

use tauri::{Manager, RunEvent};

pub fn run() {
    let show_on_ready = !std::env::args().any(|argument| argument == "--background");
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            if !args.iter().any(|argument| argument == "--background") {
                // Return from the IPC callback before touching the window. This lets
                // the second process exit immediately and avoids cross-process hangs.
                let app_handle = app.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(40));
                    let restore_handle = app_handle.clone();
                    let _ = app_handle.run_on_main_thread(move || {
                        let _ = desktop::show_main_window(&restore_handle);
                    });
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
            // A shortcut may temporarily be owned by another application. CorteX
            // must still open normally even when one global shortcut cannot register.
            if let Err(error) = shortcuts::register(app) {
                eprintln!("CorteX shortcut registration failed: {error}");
            }
            let _ = desktop::sync_launch_at_startup(launch_at_startup);
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
        .build(tauri::generate_context!())
        .expect("error while building CorteX");

    app.run(move |app, event| {
        if show_on_ready && matches!(event, RunEvent::Ready) {
            let _ = desktop::show_main_window(app);
        }
    });
}
