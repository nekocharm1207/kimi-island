# EchoIsland Windows 灵动岛实现原理研究笔记

> 源码来源：https://github.com/FunplayAI/EchoIsland  
> 研究日期：2026-05-22  
> 研究目标：提取可复用的 Windows 灵动岛窗口技术，用于构建 Kimi 额度浮动面板

---

## 1. 项目架构总览

EchoIsland 采用 **Tauri v2 + Rust** 构建，是一个 AI 编码会话聚合器。其 Windows 端灵动岛实现分为**两条技术路径**：

| 路径 | 技术栈 | 复杂度 | 适用场景 |
|------|--------|--------|----------|
| **Webview 路径** | Tauri WebviewWindow + Win32 API 注入 | 中等 | 快速开发、UI 复杂、需频繁迭代 |
| **原生渲染路径** | Direct2D/DirectWrite + 分层窗口自绘 | 极高 | 极致性能、动画复杂、内存敏感 |

> **结论**：对于 Kimi 额度灵动岛项目，采用 **Webview 路径** 即可满足需求（UI 不复杂，动画以 CSS 为主），原生渲染路径过重，仅作原理参考。

---

## 2. Webview 路径详解（推荐复用）

源码核心文件：`apps/desktop/src-tauri/src/island_window.rs`

### 2.1 窗口创建配置（Tauri 层）

Tauri 的 `tauri.conf.json` 中窗口配置：

```json
{
  "windows": [{
    "label": "main",
    "width": 420,
    "height": 80,
    "decorations": false,      // 去标题栏/边框
    "transparent": true,       // 允许透明背景
    "alwaysOnTop": true,       // 置顶（基础层）
    "skipTaskbar": true,       // 不显示在任务栏
    "resizable": false,        // 固定尺寸
    "shadow": false            // 去系统阴影
  }]
}
```

### 2.2 窗口行为控制（Rust 层）

```rust
// 应用叠加层窗口标志
fn apply_overlay_window_flags<R: tauri::Runtime>(window: &WebviewWindow<R>) -> tauri::Result<()> {
    window.set_decorations(false)?;      // 无装饰
    window.set_shadow(false)?;           // 无阴影
    window.set_resizable(false)?;        // 不可调整大小
    window.set_skip_taskbar(true)?;      // 隐藏任务栏图标
    refresh_overlay_topmost(window)?;    // 强制置顶
    Ok(())
}

// 透明背景
fn apply_transparent_background<R: tauri::Runtime>(window: &WebviewWindow<R>) -> tauri::Result<()> {
    window.set_background_color(Some(Color(0, 0, 0, 0)))  // ARGB 全透明
}
```

### 2.3 强制置顶（Win32 HWND_TOPMOST）

Tauri 的 `alwaysOnTop` 在某些场景下可能失效，EchoIsland 直接调用 Win32 API：

```rust
#[cfg(windows)]
fn apply_native_topmost<R: tauri::Runtime>(window: &WebviewWindow<R>) -> tauri::Result<()> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOOWNERZORDER, SWP_NOSIZE, SetWindowPos,
    };

    let hwnd = window.hwnd()?.0 as isize;
    let flags = SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOOWNERZORDER;
    let ok = unsafe { SetWindowPos(hwnd as _, HWND_TOPMOST, 0, 0, 0, 0, flags) };
    if ok == 0 {
        return Err(io::Error::last_os_error().into());
    }
    Ok(())
}
```

**关键点**：
- `HWND_TOPMOST` 确保窗口始终在所有非置顶窗口之上
- `SWP_NOACTIVATE` 避免窗口获得焦点（不打断用户当前操作）
- `SWP_NOMOVE | SWP_NOSIZE` 只改 Z-Order，不动位置尺寸

### 2.4 圆角胶囊形状 —— SetWindowRgn

这是灵动岛**最核心的 trick**。通过 `SetWindowRgn` 将矩形窗口裁剪为圆角胶囊形状：

