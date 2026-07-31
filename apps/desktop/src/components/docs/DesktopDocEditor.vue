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
  import { languages as codeMirrorLanguages } from "@codemirror/language-data";
  import { githubDark, githubLight } from "@uiw/codemirror-theme-github";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { remarkStringifyOptionsCtx } from "@milkdown/kit/core";
  import { Crepe } from "@milkdown/crepe";
  import { replaceAll } from "@milkdown/kit/utils";
  import mermaid from "mermaid";
  import DesktopDocImagePreview from "./DesktopDocImagePreview.vue";
  import type { DesktopMarkdownThemeId } from "@/composables/useDesktopPreferences";
  import type { DocDetail } from "@/types/docs";

  type PreviewImage = {
    alt: string;
    src: string;
    title: string;
  };

  type EditorPalette = {
    panelBackground: string;
    surfaceLow: string;
    codeBackground: string;
    codeBorder: string;
    codeText: string;
    inlineCodeBackground: string;
    inlineCodeColor: string;
    quoteBackground: string;
    tableHeadBackground: string;
    tableZebraBackground: string;
    mutedLine: string;
    softFill: string;
    hoverFill: string;
    selectionFill: string;
  };

  const props = withDefaults(
    defineProps<{
      doc: DocDetail;
      highlightQuery?: string;
      markdownThemeId?: DesktopMarkdownThemeId;
      saveDoc: (absolutePath: string, markdown: string) => Promise<void>;
    }>(),
    {
      highlightQuery: "",
      markdownThemeId: "atlas",
    },
  );

  const emit = defineEmits<{
    copied: [message: string];
    saved: [payload: { mode: "auto" | "manual"; modifiedAt: string }];
  }>();

  const hostRef = useTemplateRef<HTMLDivElement>("host");
  const editorRef = shallowRef<Crepe | null>(null);
  const draftMarkdown = shallowRef(props.doc.markdown ?? "");
  const savedMarkdown = shallowRef(props.doc.markdown ?? "");
  const isSaving = shallowRef(false);
  const saveError = shallowRef("");
  const previewImages = shallowRef<PreviewImage[]>([]);
  const activePreviewImageIndex = shallowRef(-1);
  const currentDocAbsolutePath = shallowRef(
    props.doc.absolutePath?.trim() ?? "",
  );
  const currentDocModifiedAt = shallowRef(props.doc.modifiedAt ?? "");
  const resolvedThemeName = shallowRef<"dark" | "light">("light");
  const isEditorMounted = shallowRef(false);
  const lastHydratedSlug = shallowRef(props.doc.slug);
  const mermaidPreviewCache = new Map<string, Promise<string> | string>();
  const mermaidPreviewModeByKey = new Map<string, boolean>();
  let debugResizeObserver: ResizeObserver | null = null;
  let imageMutationObserver: MutationObserver | null = null;
  let themeObserver: MutationObserver | null = null;
  let previewEnhancementTimer: number | null = null;
  let outlineSyncTimer: number | null = null;
  let mermaidSequence = 0;
  let pendingHeadingSync = false;
  let pendingImageSync = false;
  let pendingOutlineSync = false;
  let removeImagePreviewListeners: (() => void) | null = null;
  let isUnmounted = false;

  const hasEditableSource = computed(() =>
    Boolean(props.doc.absolutePath && typeof props.doc.markdown === "string"),
  );
  const isDirty = computed(() => draftMarkdown.value !== savedMarkdown.value);

  watch(
    () => props.doc.slug,
    async (nextSlug, previousSlug) => {
      if (!previousSlug || nextSlug === previousSlug) {
        return;
      }

      await persistDraft({
        absolutePath: currentDocAbsolutePath.value,
        markdown: draftMarkdown.value,
        mode: "auto",
      });
    },
    { flush: "sync" },
  );

  watch(
    () =>
      [
        props.doc.slug,
        props.doc.markdown ?? "",
        props.doc.absolutePath?.trim() ?? "",
        props.doc.modifiedAt ?? "",
      ] as const,
    async ([slug, markdown, absolutePath, modifiedAt]) => {
      currentDocAbsolutePath.value = absolutePath;
      currentDocModifiedAt.value = modifiedAt;
      saveError.value = "";

      const editor = editorRef.value;
      if (!editor) {
        return;
      }

      const shouldHydrateContent = lastHydratedSlug.value !== slug;
      lastHydratedSlug.value = slug;

      if (shouldHydrateContent) {
        draftMarkdown.value = markdown;
        savedMarkdown.value = markdown;

        await nextTick();

        if (editor.getMarkdown() !== markdown) {
          editor.editor.action(replaceAll(markdown, true));
        }
      }

      editor.setReadonly(!hasEditableSource.value);
      schedulePreviewEnhancements({
        syncHeadings: true,
        syncImages: true,
        syncOutline: true,
      });
    },
    { immediate: true },
  );

  watch(
    () => props.markdownThemeId,
    () => {
      applyEditorTheme();
    },
  );

  watch(hasEditableSource, (nextValue) => {
    editorRef.value?.setReadonly(!nextValue);
  });

  onMounted(async () => {
    isUnmounted = false;
    await nextTick();
    syncTheme();
    await createEditor(draftMarkdown.value);
    bindThemeObserver();
    window.addEventListener("keydown", handleWindowKeydown, true);
  });

  onBeforeUnmount(() => {
    isUnmounted = true;
    window.removeEventListener("keydown", handleWindowKeydown, true);
    removeImagePreviewListeners?.();
    removeImagePreviewListeners = null;
    themeObserver?.disconnect();
    themeObserver = null;
    imageMutationObserver?.disconnect();
    imageMutationObserver = null;
    debugResizeObserver?.disconnect();
    debugResizeObserver = null;
    mermaidPreviewCache.clear();
    if (previewEnhancementTimer !== null) {
      window.clearTimeout(previewEnhancementTimer);
      previewEnhancementTimer = null;
    }
    if (outlineSyncTimer !== null) {
      window.clearTimeout(outlineSyncTimer);
      outlineSyncTimer = null;
    }
    void persistDraft({
      absolutePath: currentDocAbsolutePath.value,
      markdown: draftMarkdown.value,
      mode: "auto",
    });
    void destroyEditor();
  });

  async function createEditor(defaultValue: string) {
    if (!hostRef.value) {
      return;
    }

    hostRef.value.innerHTML = "";

    const nextEditor = new Crepe({
      root: hostRef.value,
      defaultValue,
      features: {
        [Crepe.Feature.Toolbar]: false,
        [Crepe.Feature.TopBar]: false,
        [Crepe.Feature.BlockEdit]: false,
        [Crepe.Feature.CodeMirror]: true,
        [Crepe.Feature.Latex]: false,
      },
      featureConfigs: {
        [Crepe.Feature.CodeMirror]: {
          copyText: "复制代码",
          languages: codeMirrorLanguages,
          noResultText: "未找到语言",
          onCopy: () => {
            emit("copied", "代码已复制");
          },
          previewToggleText: (previewOnlyMode) =>
            previewOnlyMode ? "编辑" : "预览",
          previewOnlyByDefault: true,
          renderPreview: renderCodePreview,
          searchPlaceholder: "搜索语言",
          theme: resolvedThemeName.value === "dark" ? githubDark : githubLight,
        },
        [Crepe.Feature.Placeholder]: {
          text: "开始编写 Markdown 文档",
          mode: "doc",
        },
        [Crepe.Feature.ImageBlock]: {
          proxyDomURL(url: string) {
            return resolvePreviewAssetSource(url, currentDocAbsolutePath.value);
          },
        },
      },
    });

    nextEditor.editor.config((ctx) => {
      ctx.update(remarkStringifyOptionsCtx, (prev) => ({
        ...prev,
        bullet: "-",
        bulletOther: "*",
      }));
    });

    nextEditor.on((listener) => {
      listener.markdownUpdated((_, markdown) => {
        if (markdown === draftMarkdown.value) {
          return;
        }

        draftMarkdown.value = markdown;
        if (saveError.value) {
          saveError.value = "";
        }
        scheduleOutlineSync();
      });

      listener.mounted(() => {
        isEditorMounted.value = true;
        schedulePreviewEnhancements({
          syncHeadings: true,
          syncImages: true,
          syncOutline: true,
        });
      });
    });

    editorRef.value = nextEditor;
    const restoreIntersectionObserver = installStableCodeBlockObserver();
    try {
      await nextEditor.create();
    } finally {
      restoreIntersectionObserver();
    }
    nextEditor.setReadonly(!hasEditableSource.value);
    applyEditorTheme();
    bindImagePreviewListeners();
  }

  function installStableCodeBlockObserver() {
    if (typeof window === "undefined" || !window.IntersectionObserver) {
      return () => {};
    }

    // Milkdown tears down off-screen code blocks, which changes Mermaid/code block
    // height and makes the document scroll by itself. Keep editor code blocks mounted.
    const OriginalIntersectionObserver = window.IntersectionObserver;

    class StableCodeBlockObserver {
      readonly root: Element | Document | null = null;
      readonly rootMargin = "200px";
      readonly thresholds = [0];
      private readonly callback: IntersectionObserverCallback;
      private readonly observedTargets = new Set<Element>();

      constructor(callback: IntersectionObserverCallback) {
        this.callback = callback;
      }

      observe(target: Element) {
        this.observedTargets.add(target);
        window.requestAnimationFrame(() => {
          if (!this.observedTargets.has(target)) {
            return;
          }

          this.callback(
            [
              {
                boundingClientRect: target.getBoundingClientRect(),
                intersectionRatio: 1,
                intersectionRect: target.getBoundingClientRect(),
                isIntersecting: true,
                rootBounds: null,
                target,
                time: performance.now(),
              } as IntersectionObserverEntry,
            ],
            this as unknown as IntersectionObserver,
          );
        });
      }

      unobserve(target: Element) {
        this.observedTargets.delete(target);
      }

      disconnect() {
        this.observedTargets.clear();
      }

      takeRecords() {
        return [];
      }
    }

    window.IntersectionObserver =
      StableCodeBlockObserver as unknown as typeof IntersectionObserver;

    return () => {
      window.IntersectionObserver = OriginalIntersectionObserver;
    };
  }

  async function destroyEditor() {
    isEditorMounted.value = false;
    removeImagePreviewListeners?.();
    removeImagePreviewListeners = null;
    imageMutationObserver?.disconnect();
    imageMutationObserver = null;
    const editor = editorRef.value;
    editorRef.value = null;
    if (editor) {
      await editor.destroy();
    }
    if (hostRef.value) {
      hostRef.value.innerHTML = "";
    }
  }

  async function rebuildEditor() {
    const currentMarkdown = draftMarkdown.value;
    await destroyEditor();
    await nextTick();
    await createEditor(currentMarkdown);
  }

  function bindThemeObserver() {
    if (typeof window === "undefined") {
      return;
    }

    themeObserver = new MutationObserver((mutations) => {
      let shouldRebuild = false;
      let shouldApplyTheme = false;

      for (const mutation of mutations) {
        const attributeName = mutation.attributeName;
        if (
          attributeName === "data-theme" ||
          attributeName === "data-theme-mode"
        ) {
          const previousTheme = resolvedThemeName.value;
          syncTheme();
          shouldApplyTheme = true;
          if (previousTheme !== resolvedThemeName.value) {
            shouldRebuild = true;
          }
          continue;
        }

        if (attributeName === "data-theme-accent") {
          shouldApplyTheme = true;
        }
      }

      if (shouldRebuild) {
        void rebuildEditor();
        return;
      }

      if (shouldApplyTheme) {
        applyEditorTheme();
      }
    });

    themeObserver.observe(document.documentElement, {
      attributeFilter: ["data-theme", "data-theme-mode", "data-theme-accent"],
      attributes: true,
    });
  }

  function syncTheme() {
    resolvedThemeName.value =
      document.documentElement.dataset.theme === "dark" ? "dark" : "light";
  }

  function applyEditorTheme() {
    const host = hostRef.value;
    if (!host || typeof window === "undefined") {
      return;
    }

    const rootStyles = window.getComputedStyle(document.documentElement);
    const palette = resolveEditorPalette(
      resolvedThemeName.value,
      props.markdownThemeId,
    );
    const tokenEntries = [
      ["--crepe-color-background", palette.panelBackground],
      ["--crepe-color-surface", palette.surfaceLow],
      ["--crepe-color-surface-low", palette.softFill],
      [
        "--crepe-color-on-background",
        readRootToken("--desktop-ink", rootStyles),
      ],
      ["--crepe-color-on-surface", readRootToken("--desktop-ink", rootStyles)],
      [
        "--crepe-color-on-surface-variant",
        readRootToken("--desktop-muted", rootStyles),
      ],
      ["--crepe-color-outline", palette.mutedLine],
      ["--crepe-color-primary", readRootToken("--desktop-accent", rootStyles)],
      ["--crepe-color-secondary", palette.hoverFill],
      [
        "--crepe-color-on-secondary",
        readRootToken("--desktop-ink", rootStyles),
      ],
      ["--crepe-color-inverse", readRootToken("--desktop-soft", rootStyles)],
      [
        "--crepe-color-on-inverse",
        readRootToken("--desktop-surface-strong", rootStyles),
      ],
      ["--crepe-color-inline-code", palette.inlineCodeColor],
      ["--crepe-color-error", "rgb(210, 69, 69)"],
      ["--crepe-color-hover", palette.hoverFill],
      ["--crepe-color-selected", palette.selectionFill],
      ["--crepe-color-inline-area", palette.softFill],
      ["--crepe-font-default", "var(--desktop-font-sans)"],
      ["--crepe-font-title", "var(--desktop-font-sans)"],
      [
        "--crepe-font-code",
        "'SFMono-Regular', 'JetBrains Mono', 'Fira Code', monospace",
      ],
      ["--docs-atlas-code-bg", palette.codeBackground],
      ["--docs-atlas-code-border", palette.codeBorder],
      ["--docs-atlas-code-text", palette.codeText],
      ["--docs-atlas-inline-code-bg", palette.inlineCodeBackground],
      ["--docs-atlas-inline-code-color", palette.inlineCodeColor],
      ["--docs-atlas-quote-bg", palette.quoteBackground],
      ["--docs-atlas-table-head-bg", palette.tableHeadBackground],
      ["--docs-atlas-table-zebra-bg", palette.tableZebraBackground],
      ["--docs-atlas-soft-fill", palette.softFill],
      ["--docs-atlas-muted-line", palette.mutedLine],
    ] as const;

    for (const [name, value] of tokenEntries) {
      host.style.setProperty(name, value);
    }

    host.dataset.themeMode = resolvedThemeName.value;
  }

  function schedulePreviewEnhancements(options?: {
    syncHeadings?: boolean;
    syncImages?: boolean;
    syncOutline?: boolean;
  }) {
    if (typeof window === "undefined") {
      return;
    }

    pendingHeadingSync = pendingHeadingSync || Boolean(options?.syncHeadings);
    pendingImageSync = pendingImageSync || Boolean(options?.syncImages);
    pendingOutlineSync = pendingOutlineSync || Boolean(options?.syncOutline);

    if (previewEnhancementTimer !== null) {
      window.clearTimeout(previewEnhancementTimer);
    }

    previewEnhancementTimer = window.setTimeout(() => {
      if (isUnmounted) {
        previewEnhancementTimer = null;
        return;
      }
      previewEnhancementTimer = null;
      applyPreviewEnhancements();
    }, 80);
  }

  function scheduleOutlineSync() {
    if (typeof window === "undefined") {
      return;
    }

    if (outlineSyncTimer !== null) {
      window.clearTimeout(outlineSyncTimer);
    }

    outlineSyncTimer = window.setTimeout(() => {
      if (isUnmounted) {
        outlineSyncTimer = null;
        return;
      }
      outlineSyncTimer = null;
      notifyOutlineSync();
    }, 180);
  }

  function applyPreviewEnhancements() {
    const host = hostRef.value;
    if (!host) {
      return;
    }

    const shouldSyncHeadings = pendingHeadingSync;
    const shouldSyncImages = pendingImageSync;
    const shouldSyncOutline = pendingOutlineSync;

    pendingHeadingSync = false;
    pendingImageSync = false;
    pendingOutlineSync = false;

    if (shouldSyncHeadings) {
      syncHeadingAnchors(host);
    }
    if (shouldSyncImages) {
      collectPreviewImages(host);
      bindImageMutationObserver(host);
    }
    if (shouldSyncOutline) {
      notifyOutlineSync();
    }
    syncDebugObservers(host);
  }

  function syncDebugObservers(root: HTMLElement) {
    if (!import.meta.env.DEV || typeof ResizeObserver === "undefined") {
      return;
    }

    debugResizeObserver?.disconnect();

    const targets = root.querySelectorAll<HTMLElement>(
      ".milkdown-code-block, .milkdown-code-block-placeholder, .preview, .desktop-doc-editor__diagram-preview",
    );

    debugResizeObserver = new ResizeObserver((entries) => {
      entries.forEach((entry) => {
        const target = entry.target as HTMLElement;
        const text =
          target.textContent?.replace(/\s+/g, " ").trim().slice(0, 48) ?? "";
        console.debug("[DocsAtlas][code-block] resize", {
          className: target.className,
          height: Math.round(entry.contentRect.height),
          slug: props.doc.slug,
          text,
        });
      });
    });

    targets.forEach((target) => {
      debugResizeObserver?.observe(target);
    });

    const editorRoot = root.querySelector(".ProseMirror");
    console.debug("[DocsAtlas][editor] preview sync", {
      codeBlockCount: root.querySelectorAll(".milkdown-code-block").length,
      headingCount: editorRoot?.querySelectorAll("h2, h3").length ?? 0,
      slug: props.doc.slug,
    });
  }

  function syncHeadingAnchors(root: HTMLElement) {
    const headingIds = props.doc.headings.map((heading) => heading.id);
    if (!headingIds.length) {
      return;
    }

    const headingElements = root.querySelectorAll<HTMLElement>(
      ".ProseMirror h2, .ProseMirror h3",
    );

    headingElements.forEach((element, index) => {
      const nextId = headingIds[index];
      if (!nextId) {
        element.removeAttribute("id");
        delete element.dataset.docHeadingId;
        delete element.dataset.docHeadingIndex;
        return;
      }

      element.id = nextId;
      element.dataset.docHeadingId = nextId;
      element.dataset.docHeadingIndex = String(index);
    });
  }

  function notifyOutlineSync() {
    if (typeof window === "undefined") {
      return;
    }

    if (import.meta.env.DEV) {
      console.debug("[DocsAtlas][outline] doc-rendered", {
        slug: props.doc.slug,
      });
    }
    window.dispatchEvent(new CustomEvent("docs-atlas:doc-rendered"));
  }

  function collectPreviewImages(root: HTMLElement) {
    const nextImages = Array.from(
      root.querySelectorAll<HTMLImageElement>("img[src]"),
    ).map((image, index) => {
      const openPreview = (event: Event) => {
        event.preventDefault();
        event.stopPropagation();
        activePreviewImageIndex.value = index;
      };

      image.dataset.previewIndex = String(index);
      image.dataset.docPreviewImage = "true";
      image.draggable = false;
      image.style.pointerEvents = "auto";
      image.onmousedown = openPreview;
      image.onclick = openPreview;
      image.onpointerdown = openPreview;
      image.onpointerup = openPreview;

      const imageBlock = image.closest(
        ".milkdown-image-block",
      ) as HTMLElement | null;
      if (imageBlock) {
        imageBlock.draggable = false;
        imageBlock.style.pointerEvents = "auto";
        imageBlock.onmousedown = openPreview;
        imageBlock.onclick = openPreview;
        imageBlock.onpointerdown = openPreview;
        imageBlock.onpointerup = openPreview;
      }

      const imageWrapper = image.closest(
        ".image-wrapper",
      ) as HTMLElement | null;
      if (imageWrapper) {
        imageWrapper.style.pointerEvents = "auto";
        imageWrapper.onmousedown = openPreview;
        imageWrapper.onclick = openPreview;
        imageWrapper.onpointerdown = openPreview;
        imageWrapper.onpointerup = openPreview;

        const existingHitbox = imageWrapper.querySelector<HTMLElement>(
          ".docs-atlas-image-preview-hitbox",
        );
        existingHitbox?.remove();

        const hitbox = document.createElement("button");
        hitbox.type = "button";
        hitbox.className = "docs-atlas-image-preview-hitbox";
        hitbox.setAttribute("aria-label", "预览图片");
        hitbox.onmousedown = openPreview;
        hitbox.onclick = openPreview;
        hitbox.onpointerdown = openPreview;
        hitbox.onpointerup = openPreview;
        imageWrapper.appendChild(hitbox);
      }

      return {
        alt: image.getAttribute("alt")?.trim() ?? "",
        src: image.currentSrc?.trim() || image.src?.trim() || "",
        title: image.getAttribute("title")?.trim() ?? "",
      };
    });

    previewImages.value = nextImages;
    if (activePreviewImageIndex.value >= nextImages.length) {
      activePreviewImageIndex.value = -1;
    }
  }

  function bindImageMutationObserver(root: HTMLElement) {
    if (typeof MutationObserver === "undefined") {
      return;
    }

    imageMutationObserver?.disconnect();

    imageMutationObserver = new MutationObserver((mutations) => {
      let shouldRefreshImages = false;

      for (const mutation of mutations) {
        if (mutation.type === "childList") {
          const hasImageNode =
            Array.from(mutation.addedNodes).some(
              (node) =>
                node instanceof HTMLImageElement ||
                (node instanceof HTMLElement && node.querySelector("img[src]")),
            ) ||
            Array.from(mutation.removedNodes).some(
              (node) =>
                node instanceof HTMLImageElement ||
                (node instanceof HTMLElement && node.querySelector("img[src]")),
            );

          if (hasImageNode) {
            shouldRefreshImages = true;
            break;
          }
        }

        if (
          mutation.type === "attributes" &&
          mutation.target instanceof HTMLImageElement
        ) {
          shouldRefreshImages = true;
          break;
        }
      }

      if (!shouldRefreshImages) {
        return;
      }

      collectPreviewImages(root);
    });

    imageMutationObserver.observe(root, {
      subtree: true,
      childList: true,
      attributes: true,
      attributeFilter: ["src"],
    });
  }

  function bindImagePreviewListeners() {
    removeImagePreviewListeners?.();
    removeImagePreviewListeners = null;

    const host = hostRef.value;
    if (!host) {
      return;
    }

    const openImagePreviewFromEvent = (event: Event) => {
      const target = event.target;
      if (!(target instanceof HTMLElement)) {
        return;
      }

      const imageBlock = target.closest(".milkdown-image-block");
      const image =
        target.closest(
          'img[data-doc-preview-image="true"][data-preview-index]',
        ) ??
        imageBlock?.querySelector(
          'img[data-doc-preview-image="true"][data-preview-index]',
        );
      if (!(image instanceof HTMLImageElement)) {
        return;
      }

      const index = Number.parseInt(image.dataset.previewIndex ?? "-1", 10);
      if (Number.isNaN(index) || index < 0) {
        return;
      }

      event.preventDefault();
      event.stopPropagation();
      activePreviewImageIndex.value = index;
    };

    host.addEventListener("pointerdown", openImagePreviewFromEvent, true);
    host.addEventListener("mousedown", openImagePreviewFromEvent, true);
    host.addEventListener("click", openImagePreviewFromEvent, true);

    removeImagePreviewListeners = () => {
      host.removeEventListener("pointerdown", openImagePreviewFromEvent, true);
      host.removeEventListener("mousedown", openImagePreviewFromEvent, true);
      host.removeEventListener("click", openImagePreviewFromEvent, true);
    };
  }

  function resolvePreviewAssetSource(src: string, absolutePath: string) {
    if (
      !src ||
      src.startsWith("#") ||
      /^data:/i.test(src) ||
      /^asset:/i.test(src) ||
      /^https?:\/\//i.test(src) ||
      /^http:\/\/asset\.localhost/i.test(src) ||
      /^\/\//.test(src)
    ) {
      return src;
    }

    const { pathname, suffix } = splitUrlReference(src);
    const decodedPath = decodeURIComponent(pathname);
    const resolvedPath = isAbsoluteFilePath(decodedPath)
      ? normalizeResolvedPath(decodedPath)
      : resolveRelativeFilePath(absolutePath, decodedPath);

    return `${convertFileSrc(resolvedPath)}${suffix}`;
  }

  function renderCodePreview(
    language: string,
    content: string,
    applyPreview: (value: null | string | HTMLElement) => void,
  ) {
    if (language.trim().toLowerCase() !== "mermaid") {
      return null;
    }

    const cacheKey = `${resolvedThemeName.value}::${content.trim()}`;
    const cachedPreview = mermaidPreviewCache.get(cacheKey);
    if (typeof cachedPreview === "string") {
      return wrapMermaidPreview(cachedPreview);
    }

    applyPreview(
      '<div class="desktop-doc-editor__diagram-preview">Rendering Mermaid...</div>',
    );

    if (cachedPreview instanceof Promise) {
      void cachedPreview
        .then((svg) => {
          applyPreview(wrapMermaidPreview(svg));
        })
        .catch(() => {
          applyPreview(createMermaidErrorMarkup());
        });
      return undefined;
    }

    mermaid.initialize({
      startOnLoad: false,
      securityLevel: "loose",
      theme: resolvedThemeName.value === "dark" ? "dark" : "default",
    });

    const renderTask = mermaid
      .render(`docs-atlas-mermaid-${++mermaidSequence}`, content)
      .then(({ svg }) => {
        mermaidPreviewCache.set(cacheKey, svg);
        return svg;
      })
      .catch(() => {
        mermaidPreviewCache.delete(cacheKey);
        throw new Error("Mermaid render failed");
      });

    mermaidPreviewCache.set(cacheKey, renderTask);
    void renderTask
      .then((svg) => {
        applyPreview(wrapMermaidPreview(svg));
      })
      .catch(() => {
        applyPreview(createMermaidErrorMarkup());
      });

    return undefined;
  }

  function wrapMermaidPreview(svg: string) {
    return `<div class="desktop-doc-editor__diagram-preview">${svg}</div>`;
  }

  function createMermaidErrorMarkup() {
    return '<div class="desktop-doc-editor__diagram-preview"><div class="desktop-doc-editor__diagram-error">Mermaid 渲染失败</div></div>';
  }

  function handleEditorClick(event: MouseEvent) {
    if (isUnmounted) {
      return;
    }

    const target = event.target;
    if (!(target instanceof HTMLElement)) {
      return;
    }

    const previewToggleButton = target.closest(
      ".preview-toggle-button",
    ) as HTMLButtonElement | null;
    if (previewToggleButton) {
      window.setTimeout(() => {
        if (isUnmounted) {
          return;
        }
        syncMermaidPreviewModeState(previewToggleButton);
      }, 0);
    }

    const copyButton = target.closest(
      ".tools-button-group button",
    ) as HTMLButtonElement | null;
    if (copyButton && /复制/.test(copyButton.textContent ?? "")) {
      window.setTimeout(() => {
        if (!isUnmounted) {
          emit("copied", "代码已复制");
        }
      }, 0);
    }

    const image = target.closest(
      'img[data-doc-preview-image="true"][data-preview-index]',
    ) as HTMLImageElement | null;
    if (!image) {
      scheduleSelectedImagePreview();
      return;
    }

    event.preventDefault();
    event.stopPropagation();

    const index = Number.parseInt(image.dataset.previewIndex ?? "-1", 10);
    if (Number.isNaN(index) || index < 0) {
      return;
    }

    activePreviewImageIndex.value = index;
  }

  function handleEditorFocusIn(event: FocusEvent) {
    const target = event.target;
    if (!import.meta.env.DEV) {
      return;
    }

    console.debug("[DocsAtlas][editor] focusin", {
      slug: props.doc.slug,
      tagName: target instanceof HTMLElement ? target.tagName : "",
      className: target instanceof HTMLElement ? target.className : "",
    });
  }

  function handleEditorFocusOut(event: FocusEvent) {
    const target = event.target;
    if (!import.meta.env.DEV) {
      return;
    }

    console.debug("[DocsAtlas][editor] focusout", {
      slug: props.doc.slug,
      tagName: target instanceof HTMLElement ? target.tagName : "",
      className: target instanceof HTMLElement ? target.className : "",
    });
  }

  function handleEditorPointerUp(event: PointerEvent) {
    if (isUnmounted) {
      return;
    }

    const target = event.target;
    if (!(target instanceof HTMLElement)) {
      return;
    }

    const image = target.closest(
      'img[data-doc-preview-image="true"][data-preview-index]',
    ) as HTMLImageElement | null;
    if (!image) {
      scheduleSelectedImagePreview();
      return;
    }

    event.preventDefault();
    event.stopPropagation();

    const index = Number.parseInt(image.dataset.previewIndex ?? "-1", 10);
    if (Number.isNaN(index) || index < 0) {
      return;
    }

    activePreviewImageIndex.value = index;
  }

  function scheduleSelectedImagePreview() {
    if (typeof window === "undefined") {
      return;
    }

    window.requestAnimationFrame(() => {
      if (isUnmounted) {
        return;
      }
      openSelectedImagePreview();
    });
  }

  function openSelectedImagePreview() {
    const host = hostRef.value;
    if (!host) {
      return;
    }

    const selectedImage = host.querySelector<HTMLImageElement>(
      '.milkdown-image-block.ProseMirror-selectednode img[data-doc-preview-image="true"][data-preview-index]',
    );

    if (!selectedImage) {
      return;
    }

    const index = Number.parseInt(
      selectedImage.dataset.previewIndex ?? "-1",
      10,
    );
    if (Number.isNaN(index) || index < 0) {
      return;
    }

    activePreviewImageIndex.value = index;
  }

  function syncMermaidPreviewModeState(toggleButton: HTMLButtonElement) {
    const block = toggleButton.closest(
      ".milkdown-code-block",
    ) as HTMLElement | null;
    if (!block) {
      return;
    }

    const language =
      block
        .querySelector<HTMLElement>(".language-button")
        ?.textContent?.trim()
        .toLowerCase() ?? "";
    if (language !== "mermaid") {
      return;
    }

    const content = extractCodeBlockContent(block);
    if (!content) {
      return;
    }

    const key =
      block.dataset.mermaidPreviewKey ??
      createMermaidPreviewStateKey(props.doc.slug, content);
    mermaidPreviewModeByKey.set(key, isCodeBlockPreviewOnly(block));
    block.dataset.mermaidPreviewKey = key;
  }

  function extractCodeBlockContent(block: HTMLElement) {
    const codeContent = block.querySelector<HTMLElement>(
      ".cm-content, .milkdown-code-block-placeholder code",
    );
    return codeContent?.textContent?.replace(/\u00a0/g, " ").trim() ?? "";
  }

  function isCodeBlockPreviewOnly(block: HTMLElement) {
    return (
      block.querySelector(".codemirror-host")?.classList.contains("hidden") ??
      false
    );
  }

  function createMermaidPreviewStateKey(slug: string, content: string) {
    return `${slug}::${content.trim()}`;
  }

  function closePreviewImage() {
    activePreviewImageIndex.value = -1;
  }

  function splitUrlReference(value: string) {
    const hashIndex = value.indexOf("#");
    const queryIndex = value.indexOf("?");
    const cutoffCandidates = [hashIndex, queryIndex].filter(
      (index) => index >= 0,
    );
    const cutoff =
      cutoffCandidates.length > 0
        ? Math.min(...cutoffCandidates)
        : value.length;

    return {
      pathname: value.slice(0, cutoff),
      suffix: value.slice(cutoff),
    };
  }

  function isAbsoluteFilePath(pathname: string) {
    return (
      pathname.startsWith("/") ||
      /^[a-zA-Z]:[\\/]/.test(pathname) ||
      pathname.startsWith("file://")
    );
  }

  function resolveRelativeFilePath(fromPath: string, targetPath: string) {
    const base = normalizeResolvedPath(fromPath);
    const normalizedTarget = targetPath.replace(/\\/g, "/");
    const fromSegments = base.split("/");
    fromSegments.pop();

    for (const segment of normalizedTarget.split("/")) {
      if (!segment || segment === ".") {
        continue;
      }

      if (segment === "..") {
        if (fromSegments.length > 1) {
          fromSegments.pop();
        }
        continue;
      }

      fromSegments.push(segment);
    }

    return fromSegments.join("/");
  }

  function normalizeResolvedPath(value: string) {
    return value.replace(/^file:\/\//i, "").replace(/\\/g, "/");
  }

  function resolveEditorPalette(
    theme: "dark" | "light",
    markdownThemeId: DesktopMarkdownThemeId,
  ): EditorPalette {
    const isDark = theme === "dark";
    const surfaceStrong = readRootToken("--desktop-surface-strong");
    const surface = readRootToken("--desktop-surface");

    if (markdownThemeId === "github") {
      return {
        panelBackground: isDark ? surfaceStrong : "#ffffff",
        surfaceLow: isDark ? "rgba(110, 118, 129, 0.08)" : "#f8fafc",
        codeBackground: isDark ? "#111a29" : "#ffffff",
        codeBorder: isDark
          ? "rgba(var(--desktop-accent-rgb), 0.22)"
          : "rgba(31, 35, 40, 0.12)",
        codeText: isDark ? "#e6edf3" : "#24292f",
        inlineCodeBackground: isDark
          ? "rgba(110, 118, 129, 0.18)"
          : "rgba(175, 184, 193, 0.2)",
        inlineCodeColor: isDark ? "#ffb86b" : "#b42318",
        quoteBackground: isDark
          ? "rgba(110, 118, 129, 0.1)"
          : "rgba(175, 184, 193, 0.12)",
        tableHeadBackground: isDark
          ? "rgba(110, 118, 129, 0.14)"
          : "rgba(246, 248, 250, 0.96)",
        tableZebraBackground: isDark
          ? "rgba(110, 118, 129, 0.05)"
          : "rgba(246, 248, 250, 0.72)",
        mutedLine: isDark
          ? "rgba(110, 118, 129, 0.3)"
          : "rgba(31, 35, 40, 0.12)",
        softFill: isDark
          ? "rgba(110, 118, 129, 0.1)"
          : "rgba(175, 184, 193, 0.12)",
        hoverFill: isDark
          ? "rgba(110, 118, 129, 0.12)"
          : "rgba(175, 184, 193, 0.14)",
        selectionFill: isDark
          ? "rgba(var(--desktop-accent-rgb), 0.18)"
          : "rgba(var(--desktop-accent-rgb), 0.12)",
      };
    }

    if (markdownThemeId === "compact") {
      return {
        panelBackground: surfaceStrong,
        surfaceLow: isDark
          ? "rgba(var(--desktop-accent-rgb), 0.08)"
          : "rgba(var(--desktop-accent-rgb), 0.04)",
        codeBackground: isDark ? "#0f1a2a" : "#ffffff",
        codeBorder: isDark
          ? "rgba(var(--desktop-accent-rgb), 0.18)"
          : "rgba(var(--desktop-accent-rgb), 0.12)",
        codeText: isDark ? "#edf3ff" : "#1f2b3d",
        inlineCodeBackground: isDark
          ? "rgba(var(--desktop-accent-rgb), 0.14)"
          : "rgba(var(--desktop-accent-rgb), 0.08)",
        inlineCodeColor: readRootToken("--desktop-accent"),
        quoteBackground: isDark
          ? "rgba(var(--desktop-accent-rgb), 0.1)"
          : "rgba(var(--desktop-accent-rgb), 0.05)",
        tableHeadBackground: isDark
          ? "rgba(var(--desktop-accent-rgb), 0.12)"
          : "rgba(var(--desktop-accent-rgb), 0.06)",
        tableZebraBackground: isDark
          ? "rgba(255, 255, 255, 0.03)"
          : "rgba(var(--desktop-accent-rgb), 0.025)",
        mutedLine: isDark
          ? "rgba(var(--desktop-accent-rgb), 0.22)"
          : "rgba(var(--desktop-accent-rgb), 0.12)",
        softFill: isDark
          ? "rgba(var(--desktop-accent-rgb), 0.12)"
          : "rgba(var(--desktop-accent-rgb), 0.06)",
        hoverFill: isDark
          ? "rgba(var(--desktop-accent-rgb), 0.14)"
          : "rgba(var(--desktop-accent-rgb), 0.08)",
        selectionFill: isDark
          ? "rgba(var(--desktop-accent-rgb), 0.18)"
          : "rgba(var(--desktop-accent-rgb), 0.12)",
      };
    }

    if (markdownThemeId === "reading") {
      return {
        panelBackground: surfaceStrong,
        surfaceLow: isDark ? "rgba(255, 255, 255, 0.03)" : "#f8fafc",
        codeBackground: isDark ? "#111b2c" : "#ffffff",
        codeBorder: isDark
          ? "rgba(var(--desktop-accent-rgb), 0.16)"
          : "rgba(99, 112, 138, 0.12)",
        codeText: isDark ? "#f3f6fd" : "#243146",
        inlineCodeBackground: isDark
          ? "rgba(var(--desktop-accent-rgb), 0.12)"
          : "rgba(99, 112, 138, 0.1)",
        inlineCodeColor: isDark ? "#9ac2ff" : "#304d93",
        quoteBackground: isDark
          ? "rgba(var(--desktop-accent-rgb), 0.08)"
          : "rgba(99, 112, 138, 0.06)",
        tableHeadBackground: isDark
          ? "rgba(var(--desktop-accent-rgb), 0.11)"
          : "rgba(99, 112, 138, 0.06)",
        tableZebraBackground: isDark
          ? "rgba(255, 255, 255, 0.028)"
          : "rgba(99, 112, 138, 0.028)",
        mutedLine: isDark
          ? "rgba(var(--desktop-accent-rgb), 0.18)"
          : "rgba(99, 112, 138, 0.12)",
        softFill: isDark
          ? "rgba(var(--desktop-accent-rgb), 0.1)"
          : "rgba(99, 112, 138, 0.05)",
        hoverFill: isDark
          ? "rgba(var(--desktop-accent-rgb), 0.12)"
          : "rgba(99, 112, 138, 0.08)",
        selectionFill: isDark
          ? "rgba(var(--desktop-accent-rgb), 0.18)"
          : "rgba(99, 112, 138, 0.1)",
      };
    }

    return {
      panelBackground: surfaceStrong,
      surfaceLow: isDark ? "rgba(255, 255, 255, 0.03)" : surface,
      codeBackground: isDark ? "#0f1928" : "#ffffff",
      codeBorder: isDark
        ? "rgba(var(--desktop-accent-rgb), 0.18)"
        : "rgba(var(--desktop-accent-rgb), 0.12)",
      codeText: isDark ? "#f1f5ff" : "#22314a",
      inlineCodeBackground: isDark
        ? "rgba(var(--desktop-accent-rgb), 0.14)"
        : "rgba(var(--desktop-accent-rgb), 0.08)",
      inlineCodeColor: isDark ? "#9dc2ff" : readRootToken("--desktop-accent"),
      quoteBackground: isDark
        ? "rgba(var(--desktop-accent-rgb), 0.08)"
        : "rgba(var(--desktop-accent-rgb), 0.05)",
      tableHeadBackground: isDark
        ? "rgba(var(--desktop-accent-rgb), 0.12)"
        : "rgba(var(--desktop-accent-rgb), 0.05)",
      tableZebraBackground: isDark
        ? "rgba(255, 255, 255, 0.026)"
        : "rgba(var(--desktop-accent-rgb), 0.024)",
      mutedLine: isDark
        ? "rgba(var(--desktop-accent-rgb), 0.2)"
        : "rgba(var(--desktop-accent-rgb), 0.12)",
      softFill: isDark
        ? "rgba(var(--desktop-accent-rgb), 0.11)"
        : "rgba(var(--desktop-accent-rgb), 0.06)",
      hoverFill: isDark
        ? "rgba(var(--desktop-accent-rgb), 0.14)"
        : "rgba(var(--desktop-accent-rgb), 0.08)",
      selectionFill: isDark
        ? "rgba(var(--desktop-accent-rgb), 0.18)"
        : "rgba(var(--desktop-accent-rgb), 0.12)",
    };
  }

  function readRootToken(name: string, rootStyles?: CSSStyleDeclaration) {
    const styles =
      rootStyles ?? window.getComputedStyle(document.documentElement);
    return styles.getPropertyValue(name).trim();
  }

  function normalizeSerializedMarkdown(markdown: string) {
    const lines = markdown.split("\n");
    const normalized: string[] = [];
    const listItemPattern = /^(\s*)(?:[-+*]|\d+[.)])\s+/;

    for (let index = 0; index < lines.length; index += 1) {
      const line = lines[index] ?? "";
      const previousLine = normalized[normalized.length - 1] ?? "";
      const nextLine = lines[index + 1] ?? "";

      if (
        line.trim() === "" &&
        previousLine.trim() !== "" &&
        nextLine.trim() !== ""
      ) {
        const previousMatch = previousLine.match(listItemPattern);
        const nextMatch = nextLine.match(listItemPattern);

        if (
          previousMatch &&
          nextMatch &&
          previousMatch[1] === nextMatch[1]
        ) {
          continue;
        }
      }

      normalized.push(line);
    }

    return normalized.join("\n");
  }

  async function persistDraft(options?: {
    absolutePath?: string;
    markdown?: string;
    mode?: "auto" | "manual";
  }) {
    const editor = editorRef.value;
    const absolutePath =
      options?.absolutePath ?? currentDocAbsolutePath.value ?? "";
    const mode = options?.mode ?? "manual";
    const nextMarkdown = normalizeSerializedMarkdown(
      options?.markdown ?? editor?.getMarkdown() ?? draftMarkdown.value,
    );

    if (
      !absolutePath ||
      isSaving.value ||
      nextMarkdown === savedMarkdown.value
    ) {
      return false;
    }

    isSaving.value = true;
    saveError.value = "";

    try {
      await props.saveDoc(absolutePath, nextMarkdown);
      const modifiedAt = new Date().toISOString();
      savedMarkdown.value = nextMarkdown;
      draftMarkdown.value = nextMarkdown;
      currentDocModifiedAt.value = modifiedAt;
      emit("saved", { mode, modifiedAt });
      return true;
    } catch (error) {
      saveError.value =
        error instanceof Error ? error.message : "保存失败，请稍后重试";
      return false;
    } finally {
      isSaving.value = false;
    }
  }

  async function handleSave() {
    await persistDraft({ mode: "manual" });
  }

  function handleWindowKeydown(event: KeyboardEvent) {
    if (!(event.metaKey || event.ctrlKey) || event.key.toLowerCase() !== "s") {
      return;
    }

    const activeElement = document.activeElement;
    if (
      !hostRef.value ||
      !activeElement ||
      !hostRef.value.contains(activeElement)
    ) {
      return;
    }

    event.preventDefault();
    void handleSave();
  }
