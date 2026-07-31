import { invoke } from '@tauri-apps/api/core'

export type AgentSessionProviderId = 'claude-code' | 'codex' | 'opencode'

export type AgentSessionProviderStatus = 'ready' | 'unavailable' | 'limited' | 'error'

export type AgentSessionDeletionSupport =
  | 'official'
  | 'file'
  | 'high-risk'
  | 'read-only'
  | 'unsupported'

export type AgentSessionRiskLevel = 'low' | 'medium' | 'high'

export type AgentSessionProvider = {
  id: AgentSessionProviderId
  name: string
  status: AgentSessionProviderStatus
  dataDir: string
  cliPath: string | null
  sessionCount: number
  deletionSupport: AgentSessionDeletionSupport
  message: string
  riskNote: string
  scannedAt: string
}

export type AgentSessionEntry = {
  id: string
  providerId: AgentSessionProviderId
  title: string
  projectPath: string
  updatedAt: string | null
  sizeBytes: number
  status: string
  deletionSupport: AgentSessionDeletionSupport
  riskLevel: AgentSessionRiskLevel
  metadata: string[]
}

export type AgentSessionScanPayload = {
  providers: AgentSessionProvider[]
  sessions: AgentSessionEntry[]
  scannedAt: string
}

export type AgentSessionDeleteSelection = {
  providerId: AgentSessionProviderId
  sessionId: string
}

export type AgentSessionDeletePlanRequest = {
  selections: AgentSessionDeleteSelection[]
}

export type AgentSessionDeletePlanItem = {
  id: string
  providerId: AgentSessionProviderId
  sessionId: string
  label: string
  action: 'delete-file' | 'delete-directory' | 'delete-opencode-session' | 'skip' | 'manual'
  target: string
  sizeBytes: number
  riskLevel: AgentSessionRiskLevel
  protected: boolean
  message: string
}

export type AgentSessionDeletePlan = {
  id: string
  createdAt: string
  items: AgentSessionDeletePlanItem[]
  totalSizeBytes: number
  backupRequired: boolean
  highRisk: boolean
  summary: string
}

export type AgentSessionDeleteResultItem = {
  planItemId: string
  providerId: AgentSessionProviderId
  sessionId: string
  status: 'deleted' | 'skipped' | 'failed'
  target: string
  message: string
}

export type AgentSessionDeleteResult = {
  planId: string
  backupPath: string | null
  deletedCount: number
  skippedCount: number
  failedCount: number
  releasedBytes: number
  items: AgentSessionDeleteResultItem[]
  completedAt: string
}

export async function scanAgentSessions(): Promise<AgentSessionScanPayload> {
  if (isTauriRuntime()) {
    return invoke<AgentSessionScanPayload>('scan_agent_sessions')
  }

  const scannedAt = new Date().toISOString()
  return {
    providers: [
      {
        id: 'claude-code',
        name: 'Claude Code',
        status: 'limited',
        dataDir: '~/.claude/projects',
        cliPath: null,
        sessionCount: 1,
        deletionSupport: 'file',
        message: '浏览器预览模式使用模拟数据',
        riskNote: '桌面端运行时才会扫描真实目录',
        scannedAt,
      },
      {
        id: 'codex',
        name: 'Codex',
        status: 'limited',
        dataDir: '~/.codex',
        cliPath: null,
        sessionCount: 1,
        deletionSupport: 'high-risk',
        message: '浏览器预览模式使用模拟数据',
        riskNote: 'Codex 会话涉及索引和数据库，删除前必须确认',
        scannedAt,
      },
      {
        id: 'opencode',
        name: 'OpenCode',
        status: 'limited',
        dataDir: '~/.local/share/opencode',
        cliPath: null,
        sessionCount: 1,
        deletionSupport: 'official',
        message: '浏览器预览模式使用模拟数据',
        riskNote: '桌面端优先调用 OpenCode 官方 CLI',
        scannedAt,
      },
    ],
    sessions: [
      {
        id: 'mock-claude-session',
        providerId: 'claude-code',
        title: 'Claude Code 示例项目会话',
        projectPath: '~/Projects/example',
        updatedAt: scannedAt,
        sizeBytes: 42112,
        status: '可删除文件',
        deletionSupport: 'file',
        riskLevel: 'medium',
        metadata: ['模拟数据', 'JSONL transcript'],
      },
      {
        id: 'mock-codex-session',
        providerId: 'codex',
        title: 'Codex 示例会话',
        projectPath: '~/Projects/example',
        updatedAt: scannedAt,
        sizeBytes: 65536,
        status: '高风险',
        deletionSupport: 'high-risk',
        riskLevel: 'high',
        metadata: ['模拟数据', '需要处理索引一致性'],
      },
      {
        id: 'mock-opencode-session',
        providerId: 'opencode',
        title: 'OpenCode 示例会话',
        projectPath: '~/Projects/example',
        updatedAt: scannedAt,
        sizeBytes: 20480,
        status: '官方 CLI',
        deletionSupport: 'official',
        riskLevel: 'low',
        metadata: ['模拟数据', 'opencode session delete'],
      },
    ],
    scannedAt,
  }
}

