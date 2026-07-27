<script setup lang="ts">
  import {
    computed,
    onBeforeUnmount,
    onMounted,
    shallowRef,
    useTemplateRef,
  } from "vue";
  import { videoPlay as VideoPlay } from "vue3-video-play";
  import "vue3-video-play/dist/style.css";
  import DesktopUiIcon from "@/components/ui/DesktopUiIcon.vue";
  import DesktopVideoSourceGroup from "@/components/tools/DesktopVideoSourceGroup.vue";
  import DesktopVideoSourceDialog from "@/components/tools/DesktopVideoSourceDialog.vue";
  import { useDesktopVideoLibrary } from "@/composables/useDesktopVideoLibrary";
  import { getVideoMimeType } from "@/api/videos";

  const emit = defineEmits<{
    backToTools: [];
  }>();

  const {
    currentPlaybackMemory,
    currentVideo,
    currentVideoId,
    currentVideoUrl,
    feedbackMessage,
    isScanning,
    lastPlaybackVideo,
    sources,
    addSource,
    chooseFolderPath,
    rememberPlayback,
    removeSource,
    rescanAllSources,
    rescanSource,
    restoreSources,
    resumeLastVideo,
    selectVideo,
    setFeedback,
  } = useDesktopVideoLibrary();

  const sourceDialog =
    useTemplateRef<InstanceType<typeof DesktopVideoSourceDialog>>(
      "sourceDialog",
    );
  const isSourceDialogOpen = shallowRef(false);
  let lastRememberedSecond = -1;

  const playerOptions = computed(() => ({
    autoPlay: false,
    color: "#1f54d9",
    control: true,
    controlBtns: [
      "speedRate",
      "volume",
      "setting",
      "pip",
      "pageFullScreen",
      "fullScreen",
    ],
    currentTime: currentPlaybackMemory.value?.position ?? 0,
    height: "100%",
    loop: false,
    muted: false,
    playsinline: true,
    preload: "auto",
    speedRate: ["0.75", "1.0", "1.25", "1.5", "2.0"],
    src: currentVideoUrl.value,
    title: currentVideo.value?.name ?? "",
    type: currentVideo.value
      ? getVideoMimeType(currentVideo.value.path)
      : "video/mp4",
    volume: 0.7,
    width: "100%",
  }));

  const emptyStateTitle = computed(() =>
    sources.value.length === 0 ? "添加视频目录" : "选择一个视频开始播放",
  );
  const emptyStateSummary = computed(() =>
    sources.value.length === 0
      ? "从本机选择视频目录，Docs Atlas 会自动整理子目录。"
      : "视频目录已经准备好，请从左侧选择视频。",
  );

  onMounted(() => {
    void restoreSources();
  });

  onBeforeUnmount(() => {
    persistPlaybackFromDocument();
  });

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
  }

  function handleResumeLastVideo() {
    persistPlaybackFromDocument();
    lastRememberedSecond = -1;
    resumeLastVideo();
  }

  function handleTimeUpdate(event: Event) {
    const video = event.target as HTMLVideoElement | null;
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

  function handlePlaybackEvent(event: Event) {
    const video = event.target as HTMLVideoElement | null;
    if (video) {
      rememberPlayback(video.currentTime, video.duration);
    }
  }

  function persistPlaybackFromDocument() {
    const video = document.querySelector<HTMLVideoElement>(
      ".desktop-video-tool__video-frame video",
    );
    if (video) {
      rememberPlayback(video.currentTime, video.duration);
    }
  }

  function openSourceDialog() {
    setFeedback("");
    isSourceDialogOpen.value = true;
  }
</script>

<template>
  <section class="desktop-video-tool">
    <aside class="desktop-video-tool__library">
      <header class="desktop-video-tool__library-header">
        <div>
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

      <div class="desktop-video-tool__tree desktop-scroll">
        <button
          class="desktop-video-tool__back"
          type="button"
          @click="emit('backToTools')"
        >
          <DesktopUiIcon name="chevron-left" :size="13" />
          工具中心
        </button>

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
          :source="source"
          @remove="removeSource"
          @rescan="rescanSource"
          @select-video="handleVideoSelect"
        />
      </div>
    </aside>

    <main class="desktop-video-tool__main">
      <header v-if="currentVideo" class="desktop-video-tool__now-playing">
        <div>
          <strong>{{ currentVideo.name }}</strong>
          <span
            >{{ currentVideo.sourceTitle }} /
            {{ currentVideo.relativePath }}</span
          >
        </div>
        <button
          v-if="lastPlaybackVideo"
          type="button"
          @click="handleResumeLastVideo"
        >
          继续上次播放
        </button>
      </header>

      <section class="desktop-video-tool__stage">
        <div
          v-if="currentVideo && currentVideoUrl"
          :key="currentVideo.id"
          class="desktop-video-tool__video-frame"
        >
          <VideoPlay
            class="desktop-video-tool__video"
            v-bind="playerOptions"
            @ended="handlePlaybackEvent"
            @pause="handlePlaybackEvent"
            @timeupdate="handleTimeUpdate"
          />
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
    display: grid;
    grid-template-columns: 284px minmax(0, 1fr);
    width: 100%;
    height: 100%;
    min-width: 0;
    min-height: 0;
    background: var(--desktop-bg);
  }

  .desktop-video-tool button {
    border: 0;
    font: inherit;
    cursor: pointer;
  }

  .desktop-video-tool__library {
    display: grid;
    grid-template-rows: auto minmax(0, 1fr);
    min-width: 0;
    min-height: 0;
    border-right: 1px solid var(--desktop-line-strong);
    background: var(--desktop-surface-strong);
  }

  .desktop-video-tool__library-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    min-height: 3.35rem;
    padding: 0 0.72rem 0 0.9rem;
    border-bottom: 1px solid var(--desktop-line);
  }

  .desktop-video-tool__library-header > div:first-child {
    display: grid;
    gap: 0.08rem;
  }

  .desktop-video-tool__library-header strong {
    color: var(--desktop-ink);
    font-size: 0.82rem;
  }

  .desktop-video-tool__library-header span {
    color: var(--desktop-soft);
    font-size: 0.65rem;
  }

  .desktop-video-tool__library-actions {
    display: inline-flex;
    gap: 0.2rem;
  }

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

  .desktop-video-tool__back {
    display: inline-flex;
    align-items: center;
    gap: 0.28rem;
    min-height: 1.85rem;
    margin-bottom: 0.42rem;
    padding: 0 0.45rem;
    border-radius: 7px;
    background: transparent;
    color: var(--desktop-muted);
    font-size: 0.7rem;
  }

  .desktop-video-tool__back:hover {
    color: var(--desktop-accent);
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
  .desktop-video-tool__empty button,
  .desktop-video-tool__now-playing button {
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
    display: grid;
    grid-template-rows: auto minmax(0, 1fr);
    min-width: 0;
    min-height: 0;
    background: color-mix(
      in srgb,
      var(--desktop-bg) 96%,
      var(--desktop-accent)
    );
  }

  .desktop-video-tool__now-playing {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    min-height: 3.35rem;
    padding: 0.45rem 1rem;
    border-bottom: 1px solid var(--desktop-line);
    background: var(--desktop-surface-strong);
  }

  .desktop-video-tool__now-playing > div {
    display: grid;
    gap: 0.12rem;
    min-width: 0;
  }

  .desktop-video-tool__now-playing strong,
  .desktop-video-tool__now-playing span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .desktop-video-tool__now-playing strong {
    color: var(--desktop-ink);
    font-size: 0.82rem;
  }

  .desktop-video-tool__now-playing span {
    color: var(--desktop-muted);
    font-size: 0.66rem;
  }

  .desktop-video-tool__stage {
    display: grid;
    min-width: 0;
    min-height: 0;
  }

  .desktop-video-tool__video-frame {
    width: 100%;
    height: 100%;
    min-height: 0;
    overflow: hidden;
    background: #05070b;
    box-shadow: 0 12px 36px rgba(var(--desktop-shadow), 0.16);
  }

  .desktop-video-tool__video {
    display: block;
    width: 100%;
    height: 100%;
  }

  .desktop-video-tool__video :deep(.d-player-wrap),
  .desktop-video-tool__video :deep(.d-player-video),
  .desktop-video-tool__video :deep(.d-player-video-main) {
    width: 100% !important;
    height: 100% !important;
  }

  .desktop-video-tool__video :deep(.d-player-wrap) {
    background: #05070b;
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
