use std::sync::atomic::Ordering;

use tauri::{AppHandle, Manager, State};

use crate::{
    desktop,
    models::{AppSettings, PopupPayload, ProviderId, ProviderSettings, RewriteMode, RewriteRequest, RewriteResponse},
    providers, shortcuts,
    state::AppState,
    text,
};

#[tauri::command]
pub async fn rewrite_text(
    state: State<'_, AppState>,
    request: RewriteRequest,
) -> Result<RewriteResponse, String> {
    let response = rewrite_inner(state.inner(), request).await?;
    apply_auto_copy(state.inner(), &response)?;
    Ok(response)
}

#[tauri::command]
pub fn copy_text(text: String) -> Result<(), String> {
    desktop::write_clipboard_text(&text)
}

#[tauri::command]
pub fn replace_selected_text(text: String) -> Result<(), String> {
    desktop::replace_selected_text(&text, None)
}

#[tauri::command]
pub fn capture_selected_text() -> Result<String, String> {
    desktop::capture_selected_text().map(|capture| capture.text)
}

#[tauri::command]
pub async fn show_popup(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let capture = desktop::capture_selected_text()?;
    let input = capture.text.trim().to_string();
    let payload = if input.is_empty() {
        empty_popup_payload()
    } else {
        let response = rewrite_inner(
            state.inner(),
            RewriteRequest {
                input,
                mode: RewriteMode::FixGrammar,
                target_language: None,
                custom_prompt: None,
            },
        )
        .await?;
        PopupPayload::from((response, "manual"))
    };

    store_popup_payload(state.inner(), &payload)?;
    desktop::show_popup_window(&app, &payload)
}

#[tauri::command]
pub fn get_popup_payload(state: State<'_, AppState>) -> Result<Option<PopupPayload>, String> {
    state
        .last_popup
        .lock()
        .map_err(|_| "popup state lock poisoned".to_string())
        .map(|payload| payload.clone())
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    state.db.get_settings()
}

#[tauri::command]
pub fn save_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<AppSettings, String> {
    desktop::sync_launch_at_startup(settings.launch_at_startup)?;
    let saved = state.db.save_settings(&settings)?;
    shortcuts::sync_registered_shortcuts(&app)?;
    Ok(saved)
}

#[tauri::command]
pub async fn test_provider_connection(
    state: State<'_, AppState>,
    settings: ProviderSettings,
) -> Result<String, String> {
    providers::test_connection(&state.client, &settings).await
}

#[tauri::command]
pub fn set_shortcuts_paused(state: State<'_, AppState>, paused: bool) -> Result<bool, String> {
    state.shortcuts_paused.store(paused, Ordering::Relaxed);
    Ok(paused)
}

#[tauri::command]
pub fn get_shortcuts_paused(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(state.shortcuts_paused.load(Ordering::Relaxed))
}

#[tauri::command]
pub fn hide_main_window(app: AppHandle) -> Result<(), String> {
    desktop::hide_main_window(&app)
}

#[tauri::command]
pub fn hide_popup_window(app: AppHandle) -> Result<(), String> {
    desktop::hide_popup_window(&app)
}

#[tauri::command]
pub fn show_main_window(app: AppHandle) -> Result<(), String> {
    desktop::show_main_window(&app)
}

pub async fn open_popup_from_shortcut(app: AppHandle) {
    let state = app.state::<AppState>();
    if state.shortcuts_paused.load(Ordering::Relaxed) {
        return;
    }

    let capture = match desktop::capture_selected_text() {
        Ok(value) => value,
        Err(_) => {
            let payload = empty_popup_payload();
            let _ = store_popup_payload(state.inner(), &payload);
            let _ = desktop::show_popup_window(&app, &payload);
            return;
        }
    };

    let input = capture.text.trim().to_string();
    let payload = if input.is_empty() {
        empty_popup_payload()
    } else {
        match rewrite_inner(
            state.inner(),
            RewriteRequest {
                input,
                mode: RewriteMode::FixGrammar,
                target_language: None,
                custom_prompt: None,
            },
        )
        .await
        {
            Ok(response) => PopupPayload::from((response, "shortcut")),
            Err(error) => error_popup_payload(error),
        }
    };

    let _ = store_popup_payload(state.inner(), &payload);
    let _ = desktop::show_popup_window(&app, &payload);
}

