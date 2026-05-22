

## 背景

在修复额度显示错误（#4/#5/#6）的过程中，发现当前前后端数据模型的字段命名与实际 Kimi API 返回的数据含义存在多处错位。部分字段名具有误导性，部分 API 字段未被利用，还有一些字段属于 dead code。

本文档旨在全面列出所有相关字段，供后续统一梳理和重构。

---

## 一、前端契约字段（`KimeUsageData`）

| 字段 | 当前类型 | 当前含义 | 实际问题 | 建议 |
|------|---------|---------|---------|------|
| `weekly_usage` | `UsageInfo` | 顶部卡片显示的额度 | **名称误导**：实际映射的是 `coding_usage.detail`（约7天周期额度），不是字面意义的"周" | 🔧 **重命名**为 `period_quota` 或 `coding_quota` |
| `usage_ratio` | `f64` | 顶部进度条比例 | 依赖 `weekly_usage`，逻辑上没问题 | ✅ 保留 |
| `rate_limit_details.rpm` | `RateLimitItem` | "每分钟请求数" | **名称+含义双重误导**：实际映射 `TIME_UNIT_MINUTE` limit，但用户侧显示为"五小时额度" | 🔧 **重命名**为 `short_term_quota` 或映射到新的语义化字段 |
| `rate_limit_details.tpm` | `RateLimitItem` | "每小时/每分 token 数" | 映射 `TIME_UNIT_HOUR`，目前 API 返回零值，字段名完全不对 | 🔧 **重命名**为 `hourly_limit` 或根据实际用途调整 |
| `rate_limit_details.rpd` | `RateLimitItem` | "每日请求数" | 实际映射 `coding_usage.detail`（和 `weekly_usage` 同数据源），用户侧叫"本周用量" | 🔧 **重命名**为 `period_usage` 或合并到 `weekly_usage` |
| `model_permissions` | `Vec<String>` | 功能列表 | 从 `capabilities` 提取，没问题 | ✅ 保留 |
| `current_plan` | `String` | 套餐名称 | 从 `subscription.goods.title` 提取 | ✅ 保留 |
| `validity` | `ValidityInfo` | 有效期 | 从 `subscription.current_end_time` 计算 | ✅ 保留 |

### 核心矛盾

```
当前布局：
┌─────────────────────────────────────┐
│ 本周额度 (weekly_usage)  66%        │  ← 实际数据 = coding_usage.detail (7天)
├─────────────────────────────────────┤
│ 本周用量 (rpd)           66/100/34  │  ← 和上面同一个数据源，重复显示
│ 五小时额度 (rpm)         18/100/82  │  ← 实际数据 = TIME_UNIT_MINUTE limit
│ 小时频限 (tpm)           0/0/0      │  ← 实际数据 = TIME_UNIT_HOUR limit
└─────────────────────────────────────┘
```

**问题**：`weekly_usage` 和 `rpd` 是同一个数据（`coding_usage.detail`）的两个副本，前端同时显示造成冗余。

---

## 二、后端原始 API 字段（`GetUsagesResponse`）

| 字段 | 类型 | 当前用途 | 状态 | 建议 |
|------|------|---------|------|------|
| `usages[].scope` | `String` | 筛选 `FEATURE_CODING` | ✅ 使用中 | 保留 |
| `usages[].detail` | `UsageDetail` | 映射到 `weekly_usage` + `rpd` | ✅ 使用中 | 保留，考虑拆分 |
| `usages[].limits[]` | `Vec<UsageLimit>` | 映射 `rpm`/`tpm` | ✅ 使用中 | 保留 |
| `usages[].limits[].window.duration` | `i32` | **未使用** | ⚠️ dead code | 🔧 **利用**：决定实际周期（如 300 = 5小时） |
| `usages[].limits[].window.time_unit` | `String` | 路由到 rpm/tpm/rpd | ✅ 使用中 | 保留 |
| `usages[].limits[].detail` | `UsageDetail` | 提供 limit/remaining | ✅ 使用中 | 保留 |
| `total_quota` | `Option<TotalQuota>` | fallback 数据源 | ⚠️ 当前优先级已降低 | 🔧 **决定**：是否保留？数据显示 1/100，与控制台不符 |
| `total_quota.limit` | `String` | 总配额上限 | ⚠️ fallback | 同上 |
| `total_quota.remaining` | `String` | 总配额剩余 | ⚠️ fallback | 同上 |

### 关键发现

- `coding_usage.detail` 和 `total_quota` 是**两个不同的额度体系**：
  - `coding_usage.detail` = 控制台"本周用量"（66/100）
  - `total_quota` = 某种总配额（1/100），与控制台显示无关
- `limits[].window.duration` 可能解释为什么 `TIME_UNIT_MINUTE` 实际是"五小时额度"（duration = 300 分钟？）

---

## 三、后端原始 API 字段（`GetSubscriptionResponse`）—— Dead Code 清单

以下字段在 `types.rs` 中定义但 **没有任何业务逻辑使用**：

