<script setup lang="ts">
  import {
    computed,
    nextTick,
    onBeforeUnmount,
    onMounted,
    shallowRef,
    useTemplateRef,
    watch,
  } from "vue";
  import type { UnlistenFn } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import Artplayer from "artplayer";
  import DesktopUiIcon from "@/components/ui/DesktopUiIcon.vue";
  import DesktopVideoSourceGroup from "@/components/tools/DesktopVideoSourceGroup.vue";
  import DesktopVideoSourceDialog from "@/components/tools/DesktopVideoSourceDialog.vue";
  import { useDesktopVideoLibrary } from "@/composables/useDesktopVideoLibrary";

  const emit = defineEmits<{
    backToTools: [];
  }>();

  const {
    currentPlaybackMemory,
    currentVideo,
    currentVideoId,
    currentVideoUrl,
    expandedFolderIds,
    expandedSourceIds,
    feedbackMessage,
    isLibraryCollapsed,
    isScanning,
    playbackRate,
    sources,
    addSource,
    chooseFolderPath,
    rememberPlayback,
    rememberPlaybackRate,
    removeSource,
    rescanAllSources,
    rescanSource,
    restoreSources,
    selectVideo,
    setFeedback,
    toggleFolder,
    toggleLibrary,
    toggleSource,
  } = useDesktopVideoLibrary();

  const sourceDialog =
    useTemplateRef<InstanceType<typeof DesktopVideoSourceDialog>>(
      "sourceDialog",
    );
  const artPlayerContainer =
    useTemplateRef<HTMLDivElement>("artPlayerContainer");
  const libraryTree = useTemplateRef<HTMLDivElement>("libraryTree");
  const isSourceDialogOpen = shallowRef(false);
  const isNativeFullscreen = shallowRef(false);
  const isVideoReady = shallowRef(false);
  const artPlayer = shallowRef<Artplayer | null>(null);
  const VIDEO_NATIVE_FULLSCREEN_CLASS = "docs-atlas-video-native-fullscreen";
  const VIDEO_FULLSCREEN_SYNC_INTERVAL = 700;
  let fullscreenSyncTimer: number | null = null;
  let fullscreenSyncInterval: number | null = null;
  let windowEventUnlisteners: UnlistenFn[] = [];
  let lastRememberedSecond = -1;

  const emptyStateTitle = computed(() =>
    sources.value.length === 0 ? "添加视频目录" : "选择一个视频开始播放",
  );
  const emptyStateSummary = computed(() =>
    sources.value.length === 0
      ? "从本机选择视频目录，Docs Atlas 会自动整理子目录。"
      : "视频目录已经准备好，请从左侧选择视频。",
  );

  onMounted(async () => {
    window.addEventListener("keydown", handleNativeFullscreenKeydown);
    window.addEventListener("resize", scheduleNativeFullscreenSync);
    windowEventUnlisteners = await bindNativeFullscreenWindowEvents();
    await restoreSources();
    await syncNativeFullscreenState();
    await revealCurrentVideoInTree();
  });

  onBeforeUnmount(() => {
    persistPlaybackFromDocument();
    window.removeEventListener("keydown", handleNativeFullscreenKeydown);
    window.removeEventListener("resize", scheduleNativeFullscreenSync);
    if (fullscreenSyncTimer !== null) {
      window.clearTimeout(fullscreenSyncTimer);
      fullscreenSyncTimer = null;
    }
    stopNativeFullscreenPolling();
    for (const unlisten of windowEventUnlisteners) {
      unlisten();
    }
    windowEventUnlisteners = [];
    void setNativeVideoFullscreen(false);
    destroyArtPlayer();
  });

  watch(
    [currentVideoId, currentVideoUrl],
    async () => {
      lastRememberedSecond = -1;
      isVideoReady.value = false;
      await nextTick();
      mountArtPlayer();
    },
    { flush: "post" },
  );

  async function handleChooseFolder() {
    const path = await chooseFolderPath();
    if (path) {
      sourceDialog.value?.setPath(path);
    }
  }

  async function handleAddSource(input: { title: string; path: string }) {
    const saved = await addSource(input);
    if (saved) {
      isSourceDialogOpen.value = false;
    }
  }

  function handleVideoSelect(videoId: string) {
    persistPlaybackFromDocument();
    lastRememberedSecond = -1;
    isVideoReady.value = false;
    selectVideo(videoId);
    void revealCurrentVideoInTree();
  }

  function handleArtTimeUpdate() {
    const video = artPlayer.value?.video ?? null;
    if (!video) {
      return;
    }

    const currentSecond = Math.round(video.currentTime);
    if (currentSecond === lastRememberedSecond || currentSecond % 5 !== 0) {
      return;
    }

    lastRememberedSecond = currentSecond;
    rememberPlayback(video.currentTime, video.duration);
  }

  function handleArtPlaybackEvent() {
    const video = artPlayer.value?.video ?? null;
    if (video) {
      rememberPlayback(video.currentTime, video.duration);
    }
  }

  function handleArtPlaybackRateChange() {
    const rate = artPlayer.value?.video.playbackRate;
    if (rate) {
      rememberPlaybackRate(rate);
    }
  }

  function persistPlaybackFromDocument() {
    const video =
      artPlayer.value?.video ??
      document.querySelector<HTMLVideoElement>(
        ".desktop-video-tool__video-frame video",
      );
    if (video) {
      rememberPlayback(video.currentTime, video.duration);
    }
  }

  function mountArtPlayer() {
    destroyArtPlayer();

    const container = artPlayerContainer.value;
    if (!container || !currentVideo.value || !currentVideoUrl.value) {
      return;
    }

    Artplayer.DBCLICK_FULLSCREEN = false;

    const player = new Artplayer({
      aspectRatio: true,
      autoplay: false,
      container,
      fullscreen: false,
      fullscreenWeb: false,
      hotkey: true,
      lang: "zh-cn",
      loop: false,
      moreVideoAttr: {
        crossorigin: "anonymous",
        playsInline: true,
        preload: "auto",
      },
      mutex: true,
      muted: false,
      pip: true,
      playbackRate: true,
      setting: true,
      theme: resolvePlayerThemeColor(),
      title: currentVideo.value.name,
      url: currentVideoUrl.value,
      volume: 0.7,
      controls: [
        {
          name: "nativeFullscreen",
          position: "right",
          index: 99,
          html: createNativeFullscreenControlHtml(),
          tooltip: "全屏",
          click: () => {
            void toggleNativeVideoFullscreen();
          },
        },
      ],
    });

    artPlayer.value = player;
    player.playbackRate = playbackRate.value;
    player.on("video:loadeddata", revealDecodedVideoFrame);
    player.on("video:canplay", revealDecodedVideoFrame);
    player.on("video:seeked", revealDecodedVideoFrame);
    player.on("video:error", handleVideoReady);
    player.on("video:loadedmetadata", restoreArtPlayerPosition);
    player.on("video:timeupdate", handleArtTimeUpdate);
    player.on("video:pause", handleArtPlaybackEvent);
    player.on("video:ended", handleArtPlaybackEvent);
    player.on("video:ratechange", handleArtPlaybackRateChange);
    player.on("dblclick", handleArtDoubleClick);
  }

  function destroyArtPlayer() {
    artPlayer.value?.destroy(false);
    artPlayer.value = null;
  }

  async function toggleNativeVideoFullscreen() {
    await setNativeVideoFullscreen(!isNativeFullscreen.value);
  }

  function handleArtDoubleClick(event: Event) {
    event.preventDefault();
    event.stopPropagation();
    artPlayer.value?.toggle();
    void toggleNativeVideoFullscreen();
  }

  async function setNativeVideoFullscreen(state: boolean) {
    try {
      const appWindow = getCurrentWindow();
      await appWindow.setFullscreen(state);
      applyNativeFullscreenState(state);
    } catch (error) {
      console.error("Failed to toggle video fullscreen", error);
      setFeedback("当前系统环境暂时无法进入全屏");
    }
  }

  async function syncNativeFullscreenState() {
    try {
      applyNativeFullscreenState(await getCurrentWindow().isFullscreen());
    } catch (error) {
      console.error("Failed to sync video fullscreen state", error);
    }
  }

  function scheduleNativeFullscreenSync() {
    if (fullscreenSyncTimer !== null) {
      window.clearTimeout(fullscreenSyncTimer);
    }
    fullscreenSyncTimer = window.setTimeout(() => {
      fullscreenSyncTimer = null;
      void syncNativeFullscreenState();
    }, 120);
  }

  async function bindNativeFullscreenWindowEvents() {
    const appWindow = getCurrentWindow();
    const unlisteners = await Promise.all([
      appWindow.onResized(scheduleNativeFullscreenSync),
      appWindow.onFocusChanged(scheduleNativeFullscreenSync),
      appWindow.onScaleChanged(scheduleNativeFullscreenSync),
    ]);
    return unlisteners;
  }

  function applyNativeFullscreenState(state: boolean) {
    isNativeFullscreen.value = state;
    document.documentElement.classList.toggle(
      VIDEO_NATIVE_FULLSCREEN_CLASS,
      state,
    );
    document.body.classList.toggle(VIDEO_NATIVE_FULLSCREEN_CLASS, state);

    if (state) {
      startNativeFullscreenPolling();
    } else {
      stopNativeFullscreenPolling();
    }
  }

  function startNativeFullscreenPolling() {
    if (fullscreenSyncInterval !== null) {
      return;
    }
    fullscreenSyncInterval = window.setInterval(() => {
      void syncNativeFullscreenState();
    }, VIDEO_FULLSCREEN_SYNC_INTERVAL);
  }

  function stopNativeFullscreenPolling() {
    if (fullscreenSyncInterval === null) {
      return;
    }
    window.clearInterval(fullscreenSyncInterval);
    fullscreenSyncInterval = null;
  }

  function handleNativeFullscreenKeydown(event: KeyboardEvent) {
    if (event.key !== "Escape" || !isNativeFullscreen.value) {
      return;
    }

    event.preventDefault();
    event.stopPropagation();
    void setNativeVideoFullscreen(false);
  }

  function createNativeFullscreenControlHtml() {
    return `
      <span class="docs-atlas-player-fullscreen-control" aria-hidden="true">
        <svg class="docs-atlas-player-fullscreen-control__enter" viewBox="0 0 24 24">
          <path d="M4 9.5V5.75C4 4.78 4.78 4 5.75 4H9.5V5.8H6.05v3.7H4Zm10.5-3.7V4h3.75C19.22 4 20 4.78 20 5.75V9.5h-1.8V5.8h-3.7ZM4 14.5h1.8v3.7h3.7V20H5.75A1.75 1.75 0 0 1 4 18.25V14.5Zm14.2 0H20v3.75c0 .97-.78 1.75-1.75 1.75H14.5v-1.8h3.7v-3.7Z" />
        </svg>
        <svg class="docs-atlas-player-fullscreen-control__exit" viewBox="0 0 24 24">
          <path d="M8.95 4v3.75C8.95 8.72 8.17 9.5 7.2 9.5H4V7.7h3.15V4h1.8Zm6.1 0h1.8v3.7H20v1.8h-3.2c-.97 0-1.75-.78-1.75-1.75V4ZM4 14.5h3.2c.97 0 1.75.78 1.75 1.75V20h-1.8v-3.7H4v-1.8Zm12.85 1.8V20h-1.8v-3.75c0-.97.78-1.75 1.75-1.75H20v1.8h-3.15Z" />
        </svg>
      </span>
    `;
  }

  function handleVideoReady() {
    isVideoReady.value = true;
  }

  function revealDecodedVideoFrame() {
    const player = artPlayer.value;
    const video = player?.video;
    if (!player || !video) {
      return;
    }

    if (typeof video.requestVideoFrameCallback === "function") {
      video.requestVideoFrameCallback(() => {
        if (artPlayer.value === player) {
          handleVideoReady();
        }
      });
      return;
    }

    handleVideoReady();
  }

  function restoreArtPlayerPosition() {
    const video = artPlayer.value?.video;
    const position = currentPlaybackMemory.value?.position ?? 0;
    if (!video || !Number.isFinite(video.duration)) {
      return;
    }

    const previewPosition = position > 0 ? position : Math.min(0.05, video.duration / 2);
    video.currentTime = Math.min(
      previewPosition,
      Math.max(video.duration - 0.01, 0),
    );
  }

  function resolvePlayerThemeColor() {
    const root = document.documentElement;
    return (
      getComputedStyle(root).getPropertyValue("--desktop-accent").trim() ||
      "#1f54d9"
    );
  }

  function openSourceDialog() {
    setFeedback("");
    isSourceDialogOpen.value = true;
  }

  async function handleToggleLibrary() {
    toggleLibrary();
    await nextTick();
    if (!isLibraryCollapsed.value) {
      await revealCurrentVideoInTree();
    }
    window.dispatchEvent(new Event("resize"));
  }

  async function revealCurrentVideoInTree() {
    await nextTick();
    const tree = libraryTree.value;
    const videoId = currentVideoId.value;
    if (!tree || !videoId) {
      return;
    }

    const activeItem = tree.querySelector<HTMLElement>(
      `[data-video-id="${CSS.escape(videoId)}"]`,
    );
    if (!activeItem) {
      return;
    }

    const treeRect = tree.getBoundingClientRect();
    const itemRect = activeItem.getBoundingClientRect();
    const isVisible =
      itemRect.top >= treeRect.top && itemRect.bottom <= treeRect.bottom;
    if (isVisible) {
      return;
    }

    tree.scrollTo({
      behavior: "smooth",
      top:
        tree.scrollTop +
        itemRect.top -
        treeRect.top -
        (tree.clientHeight - itemRect.height) / 2,
    });
  }
