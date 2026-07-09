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
  import type { DesktopMarkdownThemeId } from "@/composables/useDesktopPreferences";
  import type { DocDetail } from "@/types/docs";
  import DesktopUiIcon from "@/components/ui/DesktopUiIcon.vue";
  import DesktopDocEditor from "./DesktopDocEditor.vue";

  const props = withDefaults(
    defineProps<{
      doc: DocDetail;
      isFavorite?: boolean;
      highlightQuery?: string;
      markdownThemeId?: DesktopMarkdownThemeId;
      restoreScrollTop?: number;
      saveDoc: (absolutePath: string, markdown: string) => Promise<void>;
    }>(),
    {
      isFavorite: false,
      highlightQuery: "",
      markdownThemeId: "atlas",
      restoreScrollTop: 0,
    },
  );

  const emit = defineEmits<{
    scrollTopChange: [top: number];
    toggleFavorite: [];
  }>();

  const bodyScrollRef = useTemplateRef<HTMLElement>("bodyScroll");
  const currentModifiedAt = shallowRef(props.doc.modifiedAt ?? "");
  const toastMessage = shallowRef("");
  const saveFeedbackMessage = shallowRef("");
  const pathCopyHint = shallowRef<"idle" | "copied" | "failed">("idle");
  let stopScrollDiagnostics: (() => void) | null = null;
  let toastTimer: number | null = null;
  let saveFeedbackTimer: number | null = null;
  let pathCopyHintTimer: number | null = null;

  const formattedModifiedAt = computed(() => {
    if (!currentModifiedAt.value) {
      return "未记录编辑时间";
    }

    const date = new Date(currentModifiedAt.value);
    if (Number.isNaN(date.getTime())) {
      return "未记录编辑时间";
    }

    return new Intl.DateTimeFormat("zh-CN", {
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
    }).format(date);
  });

  const fileSystemPath = computed(() => props.doc.absolutePath?.trim() ?? "");

  const hasFileSystemPath = computed(() => fileSystemPath.value.length > 0);

  const pathCopyHintLabel = computed(() => {
    if (pathCopyHint.value === "copied") {
      return "已复制";
    }

    if (pathCopyHint.value === "failed") {
      return "复制失败";
    }

    return "点击复制路径";
  });

  async function copyFileSystemPath() {
    if (!hasFileSystemPath.value) {
      return;
    }

    try {
      await navigator.clipboard.writeText(fileSystemPath.value);
      showPathCopyFeedback("copied");
    } catch {
      showPathCopyFeedback("failed");
    }
  }

  function showPathCopyFeedback(state: "copied" | "failed") {
    pathCopyHint.value = state;
    if (pathCopyHintTimer !== null) {
      window.clearTimeout(pathCopyHintTimer);
    }
    pathCopyHintTimer = window.setTimeout(() => {
      pathCopyHint.value = "idle";
      pathCopyHintTimer = null;
    }, 1800);
  }

  function clearPathCopyFeedback() {
    pathCopyHint.value = "idle";
    if (pathCopyHintTimer !== null) {
      window.clearTimeout(pathCopyHintTimer);
      pathCopyHintTimer = null;
    }
  }

  watch(
    () =>
      [props.doc.slug, props.restoreScrollTop, props.highlightQuery] as const,
    async ([slug, restoreScrollTop, highlightQuery]) => {
      if (!slug) {
        return;
      }

      await nextTick();

      const scrollElement = bodyScrollRef.value;
      if (!scrollElement) {
        return;
      }

      const nextTop = highlightQuery.trim() ? 0 : Math.max(0, restoreScrollTop);
      if (Math.abs(scrollElement.scrollTop - nextTop) > 1) {
        scrollElement.scrollTop = nextTop;
      }
      emit("scrollTopChange", scrollElement.scrollTop);
    },
    { immediate: true },
  );

  watch(
    () => props.doc.modifiedAt ?? "",
    (nextValue) => {
      currentModifiedAt.value = nextValue;
    },
    { immediate: true },
  );

  watch(
    () => props.doc.slug,
    () => {
      clearToast();
      clearSaveFeedback();
      clearPathCopyFeedback();
    },
  );

  function handleBodyScroll(event: Event) {
    const target = event.target;
    if (!(target instanceof HTMLElement)) {
      return;
    }

    emit("scrollTopChange", target.scrollTop);
  }

  function releaseEditorFocus() {
    const activeElement = document.activeElement;
    if (
      activeElement instanceof HTMLElement &&
      activeElement.closest(".desktop-doc-editor__editor")
    ) {
      activeElement.blur();
    }
  }

  function handleDocSaved(payload: {
    mode: "auto" | "manual";
    modifiedAt: string;
  }) {
    currentModifiedAt.value = payload.modifiedAt;
    if (payload.mode === "manual") {
      showSaveFeedback("已保存");
    }
  }

  function handleCopied(message: string) {
    showToast(message);
  }

  function showSaveFeedback(message: string) {
    saveFeedbackMessage.value = message;
    if (saveFeedbackTimer !== null) {
      window.clearTimeout(saveFeedbackTimer);
    }
    saveFeedbackTimer = window.setTimeout(() => {
      saveFeedbackMessage.value = "";
      saveFeedbackTimer = null;
    }, 1800);
  }

  function clearSaveFeedback() {
    saveFeedbackMessage.value = "";
    if (saveFeedbackTimer !== null) {
      window.clearTimeout(saveFeedbackTimer);
      saveFeedbackTimer = null;
    }
  }

  function showToast(message: string) {
    toastMessage.value = message;
    if (toastTimer !== null) {
      window.clearTimeout(toastTimer);
    }
    toastTimer = window.setTimeout(() => {
      clearToast();
    }, 1800);
  }

  function clearToast() {
    toastMessage.value = "";
    if (toastTimer !== null) {
      window.clearTimeout(toastTimer);
      toastTimer = null;
    }
  }

  function startScrollDiagnostics() {
    if (!import.meta.env.DEV) {
      return;
    }

    const scrollElement = bodyScrollRef.value;
    if (!scrollElement) {
      return;
    }

    let lastInteraction = "initial";
    let lastTop = scrollElement.scrollTop;
    const interactionListeners: Array<[keyof WindowEventMap, EventListener]> = [
      ["wheel", () => (lastInteraction = "wheel")],
      ["touchstart", () => (lastInteraction = "touchstart")],
      ["keydown", () => (lastInteraction = "keydown")],
      ["mousedown", () => (lastInteraction = "mousedown")],
    ];

    interactionListeners.forEach(([type, listener]) => {
      window.addEventListener(type, listener, { passive: true });
    });

    const originalScrollTo = scrollElement.scrollTo.bind(scrollElement);
    scrollElement.scrollTo = ((
      ...args: Parameters<HTMLElement["scrollTo"]>
    ) => {
      console.debug("[DocsAtlas][scroll] scrollTo called", {
        args,
        lastInteraction,
        slug: props.doc.slug,
        from: scrollElement.scrollTop,
      });
      return originalScrollTo(...args);
    }) as HTMLElement["scrollTo"];

    const handleScroll = () => {
      const nextTop = scrollElement.scrollTop;
      console.debug("[DocsAtlas][scroll] scrollTop changed", {
        delta: nextTop - lastTop,
        lastInteraction,
        nextTop,
        slug: props.doc.slug,
      });
      lastTop = nextTop;
      lastInteraction = "scroll";
    };

    scrollElement.addEventListener("scroll", handleScroll, { passive: true });

    stopScrollDiagnostics = () => {
      scrollElement.removeEventListener("scroll", handleScroll);
      scrollElement.scrollTo = originalScrollTo as HTMLElement["scrollTo"];
      interactionListeners.forEach(([type, listener]) => {
        window.removeEventListener(type, listener);
      });
      stopScrollDiagnostics = null;
    };

    console.debug("[DocsAtlas][scroll] diagnostics attached", {
      slug: props.doc.slug,
    });
  }

  onMounted(() => {
    startScrollDiagnostics();
  });

  onBeforeUnmount(() => {
    stopScrollDiagnostics?.();
    clearToast();
    clearSaveFeedback();
    clearPathCopyFeedback();
  });
