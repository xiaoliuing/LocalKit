<script setup lang="ts">
import { computed, onBeforeUnmount, shallowRef } from 'vue'
import type { PreviewTreeNode } from '@/api/filePreview'
import DesktopUiIcon from '@/components/ui/DesktopUiIcon.vue'

const props = defineProps<{
  activeFileId: string
  expandedFolderIds: readonly string[]
  node: PreviewTreeNode
  sourceId: string
}>()

const emit = defineEmits<{
  selectFile: [fileId: string]
  toggleFolder: [folderId: string]
}>()

const isTooltipVisible = shallowRef(false)
const tooltipLeft = shallowRef(0)
const tooltipTop = shallowRef(0)
const isFolder = computed(() => props.node.kind === 'folder')
const nodeId = computed(() => `${props.sourceId}::${props.node.id}`)
const isActive = computed(() => props.activeFileId === nodeId.value)
const isOpen = computed(() => props.expandedFolderIds.includes(nodeId.value))

onBeforeUnmount(() => {
  hideTooltip()
})

function handleClick() {
  if (isFolder.value) {
    emit('toggleFolder', nodeId.value)
    return
  }

  emit('selectFile', nodeId.value)
}

function showTooltip(event: MouseEvent | FocusEvent) {
  const target = event.currentTarget
  if (!(target instanceof HTMLElement)) {
    return
  }

  const rect = target.getBoundingClientRect()
  tooltipLeft.value = Math.max(8, Math.min(rect.left + 24, window.innerWidth - 328))
  tooltipTop.value = Math.max(8, Math.min(rect.bottom + 8, window.innerHeight - 44))
  isTooltipVisible.value = true
}

function hideTooltip() {
  isTooltipVisible.value = false
}
</script>

<template>
  <li class="desktop-file-preview-tree-node">
    <button
      :class="[
        'desktop-file-preview-tree-node__button',
        {
          'desktop-file-preview-tree-node__button--folder': isFolder,
          'desktop-file-preview-tree-node__button--active': isActive,
        },
      ]"
      :data-file-id="isFolder ? undefined : nodeId"
      type="button"
      @blur="hideTooltip"
      @click="handleClick"
      @focus="showTooltip"
      @mouseenter="showTooltip"
      @mouseleave="hideTooltip"
    >
      <DesktopUiIcon
        v-if="isFolder"
        class="desktop-file-preview-tree-node__chevron"
        :class="{ 'desktop-file-preview-tree-node__chevron--open': isOpen }"
        name="chevron-right"
        :size="13"
      />
      <span v-else class="desktop-file-preview-tree-node__spacer" />
      <DesktopUiIcon
        :name="isFolder ? 'folder' : 'file'"
        class="desktop-file-preview-tree-node__kind-icon"
        :size="15"
      />
      <span class="desktop-file-preview-tree-node__label">{{ node.name }}</span>
    </button>

    <Teleport to="body">
      <div
        v-if="isTooltipVisible"
        class="desktop-file-preview-tree-node__tooltip"
        :style="{ left: `${tooltipLeft}px`, top: `${tooltipTop}px` }"
        role="tooltip"
      >
        {{ node.name }}
      </div>
    </Teleport>

    <ul v-if="isFolder && isOpen" class="desktop-file-preview-tree-node__children">
      <DesktopFilePreviewTreeNode
        v-for="child in node.children"
        :key="child.id"
        :active-file-id="activeFileId"
        :expanded-folder-ids="expandedFolderIds"
        :node="child"
        :source-id="sourceId"
        @select-file="emit('selectFile', $event)"
        @toggle-folder="emit('toggleFolder', $event)"
      />
    </ul>
  </li>
</template>

<style scoped>
.desktop-file-preview-tree-node {
  display: grid;
  gap: 0.12rem;
}

.desktop-file-preview-tree-node__button {
  display: flex;
  align-items: center;
  gap: 0.44rem;
  width: 100%;
  min-width: 0;
  min-height: 1.9rem;
  padding: 0 0.48rem;
  border: 1px solid transparent;
  border-radius: 8px;
  background: transparent;
  color: var(--desktop-muted);
  text-align: left;
  cursor: pointer;
}

.desktop-file-preview-tree-node__button:hover {
  background: rgba(var(--desktop-accent-rgb), 0.07);
  color: var(--desktop-ink);
}

.desktop-file-preview-tree-node__button--folder {
  color: var(--desktop-ink);
  font-weight: 720;
}

.desktop-file-preview-tree-node__button--active {
  border-color: rgba(var(--desktop-accent-rgb), 0.3);
  background:
    linear-gradient(
      135deg,
      rgba(var(--desktop-accent-rgb), 0.16),
      rgba(var(--desktop-accent-rgb), 0.07)
    );
  color: var(--desktop-accent);
  font-weight: 760;
  box-shadow: inset 3px 0 0 var(--desktop-accent);
}

.desktop-file-preview-tree-node__chevron {
  transform: rotate(0deg);
  transition: transform 0.16s ease;
}

.desktop-file-preview-tree-node__chevron--open {
  transform: rotate(90deg);
}

.desktop-file-preview-tree-node__spacer {
  width: 13px;
  flex: none;
}

.desktop-file-preview-tree-node__kind-icon {
  flex: none;
}

.desktop-file-preview-tree-node__label {
  overflow: hidden;
  min-width: 0;
  white-space: nowrap;
  text-overflow: ellipsis;
  font-size: 0.78rem;
}

.desktop-file-preview-tree-node__tooltip {
  position: fixed;
  z-index: 700;
  max-width: 20rem;
  padding: 0.45rem 0.58rem;
  border: 1px solid var(--desktop-line-strong);
  border-radius: 8px;
  background: var(--desktop-surface-strong);
  box-shadow: 0 12px 34px rgba(var(--desktop-shadow), 0.24);
  color: var(--desktop-ink);
  font-size: 0.72rem;
  line-height: 1.45;
  overflow-wrap: anywhere;
  pointer-events: none;
}

.desktop-file-preview-tree-node__children {
  display: grid;
  gap: 0.08rem;
  margin: 0;
  padding: 0 0 0 0.95rem;
  list-style: none;
}
</style>
