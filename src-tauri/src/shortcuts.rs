use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

use crate::{commands, models::RewriteMode};

pub fn register(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let popup_shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyZ);
    let grammar_shortcut = Shortcut::new(Some(Modifiers::CONTROL), Code::Digit1);
    let professional_shortcut = Shortcut::new(Some(Modifiers::CONTROL), Code::Digit2);

    let popup_for_handler = popup_shortcut.clone();
    let grammar_for_handler = grammar_shortcut.clone();
    let professional_for_handler = professional_shortcut.clone();

    app.handle().plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |app, shortcut, event| {
                if event.state() != ShortcutState::Pressed {
                    return;
                }

                if shortcut == &popup_for_handler {
                    let app = app.clone();
                    tauri::async_runtime::spawn(async move {
                        commands::open_popup_from_shortcut(app).await;
                    });
                } else if shortcut == &grammar_for_handler {
                    let app = app.clone();
                    tauri::async_runtime::spawn(async move {
                        commands::run_direct_rewrite_shortcut(app, RewriteMode::FixGrammar).await;
                    });
                } else if shortcut == &professional_for_handler {
                    let app = app.clone();
                    tauri::async_runtime::spawn(async move {
                        commands::run_direct_rewrite_shortcut(app, RewriteMode::Professional).await;
                    });
                }
            })
            .build(),
    )?;

    app.global_shortcut().register(popup_shortcut)?;
    app.global_shortcut().register(grammar_shortcut)?;
    app.global_shortcut().register(professional_shortcut)?;
    Ok(())
}