</script>

<template>
  <article class="doc-content">
    <div v-if="toastMessage" class="doc-content__toast">
      {{ toastMessage }}
    </div>

    <div
      class="doc-content__panel"
      :data-markdown-theme="props.markdownThemeId"
    >
      <header class="doc-content__header">
        <div class="doc-content__header-main">
          <div class="doc-content__header-copy">
            <div class="doc-content__header-row">
              <p class="doc-content__breadcrumb">
                {{
                  doc.sectionTitle
                    ? `${doc.sourceLabel} / ${doc.sectionTitle}`
                    : doc.sourceLabel
                }}
              </p>

              <button
                :class="[
                  'doc-content__path-copy',
                  {
                    'doc-content__path-copy--copied':
                      pathCopyHint === 'copied',
                    'doc-content__path-copy--failed':
                      pathCopyHint === 'failed',
                  },
                ]"
                type="button"
                :disabled="!hasFileSystemPath"
                :aria-label="
                  hasFileSystemPath
                    ? `复制文件路径：${fileSystemPath}`
                    : '未记录文件路径'
                "
                @click="copyFileSystemPath"
              >
                <DesktopUiIcon name="copy" :size="14" :stroke-width="2" />
                <span class="doc-content__path-copy-hint">{{
                  pathCopyHintLabel
                }}</span>
              </button>
            </div>

            <div class="doc-content__meta">
              <span class="doc-content__meta-label">最后编辑</span>
              <span class="doc-content__meta-sep" aria-hidden="true">·</span>
              <span class="doc-content__meta-value">{{
                formattedModifiedAt
              }}</span>
              <span
                v-if="saveFeedbackMessage"
                class="doc-content__save-feedback"
              >
                {{ saveFeedbackMessage }}
              </span>
            </div>
          </div>

          <button
            :class="[
              'doc-content__favorite',
              { 'doc-content__favorite--active': props.isFavorite },
            ]"
            type="button"
            @click="emit('toggleFavorite')"
          >
            <DesktopUiIcon name="bookmark" :size="14" />
            <span>{{ props.isFavorite ? "已收藏" : "收藏" }}</span>
          </button>
        </div>
      </header>

      <div
        ref="bodyScroll"
        id="desktop-doc-scroll"
        class="doc-content__body-scroll desktop-scroll"
        @touchstart.passive="releaseEditorFocus"
        @wheel.passive="releaseEditorFocus"
        @scroll="handleBodyScroll"
      >
        <DesktopDocEditor
          :doc="doc"
          :highlight-query="props.highlightQuery"
          :markdown-theme-id="props.markdownThemeId"
          :save-doc="props.saveDoc"
          @copied="handleCopied"
          @saved="handleDocSaved"
        />
      </div>
    </div>
  </article>
