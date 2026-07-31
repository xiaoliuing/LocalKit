## Purpose

为桌面端用户提供一个可审计、可预览、可备份的 Coding Agent 会话清理能力，用于安全管理 Claude Code、Codex、OpenCode 等工具在本机保存的会话数据。

## Requirements

### Requirement: Provider 检测与状态展示
系统 SHALL 检测 Claude Code、Codex、OpenCode 的本地会话数据来源，并展示每个 Provider 的可用状态、数据目录、支持的删除方式和风险说明。

#### Scenario: 检测到支持的 Provider
- **WHEN** 用户打开 Agent 会话清理工具
- **THEN** 系统展示检测到的 Provider、数据目录、会话数量、可用删除能力和最近扫描时间

#### Scenario: Provider 未安装或数据目录不存在
- **WHEN** 某个 Provider 的 CLI 或数据目录不可用
- **THEN** 系统展示该 Provider 为不可用状态，并说明无法扫描或删除的原因

### Requirement: 会话扫描与列表展示
系统 SHALL 按 Provider 列出可识别的本机会话，并为每个会话展示足够信息帮助用户判断是否删除。

#### Scenario: 展示可识别会话
- **WHEN** 扫描完成且存在可识别会话
- **THEN** 系统展示会话所属 Provider、会话标识、标题或项目路径、更新时间、占用大小、状态标签和删除支持等级

#### Scenario: 会话信息不完整
- **WHEN** 会话缺少标题、项目路径或更新时间等元数据
- **THEN** 系统仍 SHALL 展示可用标识，并标记缺失字段而不是阻止整个 Provider 的扫描结果

### Requirement: 删除前 dry-run 计划
系统 SHALL 在执行硬删除前生成 dry-run 删除计划，列出将删除、修改、跳过或需要人工处理的项目。

#### Scenario: 用户选择会话并生成删除计划
- **WHEN** 用户选择一个或多个会话并请求删除
- **THEN** 系统展示 dry-run 计划，包括目标会话、目标路径或记录、预计释放空间、保护项、跳过项和风险提示

#### Scenario: 删除计划包含高风险目标
- **WHEN** 删除计划涉及索引、数据库或非官方删除路径
- **THEN** 系统 SHALL 标记为高风险，并要求用户明确确认后才能继续

### Requirement: 删除模式与备份
系统 SHALL 在删除计划确认后提供“备份并硬删除”和“永久删除”两种操作，并清晰区分是否会创建备份。

#### Scenario: 备份并硬删除
- **WHEN** 用户确认删除计划并选择备份并硬删除
- **THEN** 系统 SHALL 先创建备份，备份成功后执行硬删除，并在结果中展示备份位置

#### Scenario: 备份失败
- **WHEN** 用户选择备份并硬删除且系统无法创建完整备份
- **THEN** 系统 SHALL 取消删除，保留原始会话数据，并展示失败原因

#### Scenario: 永久删除
- **WHEN** 用户确认删除计划并选择永久删除
- **THEN** 系统 SHALL 跳过备份并直接执行删除，同时在结果和日志中记录该操作未创建备份

### Requirement: Provider 专属删除策略
系统 SHALL 根据 Provider 使用对应的安全删除策略，并优先使用官方命令或稳定接口。

#### Scenario: 删除 OpenCode 会话
- **WHEN** 用户删除 OpenCode 会话且 OpenCode CLI 可用
- **THEN** 系统 SHALL 优先调用官方会话删除能力，并展示 CLI 删除结果

#### Scenario: 删除 Claude Code 会话
- **WHEN** 用户删除 Claude Code 会话
- **THEN** 系统 SHALL 优先提供项目级官方清理能力，并明确说明该操作的影响范围

#### Scenario: 删除 Codex 会话
- **WHEN** 用户删除 Codex 会话
- **THEN** 系统 SHALL 避免粗暴删除整个 Codex 数据目录，并在删除计划中显式处理会话文件、归档会话、索引或数据库一致性风险

### Requirement: 受保护数据不得删除
系统 MUST NOT 删除认证、配置、偏好设置、模型配置、MCP 配置或其他非会话数据。

#### Scenario: 删除目标包含受保护路径
- **WHEN** dry-run 计划发现目标路径属于认证或配置数据
- **THEN** 系统 SHALL 将该目标标记为受保护并跳过删除

#### Scenario: 用户请求清空整个 Provider 数据目录
- **WHEN** 用户试图删除 Provider 的根数据目录
- **THEN** 系统 SHALL 拒绝执行，并提示只能删除已识别的会话数据

### Requirement: 删除结果与审计日志
系统 SHALL 展示删除结果并记录审计日志，便于用户追踪发生了什么。

#### Scenario: 删除完成
- **WHEN** 删除操作结束
- **THEN** 系统展示每个会话的成功、失败或跳过状态，并记录 Provider、会话标识、删除模式、备份位置、删除目标数量和错误原因

#### Scenario: 部分删除失败
- **WHEN** 批量删除中的部分会话失败
- **THEN** 系统 SHALL 保留其他会话的结果，展示失败原因，并不得把批量操作显示为完全成功