export async function createAgentSessionDeletePlan(
  request: AgentSessionDeletePlanRequest,
): Promise<AgentSessionDeletePlan> {
  if (isTauriRuntime()) {
    return invoke<AgentSessionDeletePlan>('create_agent_session_delete_plan', { request })
  }

  const createdAt = new Date().toISOString()
  const items = request.selections.map<AgentSessionDeletePlanItem>((selection, index) => ({
    id: `mock-plan-item-${index + 1}`,
    providerId: selection.providerId,
    sessionId: selection.sessionId,
    label: selection.sessionId,
    action: selection.providerId === 'opencode' ? 'delete-opencode-session' : 'delete-file',
    target: `mock://${selection.providerId}/${selection.sessionId}`,
    sizeBytes: selection.providerId === 'codex' ? 65536 : 20480,
    riskLevel: selection.providerId === 'codex' ? 'high' : 'medium',
    protected: false,
    message: '浏览器预览模式不会删除真实文件',
  }))

  return {
    id: `mock-plan-${Date.now()}`,
    createdAt,
    items,
    totalSizeBytes: items.reduce((total, item) => total + item.sizeBytes, 0),
    backupRequired: true,
    highRisk: items.some((item) => item.riskLevel === 'high'),
    summary: `将处理 ${items.length} 个模拟目标，预计释放 ${formatHumanBytes(items.reduce((total, item) => total + item.sizeBytes, 0))}`,
  }
}

export async function executeAgentSessionDeletePlan(
  plan: AgentSessionDeletePlan,
  confirmHighRisk: boolean,
  options: { skipBackup?: boolean } = {},
): Promise<AgentSessionDeleteResult> {
  if (isTauriRuntime()) {
    return invoke<AgentSessionDeleteResult>('execute_agent_session_delete_plan', {
      plan,
      confirmHighRisk,
      skipBackup: options.skipBackup ?? false,
    })
  }

  return {
    planId: plan.id,
    backupPath: options.skipBackup ? null : `mock://agent-session-backups/${plan.id}`,
    deletedCount: 0,
    skippedCount: plan.items.length,
    failedCount: 0,
    releasedBytes: 0,
    items: plan.items.map((item) => ({
      planItemId: item.id,
      providerId: item.providerId,
      sessionId: item.sessionId,
      status: 'skipped',
      target: item.target,
      message: confirmHighRisk ? '浏览器预览模式已跳过' : '浏览器预览模式已跳过',
    })),
    completedAt: new Date().toISOString(),
  }
}

export async function openAgentSessionBackupsDirectory(): Promise<boolean> {
  if (!isTauriRuntime()) {
    return false
  }

  return invoke<boolean>('open_agent_session_backups_directory')
}

function isTauriRuntime() {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

function formatHumanBytes(bytes: number) {
  if (bytes <= 0) {
    return '0 B'
  }

  const units = ['B', 'KB', 'MB', 'GB']
  let size = bytes
  let unitIndex = 0
  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024
    unitIndex += 1
  }

  return `${size.toFixed(unitIndex === 0 ? 0 : 1)} ${units[unitIndex]}`
}
