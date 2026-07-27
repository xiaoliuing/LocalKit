<script setup lang="ts">
import { computed, onBeforeUnmount, shallowRef } from 'vue'
import type { VideoTreeNode } from '@/api/videos'
import DesktopUiIcon from '@/components/ui/DesktopUiIcon.vue'

const props = defineProps<{
  activeVideoId: string
  node: VideoTreeNode
  sourceId: string
}>()

const emit = defineEmits<{
  selectVideo: [videoId: string]
}>()

const isOpen = shallowRef(false)
const isTooltipVisible = shallowRef(false)
const tooltipLeft = shallowRef(0)
const tooltipTop = shallowRef(0)
const isFolder = computed(() => props.node.kind === 'folder')
const nodeId = computed(() => `${props.sourceId}::${props.node.id}`)
const isActive = computed(() => props.activeVideoId === nodeId.value)

onBeforeUnmount(() => {
  hideTooltip()
})

function handleClick() {
  if (isFolder.value) {
    isOpen.value = !isOpen.value
    return
  }

  emit('selectVideo', nodeId.value)
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
  <li class="desktop-video-tree-node">
    <button
      :class="[
        'desktop-video-tree-node__button',
        {
          'desktop-video-tree-node__button--folder': isFolder,
          'desktop-video-tree-node__button--active': isActive,
        },
      ]"
      type="button"
      @blur="hideTooltip"
      @click="handleClick"
      @focus="showTooltip"
      @mouseenter="showTooltip"
      @mouseleave="hideTooltip"
    >
      <DesktopUiIcon
        v-if="isFolder"
        class="desktop-video-tree-node__chevron"
        :class="{ 'desktop-video-tree-node__chevron--open': isOpen }"
        name="chevron-right"
        :size="13"
      />
      <span
        v-else
        class="desktop-video-tree-node__spacer"
      />
      <DesktopUiIcon
        :name="isFolder ? 'folder' : 'video'"
        class="desktop-video-tree-node__kind-icon"
        :size="15"
      />
      <span class="desktop-video-tree-node__label">{{ node.name }}</span>
    </button>

    <Teleport to="body">
      <div
        v-if="isTooltipVisible"
        class="desktop-video-tree-node__tooltip"
        :style="{ left: `${tooltipLeft}px`, top: `${tooltipTop}px` }"
        role="tooltip"
      >
        {{ node.name }}
      </div>
    </Teleport>

    <ul
      v-if="isFolder && isOpen"
      class="desktop-video-tree-node__children"
    >
      <DesktopVideoTreeNode
        v-for="child in node.children"
        :key="child.id"
        :active-video-id="activeVideoId"
        :node="child"
        :source-id="sourceId"
        @select-video="emit('selectVideo', $event)"
      />
    </ul>
  </li>
</template>

<style scoped>
.desktop-video-tree-node {
  display: grid;
  gap: 0.12rem;
}

.desktop-video-tree-node__button {
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

.desktop-video-tree-node__button:hover {
  background: rgba(var(--desktop-accent-rgb), 0.07);
  color: var(--desktop-ink);
}

.desktop-video-tree-node__button--folder {
  color: var(--desktop-ink);
  font-weight: 720;
}

.desktop-video-tree-node__button--active {
  border-color: rgba(var(--desktop-accent-rgb), 0.3);
  background:
    linear-gradient(
      135deg,
      rgba(var(--desktop-accent-rgb), 0.16),
      rgba(var(--desktop-accent-rgb), 0.07)
    );
  color: var(--desktop-accent);
  font-weight: 760;
}

.desktop-video-tree-node__chevron {
  transform: rotate(0deg);
  transition: transform 0.16s ease;
}

.desktop-video-tree-node__chevron--open {
  transform: rotate(90deg);
}

.desktop-video-tree-node__spacer {
  width: 13px;
  flex: none;
}

.desktop-video-tree-node__kind-icon {
  flex: none;
}

.desktop-video-tree-node__label {
  overflow: hidden;
  min-width: 0;
  white-space: nowrap;
  text-overflow: ellipsis;
  font-size: 0.78rem;
}

.desktop-video-tree-node__tooltip {
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

.desktop-video-tree-node__children {
  display: grid;
  gap: 0.08rem;
  margin: 0;
  padding: 0 0 0 0.95rem;
  list-style: none;
}
</style>