```rust
#[cfg(windows)]
fn apply_island_hit_region<R: tauri::Runtime>(
    window: &WebviewWindow<R>,
    region: IslandRegion,
) -> tauri::Result<()> {
    use windows_sys::Win32::Graphics::Gdi::{
        CombineRgn, CreateRectRgn, DeleteObject, RGN_OR, SetWindowRgn,
    };

    let hwnd = window.hwnd()?.0 as isize;
    let scale = window.scale_factor()?;           // DPI 缩放因子
    let window_size = window.inner_size()?;
    let client_width = window_size.width as f64 / scale;
    let offset_x = ((client_width - region.hit_width) / 2.0).max(0.0);

    // 逻辑坐标 → 物理像素（DPI-aware）
    let left = logical_to_physical(offset_x, scale);
    let top = 0;
    let right = logical_to_physical(offset_x + region.hit_width, scale);
    let bottom = logical_to_physical(region.hit_height, scale);

    let hrgn = if region.compact_shoulders {
        // 圆角肩部：通过逐行计算圆弧阈值，拼接多个小矩形逼近曲线
        let shoulder = logical_to_physical(6.0, scale).max(1);
        let main = unsafe { CreateRectRgn(left + shoulder, top, right - shoulder, bottom) };
        
        for row in 0..shoulder {
            let dy = shoulder - row;
            let threshold = ((shoulder * shoulder - dy * dy) as f64).sqrt().round() as i32;
            let shoulder_bottom = top + row + 1;
            
            // 左侧圆弧
            let left_rgn = unsafe {
                CreateRectRgn(left + threshold, top + row, left + shoulder, shoulder_bottom)
            };
            if left_rgn != null_mut() {
                unsafe { CombineRgn(main, main, left_rgn, RGN_OR); DeleteObject(left_rgn); }
            }
            
            // 右侧圆弧
            let right_rgn = unsafe {
                CreateRectRgn(right - shoulder, top + row, right - threshold, shoulder_bottom)
            };
            if right_rgn != null_mut() {
                unsafe { CombineRgn(main, main, right_rgn, RGN_OR); DeleteObject(right_rgn); }
            }
        }
        main
    } else {
        // 展开模式：纯矩形区域
        unsafe { CreateRectRgn(left, top, right, bottom) }
    };

    unsafe { SetWindowRgn(hwnd as _, hrgn, 1) };  // 应用区域，触发重绘
    Ok(())
}
```

**算法原理**：
- 胶囊主体是一个矩形
- 左右肩部（shoulder）是圆角，通过 **圆的方程 `x² + y² = r²`** 逐行计算阈值
- 每一行用若干个小矩形拼接逼近圆弧
- `SetWindowRgn` 裁剪后，区域外的窗口部分完全透明且**鼠标穿透**

**DPI 缩放处理**：
```rust
fn logical_to_physical(value: f64, scale: f64) -> i32 {
    (value * scale).round() as i32
}
```
所有逻辑坐标（设计稿像素）在调用 Win32 API 前必须转换为物理像素。

### 2.5 窗口定位 —— 屏幕顶部居中

```rust
fn position_island_window<R: tauri::Runtime>(
    window: &WebviewWindow<R>,
    width: f64,
    expanded: bool,
) -> tauri::Result<()> {
    let monitors = window.available_monitors()?;
    let preferred_index = resolve_preferred_display_index(...);
    let monitor = monitors.into_iter().nth(preferred_index)
        .or_else(|| window.primary_monitor().ok().flatten())
        .or_else(|| window.current_monitor().ok().flatten());
    
    if let Some(monitor) = monitor {
        let monitor_size = monitor.size();
        let monitor_position = monitor.position();
        let scale = monitor.scale_factor();
        let monitor_x = monitor_position.x as f64 / scale;
        let monitor_y = monitor_position.y as f64 / scale;
        let monitor_width = monitor_size.width as f64 / scale;
        let x = monitor_x + ((monitor_width - width) / 2.0).max(0.0);  // 水平居中
        let y = monitor_y + if expanded { 8.0 } else { 0.0 };          // 顶部贴合
        window.set_position(LogicalPosition::new(x, y))?;
    }
    Ok(())
}
```

### 2.6 双模尺寸切换

| 模式 | 尺寸 | 命中区域 |
|------|------|----------|
| Compact | 420×80 | 胶囊形状（带圆角肩部） |
| Expanded | 784×560 | 矩形（全窗口可交互） |
| Bar Stage | 420×80 | 矩形（过渡状态） |
| Panel Stage | 420×动态高 | 矩形（面板状态） |

切换时同步执行：
1. `window.set_size(LogicalSize::new(width, height))`
2. `position_island_window(...)` 重新计算坐标
3. `apply_island_hit_region(...)` 重新设置命中区域

---

## 3. 原生渲染路径（原理参考）

源码目录：`apps/desktop/src-tauri/src/windows_native_panel/`

### 3.1 核心模块

