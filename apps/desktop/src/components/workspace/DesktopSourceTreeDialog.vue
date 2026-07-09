<script setup lang="ts">
  import { computed, onBeforeUnmount, reactive, watch } from "vue";
  import type {
    WorkspaceDetail,
    WorkspaceSourceStatus,
  } from "@docs-atlas/shared-types/workspace";
  import {
    pickFolderPath,
    pickFolderPaths,
    validateSourcePath,
  } from "@/api/workspaces";
  import DesktopUiIcon from "@/components/ui/DesktopUiIcon.vue";
  import DesktopSourceTreeNodeEditor from "./DesktopSourceTreeNodeEditor.vue";
  import {
    collectSourceTreeIssues,
    countDraftGroups,
    countDraftSources,
    createFolderDraft,
    createGroupDraft,
    type DraftNodeDropPlacement,
    findDraftNode,
    flattenDraftNodes,
    inferFolderDisplayName,
    insertDraftChild,
    moveDraftNodeByDrop,
    moveDraftNode,
    normalizeDraftPositions,
    removeDraftNode,
    serializeWorkspaceSources,
    syncDraftParentIds,
    cloneSourceNodes,
    type WorkspaceSourceNodeDraft,
  } from "@/utils/workspaceTree";

  const isOpen = defineModel<boolean>("open", { default: false });

  const props = defineProps<{
    isSaving: boolean;
    isScanning: boolean;
    runtimeSourceStatusesByNodeId: Record<
      string,
      WorkspaceSourceStatus | undefined
    >;
    workspace: WorkspaceDetail | null;
  }>();

  const emit = defineEmits<{
    close: [];
    save: [sources: ReturnType<typeof serializeWorkspaceSources>];
  }>();

  const state = reactive({
    draftNodes: [] as WorkspaceSourceNodeDraft[],
    dragOverNodeId: null as string | null,
    dragPlacement: null as DraftNodeDropPlacement | null,
    draggedNodeId: null as string | null,
    expandedNodeIds: [] as string[],
    feedbackMessage: "",
    feedbackTone: "muted" as "muted" | "success" | "warning",
    pathStatuses: {} as Record<
      string,
      { exists: boolean; isDirectory: boolean } | undefined
    >,
    pathValidationTokens: {} as Record<string, number>,
    validatingPathIds: {} as Record<string, boolean | undefined>,
  });

  let dragExpandTimer: ReturnType<typeof window.setTimeout> | null = null;
  let dragExpandTargetId: string | null = null;

  const issues = computed(() =>
    collectSourceTreeIssues(state.draftNodes, state.pathStatuses),
  );
  const issuesByNodeId = computed<Record<string, string[]>>(() =>
    issues.value.reduce<Record<string, string[]>>((result, issue) => {
      result[issue.nodeId] = [...(result[issue.nodeId] ?? []), issue.message];
      return result;
    }, {}),
  );
  const folderPathSnapshot = computed(() =>
    flattenDraftNodes(state.draftNodes)
      .filter((node) => node.kind === "folder")
      .map((node) => ({
        id: node.id,
        path: node.path.trim(),
      })),
  );
  const sourceCount = computed(() => countDraftSources(state.draftNodes));
  const groupCount = computed(() => countDraftGroups(state.draftNodes));
  const hasErrors = computed(() =>
    issues.value.some((issue) => issue.severity === "error"),
  );
  const canSave = computed(
    () => !props.isSaving && !hasErrors.value && props.workspace !== null,
  );

  watch(
    () => [isOpen.value, props.workspace?.id] as const,
    ([open]) => {
      if (!open || !props.workspace) {
        return;
      }

      state.draftNodes = cloneSourceNodes(props.workspace.sources);
      state.expandedNodeIds = collectExpandedNodeIds(state.draftNodes);
      resetDragState();
      clearFeedback();
      state.pathStatuses = {};
      void validateAllFolderNodes();
    },
    { immediate: true },
  );

  watch(folderPathSnapshot, (entries, previousEntries = []) => {
    if (!isOpen.value) {
      return;
    }

    const previousMap = new Map(
      previousEntries.map((entry) => [entry.id, entry.path]),
    );
    const nextNodeIds = new Set(entries.map((entry) => entry.id));

    entries.forEach((entry) => {
      if (previousMap.get(entry.id) !== entry.path) {
        syncFolderNameFromPath(
          entry.id,
          entry.path,
          previousMap.get(entry.id) ?? "",
        );
        void validateFolderNode(entry.id, entry.path);
      }
    });

    Object.keys(state.pathStatuses).forEach((nodeId) => {
      if (!nextNodeIds.has(nodeId)) {
        delete state.pathStatuses[nodeId];
        delete state.pathValidationTokens[nodeId];
        delete state.validatingPathIds[nodeId];
      }
    });
  });

  async function handleAddRootGroup() {
    const draft = createGroupDraft(null, state.draftNodes.length);
    state.draftNodes = insertDraftChild(state.draftNodes, null, () => draft);
    syncDraftParentIds(state.draftNodes);
    ensureNodeExpanded(draft.id);
    setFeedback("已添加根分组。");
  }

  function handleAddRootFolder() {
    addFolderNode(null);
  }

  async function handleAddRootFolders() {
    const folderPaths = await pickFolderPaths();
    if (folderPaths.length === 0) {
      return;
    }

    const { addedCount, skippedCount } = appendFolderNodes(null, folderPaths);
    if (addedCount === 0 && skippedCount > 0) {
      setFeedback(
        `没有导入新目录，已跳过 ${skippedCount} 个重复路径。`,
        "warning",
      );
      return;
    }

    setFeedback(
      skippedCount > 0
        ? `已导入 ${addedCount} 个目录，跳过 ${skippedCount} 个重复路径。`
        : `已导入 ${addedCount} 个目录。`,
      "success",
    );

    await validateAllFolderNodes();
  }

  async function handleAddNestedGroup(parentId: string) {
    const parentNode = findDraftNode(state.draftNodes, parentId);
    const draft = createGroupDraft(parentId, parentNode?.children.length ?? 0);
    state.draftNodes = insertDraftChild(
      state.draftNodes,
      parentId,
      () => draft,
    );
    syncDraftParentIds(state.draftNodes);
    ensureNodeExpanded(parentId);
    ensureNodeExpanded(draft.id);
    setFeedback("已添加子分组。");
  }

  function handleAddNestedFolder(parentId: string) {
    addFolderNode(parentId);
  }

  async function handleBrowseFolder(nodeId: string) {
    const folderPath = await pickFolderPath();
    if (!folderPath) {
      return;
    }

    const target = findDraftNode(state.draftNodes, nodeId);
    if (!target) {
      return;
    }

    target.path = folderPath;
    if (!target.name.trim() || target.name === "新文档源") {
      target.name = inferUniqueFolderName(
        folderPath,
        target.parentId,
        target.id,
      );
    }
    await validateFolderNode(target.id, target.path);
  }

  function handleRemoveNode(nodeId: string) {
    const targetNode = findDraftNode(state.draftNodes, nodeId);
    state.draftNodes = removeDraftNode(state.draftNodes, nodeId);
    if (targetNode) {
      const relatedNodeIds = flattenDraftNodes([targetNode]).map(
        (node) => node.id,
      );
      const relatedNodeIdSet = new Set(relatedNodeIds);
      state.expandedNodeIds = state.expandedNodeIds.filter(
        (id) => !relatedNodeIdSet.has(id),
      );
      relatedNodeIds.forEach((relatedNodeId) => {
        delete state.pathStatuses[relatedNodeId];
        delete state.pathValidationTokens[relatedNodeId];
        delete state.validatingPathIds[relatedNodeId];
      });
    }
    syncDraftParentIds(state.draftNodes);
    setFeedback("已删除当前节点。");
  }

  function handleMoveNode(nodeId: string, direction: -1 | 1) {
    const nextNodes = moveDraftNode(state.draftNodes, nodeId, direction);
    if (nextNodes === state.draftNodes) {
      return;
    }

    state.draftNodes = nextNodes;
    syncDraftParentIds(state.draftNodes);
    setFeedback(direction < 0 ? "已上移节点。" : "已下移节点。", "success");
  }

  function handleDragStart(nodeId: string) {
    if (props.isSaving) {
      return;
    }

    state.draggedNodeId = nodeId;
    bindDragListeners();
    document.body.classList.add("desktop-source-tree-dialog--dragging");
  }

  function handleWindowPointerMove(event: PointerEvent) {
    if (!state.draggedNodeId) {
      return;
    }

    const nextTarget = resolveDragTarget(event.clientX, event.clientY);
    if (!nextTarget || nextTarget.targetNodeId === state.draggedNodeId) {
      clearActiveDropTarget();
      return;
    }

    state.dragOverNodeId = nextTarget.targetNodeId;
    state.dragPlacement = nextTarget.placement;

    if (nextTarget.placement === "inside") {
      scheduleDragExpand(nextTarget.targetNodeId);
      return;
    }

    clearDragExpandTimer();
  }

  function handleWindowPointerUp() {
    if (!state.draggedNodeId) {
      return;
    }

    if (state.dragOverNodeId && state.dragPlacement) {
      const nextNodes = moveDraftNodeByDrop(
        state.draftNodes,
        state.draggedNodeId,
        state.dragOverNodeId,
        state.dragPlacement,
      );
      const didMove = nextNodes !== state.draftNodes;
      state.draftNodes = nextNodes;
      syncDraftParentIds(state.draftNodes);
      if (didMove && state.dragPlacement === "inside") {
        ensureNodeExpanded(state.dragOverNodeId);
      }
      if (didMove) {
        setFeedback("已更新文档源层级。", "success");
      }
    }

    resetDragState();
  }

  function resetDragState() {
    unbindDragListeners();
    clearDragExpandTimer();
    clearActiveDropTarget();
    state.draggedNodeId = null;
    document.body.classList.remove("desktop-source-tree-dialog--dragging");
  }

  function handleClose() {
    isOpen.value = false;
    emit("close");
  }

  function handleSave() {
    if (!canSave.value) {
      return;
    }

    normalizeDraftPositions(state.draftNodes);
    syncDraftParentIds(state.draftNodes);
    emit("save", serializeWorkspaceSources(state.draftNodes));
  }

  function addFolderNode(parentId: string | null) {
    const parentNode = parentId
      ? findDraftNode(state.draftNodes, parentId)
      : null;
    const draft = createFolderDraft(
      parentId,
      parentNode?.children.length ?? state.draftNodes.length,
    );
    state.draftNodes = insertDraftChild(
      state.draftNodes,
      parentId,
      () => draft,
    );
    syncDraftParentIds(state.draftNodes);
    if (parentId) {
      ensureNodeExpanded(parentId);
    }
    setFeedback(parentId ? "已添加子目录卡片。" : "已添加根目录卡片。");
  }

  function appendFolderNodes(parentId: string | null, folderPaths: string[]) {
    let addedCount = 0;
    let skippedCount = 0;
    const normalizedExistingPaths = new Set(
      flattenDraftNodes(state.draftNodes)
        .filter((node) => node.kind === "folder")
        .map((node) => normalizePathValue(node.path)),
    );

    for (const rawPath of folderPaths) {
      const normalizedPath = normalizePathValue(rawPath);
      if (!normalizedPath || normalizedExistingPaths.has(normalizedPath)) {
        skippedCount += 1;
        continue;
      }

      normalizedExistingPaths.add(normalizedPath);
      const draft = createFolderDraft(
        parentId,
        getSiblingNodes(parentId).length,
        rawPath,
        inferUniqueFolderName(rawPath, parentId),
      );
      state.draftNodes = insertDraftChild(
        state.draftNodes,
        parentId,
        () => draft,
      );
      syncDraftParentIds(state.draftNodes);
      if (parentId) {
        ensureNodeExpanded(parentId);
      }
      addedCount += 1;
    }

    return { addedCount, skippedCount };
  }

  async function validateAllFolderNodes() {
    const folderNodes = flattenDraftNodes(state.draftNodes).filter(
      (node) => node.kind === "folder",
    );
    await Promise.all(
      folderNodes.map((node) => validateFolderNode(node.id, node.path)),
    );
  }

  function syncFolderNameFromPath(
    nodeId: string,
    nextPath: string,
    previousPath: string,
  ) {
    const target = findDraftNode(state.draftNodes, nodeId);
    if (!target || target.kind !== "folder") {
      return;
    }

    const normalizedNextPath = nextPath.trim();
    const normalizedPreviousPath = previousPath.trim();
    const nextName = normalizedNextPath
      ? inferFolderDisplayName(normalizedNextPath)
      : "新文档源";
    const previousAutoName = normalizedPreviousPath
      ? inferFolderDisplayName(normalizedPreviousPath)
      : "新文档源";

    if (
      !target.name.trim() ||
      target.name === "新文档源" ||
      target.name === previousAutoName
    ) {
      target.name = normalizedNextPath
        ? inferUniqueFolderName(normalizedNextPath, target.parentId, target.id)
        : nextName;
    }
  }

  async function validateFolderNode(nodeId: string, pathValue: string) {
    if (!pathValue.trim()) {
      state.pathStatuses[nodeId] = undefined;
      delete state.validatingPathIds[nodeId];
      return;
    }

    const token = (state.pathValidationTokens[nodeId] ?? 0) + 1;
    state.pathValidationTokens[nodeId] = token;
    state.validatingPathIds[nodeId] = true;

    const status = await validateSourcePath(pathValue);
    const target = findDraftNode(state.draftNodes, nodeId);
    const currentPath = target?.kind === "folder" ? target.path.trim() : "";

    if (
      state.pathValidationTokens[nodeId] !== token ||
      currentPath !== pathValue.trim()
    ) {
      return;
    }

    state.pathStatuses[nodeId] = status;
    delete state.validatingPathIds[nodeId];
  }

  function inferUniqueFolderName(
    pathValue: string,
    parentId: string | null,
    excludeNodeId?: string,
  ) {
    const normalizedPath = pathValue.trim().replace(/[\\/]+$/, "");
    const segments = normalizedPath.split(/[\\/]/).filter(Boolean);
    const baseName = inferFolderDisplayName(normalizedPath);
    const siblingNames = new Set(
      getSiblingNodes(parentId)
        .filter((node) => node.id !== excludeNodeId)
        .map((node) => node.name.trim())
        .filter(Boolean),
    );

    if (!siblingNames.has(baseName)) {
      return baseName;
    }

    const parentSegment = segments.at(-2);
    const compositeName = parentSegment
      ? `${parentSegment} / ${baseName}`
      : baseName;
    if (!siblingNames.has(compositeName)) {
      return compositeName;
    }

    let counter = 2;
    let candidate = `${baseName} (${counter})`;
    while (siblingNames.has(candidate)) {
      counter += 1;
      candidate = `${baseName} (${counter})`;
    }

    return candidate;
  }

  function getSiblingNodes(parentId: string | null) {
    if (!parentId) {
      return state.draftNodes;
    }

    const parentNode = findDraftNode(state.draftNodes, parentId);
    return parentNode?.children ?? [];
  }

  function normalizePathValue(pathValue: string) {
    return pathValue
      .replace(/\\/g, "/")
      .replace(/\/+$/, "")
      .trim()
      .toLowerCase();
  }

  function toggleExpand(nodeId: string) {
    if (state.expandedNodeIds.includes(nodeId)) {
      state.expandedNodeIds = state.expandedNodeIds.filter(
        (id) => id !== nodeId,
      );
      return;
    }

    state.expandedNodeIds = [...state.expandedNodeIds, nodeId];
  }

  function ensureNodeExpanded(nodeId: string) {
    if (!nodeId || state.expandedNodeIds.includes(nodeId)) {
      return;
    }

    state.expandedNodeIds = [...state.expandedNodeIds, nodeId];
  }

  function collectExpandedNodeIds(nodes: WorkspaceSourceNodeDraft[]): string[] {
    return nodes.flatMap((node) => [
      ...(node.children.length > 0 ? [node.id] : []),
      ...collectExpandedNodeIds(node.children),
    ]);
  }

  function clearFeedback() {
    state.feedbackMessage = "";
    state.feedbackTone = "muted";
  }

  function setFeedback(
    message: string,
    tone: "muted" | "success" | "warning" = "muted",
  ) {
    state.feedbackMessage = message;
    state.feedbackTone = tone;
  }

  function scheduleDragExpand(nodeId: string) {
    const targetNode = findDraftNode(state.draftNodes, nodeId);
    if (
      !targetNode ||
      targetNode.children.length === 0 ||
      state.expandedNodeIds.includes(nodeId)
    ) {
      clearDragExpandTimer();
      return;
    }

    if (dragExpandTargetId === nodeId && dragExpandTimer !== null) {
      return;
    }

    clearDragExpandTimer();
    dragExpandTargetId = nodeId;
    dragExpandTimer = window.setTimeout(() => {
      ensureNodeExpanded(nodeId);
      dragExpandTimer = null;
      dragExpandTargetId = null;
    }, 380);
  }

  function clearDragExpandTimer() {
    if (dragExpandTimer !== null) {
      window.clearTimeout(dragExpandTimer);
    }

    dragExpandTimer = null;
    dragExpandTargetId = null;
  }

  function clearActiveDropTarget() {
    state.dragOverNodeId = null;
    state.dragPlacement = null;
  }

  function resolveDragTarget(
    clientX: number,
    clientY: number,
  ): { targetNodeId: string; placement: DraftNodeDropPlacement } | null {
    const element = document.elementFromPoint(clientX, clientY);
    const nodeCard =
      element instanceof HTMLElement
        ? element.closest<HTMLElement>("[data-source-tree-node-id]")
        : null;

    if (!nodeCard) {
      return null;
    }

    const targetNodeId = nodeCard.dataset.sourceTreeNodeId;
    if (!targetNodeId) {
      return null;
    }

    const { top, height } = nodeCard.getBoundingClientRect();
    const offsetY = clientY - top;
    const threshold = Math.min(18, height * 0.24);
    const placement =
      offsetY <= threshold
        ? "before"
        : offsetY >= height - threshold
          ? "after"
          : "inside";

    return {
      targetNodeId,
      placement,
    };
  }

  function bindDragListeners() {
    unbindDragListeners();
    window.addEventListener("pointermove", handleWindowPointerMove);
    window.addEventListener("pointerup", handleWindowPointerUp);
    window.addEventListener("pointercancel", handleWindowPointerUp);
  }

  function unbindDragListeners() {
    window.removeEventListener("pointermove", handleWindowPointerMove);
    window.removeEventListener("pointerup", handleWindowPointerUp);
    window.removeEventListener("pointercancel", handleWindowPointerUp);
  }

  onBeforeUnmount(() => {
    resetDragState();
  });
