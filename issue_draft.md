## Issue 草稿（请复制到 GitHub 创建）

---

### Issue 1: Expanded 模式窗口高度过大，下方存在大量空白

**现象**  
展开 Island 后，窗口申请了约 630×630（物理像素），但实际内容仅占上半部分，下方大片空白区域遮挡桌面内容。从截图可见面板下方直接露出桌面图标和背景。

**根因**  
`src-tauri/src/window_manager.rs` 中 `EXPANDED_HEIGHT` 固定为 `460.0` 逻辑像素。在高 DPI（如 1.5×）屏幕上，物理高度达到 690px，而 `ExpandedIsland.tsx` 的实际内容高度仅约 340px，导致下方出现约 150+ 逻辑像素的空白。

**修复方向**  
- 方案A：根据实际内容动态计算并设置窗口高度  
- 方案B：将 `EXPANDED_HEIGHT` 缩小至与内容匹配（约 340-360）

---

### Issue 2: 额度单位硬编码为 "tokens"，与 API 返回不符

**现象**  
用量卡片显示"已用 1 / 总额 100 tokens"，但用户实际额度单位并非 tokens。

**根因**  
`src-tauri/src/kime_service.rs:225` 硬编码：
```rust
unit: "tokens".to_string()
```
而 API `GetUsagesResponse` / `Balance` 中已经返回了真实 unit（如 `"5h"`），代码未读取。

**修复方向**  
从 API 响应的 `Balance.unit` 或 `Usage` 结构中提取真实单位，不再硬编码。

---

### Issue 3: RPM / TPM / RPD 时间窗口映射错误

**现象**  
- `rpd`（日限）实际显示的是**周**度额度数据  
- `rpm` 实际对应的是 **5 小时**窗口的限流，不是每分钟  
- `tpm` 始终为 `0/0`，意义不明，数据丢失

**根因分析**  

1. **RPD 被错误映射为周限制**  
   `kime_service.rs:181-188` 将 `coding_usage.detail`（主 detail）直接映射为 `rpd`，注释写"可能是 RPD (daily)"。但实际 API 中该 `detail` 对应的是**周度**配额（`reset_time` 可验证），导致周限被错误标记为 "RPD"。

2. **5 小时窗口限制丢失，错误落入 RPM**  
   代码在 `limits` 数组循环中：
   - `TIME_UNIT_MINUTE` → 映射到 `rpm`  
   - `TIME_UNIT_HOUR` → 空处理，数据丢弃  
   
   但 API 返回的 `TIME_UNIT_MINUTE` 实际上可能 duration > 1（如 5 分钟），或者用户的 API 中根本不存在 `TIME_UNIT_MINUTE`，真正的 5 小时限制应该来自 `TIME_UNIT_HOUR` with `duration=5`。由于 `TIME_UNIT_HOUR` 分支为空，5 小时限制未被捕获。

3. **TPM 无数据来源**  
   代码中 `tpm` 变量初始化为 0，没有任何逻辑向其赋值，因此永远显示 `0/0`。

**修复方向**  
- 检查 `coding_usage.detail.reset_time` 来判断主 detail 的真实周期（周 / 日 / 其他），正确映射到 `rpd` 或新增字段  
- 正确处理 `TIME_UNIT_HOUR`（特别是 `duration=5`）并映射到合适的字段  
- 明确 `rpm`、`tpm`、`rpd` 各自的真实业务含义，与 API 字段一一对应

---
