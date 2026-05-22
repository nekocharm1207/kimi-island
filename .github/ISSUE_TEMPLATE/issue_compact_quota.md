---
title: "[Bug] Compact 模式显示周用量而非五小时额度"
labels: ["bug", "data-model"]
---

## 问题描述

Compact 模式（顶部悬浮条）的进度条显示的是**周用量**（约7天周期），而非更敏感的**五小时额度**。由于五小时额度通常比周用量消耗更快，用户无法从 Compact 视图快速获知短周期额度的紧张程度。

## 当前行为

Compact 显示：`usage_ratio`（周用量，如 67%）

## 预期行为

Compact 显示：`rate_limit_details.rpm.current / rate_limit_details.rpm.limit`（五小时额度，如 22%）

## 根因分析

`CompactIsland.tsx` 直接使用了 `data.usage_ratio`，该字段映射自 `coding_usage.detail`（7天周期额度）。而五小时额度的数据在 `rate_limit_details.rpm` 中。

## 修复方案

- [ ] `CompactIsland.tsx` 中 ratio 改为 `rpm.current / rpm.limit`
- [ ] Compact 的 warning level（脉冲动画）也应基于五小时额度比例计算

## 相关代码

```tsx
// CompactIsland.tsx
const ratio = data?.usage_ratio ?? 0; // ← 应改为 rpm 比例
```

## 备注

此改动后，Compact 将成为"短周期敏感指标"的快捷视图，展开面板仍保留完整的周用量 + 五小时额度两张卡片。
