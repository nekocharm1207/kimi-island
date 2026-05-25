mod cache;
mod config;
mod kime_service;
mod tray;
mod types;
mod window_manager;

use tauri::{Manager, Emitter};

#[tauri::command]
async fn get_usage_data(
    force: bool,
    _app: tauri::AppHandle,
) -> Result<types::KimeUsageData, String> {
    let cfg = config::read_config();
    
    if !force {
        if let Some(cached) = cache::read_cache() {
            return Ok(cached);
        }
    }

    let data = kime_service::fetch_usage_data(&cfg).await?;
    let _ = cache::write_cache(&data);
    Ok(data)
}

#[tauri::command]
async fn save_kimi_token(
    token: String,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut cfg = config::read_config();
    cfg.kimi_token = Some(token.clone());
    // Deduplicate and prioritize new token
    cfg.kimi_tokens.retain(|t| t != &token);
    cfg.kimi_tokens.insert(0, token);
    config::write_config(&cfg)?;
    
    // Destroy auth window completely so it can be reopened
    if let Some(auth_window) = app.get_webview_window("auth") {
        let _ = auth_window.destroy();
    }
    
    // Clear cache and notify frontend to refresh
    let _ = cache::clear_cache();
    let _ = app.emit("kimi:token_saved", ());
    
    Ok(())
}

#[tauri::command]
async fn open_auth_window(app: tauri::AppHandle) -> Result<(), String> {
    // If already open, just focus it
    if let Some(w) = app.get_webview_window("auth") {
        let _ = w.set_focus();
        return Ok(());
    }

    let init_script = r#"
(function() {
    function checkToken() {
        try {
            const token = localStorage.getItem('access_token');
            if (token && token.length > 10) {
                window.__TAURI_INTERNALS__.invoke('save_kimi_token', { token: token })
                    .then(() => console.log('[KimiIsland] Token saved successfully'))
                    .catch(e => console.error('[KimiIsland] Token save failed:', e));
            } else {
                console.log('[KimiIsland] No access_token found yet');
            }
        } catch(e) {
            console.error('[KimiIsland] Check token error:', e);
        }
    }
    // Check on load
    if (document.readyState === 'complete') {
        checkToken();
    } else {
        window.addEventListener('load', checkToken);
    }
    // Also check periodically (user might login after page load)
    setInterval(checkToken, 2000);
})();
"#;

    let auth_window = tauri::WebviewWindowBuilder::new(
        &app,
        "auth",
        tauri::WebviewUrl::External("https://kimi.com/code/console".parse().unwrap())
    )
    .title("Kimi 登录")
    .inner_size(960.0, 720.0)
    .center()
    .initialization_script(init_script)
    .build()
    .map_err(|e| e.to_string())?;

    // Listen for window destruction to ensure clean state
    let app_handle = app.clone();
    auth_window.on_window_event(move |event| {
        if matches!(event, tauri::WindowEvent::Destroyed) {
            let _ = app_handle.emit("kimi:auth_closed", ());
        }
    });

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_usage_data,
            save_kimi_token,
            open_auth_window,
            window_manager::set_island_mode,
            window_manager::open_kimi_website,
            config::get_config,
            config::set_config,
        ])
        .setup(|app| {
            tray::create_tray(app.handle())?;

            if let Some(window) = app.get_webview_window("island") {
                let _ = window.set_decorations(false);
                let _ = window.set_shadow(false);
                let _ = window.set_resizable(false);
                let _ = window.set_skip_taskbar(true);
                let _ = window.set_background_color(Some(tauri::window::Color(0, 0, 0, 0)));
                let _ = window_manager::apply_native_topmost(&window);
                let _ = window_manager::apply_capsule_hit_region(&window, 320.0, 48.0);
                let _ = window_manager::position_window_top_center(&window, 320.0);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