</script>

<template>
  <section
    :class="[
      'desktop-video-tool',
      {
        'desktop-video-tool--library-collapsed': isLibraryCollapsed,
        'desktop-video-tool--native-fullscreen': isNativeFullscreen,
      },
    ]"
  >
    <aside
      v-if="!isLibraryCollapsed && !isNativeFullscreen"
      class="desktop-video-tool__library"
    >
      <header class="desktop-video-tool__library-header">
        <button
          class="desktop-video-tool__back"
          type="button"
          title="返回工具中心"
          @click="emit('backToTools')"
        >
          <DesktopUiIcon name="chevron-left" :size="15" />
        </button>
        <div class="desktop-video-tool__library-title">
          <strong>视频目录</strong>
          <span>{{ sources.length }} 个目录</span>
        </div>
        <div class="desktop-video-tool__library-actions">
          <button
            type="button"
            title="刷新全部目录"
            :disabled="isScanning"
            @click="rescanAllSources"
          >
            <DesktopUiIcon name="reset-view" :size="15" />
          </button>
          <button
            class="desktop-video-tool__add"
            type="button"
            title="添加视频目录"
            @click="openSourceDialog"
          >
            <DesktopUiIcon name="plus" :size="16" />
          </button>
        </div>
      </header>

      <div ref="libraryTree" class="desktop-video-tool__tree desktop-scroll">
        <div
          v-if="sources.length === 0"
          class="desktop-video-tool__library-empty"
        >
          <DesktopUiIcon name="file" :size="22" />
          <p>还没有视频目录</p>
          <button type="button" @click="openSourceDialog">添加目录</button>
        </div>

        <DesktopVideoSourceGroup
          v-for="source in sources"
          :key="source.id"
          :active-video-id="currentVideoId"
          :expanded-folder-ids="expandedFolderIds"
          :is-open="expandedSourceIds.includes(source.id)"
          :source="source"
          @remove="removeSource"
          @rescan="rescanSource"
          @select-video="handleVideoSelect"
          @toggle-folder="toggleFolder"
          @toggle-source="toggleSource"
        />
      </div>

      <button
        class="desktop-video-tool__library-collapse"
        type="button"
        title="收起视频目录"
        @click="handleToggleLibrary"
      >
        <DesktopUiIcon name="chevron-left" :size="16" />
      </button>
    </aside>

    <div
      v-if="isLibraryCollapsed && !isNativeFullscreen"
      class="desktop-video-tool__library-reveal-zone"
    >
      <button type="button" title="展开视频目录" @click="handleToggleLibrary">
        <DesktopUiIcon name="chevron-right" :size="16" />
      </button>
    </div>

    <main class="desktop-video-tool__main">
      <section class="desktop-video-tool__stage">
        <div
          v-if="currentVideo && currentVideoUrl"
          class="desktop-video-tool__video-frame"
        >
          <div ref="artPlayerContainer" class="desktop-video-tool__video" />
          <div
            v-if="!isVideoReady"
            class="desktop-video-tool__video-loading"
            aria-label="正在加载视频"
          >
            <span />
          </div>
        </div>

        <div v-else class="desktop-video-tool__empty">
          <span><DesktopUiIcon name="video" :size="36" /></span>
          <strong>{{ emptyStateTitle }}</strong>
          <p>{{ emptyStateSummary }}</p>
          <button
            v-if="sources.length === 0"
            type="button"
            @click="openSourceDialog"
          >
            <DesktopUiIcon name="plus" :size="14" />
            添加视频目录
          </button>
        </div>
      </section>
    </main>

    <DesktopVideoSourceDialog
      ref="sourceDialog"
      :feedback-message="feedbackMessage"
      :is-open="isSourceDialogOpen"
      :is-saving="isScanning"
      @cancel="isSourceDialogOpen = false"
      @choose-folder="handleChooseFolder"
      @save="handleAddSource"
    />
  </section>
