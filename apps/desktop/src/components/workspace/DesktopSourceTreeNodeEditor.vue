<script setup lang="ts">
import { computed } from 'vue'
import type { WorkspaceSourceStatus } from '@docs-atlas/shared-types/workspace'
import DesktopUiIcon from '@/components/ui/DesktopUiIcon.vue'
import type { DraftNodeDropPlacement, WorkspaceSourceNodeDraft } from '@/utils/workspaceTree'

const node = defineModel<WorkspaceSourceNodeDraft>('node', { required: true })

const props = defineProps<{
  depth: number
  disabled?: boolean
  dragOverNodeId?: string | null
  dragPlacement?: DraftNodeDropPlacement | null
  draggedNodeId?: string | null
  expandedNodeIds: string[]
  isValidatingPathByNodeId: Record<string, boolean | undefined>
  issuesByNodeId: Record<string, string[]>
  siblingCount?: number
  siblingIndex?: number
  runtimeSourceStatusesByNodeId: Record<string, WorkspaceSourceStatus | undefined>
  pathStatusesByNodeId: Record<string, {
    exists: boolean
    isDirectory: boolean
  } | undefined>
}>()

const emit = defineEmits<{
  addFolder: [parentId: string]
  addGroup: [parentId: string]
  browseFolder: [nodeId: string]
  dragStart: [nodeId: string]
  moveNode: [nodeId: string, direction: -1 | 1]
  removeNode: [nodeId: string]
  toggleExpand: [nodeId: string]
}>()

const nodeIssues = computed(() => props.issuesByNodeId[node.value.id] ?? [])
const pathStatus = computed(() => props.pathStatusesByNodeId[node.value.id])
const isValidatingPath = computed(() => Boolean(props.isValidatingPathByNodeId[node.value.id]))
const runtimeSourceStatus = computed(() => props.runtimeSourceStatusesByNodeId[node.value.id])
const pathStatusLabel = computed(() => {
  if (node.value.kind !== 'folder') {
    return ''
  }

  if (!node.value.path.trim()) {
    return '未设置目录'
  }

  if (isValidatingPath.value) {
    return '校验中'
  }

  if (!pathStatus.value) {
    return '待校验'
  }

  if (!pathStatus.value.exists) {
    return '目录不存在'
  }

  if (!pathStatus.value.isDirectory) {
    return '路径不是目录'
  }

  return '目录有效'
})

const pathStatusTone = computed(() => {
  if (node.value.kind !== 'folder' || !node.value.path.trim()) {
    return 'muted'
  }

  if (isValidatingPath.value) {
    return 'pending'
  }

  if (pathStatus.value?.exists && pathStatus.value.isDirectory) {
    return 'success'
  }

  return 'error'
})

const runtimeStatusTone = computed(() => {
  const status = runtimeSourceStatus.value
  if (!status || node.value.kind !== 'folder') {
    return 'muted'
  }

  return status.state === 'ready' ? 'success' : 'error'
})
const canMoveUp = computed(() => (props.siblingIndex ?? 0) > 0)
const canMoveDown = computed(() => (props.siblingIndex ?? 0) < (props.siblingCount ?? 1) - 1)
const isDraggingSelf = computed(() => props.draggedNodeId === node.value.id)
const isDropBeforeActive = computed(() => props.dragOverNodeId === node.value.id && props.dragPlacement === 'before')
const isDropInsideActive = computed(() => props.dragOverNodeId === node.value.id && props.dragPlacement === 'inside')
const isDropAfterActive = computed(() => props.dragOverNodeId === node.value.id && props.dragPlacement === 'after')
const hasChildren = computed(() => node.value.children.length > 0)
const isExpanded = computed(() => props.expandedNodeIds.includes(node.value.id))

function forwardMoveNode(nodeId: string, direction: -1 | 1) {
  emit('moveNode', nodeId, direction)
}

