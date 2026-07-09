<script setup lang="ts">
import { computed, shallowRef, watch } from 'vue'
import DesktopUiIcon from '@/components/ui/DesktopUiIcon.vue'
import DesktopUiSelect from '@/components/ui/DesktopUiSelect.vue'

export type DesktopRecentViewEntry = {
  id: string
  openedAt: string
  scrollTop: number
  slug: string
  sourceLabel: string
  summary: string
  title: string
  workspaceId: string
  workspaceName: string
}

const props = defineProps<{
  entries: DesktopRecentViewEntry[]
}>()

const emit = defineEmits<{
  backToReader: []
  openEntry: [entryId: string]
}>()

const ALL_WORKSPACES = '__all__'
const selectedWorkspaceId = shallowRef(ALL_WORKSPACES)

const workspaceOptions = computed(() => {
  const grouped = new Map<string, { id: string, label: string, count: number }>()

  for (const entry of props.entries) {
    const current = grouped.get(entry.workspaceId)
    if (current) {
      current.count += 1
      continue
    }

    grouped.set(entry.workspaceId, {
      id: entry.workspaceId,
      label: entry.workspaceName,
      count: 1,
    })
  }

  return Array.from(grouped.values())
})

const workspaceSelectOptions = computed(() => [
  { value: ALL_WORKSPACES, label: '全部文档仓库' },
  ...workspaceOptions.value.map((option) => ({
    value: option.id,
    label: `${option.label} (${option.count})`,
  })),
])

const filteredEntries = computed(() =>
  selectedWorkspaceId.value === ALL_WORKSPACES
    ? props.entries
    : props.entries.filter((entry) => entry.workspaceId === selectedWorkspaceId.value),
)

watch(
  workspaceOptions,
  (options) => {
    if (selectedWorkspaceId.value === ALL_WORKSPACES) {
      return
    }

    if (!options.some((option) => option.id === selectedWorkspaceId.value)) {
      selectedWorkspaceId.value = ALL_WORKSPACES
    }
  },
  { immediate: true },
)

const emptyDescription = computed(() =>
  props.entries.length === 0
    ? '打开文档后，阅读记录会自动出现在这里。'
    : '当前筛选范围内还没有阅读记录，试试切换文档仓库。',
)

function formatOpenedAt(value: string) {
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) {
    return '最近阅读'
  }

  return new Intl.DateTimeFormat('zh-CN', {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(date)
}
</script>

<template>
  <section class="desktop-recent-view">
    <header class="desktop-recent-view__header">
      <div class="desktop-recent-view__header-main">
        <h2 class="desktop-recent-view__title">最近阅读</h2>
        <DesktopUiSelect
          v-model="selectedWorkspaceId"
          class="desktop-recent-view__workspace-filter"
          :options="workspaceSelectOptions"
        />
      </div>

      <button class="desktop-recent-view__back" type="button" @click="emit('backToReader')">
        <DesktopUiIcon name="chevron-left" :size="14" />
        <span>返回阅读</span>
      </button>
    </header>

    <div class="desktop-recent-view__stage">
      <div class="desktop-recent-view__body desktop-scroll">
        <div v-if="filteredEntries.length === 0" class="desktop-recent-view__empty">
          <h2>还没有最近阅读记录</h2>
          <p>{{ emptyDescription }}</p>
        </div>

        <div v-else class="desktop-recent-view__list">
          <button
            v-for="entry in filteredEntries"
            :key="entry.id"
            class="desktop-recent-view__entry"
            type="button"
            @click="emit('openEntry', entry.id)"
          >
            <div class="desktop-recent-view__entry-meta">
              <span class="desktop-recent-view__chip">{{ entry.workspaceName }}</span>
              <span class="desktop-recent-view__chip desktop-recent-view__chip--muted">{{ entry.sourceLabel }}</span>
              <span class="desktop-recent-view__entry-time">{{ formatOpenedAt(entry.openedAt) }}</span>
            </div>

            <strong class="desktop-recent-view__entry-title">{{ entry.title }}</strong>
            <p class="desktop-recent-view__entry-summary">{{ entry.summary }}</p>

            <div class="desktop-recent-view__entry-footer">
              <span class="desktop-recent-view__entry-progress">
                {{ entry.scrollTop > 0 ? `上次滚动到 ${entry.scrollTop}px` : '从文档顶部开始阅读' }}
              </span>
              <span class="desktop-recent-view__entry-action">
                继续阅读
                <DesktopUiIcon name="chevron-right" :size="14" />
              </span>
            </div>
          </button>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.desktop-recent-view {
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  gap: 0;
  min-height: 0;
  height: 100%;
  width: 100%;
}

.desktop-recent-view__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
  padding: 0.9rem 1.5rem 0.75rem;
  border-bottom: 1px solid var(--desktop-line-subtle, var(--desktop-line));
}

