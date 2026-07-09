<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, shallowRef, useTemplateRef, watch } from 'vue'
import type { WorkspaceDetail } from '@docs-atlas/shared-types/workspace'
import type { DocsSourceGroup } from '@/types/docs'
import DesktopUiIcon from '@/components/ui/DesktopUiIcon.vue'
import DesktopDocsSidebarNode from './DesktopDocsSidebarNode.vue'

const props = defineProps<{
  activeView: 'reader' | 'recent' | 'favorites' | 'settings'
  currentDocSlug: string | null
  currentSectionId: string | null
  currentSourceId: string | null
  currentWorkspaceDocCount: number
  currentWorkspaceId: string
  currentWorkspaceSourceCount: number
  currentWorkspaceUnhealthySourceCount: number
  favoriteCount: number
  recentCount: number
  sourceGroups: DocsSourceGroup[]
  workspaces: WorkspaceDetail[]
}>()

const emit = defineEmits<{
  createWorkspace: []
  editWorkspace: []
  openFavorites: []
  openReader: []
  openRecent: []
  selectDoc: [slug: string]
  selectWorkspace: [workspaceId: string]
}>()

const openBranchIds = defineModel<string[]>('openBranchIds', { default: () => [] })
const openSectionId = defineModel<string | null>('openSectionId', { default: null })
const isWorkspaceMenuOpen = shallowRef(false)
const sidebarInnerRef = useTemplateRef<HTMLElement>('sidebarInner')
const workspaceSwitcherRef = useTemplateRef<HTMLElement>('workspaceSwitcher')

const activePath = computed(() => ({
  sectionId: props.currentSectionId,
  sourceId: props.currentSourceId,
}))
const currentWorkspace = computed(
  () => props.workspaces.find((workspace) => workspace.id === props.currentWorkspaceId) ?? null,
)
const isReaderView = computed(() => props.activeView === 'reader')
const isRecentView = computed(() => props.activeView === 'recent')
const isFavoritesView = computed(() => props.activeView === 'favorites')

function toggleNode(id: string, depth: number) {
  const currentId = openBranchIds.value[depth] ?? null

  if (currentId === id) {
    openBranchIds.value = openBranchIds.value.slice(0, depth)
    openSectionId.value = null
    return
  }

  const nextBranch = openBranchIds.value.slice(0, depth)
  nextBranch[depth] = id
  openBranchIds.value = nextBranch
  openSectionId.value = null
}

function toggleSection(sectionId: string) {
  openSectionId.value = openSectionId.value === sectionId ? null : sectionId
}

function handleSelectWorkspace(workspaceId: string) {
  emit('selectWorkspace', workspaceId)
  isWorkspaceMenuOpen.value = false
}

function syncOpenState() {
  if (!activePath.value.sourceId) {
    openBranchIds.value = []
    openSectionId.value = null
    return
  }

  const nodePath = findNodePathBySourceId(props.sourceGroups, activePath.value.sourceId)
  openBranchIds.value = nodePath
  openSectionId.value = activePath.value.sectionId
}

function scrollToActiveItem() {
  const container = sidebarInnerRef.value
  if (!container) {
    return
  }

  const activeItem =
    container.querySelector<HTMLElement>('.desktop-docs-sidebar-node__doc-link--active') ||
    container.querySelector<HTMLElement>('.desktop-docs-sidebar-node__section-row--active') ||
    container.querySelector<HTMLElement>('.desktop-docs-sidebar-node__toggle--active')

  activeItem?.scrollIntoView({
    block: 'nearest',
    inline: 'nearest',
  })
}

watch(
  activePath,
  async () => {
    syncOpenState()
    await nextTick()
    scrollToActiveItem()
  },
  { immediate: true },
)

watch(
  () => props.currentWorkspaceId,
  () => {
    isWorkspaceMenuOpen.value = false
  },
)

function handleWindowPointerDown(event: PointerEvent) {
  const switcher = workspaceSwitcherRef.value
  const target = event.target

  if (!switcher || !(target instanceof Node)) {
    return
  }

  if (!switcher.contains(target)) {
    isWorkspaceMenuOpen.value = false
  }
}

