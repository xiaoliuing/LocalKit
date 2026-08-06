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
  import type { WorkspaceSourceNode } from "@docs-atlas/shared-types/workspace";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { saveMarkdownDocument } from "@/api/documents";
  import {
    exportLogsFile,
    listenDesktopMenuActions,
    openAppDataDirectory,
    openLogsDirectory,
    type DesktopMenuAction,
  } from "@/api/system";
  import DesktopFavoritesView, {
    type DesktopFavoriteViewEntry,
  } from "@/components/docs/DesktopFavoritesView.vue";
  import DesktopDocReader from "@/components/docs/DesktopDocReader.vue";
  import DesktopRecentView, {
    type DesktopRecentViewEntry,
  } from "@/components/docs/DesktopRecentView.vue";
  import DesktopDocsSidebar from "@/components/docs/DesktopDocsSidebar.vue";
  import DesktopDocToc from "@/components/docs/DesktopDocToc.vue";
  import DesktopSearchPanel from "@/components/docs/DesktopSearchPanel.vue";
  import DesktopSettingsView from "@/components/settings/DesktopSettingsView.vue";
  import DesktopAgentSessionCleanerView from "@/components/tools/DesktopAgentSessionCleanerView.vue";
  import DesktopToolHubView from "@/components/tools/DesktopToolHubView.vue";
  import DesktopVideoToolView from "@/components/tools/DesktopVideoToolView.vue";
  import type { DesktopSettingsSection } from "@/components/settings/DesktopSettingsNav.vue";
  import DesktopUiIcon from "@/components/ui/DesktopUiIcon.vue";
  import DesktopWorkspaceDialog from "@/components/workspace/DesktopWorkspaceDialog.vue";
  import { useDesktopActiveHeadings } from "@/composables/useDesktopActiveHeadings";
  import { useDesktopDocsBrowser } from "@/composables/useDesktopDocsBrowser";
  import { useDesktopPreferences } from "@/composables/useDesktopPreferences";
  import { useDesktopReleaseUpdates } from "@/composables/useDesktopReleaseUpdates";
  import { useDesktopReadingState } from "@/composables/useDesktopReadingState";
  import { useDesktopDocsSearch } from "@/composables/useDesktopDocsSearch";
  import { useDesktopSearchCatalog } from "@/composables/useDesktopSearchCatalog";
  import { useDesktopWorkspaceDocs } from "@/composables/useDesktopWorkspaceDocs";
  import { useDesktopReaderLayout } from "@/composables/useDesktopReaderLayout";
  import { useWorkspaceSelection } from "@/composables/useWorkspaceSelection";

  const {
    createWorkspace,
    currentWorkspace,
    currentWorkspaceId,
    currentWorkspaceSourceIds,
    deleteWorkspace,
    ensureLoaded,
    exportWorkspaceConfig,
    importWorkspaceConfig,
    isDeletingWorkspace,
    isExportingWorkspace,
    isImportingWorkspace,
    isLoadingWorkspaces,
    isSavingWorkspace,
    selectWorkspace,
    updateWorkspaceMeta,
    workspaces,
  } = useWorkspaceSelection();
  const workspaceDocs = useDesktopWorkspaceDocs({
    workspace: currentWorkspace,
  });
  const searchCatalog = useDesktopSearchCatalog({
    workspaces,
  });
  const {
    clearSelection,
    currentDoc,
    currentSectionId,
    currentSourceId,
    docs,
    headings,
    nextDoc,
    prevDoc,
    selectDoc,
    selectFirstDocBySourceIds,
    selectedDocSlug,
    sourceGroups,
  } = useDesktopDocsBrowser({
    docs: workspaceDocs.docs,
    docsBySlug: workspaceDocs.docsBySlug,
    docDetailsBySlug: workspaceDocs.docDetailsBySlug,
    sourceGroups: workspaceDocs.sourceGroups,
  });
  const readingState = useDesktopReadingState();
  const {
    activeResult,
    close: closeSearch,
    isOpen,
    moveSelection,
    open: openSearch,
    query,
    results,
    scope,
    selectedIndex,
    setSourceFilter,
    setQuery,
    setScope,
    setWorkspaceFilter,
    sourceFilter,
    sourceOptions,
    workspaceFilter,
    workspaceOptions,
  } = useDesktopDocsSearch({
    currentWorkspaceId,
    docsBySlug: searchCatalog.docsBySlug,
    searchIndex: searchCatalog.searchIndex,
    workspaceIdBySearchSlug: searchCatalog.workspaceIdBySearchSlug,
    workspaces,
    workspaceSourceIds: currentWorkspaceSourceIds,
  });
  const {
    accentOptions,
    markdownThemeOptions,
    preferences,
    setAccent,
    setCustomAccentColor,
    setMarkdownTheme,
    setThemeMode,
  } = useDesktopPreferences();
  const { activeId, scrollToHeading } = useDesktopActiveHeadings(headings);
  const {
    sidebarWidth,
    startSidebarResize,
    startTocResize,
    tocWidth,
  } = useDesktopReaderLayout();
  const {
    checkForUpdates,
    currentVersion,
    installUpdate,
    lastCheckedAt,
    latestRelease,
    loadCurrentVersion,
    message: updateMessage,
    openLatestRelease,
    status: updateStatus,
  } = useDesktopReleaseUpdates();

  type DesktopPrimaryView =
    | "reader"
    | "recent"
    | "favorites"
    | "settings"
    | "tools"
    | "agent-sessions"
    | "video-player";
  type DesktopSidebarView = "reader" | "recent" | "favorites" | "settings";
  type DesktopDocEntryKey = `${string}::${string}`;

  const PRIMARY_VIEW_STORAGE_KEY = "docs-atlas.desktop.primary-view.v2";

  const primaryView = shallowRef<DesktopPrimaryView>(readPersistedPrimaryView());
  const settingsReturnView =
    shallowRef<Exclude<DesktopPrimaryView, "settings">>("tools");
  const settingsSection = shallowRef<DesktopSettingsSection>("appearance");
  const isWorkspaceDialogOpen = shallowRef(false);
  const currentReaderScrollTop = shallowRef(0);
  const restoredScrollTop = shallowRef(0);
  const settingsActionMessage = shallowRef("");
  const settingsBusyAction = shallowRef<"app-data" | "logs" | "export" | null>(
    null,
  );
  const sidebarOpenBranchIds = shallowRef<string[]>([]);
  const sidebarOpenSectionId = shallowRef<string | null>(null);
  const workspaceDialogMode = shallowRef<"create" | "edit">("create");
  const hasRestoredInitialWorkspace = shallowRef(false);
  const pendingRestoreWorkspaceId = shallowRef("");
  const pendingRestoreDocSlug = shallowRef("");
  const titlebarSearchInputRef =
    useTemplateRef<HTMLInputElement>("titlebarSearchInput");
  let desktopMenuActionUnlisten: UnlistenFn | null = null;
  let settingsActionMessageTimer: number | null = null;

  const isReaderView = computed(() => primaryView.value === "reader");
  const isRecentView = computed(() => primaryView.value === "recent");
  const isFavoritesView = computed(() => primaryView.value === "favorites");
  const isSettingsView = computed(() => primaryView.value === "settings");
  const isToolsHubView = computed(() => primaryView.value === "tools");
  const isAgentSessionsToolView = computed(
    () => primaryView.value === "agent-sessions",
  );
  const isVideoToolView = computed(() => primaryView.value === "video-player");
  const isToolView = computed(
    () =>
      isToolsHubView.value ||
      isAgentSessionsToolView.value ||
      isVideoToolView.value,
  );
  const canUseTitlebarSearch = computed(() => !isToolView.value);
  const isReaderLoading = computed(
    () => workspaceDocs.isLoading.value && !currentDoc.value,
  );
  const showReaderToc = computed(
    () => isReaderView.value && (headings.value.length > 0 || isReaderLoading.value),
  );
  const sidebarActiveView = computed<DesktopSidebarView>(() => {
    if (primaryView.value === "recent" || primaryView.value === "favorites") {
      return primaryView.value;
    }

    if (primaryView.value === "settings") {
      return "settings";
    }

    return "reader";
  });
  const workbenchStyle = computed(() => {
    if (isSettingsView.value || isToolView.value) {
      return {};
    }

    if (isRecentView.value || isFavoritesView.value) {
      return {
        gridTemplateColumns: "var(--desktop-rail-w) minmax(0, 1fr)",
      };
    }

    return {
      gridTemplateColumns: `${sidebarWidth.value}px 1px minmax(0, 1fr)`,
    };
  });
  const readerSidebarStyle = computed(() =>
    isReaderView.value ? { width: `${sidebarWidth.value}px` } : undefined,
  );
  const floatingPanelVisible = computed(
    () => canUseTitlebarSearch.value && isOpen.value,
  );
  const searchQuery = computed({
    get: () => query.value,
    set: (value: string) => {
      setQuery(value);
    },
  });
  const sourceCount = computed(() =>
    countWorkspaceFolderSources(currentWorkspace.value?.sources ?? []),
  );
  const visibleSourceGroups = computed(() => sourceGroups.value);
  const docCount = computed(() => docs.value.length);
  const workspaceDialogWorkspace = computed(() =>
    workspaceDialogMode.value === "edit" ? currentWorkspace.value : null,
  );
  const recentEntries = computed<DesktopRecentViewEntry[]>(() =>
    readingState.recentEntries.value.flatMap((entry) => {
      const entryId = createDocEntryId(entry.workspaceId, entry.slug);
      const workspace = workspaces.value.find(
        (item) => item.id === entry.workspaceId,
      );
      const docMeta = searchCatalog.docsBySlug.value[entryId];

      if (!workspace || !docMeta) {
        return [];
      }

      return [
        {
          id: entryId,
          openedAt: entry.openedAt,
          scrollTop: readingState.getDocScrollTop(
            entry.workspaceId,
            entry.slug,
          ),
          slug: entry.slug,
          sourceLabel: docMeta.sectionTitle
            ? `${docMeta.sourceLabel} / ${docMeta.sectionTitle}`
            : docMeta.sourceLabel,
          summary: docMeta.summary,
          title: docMeta.title,
          workspaceId: entry.workspaceId,
          workspaceName: workspace.name,
        },
      ];
    }),
  );
  const favoriteEntries = computed<DesktopFavoriteViewEntry[]>(() =>
    readingState.favoriteEntries.value.flatMap((entry) => {
      const entryId = createDocEntryId(entry.workspaceId, entry.slug);
      const workspace = workspaces.value.find(
        (item) => item.id === entry.workspaceId,
      );
      const docMeta = searchCatalog.docsBySlug.value[entryId];

      if (!workspace || !docMeta) {
        return [];
      }

      return [
        {
          id: entryId,
          savedAt: entry.savedAt,
          slug: entry.slug,
          sourceLabel: docMeta.sectionTitle
            ? `${docMeta.sourceLabel} / ${docMeta.sectionTitle}`
            : docMeta.sourceLabel,
          summary: docMeta.summary,
          title: docMeta.title,
          workspaceId: entry.workspaceId,
          workspaceName: workspace.name,
        },
      ];
    }),
  );
  const currentDocIsFavorite = computed(() => {
    if (!currentWorkspaceId.value || !selectedDocSlug.value) {
      return false;
    }

    return readingState.isDocFavorite(
      currentWorkspaceId.value,
      selectedDocSlug.value,
    );
  });

  async function handleSelectWorkspace(workspaceId: string) {
    workspaceDocs.resumeWatchRefresh();
    const restoredSlug = readingState.getSelectedDocForWorkspace(workspaceId);

    await selectWorkspace(workspaceId);

    if (restoredSlug) {
      await waitForDocAvailability(restoredSlug);
      selectDoc(restoredSlug);
    }

    closeSearch();
  }

  function handleSelectDoc(slug: string) {
    workspaceDocs.resumeWatchRefresh();
    selectDoc(slug);
  }

  async function showSearchPanel() {
    if (!canUseTitlebarSearch.value) {
      closeSearch();
      return;
    }

    openSearch();
    await nextTick();
    titlebarSearchInputRef.value?.focus();
  }

  function onTitlebarSearchFocus() {
    if (!canUseTitlebarSearch.value) {
      closeSearch();
      return;
    }

    openSearch();
  }

  function onTitlebarSearchKeydown(event: KeyboardEvent) {
    if (!canUseTitlebarSearch.value) {
      closeSearch();
      return;
    }

    if (!isOpen.value) {
      openSearch();
    }

    if (event.key === "ArrowDown") {
      event.preventDefault();
      moveSelection(1);
      return;
    }

    if (event.key === "ArrowUp") {
      event.preventDefault();
      moveSelection(-1);
      return;
    }

    if (event.key === "Enter") {
      event.preventDefault();
      handleSubmitSearch(results.value[selectedIndex.value]?.slug);
      return;
    }

    if (event.key === "Escape") {
      closeSearch();
      titlebarSearchInputRef.value?.blur();
    }
  }

  function toggleSettingsPanel() {
    if (isSettingsView.value) {
      closeSettingsView();
      return;
    }

    openSettingsView("appearance");
  }

  function openRecentView() {
    persistCurrentDocScrollTop();
    primaryView.value = "recent";
    closeSearch();
  }

  function closeRecentView() {
    openReaderView();
  }

  function openFavoritesView() {
    persistCurrentDocScrollTop();
    primaryView.value = "favorites";
    closeSearch();
  }

  function closeFavoritesView() {
    openReaderView();
  }

  function openSettingsView(section: DesktopSettingsSection = "appearance") {
    persistCurrentDocScrollTop();
    if (!isSettingsView.value) {
      settingsReturnView.value =
        primaryView.value === "settings" ? "tools" : primaryView.value;
    }
    settingsSection.value = section;
    primaryView.value = "settings";
    closeSearch();
  }

  function openToolsView() {
    persistCurrentDocScrollTop();
    primaryView.value = "tools";
    closeSearch();
  }

  function openVideoToolView() {
    primaryView.value = "video-player";
    closeSearch();
  }

  function openAgentSessionsToolView() {
    primaryView.value = "agent-sessions";
    closeSearch();
  }

  function handleOpenTool(
    toolId: "docs" | "video" | "agent-sessions" | "audio" | "knowledge",
  ) {
    if (toolId === "docs") {
      openReaderView();
      return;
    }

    if (toolId === "video") {
      openVideoToolView();
      return;
    }

    if (toolId === "agent-sessions") {
      openAgentSessionsToolView();
    }
  }

  function closeSettingsView() {
    const returnView = settingsReturnView.value;
    primaryView.value = returnView;
    closeSearch();
    clearSettingsActionMessage();

    if (returnView !== "reader") {
      return;
    }

    const workspaceId = currentWorkspaceId.value;
    const slug = selectedDocSlug.value;
    const savedScrollTop =
      workspaceId && slug ? readingState.getDocScrollTop(workspaceId, slug) : 0;

    restoredScrollTop.value = -1;
    currentReaderScrollTop.value = savedScrollTop;

    window.requestAnimationFrame(() => {
      window.requestAnimationFrame(() => {
        restoredScrollTop.value = savedScrollTop;
      });
    });
  }

  function openReaderView() {
    primaryView.value = "reader";
    void nextTick(() => {
      restoreCurrentDocScrollTop();
    });
    closeSearch();
  }

  async function handleSubmitSearch(slug?: string) {
    const targetSearchSlug = slug || activeResult.value?.slug;
    if (!targetSearchSlug) {
      return;
    }

    const targetWorkspaceId =
      searchCatalog.workspaceIdBySearchSlug.value[targetSearchSlug];
    const targetDocSlug =
      searchCatalog.docSlugBySearchSlug.value[targetSearchSlug] ??
      targetSearchSlug;

    if (targetWorkspaceId && targetWorkspaceId !== currentWorkspaceId.value) {
      await selectWorkspace(targetWorkspaceId);
      await waitForDocAvailability(targetDocSlug);
    }

    selectDoc(targetDocSlug);
    primaryView.value = "reader";
    closeSearch();
  }

  async function handleOpenRecentEntry(entryId: string) {
    const separatorIndex = entryId.indexOf("::");
    if (separatorIndex === -1) {
      return;
    }

    const targetWorkspaceId = entryId.slice(0, separatorIndex);
    const targetDocSlug = entryId.slice(separatorIndex + 2);
    if (!targetWorkspaceId || !targetDocSlug) {
      return;
    }

    if (targetWorkspaceId !== currentWorkspaceId.value) {
      await selectWorkspace(targetWorkspaceId);
      await waitForDocAvailability(targetDocSlug);
    }

    selectDoc(targetDocSlug);
    primaryView.value = "reader";
    closeSearch();
  }

  async function handleOpenFavoriteEntry(entryId: string) {
    await handleOpenRecentEntry(entryId);
  }

  function handleRemoveFavoriteEntry(entryId: string) {
    const separatorIndex = entryId.indexOf("::");
    if (separatorIndex === -1) {
      return;
    }

    const targetWorkspaceId = entryId.slice(0, separatorIndex);
    const targetDocSlug = entryId.slice(separatorIndex + 2);
    if (!targetWorkspaceId || !targetDocSlug) {
      return;
    }

    readingState.removeFavoriteDoc(targetWorkspaceId, targetDocSlug);
  }

  function handleToggleCurrentDocFavorite() {
    if (!currentWorkspaceId.value || !selectedDocSlug.value) {
      return;
    }

    readingState.toggleFavoriteDoc(
      currentWorkspaceId.value,
      selectedDocSlug.value,
    );
  }

  async function handleSaveCurrentDoc(absolutePath: string, markdown: string) {
    workspaceDocs.pauseWatchRefresh();
    try {
      await saveMarkdownDocument(absolutePath, markdown);
    } finally {
      workspaceDocs.resumeWatchRefresh({ suppressForMs: 2_500 });
    }
  }

  function closeFloatingPanels() {
    closeSearch();
  }

  function handleTitlebarClick(event: MouseEvent) {
    if (!isOpen.value) {
      return;
    }

    const target = event.target;
    if (
      target instanceof Element &&
      target.closest(
        ".desktop-titlebar__search-shell, .desktop-titlebar__actions",
      )
    ) {
      return;
    }

    closeSearch();
    titlebarSearchInputRef.value?.blur();
  }

  async function handleCreateWorkspace(payload: {
    name: string;
    description: string;
    color: string;
    sources: Parameters<typeof createWorkspace>[0]["sources"];
  }) {
    const workspace =
      workspaceDialogMode.value === "edit" && currentWorkspace.value
        ? await updateWorkspaceMeta(currentWorkspace.value.id, payload)
        : await createWorkspace({
            name: payload.name,
            description: payload.description,
            color: payload.color,
            defaultSearchScope: "global",
            icon: "folder",
            lastOpenedAt: new Date().toISOString(),
            sources: payload.sources,
          });

    if (!workspace) {
      return;
    }

    isWorkspaceDialogOpen.value = false;
  }

  function openCreateWorkspaceDialog() {
    workspaceDialogMode.value = "create";
    isWorkspaceDialogOpen.value = true;
  }

  function openEditWorkspaceDialog() {
    if (!currentWorkspace.value) {
      return;
    }

    workspaceDialogMode.value = "edit";
    isWorkspaceDialogOpen.value = true;
  }

  async function handleDeleteWorkspace() {
    if (!currentWorkspace.value) {
      return;
    }

    const deletedWorkspaceId = currentWorkspace.value.id;
    const nextWorkspaceId = await deleteWorkspace(deletedWorkspaceId);
    if (!nextWorkspaceId) {
      return;
    }

    isWorkspaceDialogOpen.value = false;
    sidebarOpenBranchIds.value = [];
    sidebarOpenSectionId.value = null;

    if (readingState.currentWorkspaceId.value === deletedWorkspaceId) {
      readingState.setCurrentWorkspaceId(nextWorkspaceId);
    }
  }

  async function handleImportWorkspace() {
    const importedWorkspace = await importWorkspaceConfig();
    if (!importedWorkspace) {
      return;
    }

    pendingRestoreWorkspaceId.value = "";
    pendingRestoreDocSlug.value = "";
    isWorkspaceDialogOpen.value = false;
  }

  async function handleExportWorkspace() {
    if (!currentWorkspace.value) {
      return;
    }

    await exportWorkspaceConfig(currentWorkspace.value.id);
  }

  function setSettingsActionFeedback(message: string) {
    settingsActionMessage.value = message;

    if (settingsActionMessageTimer !== null) {
      window.clearTimeout(settingsActionMessageTimer);
    }

    settingsActionMessageTimer = window.setTimeout(() => {
      settingsActionMessage.value = "";
      settingsActionMessageTimer = null;
    }, 3200);
  }

  function clearSettingsActionMessage() {
    settingsActionMessage.value = "";

    if (settingsActionMessageTimer !== null) {
      window.clearTimeout(settingsActionMessageTimer);
      settingsActionMessageTimer = null;
    }
  }

  async function runSystemSettingsAction(
    action: "app-data" | "logs" | "export",
    task: () => Promise<boolean>,
    successMessage: string,
  ) {
    settingsBusyAction.value = action;

    try {
      const success = await task();
      setSettingsActionFeedback(success ? successMessage : "操作已取消");
    } catch (error) {
      setSettingsActionFeedback(
        error instanceof Error ? error.message : "操作失败，请稍后重试",
      );
    } finally {
      settingsBusyAction.value = null;
    }
  }

  async function handleOpenAppDataDirectory() {
    await runSystemSettingsAction(
      "app-data",
      openAppDataDirectory,
      "已打开应用数据目录",
    );
  }

  async function handleOpenLogsDirectory() {
    await runSystemSettingsAction("logs", openLogsDirectory, "已打开日志目录");
  }

  async function handleExportLogsFile() {
    await runSystemSettingsAction("export", exportLogsFile, "日志文件已导出");
  }

  async function handleTitlebarMouseDown(event: MouseEvent) {
    if (!isTauriRuntime() || event.button !== 0) {
      return;
    }

    const target = event.target;
    if (
      target instanceof Element &&
      target.closest(
        ".desktop-titlebar__search-shell, .desktop-titlebar__actions, button, input, select, label, option",
      )
    ) {
      return;
    }

    if (event.detail === 2) {
      // Tauri already toggles maximize on double click for drag regions.
      return;
    }

    event.preventDefault();
    await getCurrentWindow().startDragging();
  }

  async function handleDesktopMenuAction(action: DesktopMenuAction) {
    switch (action) {
      case "open-search":
        await showSearchPanel();
        break;
      case "open-settings":
        openSettingsView("appearance");
        break;
      case "import-workspace":
        await handleImportWorkspace();
        break;
      case "export-workspace":
        await handleExportWorkspace();
        break;
    }
  }

  async function bindDesktopMenuActions() {
    desktopMenuActionUnlisten = await listenDesktopMenuActions((action) => {
      void handleDesktopMenuAction(action);
    });
  }

  function persistDesktopSession() {
    persistCurrentDocScrollTop();
    readingState.flushPersist();
    window.localStorage.setItem(PRIMARY_VIEW_STORAGE_KEY, primaryView.value);
  }

  function readPersistedPrimaryView(): DesktopPrimaryView {
    if (typeof window === "undefined") {
      return "tools";
    }

    const storedView = window.localStorage.getItem(PRIMARY_VIEW_STORAGE_KEY);
    return isDesktopPrimaryView(storedView) ? storedView : "tools";
  }

  function isDesktopPrimaryView(value: unknown): value is DesktopPrimaryView {
    return (
      value === "reader" ||
      value === "recent" ||
      value === "favorites" ||
      value === "settings" ||
      value === "tools" ||
      value === "agent-sessions" ||
      value === "video-player"
    );
  }

  onMounted(() => {
    void restoreInitialWorkspace();
    void bindDesktopMenuActions();
    void loadCurrentVersion();
    window.addEventListener("beforeunload", persistDesktopSession);
  });

  onBeforeUnmount(() => {
    persistDesktopSession();
    clearSettingsActionMessage();
    desktopMenuActionUnlisten?.();
    desktopMenuActionUnlisten = null;
    window.removeEventListener("beforeunload", persistDesktopSession);
  });

  watch(primaryView, (view) => {
    window.localStorage.setItem(PRIMARY_VIEW_STORAGE_KEY, view);

    if (isToolView.value) {
      closeSearch();
    }
  });

  watch(
    [currentWorkspaceId, currentWorkspaceSourceIds, docs, selectedDocSlug],
    ([workspaceId, sourceIds, docsList, activeSlug]) => {
      if (isLoadingWorkspaces.value) {
        return;
      }

      if (docsList.length === 0 || sourceIds.length === 0) {
        clearSelection();
        restoredScrollTop.value = 0;
        return;
      }

      const docsBySlug = workspaceDocs.docsBySlug.value;
      const currentDocMeta = activeSlug
        ? (docsBySlug[activeSlug] ?? null)
        : null;
      const isCurrentDocValid = Boolean(
        currentDocMeta && sourceIds.includes(currentDocMeta.sourceId),
      );
      const restoreSlug =
        workspaceId && pendingRestoreWorkspaceId.value === workspaceId
          ? pendingRestoreDocSlug.value
          : "";
      const restoreDocMeta = restoreSlug
        ? (docsBySlug[restoreSlug] ?? null)
        : null;
      const isRestoreDocValid = Boolean(
        restoreDocMeta && sourceIds.includes(restoreDocMeta.sourceId),
      );

      if (restoreSlug) {
        if (isRestoreDocValid && activeSlug !== restoreSlug) {
          selectDoc(restoreSlug);
          return;
        }

        pendingRestoreWorkspaceId.value = "";
        pendingRestoreDocSlug.value = "";
      }

      if (!isCurrentDocValid) {
        selectFirstDocBySourceIds(sourceIds);
      }
    },
    { immediate: true },
  );

  watch(
    [currentWorkspaceId, () => currentWorkspace.value?.defaultSearchScope],
    ([workspaceId, defaultScope]) => {
      if (!workspaceId) {
        setScope(defaultScope ?? "global");
        return;
      }

      setScope(
        readingState.getSearchScopeForWorkspace(
          workspaceId,
          defaultScope ?? "global",
        ),
      );
    },
    { immediate: true },
  );

  watch(
    currentWorkspaceId,
    (workspaceId) => {
      if (!workspaceId) {
        sidebarOpenBranchIds.value = [];
        sidebarOpenSectionId.value = null;
        return;
      }

      if (hasRestoredInitialWorkspace.value) {
        readingState.setCurrentWorkspaceId(workspaceId);
      }

      const restoredSlug = readingState.getSelectedDocForWorkspace(workspaceId);
      pendingRestoreWorkspaceId.value = restoredSlug ? workspaceId : "";
      pendingRestoreDocSlug.value = restoredSlug;

      const restoredSidebarState =
        readingState.getSidebarStateForWorkspace(workspaceId);
      sidebarOpenBranchIds.value = restoredSidebarState?.openBranchIds ?? [];
      sidebarOpenSectionId.value = restoredSidebarState?.openSectionId ?? null;
    },
    { immediate: true },
  );

  watch(
    [currentWorkspaceId, selectedDocSlug],
    ([workspaceId, slug]) => {
      if (!workspaceId || !slug) {
        currentReaderScrollTop.value = 0;
        restoredScrollTop.value = 0;
        return;
      }

      if (
        pendingRestoreWorkspaceId.value === workspaceId &&
        pendingRestoreDocSlug.value &&
        pendingRestoreDocSlug.value !== slug
      ) {
        return;
      }

      const currentDocMeta = workspaceDocs.docsBySlug.value[slug];
      if (
        !currentDocMeta ||
        !currentWorkspaceSourceIds.value.includes(currentDocMeta.sourceId)
      ) {
        return;
      }

      readingState.setSelectedDocForWorkspace(workspaceId, slug);
      readingState.recordRecentDoc(workspaceId, slug);
      restoredScrollTop.value = readingState.getDocScrollTop(workspaceId, slug);
      currentReaderScrollTop.value = restoredScrollTop.value;
    },
    { immediate: true },
  );

  watch(
    [currentWorkspaceId, scope],
    ([workspaceId, nextScope]) => {
      if (!workspaceId) {
        return;
      }

      readingState.setSearchScopeForWorkspace(workspaceId, nextScope);
    },
    { immediate: true },
  );

  watch(
    [currentWorkspaceId, sidebarOpenBranchIds, sidebarOpenSectionId],
    ([workspaceId, openBranchIds, openSectionId]) => {
      if (!workspaceId) {
        return;
      }

      readingState.setSidebarStateForWorkspace(workspaceId, {
        openBranchIds,
        openSectionId,
      });
    },
    { deep: true, immediate: true },
  );

  function countWorkspaceFolderSources(nodes: WorkspaceSourceNode[]): number {
    return nodes.reduce((count, node) => {
      const selfCount = node.kind === "folder" ? 1 : 0;
      return count + selfCount + countWorkspaceFolderSources(node.children);
    }, 0);
  }

  async function restoreInitialWorkspace() {
    const restoredWorkspaceId = readingState.currentWorkspaceId.value;
    await ensureLoaded();

    if (
      restoredWorkspaceId &&
      restoredWorkspaceId !== currentWorkspaceId.value &&
      workspaces.value.some((workspace) => workspace.id === restoredWorkspaceId)
    ) {
      await selectWorkspace(restoredWorkspaceId);
    }

    const activeWorkspaceId = currentWorkspaceId.value;
    if (activeWorkspaceId) {
      readingState.setCurrentWorkspaceId(activeWorkspaceId);
    }

    hasRestoredInitialWorkspace.value = true;
  }

  function handleDocScrollTopChange(top: number) {
    const workspaceId = currentWorkspaceId.value;
    const slug = selectedDocSlug.value;

    currentReaderScrollTop.value = Math.max(0, Math.round(top));

    if (!workspaceId || !slug) {
      return;
    }

    readingState.setDocScrollTop(
      workspaceId,
      slug,
      currentReaderScrollTop.value,
    );
  }

  function persistCurrentDocScrollTop() {
    const workspaceId = currentWorkspaceId.value;
    const slug = selectedDocSlug.value;

    if (!workspaceId || !slug) {
      return;
    }

    restoredScrollTop.value = currentReaderScrollTop.value;
    readingState.setDocScrollTop(
      workspaceId,
      slug,
      currentReaderScrollTop.value,
    );
  }

  function restoreCurrentDocScrollTop() {
    const workspaceId = currentWorkspaceId.value;
    const slug = selectedDocSlug.value;

    if (!workspaceId || !slug) {
      return;
    }

    const savedScrollTop = readingState.getDocScrollTop(workspaceId, slug);
    restoredScrollTop.value = -1;
    void nextTick(() => {
      restoredScrollTop.value = savedScrollTop;
      currentReaderScrollTop.value = savedScrollTop;
    });
  }

  function waitForDocAvailability(slug: string, timeoutMs = 5000) {
    if (workspaceDocs.docsBySlug.value[slug]) {
      return Promise.resolve();
    }

    return new Promise<void>((resolve) => {
      const stop = watch(
        [workspaceDocs.docsBySlug, workspaceDocs.isLoading],
        ([docsBySlugValue, isLoadingValue]) => {
          if (
            docsBySlugValue[slug] ||
            (!isLoadingValue && Object.keys(docsBySlugValue).length > 0)
          ) {
            stop();
            resolve();
          }
        },
        { immediate: true },
      );

      window.setTimeout(() => {
        stop();
        resolve();
      }, timeoutMs);
    });
  }

  function createDocEntryId(
    workspaceId: string,
    slug: string,
  ): DesktopDocEntryKey {
    return `${workspaceId}::${slug}`;
  }

  function isTauriRuntime() {
    return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
  }