function handlePointerDown(event: PointerEvent) {
  if (props.disabled || event.button !== 0) {
    return
  }

  event.preventDefault()
  emit('dragStart', node.value.id)
}
</script>

<template>
  <section
    class="desktop-source-tree-node"
    :style="{ '--source-tree-depth': depth }"
  >
    <div
      :class="[
        'desktop-source-tree-node__card',
        {
          'desktop-source-tree-node__card--dragging': isDraggingSelf,
          'desktop-source-tree-node__card--drop-before': isDropBeforeActive,
          'desktop-source-tree-node__card--drop-inside': isDropInsideActive,
          'desktop-source-tree-node__card--drop-after': isDropAfterActive,
        },
      ]"
      :data-source-tree-node-id="node.id"
    >
      <div class="desktop-source-tree-node__row">
        <button
          v-if="hasChildren"
          :aria-label="isExpanded ? '收起子节点' : '展开子节点'"
          :disabled="disabled"
          class="desktop-source-tree-node__expand"
          type="button"
          @click="emit('toggleExpand', node.id)"
        >
          <DesktopUiIcon
            name="chevron-down"
            :size="14"
            :class="[
              'desktop-source-tree-node__expand-icon',
              { 'desktop-source-tree-node__expand-icon--collapsed': !isExpanded },
            ]"
          />
        </button>
        <button
          :disabled="disabled"
          class="desktop-source-tree-node__drag-handle"
          type="button"
          @pointerdown="handlePointerDown"
        >
          <DesktopUiIcon name="grip" :size="15" />
        </button>
        <span
          :class="[
            'desktop-source-tree-node__kind',
            `desktop-source-tree-node__kind--${node.kind}`,
          ]"
        >
          {{ node.kind === 'group' ? '分组' : '目录' }}
        </span>

        <input
          v-model="node.name"
          :disabled="disabled"
          class="desktop-source-tree-node__name"
          :placeholder="node.kind === 'group' ? '分组名称' : '文档源名称'"
          type="text"
        />

        <label class="desktop-source-tree-node__switch">
          <input
            v-model="node.enabled"
            :disabled="disabled"
            type="checkbox"
          />
          <span>{{ node.enabled ? '启用' : '停用' }}</span>
        </label>
      </div>

      <div
        v-if="node.kind === 'folder'"
        class="desktop-source-tree-node__path-row"
      >
        <input
          v-model="node.path"
          :disabled="disabled"
          class="desktop-source-tree-node__path"
          placeholder="输入本地文档目录路径"
          type="text"
        />
        <button
          :disabled="disabled"
          class="desktop-source-tree-node__browse"
          type="button"
          @click="emit('browseFolder', node.id)"
        >
          选择
        </button>
      </div>

      <div
        v-if="node.kind === 'folder' || nodeIssues.length > 0"
        class="desktop-source-tree-node__meta"
      >
        <span
          v-if="node.kind === 'folder'"
          :class="[
            'desktop-source-tree-node__status-chip',
            `desktop-source-tree-node__status-chip--${pathStatusTone}`,
          ]"
        >
          {{ pathStatusLabel }}
        </span>

        <span
          v-if="node.kind === 'folder' && runtimeSourceStatus"
          :class="[
            'desktop-source-tree-node__status-chip',
            `desktop-source-tree-node__status-chip--${runtimeStatusTone}`,
          ]"
        >
          {{ runtimeSourceStatus.message }}
        </span>

        <span
          v-for="issue in nodeIssues"
          :key="issue"
          class="desktop-source-tree-node__status-chip desktop-source-tree-node__status-chip--error"
        >
          {{ issue }}
        </span>
      </div>

      <div class="desktop-source-tree-node__actions">
        <button
          :disabled="disabled || !canMoveUp"
          class="desktop-source-tree-node__icon-action"
          type="button"
          aria-label="上移节点"
          @click="emit('moveNode', node.id, -1)"
        >
          <DesktopUiIcon name="chevron-down" :size="14" class="desktop-source-tree-node__icon-action-icon desktop-source-tree-node__icon-action-icon--up" />
        </button>
        <button
          :disabled="disabled || !canMoveDown"
          class="desktop-source-tree-node__icon-action"
          type="button"
          aria-label="下移节点"
          @click="emit('moveNode', node.id, 1)"
        >
          <DesktopUiIcon name="chevron-down" :size="14" class="desktop-source-tree-node__icon-action-icon" />
        </button>
        <button
          :disabled="disabled"
          class="desktop-source-tree-node__action"
          type="button"
          @click="emit('addGroup', node.id)"
        >
          加分组
        </button>
        <button
          :disabled="disabled"
          class="desktop-source-tree-node__action"
          type="button"
          @click="emit('addFolder', node.id)"
        >
          加目录
        </button>
        <button
          :disabled="disabled"
          class="desktop-source-tree-node__action desktop-source-tree-node__action--danger"
          type="button"
          @click="emit('removeNode', node.id)"
        >
          删除
        </button>
      </div>
    </div>

    <div
      v-if="hasChildren && isExpanded"
      class="desktop-source-tree-node__children"
    >
      <DesktopSourceTreeNodeEditor
        v-for="(child, index) in node.children"
        :key="child.id"
        v-model:node="node.children[index]"
        :depth="depth + 1"
        :disabled="disabled"
        :drag-over-node-id="dragOverNodeId"
        :drag-placement="dragPlacement"
        :dragged-node-id="draggedNodeId"
        :expanded-node-ids="expandedNodeIds"
        :is-validating-path-by-node-id="isValidatingPathByNodeId"
        :issues-by-node-id="issuesByNodeId"
        :sibling-count="node.children.length"
        :sibling-index="index"
        :runtime-source-statuses-by-node-id="runtimeSourceStatusesByNodeId"
        :path-statuses-by-node-id="pathStatusesByNodeId"
        @add-folder="emit('addFolder', $event)"
        @add-group="emit('addGroup', $event)"
        @browse-folder="emit('browseFolder', $event)"
        @drag-start="emit('dragStart', $event)"
        @move-node="forwardMoveNode"
        @remove-node="emit('removeNode', $event)"
        @toggle-expand="emit('toggleExpand', $event)"
      />
    </div>
  </section>