| 文件 | 职责 |
|------|------|
| `layered_window.rs` | `WS_EX_LAYERED` 分层窗口的像素级 Alpha 合成 |
| `direct2d.rs` / `directwrite.rs` | D2D 设备上下文、DWrite 文本排版 |
| `d2d_painter.rs` | 绘制命令执行（圆角矩形、文字、图标） |
| `renderer.rs` | 渲染管线调度 |
| `platform_loop.rs` | Win32 消息泵（`GetMessage`/`DispatchMessage`） |
| `message_dispatch.rs` | 窗口消息分发（`WM_PAINT`, `WM_MOUSEMOVE` 等） |

### 3.2 分层窗口像素合成

```rust
// layered_window.rs
pub(super) struct WindowsLayeredAlphaBitmap {
    size: WindowsLayeredBitmapSize,
    pixels: Vec<u8>,  // BGRA 像素数据
}

impl WindowsLayeredAlphaBitmap {
    pub(super) fn new(size: WindowsLayeredBitmapSize) -> Self {
        Self { size, pixels: vec![0; size.byte_len()] }
    }
    
    fn clear_transparent(&mut self) {
        self.pixels.fill(0);  // 全透明
    }
}
```

Windows 的 `UpdateLayeredWindow` API 允许直接提交 BGRA 像素数组，实现**每像素独立透明度**。

### 3.3 为什么原生渲染更重

- 需要自研**布局系统**（计算每个元素的位置/大小）
- 需要自研**动画系统**（插值计算 + 脏区重绘）
- 需要自研**文本排版**（DWrite 的 TextLayout）
- 需要自研**图片解码**（WIC 或自定义解码器）
- 代码量：Webview 路径约 300 行，原生路径约 5000+ 行

---

## 4. 鼠标交互模型

### 4.1 消息映射

```rust
const WINDOWS_WM_MOUSEMOVE: u32 = 0x0200;   // 鼠标移动
const WINDOWS_WM_LBUTTONUP: u32 = 0x0202;   // 左键释放（作为 Click）
const WINDOWS_WM_MOUSELEAVE: u32 = 0x02A3;  // 鼠标离开窗口
const WINDOWS_WM_PAINT: u32 = 0x000F;       // 需要重绘
```

### 4.2 指针区域（Pointer Regions）

EchoIsland 将窗口划分为多个**可交互区域**：

```rust
enum NativePanelPointerRegionKind {
    CompactBar,      // 紧凑胶囊条
    Card,            // 卡片
    ActionButton,    // 操作按钮
    Dismiss,         // 关闭/忽略
}
```

每个区域定义一个 `PanelRect`，运行时判断鼠标坐标落在哪个区域，触发对应行为。

---

## 5. 可复用代码清单（直接搬运到 kimi-island）

### 5.1 必搬运：窗口置顶
```rust
// apply_native_topmost + SetWindowPos
```

### 5.2 必搬运：圆角胶囊命中区域
```rust
// apply_island_hit_region + CreateRectRgn + CombineRgn + SetWindowRgn
// 简化版：只需 Compact 模式的肩部圆角算法
```

### 5.3 必搬运：DPI 缩放工具
```rust
fn logical_to_physical(value: f64, scale: f64) -> i32 {
    (value * scale).round() as i32
}
```

### 5.4 必搬运：显示器定位
```rust
// position_island_window 的核心逻辑
```

### 5.5 必搬运：窗口标志组合
```rust
// apply_overlay_window_flags
// apply_transparent_background
```

---

## 6. 与 kimi-island 的差异点

| 维度 | EchoIsland | kimi-island |
|------|-----------|-------------|
| 渲染路径 | 双路径（Webview + 原生） | 仅 Webview 路径 |
| 动画复杂度 |  mascot 动画、卡片堆叠、光晕 | CSS 过渡 + 脉冲动画 |
| 交互模型 | 多区域点击（approval/question/dismiss） | 展开/收缩 + 刷新按钮 |
| 数据源 | 本地文件扫描 + TCP IPC | kime CLI 调用 |
| 状态管理 | 复杂状态机（session/approval/reminder） | 简单轮询 + 缓存 |

---

## 7. 风险与注意事项

1. **DPI 感知**：Windows 的 DPI 缩放会导致窗口模糊或尺寸错误，必须在所有 Win32 API 调用前将逻辑坐标转为物理像素。
2. **SetWindowRgn 性能**：频繁修改窗口区域有性能开销，只在 compact/expanded 切换时调用即可。
3. **Tauri v2 API 变更**：`window.hwnd()` 等 API 在 v2 中为 `window.hwnd()?`，注意错误处理。
4. **杀毒软件误报**：调用 `SetWindowRgn` 和 `SetWindowPos` 等 API 可能被部分杀软标记为可疑行为，需测试。
