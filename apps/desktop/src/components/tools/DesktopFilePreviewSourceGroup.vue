<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, shallowRef } from 'vue'
import type { DesktopFilePreviewSource } from '@/composables/useDesktopFilePreviewLibrary'
import DesktopUiIcon from '@/components/ui/DesktopUiIcon.vue'
import DesktopFilePreviewTreeNode from '@/components/tools/DesktopFilePreviewTreeNode.vue'

const props = defineProps<{
  activeFileId: string
  expandedFolderIds: readonly string[]
  isOpen: boolean
  source: DesktopFilePreviewSource
}>()

const emit = defineEmits<{
  edit: [sourceId: string]
  remove: [sourceId: string]
  rescan: [sourceId: string]
  selectFile: [fileId: string]
  toggleFolder: [folderId: string]
  toggleSource: [sourceId: string]
}>()

const isMenuOpen = shallowRef(false)
const isTooltipVisible = shallowRef(false)
const menuPosition = shallowRef({ left: 0, top: 0 })
const tooltipLeft = shallowRef(0)
const tooltipTop = shallowRef(0)
const containsActiveFile = computed(() =>
  props.activeFileId.startsWith(`${props.source.id}::`),
)

onMounted(() => {
  window.addEventListener('pointerdown', closeMenu)
  window.addEventListener('blur', closeMenu)
  window.addEventListener('keydown', handleKeydown)
})

onBeforeUnmount(() => {
  window.removeEventListener('pointerdown', closeMenu)
  window.removeEventListener('blur', closeMenu)
  window.removeEventListener('keydown', handleKeydown)
  hideTooltip()
})

function toggleOpen() {
  emit('toggleSource', props.source.id)
}

function openMenu(event: MouseEvent) {
  event.preventDefault()
  const menuWidth = 144
  const menuHeight = 120
  menuPosition.value = {
    left: Math.min(event.clientX, window.innerWidth - menuWidth - 8),
    top: Math.min(event.clientY, window.innerHeight - menuHeight - 8),
  }
  isMenuOpen.value = true
}

function closeMenu() {
  isMenuOpen.value = false
}

function showTooltip(event: MouseEvent | FocusEvent) {
  const target = event.currentTarget
  if (!(target instanceof HTMLElement)) {
    return
  }

  const rect = target.getBoundingClientRect()
  tooltipLeft.value = Math.max(
    8,
    Math.min(rect.left + 24, window.innerWidth - 328),
  )
  tooltipTop.value = Math.max(
    8,
    Math.min(rect.bottom + 8, window.innerHeight - 44),
  )
  isTooltipVisible.value = true
}

function hideTooltip() {
  isTooltipVisible.value = false
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    closeMenu()
  }
}

function handleRescan() {
  emit('rescan', props.source.id)
  closeMenu()
}

function handleEdit() {
  emit('edit', props.source.id)
  closeMenu()
}

function handleRemove() {
  emit('remove', props.source.id)
  closeMenu()
}
</script>

<template>
  <section
    :class="[
      'desktop-file-preview-source-group',
      { 'desktop-file-preview-source-group--active': containsActiveFile },
    ]"
    @contextmenu="openMenu"
  >
    <button
      class="desktop-file-preview-source-group__header"
      type="button"
      @blur="hideTooltip"
      @click="toggleOpen"
      @focus="showTooltip"
      @mouseenter="showTooltip"
      @mouseleave="hideTooltip"
    >
      <DesktopUiIcon
        class="desktop-file-preview-source-group__chevron"
        :class="{ 'desktop-file-preview-source-group__chevron--open': isOpen }"
        name="chevron-right"
        :size="13"
      />
      <div>
        <strong :title="source.title">{{ source.title }}</strong>
        <span>{{ source.fileCount }} 个文件</span>
      </div>
    </button>

    <div v-if="isOpen" class="desktop-file-preview-source-group__content">
      <ul
        v-if="source.tree.length"
        class="desktop-file-preview-source-group__tree-list"
      >
        <DesktopFilePreviewTreeNode
          v-for="node in source.tree"
          :key="node.id"
          :active-file-id="activeFileId"
          :expanded-folder-ids="expandedFolderIds"
          :node="node"
          :source-id="source.id"
          @select-file="emit('selectFile', $event)"
          @toggle-folder="emit('toggleFolder', $event)"
        />
      </ul>
      <p v-else class="desktop-file-preview-source-group__message">
        {{ source.message }}
      </p>
    </div>

    <Teleport to="body">
      <div
        v-if="isTooltipVisible"
        class="desktop-file-preview-source-group__tooltip"
        :style="{
          left: `${tooltipLeft}px`,
          top: `${tooltipTop}px`,
        }"
        role="tooltip"
      >
        {{ source.title }}
      </div>

      <div
        v-if="isMenuOpen"
        class="desktop-file-preview-source-group__menu"
        :style="{
          left: `${menuPosition.left}px`,
          top: `${menuPosition.top}px`,
        }"
        @pointerdown.stop
      >
        <button type="button" @click="handleEdit">
          <DesktopUiIcon name="settings" :size="14" />
          编辑目录
        </button>
        <button type="button" @click="handleRescan">
          <DesktopUiIcon name="reset-view" :size="14" />
          重新扫描
        </button>
        <button
          class="desktop-file-preview-source-group__menu-danger"
          type="button"
          @click="handleRemove"
        >
          <DesktopUiIcon name="close" :size="14" />
          删除目录
        </button>
      </div>
    </Teleport>
  </section>