onMounted(() => {
  window.addEventListener('pointerdown', handleWindowPointerDown)
})

onBeforeUnmount(() => {
  window.removeEventListener('pointerdown', handleWindowPointerDown)
})

function findNodePathBySourceId(nodes: DocsSourceGroup[], sourceId: string): string[] {
  for (const node of nodes) {
    if (node.sourceId === sourceId) {
      return [node.id]
    }

    const childPath = findNodePathBySourceId(node.children, sourceId)
    if (childPath.length > 0) {
      return [node.id, ...childPath]
    }
  }

  return []
}
</script>

<template>
  <aside
    :class="[
      'desktop-docs-sidebar',
      { 'desktop-docs-sidebar--compact': !isReaderView },
    ]"
  >
    <div class="desktop-docs-sidebar__rail">
      <button
        :class="['desktop-docs-sidebar__rail-button', { 'desktop-docs-sidebar__rail-button--active': isReaderView }]"
        type="button"
        @click="emit('openReader')"
      >
        <span class="desktop-docs-sidebar__rail-icon">
          <DesktopUiIcon name="reader" :size="18" />
        </span>
        <span class="desktop-docs-sidebar__rail-text">阅读</span>
      </button>

      <button
        :class="['desktop-docs-sidebar__rail-button', { 'desktop-docs-sidebar__rail-button--active': isRecentView }]"
        type="button"
        @click="emit('openRecent')"
      >
        <span class="desktop-docs-sidebar__rail-icon">
          <DesktopUiIcon name="recent" :size="18" />
        </span>
        <span class="desktop-docs-sidebar__rail-text">最近</span>
        <!-- <span class="desktop-docs-sidebar__rail-count">{{ props.recentCount }}</span> -->
      </button>

      <button
        :class="['desktop-docs-sidebar__rail-button', { 'desktop-docs-sidebar__rail-button--active': isFavoritesView }]"
        type="button"
        @click="emit('openFavorites')"
      >
        <span class="desktop-docs-sidebar__rail-icon">
          <DesktopUiIcon name="bookmark" :size="18" />
        </span>
        <span class="desktop-docs-sidebar__rail-text">收藏</span>
        <!-- <span class="desktop-docs-sidebar__rail-count">{{ props.favoriteCount }}</span> -->
      </button>
    </div>

    <div v-if="isReaderView" class="desktop-docs-sidebar__panel">
      <div class="desktop-docs-sidebar__header">
        <div class="desktop-docs-sidebar__workspace-shell">
          <div class="desktop-docs-sidebar__workspace-topline">
            <p class="desktop-docs-sidebar__header-tag">文档仓库</p>
            <button
              class="desktop-docs-sidebar__workspace-create"
              type="button"
              @click="emit('createWorkspace')"
            >
              <DesktopUiIcon name="plus" :size="16" />
            </button>
          </div>

          <div
            ref="workspaceSwitcher"
            class="desktop-docs-sidebar__header-actions"
          >
            <button
              :aria-expanded="isWorkspaceMenuOpen"
              class="desktop-docs-sidebar__workspace-card"
              type="button"
              @click="isWorkspaceMenuOpen = !isWorkspaceMenuOpen"
            >
              <span
                class="desktop-docs-sidebar__workspace-card-icon"
                :style="{ color: currentWorkspace?.color || 'var(--desktop-accent)' }"
              />
              <span class="desktop-docs-sidebar__workspace-card-copy">
                <strong>{{ currentWorkspace?.name || '选择文档仓库' }}</strong>
                <span>{{ currentWorkspace?.description || '当前阅读入口' }}</span>
              </span>
              <DesktopUiIcon
                name="chevron-down"
                :size="15"
                :class="[
                  'desktop-docs-sidebar__workspace-card-chevron',
                  { 'desktop-docs-sidebar__workspace-card-chevron--open': isWorkspaceMenuOpen },
                ]"
              />
            </button>

            <div
              v-if="isWorkspaceMenuOpen"
              class="desktop-docs-sidebar__workspace-menu"
            >
              <button
                v-for="workspace in props.workspaces"
                :key="workspace.id"
                :class="[
                  'desktop-docs-sidebar__workspace-option',
                  { 'desktop-docs-sidebar__workspace-option--active': workspace.id === props.currentWorkspaceId },
                ]"
                type="button"
                @click="handleSelectWorkspace(workspace.id)"
              >
                <span
                  class="desktop-docs-sidebar__workspace-option-dot"
                  :style="{ backgroundColor: workspace.color }"
                />
                <span class="desktop-docs-sidebar__workspace-option-copy">
                  <strong>{{ workspace.name }}</strong>
                  <span>{{ `${workspace.sources.length} 个文档源` }}</span>
                </span>
                <span class="desktop-docs-sidebar__workspace-option-meta">
                  {{ workspace.id === props.currentWorkspaceId ? '当前' : '' }}
                </span>
              </button>
            </div>
          </div>

          <div class="desktop-docs-sidebar__workspace-footer">
            <div class="desktop-docs-sidebar__header-stats">
              <span class="desktop-docs-sidebar__header-stat">{{ `${props.currentWorkspaceSourceCount} 个文档源` }}</span>
              <span class="desktop-docs-sidebar__header-stat">{{ `${props.currentWorkspaceDocCount} 篇文档` }}</span>
              <span
                v-if="props.currentWorkspaceUnhealthySourceCount > 0"
                class="desktop-docs-sidebar__header-stat desktop-docs-sidebar__header-stat--warning"
              >
                {{ `${props.currentWorkspaceUnhealthySourceCount} 个异常` }}
              </span>
            </div>

            <button
              class="desktop-docs-sidebar__workspace-settings"
              type="button"
              @click="emit('editWorkspace')"
            >
              仓库设置
            </button>
          </div>
        </div>
      </div>

      <div
        ref="sidebarInner"
        class="desktop-docs-sidebar__scroll desktop-scroll"
      >
        <nav
          v-if="props.sourceGroups.length > 0"
          class="desktop-docs-sidebar__nav"
        >
          <DesktopDocsSidebarNode
            v-for="node in props.sourceGroups"
            :key="node.id"
            :current-doc-slug="props.currentDocSlug"
            :current-section-id="props.currentSectionId"
            :current-source-id="props.currentSourceId"
            :depth="0"
            :node="node"
            :open-branch-ids="openBranchIds"
            :open-section-id="openSectionId"
            @select-doc="emit('selectDoc', $event)"
            @toggle-node="toggleNode"
            @toggle-section="toggleSection"
          />
        </nav>

        <div v-else class="desktop-docs-sidebar__empty">
          当前文档仓库还没有可显示的文档。
        </div>
      </div>
    </div>

  </aside>
