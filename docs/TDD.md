# Kimi 额度灵动岛 —— 技术设计文档 (TDD)

> 版本：v1.0  
> 日期：2026-05-22  
> 对应 PRD：v1.0  
> 面向文档开发流程 Phase 2 产出

---

## 1. 技术选型

### 1.1 技术栈总览

| 层级 | 技术 | 版本 | 选型理由 |
|------|------|------|----------|
| 桌面框架 | Tauri | v2 | 轻量（<50MB）、使用系统 Webview、Rust 后端安全高效 |
| 前端框架 | React | v19 | 组件化、生态成熟、配合 Tauri 官方推荐 |
| 前端语言 | TypeScript | v5.6 | 类型安全，与 Rust 类型可对齐 |
| 样式方案 | TailwindCSS | v4 | 原子化 CSS，快速实现动态岛 UI |
| 构建工具 | Vite | v6 | 极速 HMR，Tauri 官方集成 |
| 后端语言 | Rust | edition 2021 | Tauri 原生语言，可调用 Win32 API |
| Win32 绑定 | windows-sys | v0.59 | 直接调用底层 API（SetWindowRgn、SetWindowPos 等） |

### 1.2 不采用的方案及原因

| 方案 | 不采用原因 |
|------|-----------|
| Electron | 体积大（~200MB），内存占用高，与灵动岛常驻轻量理念相悖 |
| Python + PyQt | 打包体积大，Win32 API 调用繁琐，无类型安全 |
| 纯 Win32 + Rust 自绘 | 开发效率低，EchoIsland 已证明 Webview 路径足够 |
| WPF / WinUI 3 | 与 Tauri 生态不兼容，增加技术栈复杂度 |

---

## 2. 系统架构

