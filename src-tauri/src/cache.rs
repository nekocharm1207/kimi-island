use crate::types::KimeUsageData;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const CACHE_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    version: u32,
    fetched_at: String,
    data: KimeUsageData,
}

pub fn cache_dir() -> PathBuf {
    let mut path = dirs::cache_dir().unwrap_or_else(|| std::env::temp_dir());
    path.push("kimi-island");
    std::fs::create_dir_all(&path).ok();
    path
}

pub fn cache_file() -> PathBuf {
    let mut path = cache_dir();
    path.push("cache.json");
    path
}

pub fn read_cache() -> Option<KimeUsageData> {
    let path = cache_file();
    let content = std::fs::read_to_string(path).ok()?;
    let entry: CacheEntry = serde_json::from_str(&content).ok()?;
    if entry.version != CACHE_VERSION {
        return None;
    }
    // Simple TTL: 24 hours
    let fetched: chrono::DateTime<chrono::Utc> = entry.fetched_at.parse().ok()?;
    let now = chrono::Utc::now();
    if now.signed_duration_since(fetched).num_hours() > 24 {
        return None;
    }
    Some(entry.data)
}

pub fn write_cache(data: &KimeUsageData) -> Result<(), String> {
    let path = cache_file();
    let entry = CacheEntry {
        version: CACHE_VERSION,
        fetched_at: chrono::Utc::now().to_rfc3339(),
        data: data.clone(),
    };
    let json = serde_json::to_string_pretty(&entry).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())
}

pub fn clear_cache() -> Result<(), String> {
    let path = cache_file();
    if path.exists() {
        std::fs::remove_file(path).map_err(|e| e.to_string())
    } else {
        Ok(())
    }
}
