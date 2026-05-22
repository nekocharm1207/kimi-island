use serde::{Deserialize, Serialize};

// ============================================
// Frontend-facing types (stable contract)
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KimeUsageData {
    pub current_plan: String,
    pub validity: ValidityInfo,
    pub weekly_usage: UsageInfo,
    pub usage_ratio: f64,
    pub rate_limit_details: RateLimitDetails,
    pub model_permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidityInfo {
    pub current_end_time: String,
    pub days_remaining: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageInfo {
    pub used: u64,
    pub total: u64,
    pub unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitDetails {
    pub rpm: RateLimitItem,
    pub tpm: RateLimitItem,
    pub rpd: RateLimitItem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitItem {
    pub current: u32,
    pub limit: u32,
    pub remaining: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_preferred_display")]
    pub preferred_display: String,
    #[serde(default = "default_compact_width")]
    pub compact_width: u32,
    #[serde(default = "default_yellow_threshold")]
    pub yellow_threshold: u32,
    #[serde(default = "default_red_threshold")]
    pub red_threshold: u32,
    #[serde(default = "default_poll_interval_normal")]
    pub poll_interval_normal: u64,
    #[serde(default = "default_poll_interval_warning")]
    pub poll_interval_warning: u64,
    #[serde(default = "default_poll_interval_critical")]
    pub poll_interval_critical: u64,
    #[serde(default = "default_auto_collapse_delay")]
    pub auto_collapse_delay: u64,
    #[serde(default)]
    pub auto_expand_on_warning: bool,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default)]
    pub sound_on_warning: bool,
    #[serde(default)]
    pub autostart: bool,
    #[serde(default)]
    pub kimi_token: Option<String>,
}

fn default_preferred_display() -> String { "primary".to_string() }
fn default_compact_width() -> u32 { 320 }
fn default_yellow_threshold() -> u32 { 30 }
fn default_red_threshold() -> u32 { 10 }
fn default_poll_interval_normal() -> u64 { 60 }
fn default_poll_interval_warning() -> u64 { 30 }
fn default_poll_interval_critical() -> u64 { 15 }
fn default_auto_collapse_delay() -> u64 { 2000 }
fn default_theme() -> String { "system".to_string() }

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            preferred_display: default_preferred_display(),
            compact_width: default_compact_width(),
            yellow_threshold: default_yellow_threshold(),
            red_threshold: default_red_threshold(),
            poll_interval_normal: default_poll_interval_normal(),
            poll_interval_warning: default_poll_interval_warning(),
            poll_interval_critical: default_poll_interval_critical(),
            auto_collapse_delay: default_auto_collapse_delay(),
            auto_expand_on_warning: false,
            theme: default_theme(),
            sound_on_warning: false,
            autostart: false,
            kimi_token: None,
        }
    }
}

// ============================================
// Raw API response types (Kimi internal API)
// ============================================

#[derive(Debug, Clone, Deserialize)]
pub struct GetSubscriptionResponse {
    pub subscription: Option<Subscription>,
    pub balances: Vec<Balance>,
    pub subscribed: bool,
    pub capabilities: Vec<Capability>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Subscription {
    #[serde(rename = "subscriptionId")]
    pub subscription_id: String,
    pub goods: Goods,
    #[serde(rename = "currentEndTime")]
    pub current_end_time: String,
    #[serde(rename = "currentStartTime")]
    pub current_start_time: Option<String>,
    pub status: String,
    pub active: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Goods {
    pub id: String,
    pub title: String,
    #[serde(rename = "durationDays")]
    pub duration_days: i32,
    #[serde(rename = "membershipLevel")]
    pub membership_level: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Balance {
    pub id: String,
    pub feature: String,
    #[serde(rename = "type")]
    pub balance_type: String,
    pub unit: String,
    #[serde(rename = "amountUsedRatio")]
    pub amount_used_ratio: f64,
    #[serde(rename = "expireTime")]
    pub expire_time: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Capability {
    pub feature: String,
    pub constraint: Constraint,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Constraint {
    pub parallelism: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetUsagesResponse {
    pub usages: Vec<Usage>,
    #[serde(rename = "totalQuota")]
    pub total_quota: Option<TotalQuota>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TotalQuota {
    pub limit: String,
    pub remaining: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
    pub scope: String,
    pub detail: UsageDetail,
    pub limits: Vec<UsageLimit>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UsageDetail {
    pub limit: String,
    pub remaining: String,
    #[serde(rename = "resetTime")]
    pub reset_time: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UsageLimit {
    pub window: LimitWindow,
    pub detail: UsageDetail,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LimitWindow {
    pub duration: i32,
    #[serde(rename = "timeUnit")]
    pub time_unit: String,
}
