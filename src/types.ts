export interface KimeUsageData {
  current_plan: string;
  validity: ValidityInfo;
  weekly_usage: UsageInfo;
  usage_ratio: number;
  rate_limit_details: RateLimitDetails;
  model_permissions: string[];
}

export interface ValidityInfo {
  current_end_time: string;
  days_remaining: number;
}

export interface UsageInfo {
  used: number;
  total: number;
  unit: string;
}

export interface RateLimitDetails {
  rpm: RateLimitItem;
  tpm: RateLimitItem;
  rpd: RateLimitItem;
}

export interface RateLimitItem {
  current: number;
  limit: number;
  remaining: number;
}

export type IslandMode = 'compact' | 'expanded' | 'hidden';
export type WarningLevel = 'none' | 'yellow' | 'red';

export interface AppConfig {
  preferred_display: string;
  compact_width: number;
  yellow_threshold: number;
  red_threshold: number;
  poll_interval_normal: number;
  poll_interval_warning: number;
  poll_interval_critical: number;
  auto_collapse_delay: number;
  auto_expand_on_warning: boolean;
  theme: 'system' | 'dark' | 'light';
  sound_on_warning: boolean;
  autostart: boolean;
}