</script>

<template>
  <div class="desktop-app-shell">
    <header
      :class="[
        'desktop-titlebar',
        { 'desktop-titlebar--search-open': floatingPanelVisible },
      ]"
      data-tauri-drag-region
      @click="handleTitlebarClick"
      @mousedown="handleTitlebarMouseDown"
    >
      <div
        class="desktop-titlebar__left"
        data-tauri-drag-region
      >
        <span
          class="desktop-titlebar__traffic-gap"
          data-tauri-drag-region
        />
      </div>

      <div
        v-if="canUseTitlebarSearch"
        class="desktop-titlebar__search-shell"
        @mousedown.stop
        @click.stop
      >
        <label
          class="desktop-titlebar__search"
          @mousedown.stop
          @click.stop
        >
          <DesktopUiIcon
            class="desktop-titlebar__search-icon"
            name="search"
            :size="14"
          />
          <input
            ref="titlebarSearchInput"
            v-model="searchQuery"
            class="desktop-titlebar__search-input"
            placeholder="搜索文档…"
            type="search"
            @focus="onTitlebarSearchFocus"
            @keydown="onTitlebarSearchKeydown"
            @mousedown.stop
            @click.stop
          />
          <span class="desktop-titlebar__search-hint">⌘K</span>
        </label>

        <div
          v-if="isOpen"
          class="desktop-titlebar__search-dropdown"
        >
          <DesktopSearchPanel
            :query="searchQuery"
            :results="results"
            :scope="scope"
            :selected-index="selectedIndex"
            :source-filter="sourceFilter"
            :source-options="sourceOptions"
            :workspace-name="currentWorkspace?.name ?? '当前文档仓库'"
            :workspace-filter="workspaceFilter"
            :workspace-options="workspaceOptions"
            @close="closeSearch"
            @move-selection="moveSelection"
            @set-source-filter="setSourceFilter"
            @set-scope="setScope"
            @set-workspace-filter="setWorkspaceFilter"
            @submit="handleSubmitSearch"
          />
        </div>
      </div>

      <div
        class="desktop-titlebar__actions"
        @mousedown.stop
        @click.stop
      >
        <button
          aria-label="打开设置"
          :class="[
            'desktop-titlebar__icon-button',
            { 'desktop-titlebar__icon-button--active': isSettingsView },
          ]"
          type="button"
          @mousedown.stop
          @dblclick.stop
          @click="toggleSettingsPanel"
        >
          <DesktopUiIcon name="settings" :size="19" />
        </button>
      </div>
    </header>

    <div
      v-if="floatingPanelVisible"
      class="desktop-floating-layer"
      @click="closeFloatingPanels"
    />

    <div
      class="desktop-workbench"
      :class="{
        'desktop-workbench--rail-only': isRecentView || isFavoritesView,
        'desktop-workbench--settings': isSettingsView,
        'desktop-workbench--tools': isToolView,
      }"
      :style="workbenchStyle"
    >
      <template v-if="isToolView">
        <main class="desktop-workbench__main desktop-workbench__main--tools">
          <DesktopToolHubView
            v-if="isToolsHubView"
            @open-tool="handleOpenTool"
          />

          <DesktopVideoToolView
            v-else-if="isVideoToolView"
            @back-to-tools="openToolsView"
          />

          <DesktopAgentSessionCleanerView
            v-else-if="isAgentSessionsToolView"
            @back-to-tools="openToolsView"
          />
        </main>
      </template>

      <template v-else-if="!isSettingsView">
        <aside
          class="desktop-workbench__sidebar"
          :style="readerSidebarStyle"
        >
          <DesktopDocsSidebar
            v-model:open-branch-ids="sidebarOpenBranchIds"
            v-model:open-section-id="sidebarOpenSectionId"
            :active-view="sidebarActiveView"
            :current-doc-slug="selectedDocSlug || null"
            :favorite-count="favoriteEntries.length"
            :current-workspace-doc-count="docCount"
            :current-section-id="currentSectionId"
            :current-source-id="currentSourceId"
            :current-workspace-id="currentWorkspaceId"
            :current-workspace-unhealthy-source-count="
              workspaceDocs.unhealthySourceCount.value
            "
            :current-workspace-source-count="sourceCount"
            :recent-count="recentEntries.length"
            :source-groups="visibleSourceGroups"
            :workspaces="workspaces"
            @create-workspace="openCreateWorkspaceDialog"
            @edit-workspace="openEditWorkspaceDialog"
            @open-favorites="openFavoritesView"
            @open-reader="openReaderView"
            @open-recent="openRecentView"
            @back-to-tools="openToolsView"
            @select-doc="handleSelectDoc"
            @select-workspace="handleSelectWorkspace"
          />
        </aside>

        <div
          v-if="isReaderView"
          aria-orientation="vertical"
          aria-label="调整目录宽度"
          class="desktop-panel-resizer desktop-panel-resizer--sidebar"
          role="separator"
          @mousedown="startSidebarResize"
        />

        <main
          class="desktop-workbench__main"
          :class="{
            'desktop-workbench__main--reader': isReaderView,
            'desktop-workbench__main--page': !isReaderView,
            'desktop-workbench__main--with-toc': showReaderToc,
          }"
        >
          <template v-if="isReaderView">
            <div
              v-if="showReaderToc"
              class="desktop-reader-split"
            >
              <div class="desktop-reader-split__content">
                <DesktopDocReader
                  :doc="currentDoc"
                  :is-loading="isReaderLoading"
                  :is-favorite="currentDocIsFavorite"
                  :highlight-query="query"
                  :next-doc="nextDoc"
                  :prev-doc="prevDoc"
                  :markdown-theme-id="preferences.markdownThemeId"
                  :restore-scroll-top="restoredScrollTop"
                  :save-doc="handleSaveCurrentDoc"
                  @select-doc="handleSelectDoc"
                  @scroll-top-change="handleDocScrollTopChange"
                  @toggle-favorite="handleToggleCurrentDocFavorite"
                />
              </div>

              <div
                aria-orientation="vertical"
                aria-label="调整大纲宽度"
                class="desktop-panel-resizer desktop-panel-resizer--toc"
                role="separator"
                @mousedown="startTocResize"
              />

              <aside
                class="desktop-workbench__toc"
                :style="{ width: `${tocWidth}px` }"
              >
              <div
                v-if="isReaderLoading"
                class="desktop-workbench__toc-loading"
                aria-hidden="true"
              >
                <span class="desktop-workbench__toc-loading-title" />
                <div class="desktop-workbench__toc-loading-list">
                  <span class="desktop-workbench__toc-loading-line desktop-workbench__toc-loading-line--wide" />
                  <span class="desktop-workbench__toc-loading-line" />
                  <span class="desktop-workbench__toc-loading-line desktop-workbench__toc-loading-line--child" />
                  <span class="desktop-workbench__toc-loading-line desktop-workbench__toc-loading-line--soft" />
                  <span class="desktop-workbench__toc-loading-line desktop-workbench__toc-loading-line--child" />
                  <span class="desktop-workbench__toc-loading-line desktop-workbench__toc-loading-line--wide" />
                </div>
              </div>

              <DesktopDocToc
                v-else
                :active-id="activeId"
                :headings="headings"
                @select="scrollToHeading"
              />
              </aside>
            </div>

            <DesktopDocReader
              v-else
              :doc="currentDoc"
              :is-loading="isReaderLoading"
              :is-favorite="currentDocIsFavorite"
              :highlight-query="query"
              :next-doc="nextDoc"
              :prev-doc="prevDoc"
              :markdown-theme-id="preferences.markdownThemeId"
              :restore-scroll-top="restoredScrollTop"
              :save-doc="handleSaveCurrentDoc"
              @select-doc="handleSelectDoc"
              @scroll-top-change="handleDocScrollTopChange"
              @toggle-favorite="handleToggleCurrentDocFavorite"
            />
          </template>

          <DesktopRecentView
            v-else-if="isRecentView"
            :entries="recentEntries"
            @back-to-reader="closeRecentView"
            @open-entry="handleOpenRecentEntry"
          />

          <DesktopFavoritesView
            v-else
            :entries="favoriteEntries"
            @back-to-reader="closeFavoritesView"
            @open-entry="handleOpenFavoriteEntry"
            @remove-entry="handleRemoveFavoriteEntry"
          />
        </main>
      </template>

      <DesktopSettingsView
        v-else
        :accent-id="preferences.accentId"
        :accent-options="accentOptions"
        :action-message="settingsActionMessage"
        :active-section="settingsSection"
        :busy-action="settingsBusyAction"
        :current-version="currentVersion"
        :custom-accent-color="preferences.customAccentColor"
        :last-checked-at="lastCheckedAt"
        :latest-release="latestRelease"
        :markdown-theme-id="preferences.markdownThemeId"
        :markdown-theme-options="markdownThemeOptions"
        :theme-mode="preferences.themeMode"
        :update-message="updateMessage"
        :update-status="updateStatus"
        @check-updates="checkForUpdates"
        @close="closeSettingsView"
        @export-logs="handleExportLogsFile"
        @install-update="installUpdate"
        @open-latest-release="openLatestRelease"
        @open-app-data-directory="handleOpenAppDataDirectory"
        @open-logs-directory="handleOpenLogsDirectory"
        @select-section="settingsSection = $event"
        @update-accent="setAccent"
        @update-custom-accent-color="setCustomAccentColor"
        @update-markdown-theme="setMarkdownTheme"
        @update-theme-mode="setThemeMode"
      />
    </div>

    <DesktopWorkspaceDialog
      v-model:open="isWorkspaceDialogOpen"
      :accent-options="accentOptions"
      :can-delete="workspaces.length > 1"
      :is-deleting="isDeletingWorkspace"
      :is-exporting="isExportingWorkspace"
      :is-importing="isImportingWorkspace"
      :is-saving="isSavingWorkspace"
      :doc-count="docCount"
      :mode="workspaceDialogMode"
      :source-count="sourceCount"
      :unhealthy-source-count="workspaceDocs.unhealthySourceCount.value"
      :workspace-count="workspaces.length"
      :workspace="workspaceDialogWorkspace"
      @close="isWorkspaceDialogOpen = false"
      @delete="handleDeleteWorkspace"
      @export="handleExportWorkspace"
      @import="handleImportWorkspace"
      @submit="handleCreateWorkspace"
    />
  </div>
