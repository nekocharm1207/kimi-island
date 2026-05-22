---
title: "[Bug] Dot 模式不可拖拽"
labels: ["bug", "ui"]
---

## 问题描述

Dot 模式（点击 Compact 的 K 图标进入的 48x48 圆点状态）无法通过鼠标拖拽移动窗口位置。

## 复现步骤

1. 启动 kimi-island
2. 点击 Compact 模式左侧的 K 图标，进入 Dot 模式
3. 尝试按住 Dot 拖动到屏幕其他位置
4. **预期**：窗口跟随鼠标移动
5. **实际**：窗口无响应，无法拖动

## 根因分析

`DotIsland.tsx` 中虽然调用了 `getCurrentWebviewWindow().startDragging()`，但缺少两个必要条件：

1. **Capability 窗口标签不匹配**：`src-tauri/capabilities/default.json` 中 `"windows": ["main"]` 应为 `["island"]`（与 `tauri.conf.json` 中的窗口 label 一致）
2. **缺少拖拽权限**：capability 中未添加 `core:window:allow-start-dragging`

## 修复方案

- [ ] 修正 `default.json`：`"windows": ["island"]`
- [ ] 添加权限：`"core:window:allow-start-dragging"`

## 备注

Compact 模式不拖拽是**设计意图**（只有 Dot 模式需要拖拽）。