</script>

<template>
  <div v-if="isOpen" class="desktop-source-tree-dialog" @click="handleClose">
    <section class="desktop-source-tree-dialog__panel" @click.stop>
      <header class="desktop-source-tree-dialog__header">
        <div class="desktop-source-tree-dialog__header-copy">
          <h2 class="desktop-source-tree-dialog__title">
            {{ props.workspace?.name ?? "当前文档仓库" }} 的文档源
          </h2>
          <p class="desktop-source-tree-dialog__summary">
            {{ groupCount }} 个分组 · {{ sourceCount }} 个目录源{{
              props.isScanning ? " · 检查中" : ""
            }}
          </p>
        </div>

        <button
          aria-label="关闭文档源编辑器"
          class="desktop-source-tree-dialog__close"
          type="button"
          @click="handleClose"
        >
          <DesktopUiIcon name="close" :size="16" />
        </button>
      </header>

      <div class="desktop-source-tree-dialog__toolbar">
        <button
          :disabled="props.isSaving"
          class="desktop-source-tree-dialog__tool"
          type="button"
          @click="handleAddRootGroup"
        >
          + 分组
        </button>
        <button
          :disabled="props.isSaving"
          class="desktop-source-tree-dialog__tool"
          type="button"
          @click="handleAddRootFolder"
        >
          + 目录
        </button>
        <button
          :disabled="props.isSaving"
          class="desktop-source-tree-dialog__tool"
          type="button"
          @click="handleAddRootFolders"
        >
          批量导入目录
        </button>
      </div>

      <div
        v-if="state.feedbackMessage"
        :class="[
          'desktop-source-tree-dialog__toolbar-note',
          `desktop-source-tree-dialog__toolbar-note--${state.feedbackTone}`,
        ]"
      >
        {{ state.feedbackMessage }}
      </div>

      <div
        v-if="state.draftNodes.length === 0"
        class="desktop-source-tree-dialog__empty"
      >
        还没有文档源，先添加一个目录，输入路径或直接选择本地文件夹即可。
      </div>

      <div v-else class="desktop-source-tree-dialog__tree desktop-scroll">
        <DesktopSourceTreeNodeEditor
          v-for="(node, index) in state.draftNodes"
          :key="node.id"
          v-model:node="state.draftNodes[index]"
          :depth="0"
          :disabled="props.isSaving"
          :drag-over-node-id="state.dragOverNodeId"
          :drag-placement="state.dragPlacement"
          :dragged-node-id="state.draggedNodeId"
          :expanded-node-ids="state.expandedNodeIds"
          :is-validating-path-by-node-id="state.validatingPathIds"
          :issues-by-node-id="issuesByNodeId"
          :sibling-count="state.draftNodes.length"
          :sibling-index="index"
          :path-statuses-by-node-id="state.pathStatuses"
          :runtime-source-statuses-by-node-id="
            props.runtimeSourceStatusesByNodeId
          "
          @add-folder="handleAddNestedFolder"
          @add-group="handleAddNestedGroup"
          @browse-folder="handleBrowseFolder"
          @drag-start="handleDragStart"
          @move-node="handleMoveNode"
          @remove-node="handleRemoveNode"
          @toggle-expand="toggleExpand"
        />
      </div>

      <footer class="desktop-source-tree-dialog__footer">
        <p
          v-if="issues.length > 0"
          class="desktop-source-tree-dialog__footer-note"
        >
          还有 {{ issues.length }} 个问题未处理
        </p>
        <div class="desktop-source-tree-dialog__footer-actions">
          <button
            class="desktop-source-tree-dialog__ghost"
            type="button"
            @click="handleClose"
          >
            取消
          </button>
          <button
            :disabled="!canSave"
            class="desktop-source-tree-dialog__primary"
            type="button"
            @click="handleSave"
          >
            {{ props.isSaving ? "保存中..." : "保存文档源" }}
          </button>
        </div>
      </footer>
    </section>
  </div>
