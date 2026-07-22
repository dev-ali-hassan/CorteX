use std::{sync::atomic::Ordering, time::Instant};

use tauri::{AppHandle, Emitter, Manager, State};

use crate::{
    desktop,
    models::{
        AppSettings, PopupPayload, ProviderId, ProviderSettings, RewriteMode, RewriteRequest,
        RewriteResponse,
    },
    providers, shortcuts,
    state::AppState,
    text,
};

#[tauri::command]
pub async fn rewrite_text(
    state: State<'_, AppState>,
    request: RewriteRequest,
) -> Result<RewriteResponse, String> {
    rewrite_inner(state.inner(), request).await
}

#[tauri::command]
pub fn copy_text(text: String) -> Result<(), String> {
    desktop::write_clipboard_text(&text)
}

#[tauri::command]
pub fn replace_selected_text(state: State<'_, AppState>, text: String) -> Result<(), String> {
    let target = last_selection_window(state.inner())?;
    desktop::replace_selected_text(&text, None, target)
}

#[tauri::command]
pub fn capture_selected_text(state: State<'_, AppState>) -> Result<String, String> {
    let capture = desktop::capture_selected_text()?;
    store_selection_window(state.inner(), capture.source_window)?;
    Ok(capture.text)
}

#[tauri::command]
pub async fn show_popup(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let capture = desktop::capture_selected_text()?;
    store_selection_window(state.inner(), capture.source_window)?;
    let input = capture.text.trim().to_string();
    let payload = if input.is_empty() {
        empty_popup_payload()
    } else if is_non_prose_selection(&input) {
        invalid_selection_popup_payload()
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
    let mut settings = state.db.get_settings()?;
    let startup_enabled = desktop::launch_at_startup_enabled()?;
    if settings.launch_at_startup != startup_enabled {
        settings.launch_at_startup = startup_enabled;
        return state.db.save_settings(&settings);
    }
    Ok(settings)
}

#[tauri::command]
pub fn save_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    mut settings: AppSettings,
) -> Result<AppSettings, String> {
    settings.global_shortcut = shortcuts::FLOATING_WINDOW_SHORTCUT.to_string();
    shortcuts::validate_shortcuts(&settings)?;
    let previous = state.db.get_settings()?;
    desktop::sync_launch_at_startup(settings.launch_at_startup)?;
    let saved = state.db.save_settings(&settings)?;
    if let Err(error) = shortcuts::sync_registered_shortcuts(&app) {
        let _ = state.db.save_settings(&previous);
        let _ = desktop::sync_launch_at_startup(previous.launch_at_startup);
        let _ = shortcuts::sync_registered_shortcuts(&app);
        return Err(format!("Could not activate that shortcut: {error}"));
    }
    let _ = app.emit_to("popup", "settings-updated", saved.clone());
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
pub fn open_provider_guide(url: String) -> Result<(), String> {
    let parsed =
        reqwest::Url::parse(&url).map_err(|_| "Invalid provider guide URL.".to_string())?;
    if !is_allowed_provider_guide(&parsed) {
        return Err("This provider guide is not allowed.".to_string());
    }

    std::process::Command::new("rundll32.exe")
        .args(["url.dll,FileProtocolHandler", parsed.as_str()])
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Could not open the provider guide: {error}"))
}

fn is_allowed_provider_guide(url: &reqwest::Url) -> bool {
    const ALLOWED_HOSTS: [&str; 10] = [
        "aistudio.google.com",
        "console.groq.com",
        "openrouter.ai",
        "platform.openai.com",
        "platform.claude.com",
        "console.mistral.ai",
        "dashboard.cohere.com",
        "console.x.ai",
        "platform.deepseek.com",
        "ollama.com",
    ];
    url.scheme() == "https"
        && url
            .host_str()
            .is_some_and(|host| ALLOWED_HOSTS.contains(&host))
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
pub fn close_main_window(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let minimize_to_tray = state.db.get_settings()?.minimize_to_tray;
    if minimize_to_tray {
        desktop::hide_main_window(&app)
    } else {
        app.exit(0);
        Ok(())
    }
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
    let _ = store_selection_window(state.inner(), capture.source_window);

    let input = capture.text.trim().to_string();
    let payload = if input.is_empty() {
        empty_popup_payload()
    } else if is_non_prose_selection(&input) {
        invalid_selection_popup_payload()
    } else {
        let loading_payload = loading_popup_payload(input.clone());
        let _ = store_popup_payload(state.inner(), &loading_payload);
        let _ = desktop::show_popup_window_passive(&app, &loading_payload);

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
            // The panel flow is preview-first: never alter the source application
            // until the user explicitly presses Replace in the popup.
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
    let _ = store_selection_window(state.inner(), capture.source_window);
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
    let _ =
        apply_automatic_shortcut_output(
            &settings,
            &response.output,
            capture.previous_clipboard,
            capture.source_window,
        );

    let payload = PopupPayload::from((response, "shortcut"));
    let _ = store_popup_payload(state.inner(), &payload);
    if !settings.auto_replace && !settings.auto_copy {
        let _ = desktop::show_popup_window(&app, &payload);
    }
}

fn apply_automatic_shortcut_output(
    settings: &AppSettings,
    output: &str,
    previous_clipboard: Option<String>,
    target_window: Option<isize>,
) -> Result<(), String> {
    if settings.auto_replace {
        desktop::replace_selected_text(output, previous_clipboard, target_window)?;
        if settings.auto_copy {
            desktop::write_clipboard_text(output)?;
        }
    } else if settings.auto_copy {
        desktop::write_clipboard_text(output)?;
    }
    Ok(())
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
    let started = Instant::now();
    let settings = state.db.get_settings().unwrap_or_default();
    const MAX_INPUT_CHARS: usize = 50_000;
    let clean_input = request.input.trim().to_string();
    if clean_input.is_empty() {
        return Ok(RewriteResponse {
            input: String::new(),
            output: String::new(),
            mode: request.mode,
            provider: ProviderId::Offline,
            used_offline_fallback: true,
            character_count: 0,
            elapsed_ms: started.elapsed().as_millis() as u64,
        });
    }
    if clean_input.chars().count() > MAX_INPUT_CHARS {
        return Err(format!("Text is too long. Use {MAX_INPUT_CHARS} characters or fewer."));
    }

    let mut provider_settings = settings.provider.clone();
    provider_settings.custom_prompt = settings.custom_prompt.clone();
    let custom_requires_provider = request
        .custom_prompt
        .as_deref()
        .map(str::trim)
        .is_some_and(|instruction| !instruction.is_empty());
    let requires_provider = custom_requires_provider || matches!(request.mode, RewriteMode::Translate);
    let provider_result =
        providers::rewrite_with_provider(&state.client, &provider_settings, &request).await;
    let (output, provider, used_offline_fallback) = match provider_result {
        Ok(Some(output)) => (output, settings.provider.provider.clone(), false),
        Ok(None) if requires_provider => {
            let feature = if matches!(request.mode, RewriteMode::Translate) { "translation" } else { "Custom Prompt" };
            return Err(format!("Connect an AI provider to use {feature}."))
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
        elapsed_ms: started.elapsed().as_millis() as u64,
    };
    if let Err(error) = state.db.save_rewrite(&response) {
        eprintln!("CorteX could not save rewrite history: {error}");
    }
    Ok(response)
}

fn store_popup_payload(state: &AppState, payload: &PopupPayload) -> Result<(), String> {
    let mut last_popup = state
        .last_popup
        .lock()
        .map_err(|_| "popup state lock poisoned".to_string())?;
    *last_popup = Some(payload.clone());
    Ok(())
}

fn store_selection_window(state: &AppState, window: Option<isize>) -> Result<(), String> {
    let mut target = state
        .last_selection_window
        .lock()
        .map_err(|_| "selection target lock poisoned".to_string())?;
    *target = window;
    Ok(())
}

fn last_selection_window(state: &AppState) -> Result<Option<isize>, String> {
    state
        .last_selection_window
        .lock()
        .map_err(|_| "selection target lock poisoned".to_string())
        .map(|target| *target)
}

fn empty_popup_payload() -> PopupPayload {
    let output =
        "Select text in any app, then press Ctrl + Alt + X to rewrite it here.".to_string();
    let character_count = output.chars().count();
    PopupPayload {
        input: String::new(),
        output,
        mode: RewriteMode::FixGrammar,
        provider: ProviderId::Offline,
        used_offline_fallback: true,
        character_count,
        elapsed_ms: 0,
        source: "shortcut".to_string(),
        loading: false,
    }
}

fn invalid_selection_popup_payload() -> PopupPayload {
    let output = "CorteX copied a file or system ID instead of normal text. Select the sentence you want to rewrite, then press Ctrl + Alt + X again.".to_string();
    let character_count = output.chars().count();
    PopupPayload {
        input: String::new(), output, mode: RewriteMode::FixGrammar, provider: ProviderId::Offline,
        used_offline_fallback: true, character_count, elapsed_ms: 0,
        source: "shortcut".to_string(), loading: false,
    }
}

fn is_non_prose_selection(input: &str) -> bool {
    let parts: Vec<&str> = input.trim().split('-').collect();
    let uuid_lengths = [8, 4, 4, 4, 12];
    parts.len() == uuid_lengths.len() && parts.iter().zip(uuid_lengths).all(|(part, expected_length)| {
        part.len() == expected_length && part.bytes().all(|byte| byte.is_ascii_hexdigit())
    })
}

fn loading_popup_payload(input: String) -> PopupPayload {
    PopupPayload {
        input,
        output: String::new(),
        mode: RewriteMode::FixGrammar,
        provider: ProviderId::Offline,
        used_offline_fallback: false,
        character_count: 0,
        elapsed_ms: 0,
        source: "shortcut".to_string(),
        loading: true,
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
        elapsed_ms: 0,
        source: "shortcut".to_string(),
        loading: false,
    }
}

#[cfg(test)]
mod provider_guide_tests {
    use super::{is_allowed_provider_guide, is_non_prose_selection};

    #[test]
    fn allows_only_known_https_provider_guides() {
        let allowed = reqwest::Url::parse("https://console.groq.com/keys").unwrap();
        let wrong_scheme = reqwest::Url::parse("http://console.groq.com/keys").unwrap();
        let wrong_host = reqwest::Url::parse("https://example.com/keys").unwrap();

        assert!(is_allowed_provider_guide(&allowed));
        assert!(!is_allowed_provider_guide(&wrong_scheme));
        assert!(!is_allowed_provider_guide(&wrong_host));
    }

    #[test]
    fn detects_uuid_values_that_are_not_writing() {
        assert!(is_non_prose_selection("019f30fc-8b49-7022-b19f-7f1878543c00"));
        assert!(!is_non_prose_selection("Hey, I am going to sleep."));
    }
}
