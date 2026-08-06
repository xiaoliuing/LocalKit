<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, shallowRef, useTemplateRef, watch } from 'vue'
import type { FileViewerViewStateChange, FileViewerVue3Handle } from '@file-viewer/vue3'
import DesktopUiIcon from '@/components/ui/DesktopUiIcon.vue'
import DesktopFilePreviewFormatDialog from '@/components/tools/DesktopFilePreviewFormatDialog.vue'
import DesktopFilePreviewSourceDialog from '@/components/tools/DesktopFilePreviewSourceDialog.vue'
import DesktopFilePreviewSourceGroup from '@/components/tools/DesktopFilePreviewSourceGroup.vue'
import { getDesktopFilePreviewOptions } from '@/composables/useDesktopFilePreviewOptions'
import {
  isLegacyPlainTextPreview,
  readPreviewTextFile,
} from '@/api/filePreview'
import {
  countSupportedPreviewExtensions,
  isFilePreviewSupportedExtension,
} from '@/constants/filePreviewFormats'
import {
  useDesktopFilePreviewLibrary,
  type DesktopFilePreviewSource,
} from '@/composables/useDesktopFilePreviewLibrary'
import { bindPreviewTextSelection } from '@/utils/filePreviewTextSelection'

const emit = defineEmits<{
  backToTools: []
}>()

const {
  currentFile,
  currentFileId,
  currentFileUrl,
  currentViewStateMemory,
  expandedFolderIds,
  expandedSourceIds,
  feedbackMessage,
  isLibraryCollapsed,
  isScanning,
  sources,
  addSource,
  chooseFolderPath,
  removeSource,
  rememberViewState,
  rescanAllSources,
  rescanSource,
  restoreSources,
  selectFile,
  setFeedback,
  toggleFolder,
  toggleLibrary,
  toggleSource,
  updateSource,
} = useDesktopFilePreviewLibrary()

const previewViewer =
  useTemplateRef<FileViewerVue3Handle>('previewViewer')
const previewViewerHost = useTemplateRef<HTMLDivElement>('previewViewerHost')
const previewOptions = computed(() => ({
  ...getDesktopFilePreviewOptions(),
  initialViewState: currentViewStateMemory.value ?? undefined,
}))
const sourceDialog =
  useTemplateRef<InstanceType<typeof DesktopFilePreviewSourceDialog>>('sourceDialog')
const libraryTree = useTemplateRef<HTMLDivElement>('libraryTree')
const isSourceDialogOpen = shallowRef(false)
const isFormatDialogOpen = shallowRef(false)
const sourceDialogMode = shallowRef<'add' | 'edit'>('add')
const editingSource = shallowRef<DesktopFilePreviewSource | null>(null)
const previewFile = shallowRef<File | null>(null)
const previewSourceKey = shallowRef('')
const pendingViewStateFileId = shallowRef('')
const pendingViewState = shallowRef<FileViewerViewStateChange['state'] | null>(null)
let viewStateSaveTimer: ReturnType<typeof setTimeout> | undefined
let previewSelectionBinding: ReturnType<typeof bindPreviewTextSelection> | null = null

const previewSource = computed(() => {
  if (previewFile.value) {
    return { kind: 'file' as const, file: previewFile.value }
  }

  if (currentFileUrl.value) {
    return { kind: 'url' as const, url: currentFileUrl.value }
  }

  return null
})

const emptyStateTitle = computed(() =>
  sources.value.length === 0 ? '添加预览目录' : '选择一个文件开始预览',
)
const emptyStateSummary = computed(() =>
  sources.value.length === 0
    ? `从本机选择目录，支持 ${countSupportedPreviewExtensions()} 种常见格式预览。`
    : '目录已经准备好，请从左侧选择文件。',
)
const currentFileSupportLabel = computed(() => {
  const name = currentFile.value?.name
  if (!name) {
    return ''
  }

  return isFilePreviewSupportedExtension(name)
    ? '当前格式已支持预览'
    : '当前格式可能无法内联预览，可尝试下载'
})

onMounted(async () => {
  await restoreSources()
  await revealCurrentFileInTree()
})

onBeforeUnmount(() => {
  flushPendingViewState()
  previewSelectionBinding?.disconnect()
  previewSelectionBinding = null
})

function syncPreviewTextSelection() {
  previewSelectionBinding?.disconnect()
  previewSelectionBinding = bindPreviewTextSelection(previewViewerHost.value)
}

