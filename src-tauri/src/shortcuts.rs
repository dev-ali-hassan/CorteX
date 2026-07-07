use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

use crate::{commands, models::RewriteMode, state::AppState};

pub fn register(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    app.handle().plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |app, shortcut, event| {
                if event.state() != ShortcutState::Pressed {
                    return;
                }

                let settings = app
                    .state::<AppState>()
                    .db
                    .get_settings()
                    .unwrap_or_default();

                if shortcut_matches(shortcut, &settings.global_shortcut) {
                    let app = app.clone();
                    tauri::async_runtime::spawn(async move {
                        commands::open_popup_from_shortcut(app).await;
                    });
                } else if shortcut_matches(shortcut, &settings.grammar_shortcut) {
                    let app = app.clone();
                    tauri::async_runtime::spawn(async move {
                        commands::run_direct_rewrite_shortcut(app, RewriteMode::FixGrammar).await;
                    });
                } else if shortcut_matches(shortcut, &settings.professional_shortcut) {
                    let app = app.clone();
                    tauri::async_runtime::spawn(async move {
                        commands::run_direct_rewrite_shortcut(app, RewriteMode::Professional).await;
                    });
                } else if shortcut_matches(shortcut, &settings.friendly_shortcut) {
                    let app = app.clone();
                    tauri::async_runtime::spawn(async move {
                        commands::run_direct_rewrite_shortcut(app, RewriteMode::Friendly).await;
                    });
                } else if shortcut_matches(shortcut, &settings.shorter_shortcut) {
                    let app = app.clone();
                    tauri::async_runtime::spawn(async move {
                        commands::run_direct_rewrite_shortcut(app, RewriteMode::Shorter).await;
                    });
                } else if shortcut_matches(shortcut, &settings.translate_shortcut) {
                    let app = app.clone();
                    tauri::async_runtime::spawn(async move {
                        commands::run_direct_rewrite_shortcut(app, RewriteMode::Translate).await;
                    });
                } else if shortcut_matches(shortcut, &settings.summarize_shortcut) {
                    let app = app.clone();
                    tauri::async_runtime::spawn(async move {
                        commands::run_direct_rewrite_shortcut(app, RewriteMode::Summarize).await;
                    });
                } else if shortcut_matches(shortcut, &settings.confident_shortcut) {
                    let app = app.clone();
                    tauri::async_runtime::spawn(async move {
                        commands::run_direct_rewrite_shortcut(app, RewriteMode::Confident).await;
                    });
                } else if shortcut_matches(shortcut, &settings.simplify_shortcut) {
                    let app = app.clone();
                    tauri::async_runtime::spawn(async move {
                        commands::run_direct_rewrite_shortcut(app, RewriteMode::Simplify).await;
                    });
                }
            })
            .build(),
    )?;

    sync_registered_shortcuts(app.handle())
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error))?;
    Ok(())
}

pub fn sync_registered_shortcuts(app: &AppHandle) -> Result<(), String> {
    let settings = app
        .state::<AppState>()
        .db
        .get_settings()
        .unwrap_or_default();
    let manager = app.global_shortcut();
    manager.unregister_all().map_err(|error| error.to_string())?;

    let mut shortcuts = Vec::new();
    for value in [
        settings.global_shortcut,
        settings.grammar_shortcut,
        settings.professional_shortcut,
        settings.friendly_shortcut,
        settings.shorter_shortcut,
        settings.translate_shortcut,
        settings.summarize_shortcut,
        settings.confident_shortcut,
        settings.simplify_shortcut,
    ] {
        if let Some(shortcut) = parse_shortcut(&value) {
            if !shortcuts.contains(&shortcut) {
                shortcuts.push(shortcut);
            }
        }
    }

    for shortcut in shortcuts {
        manager
            .register(shortcut)
            .map_err(|error| error.to_string())?;
    }

    Ok(())
}

fn shortcut_matches(shortcut: &Shortcut, configured: &str) -> bool {
    parse_shortcut(configured)
        .map(|value| &value == shortcut)
        .unwrap_or(false)
}

fn parse_shortcut(value: &str) -> Option<Shortcut> {
    let mut modifiers = Modifiers::empty();
    let mut code = None;

    for part in value.split('+').map(|part| part.trim().to_lowercase()) {
        match part.as_str() {
            "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
            "alt" => modifiers |= Modifiers::ALT,
            "shift" => modifiers |= Modifiers::SHIFT,
            "win" | "meta" | "super" => modifiers |= Modifiers::SUPER,
            key => code = parse_key_code(key),
        }
    }

    code.map(|key| {
        let modifier_set = (!modifiers.is_empty()).then_some(modifiers);
        Shortcut::new(modifier_set, key)
    })
}

fn parse_key_code(key: &str) -> Option<Code> {
    match key {
        "0" => Some(Code::Digit0),
        "1" => Some(Code::Digit1),
        "2" => Some(Code::Digit2),
        "3" => Some(Code::Digit3),
        "4" => Some(Code::Digit4),
        "5" => Some(Code::Digit5),
        "6" => Some(Code::Digit6),
        "7" => Some(Code::Digit7),
        "8" => Some(Code::Digit8),
        "9" => Some(Code::Digit9),
        "a" => Some(Code::KeyA),
        "b" => Some(Code::KeyB),
        "c" => Some(Code::KeyC),
        "d" => Some(Code::KeyD),
        "e" => Some(Code::KeyE),
        "f" => Some(Code::KeyF),
        "g" => Some(Code::KeyG),
        "h" => Some(Code::KeyH),
        "i" => Some(Code::KeyI),
        "j" => Some(Code::KeyJ),
        "k" => Some(Code::KeyK),
        "l" => Some(Code::KeyL),
        "m" => Some(Code::KeyM),
        "n" => Some(Code::KeyN),
        "o" => Some(Code::KeyO),
        "p" => Some(Code::KeyP),
        "q" => Some(Code::KeyQ),
        "r" => Some(Code::KeyR),
        "s" => Some(Code::KeyS),
        "t" => Some(Code::KeyT),
        "u" => Some(Code::KeyU),
        "v" => Some(Code::KeyV),
        "w" => Some(Code::KeyW),
        "x" => Some(Code::KeyX),
        "y" => Some(Code::KeyY),
        "z" => Some(Code::KeyZ),
        "space" => Some(Code::Space),
        "enter" => Some(Code::Enter),
        "escape" | "esc" => Some(Code::Escape),
        "tab" => Some(Code::Tab),
        "backspace" => Some(Code::Backspace),
        "delete" => Some(Code::Delete),
        "up" => Some(Code::ArrowUp),
        "down" => Some(Code::ArrowDown),
        "left" => Some(Code::ArrowLeft),
        "right" => Some(Code::ArrowRight),
        "f1" => Some(Code::F1),
        "f2" => Some(Code::F2),
        "f3" => Some(Code::F3),
        "f4" => Some(Code::F4),
        "f5" => Some(Code::F5),
        "f6" => Some(Code::F6),
        "f7" => Some(Code::F7),
        "f8" => Some(Code::F8),
        "f9" => Some(Code::F9),
        "f10" => Some(Code::F10),
        "f11" => Some(Code::F11),
        "f12" => Some(Code::F12),
        _ => None,
    }
}
