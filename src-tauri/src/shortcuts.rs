use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

use crate::{
    commands,
    models::{AppSettings, RewriteMode},
    state::AppState,
};

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

    repair_saved_shortcuts(app.handle())?;
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
    manager
        .unregister_all()
        .map_err(|error| error.to_string())?;

    let shortcuts = validated_shortcuts(&settings)?;

    for shortcut in shortcuts {
        manager
            .register(shortcut)
            .map_err(|error| error.to_string())?;
    }

    Ok(())
}

pub fn validate_shortcuts(settings: &AppSettings) -> Result<(), String> {
    validated_shortcuts(settings).map(|_| ())
}

fn validated_shortcuts(settings: &AppSettings) -> Result<Vec<Shortcut>, String> {
    let mut parsed: Vec<(Shortcut, &'static str)> = Vec::new();
    for (label, value) in shortcut_entries(settings) {
        if is_unsafe_windows_shortcut(value) {
            return Err(format!(
                "{label} cannot use {value}. Add Alt, Shift, or Win, or use Ctrl with a number or function key."
            ));
        }

        let shortcut = parse_shortcut(value)
            .ok_or_else(|| format!("{label} has an unsupported shortcut: {value}."))?;
        if let Some((_, existing_label)) = parsed.iter().find(|(item, _)| item == &shortcut) {
            return Err(format!(
                "{label} and {existing_label} cannot use the same shortcut ({value})."
            ));
        }
        parsed.push((shortcut, label));
    }
    Ok(parsed.into_iter().map(|(shortcut, _)| shortcut).collect())
}

fn shortcut_entries(settings: &AppSettings) -> [(&'static str, &str); 9] {
    [
        ("Floating Window", settings.global_shortcut.as_str()),
        ("Grammar", settings.grammar_shortcut.as_str()),
        ("Professional", settings.professional_shortcut.as_str()),
        ("Friendly", settings.friendly_shortcut.as_str()),
        ("Shorter", settings.shorter_shortcut.as_str()),
        ("Translate", settings.translate_shortcut.as_str()),
        ("Summarize", settings.summarize_shortcut.as_str()),
        ("Confident", settings.confident_shortcut.as_str()),
        ("Simplify", settings.simplify_shortcut.as_str()),
    ]
}

fn is_unsafe_windows_shortcut(value: &str) -> bool {
    let parts: Vec<String> = value
        .split('+')
        .map(|part| part.trim().to_ascii_lowercase())
        .filter(|part| !part.is_empty())
        .collect();
    let has_ctrl = parts.iter().any(|part| part == "ctrl" || part == "control");
    let has_alt = parts.iter().any(|part| part == "alt");
    let has_shift = parts.iter().any(|part| part == "shift");
    let has_super = parts
        .iter()
        .any(|part| part == "win" || part == "meta" || part == "super");
    let key = parts.iter().find(|part| {
        !matches!(
            part.as_str(),
            "ctrl" | "control" | "alt" | "shift" | "win" | "meta" | "super"
        )
    });

    if !has_ctrl && !has_alt && !has_shift && !has_super {
        return true;
    }

    has_ctrl
        && !has_alt
        && !has_shift
        && !has_super
        && key.is_some_and(|key| key.len() == 1 && key.as_bytes()[0].is_ascii_alphabetic())
}

fn repair_saved_shortcuts(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let state = app.state::<AppState>();
    let mut settings = state.db.get_settings()?;
    if validate_shortcuts(&settings).is_ok() {
        return Ok(());
    }

    let defaults = AppSettings::default();
    settings.global_shortcut = defaults.global_shortcut;
    settings.grammar_shortcut = defaults.grammar_shortcut;
    settings.professional_shortcut = defaults.professional_shortcut;
    settings.friendly_shortcut = defaults.friendly_shortcut;
    settings.shorter_shortcut = defaults.shorter_shortcut;
    settings.translate_shortcut = defaults.translate_shortcut;
    settings.summarize_shortcut = defaults.summarize_shortcut;
    settings.confident_shortcut = defaults.confident_shortcut;
    settings.simplify_shortcut = defaults.simplify_shortcut;
    state.db.save_settings(&settings)?;
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

#[cfg(test)]
mod tests {
    use super::{is_unsafe_windows_shortcut, validate_shortcuts};
    use crate::models::AppSettings;

    #[test]
    fn default_shortcuts_are_valid_and_unique() {
        assert!(validate_shortcuts(&AppSettings::default()).is_ok());
    }

    #[test]
    fn plain_control_letter_shortcuts_are_rejected() {
        assert!(is_unsafe_windows_shortcut("Ctrl + C"));
        assert!(is_unsafe_windows_shortcut("Ctrl + V"));
        assert!(!is_unsafe_windows_shortcut("Ctrl + Shift + C"));
        assert!(!is_unsafe_windows_shortcut("Ctrl + 1"));
    }

    #[test]
    fn duplicate_shortcuts_are_rejected() {
        let mut settings = AppSettings::default();
        settings.professional_shortcut = settings.grammar_shortcut.clone();
        let error = validate_shortcuts(&settings).unwrap_err();
        assert!(error.contains("same shortcut"));
    }
}
