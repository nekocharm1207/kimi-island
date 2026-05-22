# Changelog

所有版本的变更记录遵循 [Keep a Changelog](https://keepachangelog.com/) 格式。

---

## [0.1.0] - 2026-05-22

### Added
- 灵动岛窗口：支持紧凑（胶囊）和展开两种模式
- 实时额度展示：已用 / 总额、使用率百分比、预警颜色
- 频限详情卡片：RPM / TPM / RPD 实时数据
- 有效期显示：订阅到期时间、剩余天数
- 手动 Token 配置：支持粘贴 Kimi 浏览器 `access_token`
- 自动轮询刷新：根据额度紧张程度智能调整间隔（15s / 30s / 60s）
- 智能预警：额度不足时自动变色 + 可配置阈值
- Win32 原生置顶：`HWND_TOPMOST` 确保始终悬浮
- 胶囊 hit region：`SetWindowRgn` 实现圆角点击区域
- 系统托盘图标：右键可退出应用
- 生产构建：支持 `.msi` 安装包和 `.exe` 便携版

### Fixed
- 移除所有 Mock 数据，接入 Kimi 真实 API
- 修复 CLI OAuth token 误用导致的 401 错误
- 修复 Expanded 面板底部按钮区域截断
- 修复窗口关闭后无法重新打开的问题

### Changed
- Token 获取方式从「WebView 自动提取」改为「手动粘贴」（更稳定可靠）
- 额度 API scope 从 `"kimi-code"` 修正为 `"FEATURE_CODING"`
- `device_id` 缺失时降级为 `"unknown_device"`，不再阻断启动

### Technical
- 接入 Kimi Connect Protocol：`GetSubscription` + `GetUsages`
- 解析 JWT claims 提取 `user_id` 和 `session_id`
- 添加 `totalQuota` / `balances` / `capabilities` 等 API 响应结构
- 本地缓存机制：24h TTL 避免频繁请求

---

## [0.0.1] - 2026-05-21

### Added
- 项目初始化：Tauri v2 + React + TypeScript 脚手架
- 文档体系：`PRD.md`、`TDD.md`、`EchoIsland-Research.md`
- Win32 窗口管理层：`SetWindowRgn`、`SetWindowPos` 封装
- 前端原型：CompactIsland + ExpandedIsland 组件
