use tauri::{Emitter, Manager, WebviewWindow};

const COMPACT_WIDTH: f64 = 320.0;
const COMPACT_HEIGHT: f64 = 48.0;
const EXPANDED_WIDTH: f64 = 420.0;
const EXPANDED_HEIGHT: f64 = 460.0;
const SHOULDER_RADIUS: f64 = 6.0;

#[cfg(windows)]
use windows_sys::Win32::Graphics::Gdi::{
    CombineRgn, CreateRectRgn, DeleteObject, RGN_OR, SetWindowRgn,
};
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    SetWindowPos, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOOWNERZORDER, SWP_NOSIZE,
};

fn logical_to_physical(value: f64, scale: f64) -> i32 {
    (value * scale).round() as i32
}

#[cfg(windows)]
pub fn apply_native_topmost<R: tauri::Runtime>(window: &WebviewWindow<R>) -> tauri::Result<()> {
    let hwnd = window.hwnd()?.0 as isize;
    let flags = SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOOWNERZORDER;
    unsafe {
        SetWindowPos(hwnd as _, HWND_TOPMOST, 0, 0, 0, 0, flags);
    }
    Ok(())
}

#[cfg(not(windows))]
pub fn apply_native_topmost<R: tauri::Runtime>(_window: &WebviewWindow<R>) -> tauri::Result<()> {
    Ok(())
}

#[cfg(windows)]
pub fn apply_capsule_hit_region<R: tauri::Runtime>(
    window: &WebviewWindow<R>,
    width: f64,
    height: f64,
) -> tauri::Result<()> {
    let hwnd = window.hwnd()?.0 as isize;
    let scale = window.scale_factor()?;
    let right = logical_to_physical(width, scale);
    let bottom = logical_to_physical(height, scale);
    let r = logical_to_physical(SHOULDER_RADIUS, scale).max(1);

    let main = unsafe { CreateRectRgn(r, 0, right - r, bottom) };

    for row in 0..r {
        let dy = r - row;
        let threshold = ((r * r - dy * dy) as f64).sqrt().round() as i32;
        let y_top = row;
        let y_bottom = row + 1;

        let left_rgn = unsafe { CreateRectRgn(threshold, y_top, r, y_bottom) };
        if left_rgn != std::ptr::null_mut() {
            unsafe {
                CombineRgn(main, main, left_rgn, RGN_OR);
                DeleteObject(left_rgn);
            }
        }

        let right_rgn = unsafe { CreateRectRgn(right - r, y_top, right - threshold, y_bottom) };
        if right_rgn != std::ptr::null_mut() {
            unsafe {
                CombineRgn(main, main, right_rgn, RGN_OR);
                DeleteObject(right_rgn);
            }
        }
    }

    unsafe { SetWindowRgn(hwnd as _, main, 1) };
    Ok(())
}

#[cfg(not(windows))]
pub fn apply_capsule_hit_region<R: tauri::Runtime>(
    _window: &WebviewWindow<R>,
    _width: f64,
    _height: f64,
) -> tauri::Result<()> {
    Ok(())
}

#[cfg(windows)]
pub fn apply_rectangular_hit_region<R: tauri::Runtime>(
    window: &WebviewWindow<R>,
    width: f64,
    height: f64,
) -> tauri::Result<()> {
    let hwnd = window.hwnd()?.0 as isize;
    let scale = window.scale_factor()?;
    let right = logical_to_physical(width, scale);
    let bottom = logical_to_physical(height, scale);
    let rgn = unsafe { CreateRectRgn(0, 0, right, bottom) };
    unsafe { SetWindowRgn(hwnd as _, rgn, 1) };
    Ok(())
}

#[cfg(not(windows))]
pub fn apply_rectangular_hit_region<R: tauri::Runtime>(
    _window: &WebviewWindow<R>,
    _width: f64,
    _height: f64,
) -> tauri::Result<()> {
    Ok(())
}

pub fn position_window_top_center<R: tauri::Runtime>(
    window: &WebviewWindow<R>,
    width: f64,
) -> tauri::Result<()> {
    let monitor = window
        .primary_monitor()
        .ok()
        .flatten()
        .or_else(|| window.current_monitor().ok().flatten());

    if let Some(monitor) = monitor {
        let size = monitor.size();
        let pos = monitor.position();
        let scale = monitor.scale_factor();
        let monitor_x = pos.x as f64 / scale;
        let monitor_y = pos.y as f64 / scale;
        let monitor_width = size.width as f64 / scale;
        let x = monitor_x + ((monitor_width - width) / 2.0).max(0.0);
        let y = monitor_y;
        window.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(x, y)))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn set_island_mode(
    app: tauri::AppHandle,
    mode: String,
) -> Result<(), String> {
    let window = app
        .get_webview_window("island")
        .ok_or("window not found")?;

    match mode.as_str() {
        "compact" => {
            window
                .set_size(tauri::Size::Logical(tauri::LogicalSize::new(COMPACT_WIDTH, COMPACT_HEIGHT)))
                .map_err(|e| e.to_string())?;
            position_window_top_center(&window, COMPACT_WIDTH).map_err(|e| e.to_string())?;
            apply_capsule_hit_region(&window, COMPACT_WIDTH, COMPACT_HEIGHT).map_err(|e| e.to_string())?;
        }
        "expanded" => {
            window
                .set_size(tauri::Size::Logical(tauri::LogicalSize::new(EXPANDED_WIDTH, EXPANDED_HEIGHT)))
                .map_err(|e| e.to_string())?;
            position_window_top_center(&window, EXPANDED_WIDTH).map_err(|e| e.to_string())?;
            apply_rectangular_hit_region(&window, EXPANDED_WIDTH, EXPANDED_HEIGHT).map_err(|e| e.to_string())?;
        }
        "hidden" => {
            window.hide().map_err(|e| e.to_string())?;
        }
        _ => {}
    }

    if mode != "hidden" {
        window.show().map_err(|e| e.to_string())?;
        apply_native_topmost(&window).map_err(|e| e.to_string())?;
    }

    let _ = app.emit("island:mode_changed", &mode);
    Ok(())
}

#[tauri::command]
pub async fn open_kimi_website() -> Result<(), String> {
    tauri::async_runtime::spawn(async move {
        let _ = tauri_plugin_opener::open_url("https://www.kimi.com", None::<&str>);
    });
    Ok(())
}