</template>

<style scoped>
.desktop-source-tree-node {
  display: grid;
  gap: 0.45rem;
}

.desktop-source-tree-node__card {
  position: relative;
  display: grid;
  gap: 0.42rem;
  padding: 0.62rem 0.68rem;
  margin-left: calc(var(--source-tree-depth, 0) * 0.72rem);
  border: 1px solid var(--desktop-line-subtle, var(--desktop-line));
  border-radius: var(--desktop-radius-sm);
  background: var(--desktop-surface);
  transition:
    border-color 0.14s ease,
    background-color 0.14s ease,
    opacity 0.14s ease;
}

.desktop-source-tree-node__card--dragging {
  opacity: 0.55;
}

.desktop-source-tree-node__card--drop-inside {
  border-color: rgba(var(--desktop-accent-rgb), 0.32);
  background: rgba(var(--desktop-accent-rgb), 0.04);
}

.desktop-source-tree-node__card--drop-before::before,
.desktop-source-tree-node__card--drop-after::after {
  content: '';
  position: absolute;
  left: 0.75rem;
  right: 0.75rem;
  height: 2px;
  border-radius: 999px;
  background: var(--desktop-accent);
}

.desktop-source-tree-node__card--drop-before::before {
  top: -1px;
}

.desktop-source-tree-node__card--drop-after::after {
  bottom: -1px;
}

.desktop-source-tree-node__row,
.desktop-source-tree-node__path-row,
.desktop-source-tree-node__actions {
  display: flex;
  align-items: center;
  gap: 0.45rem;
}