| 字段 | 所属结构体 | 建议 |
|------|-----------|------|
| `subscribed` | `GetSubscriptionResponse` | 🗑️ 删除 或 🔧 用于判断登录状态 |
| `subscription_id` | `Subscription` | 🗑️ 删除 |
| `current_start_time` | `Subscription` | 🗑️ 删除 或 🔧 显示已用天数 |
| `status` | `Subscription` | 🗑️ 删除 或 🔧 用于异常状态提示 |
| `active` | `Subscription` | 🗑️ 删除 或 🔧 用于判断订阅是否有效 |
| `id` | `Goods` | 🗑️ 删除 |
| `duration_days` | `Goods` | 🗑️ 删除 或 🔧 显示套餐周期 |
| `membership_level` | `Goods` | 🗑️ 删除 或 🔧 替代 `title` 作为 plan 名称 |
| `id` | `Balance` | 🗑️ 删除 |
| `feature` | `Balance` | 🗑️ 删除 |
| `balance_type` | `Balance` | 🗑️ 删除 |
| `unit` | `Balance` | 🗑️ 删除 |
| `expire_time` | `Balance` | 🗑️ 删除 或 🔧 显示余额过期时间 |
| `constraint` | `Capability` | 🗑️ 删除 |
| `parallelism` | `Constraint` | 🗑️ 删除 |
| `reset_time` | `UsageDetail` | 🗑️ 删除 或 🔧 显示"X小时后重置" |
| `duration` | `LimitWindow` | 🗑️ 删除 或 🔧 显示实际限制周期 |

---

## 四、配置不一致

| 位置 | 字段 | 值 | 问题 |
|------|------|-----|------|
| `types.rs` | `default_auto_collapse_delay` | 2000ms | 与 `state.tsx` 中的 800ms 默认值不一致 |
| `state.tsx` | `defaultConfig.auto_collapse_delay` | 800ms | 实际生效的是这个 |

---

## 五、重构建议（供讨论）

### 方案 A：最小改动（保留现有结构，仅重命名）

```rust
pub struct KimeUsageData {
    pub current_plan: String,
    pub validity: ValidityInfo,
    pub period_quota: UsageInfo,        // 原 weekly_usage
    pub usage_ratio: f64,
    pub rate_limit_details: RateLimitDetails,  // 内部字段重命名
    pub model_permissions: Vec<String>,
}

pub struct RateLimitDetails {
    pub short_term: RateLimitItem,      // 原 rpm（五小时额度）
    pub hourly: RateLimitItem,          // 原 tpm（小时频限）
    pub period_usage: RateLimitItem,    // 原 rpd（本周用量，与 period_quota 同源）
}
```

### 方案 B：合并冗余（推荐）

既然 `period_quota` 和 `period_usage`（rpd）是同一个数据，前端不需要同时显示两张卡片：

```rust
pub struct KimeUsageData {
    pub current_plan: String,
    pub validity: ValidityInfo,
    pub period_quota: UsageInfo,        // 7天周期额度
    pub usage_ratio: f64,
    pub short_term_quota: RateLimitItem, // 五小时额度（原 rpm）
    pub hourly_limit: RateLimitItem,     // 小时频限（原 tpm）
    pub model_permissions: Vec<String>,
    pub reset_time: String,              // 从 UsageDetail.reset_time 提取
}
```

前端对应两张卡片：
1. **周期额度**（7天）— 使用 `period_quota` + `usage_ratio`
2. **短周期额度**（五小时）— 使用 `short_term_quota`

### 方案 C：完整模型（面向未来）

支持多 scope（不止 FEATURE_CODING）：

```rust
pub struct KimeUsageData {
    pub current_plan: String,
    pub validity: ValidityInfo,
    pub scopes: Vec<ScopeUsage>,         // FEATURE_CODING, FEATURE_CHAT, ...
    pub model_permissions: Vec<String>,
}

pub struct ScopeUsage {
    pub scope: String,
    pub period_quota: UsageInfo,
    pub usage_ratio: f64,
    pub limits: Vec<RateLimitItem>,      // 各时间窗口的频限
}
```

---

## 六、待确认问题

1. `total_quota`（1/100）到底是什么？是否与控制台任何显示对应？如果没有任何对应，是否可以删除？
2. `limits[].window.duration` 的具体值是多少？能否验证 `TIME_UNIT_MINUTE` 的 duration 确实是 300（5小时）？
3. `UsageDetail.reset_time` 的格式和时区？能否用于显示"X小时后重置"？
4. `Balance` 和 `Capability` 数据是否有未来用途？还是可以直接从类型定义中移除？
5. 是否需要支持除 `FEATURE_CODING` 之外的其他 scope（如 CHAT、DOCUMENTS）？

---

## 七、验收标准

- [ ] 确定最终字段命名方案
- [ ] 删除或利用所有 dead code 字段
- [ ] 前后端类型定义同步
- [ ] 前端显示与实际 API 数据 100% 对应
- [ ] 测试覆盖更新后的字段映射