</template>

<style scoped>
  .desktop-source-tree-dialog {
    position: fixed;
    inset: 0;
    z-index: 90;
    display: grid;
    place-items: center;
    padding: 1.2rem;
    background: rgba(7, 13, 24, 0.2);
  }

  .desktop-source-tree-dialog__panel {
    width: min(56rem, calc(100vw - 2rem));
    max-height: min(82vh, 56rem);
    display: flex;
    flex-direction: column;
    border: 1px solid var(--desktop-line);
    border-radius: var(--desktop-radius-md);
    background: var(--desktop-surface-strong);
    box-shadow: 0 16px 48px rgba(var(--desktop-shadow), 0.18);
    overflow: hidden;
  }

  .desktop-source-tree-dialog__header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 0.75rem;
    padding: 1.1rem 1.2rem 0.9rem;
    border-bottom: 1px solid var(--desktop-line-subtle, var(--desktop-line));
  }

  .desktop-source-tree-dialog__toolbar {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    padding: 0.72rem 1.2rem 0;
  }

  .desktop-source-tree-dialog__toolbar-note {
    padding: 0 1.2rem;
    color: var(--desktop-soft);
    font-size: 0.74rem;
    line-height: 1.45;
  }

  .desktop-source-tree-dialog__toolbar-note--success {
    color: #2f7b5f;
  }

  .desktop-source-tree-dialog__toolbar-note--warning {
    color: #b87529;
  }

  .desktop-source-tree-dialog__header-copy {
    display: grid;
    gap: 0.22rem;
    min-width: 0;
  }

  .desktop-source-tree-dialog__title,
  .desktop-source-tree-dialog__summary {
    margin: 0;
  }

  .desktop-source-tree-dialog__title {
    color: var(--desktop-ink);
    font-size: 1rem;
    font-weight: 600;
    line-height: 1.3;
  }

  .desktop-source-tree-dialog__summary {
    color: var(--desktop-soft);
    font-size: 0.76rem;
  }

  .desktop-source-tree-dialog__close {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 1.75rem;
    height: 1.75rem;
    border: 0;
    border-radius: var(--desktop-radius-sm);
    background: transparent;
    color: var(--desktop-soft);
    cursor: pointer;
    flex-shrink: 0;
  }

  .desktop-source-tree-dialog__close:hover {
    background: rgba(0, 0, 0, 0.04);
    color: var(--desktop-muted);
  }

  .desktop-source-tree-dialog__tool,
  .desktop-source-tree-dialog__ghost,
  .desktop-source-tree-dialog__primary {
    min-height: 1.88rem;
    padding: 0 0.72rem;
    border-radius: var(--desktop-radius-sm);
    font: inherit;
    font-size: 0.76rem;
    font-weight: 500;
    cursor: pointer;
  }

  .desktop-source-tree-dialog__tool,
  .desktop-source-tree-dialog__ghost {
    border: 1px solid var(--desktop-line);
    background: transparent;
    color: var(--desktop-muted);
  }

  .desktop-source-tree-dialog__tool:hover:not(:disabled),
  .desktop-source-tree-dialog__ghost:hover:not(:disabled) {
    background: rgba(0, 0, 0, 0.03);
    color: var(--desktop-ink);
  }

  .desktop-source-tree-dialog__primary {
    border: 1px solid var(--desktop-accent);
    background: var(--desktop-accent);
    color: #fff;
  }

  .desktop-source-tree-dialog__primary:disabled,
  .desktop-source-tree-dialog__tool:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .desktop-source-tree-dialog__tree {
    min-height: 0;
    overflow-y: auto;
    display: grid;
    gap: 0.42rem;
    padding: 0.72rem 1.2rem 0.2rem;
  }

  .desktop-source-tree-dialog__empty {
    margin: 0.72rem 1.2rem;
    padding: 0.85rem 0.9rem;
    border: 1px dashed var(--desktop-line);
    border-radius: var(--desktop-radius-sm);
    color: var(--desktop-soft);
    font-size: 0.78rem;
    background: transparent;
  }

  .desktop-source-tree-dialog__footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    padding: 0.72rem 1.2rem;
    border-top: 1px solid var(--desktop-line-subtle, var(--desktop-line));
  }

  .desktop-source-tree-dialog__footer-note {
    margin: 0;
    color: #b14656;
    font-size: 0.74rem;
  }

  .desktop-source-tree-dialog__footer-actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-left: auto;
  }
</style>