</template>

<style scoped>
.desktop-file-preview-source-group + .desktop-file-preview-source-group {
  margin-top: 0.55rem;
  padding-top: 0.55rem;
  border-top: 1px solid var(--desktop-line);
}

.desktop-file-preview-source-group__header {
  display: flex;
  align-items: center;
  gap: 0.34rem;
  width: 100%;
  min-height: 2.4rem;
  padding: 0.08rem 0.18rem 0.08rem 0.44rem;
  border: 1px solid rgba(var(--desktop-accent-rgb), 0.28);
  border-radius: 10px;
  background:
    linear-gradient(
      135deg,
      rgba(var(--desktop-accent-rgb), 0.18),
      rgba(var(--desktop-accent-rgb), 0.06)
    ),
    var(--desktop-surface);
  color: var(--desktop-ink);
  text-align: left;
  cursor: pointer;
}

.desktop-file-preview-source-group__header:hover {
  border-color: rgba(var(--desktop-accent-rgb), 0.44);
  background:
    linear-gradient(
      135deg,
      rgba(var(--desktop-accent-rgb), 0.24),
      rgba(var(--desktop-accent-rgb), 0.09)
    ),
    var(--desktop-surface);
}

.desktop-file-preview-source-group--active
  .desktop-file-preview-source-group__header {
  border-color: rgba(var(--desktop-accent-rgb), 0.48);
}

.desktop-file-preview-source-group__chevron {
  flex: none;
  transform: rotate(0deg);
  transform-origin: center;
  transition: transform 0.15s ease;
}

.desktop-file-preview-source-group__chevron--open {
  transform: rotate(90deg);
}

.desktop-file-preview-source-group__header > div {
  display: grid;
  gap: 0.04rem;
  min-width: 0;
}

.desktop-file-preview-source-group__header strong {
  overflow: hidden;
  color: var(--desktop-ink);
  font-size: 0.78rem;
  font-weight: 780;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.desktop-file-preview-source-group__header span,
.desktop-file-preview-source-group__message {
  color: var(--desktop-soft);
  font-size: 0.64rem;
}

.desktop-file-preview-source-group__content {
  padding-top: 0.18rem;
}

.desktop-file-preview-source-group__tree-list {
  display: grid;
  gap: 0.08rem;
  margin: 0;
  padding: 0;
  list-style: none;
}

.desktop-file-preview-source-group__message {
  margin: 0.25rem 0.7rem 0.45rem;
}

.desktop-file-preview-source-group__tooltip {
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

.desktop-file-preview-source-group__menu {
  position: fixed;
  z-index: 460;
  display: grid;
  width: 9.5rem;
  padding: 0.3rem;
  border: 1px solid var(--desktop-line-strong);
  border-radius: 10px;
  background: var(--desktop-surface-strong);
  box-shadow: 0 14px 42px rgba(var(--desktop-shadow), 0.28);
}

.desktop-file-preview-source-group__menu button {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  min-height: 2rem;
  padding: 0 0.58rem;
  border: 0;
  border-radius: 7px;
  background: transparent;
  color: var(--desktop-ink);
  font: inherit;
  font-size: 0.72rem;
  text-align: left;
  cursor: pointer;
}

.desktop-file-preview-source-group__menu button:hover {
  background: rgba(var(--desktop-accent-rgb), 0.08);
  color: var(--desktop-accent);
}

.desktop-file-preview-source-group__menu
  .desktop-file-preview-source-group__menu-danger:hover {
  background: rgba(220, 54, 68, 0.09);
  color: #d93644;
}
</style>
