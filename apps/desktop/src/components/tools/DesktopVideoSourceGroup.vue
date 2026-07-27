<script setup lang="ts">
  import { onBeforeUnmount, onMounted, shallowRef } from "vue";
  import type { DesktopVideoSource } from "@/composables/useDesktopVideoLibrary";
  import DesktopUiIcon from "@/components/ui/DesktopUiIcon.vue";
  import DesktopVideoTreeNode from "@/components/tools/DesktopVideoTreeNode.vue";

  const props = defineProps<{
    activeVideoId: string;
    source: DesktopVideoSource;
  }>();

  const emit = defineEmits<{
    remove: [sourceId: string];
    rescan: [sourceId: string];
    selectVideo: [videoId: string];
  }>();

  const isOpen = shallowRef(true);
  const isMenuOpen = shallowRef(false);
  const menuPosition = shallowRef({ left: 0, top: 0 });

  onMounted(() => {
    window.addEventListener("pointerdown", closeMenu);
    window.addEventListener("blur", closeMenu);
    window.addEventListener("keydown", handleKeydown);
  });

  onBeforeUnmount(() => {
    window.removeEventListener("pointerdown", closeMenu);
    window.removeEventListener("blur", closeMenu);
    window.removeEventListener("keydown", handleKeydown);
  });

  function toggleOpen() {
    isOpen.value = !isOpen.value;
  }

  function openMenu(event: MouseEvent) {
    event.preventDefault();
    const menuWidth = 144;
    const menuHeight = 82;
    menuPosition.value = {
      left: Math.min(event.clientX, window.innerWidth - menuWidth - 8),
      top: Math.min(event.clientY, window.innerHeight - menuHeight - 8),
    };
    isMenuOpen.value = true;
  }

  function closeMenu() {
    isMenuOpen.value = false;
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      closeMenu();
    }
  }

  function handleRescan() {
    emit("rescan", props.source.id);
    closeMenu();
  }

  function handleRemove() {
    emit("remove", props.source.id);
    closeMenu();
  }
</script>

<template>
  <section class="desktop-video-source-group" @contextmenu="openMenu">
    <button
      class="desktop-video-source-group__header"
      type="button"
      @click="toggleOpen"
    >
      <DesktopUiIcon
        class="desktop-video-source-group__chevron"
        :class="{ 'desktop-video-source-group__chevron--open': isOpen }"
        name="chevron-right"
        :size="13"
      />
      <div>
        <strong :title="source.title">{{ source.title }}</strong>
        <span>{{ source.videoCount }} 个视频</span>
      </div>
    </button>

    <div v-if="isOpen" class="desktop-video-source-group__content">
      <ul
        v-if="source.tree.length"
        class="desktop-video-source-group__tree-list"
      >
        <DesktopVideoTreeNode
          v-for="node in source.tree"
          :key="node.id"
          :active-video-id="activeVideoId"
          :node="node"
          :source-id="source.id"
          @select-video="emit('selectVideo', $event)"
        />
      </ul>
      <p v-else class="desktop-video-source-group__message">
        {{ source.message }}
      </p>
    </div>

    <Teleport to="body">
      <div
        v-if="isMenuOpen"
        class="desktop-video-source-group__menu"
        :style="{
          left: `${menuPosition.left}px`,
          top: `${menuPosition.top}px`,
        }"
        @pointerdown.stop
      >
        <button type="button" @click="handleRescan">
          <DesktopUiIcon name="reset-view" :size="14" />
          重新扫描
        </button>
        <button
          class="desktop-video-source-group__menu-danger"
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
  .desktop-video-source-group + .desktop-video-source-group {
    margin-top: 0.55rem;
    padding-top: 0.55rem;
    border-top: 1px solid var(--desktop-line);
  }

  .desktop-video-source-group__header {
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

  .desktop-video-source-group__header:hover {
    border-color: rgba(var(--desktop-accent-rgb), 0.44);
    background:
      linear-gradient(
        135deg,
        rgba(var(--desktop-accent-rgb), 0.24),
        rgba(var(--desktop-accent-rgb), 0.09)
      ),
      var(--desktop-surface);
  }

  .desktop-video-source-group__chevron {
    flex: none;
    transform: rotate(0deg);
    transform-origin: center;
    transition: transform 0.15s ease;
  }

  .desktop-video-source-group__chevron--open {
    transform: rotate(90deg);
  }

  .desktop-video-source-group__header > div {
    display: grid;
    gap: 0.04rem;
    min-width: 0;
  }

  .desktop-video-source-group__header strong {
    overflow: hidden;
    color: var(--desktop-ink);
    font-size: 0.78rem;
    font-weight: 780;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .desktop-video-source-group__header span,
  .desktop-video-source-group__message {
    color: var(--desktop-soft);
    font-size: 0.64rem;
  }

  .desktop-video-source-group__content {
    padding-top: 0.18rem;
  }

  .desktop-video-source-group__tree-list {
    display: grid;
    gap: 0.08rem;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .desktop-video-source-group__message {
    margin: 0.25rem 0.7rem 0.45rem;
  }

  .desktop-video-source-group__menu {
    position: fixed;
    z-index: 460;
    display: grid;
    width: 9rem;
    padding: 0.3rem;
    border: 1px solid var(--desktop-line-strong);
    border-radius: 10px;
    background: var(--desktop-surface-strong);
    box-shadow: 0 14px 42px rgba(var(--desktop-shadow), 0.28);
  }

  .desktop-video-source-group__menu button {
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

  .desktop-video-source-group__menu button:hover {
    background: rgba(var(--desktop-accent-rgb), 0.08);
    color: var(--desktop-accent);
  }

  .desktop-video-source-group__menu
    .desktop-video-source-group__menu-danger:hover {
    background: rgba(220, 54, 68, 0.09);
    color: #d93644;
  }
</style>
