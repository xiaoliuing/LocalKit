<script setup lang="ts">
  import { computed, onMounted, shallowRef } from "vue";
  import type {
    AgentSessionDeletePlanItem,
    AgentSessionEntry,
    AgentSessionProvider,
    AgentSessionProviderId,
  } from "@/api/agentSessions";
  import DesktopUiIcon from "@/components/ui/DesktopUiIcon.vue";
  import { useDesktopAgentSessionCleaner } from "@/composables/useDesktopAgentSessionCleaner";

  const emit = defineEmits<{
    backToTools: [];
  }>();

  const {
    clearSelection,
    createPlan,
    deletePlan,
    deleteResult,
    errorMessage,
    executePlan,
    feedbackMessage,
    filteredSessions,
    hasHighRiskSelection,
    isDeleting,
    isPlanning,
    isScanning,
    isSessionSelected,
    openBackupsDirectory,
    providers,
    scan,
    selectVisibleSessions,
    selectedCount,
    selectedProviderId,
    setProviderFilter,
    setSessionsSelected,
    toggleSession,
  } = useDesktopAgentSessionCleaner();

  const highRiskConfirmed = shallowRef(false);
  const expandedDirectoryIds = shallowRef<string[]>([]);
  const providerSessionCount = computed(() =>
    providers.value.reduce((total, provider) => total + provider.sessionCount, 0),
  );
  const canExecutePlan = computed(
    () => deletePlan.value && (!deletePlan.value.highRisk || highRiskConfirmed.value),
  );
  const providerNameById = computed(() =>
    providers.value.reduce(
      (names, provider) => {
        names[provider.id] = provider.name;
        return names;
      },
      {} as Record<AgentSessionProviderId, string>,
    ),
  );
  const deletePlanGroups = computed(() => {
    if (!deletePlan.value) {
      return [];
    }

    const groups = new Map<AgentSessionProviderId, AgentSessionDeletePlanItem[]>();
    for (const item of deletePlan.value.items) {
      groups.set(item.providerId, [...(groups.get(item.providerId) ?? []), item]);
    }

    return [...groups.entries()].map(([providerId, items]) => ({
      providerId,
      providerName: providerNameById.value[providerId] ?? providerId,
      items,
      actionableCount: items.filter((item) => !item.protected && item.action !== "skip" && item.action !== "manual").length,
      totalSizeBytes: items
        .filter((item) => !item.protected && item.action !== "skip" && item.action !== "manual")
        .reduce((total, item) => total + item.sizeBytes, 0),
    }));
  });
  const deletePlanSummary = computed(() => {
    if (!deletePlan.value) {
      return "";
    }

    const providerCount = deletePlanGroups.value.length;
    const actionableCount = deletePlanGroups.value.reduce((total, group) => total + group.actionableCount, 0);
    return `${providerCount} 个 Provider · ${actionableCount} 个可执行目标 · 预计释放 ${formatBytes(deletePlan.value.totalSizeBytes)}`;
  });
  const sessionDirectoryGroups = computed(() => {
    const groups = new Map<string, AgentSessionEntry[]>();
    for (const session of filteredSessions.value) {
      const directory = normalizeDirectoryKey(session.projectPath);
      groups.set(directory, [...(groups.get(directory) ?? []), session]);
    }

    return [...groups.entries()]
      .map(([directory, sessions]) => {
        const providerIds = [...new Set(sessions.map((session) => session.providerId))];
        const selectedInGroup = sessions.filter((session) => isSessionSelected(session)).length;
        const totalSizeBytes = sessions.reduce((total, session) => total + session.sizeBytes, 0);
        const latestUpdatedAt = sessions
          .map((session) => session.updatedAt)
          .filter((value): value is string => Boolean(value))
          .sort()
          .at(-1) ?? null;

        return {
          id: directory,
          directory,
          isExpanded: expandedDirectoryIds.value.includes(directory),
          sessions,
          providerNames: providerIds.map((providerId) => providerNameById.value[providerId] ?? providerId),
          selectedInGroup,
          totalSizeBytes,
          latestUpdatedAt,
          riskLevel: sessions.some((session) => session.riskLevel === "high") ? "high" : "medium",
        };
      })
      .sort((left, right) => left.directory.localeCompare(right.directory, "zh-CN"));
  });

  onMounted(() => {
    void scan();
  });

  function handleProviderFilter(providerId: AgentSessionProvider["id"] | "all") {
    setProviderFilter(providerId);
    highRiskConfirmed.value = false;
  }

  async function handleCreatePlan() {
    highRiskConfirmed.value = false;
    await createPlan();
  }

  async function handleBackupDeletePlan() {
    await executePlan(highRiskConfirmed.value, { skipBackup: false });
    highRiskConfirmed.value = false;
  }

  async function handlePermanentDeletePlan() {
    await executePlan(highRiskConfirmed.value, { skipBackup: true });
    highRiskConfirmed.value = false;
  }

  function handleDirectorySelection(group: { sessions: AgentSessionEntry[]; selectedInGroup: number }) {
    setSessionsSelected(group.sessions, group.selectedInGroup !== group.sessions.length);
  }

  function toggleDirectoryGroup(directoryId: string) {
    expandedDirectoryIds.value = expandedDirectoryIds.value.includes(directoryId)
      ? expandedDirectoryIds.value.filter((id) => id !== directoryId)
      : [...expandedDirectoryIds.value, directoryId];
  }

  function formatBytes(bytes: number) {
    if (bytes <= 0) {
      return "0 B";
    }

    const units = ["B", "KB", "MB", "GB"];
    let size = bytes;
    let unitIndex = 0;
    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex += 1;
    }

    return `${size.toFixed(unitIndex === 0 ? 0 : 1)} ${units[unitIndex]}`;
  }

  function formatTimestamp(value: string | null) {
    if (!value) {
      return "未知";
    }

    const numericValue = Number(value);
    const date = Number.isFinite(numericValue)
      ? new Date(numericValue * 1000)
      : new Date(value);
    if (Number.isNaN(date.getTime())) {
      return value;
    }

    return date.toLocaleString("zh-CN", {
      hour12: false,
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
    });
  }

  function providerStatusLabel(status: AgentSessionProvider["status"]) {
    const labels: Record<AgentSessionProvider["status"], string> = {
      ready: "可用",
      unavailable: "不可用",
      limited: "受限",
      error: "异常",
    };
    return labels[status];
  }

  function riskLabel(item: AgentSessionEntry | AgentSessionDeletePlanItem) {
    const labels = {
      low: "低风险",
      medium: "中风险",
      high: "高风险",
    };
    return labels[item.riskLevel];
  }

  function isArchivedSession(session: AgentSessionEntry) {
    return session.providerId === "codex" && session.metadata.includes("archived_sessions");
  }

  function actionLabel(action: AgentSessionDeletePlanItem["action"]) {
    const labels: Record<AgentSessionDeletePlanItem["action"], string> = {
      "delete-file": "删除文件",
      "delete-directory": "删除目录",
      "delete-opencode-session": "官方删除",
      skip: "跳过",
      manual: "人工处理",
    };
    return labels[action];
  }

  function normalizeDirectoryKey(value: string) {
    const trimmed = value.trim();
    return trimmed && trimmed !== "未知项目" ? trimmed : "未知目录";
  }
