use std::sync::atomic::Ordering;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

use crate::{commands, state::AppState};

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Open CorteX", true, None::<&str>)?;
    let rewrite_clipboard = MenuItem::with_id(
        app,
        "rewrite_clipboard",
        "Rewrite Clipboard",
        true,
        None::<&str>,
    )?;
    let favorites = MenuItem::with_id(app, "favorites", "Favorites", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let pause_shortcut =
        MenuItem::with_id(app, "pause_shortcut", "Pause Shortcut", true, None::<&str>)?;
    let exit = MenuItem::with_id(app, "exit", "Exit", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &open,
            &rewrite_clipboard,
            &favorites,
            &settings,
            &pause_shortcut,
            &exit,
        ],
    )?;

    TrayIconBuilder::new()
        .tooltip("CorteX")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main(tray.app_handle());
            }
        })
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => show_main(app),
            "rewrite_clipboard" => {
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    commands::rewrite_clipboard_from_tray(app).await;
                });
            }
            "favorites" => {
                show_main(app);
                let _ = app.emit_to("main", "tray-navigate", "favorites");
            }
            "settings" => {
                show_main(app);
                let _ = app.emit_to("main", "tray-navigate", "settings");
            }
            "pause_shortcut" => {
                let state = app.state::<AppState>();
                let paused = !state.shortcuts_paused.load(Ordering::Relaxed);
                state.shortcuts_paused.store(paused, Ordering::Relaxed);
            }
            "exit" => app.exit(0),
            _ => {}
        })
        .build(app)?;

    Ok(())
}

fn show_main(app: &AppHandle) {
    let _ = crate::desktop::show_main_window(app);
    let _ = app.emit_to("main", "tray-navigate", "rewrite");
}