</template>

<style scoped>
  .desktop-video-tool {
    position: relative;
    display: grid;
    grid-template-columns: 284px minmax(0, 1fr);
    width: 100%;
    height: 100%;
    min-width: 0;
    min-height: 0;
    background: var(--desktop-bg);
  }

  .desktop-video-tool--library-collapsed {
    grid-template-columns: minmax(0, 1fr);
  }

  .desktop-video-tool--native-fullscreen {
    grid-template-columns: minmax(0, 1fr);
    background: #000;
  }

  .desktop-video-tool button {
    border: 0;
    font: inherit;
    cursor: pointer;
  }

  .desktop-video-tool__library {
    position: relative;
    z-index: 30;
    display: grid;
    grid-template-rows: auto minmax(0, 1fr);
    min-width: 0;
    min-height: 0;
    border-right: 1px solid var(--desktop-line-strong);
    background: var(--desktop-surface-strong);
  }

  .desktop-video-tool__library-collapse,
  .desktop-video-tool__library-reveal-zone button {
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 1.6rem;
    height: 6rem;
    padding: 0;
    border: 0 !important;
    background: var(--desktop-line-strong);
    clip-path: polygon(0 0, 100% 16%, 100% 86%, 0 100%);
    filter: drop-shadow(4px 0 6px rgba(var(--desktop-shadow), 0.18));
    color: var(--desktop-muted);
  }

  .desktop-video-tool__library-collapse::before,
  .desktop-video-tool__library-reveal-zone button::before {
    position: absolute;
    inset: 1px;
    background: var(--desktop-surface-strong);
    clip-path: inherit;
    content: "";
  }

  .desktop-video-tool__library-collapse > *,
  .desktop-video-tool__library-reveal-zone button > * {
    position: relative;
    z-index: 1;
  }

  .desktop-video-tool__library-collapse {
    position: absolute;
    top: 50%;
    right: -1.6rem;
    z-index: 31;
    opacity: 0.48;
    transform: translateY(-50%);
    transition:
      opacity 140ms ease,
      filter 140ms ease;
  }

  .desktop-video-tool__library-collapse:hover,
  .desktop-video-tool__library-reveal-zone button:hover {
    color: var(--desktop-accent);
  }

  .desktop-video-tool__library-collapse:hover {
    opacity: 1;
  }

  .desktop-video-tool__library-collapse:hover::before,
  .desktop-video-tool__library-reveal-zone button:hover::before {
    background: color-mix(
      in srgb,
      var(--desktop-surface-strong) 90%,
      var(--desktop-accent)
    );
  }

  .desktop-video-tool__library-reveal-zone {
    position: absolute;
    inset: 0 auto 0 0;
    z-index: 20;
    width: 20px;
  }

  .desktop-video-tool__library-reveal-zone button {
    position: absolute;
    top: 50%;
    left: 0;
    opacity: 0;
    pointer-events: none;
    transform: translate(-0.45rem, -50%);
    transition:
      opacity 140ms ease,
      transform 140ms ease;
  }

  .desktop-video-tool__library-reveal-zone button::before {
    background: var(--desktop-accent);
  }

  .desktop-video-tool__library-reveal-zone button {
    background: color-mix(
      in srgb,
      var(--desktop-accent) 78%,
      var(--desktop-line-strong)
    );
    color: #fff;
    filter: drop-shadow(4px 0 8px rgba(0, 0, 0, 0.28));
  }

  .desktop-video-tool__library-reveal-zone button:hover::before {
    background: color-mix(in srgb, var(--desktop-accent) 88%, #fff);
  }

  .desktop-video-tool__library-reveal-zone button:hover {
    color: #fff;
  }

  .desktop-video-tool__library-reveal-zone:hover button,
  .desktop-video-tool__library-reveal-zone:focus-within button {
    opacity: 1;
    pointer-events: auto;
    transform: translate(0, -50%);
  }

  .desktop-video-tool__library-header {
    display: grid;
    grid-template-columns: 1.85rem minmax(0, 1fr) auto;
    align-items: center;
    column-gap: 0.5rem;
    min-height: 3.35rem;
    padding: 0 0.68rem;
    border-bottom: 1px solid var(--desktop-line);
  }

  .desktop-video-tool__library-title {
    display: flex;
    align-items: baseline;
    gap: 0.45rem;
    min-width: 0;
    overflow: hidden;
  }

  .desktop-video-tool__library-title strong {
    flex: none;
    color: var(--desktop-ink);
    font-size: 0.84rem;
    font-weight: 650;
    line-height: 1.2;
  }

  .desktop-video-tool__library-title span {
    min-width: 0;
    overflow: hidden;
    color: var(--desktop-soft);
    font-size: 0.65rem;
    line-height: 1.2;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .desktop-video-tool__library-actions {
    display: inline-flex;
    align-items: center;
    flex: none;
    gap: 0.25rem;
  }

  .desktop-video-tool__back,
  .desktop-video-tool__library-actions button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 1.85rem;
    height: 1.85rem;
    border-radius: 8px;
    background: transparent;
    color: var(--desktop-muted);
  }

  .desktop-video-tool__back:hover,
  .desktop-video-tool__library-actions button:hover {
    background: rgba(var(--desktop-accent-rgb), 0.08);
    color: var(--desktop-accent);
  }

  .desktop-video-tool__library-actions .desktop-video-tool__add {
    background: var(--desktop-accent);
    color: #fff;
  }

  .desktop-video-tool__tree {
    min-height: 0;
    overflow: auto;
    padding: 0.5rem 0.58rem 1rem;
  }

  .desktop-video-tool__library-empty {
    display: grid;
    justify-items: center;
    gap: 0.45rem;
    padding: 2rem 1rem;
    color: var(--desktop-soft);
    text-align: center;
  }

  .desktop-video-tool__library-empty p {
    margin: 0;
    font-size: 0.74rem;
  }

  .desktop-video-tool__library-empty button,
  .desktop-video-tool__empty button {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    min-height: 2rem;
    padding: 0 0.72rem;
    border-radius: 8px;
    background: rgba(var(--desktop-accent-rgb), 0.09);
    color: var(--desktop-accent);
    font-size: 0.7rem;
    font-weight: 720;
  }

  .desktop-video-tool__main {
    display: block;
    min-width: 0;
    min-height: 0;
    background: color-mix(
      in srgb,
      var(--desktop-bg) 96%,
      var(--desktop-accent)
    );
  }

  .desktop-video-tool__stage {
    position: relative;
    display: grid;
    width: 100%;
    height: 100%;
    min-width: 0;
    min-height: 0;
  }

  .desktop-video-tool__video-frame {
    position: relative;
    width: 100%;
    height: 100%;
    min-height: 0;
    overflow: hidden;
    box-shadow: 0 12px 36px rgba(var(--desktop-shadow), 0.16);
  }

  .desktop-video-tool__video {
    display: block;
    width: 100%;
    height: 100%;
  }

  .desktop-video-tool__video :deep(.art-video-player),
  .desktop-video-tool__video :deep(.art-video) {
    width: 100% !important;
    height: 100% !important;
  }

  .desktop-video-tool__video :deep(.art-video-player) {
    background: rgba(var(--desktop-accent-rgb), 0.01);
  }

  .desktop-video-tool__video-loading {
    position: absolute;
    inset: 0;
    z-index: 8;
    display: grid;
    place-items: center;
    background: color-mix(
      in srgb,
      var(--desktop-bg) 96%,
      var(--desktop-accent)
    );
    pointer-events: none;
  }

  .desktop-video-tool__video-loading span {
    width: 1.65rem;
    height: 1.65rem;
    border: 2px solid rgba(var(--desktop-accent-rgb), 0.2);
    border-top-color: var(--desktop-accent);
    border-radius: 50%;
    animation: desktop-video-loading-spin 720ms linear infinite;
  }

  @keyframes desktop-video-loading-spin {
    to {
      transform: rotate(360deg);
    }
  }

  .desktop-video-tool__video :deep(.art-control) {
    color: rgba(255, 255, 255, 0.92);
  }

  .desktop-video-tool__video :deep(.art-setting) {
    color: rgba(255, 255, 255, 0.92);
  }

  :global(html.docs-atlas-video-native-fullscreen),
  :global(html.docs-atlas-video-native-fullscreen body),
  :global(html.docs-atlas-video-native-fullscreen #app) {
    width: 100%;
    height: 100%;
    overflow: hidden;
    background: #000;
  }

  :global(html.docs-atlas-video-native-fullscreen .desktop-app-shell) {
    grid-template-rows: minmax(0, 1fr);
    background: #000;
  }

  :global(html.docs-atlas-video-native-fullscreen .desktop-titlebar) {
    display: none !important;
    visibility: hidden !important;
    height: 0 !important;
    min-height: 0 !important;
    overflow: hidden !important;
  }

  :global(.art-control-nativeFullscreen) {
    order: 999;
  }

  :global(.docs-atlas-player-fullscreen-control) {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 2.15rem;
    height: 2.15rem;
    color: rgba(255, 255, 255, 0.95);
  }

  :global(.docs-atlas-player-fullscreen-control svg) {
    width: 1.35rem;
    height: 1.35rem;
    fill: currentColor;
  }

  :global(.docs-atlas-player-fullscreen-control__exit) {
    display: none;
  }

  :global(.docs-atlas-video-native-fullscreen .docs-atlas-player-fullscreen-control__enter) {
    display: none;
  }

  :global(.docs-atlas-video-native-fullscreen .docs-atlas-player-fullscreen-control__exit) {
    display: block;
  }

  .desktop-video-tool__empty {
    display: grid;
    place-content: center;
    justify-items: center;
    gap: 0.55rem;
    color: var(--desktop-muted);
    text-align: center;
  }

  .desktop-video-tool__empty > span {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 4rem;
    height: 4rem;
    border-radius: 18px;
    background: rgba(var(--desktop-accent-rgb), 0.09);
    color: var(--desktop-accent);
  }

  .desktop-video-tool__empty strong {
    color: var(--desktop-ink);
    font-size: 0.94rem;
  }

  .desktop-video-tool__empty p {
    max-width: 25rem;
    margin: 0 0 0.25rem;
    font-size: 0.76rem;
    line-height: 1.6;
  }

  @media (max-width: 920px) {
    .desktop-video-tool {
      grid-template-columns: 230px minmax(0, 1fr);
    }
  }
</style>