</template>

<style scoped>
  .doc-content {
    min-width: 0;
    min-height: 0;
    height: 100%;
    background: var(--desktop-surface-strong);
  }

  .doc-content__toast {
    position: fixed;
    top: 5rem;
    left: 50%;
    transform: translate(-50%, -50%);
    z-index: 40;
    width: fit-content;
    padding: 0.62rem 0.84rem;
    border: 1px solid var(--desktop-line);
    border-radius: var(--desktop-radius-sm);
    background: var(--desktop-surface-strong);
    box-shadow: 0 4px 16px rgba(var(--desktop-shadow), 0.1);
    color: var(--desktop-ink);
    font-size: 0.78rem;
    font-weight: 600;
    line-height: 1.3;
    pointer-events: none;
  }

  .doc-content__panel {
    display: flex;
    flex-direction: column;
    min-height: 0;
    height: 100%;
    border: 0;
    border-radius: 0;
    background: var(--desktop-surface-strong);
    box-shadow: none;
    overflow: hidden;
  }

  .doc-content__panel[data-markdown-theme="github"] {
    background: transparent;
  }

  .doc-content__header {
    flex-shrink: 0;
    z-index: 4;
    padding: 0.72rem 1.25rem 0.68rem;
    border-bottom: 1px solid var(--desktop-line-subtle, var(--desktop-line));
    background: var(--desktop-surface-strong);
    border-radius: 0;
  }

  .doc-content__header-main {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 0.85rem;
  }

  .doc-content__header-copy {
    display: grid;
    gap: 0.28rem;
    min-width: 0;
    flex: 1 1 auto;
  }

  .doc-content__header-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
    overflow: visible;
  }

  .doc-content__body-scroll {
    min-height: 0;
    flex: 1 1 auto;
    overflow-y: auto;
    overflow-anchor: none;
    background: var(--desktop-surface-strong);
  }

  .doc-content__breadcrumb {
    margin: 0;
    min-width: 0;
    flex: 1 1 auto;
    color: var(--desktop-soft);
    font-size: 0.75rem;
    font-weight: 500;
    line-height: 1.35;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .doc-content__path-copy {
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 1.88rem;
    height: 1.88rem;
    padding: 0;
    border: 1px solid
      color-mix(in srgb, var(--desktop-line) 82%, rgba(var(--desktop-accent-rgb), 0.18));
    border-radius: 0.46rem;
    background: color-mix(
      in srgb,
      var(--desktop-surface-strong) 90%,
      rgba(var(--desktop-accent-rgb), 0.04)
    );
    color: var(--desktop-muted);
    cursor: pointer;
    flex-shrink: 0;
    overflow: visible;
    transition:
      border-color 0.14s ease,
      background-color 0.14s ease,
      color 0.14s ease,
      box-shadow 0.14s ease;
  }

  .doc-content__path-copy :deep(.desktop-ui-icon) {
    display: block;
    flex: none;
  }

  .doc-content__path-copy:hover:not(:disabled),
  .doc-content__path-copy:focus-visible:not(:disabled) {
    border-color: rgba(var(--desktop-accent-rgb), 0.28);
    background: rgba(var(--desktop-accent-rgb), 0.07);
    color: var(--desktop-accent);
    box-shadow: 0 1px 0 rgba(var(--desktop-shadow), 0.04);
  }

  .doc-content__path-copy:disabled {
    border-color: var(--desktop-line);
    background: transparent;
    color: var(--desktop-soft);
    opacity: 0.5;
    cursor: not-allowed;
  }

  .doc-content__path-copy-hint {
    position: absolute;
    top: calc(100% + 0.38rem);
    left: 50%;
    z-index: 6;
    padding: 0.28rem 0.48rem;
    border: 1px solid var(--desktop-line);
    border-radius: 0.38rem;
    background: var(--desktop-surface-strong);
    box-shadow: 0 6px 18px rgba(var(--desktop-shadow), 0.1);
    color: var(--desktop-muted);
    font-size: 0.68rem;
    font-weight: 500;
    line-height: 1.2;
    white-space: nowrap;
    opacity: 0;
    visibility: hidden;
    transform: translate(-50%, -2px);
    pointer-events: none;
    transition:
      opacity 0.14s ease,
      transform 0.14s ease,
      visibility 0.14s ease;
  }

  .doc-content__path-copy:hover:not(:disabled) .doc-content__path-copy-hint,
  .doc-content__path-copy:focus-visible:not(:disabled)
    .doc-content__path-copy-hint,
  .doc-content__path-copy--copied .doc-content__path-copy-hint,
  .doc-content__path-copy--failed .doc-content__path-copy-hint {
    opacity: 1;
    visibility: visible;
    transform: translate(-50%, 0);
  }

  .doc-content__path-copy--copied .doc-content__path-copy-hint {
    color: var(--desktop-accent);
    border-color: rgba(var(--desktop-accent-rgb), 0.22);
  }

  .doc-content__path-copy--failed .doc-content__path-copy-hint {
    color: rgb(210, 69, 69);
    border-color: rgba(210, 69, 69, 0.22);
  }

  .doc-content__meta {
    display: inline-flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.38rem;
    margin: 0;
    color: var(--desktop-soft);
    font-size: 0.72rem;
    line-height: 1.4;
  }

  .doc-content__meta-label,
  .doc-content__meta-sep {
    color: var(--desktop-soft);
  }

  .doc-content__meta-value {
    color: var(--desktop-muted);
    font-weight: 500;
  }

  .doc-content__save-feedback {
    display: inline-flex;
    align-items: center;
    min-height: 1.42rem;
    padding: 0.12rem 0.48rem;
    border: 1px solid rgba(var(--desktop-accent-rgb), 0.12);
    border-radius: 999px;
    background: transparent;
    color: var(--desktop-accent);
    font-size: 0.72rem;
    font-weight: 600;
    line-height: 1;
  }

  .doc-content__save-feedback-enter-active,
  .doc-content__save-feedback-leave-active {
    transition:
      opacity 0.18s ease,
      transform 0.18s ease;
  }

  .doc-content__save-feedback-enter-from,
  .doc-content__save-feedback-leave-to {
    opacity: 0;
    transform: translateY(-2px);
  }

  .doc-content__favorite {
    display: inline-flex;
    align-items: center;
    gap: 0.32rem;
    min-height: 1.62rem;
    padding: 0 0.62rem;
    border: 0;
    border-radius: var(--desktop-radius-sm);
    background: transparent;
    color: var(--desktop-muted);
    font: inherit;
    font-size: 0.75rem;
    font-weight: 500;
    cursor: pointer;
    transition:
      background-color 0.12s ease,
      color 0.12s ease;
    flex-shrink: 0;
  }

  .doc-content__favorite:hover,
  .doc-content__favorite--active {
    background: rgba(var(--desktop-accent-rgb), 0.06);
    color: var(--desktop-accent);
  }

  @media (max-width: 1180px) {
    .doc-content__header {
      padding: 0.68rem 0.82rem 0.64rem;
    }
  }
</style>