</template>

<style scoped>
.desktop-docs-sidebar {
  height: 100%;
  display: grid;
  grid-template-columns: var(--desktop-rail-w) minmax(0, 1fr);
  gap: 0;
  min-height: 0;
  border-right: 1px solid var(--desktop-line);
  background: var(--desktop-surface);
}

.desktop-docs-sidebar--compact {
  grid-template-columns: var(--desktop-rail-w);
}

.desktop-docs-sidebar__rail,
.desktop-docs-sidebar__panel {
  border: 0;
  border-radius: 0;
  background: transparent;
  box-shadow: none;
}

.desktop-docs-sidebar__rail {
  display: flex;
  flex-direction: column;
  align-items: stretch;
  gap: 0.12rem;
  padding: 0.75rem 0.38rem;
  min-height: 0;
  border-right: 1px solid var(--desktop-line);
  background: var(--desktop-surface);
}

.desktop-docs-sidebar__rail-button {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.28rem;
  width: 100%;
  padding: 0.62rem 0.2rem;
  border: 0;
  border-left: 2px solid transparent;
  border-radius: var(--desktop-radius-sm);
  background: transparent;
  color: var(--desktop-soft);
  cursor: pointer;
  transition: background-color 0.12s ease, color 0.12s ease, border-color 0.12s ease;
}

.desktop-docs-sidebar__rail-button:hover {
  background: rgba(0, 0, 0, 0.04);
  color: var(--desktop-muted);
}

:global(:root[data-theme='dark']) .desktop-docs-sidebar__rail-button:hover {
  background: rgba(255, 255, 255, 0.05);
}

