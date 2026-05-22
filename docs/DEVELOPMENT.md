# 开发规范

> Kimi Island 的开发流程、提交规范和发布 checklist。

---

## 目录

- [日常更新流程](#日常更新流程)
- [Git 提交规范](#git-提交规范)
- [文档维护规则](#文档维护规则)
- [版本发布流程](#版本发布流程)
- [常见问题](#常见问题)

---

## 日常更新流程

```
改代码 → 本地测试 → git add → git commit → git push
```

### 详细步骤

```bash
cd C:\Users\15151\Desktop\kimi-island

# 1. 修改代码（用 VS Code / Cursor / 记事本都可以）

# 2. 查看改了哪些文件
git status

# 3. 添加到暂存区（准备提交）
git add .

# 4. 提交（必须写有意义的提交信息）
git commit -m "feat: 新增 xxx 功能"

# 5. 推送到 GitHub
git push origin main
```

> ⚠️ **注意**：本地修改不会自动同步到 GitHub，必须执行 `git push`。

---

## Git 提交规范

提交信息格式：

```
<type>: <简短描述>

<可选：详细说明>
```

### Type 类型

| 类型 | 用途 | 示例 |
|------|------|------|
| `feat` | 新增功能 | `feat: 支持多显示器选择` |
| `fix` | 修复 Bug | `fix: 修复首次启动窗口偏移` |
| `docs` | 文档变更 | `docs: 更新 Token 获取说明` |
| `style` | 代码格式（不影响功能） | `style: 统一缩进` |
| `refactor` | 重构（不新增功能也不修复 Bug） | `refactor: 提取公共组件` |
| `test` | 测试相关 | `test: 添加额度解析单元测试` |
| `chore` | 构建/工具/依赖变更 | `chore: 升级 tauri 到 v2.1` |

### 示例

```bash
# 新增功能
git commit -m "feat: 支持开机自启动"

# 修复 Bug
git commit -m "fix: 额度为 0% 时进度条异常"

# 文档更新
git commit -m "docs: 补充多显示器配置说明"

# 多行详细说明
git commit -m "feat: 支持快捷键展开/收起" -m "- Ctrl+Shift+K 切换模式
- 可在设置中自定义快捷键
- 修复快捷键和系统冲突"
```

---

## 文档维护规则

| 场景 | 必须更新的文档 | 可选更新的文档 |
|------|---------------|---------------|
| 新增功能 | `CHANGELOG.md` | `README.md` 功能列表、`ROADMAP.md` 勾选 |
| 修复 Bug | `CHANGELOG.md` | `ROADMAP.md` 已知问题移除 |
| 改界面/交互 | `CHANGELOG.md` | `README.md` 截图 |
| API 变更 | `CHANGELOG.md` + 代码注释 | — |
| 架构调整 | — | `TDD.md` |
| 发新版本 | `CHANGELOG.md` 加版本号 | — |

### 文件位置

```
kimi-island/
├── README.md              # 项目介绍（用户第一眼看到）
├── CHANGELOG.md           # 版本变更记录
├── docs/
│   ├── DEVELOPMENT.md     # 本文件：开发规范
│   ├── ROADMAP.md         # 路线图 + 需求池 + 已知问题
│   ├── PRD.md             # 产品需求文档
│   ├── TDD.md             # 技术设计文档
│   └── EchoIsland-Research.md  # 技术调研
```

---

## 版本发布流程

### 发小版本（v0.1.0 → v0.1.1）

```bash
# 1. 确保代码已提交并推送
git push origin main

# 2. 更新 CHANGELOG.md（加版本号）

# 3. 提交 CHANGELOG 更新
git add CHANGELOG.md
git commit -m "docs: update changelog for v0.1.1"
git push origin main

# 4. 打标签
git tag v0.1.1
git push origin v0.1.1

# 5. 构建安装包
npm run tauri build

# 6. GitHub → Releases → New Release → 选择 v0.1.1 标签 → 上传 .msi + .exe
```

### 发大版本（v0.1.x → v0.2.0）

```bash
# 同上，只是版本号规则不同
# 小版本：修复 Bug、小功能（v0.1.0 → v0.1.1）
# 大版本：新增功能模块、架构变更（v0.1.x → v0.2.0）
```

### Release 发布 Checklist

- [ ] `CHANGELOG.md` 已更新
- [ ] 所有代码已 commit 并 push
- [ ] Git 标签已打并推送 (`git push origin v0.x.x`)
- [ ] `cargo tauri build` 成功
- [ ] GitHub Release 页面上传了 `.msi` 和 `.exe`
- [ ] Release 说明复制了 CHANGELOG 对应版本内容
- [ ] `ROADMAP.md` 已勾选已完成项

---

## 常见问题

### Q: 改了文件但 `git status` 没显示？
A: 检查是否改对了文件夹。项目根目录是 `C:\Users\15151\Desktop\kimi-island`。

### Q: `git push` 提示 "Everything up-to-date" 但 GitHub 没更新？
A: 确认是否执行了 `git commit`。只有 commit 后的内容才会被 push。

### Q: 如何撤销上次提交？
A: 
```bash
# 撤销提交但保留文件修改
git reset --soft HEAD~1

# 撤销提交且丢弃修改（危险！）
git reset --hard HEAD~1
```

### Q: 如何查看历史提交？
A: `git log --oneline`

### Q: 多人协作时冲突怎么办？
A: 先 `git pull origin main` 拉取最新代码，解决冲突后再 push。