</script>

<template>
  <div
    class="desktop-doc-editor"
    :class="{
      'desktop-doc-editor--dirty': isDirty && !isSaving && !saveError,
      'desktop-doc-editor--error': !!saveError,
      'desktop-doc-editor--readonly': !hasEditableSource,
      'desktop-doc-editor--ready': isEditorMounted,
    }"
    :data-markdown-theme="props.markdownThemeId"
    :title="
      saveError ||
      (!hasEditableSource
        ? '当前文档不可编辑'
        : isDirty
          ? '已修改，按 Ctrl/Cmd + S 保存'
          : 'Markdown 所见即所得编辑')
    "
  >
    <div
      ref="host"
      class="desktop-doc-editor__editor"
      @click="handleEditorClick"
      @focusin="handleEditorFocusIn"
      @focusout="handleEditorFocusOut"
      @pointerup="handleEditorPointerUp"
    />
  </div>

  <DesktopDocImagePreview
    v-if="activePreviewImageIndex >= 0"
    :active-index="activePreviewImageIndex"
    :images="previewImages"
    @close="closePreviewImage"
    @update:active-index="activePreviewImageIndex = $event"
  />
</template>

<style scoped>
  .desktop-doc-editor {
    position: relative;
    width: 100%;
    max-width: 960px;
    margin: 0 auto;
    min-width: 0;
    background: transparent;
  }

  .desktop-doc-editor__editor {
    min-width: 0;
    background: transparent;
  }

  .desktop-doc-editor--readonly {
    opacity: 1;
  }

  .desktop-doc-editor--error .desktop-doc-editor__editor {
    outline: 1px solid rgba(215, 70, 70, 0.2);
    outline-offset: -1px;
  }

  .desktop-doc-editor__editor :deep(.milkdown) {
    background: transparent;
    color: var(--desktop-ink);
  }

  .desktop-doc-editor__editor :deep(.milkdown .editor) {
    background: transparent;
  }

  .desktop-doc-editor__editor :deep(.milkdown .ProseMirror) {
    min-height: 100%;
    width: 100%;
    max-width: 100%;
    color: var(--desktop-ink);
    font-family: var(--desktop-font-sans);
    font-size: 0.94rem;
    line-height: 1.78;
    letter-spacing: 0.002em;
    background: transparent;
    overflow-anchor: none;
    padding: 28px 50px 36px;
  }

  .desktop-doc-editor__editor :deep(.milkdown .ProseMirror:focus) {
    outline: none;
  }

  .desktop-doc-editor__editor :deep(.milkdown h1),
  .desktop-doc-editor__editor :deep(.milkdown h2),
  .desktop-doc-editor__editor :deep(.milkdown h3),
  .desktop-doc-editor__editor :deep(.milkdown h4),
  .desktop-doc-editor__editor :deep(.milkdown h5),
  .desktop-doc-editor__editor :deep(.milkdown h6) {
    margin: 1.45rem 0 0.72rem;
    font-family: var(--desktop-font-sans);
    color: var(--desktop-ink);
    line-height: 1.28;
    letter-spacing: -0.022em;
    font-weight: 650;
  }

  .desktop-doc-editor__editor :deep(.milkdown > :first-child) {
    margin-top: 0;
  }

  .desktop-doc-editor__editor :deep(.milkdown h1) {
    margin-top: 0.2rem;
    margin-bottom: 0.9rem;
    font-size: 1.62rem;
  }

  .desktop-doc-editor__editor :deep(.milkdown h2) {
    margin-top: 1.8rem;
    margin-bottom: 0.82rem;
    padding-bottom: 0.42rem;
    font-size: 1.14rem;
    border-bottom: 2px solid
      color-mix(
        in srgb,
        var(--docs-atlas-muted-line) 52%,
        rgba(var(--desktop-accent-rgb), 0.28)
      );
  }

  .desktop-doc-editor__editor :deep(.milkdown h3) {
    margin-top: 1.35rem;
    margin-bottom: 0.56rem;
    font-size: 1rem;
  }

  .desktop-doc-editor__editor :deep(.milkdown h4) {
    margin-top: 1.1rem;
    margin-bottom: 0.48rem;
    font-size: 0.94rem;
  }

  .desktop-doc-editor__editor :deep(.milkdown h5),
  .desktop-doc-editor__editor :deep(.milkdown h6) {
    margin-top: 1rem;
    margin-bottom: 0.42rem;
    font-size: 0.9rem;
  }

  .desktop-doc-editor__editor :deep(.milkdown p),
  .desktop-doc-editor__editor :deep(.milkdown li),
  .desktop-doc-editor__editor :deep(.milkdown td),
  .desktop-doc-editor__editor :deep(.milkdown th),
  .desktop-doc-editor__editor :deep(.milkdown blockquote) {
    font-size: 0.94rem;
  }

  .desktop-doc-editor__editor :deep(.milkdown a) {
    color: var(--desktop-accent);
    text-underline-offset: 0.18em;
    text-decoration-thickness: 0.06em;
    text-decoration-color: rgba(var(--desktop-accent-rgb), 0.34);
  }

  .desktop-doc-editor__editor :deep(.milkdown blockquote) {
    border-left: 3px solid rgba(var(--desktop-accent-rgb), 0.36);
    background: var(--docs-atlas-quote-bg);
    color: var(--desktop-muted);
    padding-left: 20px;
    padding-top: 8px;
    padding-bottom: 8px;
  }

  .desktop-doc-editor__editor :deep(.milkdown blockquote::before) {
    display: none;
  }

  .desktop-doc-editor__editor :deep(.milkdown hr) {
    border-color: color-mix(
      in srgb,
      var(--docs-atlas-muted-line) 58%,
      rgba(var(--desktop-accent-rgb), 0.24)
    );
  }

  .desktop-doc-editor__editor :deep(.milkdown code) {
    font-family: "SFMono-Regular", "JetBrains Mono", "Fira Code", monospace;
    font-size: 0.835rem;
  }

  .desktop-doc-editor__editor :deep(.milkdown :not(pre) > code) {
    border: 1px solid var(--docs-atlas-code-border);
    border-radius: 0.42rem;
    background: var(--docs-atlas-inline-code-bg);
    color: var(--docs-atlas-inline-code-color);
    padding: 0.14rem 0.38rem;
  }

  .desktop-doc-editor__editor :deep(.milkdown pre) {
    margin: 0;
    border-radius: 16px;
    background: var(--docs-atlas-code-bg);
    color: var(--docs-atlas-code-text);
  }

  .desktop-doc-editor__editor :deep(.milkdown .milkdown-code-block) {
    position: relative;
    margin: 1.05rem 0 1.25rem;
    border: 1px solid
      color-mix(
        in srgb,
        var(--docs-atlas-code-border) 58%,
        rgba(var(--desktop-accent-rgb), 0.34)
      );
    border-radius: 10px;
    background: var(--docs-atlas-code-bg);
    /* box-shadow:
      0 10px 24px rgba(var(--desktop-shadow), 0.07),
      inset 0 1px 0 rgba(255, 255, 255, 0.03); */
    overflow-anchor: none;
    padding: 0;
  }

  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-code-block-placeholder) {
    margin: 0;
    min-height: 0;
    padding: calc(1.05rem + 2.7rem) 1.4rem 1.25rem;
    border-radius: 0 0 18px 18px;
    background: transparent;
    color: var(--docs-atlas-code-text);
    white-space: pre-wrap;
    word-break: break-word;
    overflow-wrap: anywhere;
    box-sizing: border-box;
  }

  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-code-block-placeholder code) {
    display: block;
    font-family: "SFMono-Regular", "JetBrains Mono", "Fira Code", monospace;
    font-size: 0.82rem;
    line-height: 1.7;
    tab-size: 2;
  }

  .desktop-doc-editor__editor :deep(.milkdown .milkdown-code-block .tools) {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.85rem;
    min-height: 2.7rem;
    padding: 0.5rem 0.95rem 0.48rem;
    opacity: 1;
    visibility: visible;
    border-bottom: 1px solid
      color-mix(
        in srgb,
        var(--docs-atlas-code-border) 52%,
        rgba(var(--desktop-accent-rgb), 0.3)
      );
    background: transparent;
    box-sizing: border-box;
  }

  .desktop-doc-editor__editor
    :deep(
      .milkdown .milkdown-code-block .tools .language-picker .list-wrapper
    ) {
    background-color: var(--docs-atlas-code-bg);
    border: 1px solid
      color-mix(
        in srgb,
        var(--docs-atlas-code-border) 58%,
        rgba(var(--desktop-accent-rgb), 0.34)
      );
  }

  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-code-block .tools .language-button),
  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-code-block .tools .tools-button-group button) {
    display: inline-flex;
    align-items: center;
    gap: 0.34rem;
    min-height: 1.72rem;
    border: 1px solid rgba(var(--desktop-accent-rgb), 0.08);
    margin: 0;
    opacity: 1;
    visibility: visible;
    background: color-mix(
      in srgb,
      var(--docs-atlas-code-bg) 82%,
      rgba(255, 255, 255, 0.08)
    );
    color: var(--docs-atlas-code-text);
    font-family: "SFMono-Regular", "JetBrains Mono", "Fira Code", monospace;
    font-size: 0.73rem;
    font-weight: 600;
    line-height: 1;
    border-radius: 10px;
    cursor: pointer;
    transition:
      border-color 160ms ease,
      background 160ms ease,
      transform 160ms ease;
  }

  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-code-block .tools .language-button) {
    flex: 0 0 auto;
    max-width: min(42%, 14rem);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-code-block .tools .tools-button-group) {
    display: inline-flex;
    align-items: center;
    gap: 0.42rem;
    margin-left: auto;
    opacity: 1;
    visibility: visible;
  }

  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-code-block .tools .language-button:hover),
  .desktop-doc-editor__editor
    :deep(
      .milkdown .milkdown-code-block .tools .tools-button-group button:hover
    ) {
    border-color: rgba(var(--desktop-accent-rgb), 0.2);
    background: rgba(var(--desktop-accent-rgb), 0.11);
    transform: translateY(-1px);
  }

  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-code-block .tools .language-button:focus-visible),
  .desktop-doc-editor__editor
    :deep(
      .milkdown
        .milkdown-code-block
        .tools
        .tools-button-group
        button:focus-visible
    ) {
    outline: 2px solid rgba(var(--desktop-accent-rgb), 0.28);
    outline-offset: 1px;
  }

  .desktop-doc-editor__editor :deep(.milkdown .milkdown-code-block .cm-editor),
  .desktop-doc-editor__editor :deep(.milkdown .milkdown-code-block .cm-gutters),
  .desktop-doc-editor__editor :deep(.milkdown .milkdown-code-block .preview) {
    background: transparent;
    color: var(--docs-atlas-code-text);
    overflow-anchor: none;
  }

  .desktop-doc-editor__editor :deep(.milkdown .milkdown-code-block .cm-editor) {
    font-size: 0.82rem;
    line-height: 1.7;
  }

  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-code-block .cm-gutters) {
    display: none;
  }

  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-code-block .cm-content) {
    padding: 1.02rem 1.2rem 1.16rem;
  }

  .desktop-doc-editor__editor :deep(.milkdown .milkdown-code-block .cm-line) {
    padding: 0;
  }

  .desktop-doc-editor__editor :deep(.milkdown .milkdown-code-block .cm-content),
  .desktop-doc-editor__editor :deep(.milkdown .milkdown-code-block .preview),
  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-code-block .preview code) {
    font-family: "SFMono-Regular", "JetBrains Mono", "Fira Code", monospace;
  }

  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-code-block .cm-activeLineGutter),
  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-code-block .cm-activeLine) {
    background: transparent;
  }

  .desktop-doc-editor__editor
    :deep(
      .milkdown .milkdown-code-block .cm-editor.cm-focused .cm-activeLineGutter
    ),
  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-code-block .cm-editor.cm-focused .cm-activeLine) {
    background: rgba(var(--desktop-accent-rgb), 0.08);
  }

  .desktop-doc-editor__editor :deep(.milkdown .milkdown-code-block .preview) {
    border-top: 0;
    padding: 1.05rem 1.2rem 1.2rem;
  }

  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-code-block .preview pre) {
    white-space: pre-wrap;
    background: transparent;
    border-radius: 0;
  }

  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-code-block .preview-panel) {
    background: transparent;
  }

  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-code-block .preview-label) {
    display: none;
  }

  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-code-block .preview-divider) {
    display: none;
  }

  .desktop-doc-editor__editor :deep(.milkdown img) {
    display: block;
    max-width: min(100%, 920px);
    border: 1px solid rgba(var(--desktop-accent-rgb), 0.12);
    border-radius: 14px;
    box-shadow: 0 14px 30px rgba(var(--desktop-shadow), 0.12);
    cursor: zoom-in;
  }

  .desktop-doc-editor__editor :deep(.milkdown .milkdown-image-block) {
    display: block;
    width: fit-content;
    max-width: 100%;
    margin: 1rem 0 1.3rem;
    padding: 0;
    background: transparent;
    border: 0;
    box-shadow: none;
    user-select: none;
  }

  .desktop-doc-editor__editor :deep(.milkdown .milkdown-image-block::before),
  .desktop-doc-editor__editor :deep(.milkdown .milkdown-image-block::after),
  .desktop-doc-editor__editor :deep(.milkdown .milkdown-image-block .operation),
  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-image-block .image-resize-handle),
  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-image-block .caption-input) {
    display: none !important;
  }

  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-image-block .image-wrapper) {
    position: relative;
    display: block;
    width: fit-content;
    max-width: 100%;
    margin: 0;
    padding: 0;
    background: transparent;
    border: 0;
    box-shadow: none;
    pointer-events: none;
  }

  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-image-block img[data-type="image-block"]) {
    height: auto !important;
    pointer-events: auto;
  }

  .desktop-doc-editor__editor
    :deep(.milkdown .docs-atlas-image-preview-hitbox) {
    position: absolute;
    inset: 0;
    z-index: 3;
    display: block;
    border: 0;
    padding: 0;
    margin: 0;
    background: transparent;
    cursor: zoom-in;
  }

  .desktop-doc-editor__editor :deep(.milkdown table) {
    width: 100%;
    border-collapse: collapse;
    table-layout: auto;
  }

  .desktop-doc-editor__editor :deep(.milkdown th),
  .desktop-doc-editor__editor :deep(.milkdown td) {
    min-width: 7rem;
    border: 1px solid var(--docs-atlas-muted-line);
    text-align: left;
    vertical-align: top;
    word-break: break-word;
    overflow-wrap: anywhere;
    line-height: 1.58;
  }

  .desktop-doc-editor__editor :deep(.milkdown th) {
    background: var(--docs-atlas-table-head-bg);
    font-weight: 600;
  }

  .desktop-doc-editor__editor :deep(.milkdown tbody tr:nth-child(even) td) {
    background: var(--docs-atlas-table-zebra-bg);
  }

  .desktop-doc-editor__editor :deep(.milkdown .milkdown-table-block) {
    overflow: visible;
  }

  .desktop-doc-editor__editor :deep(.milkdown .milkdown-table-block .handle),
  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-table-block .line-handle .add-button),
  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-table-block .cell-handle .button-group button) {
    background: var(--crepe-color-background);
    color: var(--desktop-muted);
    border-color: var(--docs-atlas-muted-line);
  }

  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-table-block .selectedCell::after) {
    background: rgba(var(--desktop-accent-rgb), 0.12);
  }

  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-code-block .search-box) {
    display: grid;
    gap: 0.38rem;
    min-width: 13rem;
    background: var(--desktop-surface-strong);
    min-height: 1.78rem;
    border-radius: 0.78rem;
    box-shadow: 0 14px 28px rgba(var(--desktop-shadow), 0.08);
    outline: none;
    padding: 0;
  }

  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-code-block .search-box:has(input:focus)) {
    outline: none;
  }

  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-code-block .search-box input) {
    width: 100%;
    min-height: 2rem;
    padding: 0 0.78rem;
    border: 1px solid var(--docs-atlas-muted-line);
    border-radius: 0.62rem;
    background: var(--desktop-field-bg-strong);
    color: var(--desktop-ink);
    font-size: 0.76rem;
  }

  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-code-block .search-box input:focus) {
    outline: none;
    border-color: rgba(var(--desktop-accent-rgb), 0.2);
    background: rgba(var(--desktop-accent-rgb), 0.06);
  }

  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-code-block .search-box .search-icon),
  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-code-block .search-box .clear-icon) {
    color: var(--desktop-muted);
  }

  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-code-block .search-box ul) {
    display: grid;
    gap: 0.18rem;
    max-height: 15rem;
    margin: 0;
    padding: 0;
    overflow: auto;
    list-style: none;
  }

  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-code-block .search-box ul::-webkit-scrollbar) {
    width: 8px;
  }

  .desktop-doc-editor__editor
    :deep(
      .milkdown .milkdown-code-block .search-box ul::-webkit-scrollbar-thumb
    ) {
    background: rgba(var(--desktop-accent-rgb), 0.22);
    border-radius: 999px;
  }

  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-code-block .search-box button),
  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-code-block .search-box li),
  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-code-block .search-box [role="option"]) {
    display: flex;
    align-items: center;
    min-height: 2rem;
    padding: 0 0.72rem;
    border-radius: 0.62rem;
    color: var(--desktop-ink);
    background: transparent;
    transition:
      background 140ms ease,
      color 140ms ease;
  }

  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-code-block .search-box button:hover),
  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-code-block .search-box li:hover),
  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-code-block .search-box [role="option"]:hover) {
    background: rgba(var(--desktop-accent-rgb), 0.08);
    color: var(--desktop-ink);
  }

  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-code-block .search-box .active),
  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-code-block .search-box [aria-selected="true"]) {
    background: rgba(var(--desktop-accent-rgb), 0.12);
    color: var(--desktop-ink);
  }

  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-code-block-placeholder code.hljs),
  .desktop-doc-editor__editor
    :deep(.milkdown .milkdown-code-block .preview code.hljs) {
    background: transparent;
    color: var(--docs-atlas-code-text);
    opacity: 1;
  }

  .desktop-doc-editor__editor :deep(.milkdown .ProseMirror-selectednode) {
    outline-color: rgba(var(--desktop-accent-rgb), 0.28);
  }

  .desktop-doc-editor__editor :deep(.milkdown .ProseMirror ::selection) {
    background: rgba(var(--desktop-accent-rgb), 0.22);
    color: var(--desktop-ink);
  }

  .desktop-doc-editor__editor :deep(.milkdown .cm-focused) {
    outline: none;
  }

  .desktop-doc-editor__editor :deep(.milkdown .cm-selectionBackground),
  .desktop-doc-editor__editor :deep(.milkdown .cm-content ::selection),
  .desktop-doc-editor__editor :deep(.milkdown .cm-line ::selection) {
    background: rgba(var(--desktop-accent-rgb), 0.24) !important;
  }

  .desktop-doc-editor__editor :deep(.milkdown .cm-cursor) {
    border-left-color: var(--desktop-accent);
  }

  .desktop-doc-editor__editor :deep(.milkdown .cm-panels) {
    background: transparent;
    color: var(--desktop-soft);
  }

  .desktop-doc-editor__editor :deep(.milkdown .cm-panel input) {
    background: var(--docs-atlas-soft-fill);
    color: var(--desktop-ink);
    border-color: var(--docs-atlas-muted-line);
  }

  .desktop-doc-editor__diagram-preview {
    display: flex;
    justify-content: center;
    align-items: center;
    min-height: 140px;
    width: 100%;
    padding: 1.1rem 1rem;
    color: var(--desktop-soft);
    text-align: center;
  }

  .desktop-doc-editor__diagram-preview :deep(svg) {
    max-width: 100%;
    height: auto;
  }

  .desktop-doc-editor__diagram-error {
    color: rgb(210, 69, 69);
    font-size: 0.86rem;
  }

  @media (max-width: 900px) {
    .desktop-doc-editor__editor :deep(.milkdown .ProseMirror) {
      max-width: none;
    }
  }
</style>