.desktop-docs-sidebar__rail-button--active {
  background: rgba(var(--desktop-accent-rgb), 0.08);
  color: var(--desktop-accent);
}

.desktop-docs-sidebar__rail-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.5rem;
  height: 1.5rem;
  color: currentColor;
  opacity: 0.72;
}

.desktop-docs-sidebar__rail-button--active .desktop-docs-sidebar__rail-icon,
.desktop-docs-sidebar__rail-button:hover .desktop-docs-sidebar__rail-icon {
  opacity: 1;
}

.desktop-docs-sidebar__rail-text {
  font-size: 0.62rem;
  font-weight: 600;
  line-height: 1.15;
}

.desktop-docs-sidebar__rail-count {
  font-size: 0.56rem;
  font-weight: 700;
  line-height: 1;
  color: inherit;
  opacity: 0.7;
}

.desktop-docs-sidebar__panel {
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  min-height: 0;
  overflow: hidden;
  background: var(--desktop-surface);
}

.desktop-docs-sidebar__header {
  display: grid;
  gap: 0.34rem;
  padding: 0.56rem 0.68rem 0.52rem;
  border-bottom: 1px solid var(--desktop-line-subtle, var(--desktop-line));
  background: var(--desktop-surface);
}

.desktop-docs-sidebar__header-copy {
  display: grid;
  gap: 0.18rem;
  min-width: 0;
}

.desktop-docs-sidebar__workspace-shell {
  display: grid;
  gap: 0.38rem;
}

.desktop-docs-sidebar__workspace-topline {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.8rem;
}

.desktop-docs-sidebar__header-tag {
  margin: 0;
  color: var(--desktop-soft);
  font-size: 0.72rem;
  font-weight: 500;
}

.desktop-docs-sidebar__header-title-row {
  display: flex;
  align-items: center;
  gap: 0.42rem;
  min-width: 0;
}

.desktop-docs-sidebar__header-subtext {
  margin: 0;
  color: var(--desktop-muted);
  font-size: 0.74rem;
  line-height: 1.45;
}

.desktop-docs-sidebar__header-title {
  margin: 0;
  min-width: 0;
  color: var(--desktop-ink);
  font-size: 0.96rem;
  font-weight: 680;
  line-height: 1.2;
  letter-spacing: -0.02em;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.desktop-docs-sidebar__header-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 1.5rem;
  min-height: 1.32rem;
  padding: 0 0.32rem;
  border-radius: 999px;
  background: rgba(var(--desktop-accent-rgb), 0.09);
  color: var(--desktop-accent);
  font-size: 0.64rem;
  font-weight: 700;
}

.desktop-docs-sidebar__header-stats {
  display: flex;
  flex-wrap: wrap;
  gap: 0.2rem;
}

.desktop-docs-sidebar__header-stat {
  display: inline-flex;
  align-items: center;
  color: var(--desktop-soft);
  font-size: 0.68rem;
  font-weight: 500;
}

.desktop-docs-sidebar__header-stat + .desktop-docs-sidebar__header-stat::before {
  content: "·";
  margin-right: 0.38rem;
  color: var(--desktop-soft);
}

.desktop-docs-sidebar__header-stat--warning {
  color: #b56a1f;
}

.desktop-docs-sidebar__header-actions {
  position: relative;
  display: block;
}

.desktop-docs-sidebar__workspace-create {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.82rem;
  height: 1.82rem;
  border: 1px solid var(--desktop-line);
  border-radius: var(--desktop-radius-sm);
  background: transparent;
  color: var(--desktop-muted);
  cursor: pointer;
  transition: background-color 0.12s ease, color 0.12s ease;
}

.desktop-docs-sidebar__workspace-create:hover {
  background: rgba(0, 0, 0, 0.04);
  color: var(--desktop-ink);
}

.desktop-docs-sidebar__workspace-card {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  gap: 0.55rem;
  align-items: center;
  width: 100%;
  min-height: 2.5rem;
  padding: 0.5rem 0.62rem;
  border: 1px solid var(--desktop-line-subtle, var(--desktop-line));
  border-radius: var(--desktop-radius-sm);
  background: var(--desktop-surface-strong);
  color: var(--desktop-ink);
  text-align: left;
  cursor: pointer;
  transition: border-color 0.12s ease, background-color 0.12s ease;
}

