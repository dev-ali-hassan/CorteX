use std::{
    path::Path,
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use arboard::Clipboard;
use tauri::{
    AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, PhysicalSize, Theme, WebviewUrl,
    WebviewWindow, WebviewWindowBuilder,
};

use crate::models::PopupPayload;

const POPUP_WIDTH: f64 = 620.0;
const POPUP_HEIGHT: f64 = 360.0;
const POPUP_GAP: i32 = 14;

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

    // Global-shortcut modifier keys can still be physically down when the callback fires.
    // Give Windows a moment to release them before synthesizing Ctrl+C.
    thread::sleep(Duration::from_millis(70));
    write_clipboard_text(&sentinel)?;
    send_copy_shortcut()?;
    let mut copied_text = sentinel.clone();
    for _ in 0..10 {
        thread::sleep(Duration::from_millis(35));
        copied_text = read_clipboard_text().unwrap_or_else(|_| sentinel.clone());
        if copied_text != sentinel {
            break;
        }
    }
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
        .inner_size(POPUP_WIDTH, POPUP_HEIGHT)
        .decorations(false)
        .transparent(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .visible(false)
        .build()
        .map_err(|error| error.to_string())?,
    };

    popup
        .set_size(LogicalSize::new(POPUP_WIDTH, POPUP_HEIGHT))
        .map_err(|error| error.to_string())?;

    if let Some((cursor_x, cursor_y)) = cursor_position() {
        let scale = popup.scale_factor().unwrap_or(1.0);
        let popup_width = (POPUP_WIDTH * scale).round() as i32;
        let popup_height = (POPUP_HEIGHT * scale).round() as i32;
        let (left, top, right, bottom) =
            monitor_bounds_at_cursor(app, cursor_x, cursor_y).unwrap_or((0, 0, 1920, 1080));
        let margin = (12.0 * scale).round() as i32;
        let gap = (POPUP_GAP as f64 * scale).round() as i32;

        let (x, y) = popup_position(
            cursor_x,
            cursor_y,
            popup_width,
            popup_height,
            (left, top, right, bottom),
            margin,
            gap,
        );

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

#[cfg(windows)]
pub fn sync_launch_at_startup(enabled: bool) -> Result<(), String> {
    use std::{os::windows::process::CommandExt, process::Command};

    const CREATE_NO_WINDOW: u32 = 0x08000000;
    const RUN_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
    const VALUE_NAME: &str = "CorteX";

    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    let mut command = Command::new("reg.exe");
    command.creation_flags(CREATE_NO_WINDOW);

    if enabled {
        command.args([
            "ADD",
            RUN_KEY,
            "/v",
            VALUE_NAME,
            "/t",
            "REG_SZ",
            "/d",
            &startup_command_line(&executable),
            "/f",
        ]);
        let output = command.output().map_err(|error| error.to_string())?;
        if !output.status.success() {
            return Err(registry_error("enable launch at startup", &output.stderr));
        }
        return if launch_at_startup_enabled()? {
            Ok(())
        } else {
            Err("Windows did not keep the CorteX startup entry.".to_string())
        };
    }

    command.args(["DELETE", RUN_KEY, "/v", VALUE_NAME, "/f"]);
    let _ = command.output();

    let mut query = Command::new("reg.exe");
    query
        .creation_flags(CREATE_NO_WINDOW)
        .args(["QUERY", RUN_KEY, "/v", VALUE_NAME]);
    let output = query.output().map_err(|error| error.to_string())?;
    if output.status.success() {
        Err("Windows kept the CorteX startup entry. Try again as the current user.".to_string())
    } else {
        Ok(())
    }
}

#[cfg(windows)]
pub fn launch_at_startup_enabled() -> Result<bool, String> {
    use std::{os::windows::process::CommandExt, process::Command};

    const CREATE_NO_WINDOW: u32 = 0x08000000;
    const RUN_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
    const VALUE_NAME: &str = "CorteX";

    let output = Command::new("reg.exe")
        .creation_flags(CREATE_NO_WINDOW)
        .args(["QUERY", RUN_KEY, "/v", VALUE_NAME])
        .output()
        .map_err(|error| error.to_string())?;

    if !output.status.success() {
        return Ok(false);
    }

    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    let value = String::from_utf8_lossy(&output.stdout).to_ascii_lowercase();
    Ok(value.contains(&executable.display().to_string().to_ascii_lowercase()))
}

#[cfg(not(windows))]
pub fn sync_launch_at_startup(_enabled: bool) -> Result<(), String> {
    Ok(())
}

#[cfg(not(windows))]
pub fn launch_at_startup_enabled() -> Result<bool, String> {
    Ok(false)
}

fn startup_command_line(executable: &Path) -> String {
    format!("\"{}\" --background", executable.display())
}

#[cfg(windows)]
fn registry_error(action: &str, stderr: &[u8]) -> String {
    let details = String::from_utf8_lossy(stderr).trim().to_string();
    if details.is_empty() {
        format!("Windows could not {action}.")
    } else {
        format!("Windows could not {action}: {details}")
    }
}

fn popup_position(
    cursor_x: i32,
    cursor_y: i32,
    popup_width: i32,
    popup_height: i32,
    bounds: (i32, i32, i32, i32),
    margin: i32,
    gap: i32,
) -> (i32, i32) {
    let (left, top, right, bottom) = bounds;
    let preferred_right = cursor_x + gap;
    let x = if preferred_right + popup_width <= right - margin {
        preferred_right
    } else {
        cursor_x - popup_width - gap
    }
    .clamp(
        left + margin,
        (right - popup_width - margin).max(left + margin),
    );

    let preferred_below = cursor_y + gap;
    let y = if preferred_below + popup_height <= bottom - margin {
        preferred_below
    } else {
        cursor_y - popup_height - gap
    }
    .clamp(
        top + margin,
        (bottom - popup_height - margin).max(top + margin),
    );

    (x, y)
}

pub fn hide_popup_window(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("popup") {
        window.hide().map_err(|error| error.to_string())?;
    }
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

fn monitor_bounds_at_cursor(
    app: &AppHandle,
    cursor_x: i32,
    cursor_y: i32,
) -> Option<(i32, i32, i32, i32)> {
    let monitors = app.available_monitors().ok()?;
    let monitor = monitors
        .into_iter()
        .find(|monitor| {
            let position = monitor.position();
            let size = monitor.size();
            cursor_x >= position.x
                && cursor_x < position.x + size.width as i32
                && cursor_y >= position.y
                && cursor_y < position.y + size.height as i32
        })
        .or_else(|| app.primary_monitor().ok().flatten())?;
    let position = monitor.position();
    let size = monitor.size();

    Some((
        position.x,
        position.y,
        position.x + size.width as i32,
        position.y + size.height as i32,
    ))
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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{popup_position, startup_command_line};

    #[test]
    fn popup_opens_to_the_right_and_below_when_space_exists() {
        assert_eq!(
            popup_position(400, 300, 620, 360, (0, 0, 1920, 1080), 12, 14),
            (414, 314)
        );
    }

    #[test]
    fn popup_flips_left_and_above_near_monitor_edges() {
        assert_eq!(
            popup_position(1800, 1000, 620, 360, (0, 0, 1920, 1080), 12, 14),
            (1166, 626)
        );
    }

    #[test]
    fn popup_is_clamped_inside_a_secondary_monitor() {
        assert_eq!(
            popup_position(-1890, 20, 620, 360, (-1920, 0, 0, 1080), 12, 14),
            (-1876, 34)
        );
    }

    #[test]
    fn startup_command_quotes_paths_and_starts_in_background() {
        assert_eq!(
            startup_command_line(Path::new(r"C:\Program Files\CorteX\CorteX.exe")),
            r#""C:\Program Files\CorteX\CorteX.exe" --background"#
        );
    }
}
