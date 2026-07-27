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
    sources,
    addSource,
    chooseFolderPath,
    rememberPlayback,
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
  const artPlayer = shallowRef<Artplayer | null>(null);
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
    await restoreSources();
    await revealCurrentVideoInTree();
  });

  onBeforeUnmount(() => {
    persistPlaybackFromDocument();
    destroyArtPlayer();
  });

  watch(
    [currentVideoId, currentVideoUrl],
    async () => {
      lastRememberedSecond = -1;
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
    Artplayer.FULLSCREEN_WEB_IN_BODY = true;

    const player = new Artplayer({
      aspectRatio: true,
      autoplay: false,
      container,
      fullscreen: false,
      fullscreenWeb: true,
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
    });

    artPlayer.value = player;
    player.on("ready", () => {
      player.video.load();
    });
    player.on("video:loadedmetadata", restoreArtPlayerPosition);
    player.on("video:timeupdate", handleArtTimeUpdate);
    player.on("video:pause", handleArtPlaybackEvent);
    player.on("video:ended", handleArtPlaybackEvent);
    player.on("fullscreenError", () => {
      player.fullscreenWeb = true;
    });
  }

  function destroyArtPlayer() {
    artPlayer.value?.destroy(false);
    artPlayer.value = null;
  }

  function restoreArtPlayerPosition() {
    const video = artPlayer.value?.video;
    const position = currentPlaybackMemory.value?.position ?? 0;
    if (!video || position <= 0 || !Number.isFinite(video.duration)) {
      return;
    }

    video.currentTime = Math.min(position, Math.max(video.duration - 1, 0));
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
      { 'desktop-video-tool--library-collapsed': isLibraryCollapsed },
    ]"
  >
    <aside v-if="!isLibraryCollapsed" class="desktop-video-tool__library">
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

      <div
        ref="libraryTree"
        class="desktop-video-tool__tree desktop-scroll"
      >
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
      v-if="isLibraryCollapsed"
      class="desktop-video-tool__library-reveal-zone"
    >
      <button
        type="button"
        title="展开视频目录"
        @click="handleToggleLibrary"
      >
        <DesktopUiIcon name="chevron-right" :size="16" />
      </button>
    </div>

    <main class="desktop-video-tool__main">
      <section class="desktop-video-tool__stage">
        <div
          v-if="currentVideo && currentVideoUrl"
          :key="currentVideo.id"
          class="desktop-video-tool__video-frame"
        >
          <div ref="artPlayerContainer" class="desktop-video-tool__video" />
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
    height: 3.1rem;
    padding: 0;
    border: 0 !important;
    background: var(--desktop-line-strong);
    clip-path: polygon(0 12%, 100% 0, 100% 100%, 0 88%);
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
    transform: translateY(-50%);
  }

  .desktop-video-tool__library-collapse:hover,
  .desktop-video-tool__library-reveal-zone button:hover {
    color: var(--desktop-accent);
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
    width: 14px;
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

  .desktop-video-tool__video :deep(.art-control) {
    color: rgba(255, 255, 255, 0.92);
  }

  .desktop-video-tool__video :deep(.art-setting) {
    color: rgba(255, 255, 255, 0.92);
  }

  :global(.art-video-player.art-fullscreen-web) {
    position: fixed !important;
    inset: 38px 0 0 !important;
    z-index: 99999 !important;
    width: 100vw !important;
    height: calc(100vh - 38px) !important;
    max-width: none !important;
    max-height: none !important;
    background: #05070b !important;
  }

  :global(.art-video-player.art-fullscreen-web .art-video) {
    width: 100% !important;
    height: 100% !important;
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