pub async fn run_direct_rewrite_shortcut(app: AppHandle, mode: RewriteMode) {
    let state = app.state::<AppState>();
    if state.shortcuts_paused.load(Ordering::Relaxed) {
        return;
    }

    let capture = match desktop::capture_selected_text() {
        Ok(value) => value,
        Err(_) => return,
    };
    let input = capture.text.trim().to_string();
    if input.is_empty() {
        return;
    }

    let response = match rewrite_inner(
        state.inner(),
        RewriteRequest {
            input,
            mode,
            target_language: None,
            custom_prompt: None,
        },
    )
    .await
    {
        Ok(value) => value,
        Err(_) => return,
    };

    let settings = state.db.get_settings().unwrap_or_default();
    if settings.auto_replace {
        let _ = desktop::replace_selected_text(&response.output, capture.previous_clipboard);
        if settings.auto_copy {
            let _ = desktop::write_clipboard_text(&response.output);
        }
    } else if settings.auto_copy {
        let _ = desktop::write_clipboard_text(&response.output);
    }

    let payload = PopupPayload::from((response, "shortcut"));
    let _ = store_popup_payload(state.inner(), &payload);
    if !settings.auto_replace && !settings.auto_copy {
        let _ = desktop::show_popup_window(&app, &payload);
    }
}

pub async fn rewrite_clipboard_from_tray(app: AppHandle) {
    let state = app.state::<AppState>();
    let input = match desktop::read_clipboard_text() {
        Ok(value) if !value.trim().is_empty() => value,
        _ => return,
    };

    let response = match rewrite_inner(
        state.inner(),
        RewriteRequest {
            input,
            mode: RewriteMode::FixGrammar,
            target_language: None,
            custom_prompt: None,
        },
    )
    .await
    {
        Ok(value) => value,
        Err(_) => return,
    };

    let _ = desktop::write_clipboard_text(&response.output);
    let payload = PopupPayload::from((response, "tray"));
    let _ = store_popup_payload(state.inner(), &payload);
    let _ = desktop::show_popup_window(&app, &payload);
}

pub async fn rewrite_inner(
    state: &AppState,
    request: RewriteRequest,
) -> Result<RewriteResponse, String> {
    let settings = state.db.get_settings().unwrap_or_default();
    let clean_input = request.input.trim().to_string();
    if clean_input.is_empty() {
        return Ok(RewriteResponse {
            input: String::new(),
            output: String::new(),
            mode: request.mode,
            provider: ProviderId::Offline,
            used_offline_fallback: true,
            character_count: 0,
        });
    }

    let mut provider_settings = settings.provider.clone();
    provider_settings.custom_prompt = settings.custom_prompt.clone();
    let requires_provider = request
        .custom_prompt
        .as_deref()
        .map(str::trim)
        .is_some_and(|instruction| !instruction.is_empty());
    let provider_result =
        providers::rewrite_with_provider(&state.client, &provider_settings, &request).await;
    let (output, provider, used_offline_fallback) = match provider_result {
        Ok(Some(output)) => (output, settings.provider.provider.clone(), false),
        Ok(None) if requires_provider => {
            return Err("Connect an AI provider to use Custom Prompt.".to_string())
        }
        Ok(None) => (
            text::rewrite_offline(
                &clean_input,
                &request.mode,
                request
                    .target_language
                    .as_deref()
                    .or(Some(settings.default_language.as_str())),
            ),
            ProviderId::Offline,
            true,
        ),
        Err(error) if requires_provider => return Err(error),
        Err(_) if settings.provider.use_offline_fallback => (
            text::rewrite_offline(
                &clean_input,
                &request.mode,
                request
                    .target_language
                    .as_deref()
                    .or(Some(settings.default_language.as_str())),
            ),
            ProviderId::Offline,
            true,
        ),
        Err(error) => return Err(error),
    };

    let response = RewriteResponse {
        input: clean_input,
        character_count: output.chars().count(),
        output,
        mode: request.mode,
        provider,
        used_offline_fallback,
    };

    Ok(response)
}

fn apply_auto_copy(state: &AppState, response: &RewriteResponse) -> Result<(), String> {
    if state.db.get_settings().unwrap_or_default().auto_copy {
        desktop::write_clipboard_text(&response.output)?;
    }
    Ok(())
}

fn store_popup_payload(state: &AppState, payload: &PopupPayload) -> Result<(), String> {
    let mut last_popup = state
        .last_popup
        .lock()
        .map_err(|_| "popup state lock poisoned".to_string())?;
    *last_popup = Some(payload.clone());
    Ok(())
}

fn empty_popup_payload() -> PopupPayload {
    let output =
        "Select text in any app, then press Ctrl + Alt + Z to rewrite it here.".to_string();
    let character_count = output.chars().count();
    PopupPayload {
        input: String::new(),
        output,
        mode: RewriteMode::FixGrammar,
        provider: ProviderId::Offline,
        used_offline_fallback: true,
        character_count,
        source: "shortcut".to_string(),
    }
}

fn error_popup_payload(error: String) -> PopupPayload {
    PopupPayload {
        input: String::new(),
        character_count: error.chars().count(),
        output: error,
        mode: RewriteMode::FixGrammar,
        provider: ProviderId::Offline,
        used_offline_fallback: true,
        source: "shortcut".to_string(),
    }
}