</script>

<template>
  <section class="agent-cleaner">
    <header class="agent-cleaner__header">
      <button
        class="agent-cleaner__back"
        type="button"
        @click="emit('backToTools')"
      >
        <DesktopUiIcon name="chevron-left" :size="15" />
        工具中心
      </button>

      <div class="agent-cleaner__title-block">
        <p class="agent-cleaner__kicker">Agent Sessions</p>
        <h2 class="agent-cleaner__title">Agent 会话清理</h2>
        <p class="agent-cleaner__summary">
          扫描 Claude Code、Codex 和 OpenCode 的本机会话，先预览删除计划，再选择备份删除或永久删除。
        </p>
      </div>

      <div class="agent-cleaner__header-actions">
        <button
          class="agent-cleaner__ghost-button"
          type="button"
          @click="openBackupsDirectory"
        >
          备份目录
        </button>
        <button
          class="agent-cleaner__primary-button"
          type="button"
          :disabled="isScanning"
          @click="scan"
        >
          {{ isScanning ? "扫描中" : "重新扫描" }}
        </button>
      </div>
    </header>

    <div
      v-if="feedbackMessage || errorMessage"
      :class="[
        'agent-cleaner__feedback',
        { 'agent-cleaner__feedback--error': errorMessage },
      ]"
    >
      {{ errorMessage || feedbackMessage }}
    </div>

    <div class="agent-cleaner__layout">
      <aside class="agent-cleaner__providers">
        <button
          :class="[
            'agent-cleaner__provider-filter',
            { 'agent-cleaner__provider-filter--active': selectedProviderId === 'all' },
          ]"
          type="button"
          @click="handleProviderFilter('all')"
        >
          <span>全部 Provider</span>
          <strong>{{ providerSessionCount }}</strong>
        </button>

        <button
          v-for="provider in providers"
          :key="provider.id"
          :class="[
            'agent-cleaner__provider',
            `agent-cleaner__provider--${provider.status}`,
            { 'agent-cleaner__provider--active': selectedProviderId === provider.id },
          ]"
          type="button"
          @click="handleProviderFilter(provider.id)"
        >
          <span class="agent-cleaner__provider-top">
            <strong>{{ provider.name }}</strong>
            <span>{{ providerStatusLabel(provider.status) }}</span>
          </span>
          <span class="agent-cleaner__provider-path">{{ provider.dataDir }}</span>
          <span class="agent-cleaner__provider-message">{{ provider.message }}</span>
          <span class="agent-cleaner__provider-meta">
            {{ provider.sessionCount }} 个会话 · {{ provider.deletionSupport }}
          </span>
        </button>
      </aside>

      <main class="agent-cleaner__main">
        <section class="agent-cleaner__toolbar">
          <div>
            <strong>{{ filteredSessions.length }} 个会话</strong>
            <span>已选 {{ selectedCount }} 个</span>
          </div>
          <div class="agent-cleaner__toolbar-actions">
            <button type="button" @click="selectVisibleSessions">全选当前</button>
            <button type="button" @click="clearSelection">清空</button>
            <button
              type="button"
              :disabled="selectedCount === 0 || isPlanning"
              @click="handleCreatePlan"
            >
              {{ isPlanning ? "生成中" : "预览删除计划" }}
            </button>
          </div>
        </section>

        <section class="agent-cleaner__content">
          <div v-if="filteredSessions.length === 0" class="agent-cleaner__empty">
            <DesktopUiIcon name="tools" :size="30" />
            <strong>没有可展示的会话</strong>
            <span>请确认对应 Agent 已安装并产生过本机会话数据。</span>
          </div>

          <section
            v-for="group in sessionDirectoryGroups"
            :key="group.id"
            :class="[
              'agent-cleaner__directory-group',
              { 'agent-cleaner__directory-group--high': group.riskLevel === 'high' },
            ]"
          >
            <header class="agent-cleaner__directory-header">
              <button
                class="agent-cleaner__directory-toggle"
                type="button"
                :aria-expanded="group.isExpanded"
                :aria-label="group.isExpanded ? '收起会话分组' : '展开会话分组'"
                :title="group.isExpanded ? '收起会话分组' : '展开会话分组'"
                @click="toggleDirectoryGroup(group.id)"
              >
                <DesktopUiIcon
                  class="agent-cleaner__directory-toggle-icon"
                  :class="{ 'agent-cleaner__directory-toggle-icon--open': group.isExpanded }"
                  name="chevron-right"
                  :size="14"
                />
              </button>
              <button
                class="agent-cleaner__directory-check"
                type="button"
                @click="handleDirectorySelection(group)"
              >
                <span v-if="group.selectedInGroup > 0" />
              </button>
              <div class="agent-cleaner__directory-copy">
                <strong>{{ group.directory }}</strong>
                <span>{{ group.providerNames.join(" · ") }}</span>
              </div>
              <div class="agent-cleaner__directory-meta">
                <strong>{{ group.sessions.length }} 个会话</strong>
                <span>已选 {{ group.selectedInGroup }} · {{ formatBytes(group.totalSizeBytes) }}</span>
                <small>{{ formatTimestamp(group.latestUpdatedAt) }}</small>
              </div>
            </header>

            <div v-if="group.isExpanded" class="agent-cleaner__session-list">
              <button
                v-for="session in group.sessions"
                :key="`${session.providerId}:${session.id}`"
                :class="[
                  'agent-cleaner__session',
                  `agent-cleaner__session--${session.riskLevel}`,
                  { 'agent-cleaner__session--selected': isSessionSelected(session) },
                ]"
                type="button"
                @click="toggleSession(session)"
              >
                <span class="agent-cleaner__session-check">
                  <span />
                </span>
                <span class="agent-cleaner__session-copy">
                  <span class="agent-cleaner__session-title-line">
                    <strong>{{ session.title }}</strong>
                    <span
                      v-if="isArchivedSession(session)"
                      class="agent-cleaner__archive-badge"
                    >
                      已归档
                    </span>
                  </span>
                  <span>{{ providerNameById[session.providerId] ?? session.providerId }}</span>
                  <small>{{ session.metadata.join(" · ") }}</small>
                </span>
                <span class="agent-cleaner__session-meta">
                  <strong>{{ formatBytes(session.sizeBytes) }}</strong>
                  <span>{{ formatTimestamp(session.updatedAt) }}</span>
                  <em>{{ riskLabel(session) }}</em>
                </span>
              </button>
            </div>
          </section>
        </section>
      </main>

      <aside class="agent-cleaner__plan">
        <section v-if="deletePlan" class="agent-cleaner__plan-panel">
          <header class="agent-cleaner__plan-header">
            <div>
              <strong>删除计划预览</strong>
              <span>{{ deletePlanSummary }}</span>
            </div>
            <span
              :class="[
                'agent-cleaner__risk-badge',
                { 'agent-cleaner__risk-badge--high': deletePlan.highRisk },
              ]"
            >
              {{ deletePlan.highRisk ? "高风险" : "可执行" }}
            </span>
          </header>

          <div class="agent-cleaner__plan-list">
            <section
              v-for="group in deletePlanGroups"
              :key="group.providerId"
              class="agent-cleaner__plan-group"
            >
              <header class="agent-cleaner__plan-group-header">
                <strong>{{ group.providerName }}</strong>
                <span>
                  {{ group.items.length }} 项 · 可执行 {{ group.actionableCount }} 项 · {{ formatBytes(group.totalSizeBytes) }}
                </span>
              </header>

              <div
                v-for="item in group.items"
                :key="item.id"
                class="agent-cleaner__plan-item"
              >
                <strong>{{ item.label }}</strong>
                <span>{{ actionLabel(item.action) }} · {{ riskLabel(item) }} · {{ formatBytes(item.sizeBytes) }}</span>
                <small>{{ item.target || item.message }}</small>
              </div>
            </section>
          </div>

          <label
            v-if="deletePlan.highRisk || hasHighRiskSelection"
            class="agent-cleaner__confirm"
          >
            <input v-model="highRiskConfirmed" type="checkbox" />
            我已确认所有 Agent 已退出，并接受高风险删除计划。
          </label>

          <div class="agent-cleaner__delete-actions">
            <button
              class="agent-cleaner__danger-button"
              type="button"
              :disabled="!canExecutePlan || isDeleting"
              @click="handleBackupDeletePlan"
            >
              {{ isDeleting ? "执行中" : "备份并硬删除" }}
            </button>
            <button
              class="agent-cleaner__permanent-button"
              type="button"
              :disabled="!canExecutePlan || isDeleting"
              @click="handlePermanentDeletePlan"
            >
              {{ isDeleting ? "执行中" : "永久删除" }}
            </button>
          </div>
        </section>

        <section v-else class="agent-cleaner__plan-placeholder">
          <strong>先预览删除计划</strong>
          <span>执行前会列出目标、保护项、跳过项和预计释放空间。</span>
        </section>

        <section v-if="deleteResult" class="agent-cleaner__result">
          <strong>最近结果</strong>
          <span>
            成功 {{ deleteResult.deletedCount }} · 跳过 {{ deleteResult.skippedCount }} · 失败 {{ deleteResult.failedCount }}
          </span>
          <small v-if="deleteResult.backupPath">备份：{{ deleteResult.backupPath }}</small>
        </section>
      </aside>
    </div>
  </section>