.desktop-docs-sidebar__workspace-card:hover {
  border-color: var(--desktop-line);
  background: rgba(0, 0, 0, 0.02);
}

.desktop-docs-sidebar__workspace-card-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 0.5rem;
  height: 0.5rem;
  border-radius: 999px;
  background: currentColor;
}

.desktop-docs-sidebar__workspace-card-copy {
  display: grid;
  gap: 0.12rem;
  min-width: 0;
}

.desktop-docs-sidebar__workspace-card-copy strong {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 0.82rem;
  font-weight: 600;
  color: var(--desktop-ink);
}

.desktop-docs-sidebar__workspace-card-copy span {
  overflow: hidden;
  color: var(--desktop-soft);
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 0.7rem;
  font-weight: 400;
}

.desktop-docs-sidebar__workspace-card-chevron {
  transition: transform 0.18s ease;
}

.desktop-docs-sidebar__workspace-card-chevron--open {
  transform: rotate(180deg);
}

.desktop-docs-sidebar__workspace-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  padding-top: 0.12rem;
}

.desktop-docs-sidebar__workspace-settings {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 1.5rem;
  padding: 0;
  border: 0;
  background: transparent;
  color: var(--desktop-muted);
  font-size: 0.72rem;
  font-weight: 500;
  cursor: pointer;
  transition: color 0.12s ease;
}

.desktop-docs-sidebar__workspace-settings:hover {
  color: var(--desktop-accent);
}

.desktop-docs-sidebar__workspace-menu {
  position: absolute;
  top: calc(100% + 0.44rem);
  left: 0;
  right: 0;
  display: grid;
  gap: 0.2rem;
  padding: 0.3rem;
  border: 1px solid var(--desktop-line);
  border-radius: var(--desktop-radius-md);
  background: var(--desktop-surface-strong);
  box-shadow: 0 8px 24px rgba(var(--desktop-shadow), 0.12);
  z-index: 20;
}

.desktop-docs-sidebar__workspace-option {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  gap: 0.5rem;
  align-items: center;
  width: 100%;
  padding: 0.48rem 0.52rem;
  border: 0;
  border-radius: var(--desktop-radius-sm);
  background: transparent;
  text-align: left;
  cursor: pointer;
  transition: background-color 0.12s ease;
}

.desktop-docs-sidebar__workspace-option:hover,
.desktop-docs-sidebar__workspace-option--active {
  background: rgba(0, 0, 0, 0.04);
}

.desktop-docs-sidebar__workspace-option-dot {
  width: 0.5rem;
  height: 0.5rem;
  border-radius: 999px;
  flex-shrink: 0;
}

.desktop-docs-sidebar__workspace-option-copy strong {
  color: var(--desktop-ink);
  font-size: 0.78rem;
  font-weight: 600;
}

.desktop-docs-sidebar__workspace-option-copy span {
  color: var(--desktop-soft);
  font-size: 0.68rem;
  line-height: 1.3;
}

.desktop-docs-sidebar__workspace-option-meta {
  color: var(--desktop-accent);
  font-size: 0.68rem;
  font-weight: 500;
}

.desktop-docs-sidebar__workspace-option-copy {
  display: grid;
  gap: 0.1rem;
  min-width: 0;
}

.desktop-docs-sidebar__scroll {
  min-height: 0;
  overflow-y: auto;
  padding: 0.9rem 0.4rem 0;
}

.desktop-docs-sidebar__nav {
  display: grid;
  gap: 0;
}

.desktop-docs-sidebar__empty {
  padding: 1.5rem 1rem;
  color: var(--desktop-soft);
  font-size: 0.74rem;
  line-height: 1.5;
  text-align: center;
}

@media (max-width: 1240px) {
  .desktop-docs-sidebar {
    grid-template-columns: var(--desktop-rail-w) minmax(0, 1fr);
  }

  .desktop-docs-sidebar__header-actions {
    width: 100%;
  }

  .desktop-docs-sidebar__workspace-footer {
    align-items: flex-start;
    flex-direction: column;
  }
}
</style>
