## Why

Coding Agent 会话数据分散在 Claude Code、Codex、OpenCode 等工具的本地目录和数据库中，手工清理容易误删认证配置、索引文件或仍在使用的会话。Docs Atlas 已有桌面端工具中心，适合提供一个面向本机开发者的 Agent 会话清理入口，让用户能先看清会话占用，再以可回滚的方式执行硬删除。

## What Changes

- 在桌面端工具中心新增“Agent 会话清理”工具入口。
- 支持扫描主流 Coding Agent 的本地会话来源：Claude Code、Codex、OpenCode。
- 展示每个 Provider 的检测状态、数据目录、会话列表、更新时间、大小、删除能力等级和风险提示。
- 提供 dry-run 删除计划，执行前明确列出将删除或修改的文件/记录。
- 删除操作提供两个入口：默认“备份并硬删除”会先创建备份；“永久删除”跳过备份并直接执行不可恢复删除。
- 对 OpenCode 优先调用官方 CLI 删除会话；对 Claude Code 优先支持项目级官方清理能力；对 Codex 采用保守策略，第一版必须识别索引/数据库风险并避免粗暴删除整个数据目录。
- 明确保护认证、配置、偏好设置、模型配置、MCP 配置等非会话数据。
- 删除操作写入 Docs Atlas 日志，并在 UI 中展示删除模式、成功、失败和跳过原因。

## Capabilities

### New Capabilities

- `agent-session-cleaner`: 管理 Claude Code、Codex、OpenCode 的本地 Coding Agent 会话扫描、dry-run、备份和硬删除行为。

### Modified Capabilities

- 无。

## Impact

- 影响桌面端工具中心、桌面端工具视图编排、前端 Tauri API 封装、Rust 侧系统命令、日志记录和本地文件/数据库访问逻辑。
- 可能新增或扩展共享类型，用于描述 Provider、会话、删除计划、备份结果和删除结果。
- 不影响 Web 端静态文档站。
- 不引入 Pinia；复杂状态应放在桌面端 composable 中。
