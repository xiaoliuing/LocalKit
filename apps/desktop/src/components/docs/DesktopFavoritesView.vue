<script setup lang="ts">
import { computed, shallowRef, watch } from 'vue'
import DesktopUiIcon from '@/components/ui/DesktopUiIcon.vue'
import DesktopUiSelect from '@/components/ui/DesktopUiSelect.vue'

export type DesktopFavoriteViewEntry = {
  id: string
  savedAt: string
  slug: string
  sourceLabel: string
  summary: string
  title: string
  workspaceId: string
  workspaceName: string
}

const props = defineProps<{
  entries: DesktopFavoriteViewEntry[]
}>()

const emit = defineEmits<{
  backToReader: []
  openEntry: [entryId: string]
  removeEntry: [entryId: string]
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
    ? '在阅读页面点击收藏按钮，即可将文档加入收藏列表。'
    : '当前筛选范围内还没有收藏文档，试试切换文档仓库。',
)

function formatSavedAt(value: string) {
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) {
    return '已收藏'
  }

  return new Intl.DateTimeFormat('zh-CN', {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(date)
}
</script>

<template>
  <section class="desktop-favorites-view">
    <header class="desktop-favorites-view__header">
      <div class="desktop-favorites-view__header-main">
        <h2 class="desktop-favorites-view__title">收藏</h2>
        <DesktopUiSelect
          v-model="selectedWorkspaceId"
          class="desktop-favorites-view__workspace-filter"
          :options="workspaceSelectOptions"
        />
      </div>

      <button class="desktop-favorites-view__back" type="button" @click="emit('backToReader')">
        <DesktopUiIcon name="chevron-left" :size="14" />
        <span>返回阅读</span>
      </button>
    </header>

    <div class="desktop-favorites-view__stage">
      <div class="desktop-favorites-view__body desktop-scroll">
        <div v-if="filteredEntries.length === 0" class="desktop-favorites-view__empty">
          <h2>还没有收藏文档</h2>
          <p>{{ emptyDescription }}</p>
        </div>

        <div v-else class="desktop-favorites-view__list">
          <article
            v-for="entry in filteredEntries"
            :key="entry.id"
            class="desktop-favorites-view__entry"
          >
            <button
              class="desktop-favorites-view__entry-main"
              type="button"
              @click="emit('openEntry', entry.id)"
            >
              <div class="desktop-favorites-view__entry-meta">
                <span class="desktop-favorites-view__chip">{{ entry.workspaceName }}</span>
                <span class="desktop-favorites-view__chip desktop-favorites-view__chip--muted">{{ entry.sourceLabel }}</span>
                <span class="desktop-favorites-view__entry-time">{{ formatSavedAt(entry.savedAt) }}</span>
              </div>

              <strong class="desktop-favorites-view__entry-title">{{ entry.title }}</strong>
              <p class="desktop-favorites-view__entry-summary">{{ entry.summary }}</p>
            </button>

            <div class="desktop-favorites-view__entry-actions">
              <button class="desktop-favorites-view__secondary" type="button" @click="emit('openEntry', entry.id)">
                继续阅读
              </button>
              <button class="desktop-favorites-view__secondary" type="button" @click="emit('removeEntry', entry.id)">
                取消收藏
              </button>
            </div>
          </article>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.desktop-favorites-view {
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  gap: 0;
  min-height: 0;
  height: 100%;
  width: 100%;
}

.desktop-favorites-view__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
  padding: 0.9rem 1.5rem 0.75rem;
  border-bottom: 1px solid var(--desktop-line-subtle, var(--desktop-line));
}

.desktop-favorites-view__header-main {
  display: grid;
  gap: 0.55rem;
  min-width: 0;
  flex: 1 1 auto;
}

.desktop-favorites-view__title {
  margin: 0;
  color: var(--desktop-ink);
  font-size: 0.96rem;
  font-weight: 600;
}

.desktop-favorites-view__workspace-filter {
  width: min(100%, 14rem);
}

.desktop-favorites-view__back {
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

.desktop-favorites-view__back:hover {
  background: rgba(0, 0, 0, 0.03);
  color: var(--desktop-ink);
}

.desktop-favorites-view__back span {
  font-size: 0.74rem;
  font-weight: 500;
}

.desktop-favorites-view__stage {
  display: grid;
  grid-template-rows: minmax(0, 1fr);
  min-height: 0;
  overflow: hidden;
  border: 0;
  border-radius: 0;
  background: var(--desktop-surface-strong);
}

.desktop-favorites-view__body {
  min-height: 0;
  height: 100%;
  overflow: auto;
  padding: 0 0.96rem 0.4rem;
}

.desktop-favorites-view__list {
  display: grid;
  gap: 0;
  border: 0;
  border-radius: 0;
  overflow: hidden;
  background: transparent;
}

.desktop-favorites-view__entry {
  display: grid;
  gap: 0.48rem;
  padding: 0.92rem 0.18rem 0.88rem;
  border: 0;
  border-radius: 0;
  background: transparent;
}

.desktop-favorites-view__entry + .desktop-favorites-view__entry {
  border-top: 1px solid var(--desktop-line);
}

.desktop-favorites-view__entry:hover {
  background: rgba(var(--desktop-accent-rgb), 0.045);
}

.desktop-favorites-view__entry-main {
  display: grid;
  gap: 0.38rem;
  width: 100%;
  padding: 0;
  border: 0;
  background: transparent;
  text-align: left;
  cursor: pointer;
}

.desktop-favorites-view__entry-meta,
.desktop-favorites-view__entry-actions {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 0.42rem;
}

.desktop-favorites-view__chip {
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

.desktop-favorites-view__chip--muted {
  background: transparent;
  color: var(--desktop-muted);
}

.desktop-favorites-view__entry-time {
  color: var(--desktop-soft);
  font-size: 0.68rem;
}

.desktop-favorites-view__entry-title {
  color: var(--desktop-ink);
  font-size: 0.92rem;
  font-weight: 650;
}

.desktop-favorites-view__entry-summary {
  margin: 0;
  color: var(--desktop-muted);
  font-size: 0.76rem;
  line-height: 1.52;
}

.desktop-favorites-view__secondary {
  display: inline-flex;
  align-items: center;
  min-height: 1.82rem;
  padding: 0.28rem 0.68rem;
  border: 1px solid rgba(var(--desktop-accent-rgb), 0.16);
  border-radius: 8px;
  background: rgba(var(--desktop-accent-rgb), 0.055);
  color: var(--desktop-accent);
  font: inherit;
  font-size: 0.69rem;
  font-weight: 600;
  cursor: pointer;
}

.desktop-favorites-view__body:has(> .desktop-favorites-view__empty) {
  padding: 0;
}

.desktop-favorites-view__empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 100%;
  padding: 2.5rem 1.5rem;
  text-align: center;
  background: var(--desktop-surface-strong);
}

.desktop-favorites-view__empty h2 {
  margin: 0 0 0.38rem;
  color: var(--desktop-ink);
  font-size: 1rem;
  font-weight: 600;
  line-height: 1.35;
}

.desktop-favorites-view__empty p {
  margin: 0;
  max-width: 24rem;
  color: var(--desktop-soft);
  font-size: 0.82rem;
  line-height: 1.6;
}

@media (max-width: 1100px) {
  .desktop-favorites-view__header,
  .desktop-favorites-view__entry-footer {
    align-items: flex-start;
    flex-direction: column;
  }

  .desktop-favorites-view__workspace-filter {
    width: 100%;
  }
}
</style>