watch(currentFileId, async (_nextFileId, previousFileId) => {
  if (previousFileId) {
    flushPendingViewState(previousFileId)
  }
  await nextTick()
  syncPreviewTextSelection()
})

watch(
  () => [currentFileId.value, currentFile.value?.path, currentFile.value?.name] as const,
  async ([fileId, path, name]) => {
    previewFile.value = null
    previewSourceKey.value = fileId

    if (!fileId || !path || !name || !isLegacyPlainTextPreview(name)) {
      return
    }

    const decoded = await readPreviewTextFile(path)
    if (previewSourceKey.value !== fileId || !decoded) {
      return
    }

    previewFile.value = new File([decoded.content], name, {
      type: 'text/plain;charset=utf-8',
    })
  },
  { flush: 'post' },
)

async function handleChooseFolder() {
  const path = await chooseFolderPath()
  if (path) {
    sourceDialog.value?.setPath(path)
  }
}

function openSourceDialog() {
  sourceDialogMode.value = 'add'
  editingSource.value = null
  setFeedback('')
  isSourceDialogOpen.value = true
}

function openEditSourceDialog(sourceId: string) {
  const source = sources.value.find((item) => item.id === sourceId) ?? null
  if (!source) {
    return
  }

  sourceDialogMode.value = 'edit'
  editingSource.value = source
  setFeedback('')
  isSourceDialogOpen.value = true
}

async function handleSaveSource(input: {
  title: string
  path: string
  maxDepth: number
}) {
  const saved =
    sourceDialogMode.value === 'edit' && editingSource.value
      ? await updateSource(editingSource.value.id, input)
      : await addSource(input)

  if (saved) {
    isSourceDialogOpen.value = false
    editingSource.value = null
  }
}

function handleFileSelect(fileId: string) {
  flushPendingViewState()
  selectFile(fileId)
  void revealCurrentFileInTree()
}

function handleViewStateChange(change: FileViewerViewStateChange) {
  const fileId = currentFileId.value
  if (!fileId || change.action === 'init') {
    return
  }

  pendingViewStateFileId.value = fileId
  pendingViewState.value = change.state
  if (viewStateSaveTimer) {
    clearTimeout(viewStateSaveTimer)
  }

  viewStateSaveTimer = setTimeout(() => {
    flushPendingViewState()
  }, 400)
}

function flushPendingViewState(fileId = pendingViewStateFileId.value) {
  if (viewStateSaveTimer) {
    clearTimeout(viewStateSaveTimer)
    viewStateSaveTimer = undefined
  }

  const state = pendingViewState.value
  if (!fileId || !state) {
    pendingViewStateFileId.value = ''
    pendingViewState.value = null
    return
  }

  rememberViewState(fileId, state)
  pendingViewStateFileId.value = ''
  pendingViewState.value = null
}

async function handlePreviewLoadComplete() {
  syncPreviewTextSelection()

  const savedViewState = currentViewStateMemory.value
  const viewer = previewViewer.value
  if (!savedViewState || !viewer) {
    return
  }

  await viewer.applyViewState(savedViewState, {
    action: 'restore',
    source: 'api',
  })

  syncPreviewTextSelection()
}

async function handleToggleLibrary() {
  toggleLibrary()
  await nextTick()
  if (!isLibraryCollapsed.value) {
    await revealCurrentFileInTree()
  }
  window.dispatchEvent(new Event('resize'))
}

async function revealCurrentFileInTree() {
  await nextTick()
  const tree = libraryTree.value
  const fileId = currentFileId.value
  if (!tree || !fileId) {
    return
  }

  const activeItem = tree.querySelector<HTMLElement>(
    `[data-file-id="${CSS.escape(fileId)}"]`,
  )
  if (!activeItem) {
    return
  }

  const treeRect = tree.getBoundingClientRect()
  const itemRect = activeItem.getBoundingClientRect()
  const isVisible =
    itemRect.top >= treeRect.top && itemRect.bottom <= treeRect.bottom
  if (isVisible) {
    return
  }

  tree.scrollTo({
    behavior: 'smooth',
    top:
      tree.scrollTop +
      itemRect.top -
      treeRect.top -
      (tree.clientHeight - itemRect.height) / 2,
  })
}
</script>