</template>

<style scoped>
  .desktop-app-shell {
    position: relative;
    display: grid;
    grid-template-rows: 38px minmax(0, 1fr);
    height: 100vh;
    overflow: hidden;
  }

  .desktop-titlebar {
    position: relative;
    z-index: 10;
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    gap: 0.5rem;
    padding: 0 0.7rem 0 0.12rem;
    background: var(--desktop-titlebar-bg-runtime, var(--desktop-titlebar-bg));
    border-bottom: 1px solid var(--desktop-titlebar-line);
    box-shadow: none;
    user-select: none;
  }

  .desktop-titlebar--search-open {
    z-index: 30;
  }

  .desktop-titlebar__traffic-gap {
    width: 72px;
    height: 100%;
  }

  .desktop-titlebar__left {
    grid-column: 1;
    justify-self: start;
    display: flex;
    align-items: center;
    height: 100%;
    min-width: 0;
  }

  .desktop-titlebar__search-shell {
    position: relative;
    grid-column: 2;
    width: clamp(20rem, 38vw, 56rem);
    max-width: calc(100vw - 8rem);
  }

  .desktop-titlebar__search {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    width: 100%;
    height: 1.75rem;
    padding: 0 0.62rem;
    border: 1px solid var(--desktop-titlebar-search-border, var(--desktop-titlebar-control-border));
    border-radius: var(--desktop-radius-sm);
    background: var(--desktop-titlebar-search-bg, transparent);
    color: var(--desktop-titlebar-search-ink, var(--desktop-titlebar-control-ink));
    cursor: text;
    transition:
      background-color 0.15s ease,
      border-color 0.15s ease;
  }

  .desktop-titlebar__search-dropdown {
    position: absolute;
    top: calc(100% + 0.45rem);
    left: 0;
    width: 100%;
  }

  .desktop-titlebar__search:focus-within {
    background: var(
      --desktop-titlebar-search-bg-focus,
      var(--desktop-titlebar-control-bg-hover)
    );
    border-color: transparent;
  }

  .desktop-titlebar__search-icon {
    flex: none;
    opacity: 0.68;
  }

  .desktop-titlebar__search-input {
    flex: 1;
    min-width: 0;
    height: 100%;
    border: 0;
    background: transparent;
    color: var(--desktop-titlebar-search-input-ink, var(--desktop-titlebar-control-ink-hover));
    font-size: 0.75rem;
    outline: none;
  }

  .desktop-titlebar__search-input::placeholder {
    color: var(--desktop-titlebar-search-ink, var(--desktop-titlebar-control-ink));
    opacity: 0.72;
  }

  .desktop-titlebar__search-input::-webkit-search-decoration,
  .desktop-titlebar__search-input::-webkit-search-cancel-button,
  .desktop-titlebar__search-input::-webkit-search-results-button,
  .desktop-titlebar__search-input::-webkit-search-results-decoration {
    -webkit-appearance: none;
  }

  .desktop-titlebar__search-hint {
    flex: none;
    margin-left: auto;
    color: inherit;
    font-size: 0.69rem;
    opacity: 0.4;
  }

  .desktop-titlebar__actions {
    grid-column: 3;
    justify-self: end;
    display: flex;
    align-items: center;
    gap: 0.4rem;
    height: 100%;
  }

  .desktop-titlebar__icon-button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 1.65rem;
    height: 1.65rem;
    border: 0;
    border-radius: var(--desktop-radius-sm);
    background: var(--desktop-titlebar-control-bg);
    border: 1px solid var(--desktop-titlebar-control-border);
    color: var(--desktop-titlebar-control-ink);
    cursor: pointer;
    transition:
      background-color 0.15s ease,
      color 0.15s ease;
  }

  .desktop-titlebar__icon-button svg {
    width: 1rem;
    height: 1rem;
  }

  .desktop-titlebar__icon-button:hover,
  .desktop-titlebar__icon-button--active {
    background: var(--desktop-titlebar-control-bg-hover);
    color: var(--desktop-titlebar-control-ink-hover);
  }

  .desktop-floating-layer {
    position: absolute;
    inset: 38px 0 0;
    z-index: 25;
    background: rgba(var(--desktop-shadow), 0.08);
  }

  .desktop-workbench {
    position: relative;
    display: grid;
    grid-template-columns: var(--desktop-sidebar-w) minmax(0, 1fr);
    gap: 0;
    min-height: 0;
    height: 100%;
    padding: 0;
    overflow: hidden;
    background: var(--desktop-bg);
  }

  .desktop-workbench::before {
    display: none;
  }

  .desktop-workbench::after {
    display: none;
  }

  :global(:root[data-theme="dark"]) .desktop-workbench {
    background: var(--desktop-bg);
  }

  .desktop-workbench > * {
    position: relative;
    z-index: 1;
  }

  .desktop-workbench--settings {
    grid-template-columns: unset !important;
  }

  .desktop-workbench--tools {
    grid-template-columns: minmax(0, 1fr) !important;
  }

  .desktop-workbench--rail-only {
    grid-template-columns: var(--desktop-rail-w) minmax(0, 1fr);
  }

  .desktop-workbench--rail-only .desktop-workbench__sidebar {
    width: var(--desktop-rail-w);
    min-width: var(--desktop-rail-w);
  }

  .desktop-workbench__sidebar,
  .desktop-workbench__main {
    min-height: 0;
    min-width: 0;
  }

  .desktop-workbench__sidebar {
    overflow: hidden;
  }

  .desktop-workbench__main {
    display: grid;
    min-width: 0;
    min-height: 0;
    height: 100%;
    overflow: hidden;
    background: transparent;
  }

  .desktop-workbench__main--reader {
    min-height: 0;
    height: 100%;
    padding: 0;
    grid-template-rows: minmax(0, 1fr);
  }

  .desktop-workbench__main--page {
    padding: 0;
    border: 0;
    border-radius: 0;
    background: var(--desktop-surface-strong);
    box-shadow: none;
    overflow: hidden;
    height: 100%;
    grid-template-rows: minmax(0, 1fr);
  }

  .desktop-workbench__main--tools {
    min-height: 0;
    height: 100%;
    padding: 0;
    grid-template-rows: minmax(0, 1fr);
    overflow: hidden;
    background: var(--desktop-bg);
  }

  .desktop-workbench__main--with-toc {
    display: block;
    min-height: 0;
  }

  .desktop-reader-split {
    display: flex;
    align-items: stretch;
    min-width: 0;
    min-height: 0;
    height: 100%;
  }

  .desktop-reader-split__content {
    display: flex;
    flex-direction: column;
    flex: 1 1 auto;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
  }

  .desktop-panel-resizer {
    position: relative;
    z-index: 2;
    flex: none;
    width: 5px;
    margin: 0 -2px;
    cursor: col-resize;
    touch-action: none;
    background: transparent;
  }

  .desktop-panel-resizer::after {
    content: "";
    position: absolute;
    inset: 0 auto 0 50%;
    width: 1px;
    transform: translateX(-50%);
    background: var(--desktop-line);
    opacity: 0;
    transition:
      opacity 0.15s ease,
      background-color 0.15s ease;
  }

  .desktop-panel-resizer:hover::after,
  .desktop-panel-resizer:active::after {
    opacity: 1;
    background: var(--desktop-accent);
  }

  .desktop-workbench__toc {
    flex: none;
    min-width: 0;
    min-height: 0;
    border-left: 1px solid var(--desktop-line);
    border-top: 0;
    border-right: 0;
    border-bottom: 0;
    border-radius: 0;
    background: var(--desktop-surface);
    box-shadow: none;
    overflow: hidden;
  }

  .desktop-workbench__toc-loading,
  .desktop-workbench__toc-loading-list {
    display: grid;
    gap: 0.72rem;
  }

  .desktop-workbench__toc-loading {
    align-content: start;
    min-height: 100%;
    padding: 0.94rem 0.88rem;
  }

  .desktop-workbench__toc-loading-title,
  .desktop-workbench__toc-loading-line {
    display: block;
    border-radius: 999px;
    background: linear-gradient(
      90deg,
      rgba(var(--desktop-accent-rgb), 0.08),
      rgba(var(--desktop-accent-rgb), 0.16),
      rgba(var(--desktop-accent-rgb), 0.08)
    );
    background-size: 220% 100%;
    animation: desktop-doc-reader-loading 1.25s linear infinite;
  }

  .desktop-workbench__toc-loading-title {
    width: 6.8rem;
    height: 0.82rem;
    margin-bottom: 0.2rem;
  }

  .desktop-workbench__toc-loading-line {
    width: 100%;
    height: 0.74rem;
  }

  .desktop-workbench__toc-loading-line--wide {
    width: 88%;
  }

  .desktop-workbench__toc-loading-line--soft {
    width: 76%;
  }

  .desktop-workbench__toc-loading-line--child {
    width: 72%;
    margin-left: 0.92rem;
  }

  @keyframes desktop-doc-reader-loading {
    0% {
      background-position: 200% 0;
    }

    100% {
      background-position: -40% 0;
    }
  }
</style>
