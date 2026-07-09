<script setup lang="ts">
import { computed } from 'vue'
import type { DesktopSearchFilterOption, DesktopSearchScope } from '@/composables/useDesktopDocsSearch'
import type { SearchResult } from '@/types/docs'
import DesktopUiSelect from '@/components/ui/DesktopUiSelect.vue'
import DesktopSearchHighlightedText from './DesktopSearchHighlightedText.vue'

const props = defineProps<{
  query: string
  results: SearchResult[]
  scope: DesktopSearchScope
  selectedIndex: number
  sourceFilter: string
  sourceOptions: DesktopSearchFilterOption[]
  workspaceName: string
  workspaceFilter: string
  workspaceOptions: DesktopSearchFilterOption[]
}>()

const emit = defineEmits<{
  close: []
  moveSelection: [direction: 1 | -1]
  setSourceFilter: [sourceKey: string]
  setScope: [scope: DesktopSearchScope]
  submit: [slug?: string]
  setWorkspaceFilter: [workspaceId: string]
}>()

const hasResults = computed(() => props.results.length > 0)
const normalizedQuery = computed(() => props.query.trim())
const showWorkspaceFilter = computed(() => props.scope === 'global' && props.workspaceOptions.length > 1)
const showSourceFilter = computed(() => props.sourceOptions.length > 1)

const workspaceSelectOptions = computed(() => [
  { value: 'all', label: '全部文档仓库' },
  ...props.workspaceOptions.map((option) => ({
    value: option.id,
    label: `${option.label} · ${option.count}`,
  })),
])

const sourceSelectOptions = computed(() => [
  { value: 'all', label: '全部文档源' },
  ...props.sourceOptions.map((option) => ({
    value: option.id,
    label: `${option.label}${option.helper ? ` · ${option.helper}` : ''} · ${option.count}`,
  })),
])
const matchFieldLabelMap: Record<SearchResult['matchField'], string> = {
  body: '正文命中',
  heading: '目录命中',
  section: '目录分类命中',
  summary: '摘要命中',
  title: '标题命中',
}

function getMatchFieldLabel(field: SearchResult['matchField']) {
  return matchFieldLabelMap[field]
}
</script>

<template>
  <section
    class="desktop-search-panel"
    @click.stop
  >
    <div
      class="desktop-search-panel__tabs"
      role="tablist"
      aria-label="搜索范围"
    >
      <button
        :class="[
          'desktop-search-panel__tab',
          { 'desktop-search-panel__tab--active': props.scope === 'global' },
        ]"
        role="tab"
        :aria-selected="props.scope === 'global'"
        type="button"
        @click="emit('setScope', 'global')"
      >
        全局
      </button>
      <button
        :class="[
          'desktop-search-panel__tab',
          { 'desktop-search-panel__tab--active': props.scope === 'workspace' },
        ]"
        role="tab"
        :aria-selected="props.scope === 'workspace'"
        type="button"
        @click="emit('setScope', 'workspace')"
      >
        当前文档仓库
      </button>
    </div>

    <div
      v-if="showWorkspaceFilter || showSourceFilter"
      class="desktop-search-panel__filters"
    >
      <label
        v-if="showWorkspaceFilter"
        class="desktop-search-panel__filter"
      >
        <span class="desktop-search-panel__filter-label">文档仓库</span>
        <DesktopUiSelect
          :model-value="props.workspaceFilter"
          :options="workspaceSelectOptions"
          size="sm"
          @update:model-value="emit('setWorkspaceFilter', $event)"
        />
      </label>

      <label
        v-if="showSourceFilter"
        class="desktop-search-panel__filter"
      >
        <span class="desktop-search-panel__filter-label">文档源</span>
        <DesktopUiSelect
          :model-value="props.sourceFilter"
          :options="sourceSelectOptions"
          size="sm"
          @update:model-value="emit('setSourceFilter', $event)"
        />
      </label>
    </div>

    <div class="desktop-search-panel__results desktop-scroll">
      <div
        v-if="!normalizedQuery"
        class="desktop-search-panel__empty"
      >
        <p>默认会检索标题、摘要、目录和正文内容。</p>
        <p>你也可以先切换文档仓库或文档源范围。</p>
      </div>

      <div
        v-else-if="!hasResults"
        class="desktop-search-panel__empty"
      >
        <p>没有匹配结果</p>
      </div>

      <template v-else>
        <div class="desktop-search-panel__count">
          {{ props.results.length }} 条结果
        </div>
        <button
          v-for="(result, index) in props.results"
          :key="result.slug"
          :class="[
            'desktop-search-panel__result',
            {
              'desktop-search-panel__result--active': index === props.selectedIndex,
            },
          ]"
          type="button"
          @click="emit('submit', result.slug)"
        >
          <div class="desktop-search-panel__result-meta">
            <span
              v-if="result.workspaceName"
              class="desktop-search-panel__result-chip"
            >
              <DesktopSearchHighlightedText
                :query="normalizedQuery"
                :text="result.workspaceName"
              />
            </span>
            <span
              v-if="result.sourceLabel"
              class="desktop-search-panel__result-chip desktop-search-panel__result-chip--muted"
            >
              <DesktopSearchHighlightedText
                :query="normalizedQuery"
                :text="result.sourceLabel"
              />
            </span>
          </div>
          <DesktopSearchHighlightedText
            tag="strong"
            class="desktop-search-panel__result-title"
            :query="normalizedQuery"
            :text="result.title"
          />
          <span class="desktop-search-panel__result-summary">
            <DesktopSearchHighlightedText
              :query="normalizedQuery"
              :text="result.snippet || result.summary"
            />
          </span>
          <div class="desktop-search-panel__result-footer">
            <span class="desktop-search-panel__result-section">
              <DesktopSearchHighlightedText
                :query="normalizedQuery"
                :text="result.sectionTitle || result.section"
              />
            </span>
            <span class="desktop-search-panel__result-match">
              {{ getMatchFieldLabel(result.matchField) }}
            </span>
          </div>
        </button>
      </template>
    </div>
  </section>