<template>
  <section
    :class="[
      'desktop-file-preview-tool',
      { 'desktop-file-preview-tool--library-collapsed': isLibraryCollapsed },
    ]"
  >
    <aside
      v-if="!isLibraryCollapsed"
      class="desktop-file-preview-tool__library"
    >
      <header class="desktop-file-preview-tool__library-header">
        <button
          class="desktop-file-preview-tool__back"
          type="button"
          title="返回工具中心"
          @click="emit('backToTools')"
        >
          <DesktopUiIcon name="chevron-left" :size="15" />
        </button>
        <div class="desktop-file-preview-tool__library-title">
          <strong>文件目录</strong>
          <span>{{ sources.length }} 个目录</span>
        </div>
        <div class="desktop-file-preview-tool__library-actions">
          <button
            type="button"
            class="desktop-file-preview-tool__format-button"
            title="格式说明"
            @click="isFormatDialogOpen = true"
          >
            格式
          </button>
          <button
            type="button"
            title="刷新全部目录"
            :disabled="isScanning"
            @click="rescanAllSources"
          >
            <DesktopUiIcon name="reset-view" :size="15" />
          </button>
          <button
            class="desktop-file-preview-tool__add"
            type="button"
            title="添加预览目录"
            @click="openSourceDialog"
          >
            <DesktopUiIcon name="plus" :size="16" />
          </button>
        </div>
      </header>

      <div ref="libraryTree" class="desktop-file-preview-tool__tree desktop-scroll">
        <div
          v-if="sources.length === 0"
          class="desktop-file-preview-tool__library-empty"
        >
          <DesktopUiIcon name="file" :size="22" />
          <p>还没有预览目录</p>
          <button type="button" @click="openSourceDialog">添加目录</button>
        </div>

        <DesktopFilePreviewSourceGroup
          v-for="source in sources"
          :key="source.id"
          :active-file-id="currentFileId"
          :expanded-folder-ids="expandedFolderIds"
          :is-open="expandedSourceIds.includes(source.id)"
          :source="source"
          @edit="openEditSourceDialog"
          @remove="removeSource"
          @rescan="rescanSource"
          @select-file="handleFileSelect"
          @toggle-folder="toggleFolder"
          @toggle-source="toggleSource"
        />
      </div>

      <button
        class="desktop-file-preview-tool__library-collapse"
        type="button"
        title="收起文件目录"
        @click="handleToggleLibrary"
      >
        <DesktopUiIcon name="chevron-left" :size="16" />
      </button>
    </aside>

    <div
      v-if="isLibraryCollapsed"
      class="desktop-file-preview-tool__library-reveal-zone"
    >
      <button type="button" title="展开文件目录" @click="handleToggleLibrary">
        <DesktopUiIcon name="chevron-right" :size="16" />
      </button>
    </div>

    <main class="desktop-file-preview-tool__main">
      <section class="desktop-file-preview-tool__stage">
        <div
          v-if="currentFile && previewSource"
          class="desktop-file-preview-tool__viewer-frame"
        >
          <p
            v-if="currentFileSupportLabel"
            class="desktop-file-preview-tool__support-label"
          >
            {{ currentFileSupportLabel }}
          </p>
          <div
            ref="previewViewerHost"
            class="desktop-file-preview-tool__viewer-host"
          >
            <file-viewer
              ref="previewViewer"
              :key="`${currentFileId}:${previewSource.kind}`"
              class="desktop-file-preview-tool__viewer"
              :file="previewSource.kind === 'file' ? previewSource.file : undefined"
              :url="previewSource.kind === 'url' ? previewSource.url : undefined"
              :filename="currentFile.name"
              :options="previewOptions"
              @load-complete="handlePreviewLoadComplete"
              @view-state-change="handleViewStateChange"
            />
          </div>
        </div>

        <div v-else class="desktop-file-preview-tool__empty">
          <span><DesktopUiIcon name="file" :size="36" /></span>
          <strong>{{ emptyStateTitle }}</strong>
          <p>{{ emptyStateSummary }}</p>
          <button
            v-if="sources.length === 0"
            type="button"
            @click="openSourceDialog"
          >
            <DesktopUiIcon name="plus" :size="14" />
            添加预览目录
          </button>
          <button
            type="button"
            class="desktop-file-preview-tool__format-link"
            @click="isFormatDialogOpen = true"
          >
            查看支持格式
          </button>
        </div>
      </section>
    </main>

    <DesktopFilePreviewSourceDialog
      ref="sourceDialog"
      :feedback-message="feedbackMessage"
      :initial-source="editingSource"
      :is-open="isSourceDialogOpen"
      :is-saving="isScanning"
      :mode="sourceDialogMode"
      @cancel="isSourceDialogOpen = false"
      @choose-folder="handleChooseFolder"
      @save="handleSaveSource"
    />

    <DesktopFilePreviewFormatDialog
      :is-open="isFormatDialogOpen"
      @close="isFormatDialogOpen = false"
    />
  </section>
