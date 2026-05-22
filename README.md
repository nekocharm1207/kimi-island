# 🏝️ Kimi Island

Kimi 订阅额度灵动岛 —— 一个悬浮在屏幕顶部的 Windows 桌面小工具，实时展示你的 Kimi Code 额度、使用率和频限状态。

> 灵感来自 macOS Dynamic Island，专为 Windows 打造。

---

## 📸 预览

| 紧凑模式 | 展开模式 |
|---------|---------|
| 悬浮胶囊显示额度百分比 | 点击展开查看详情 |

---

## ✨ 功能特性

- **🟢 实时额度监控** — 自动拉取 Kimi 订阅数据，显示已用 / 总额、使用率
- **⚡ 频限详情** — RPM / TPM / RPD 实时展示，避免触发限流
- **🔔 智能预警** — 额度低于阈值时自动变色（黄 → 红），支持自动展开提醒
- **🖱️ 一键展开** — 紧凑胶囊点击展开，鼠标离开自动收缩
- **🔄 自动刷新** — 根据额度紧张程度智能调整轮询间隔（15s ~ 60s）
- **📌 置顶悬浮** — 始终悬浮在最顶层，不影响其他窗口操作
- **🎨 透明毛玻璃** — 黑色半透明背景 + 圆角胶囊设计

---

## 📥 安装

### 方式一：直接运行（推荐）

1. 从 [Releases](../../releases) 下载 `kimi-island_0.1.0_x64-setup.exe`
2. 双击安装，按向导完成
3. 从开始菜单或桌面快捷方式启动

### 方式二：免安装便携版

1. 从 [Releases](../../releases) 下载 `kimi-island.exe`
2. 直接双击运行

> **系统要求**: Windows 10 1809+ / Windows 11

---

## 🔑 配置 Token

Kimi Island 需要你的 **Kimi 浏览器 Token**（不是 API Key）。

### 获取步骤

1. 打开浏览器，访问 [kimi.com/code/console](https://kimi.com/code/console)
2. 确保已登录你的 Kimi 账号
3. 按 `F12` 打开开发者工具
4. 切换到 **Application** → **Local Storage** → `https://kimi.com`
5. 找到 `access_token`，复制其值（通常以 `eyJhbG...` 开头）

### 输入 Token

1. 启动 Kimi Island，点击顶部胶囊展开面板
2. 如果显示「未配置 Kimi 浏览器 token」，在输入框粘贴复制的 token
3. 点击「保存」，数据自动刷新

> Token 仅保存在本地配置文件 `%APPDATA%\kimi-island\config.json` 中，不会上传到任何服务器。

---

## 🛠️ 技术栈

- **框架**: [Tauri v2](https://tauri.app/) + React + TypeScript
- **后端**: Rust + reqwest + Win32 API
- **UI**: Tailwind CSS + Lucide Icons
- **API**: Kimi Connect Protocol (内部 Web API)

---

## 📄 开源协议

MIT License
