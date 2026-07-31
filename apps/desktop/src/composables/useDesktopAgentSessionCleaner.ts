import { computed, shallowRef } from 'vue'
import {
  createAgentSessionDeletePlan,
  executeAgentSessionDeletePlan,
  openAgentSessionBackupsDirectory,
  scanAgentSessions,
  type AgentSessionDeletePlan,
  type AgentSessionDeleteResult,
  type AgentSessionEntry,
  type AgentSessionProviderId,
  type AgentSessionScanPayload,
} from '@/api/agentSessions'

type SelectionKey = `${AgentSessionProviderId}::${string}`

const emptyScan: AgentSessionScanPayload = {
  providers: [],
  sessions: [],
  scannedAt: '',
}

export function useDesktopAgentSessionCleaner() {
  const scanPayload = shallowRef<AgentSessionScanPayload>(emptyScan)
  const selectedProviderId = shallowRef<AgentSessionProviderId | 'all'>('all')
  const selectedKeys = shallowRef<SelectionKey[]>([])
  const deletePlan = shallowRef<AgentSessionDeletePlan | null>(null)
  const deleteResult = shallowRef<AgentSessionDeleteResult | null>(null)
  const isScanning = shallowRef(false)
  const isPlanning = shallowRef(false)
  const isDeleting = shallowRef(false)
  const feedbackMessage = shallowRef('')
  const errorMessage = shallowRef('')

  const providers = computed(() => scanPayload.value.providers)
  const sessions = computed(() => scanPayload.value.sessions)
  const filteredSessions = computed(() => {
    if (selectedProviderId.value === 'all') {
      return sessions.value
    }

    return sessions.value.filter((session) => session.providerId === selectedProviderId.value)
  })
  const selectedVisibleSessions = computed(() =>
    filteredSessions.value.filter((session) => selectedKeys.value.includes(createSelectionKey(session))),
  )
  const selectedCount = computed(() => selectedVisibleSessions.value.length)
  const hasHighRiskSelection = computed(() =>
    selectedVisibleSessions.value.some((session) => session.riskLevel === 'high'),
  )

  async function scan(options: { preserveDeleteResult?: boolean } = {}) {
    isScanning.value = true
    errorMessage.value = ''
    feedbackMessage.value = ''
    deletePlan.value = null
    if (!options.preserveDeleteResult) {
      deleteResult.value = null
    }

    try {
      const payload = await scanAgentSessions()
      scanPayload.value = payload
      selectedKeys.value = selectedKeys.value.filter((key) =>
        payload.sessions.some((session) => createSelectionKey(session) === key),
      )
      feedbackMessage.value = `扫描完成：${payload.sessions.length} 个会话`
    } catch (error) {
      errorMessage.value = error instanceof Error ? error.message : '扫描 Agent 会话失败'
      scanPayload.value = emptyScan
      selectedKeys.value = []
    } finally {
      isScanning.value = false
    }
  }

  function setProviderFilter(providerId: AgentSessionProviderId | 'all') {
    selectedProviderId.value = providerId
    deletePlan.value = null
  }

  function toggleSession(session: AgentSessionEntry) {
    const key = createSelectionKey(session)
    selectedKeys.value = selectedKeys.value.includes(key)
      ? selectedKeys.value.filter((item) => item !== key)
      : [...selectedKeys.value, key]
    deletePlan.value = null
    deleteResult.value = null
  }

  function selectVisibleSessions() {
    const nextKeys = new Set(selectedKeys.value)
    for (const session of filteredSessions.value) {
      nextKeys.add(createSelectionKey(session))
    }
    selectedKeys.value = [...nextKeys]
    deletePlan.value = null
  }

  function setSessionsSelected(sessions: AgentSessionEntry[], selected: boolean) {
    const nextKeys = new Set(selectedKeys.value)
    for (const session of sessions) {
      const key = createSelectionKey(session)
      if (selected) {
        nextKeys.add(key)
      } else {
        nextKeys.delete(key)
      }
    }
    selectedKeys.value = [...nextKeys]
    deletePlan.value = null
    deleteResult.value = null
  }

  function clearSelection() {
    selectedKeys.value = []
    deletePlan.value = null
    deleteResult.value = null
  }

  async function createPlan() {
    const sessionsForPlan = selectedVisibleSessions.value
    if (sessionsForPlan.length === 0) {
      feedbackMessage.value = '请先选择要清理的会话'
      return
    }

    isPlanning.value = true
    errorMessage.value = ''
    feedbackMessage.value = ''
    deleteResult.value = null

    try {
      deletePlan.value = await createAgentSessionDeletePlan({
        selections: sessionsForPlan.map((session) => ({
          providerId: session.providerId,
          sessionId: session.id,
        })),
      })
    } catch (error) {
      errorMessage.value = error instanceof Error ? error.message : '生成删除计划失败'
      deletePlan.value = null
    } finally {
      isPlanning.value = false
    }
  }

  async function executePlan(confirmHighRisk: boolean, options: { skipBackup?: boolean } = {}) {
    if (!deletePlan.value) {
      feedbackMessage.value = '请先预览删除计划'
      return
    }

    isDeleting.value = true
    errorMessage.value = ''
    feedbackMessage.value = ''

    try {
      deleteResult.value = await executeAgentSessionDeletePlan(deletePlan.value, confirmHighRisk, options)
      feedbackMessage.value = `删除完成：成功 ${deleteResult.value.deletedCount}，跳过 ${deleteResult.value.skippedCount}，失败 ${deleteResult.value.failedCount}`
      deletePlan.value = null
      selectedKeys.value = []
      await scan({ preserveDeleteResult: true })
    } catch (error) {
      errorMessage.value = error instanceof Error ? error.message : '执行删除失败'
    } finally {
      isDeleting.value = false
    }
  }

  async function openBackupsDirectory() {
    try {
      const opened = await openAgentSessionBackupsDirectory()
      feedbackMessage.value = opened ? '已打开备份目录' : '未能打开备份目录'
    } catch (error) {
      errorMessage.value = error instanceof Error ? error.message : '打开备份目录失败'
    }
  }

  function isSessionSelected(session: AgentSessionEntry) {
    return selectedKeys.value.includes(createSelectionKey(session))
  }

  return {
    deletePlan: computed(() => deletePlan.value),
    deleteResult: computed(() => deleteResult.value),
    errorMessage: computed(() => errorMessage.value),
    feedbackMessage: computed(() => feedbackMessage.value),
    filteredSessions,
    hasHighRiskSelection,
    isDeleting: computed(() => isDeleting.value),
    isPlanning: computed(() => isPlanning.value),
    isScanning: computed(() => isScanning.value),
    providers,
    scanPayload: computed(() => scanPayload.value),
    selectedCount,
    selectedProviderId: computed(() => selectedProviderId.value),
    sessions,
    clearSelection,
    createPlan,
    executePlan,
    isSessionSelected,
    openBackupsDirectory,
    scan,
    selectVisibleSessions,
    setProviderFilter,
    setSessionsSelected,
    toggleSession,
  }
}

function createSelectionKey(session: AgentSessionEntry): SelectionKey {
  return `${session.providerId}::${session.id}`
}