</template>

<style scoped>
.desktop-file-preview-tool {
  position: relative;
  display: grid;
  grid-template-columns: 284px minmax(0, 1fr);
  width: 100%;
  height: 100%;
  min-width: 0;
  min-height: 0;
  background: var(--desktop-bg);
}

.desktop-file-preview-tool--library-collapsed {
  grid-template-columns: minmax(0, 1fr);
}

.desktop-file-preview-tool__library-collapse,
.desktop-file-preview-tool__library-reveal-zone button,
.desktop-file-preview-tool__library-actions button,
.desktop-file-preview-tool__library-empty button,
.desktop-file-preview-tool__empty button,
.desktop-file-preview-tool__back {
  border: 0;
  font: inherit;
  cursor: pointer;
}

.desktop-file-preview-tool__library {
  position: relative;
  z-index: 30;
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  min-width: 0;
  min-height: 0;
  border-right: 1px solid var(--desktop-line-strong);
  background: var(--desktop-surface-strong);
}

.desktop-file-preview-tool__library-collapse,
.desktop-file-preview-tool__library-reveal-zone button {
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

.desktop-file-preview-tool__library-collapse::before,
.desktop-file-preview-tool__library-reveal-zone button::before {
  position: absolute;
  inset: 1px;
  background: var(--desktop-surface-strong);
  clip-path: inherit;
  content: '';
}

.desktop-file-preview-tool__library-collapse > *,
.desktop-file-preview-tool__library-reveal-zone button > * {
  position: relative;
  z-index: 1;
}

.desktop-file-preview-tool__library-collapse {
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

.desktop-file-preview-tool__library-collapse:hover,
.desktop-file-preview-tool__library-reveal-zone button:hover {
  color: var(--desktop-accent);
}

.desktop-file-preview-tool__library-collapse:hover {
  opacity: 1;
}

.desktop-file-preview-tool__library-collapse:hover::before,
.desktop-file-preview-tool__library-reveal-zone button:hover::before {
  background: color-mix(
    in srgb,
    var(--desktop-surface-strong) 90%,
    var(--desktop-accent)
  );
}

.desktop-file-preview-tool__library-reveal-zone {
  position: absolute;
  inset: 0 auto 0 0;
  z-index: 20;
  width: 20px;
}

.desktop-file-preview-tool__library-reveal-zone button {
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

.desktop-file-preview-tool__library-reveal-zone button::before {
  background: var(--desktop-accent);
}

.desktop-file-preview-tool__library-reveal-zone button {
  background: color-mix(
    in srgb,
    var(--desktop-accent) 78%,
    var(--desktop-line-strong)
  );
  color: #fff;
  filter: drop-shadow(4px 0 8px rgba(0, 0, 0, 0.28));
}

.desktop-file-preview-tool__library-reveal-zone button:hover::before {
  background: color-mix(in srgb, var(--desktop-accent) 88%, #fff);
}

.desktop-file-preview-tool__library-reveal-zone button:hover {
  color: #fff;
}

.desktop-file-preview-tool__library-reveal-zone:hover button,
.desktop-file-preview-tool__library-reveal-zone:focus-within button {
  opacity: 1;
  pointer-events: auto;
  transform: translate(0, -50%);
}

.desktop-file-preview-tool__library-header {
  display: grid;
  grid-template-columns: 1.85rem minmax(0, 1fr) auto;
  align-items: center;
  column-gap: 0.5rem;
  min-height: 3.35rem;
  padding: 0 0.68rem;
  border-bottom: 1px solid var(--desktop-line);
}

.desktop-file-preview-tool__library-title {
  display: flex;
  align-items: baseline;
  gap: 0.45rem;
  min-width: 0;
  overflow: hidden;
}

.desktop-file-preview-tool__library-title strong {
  flex: none;
  color: var(--desktop-ink);
  font-size: 0.84rem;
  font-weight: 650;
  line-height: 1.2;
}

.desktop-file-preview-tool__library-title span {
  min-width: 0;
  overflow: hidden;
  color: var(--desktop-soft);
  font-size: 0.65rem;
  line-height: 1.2;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.desktop-file-preview-tool__library-actions {
  display: inline-flex;
  align-items: center;
  flex: none;
  gap: 0.25rem;
}

.desktop-file-preview-tool__back,
.desktop-file-preview-tool__library-actions button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.85rem;
  height: 1.85rem;
  border-radius: 8px;
  background: transparent;
  color: var(--desktop-muted);
}

.desktop-file-preview-tool__back:hover,
.desktop-file-preview-tool__library-actions button:hover {
  background: rgba(var(--desktop-accent-rgb), 0.08);
  color: var(--desktop-accent);
}

.desktop-file-preview-tool__library-actions .desktop-file-preview-tool__format-button {
  width: auto;
  min-width: 1.85rem;
  padding: 0 0.45rem;
  font-size: 0.68rem;
  font-weight: 620;
}

.desktop-file-preview-tool__library-actions .desktop-file-preview-tool__add {
  background: var(--desktop-accent);
  color: #fff;
}

.desktop-file-preview-tool__tree {
  min-height: 0;
  overflow: auto;
  padding: 0.5rem 0.58rem 1rem;
}

.desktop-file-preview-tool__library-empty {
  display: grid;
  justify-items: center;
  gap: 0.45rem;
  padding: 2rem 1rem;
  color: var(--desktop-soft);
  text-align: center;
}

.desktop-file-preview-tool__library-empty p {
  margin: 0;
  font-size: 0.74rem;
}

.desktop-file-preview-tool__library-empty button,
.desktop-file-preview-tool__empty button {
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

.desktop-file-preview-tool__format-link {
  background: transparent !important;
  color: var(--desktop-soft) !important;
  font-weight: 600 !important;
}

.desktop-file-preview-tool__format-link:hover {
  color: var(--desktop-accent) !important;
}

.desktop-file-preview-tool__support-label {
  position: absolute;
  z-index: 2;
  top: 0.55rem;
  right: 0.65rem;
  margin: 0;
  padding: 0.18rem 0.48rem;
  border-radius: 999px;
  background: rgba(var(--desktop-shadow), 0.42);
  color: var(--desktop-soft);
  font-size: 0.62rem;
  line-height: 1.4;
  pointer-events: none;
}

.desktop-file-preview-tool__main {
  display: block;
  min-width: 0;
  min-height: 0;
  background: color-mix(
    in srgb,
    var(--desktop-bg) 96%,
    var(--desktop-accent)
  );
}

.desktop-file-preview-tool__stage {
  position: relative;
  display: grid;
  width: 100%;
  height: 100%;
  min-width: 0;
  min-height: 0;
}

.desktop-file-preview-tool__viewer-frame {
  position: relative;
  width: 100%;
  height: 100%;
  min-height: 0;
  overflow: hidden;
  user-select: text;
  -webkit-user-select: text;
}

.desktop-file-preview-tool__viewer-host {
  width: 100%;
  height: 100%;
  min-height: 0;
}

.desktop-file-preview-tool__viewer {
  display: block;
  width: 100%;
  height: 100%;
  min-height: 0;
}

.desktop-file-preview-tool__viewer :deep(.fv-root),
.desktop-file-preview-tool__viewer :deep([class*='file-viewer']) {
  width: 100%;
  height: 100%;
  min-height: 0;
}

.desktop-file-preview-tool__empty {
  display: grid;
  place-content: center;
  justify-items: center;
  gap: 0.55rem;
  color: var(--desktop-muted);
  text-align: center;
}

.desktop-file-preview-tool__empty > span {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 4rem;
  height: 4rem;
  border-radius: 18px;
  background: rgba(var(--desktop-accent-rgb), 0.09);
  color: var(--desktop-accent);
}

.desktop-file-preview-tool__empty strong {
  color: var(--desktop-ink);
  font-size: 0.94rem;
}

.desktop-file-preview-tool__empty p {
  max-width: 25rem;
  margin: 0 0 0.25rem;
  font-size: 0.76rem;
  line-height: 1.6;
}

@media (max-width: 920px) {
  .desktop-file-preview-tool {
    grid-template-columns: 230px minmax(0, 1fr);
  }
}
</style>
