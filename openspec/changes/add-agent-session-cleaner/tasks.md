## 1. 类型与接口

- [x] 1.1 定义 Agent Provider、会话条目、删除能力、dry-run 计划、备份结果和删除结果的 TypeScript 类型。
- [x] 1.2 新增桌面端 `api/agentSessions.ts`，封装扫描、生成删除计划、执行删除、打开备份目录等 Tauri 调用。
- [x] 1.3 在 Rust 侧定义与前端类型对齐的 serde payload，覆盖 Provider 状态、会话元数据、计划项目和执行结果。

## 2. Rust Provider Adapter

- [x] 2.1 实现统一 Provider adapter 调度，支持检测、扫描、计划、备份和删除的公共流程。
- [x] 2.2 实现 OpenCode adapter，优先使用 `opencode session list`、`opencode session delete` 和 `opencode db path` 能力。
- [x] 2.3 实现 Claude Code adapter，检测 `~/.claude/projects` 和官方项目级清理能力，并明确返回影响范围。
- [x] 2.4 实现 Codex adapter，检测 `CODEX_HOME` 或 `~/.codex` 下的会话、归档会话、索引和数据库风险，禁止删除整个根目录。
- [x] 2.5 实现受保护路径规则，确保认证、配置、偏好、模型和 MCP 相关文件永不进入可删除目标。
- [x] 2.6 实现备份写入 Docs Atlas 应用数据目录的专用子目录，并在备份失败时阻止删除。
- [x] 2.7 为删除操作记录 Docs Atlas 日志，包含 Provider、会话标识、备份位置、目标数量和错误原因。

## 3. 桌面端状态与 UI

- [x] 3.1 新增 `useDesktopAgentSessionCleaner` composable，管理扫描状态、Provider 过滤、会话选择、dry-run 计划和删除结果。
- [x] 3.2 在工具中心新增“Agent 会话清理”卡片，并在 `DesktopAppShell` 中接入新工具视图。
- [x] 3.3 新增 Agent 会话清理视图，展示 Provider 状态、数据目录、风险说明、会话列表和空状态。
- [x] 3.4 实现会话选择、批量操作、删除计划预览、高风险确认、备份并硬删除、永久删除和备份位置展示。
- [x] 3.5 实现删除结果面板，区分成功、失败、跳过和部分成功，并提供重新扫描入口。
- [x] 3.6 确保新 UI 使用现有桌面端主题变量，兼容浅色、暗色和自定义主题色。

## 4. 验证与安全检查

- [x] 4.1 为 Rust 侧路径保护、Provider 检测、dry-run 计划和备份失败路径添加聚焦测试或可重复验证用例。
- [x] 4.2 使用模拟数据验证前端会话列表、dry-run 计划、高风险确认和删除结果状态。
- [x] 4.3 运行桌面端类型检查：`cd apps/desktop && node ./node_modules/vue-tsc/bin/vue-tsc.js --noEmit`。
- [x] 4.4 运行桌面端 Web shell 构建：`cd apps/desktop && node ./node_modules/vite/bin/vite.js build`。
- [ ] 4.5 手动验证未安装 Provider、空会话、CLI 缺失、备份失败和部分删除失败场景。
