use std::{
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use arboard::Clipboard;
use tauri::{
    AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, PhysicalSize, WebviewUrl,
    Theme, WebviewWindow, WebviewWindowBuilder,
};

use crate::models::PopupPayload;

#[derive(Debug, Clone)]
pub struct CapturedText {
    pub text: String,
    pub previous_clipboard: Option<String>,
}

pub fn read_clipboard_text() -> Result<String, String> {
    let mut clipboard = Clipboard::new().map_err(|error| error.to_string())?;
    clipboard.get_text().map_err(|error| error.to_string())
}

pub fn write_clipboard_text(text: &str) -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|error| error.to_string())?;
    clipboard
        .set_text(text.to_string())
        .map_err(|error| error.to_string())
}

pub fn capture_selected_text() -> Result<CapturedText, String> {
    let previous_clipboard = read_clipboard_text().ok();
    let sentinel = format!(
        "__CORTEX_EMPTY_SELECTION_{}__",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default()
    );

    write_clipboard_text(&sentinel)?;
    send_copy_shortcut()?;
    thread::sleep(Duration::from_millis(120));
    let copied_text = read_clipboard_text().unwrap_or_default();
    let text = if copied_text == sentinel {
        String::new()
    } else {
        copied_text
    };

    if let Some(previous) = previous_clipboard.as_deref() {
        let _ = write_clipboard_text(previous);
    } else {
        let _ = write_clipboard_text("");
    }

    Ok(CapturedText {
        text,
        previous_clipboard,
    })
}

pub fn replace_selected_text(text: &str, previous_clipboard: Option<String>) -> Result<(), String> {
    let restore_clipboard = previous_clipboard.or_else(|| read_clipboard_text().ok());
    write_clipboard_text(text)?;
    thread::sleep(Duration::from_millis(35));
    send_paste_shortcut()?;
    thread::sleep(Duration::from_millis(110));

    if let Some(previous) = restore_clipboard.as_deref() {
        let _ = write_clipboard_text(previous);
    }

    Ok(())
}

pub fn show_popup_window(app: &AppHandle, payload: &PopupPayload) -> Result<(), String> {
    let popup = match app.get_webview_window("popup") {
        Some(window) => window,
        None => WebviewWindowBuilder::new(
            app,
            "popup",
            WebviewUrl::App("index.html?view=popup".into()),
        )
        .title("CorteX Rewrite")
        .inner_size(1008.0, 448.0)
        .min_inner_size(760.0, 390.0)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .visible(false)
        .build()
        .map_err(|error| error.to_string())?,
    };

    if let Some((cursor_x, cursor_y)) = cursor_position() {
        let (max_x, max_y) = visible_screen_limit(app).unwrap_or((1920, 1080));
        let x = (cursor_x - 500).clamp(18, (max_x - 1030).max(18));
        let y = (cursor_y + 18).clamp(18, (max_y - 470).max(18));
        let _ = popup.set_position(PhysicalPosition::new(x, y));
    }

    let _ = popup.unminimize();
    let _ = popup.set_always_on_top(true);
    popup.show().map_err(|error| error.to_string())?;
    popup.set_focus().map_err(|error| error.to_string())?;
    popup
        .emit("popup-context", payload.clone())
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn show_main_window(app: &AppHandle) -> Result<(), String> {
    let window = get_or_create_main_window(app)?;
    restore_main_window(&window, app)
}

pub fn hide_main_window(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unmaximize();
        let _ = window.set_always_on_top(false);
        window.hide().map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn get_or_create_main_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    if let Some(window) = app.get_webview_window("main") {
        return Ok(window);
    }

    WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
        .title("CorteX")
        .inner_size(1180.0, 640.0)
        .min_inner_size(860.0, 520.0)
        .center()
        .decorations(true)
        .transparent(false)
        .resizable(true)
        .theme(Some(Theme::Dark))
        .visible(false)
        .build()
        .map_err(|error| error.to_string())
}

fn restore_main_window(window: &WebviewWindow, app: &AppHandle) -> Result<(), String> {
    let _ = window.set_always_on_top(false);
    let _ = window.unminimize();

    if is_window_off_screen(window, app) {
        let _ = window.set_size(LogicalSize::new(1180.0, 640.0));
        let _ = window.center();
    }

    window.show().map_err(|error| error.to_string())?;
    let _ = window.unminimize();
    let _ = window.set_focus();
    Ok(())
}

fn is_window_off_screen(window: &WebviewWindow, app: &AppHandle) -> bool {
    let position = match window.outer_position() {
        Ok(position) => position,
        Err(_) => return true,
    };
    let size = window
        .outer_size()
        .unwrap_or_else(|_| PhysicalSize::new(1180, 640));
    let Some((max_x, max_y)) = visible_screen_limit(app) else {
        return false;
    };

    position.x + size.width as i32 <= 40
        || position.y + size.height as i32 <= 40
        || position.x >= max_x - 40
        || position.y >= max_y - 40
}

fn visible_screen_limit(app: &AppHandle) -> Option<(i32, i32)> {
    let monitor = app.primary_monitor().ok().flatten()?;
    let size = monitor.size();
    Some((size.width as i32, size.height as i32))
}

fn send_copy_shortcut() -> Result<(), String> {
    send_ctrl_key(0x43)
}

fn send_paste_shortcut() -> Result<(), String> {
    send_ctrl_key(0x56)
}

#[cfg(windows)]
fn send_ctrl_key(key: u16) -> Result<(), String> {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
        VIRTUAL_KEY, VK_CONTROL,
    };

    fn keyboard_input(key: VIRTUAL_KEY, flags: KEYBD_EVENT_FLAGS) -> INPUT {
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: key,
                    wScan: 0,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }

    let inputs = [
        keyboard_input(VK_CONTROL, KEYBD_EVENT_FLAGS(0)),
        keyboard_input(VIRTUAL_KEY(key), KEYBD_EVENT_FLAGS(0)),
        keyboard_input(VIRTUAL_KEY(key), KEYEVENTF_KEYUP),
        keyboard_input(VK_CONTROL, KEYEVENTF_KEYUP),
    ];

    let sent = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
    if sent == inputs.len() as u32 {
        Ok(())
    } else {
        Err("Windows did not accept the keyboard shortcut".to_string())
    }
}

#[cfg(not(windows))]
fn send_ctrl_key(_key: u16) -> Result<(), String> {
    Err("Clipboard replacement shortcuts are only implemented for Windows.".to_string())
}

#[cfg(windows)]
fn cursor_position() -> Option<(i32, i32)> {
    use windows::Win32::{Foundation::POINT, UI::WindowsAndMessaging::GetCursorPos};

    let mut point = POINT::default();
    let ok = unsafe { GetCursorPos(&mut point).is_ok() };
    ok.then_some((point.x, point.y))
}

#[cfg(not(windows))]
fn cursor_position() -> Option<(i32, i32)> {
    None
}