.desktop-recent-view__header-main {
  display: grid;
  gap: 0.55rem;
  min-width: 0;
  flex: 1 1 auto;
}

.desktop-recent-view__title {
  margin: 0;
  color: var(--desktop-ink);
  font-size: 0.96rem;
  font-weight: 600;
}

.desktop-recent-view__workspace-filter {
  width: min(100%, 14rem);
}

.desktop-recent-view__back {
  display: inline-flex;
  align-items: center;
  gap: 0.32rem;
  padding: 0.34rem 0.58rem;
  border: 1px solid var(--desktop-line);
  border-radius: var(--desktop-radius-sm);
  background: transparent;
  color: var(--desktop-muted);
  cursor: pointer;
  flex-shrink: 0;
  transition: background-color 0.12s ease, color 0.12s ease;
}

.desktop-recent-view__back:hover {
  background: rgba(0, 0, 0, 0.03);
  color: var(--desktop-ink);
}

.desktop-recent-view__back span {
  font-size: 0.74rem;
  font-weight: 500;
}

.desktop-recent-view__stage {
  display: grid;
  grid-template-rows: minmax(0, 1fr);
  min-height: 0;
  overflow: hidden;
  border: 0;
  border-radius: 0;
  background: var(--desktop-surface-strong);
}

.desktop-recent-view__body {
  min-height: 0;
  height: 100%;
  overflow: auto;
  padding: 0 0.96rem 0.4rem;
}

.desktop-recent-view__list {
  display: grid;
  gap: 0;
  border: 0;
  border-radius: 0;
  overflow: hidden;
  background: transparent;
}

.desktop-recent-view__entry {
  display: grid;
  gap: 0.38rem;
  width: 100%;
  padding: 0.92rem 0.18rem 0.88rem;
  border: 0;
  border-radius: 0;
  background: transparent;
  text-align: left;
  cursor: pointer;
  transition: border-color 0.18s ease, background-color 0.18s ease, transform 0.18s ease;
}

.desktop-recent-view__entry + .desktop-recent-view__entry {
  border-top: 1px solid var(--desktop-line);
}

.desktop-recent-view__entry:hover {
  background: rgba(var(--desktop-accent-rgb), 0.045);
  transform: none;
}

.desktop-recent-view__entry-meta,
.desktop-recent-view__entry-footer {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 0.42rem;
}

.desktop-recent-view__chip {
  display: inline-flex;
  align-items: center;
  min-height: 1.36rem;
  padding: 0.12rem 0.44rem;
  border: 1px solid rgba(var(--desktop-accent-rgb), 0.1);
  border-radius: 999px;
  background: rgba(var(--desktop-accent-rgb), 0.05);
  color: var(--desktop-accent);
  font-size: 0.64rem;
  font-weight: 600;
}

.desktop-recent-view__chip--muted {
  background: transparent;
  color: var(--desktop-muted);
}

.desktop-recent-view__entry-time,
.desktop-recent-view__entry-progress {
  color: var(--desktop-soft);
  font-size: 0.68rem;
}

.desktop-recent-view__entry-title {
  color: var(--desktop-ink);
  font-size: 0.92rem;
  font-weight: 650;
}

.desktop-recent-view__entry-summary {
  margin: 0;
  color: var(--desktop-muted);
  font-size: 0.76rem;
  line-height: 1.52;
}

.desktop-recent-view__entry-footer {
  justify-content: space-between;
}

.desktop-recent-view__entry-action {
  display: inline-flex;
  align-items: center;
  gap: 0.18rem;
  color: var(--desktop-accent);
  font-size: 0.7rem;
  font-weight: 600;
}

.desktop-recent-view__body:has(> .desktop-recent-view__empty) {
  padding: 0;
}

.desktop-recent-view__empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 100%;
  padding: 2.5rem 1.5rem;
  text-align: center;
  background: var(--desktop-surface-strong);
}

.desktop-recent-view__empty h2 {
  margin: 0 0 0.38rem;
  color: var(--desktop-ink);
  font-size: 1rem;
  font-weight: 600;
  line-height: 1.35;
}

.desktop-recent-view__empty p {
  margin: 0;
  max-width: 24rem;
  color: var(--desktop-soft);
  font-size: 0.82rem;
  line-height: 1.6;
}

@media (max-width: 1100px) {
  .desktop-recent-view__header,
  .desktop-recent-view__entry-footer {
    align-items: flex-start;
    flex-direction: column;
  }

  .desktop-recent-view__workspace-filter {
    width: 100%;
  }
}
</style>