.desktop-source-tree-node__kind {
  flex: none;
  min-height: 1.4rem;
  padding: 0.12rem 0.42rem;
  border-radius: 0.32rem;
  font-size: 0.66rem;
  font-weight: 600;
}

.desktop-source-tree-node__kind--group {
  background: rgba(var(--desktop-accent-rgb), 0.08);
  color: var(--desktop-accent);
}

.desktop-source-tree-node__kind--folder {
  background: rgba(0, 0, 0, 0.04);
  color: var(--desktop-muted);
}

.desktop-source-tree-node__expand,
.desktop-source-tree-node__drag-handle {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex: none;
  width: 1.5rem;
  min-height: 1.5rem;
  padding: 0;
  border: 1px solid var(--desktop-line-subtle, var(--desktop-line));
  border-radius: var(--desktop-radius-sm);
  background: transparent;
  color: var(--desktop-soft);
  cursor: pointer;
}

.desktop-source-tree-node__expand-icon {
  transition: transform 0.16s ease;
}

.desktop-source-tree-node__expand-icon--collapsed {
  transform: rotate(-90deg);
}

.desktop-source-tree-node__drag-handle {
  cursor: grab;
  touch-action: none;
  user-select: none;
}

.desktop-source-tree-node__drag-handle:active {
  cursor: grabbing;
}

.desktop-source-tree-node__name,
.desktop-source-tree-node__path {
  flex: 1 1 auto;
  width: 100%;
  min-width: 0;
  border: 1px solid var(--desktop-line);
  border-radius: var(--desktop-radius-sm);
  background: var(--desktop-surface-strong);
  color: var(--desktop-ink);
  font: inherit;
  font-size: 0.78rem;
  padding: 0.48rem 0.62rem;
}

.desktop-source-tree-node__switch {
  display: inline-flex;
  align-items: center;
  gap: 0.28rem;
  color: var(--desktop-soft);
  font-size: 0.72rem;
  white-space: nowrap;
}

.desktop-source-tree-node__switch input {
  margin: 0;
}

.desktop-source-tree-node__browse,
.desktop-source-tree-node__action {
  flex: none;
  min-height: 1.82rem;
  padding: 0 0.58rem;
  border: 1px solid var(--desktop-line);
  border-radius: var(--desktop-radius-sm);
  background: transparent;
  color: var(--desktop-muted);
  font: inherit;
  font-size: 0.74rem;
  cursor: pointer;
}

.desktop-source-tree-node__icon-action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex: none;
  width: 1.82rem;
  min-height: 1.82rem;
  padding: 0;
  border: 1px solid var(--desktop-line);
  border-radius: var(--desktop-radius-sm);
  background: transparent;
  color: var(--desktop-muted);
  font: inherit;
  cursor: pointer;
}

.desktop-source-tree-node__icon-action:disabled,
.desktop-source-tree-node__action:disabled,
.desktop-source-tree-node__browse:disabled {
  opacity: 0.48;
  cursor: not-allowed;
}

.desktop-source-tree-node__icon-action-icon--up {
  transform: rotate(180deg);
}

.desktop-source-tree-node__action--danger {
  color: #c55353;
}

.desktop-source-tree-node__actions {
  flex-wrap: wrap;
}

.desktop-source-tree-node__meta {
  display: flex;
  flex-wrap: wrap;
  gap: 0.35rem;
}

.desktop-source-tree-node__status-chip {
  min-height: 1.32rem;
  padding: 0.14rem 0.42rem;
  border-radius: 0.32rem;
  background: rgba(0, 0, 0, 0.04);
  color: var(--desktop-soft);
  font-size: 0.68rem;
  font-weight: 500;
}

.desktop-source-tree-node__status-chip--pending {
  color: var(--desktop-soft);
}

.desktop-source-tree-node__status-chip--success {
  color: #2f7b5f;
}

.desktop-source-tree-node__status-chip--error {
  color: #c53c53;
}

.desktop-source-tree-node__children {
  display: grid;
  gap: 0.45rem;
}

</style>