</template>

<style scoped>
.desktop-search-panel {
  display: flex;
  flex-direction: column;
  width: 100%;
  border: 1px solid var(--desktop-line);
  border-radius: 8px;
  background: var(--desktop-surface-strong);
  box-shadow: 0 12px 40px rgba(var(--desktop-shadow), 0.2);
  overflow: hidden;
}

.desktop-search-panel__tabs {
  display: flex;
  gap: 0;
  padding: 0 0.75rem;
  border-bottom: 1px solid var(--desktop-line-subtle, var(--desktop-line));
}

.desktop-search-panel__tab {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0.5rem 0.88rem;
  border: 0;
  border-bottom: 2px solid transparent;
  background: transparent;
  color: var(--desktop-soft);
  font-size: 0.75rem;
  font-weight: 600;
  white-space: nowrap;
  margin-bottom: -1px;
  cursor: pointer;
  transition: color 0.15s ease, border-color 0.15s ease;
}

.desktop-search-panel__tab--active {
  color: var(--desktop-accent);
  border-bottom-color: var(--desktop-accent);
}

.desktop-search-panel__filters {
  display: flex;
  gap: 0.62rem;
  padding: 0.5rem 0.75rem;
  border-bottom: 1px solid var(--desktop-line-subtle, var(--desktop-line));
}

.desktop-search-panel__filter {
  display: grid;
  gap: 0.22rem;
  flex: 1 1 0;
  min-width: 0;
}

.desktop-search-panel__filter-label {
  color: var(--desktop-soft);
  font-size: 0.69rem;
  font-weight: 600;
}

.desktop-search-panel__results {
  display: grid;
  max-height: min(56vh, 22.5rem);
  overflow-y: auto;
}

.desktop-search-panel__count {
  padding: 0.45rem 0.88rem 0.2rem;
  color: var(--desktop-soft);
  font-size: 0.69rem;
}

.desktop-search-panel__empty {
  padding: 2rem 1.25rem;
  text-align: center;
  color: var(--desktop-soft);
  font-size: 0.75rem;
  line-height: 1.7;
}

.desktop-search-panel__empty p {
  margin: 0;
}

.desktop-search-panel__empty p + p {
  margin-top: 0.2rem;
}

.desktop-search-panel__result {
  display: grid;
  gap: 0.28rem;
  width: 100%;
  padding: 0.62rem 0.88rem;
  border: 0;
  border-bottom: 1px solid var(--desktop-line-subtle, var(--desktop-line));
  border-radius: 0;
  background: transparent;
  text-align: left;
  cursor: pointer;
  transition: background-color 0.12s ease;
}

.desktop-search-panel__result:last-child {
  border-bottom: 0;
}

.desktop-search-panel__result:hover,
.desktop-search-panel__result--active {
  background: rgba(var(--desktop-accent-rgb), 0.08);
}

.desktop-search-panel__result-meta {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 0.3rem;
}

.desktop-search-panel__result-chip {
  display: inline-flex;
  align-items: center;
  min-height: 1.25rem;
  padding: 0.06rem 0.38rem;
  border: 1px solid var(--desktop-line-subtle, var(--desktop-line));
  border-radius: 999px;
  background: transparent;
  color: var(--desktop-soft);
  font-size: 0.62rem;
  font-weight: 600;
}

.desktop-search-panel__result-chip--muted {
  color: var(--desktop-muted);
}

.desktop-search-panel__result-section {
  color: var(--desktop-soft);
  font-size: 0.62rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.desktop-search-panel__result-title {
  color: var(--desktop-ink);
  font-size: 0.82rem;
  font-weight: 600;
}

.desktop-search-panel__result-summary {
  color: var(--desktop-muted);
  font-size: 0.75rem;
  line-height: 1.5;
}

.desktop-search-panel__result-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.65rem;
}

.desktop-search-panel__result-match {
  flex: none;
  color: var(--desktop-soft);
  font-size: 0.62rem;
  font-weight: 600;
}

@media (max-width: 720px) {
  .desktop-search-panel__filters {
    flex-direction: column;
  }

  .desktop-search-panel__result-footer {
    flex-direction: column;
    align-items: flex-start;
  }
}

</style>
