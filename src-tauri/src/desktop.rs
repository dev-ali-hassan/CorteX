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

const POPUP_WIDTH: f64 = 650.0;
const POPUP_HEIGHT: f64 = 310.0;
const POPUP_GAP: i32 = 14;

#[derive(Debug, Clone)]
pub struct CapturedText {
    pub text: String,
    pub previous_clipboard: Option<String>,
    pub source_window: Option<isize>,
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
    let source_window = foreground_window_handle();
    let previous_clipboard = read_clipboard_text().ok();
    let sentinel = format!(
        "__CORTEX_EMPTY_SELECTION_{}__",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default()
    );

    // Never synthesize Ctrl+C while Ctrl/Alt from the global shortcut are still held.
    // Otherwise some editors can receive a literal `c` or lose their selection.
    wait_for_shortcut_modifiers_release();
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
        source_window,
    })
}

pub fn replace_selected_text(
    text: &str,
    previous_clipboard: Option<String>,
    target_window: Option<isize>,
) -> Result<(), String> {
    let restore_clipboard = previous_clipboard.or_else(|| read_clipboard_text().ok());
    write_clipboard_text(text)?;
    if let Some(target) = target_window {
        restore_foreground_window(target);
        thread::sleep(Duration::from_millis(90));
    }
    wait_for_shortcut_modifiers_release();
    thread::sleep(Duration::from_millis(45));
    send_paste_shortcut()?;
    thread::sleep(Duration::from_millis(180));

    if let Some(previous) = restore_clipboard.as_deref() {
        let _ = write_clipboard_text(previous);
    }

    Ok(())
}

fn get_or_create_popup_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    match app.get_webview_window("popup") {
        Some(window) => Ok(window),
        None => WebviewWindowBuilder::new(
            app,
            "popup",
            WebviewUrl::App("index.html?view=popup".into()),
        )
        .title("CorteX Rewrite")
        .inner_size(POPUP_WIDTH, POPUP_HEIGHT)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .visible(false)
        .build()
        .map_err(|error| error.to_string()),
    }
}

pub fn prepare_popup_window(app: &AppHandle) -> Result<(), String> {
    get_or_create_popup_window(app).map(|_| ())
}

fn present_popup_window(
    app: &AppHandle,
    payload: &PopupPayload,
    take_focus: bool,
) -> Result<(), String> {
    let popup = get_or_create_popup_window(app)?;

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
    if take_focus {
        popup.show().map_err(|error| error.to_string())?;
    } else {
        show_window_without_activation(&popup)?;
    }
    popup
        .emit("popup-context", payload.clone())
        .map_err(|error| error.to_string())?;
    // Result delivery must not depend on Windows granting a foreground-focus request.
    if take_focus {
        let _ = popup.set_focus();
    }
    Ok(())
}

pub fn show_popup_window(app: &AppHandle, payload: &PopupPayload) -> Result<(), String> {
    present_popup_window(app, payload, true)
}

pub fn show_popup_window_passive(app: &AppHandle, payload: &PopupPayload) -> Result<(), String> {
    present_popup_window(app, payload, false)
}

#[cfg(windows)]
fn show_window_without_activation(window: &WebviewWindow) -> Result<(), String> {
    use windows::Win32::{
        Foundation::HWND,
        UI::WindowsAndMessaging::{ShowWindowAsync, SW_SHOWNOACTIVATE},
    };

    let handle = window.hwnd().map_err(|error| error.to_string())?;
    let hwnd = HWND(handle.0 as *mut core::ffi::c_void);
    let _ = unsafe { ShowWindowAsync(hwnd, SW_SHOWNOACTIVATE) };
    Ok(())
}

#[cfg(not(windows))]
fn show_window_without_activation(window: &WebviewWindow) -> Result<(), String> {
    window.show().map_err(|error| error.to_string())
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
    force_windows_restore(window);
    let _ = window.set_focus();
    Ok(())
}

#[cfg(windows)]
fn force_windows_restore(window: &WebviewWindow) {
    use windows::Win32::{
        Foundation::HWND,
        UI::WindowsAndMessaging::{SetForegroundWindow, ShowWindowAsync, SW_RESTORE},
    };

    if let Ok(handle) = window.hwnd() {
        let hwnd = HWND(handle.0 as *mut core::ffi::c_void);
        let _ = unsafe { ShowWindowAsync(hwnd, SW_RESTORE) };
        let _ = unsafe { SetForegroundWindow(hwnd) };
    }
}

#[cfg(not(windows))]
fn force_windows_restore(_window: &WebviewWindow) {}

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
fn foreground_window_handle() -> Option<isize> {
    use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

    let window = unsafe { GetForegroundWindow() };
    (!window.0.is_null()).then_some(window.0 as isize)
}

#[cfg(not(windows))]
fn foreground_window_handle() -> Option<isize> {
    None
}

#[cfg(windows)]
fn restore_foreground_window(handle: isize) {
    use windows::Win32::{
        Foundation::HWND,
        UI::WindowsAndMessaging::{SetForegroundWindow, ShowWindowAsync, SW_RESTORE},
    };

    let window = HWND(handle as *mut core::ffi::c_void);
    let _ = unsafe { ShowWindowAsync(window, SW_RESTORE) };
    let _ = unsafe { SetForegroundWindow(window) };
}

#[cfg(not(windows))]
fn restore_foreground_window(_handle: isize) {}

#[cfg(windows)]
fn wait_for_shortcut_modifiers_release() {
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_CONTROL, VK_MENU, VK_SHIFT};

    for _ in 0..40 {
        let pressed = unsafe {
            GetAsyncKeyState(VK_CONTROL.0 as i32) < 0
                || GetAsyncKeyState(VK_MENU.0 as i32) < 0
                || GetAsyncKeyState(VK_SHIFT.0 as i32) < 0
        };
        if !pressed {
            return;
        }
        thread::sleep(Duration::from_millis(15));
    }
}

#[cfg(not(windows))]
fn wait_for_shortcut_modifiers_release() {
    thread::sleep(Duration::from_millis(70));
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