</template>

<style scoped>
  .agent-cleaner {
    display: grid;
    grid-template-rows: auto auto minmax(0, 1fr);
    gap: 1rem;
    height: 100%;
    min-width: 0;
    min-height: 0;
    padding: 1.4rem;
    background: var(--desktop-bg);
    color: var(--desktop-ink);
  }

  .agent-cleaner__header {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    gap: 1rem;
    align-items: start;
  }

  .agent-cleaner__back,
  .agent-cleaner__ghost-button,
  .agent-cleaner__primary-button,
  .agent-cleaner__danger-button,
  .agent-cleaner__permanent-button,
  .agent-cleaner__toolbar-actions button {
    border: 1px solid var(--desktop-line);
    border-radius: 10px;
    background: var(--desktop-surface);
    color: var(--desktop-ink);
    font-size: 0.76rem;
    font-weight: 720;
    cursor: pointer;
  }

  .agent-cleaner__back {
    display: inline-flex;
    align-items: center;
    gap: 0.2rem;
    padding: 0.48rem 0.62rem;
  }

  .agent-cleaner__title-block {
    display: grid;
    gap: 0.28rem;
    min-width: 0;
  }

  .agent-cleaner__kicker,
  .agent-cleaner__summary {
    margin: 0;
    color: var(--desktop-muted);
  }

  .agent-cleaner__kicker {
    font-size: 0.66rem;
    font-weight: 820;
    letter-spacing: 0.14em;
    text-transform: uppercase;
  }

  .agent-cleaner__title {
    margin: 0;
    font-size: 1.36rem;
    letter-spacing: 0;
  }

  .agent-cleaner__summary {
    max-width: 52rem;
    font-size: 0.8rem;
    line-height: 1.6;
  }

  .agent-cleaner__header-actions,
  .agent-cleaner__toolbar-actions {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }

  .agent-cleaner__ghost-button,
  .agent-cleaner__primary-button,
  .agent-cleaner__danger-button,
  .agent-cleaner__permanent-button,
  .agent-cleaner__toolbar-actions button {
    min-height: 2.1rem;
    padding: 0 0.72rem;
  }

  .agent-cleaner__primary-button {
    border-color: rgba(var(--desktop-accent-rgb), 0.32);
    background: var(--desktop-accent);
    color: var(--desktop-accent-ink);
  }

  .agent-cleaner__danger-button {
    width: 100%;
    border-color: rgba(199, 56, 56, 0.32);
    background: #b83232;
    color: #fff;
  }

  .agent-cleaner__permanent-button {
    width: 100%;
    border-color: rgba(127, 29, 29, 0.45);
    background: #7f1d1d;
    color: #fff;
  }

  .agent-cleaner__primary-button:disabled,
  .agent-cleaner__danger-button:disabled,
  .agent-cleaner__permanent-button:disabled,
  .agent-cleaner__toolbar-actions button:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .agent-cleaner__feedback {
    padding: 0.65rem 0.8rem;
    border: 1px solid rgba(var(--desktop-accent-rgb), 0.18);
    border-radius: 10px;
    background: rgba(var(--desktop-accent-rgb), 0.06);
    color: var(--desktop-accent);
    font-size: 0.78rem;
  }

  .agent-cleaner__feedback--error {
    border-color: rgba(199, 56, 56, 0.24);
    background: rgba(199, 56, 56, 0.08);
    color: #b83232;
  }

  .agent-cleaner__layout {
    display: grid;
    grid-template-columns: 17rem minmax(0, 1fr) 22rem;
    gap: 1rem;
    min-height: 0;
  }

  .agent-cleaner__providers,
  .agent-cleaner__content,
  .agent-cleaner__plan {
    min-height: 0;
    overflow: auto;
  }

  .agent-cleaner__providers,
  .agent-cleaner__main,
  .agent-cleaner__plan-panel,
  .agent-cleaner__plan-placeholder,
  .agent-cleaner__result {
    border: 1px solid var(--desktop-line);
    border-radius: 12px;
    background: var(--desktop-surface);
  }

  .agent-cleaner__delete-actions {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    gap: 0.5rem;
  }

  .agent-cleaner__providers {
    display: grid;
    align-content: start;
    gap: 0.65rem;
    padding: 0.75rem;
  }

  .agent-cleaner__provider-filter,
  .agent-cleaner__provider {
    display: grid;
    gap: 0.35rem;
    width: 100%;
    padding: 0.75rem;
    border: 1px solid transparent;
    border-radius: 10px;
    background: transparent;
    color: var(--desktop-ink);
    text-align: left;
    cursor: pointer;
  }

  .agent-cleaner__provider-filter {
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: center;
    background: var(--desktop-surface-strong);
  }

  .agent-cleaner__provider--active,
  .agent-cleaner__provider-filter--active {
    border-color: rgba(var(--desktop-accent-rgb), 0.28);
    background: rgba(var(--desktop-accent-rgb), 0.07);
  }

  .agent-cleaner__provider-top,
  .agent-cleaner__provider-meta {
    display: flex;
    justify-content: space-between;
    gap: 0.5rem;
  }

  .agent-cleaner__provider-top span,
  .agent-cleaner__provider-meta,
  .agent-cleaner__provider-message,
  .agent-cleaner__provider-path {
    color: var(--desktop-muted);
    font-size: 0.72rem;
  }

  .agent-cleaner__provider-path,
  .agent-cleaner__provider-message {
    overflow-wrap: anywhere;
    line-height: 1.4;
  }

  .agent-cleaner__main {
    display: grid;
    grid-template-rows: auto minmax(0, 1fr);
    min-width: 0;
    min-height: 0;
  }

  .agent-cleaner__toolbar {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.8rem;
    border-bottom: 1px solid var(--desktop-line);
  }

  .agent-cleaner__toolbar div:first-child {
    display: grid;
    gap: 0.16rem;
  }

  .agent-cleaner__toolbar span {
    color: var(--desktop-muted);
    font-size: 0.72rem;
  }

  .agent-cleaner__content {
    display: grid;
    align-content: start;
    gap: 0.6rem;
    padding: 0.8rem;
  }

  .agent-cleaner__empty {
    display: grid;
    place-items: center;
    gap: 0.45rem;
    min-height: 16rem;
    color: var(--desktop-muted);
    text-align: center;
  }

  .agent-cleaner__directory-group {
    display: grid;
    gap: 0.52rem;
    padding: 0.7rem;
    border: 1px solid var(--desktop-line);
    border-radius: 10px;
    background: var(--desktop-bg);
  }

  .agent-cleaner__directory-group--high {
    border-color: rgba(199, 56, 56, 0.18);
  }

  .agent-cleaner__directory-header {
    display: grid;
    grid-template-columns: auto auto minmax(0, 1fr) auto;
    gap: 0.75rem;
    align-items: center;
  }

  .agent-cleaner__directory-toggle,
  .agent-cleaner__directory-check,
  .agent-cleaner__session-check {
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .agent-cleaner__directory-toggle {
    width: 1.45rem;
    height: 1.45rem;
    border: 1px solid transparent;
    border-radius: 8px;
    background: transparent;
    color: var(--desktop-muted);
    cursor: pointer;
  }

  .agent-cleaner__directory-toggle:hover {
    border-color: var(--desktop-line);
    background: var(--desktop-surface);
    color: var(--desktop-ink);
  }

  .agent-cleaner__directory-toggle-icon {
    transition: transform 0.16s ease;
  }

  .agent-cleaner__directory-toggle-icon--open {
    transform: rotate(90deg);
  }

  .agent-cleaner__directory-check,
  .agent-cleaner__session-check {
    width: 1.1rem;
    height: 1.1rem;
    border: 1px solid var(--desktop-line-strong);
    border-radius: 6px;
  }

  .agent-cleaner__directory-check {
    background: var(--desktop-surface);
    cursor: pointer;
  }

  .agent-cleaner__directory-check span {
    width: 0.52rem;
    height: 0.52rem;
    border-radius: 3px;
    background: var(--desktop-accent);
  }

  .agent-cleaner__directory-copy {
    display: grid;
    gap: 0.2rem;
    min-width: 0;
  }

  .agent-cleaner__directory-copy strong,
  .agent-cleaner__directory-copy span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .agent-cleaner__directory-copy span,
  .agent-cleaner__directory-meta span,
  .agent-cleaner__directory-meta small {
    color: var(--desktop-muted);
    font-size: 0.72rem;
  }

  .agent-cleaner__directory-meta {
    display: grid;
    gap: 0.18rem;
    min-width: 7.5rem;
    text-align: right;
  }

  .agent-cleaner__session-list {
    display: grid;
    gap: 0.52rem;
    padding-top: 0.58rem;
    border-top: 1px solid var(--desktop-line);
  }

  .agent-cleaner__session {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    gap: 0.75rem;
    align-items: center;
    width: 100%;
    padding: 0.8rem;
    border: 1px solid var(--desktop-line);
    border-radius: 10px;
    background: var(--desktop-surface-strong);
    color: var(--desktop-ink);
    text-align: left;
    cursor: pointer;
  }

  .agent-cleaner__session--selected {
    border-color: rgba(var(--desktop-accent-rgb), 0.42);
    background: rgba(var(--desktop-accent-rgb), 0.07);
  }

  .agent-cleaner__session--selected .agent-cleaner__session-check span {
    width: 0.52rem;
    height: 0.52rem;
    border-radius: 3px;
    background: var(--desktop-accent);
  }

  .agent-cleaner__session-copy {
    display: grid;
    gap: 0.22rem;
    min-width: 0;
  }

  .agent-cleaner__session-title-line {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    min-width: 0;
  }

  .agent-cleaner__session-title-line strong {
    min-width: 0;
  }

  .agent-cleaner__archive-badge {
    flex: 0 0 auto;
    padding: 0.14rem 0.36rem;
    border: 1px solid rgba(128, 91, 28, 0.24);
    border-radius: 999px;
    background: rgba(128, 91, 28, 0.1);
    color: #8a5a12;
    font-size: 0.66rem;
    font-weight: 760;
    line-height: 1.1;
  }

  .agent-cleaner__session-copy strong,
  .agent-cleaner__session-copy span,
  .agent-cleaner__session-copy small {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .agent-cleaner__session-copy span,
  .agent-cleaner__session-copy small,
  .agent-cleaner__session-meta span {
    color: var(--desktop-muted);
    font-size: 0.72rem;
  }

  .agent-cleaner__session-title-line .agent-cleaner__archive-badge {
    color: #8a5a12;
    font-size: 0.66rem;
  }

  .agent-cleaner__session-meta {
    display: grid;
    gap: 0.2rem;
    min-width: 6rem;
    text-align: right;
  }

  .agent-cleaner__session-meta em {
    color: var(--desktop-soft);
    font-size: 0.68rem;
    font-style: normal;
  }

  .agent-cleaner__session--high .agent-cleaner__session-meta em {
    color: #b83232;
  }

  .agent-cleaner__plan {
    display: grid;
    align-content: start;
    gap: 0.75rem;
  }

  .agent-cleaner__plan-panel,
  .agent-cleaner__plan-placeholder,
  .agent-cleaner__result {
    display: grid;
    gap: 0.75rem;
    padding: 0.9rem;
  }

  .agent-cleaner__plan-header {
    display: flex;
    justify-content: space-between;
    gap: 0.75rem;
  }

  .agent-cleaner__plan-header div,
  .agent-cleaner__plan-placeholder,
  .agent-cleaner__result {
    min-width: 0;
  }

  .agent-cleaner__plan-header div,
  .agent-cleaner__result {
    display: grid;
    gap: 0.22rem;
  }

  .agent-cleaner__plan-header span,
  .agent-cleaner__plan-placeholder span,
  .agent-cleaner__result span,
  .agent-cleaner__result small {
    color: var(--desktop-muted);
    font-size: 0.74rem;
    line-height: 1.45;
    overflow-wrap: anywhere;
  }

  .agent-cleaner__risk-badge {
    align-self: start;
    flex: 0 0 auto;
    padding: 0.24rem 0.44rem;
    border-radius: 999px;
    background: rgba(var(--desktop-accent-rgb), 0.08);
    color: var(--desktop-accent);
    font-size: 0.68rem;
    font-weight: 760;
  }

  .agent-cleaner__risk-badge--high {
    background: rgba(199, 56, 56, 0.1);
    color: #b83232;
  }

  .agent-cleaner__plan-list {
    display: grid;
    gap: 0.5rem;
    max-height: 18rem;
    overflow: auto;
  }

  .agent-cleaner__plan-group {
    display: grid;
    gap: 0.45rem;
    padding: 0.55rem;
    border: 1px solid var(--desktop-line);
    border-radius: 10px;
    background: var(--desktop-surface-strong);
  }

  .agent-cleaner__plan-group-header {
    display: flex;
    justify-content: space-between;
    gap: 0.75rem;
    min-width: 0;
  }

  .agent-cleaner__plan-group-header span {
    color: var(--desktop-muted);
    font-size: 0.7rem;
    overflow-wrap: anywhere;
    text-align: right;
  }

  .agent-cleaner__plan-item {
    display: grid;
    gap: 0.2rem;
    padding: 0.62rem;
    border: 1px solid rgba(var(--desktop-accent-rgb), 0.1);
    border-radius: 9px;
    background: var(--desktop-bg);
  }

  .agent-cleaner__plan-item span,
  .agent-cleaner__plan-item small {
    color: var(--desktop-muted);
    font-size: 0.7rem;
    overflow-wrap: anywhere;
  }

  .agent-cleaner__confirm {
    display: flex;
    gap: 0.45rem;
    align-items: flex-start;
    color: var(--desktop-muted);
    font-size: 0.74rem;
    line-height: 1.45;
  }

  @media (max-width: 1180px) {
    .agent-cleaner__layout {
      grid-template-columns: 1fr;
    }

    .agent-cleaner__header {
      grid-template-columns: 1fr;
    }
  }
</style>
