use crate::types::AppConfig;
use std::path::PathBuf;

pub fn config_dir() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| std::env::temp_dir());
    path.push("kimi-island");
    std::fs::create_dir_all(&path).ok();
    path
}

pub fn config_file() -> PathBuf {
    let mut path = config_dir();
    path.push("config.json");
    path
}

pub fn read_config() -> AppConfig {
    let path = config_file();
    match std::fs::read_to_string(&path) {
        Ok(content) => {
            match serde_json::from_str::<AppConfig>(&content) {
                Ok(mut config) => {
                    // Migration: ensure kimi_token is also in kimi_tokens for multi-token support
                    if let Some(ref token) = config.kimi_token {
                        if !token.is_empty() && !config.kimi_tokens.contains(token) {
                            config.kimi_tokens.push(token.clone());
                        }
                    }
                    config.compact_width = config.compact_width.clamp(240, 480);
                    config.yellow_threshold = config.yellow_threshold.clamp(5, 50);
                    config.red_threshold = config.red_threshold.clamp(1, 20);
                    if config.yellow_threshold <= config.red_threshold {
                        std::mem::swap(&mut config.yellow_threshold, &mut config.red_threshold);
                    }
                    config.poll_interval_normal = config.poll_interval_normal.clamp(5, 3600);
                    config.poll_interval_warning = config.poll_interval_warning.clamp(5, 3600);
                    config.poll_interval_critical = config.poll_interval_critical.clamp(5, 3600);
                    config
                }
                Err(e) => {
                    eprintln!("Config parse error: {}, using defaults", e);
                    AppConfig::default()
                }
            }
        }
        Err(_) => AppConfig::default(),
    }
}

pub fn write_config(config: &AppConfig) -> Result<(), String> {
    let path = config_file();
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_config() -> AppConfig {
    read_config()
}

#[tauri::command]
pub fn set_config(config: AppConfig) -> Result<(), String> {
    write_config(&config)
}