### 2.1 架构图

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Windows Desktop                              │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  Tauri App (Rust + Webview)                                  │   │
│  │  ┌─────────────────────────┐  ┌───────────────────────────┐ │   │
│  │  │   Frontend (React/TS)   │  │   Backend (Rust)          │ │   │
│  │  │  ┌─────────────────┐   │  │  ┌─────────────────────┐  │ │   │
│  │  │  │ App.tsx         │   │  │  │ main.rs             │  │ │   │
│  │  │  │ ┌─────────────┐ │   │  │  │ ├─ commands.rs      │  │ │   │
│  │  │  │ │CompactIsland│ │   │  │  │ ├─ window_manager.rs│  │ │   │
│  │  │  │ └─────────────┘ │   │◀─┼──┼─▶├─ kime_service.rs  │  │ │   │
│  │  │  │ ┌─────────────┐ │   │  │  │ ├─ config.rs        │  │ │   │
│  │  │  │ │ExpandedIsland│ │   │  │  │ ├─ tray.rs          │  │ │   │
│  │  │  │ └─────────────┘ │   │  │  │ └─ autostart.rs     │  │ │   │
│  │  │  │ ┌─────────────┐ │   │  │  └─────────────────────┘  │ │   │
│  │  │  │ │ useKimeData │ │   │  │         ↑                 │ │   │
│  │  │  │ │  (轮询hook)  │ │   │  │         │ invoke          │ │   │
│  │  │  │ └─────────────┘ │   │  │         │                 │ │   │
│  │  │  └─────────────────┘   │  └───────────────────────────┘ │   │
│  │  │         ↑ IPC (JSON)    │                                │   │
│  │  └─────────┼───────────────┘                                │   │
│  │            │                                                │   │
│  │  ┌─────────┴───────────────────────────────────────────┐   │   │
│  │  │  Win32 API Layer (windows-sys)                        │   │   │
│  │  │  ├─ SetWindowRgn   (圆角胶囊裁剪)                      │   │   │
│  │  │  ├─ SetWindowPos   (HWND_TOPMOST 置顶)                │   │   │
│  │  │  ├─ CreateRectRgn  (区域构造)                         │   │   │
│  │  │  └─ CombineRgn     (区域合并)                         │   │   │
│  │  └───────────────────────────────────────────────────────┘   │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                              │                                       │
│  ┌───────────────────────────┴───────────────────────────────────┐  │
│  │  External Dependencies                                         │  │
│  │  ├─ kime CLI (%AppData%\kime\config.json / kime check --json) │  │
│  │  └─ Kimi Web API (https://www.kimi.com)                        │  │
│  └────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.2 模块职责

#### Frontend (React)

| 模块 | 职责 |
|------|------|
| `App.tsx` | 根组件，管理 compact/expanded 状态切换 |
| `CompactIsland` | 紧凑胶囊视图：Logo + 进度条 + 百分比 |
| `ExpandedIsland` | 展开详情视图：头部 + 额度卡片 + 频限 + 操作栏 |
| `UsageBar` | 可复用进度条组件，支持颜色渐变和动画 |
| `WarningPulse` | 预警边框脉冲动画（CSS keyframes） |
| `useKimeData` | 自定义 Hook：调用 Tauri command 获取数据，管理轮询 |
| `useAnimation` | 自定义 Hook：管理展开/收缩动画状态 |

#### Backend (Rust)

| 模块 | 职责 |
|------|------|
| `main.rs` | Tauri 应用入口，注册 commands 和 events |
| `commands.rs` | Tauri Commands：供前端调用的 API |
| `window_manager.rs` | 窗口管理：尺寸切换、圆角区域、置顶、定位 |
| `kime_service.rs` | kime CLI 调用、数据解析、缓存管理 |
| `config.rs` | 配置读写（JSON 序列化/反序列化） |
| `tray.rs` | 系统托盘：图标、菜单、事件处理 |
| `autostart.rs` | 开机自启动注册表操作 |

---

## 3. 详细设计

### 3.1 窗口层设计（核心）

#### 3.1.1 窗口创建配置

`tauri.conf.json` 窗口段：

```json
{
  "app": {
    "windows": [
      {
        "label": "island",
        "title": "kimi-island",
        "width": 320,
        "height": 48,
        "decorations": false,
        "transparent": true,
        "alwaysOnTop": true,
        "skipTaskbar": true,
        "resizable": false,
        "shadow": false,
        "focus": false,
        "visible": true,
        "center": false,
        "x": null,
        "y": null
      }
    ]
  }
}
```

#### 3.1.2 窗口状态机

```rust
enum IslandMode {
    Compact,    // 320×48，圆角胶囊，顶部居中
    Expanded,   // 420×280，矩形，顶部居中
    Hidden,     // 隐藏（托盘操作）
}

struct IslandWindowState {
    mode: IslandMode,
    scale_factor: f64,
    monitor_rect: MonitorRect,
}
```

#### 3.1.3 圆角胶囊实现

直接复用 EchoIsland 的 `SetWindowRgn` 方案，简化版：

```rust
// window_manager.rs

use windows_sys::Win32::Graphics::Gdi::{
    CombineRgn, CreateRectRgn, DeleteObject, RGN_OR, SetWindowRgn,
};
use windows_sys::Win32::Foundation::HWND;

const SHOULDER_RADIUS: f64 = 6.0;  // 圆角半径（逻辑像素）

/// 为紧凑模式应用圆角胶囊命中区域
pub fn apply_capsule_hit_region(hwnd: HWND, width: f64, height: f64, scale: f64) {
    let left = 0;
    let top = 0;
    let right = logical_to_physical(width, scale);
    let bottom = logical_to_physical(height, scale);
    let r = logical_to_physical(SHOULDER_RADIUS, scale).max(1);

    // 主体矩形（减去左右肩部）
    let main = unsafe { CreateRectRgn(left + r, top, right - r, bottom) };

    // 逐行构建左右圆弧肩部
    for row in 0..r {
        let dy = r - row;  // 从顶部向下数
        // 圆的方程: x = sqrt(r² - dy²)
        let threshold = ((r * r - dy * dy) as f64).sqrt().round() as i32;
        let y_top = top + row;
        let y_bottom = top + row + 1;

        // 左肩
        let left_rgn = unsafe {
            CreateRectRgn(left + threshold, y_top, left + r, y_bottom)
        };
        if left_rgn != std::ptr::null_mut() {
            unsafe { CombineRgn(main, main, left_rgn, RGN_OR); DeleteObject(left_rgn); }
        }

        // 右肩
        let right_rgn = unsafe {
            CreateRectRgn(right - r, y_top, right - threshold, y_bottom)
        };
        if right_rgn != std::ptr::null_mut() {
            unsafe { CombineRgn(main, main, right_rgn, RGN_OR); DeleteObject(right_rgn); }
        }
    }

    unsafe { SetWindowRgn(hwnd, main, 1) };
}

/// 展开模式：恢复为完整矩形区域
pub fn apply_rectangular_hit_region(hwnd: HWND, width: f64, height: f64, scale: f64) {
    let right = logical_to_physical(width, scale);
    let bottom = logical_to_physical(height, scale);
    let rgn = unsafe { CreateRectRgn(0, 0, right, bottom) };
    unsafe { SetWindowRgn(hwnd, rgn, 1) };
}

fn logical_to_physical(value: f64, scale: f64) -> i32 {
    (value * scale).round() as i32
}
```

#### 3.1.4 模式切换流程

```
Frontend: 用户点击 / Hover 超时 / 预警触发
    │
    ▼ invoke("set_island_mode", { mode: "expanded" })
Backend: commands::set_island_mode(mode)
    │
    ├─ 1. window.set_size(expanded_width, expanded_height)
    ├─ 2. position_island_window()  // 重新顶部居中
    ├─ 3. apply_rectangular_hit_region()  // 矩形命中区
    └─ 4. emit("island:mode_changed", { mode: "expanded" })
    │
Frontend: 监听 island:mode_changed，执行 CSS 过渡动画
```

### 3.2 数据层设计

#### 3.2.1 核心类型定义

```rust
// Rust 侧 (kime_service.rs)

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KimeUsageData {
    pub current_plan: String,
    pub validity: ValidityInfo,
    pub weekly_usage: UsageInfo,
    pub usage_ratio: f64,  // 0.0 ~ 1.0
    pub rate_limit_details: RateLimitDetails,
    pub model_permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidityInfo {
    pub current_end_time: DateTime<Utc>,
    pub days_remaining: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageInfo {
    pub used: u64,
    pub total: u64,
    pub unit: String,  // "tokens" | "requests"
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
```

```typescript
// Frontend 侧 (types.ts) —— 与 Rust 结构完全对齐

export interface KimeUsageData {
  current_plan: string;
  validity: ValidityInfo;
  weekly_usage: UsageInfo;
  usage_ratio: number;
  rate_limit_details: RateLimitDetails;
  model_permissions: string[];
}

export interface ValidityInfo {
  current_end_time: string;  // ISO 8601
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
```

#### 3.2.2 数据来源：kime CLI

**方案 A（推荐）：调用 kime CLI**

```rust
use std::process::Command;

pub async fn fetch_from_kime_cli() -> Result<KimeUsageData, KimeError> {
    let output = Command::new("kime")
        .args(["check", "--json"])
        .output()
        .map_err(|e| KimeError::CliNotFound(e.to_string()))?;
    
    if !output.status.success() {
        return Err(KimeError::CliFailed(
            String::from_utf8_lossy(&output.stderr).to_string()
        ));
    }
    
    let json = String::from_utf8_lossy(&output.stdout);
    let data: KimeUsageData = serde_json::from_str(&json)
        .map_err(|e| KimeError::ParseError(e.to_string()))?;
    
    Ok(data)
}
```

**方案 B（备选）：直接调用 Kimi HTTP API**

若用户希望不依赖 kime CLI，可直接用提供的逆向接口：

```rust
// POST https://www.kimi.com/api/usage 或类似端点
// Headers: Authorization: Bearer {token}, x-msh-device-id, x-msh-session-id, x-traffic-id
```

> **决策**：Phase 1 先实现方案 A（调用 kime CLI），简单可靠。方案 B 作为后续增强。

#### 3.2.3 缓存策略

```rust
// cache.rs

use std::path::PathBuf;
use serde_json;

const CACHE_FILE: &str = "cache.json";
const CACHE_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    version: u32,
    fetched_at: DateTime<Utc>,
    data: KimeUsageData,
}

pub fn read_cache(app_dirs: &AppDirs) -> Option<KimeUsageData> {
    let path = cache_path(app_dirs);
    let content = std::fs::read_to_string(path).ok()?;
    let entry: CacheEntry = serde_json::from_str(&content).ok()?;
    
    if entry.version != CACHE_VERSION {
        return None;  // 缓存版本不匹配，丢弃
    }
    
    // 缓存有效期：以 subscription.currentEndTime 为准，或最长 24 小时
    let ttl = chrono::Duration::hours(24);
    if Utc::now() - entry.fetched_at > ttl {
        return None;
    }
    
    Some(entry.data)
}

pub fn write_cache(app_dirs: &AppDirs, data: &KimeUsageData) -> Result<(), CacheError> {
    let path = cache_path(app_dirs);
    let entry = CacheEntry {
        version: CACHE_VERSION,
        fetched_at: Utc::now(),
        data: data.clone(),
    };
    let json = serde_json::to_string_pretty(&entry)?;
    std::fs::write(path, json)?;
    Ok(())
}
```

### 3.3 轮询策略设计

```rust
// kime_service.rs

pub struct PollScheduler {
    interval_normal: Duration,
    interval_warning: Duration,
    interval_critical: Duration,
}

impl PollScheduler {
    pub fn next_interval(&self, usage_ratio: f64, config: &AppConfig) -> Duration {
        if usage_ratio >= config.red_threshold as f64 / 100.0 {
            self.interval_critical
        } else if usage_ratio >= config.yellow_threshold as f64 / 100.0 {
            self.interval_warning
        } else {
            self.interval_normal
        }
    }
}
```

**轮询流程**：

```
启动
  │
  ▼
读取缓存 ──有缓存？──▶ 前端展示缓存数据
  │ No                │
  ▼                   ▼
调用 kime CLI    后台启动轮询定时器
  │              （tokio::time::interval）
  ▼                   │
解析数据 ◄────────────┘
  │
  ▼
写入缓存 + 推送到前端
  │
  ▼
根据 usage_ratio 计算下一次间隔
  │
  ▼
等待…… ──▶ 定时器触发 ──▶ 重复上述流程
```

### 3.4 配置系统设计

```rust
// config.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_preferred_display")]
    pub preferred_display: String,
    
    #[serde(default = "default_compact_width")]
    pub compact_width: u32,  // 240~480
    
    #[serde(default = "default_yellow_threshold")]
    pub yellow_threshold: u32,  // 1~100
    
    #[serde(default = "default_red_threshold")]
    pub red_threshold: u32,  // 1~100
    
    #[serde(default = "default_poll_interval_normal")]
    pub poll_interval_normal: u64,  // seconds
    
    #[serde(default = "default_poll_interval_warning")]
    pub poll_interval_warning: u64,
    
    #[serde(default = "default_poll_interval_critical")]
    pub poll_interval_critical: u64,
    
    #[serde(default = "default_auto_collapse_delay")]
    pub auto_collapse_delay: u64,  // milliseconds, 0 = disabled
    
    #[serde(default)]
    pub auto_expand_on_warning: bool,
    
    #[serde(default = "default_theme")]
    pub theme: Theme,
    
    #[serde(default)]
    pub sound_on_warning: bool,
    
    #[serde(default)]
    pub autostart: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    System,
    Dark,
    Light,
}
```

**配置校验规则**：

| 字段 | 最小值 | 最大值 | 非法处理 |
|------|--------|--------|----------|
| `compact_width` | 240 | 480 | clamp 到边界 |
| `yellow_threshold` | 5 | 50 | clamp 到边界 |
| `red_threshold` | 1 | 20 | clamp 到边界 |
| `poll_interval_*` | 5 | 3600 | clamp 到边界 |
| `yellow_threshold` vs `red_threshold` | — | — | 若 yellow <= red，交换两者 |

### 3.5 系统托盘设计

```rust
// tray.rs

use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::menu::{Menu, MenuItem};

pub fn create_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    let toggle_i = MenuItem::with_id(app, "toggle", "显示/隐藏", true, None::<&str>)?;
    let refresh_i = MenuItem::with_id(app, "refresh", "立即刷新", true, None::<&str>)?;
    let settings_i = MenuItem::with_id(app, "settings", "设置", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    
    let menu = Menu::with_items(app, &[&toggle_i, &refresh_i, &settings_i, &quit_i])?;
    
    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "toggle" => { /* 切换显示/隐藏 */ }
            "refresh" => { /* 触发数据刷新 */ }
            "settings" => { /* 打开设置窗口 */ }
            "quit" => { app.exit(0); }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| match event {
            TrayIconEvent::Click { .. } => { /* 左键单击切换显示 */ }
            _ => {}
        })
        .build(app)?;
    
    Ok(())
}
```

**托盘图标动态变色**：

通过 `set_icon` API 在运行时切换不同颜色的图标资源：
- `icon-normal.png` —— 白色
- `icon-warning.png` —— 黄色
- `icon-critical.png` —— 红色

### 3.6 开机自启动

```rust
// autostart.rs

#[cfg(windows)]
use windows_registry::CURRENT_USER;

const REG_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const REG_KEY: &str = "kimi-island";

#[cfg(windows)]
pub fn set_autostart(enabled: bool) -> Result<(), Box<dyn std::error::Error>> {
    let exe_path = std::env::current_exe()?;
    let key = CURRENT_USER.create(REG_PATH)?;
    
    if enabled {
        key.set_string(REG_KEY, &exe_path.to_string_lossy())?;
    } else {
        let _ = key.remove_value(REG_KEY);
    }
    Ok(())
}
```

### 3.7 前端状态管理

使用 React Context + useReducer 管理全局状态：

```typescript
// state.ts

interface AppState {
  mode: 'compact' | 'expanded' | 'hidden';
  data: KimeUsageData | null;
  loading: boolean;
  error: string | null;
  lastUpdated: Date | null;
  warningLevel: 'none' | 'yellow' | 'red';
}

type AppAction =
  | { type: 'SET_MODE'; payload: AppState['mode'] }
  | { type: 'SET_DATA'; payload: KimeUsageData }
  | { type: 'SET_LOADING'; payload: boolean }
  | { type: 'SET_ERROR'; payload: string | null }
  | { type: 'REFRESH' };
```

---

## 4. 接口契约

### 4.1 Tauri Commands（Frontend → Backend）

| Command | 输入 | 输出 | 说明 |
|---------|------|------|------|
| `get_usage_data` | `{ force?: boolean }` | `Result<KimeUsageData, string>` | 获取额度数据，force=true 绕过缓存 |
| `get_config` | — | `AppConfig` | 获取当前配置 |
| `set_config` | `AppConfig` | `Result<(), string>` | 更新配置并持久化 |
| `set_island_mode` | `{ mode: string }` | `Result<(), string>` | 切换窗口模式 |
| `get_island_mode` | — | `string` | 获取当前模式 |
| `open_kimi_website` | — | `Result<(), string>` | 打开浏览器访问 kimi.com |
| `set_autostart` | `{ enabled: boolean }` | `Result<(), string>` | 设置开机自启动 |
| `quit_app` | — | — | 退出应用 |

### 4.2 Tauri Events（Backend → Frontend）

| Event | Payload | 说明 |
|-------|---------|------|
| `usage:updated` | `KimeUsageData` | 数据更新推送 |
| `usage:error` | `{ message: string }` | 数据获取失败 |
| `island:mode_changed` | `{ mode: string }` | 窗口模式变更通知 |
| `config:changed` | `AppConfig` | 配置变更通知 |

---

## 5. 目录结构

```
kimi-island/
├── Cargo.toml                    # Rust workspace
├── package.json                  # Node.js 依赖
├── vite.config.ts                # Vite 配置
├── tsconfig.json                 # TypeScript 配置
├── tailwind.config.ts            # Tailwind 配置
├── docs/
│   ├── PRD.md                    # 产品需求文档
│   ├── TDD.md                    # 技术设计文档（本文档）
│   └── EchoIsland-Research.md    # EchoIsland 研究笔记
├── src/                          # Rust 后端源码
│   ├── main.rs
│   ├── lib.rs
│   ├── commands.rs               # Tauri Commands
│   ├── window_manager.rs         # 窗口管理
│   ├── kime_service.rs           # kime CLI 调用
│   ├── cache.rs                  # 缓存管理
│   ├── config.rs                 # 配置管理
│   ├── tray.rs                   # 系统托盘
│   ├── autostart.rs              # 开机自启动
│   └── types.rs                  # 共享类型定义
├── src-ui/                       # React 前端源码
│   ├── main.tsx                  # 入口
│   ├── App.tsx                   # 根组件
│   ├── index.css                 # 全局样式
│   ├── types.ts                  # TypeScript 类型
│   ├── state.tsx                 # React Context + Reducer
│   ├── hooks/
│   │   ├── useKimeData.ts        # 数据轮询 Hook
│   │   └── useAnimation.ts       # 动画 Hook
│   └── components/
│       ├── CompactIsland.tsx
│       ├── ExpandedIsland.tsx
│       ├── UsageBar.tsx
│       ├── WarningPulse.tsx
│       ├── RateLimitCard.tsx
│       ├── SettingsModal.tsx
│       └── LoadingSpinner.tsx
├── public/
│   └── icons/                    # 图标资源
│       ├── icon-normal.png
│       ├── icon-warning.png
│       ├── icon-critical.png
│       └── logo.svg
└── tauri.conf.json               # Tauri 应用配置
```

---

## 6. 构建与打包

### 6.1 开发命令

```bash
# 安装依赖
npm install

# 开发模式（前端热重载 + Rust 重编译）
npm run tauri dev

# 仅前端开发
npm run dev

# Rust 测试
cargo test
```

### 6.2 生产构建

```bash
# 构建生产版本
npm run tauri build

# 输出：
# src-tauri/target/release/kimi-island.exe      (便携版)
# src-tauri/target/release/bundle/msi/*.msi       (安装包)
# src-tauri/target/release/bundle/nsis/*.exe      (安装程序)
```

---

## 7. 测试策略

| 测试类型 | 范围 | 工具 | 优先级 |
|----------|------|------|--------|
| 单元测试 | Rust 模块（解析、配置、缓存） | `cargo test` | P0 |
| 集成测试 | Tauri Command 端到端 | `@tauri-apps/api` + 测试脚本 | P1 |
| 手动测试 | UI 交互、动画流畅度、DPI 适配 | 人工 | P0 |
| 兼容性测试 | Win10/Win11, 多 DPI, 多显示器 | 虚拟机 + 实体机 | P1 |
| 长期稳定性 | 7×24 运行，内存泄漏检测 | 持续运行 + Task Manager | P2 |

---

## 8. 风险与应对

| 风险 | 可能性 | 影响 | 应对措施 |
|------|--------|------|----------|
| kime CLI 接口变更 | 中 | 高 | 封装解析层，提供 fallback 文本显示模式 |
| Tauri v2 API 不兼容 | 低 | 高 | 锁定依赖版本，升级前做回归测试 |
| 高 DPI 下模糊/错位 | 中 | 中 | 严格 logical→physical 转换，多 DPI 实测 |
| 杀毒软件误报 | 低 | 中 | 使用 Tauri 官方签名流程，避免敏感 API 调用 |
| Webview2 未安装 | 低 | 高 | Tauri 自动引导安装，或提供 WebView2 Runtime 捆绑 |

---

## 9. 变更记录

| 版本 | 日期 | 变更内容 | 作者 |
|------|------|----------|------|
| v1.0 | 2026-05-22 | 初稿 | kimi-code-cli |
