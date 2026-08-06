import { computed, readonly, shallowRef } from 'vue'
import type { FileViewerViewState } from '@file-viewer/vue3'
import { pickFolderPath } from '@/api/workspaces'
import {
  DEFAULT_PREVIEW_MAX_DEPTH,
  scanPreviewDirectory,
  toPreviewAssetUrl,
  type PreviewTreeNode,
} from '@/api/filePreview'

const FILE_PREVIEW_STORAGE_KEY = 'docs-atlas.desktop.file-preview.v1'

export type DesktopFilePreviewSource = {
  id: string
  title: string
  path: string
  maxDepth: number
  message: string
  fileCount: number
  tree: PreviewTreeNode[]
  updatedAt: string
}

export type DesktopPreviewFile = PreviewTreeNode & {
  sourceId: string
  sourceTitle: string
}

type PersistedFilePreviewSource = {
  id: string
  title: string
  path: string
  maxDepth: number
  updatedAt: string
}

type PersistedPreviewViewStateMemory = {
  viewState: FileViewerViewState
  updatedAt: string
}

type PersistedFilePreviewState = {
  sources: PersistedFilePreviewSource[]
  currentFileId: string
  expandedSourceIds: string[]
  expandedFolderIds: string[]
  isLibraryCollapsed: boolean
  viewStateMemory: Record<string, PersistedPreviewViewStateMemory>
}

export function useDesktopFilePreviewLibrary() {
  const persistedState = readPersistedState()
  const sources = shallowRef<DesktopFilePreviewSource[]>(
    persistedState.sources.map((source) => ({
      ...source,
      message: '等待扫描',
      fileCount: 0,
      tree: [],
    })),
  )
  const currentFileId = shallowRef(persistedState.currentFileId)
  const expandedSourceIds = shallowRef([...persistedState.expandedSourceIds])
  const expandedFolderIds = shallowRef([...persistedState.expandedFolderIds])
  const isLibraryCollapsed = shallowRef(persistedState.isLibraryCollapsed)
  const viewStateMemory = shallowRef<Record<string, PersistedPreviewViewStateMemory>>({
    ...persistedState.viewStateMemory,
  })
  const isScanning = shallowRef(false)
  const feedbackMessage = shallowRef('')

  const files = computed<DesktopPreviewFile[]>(() =>
    sources.value.flatMap((source) => flattenPreviewNodes(source.tree, source)),
  )
  const currentFile = computed(
    () => files.value.find((file) => file.id === currentFileId.value) ?? null,
  )
  const currentFileUrl = computed(() =>
    currentFile.value ? toPreviewAssetUrl(currentFile.value.path) : '',
  )
  const currentViewStateMemory = computed(
    () => viewStateMemory.value[currentFileId.value]?.viewState ?? null,
  )

  async function restoreSources() {
    if (sources.value.length === 0) {
      return
    }

    await rescanAllSources()
    restoreCurrentFile()
    revealCurrentFile()
    persistState()
  }

  async function chooseFolderPath() {
    return pickFolderPath()
  }

  async function addSource(input: { title: string; path: string; maxDepth: number }) {
    const path = input.path.trim()
    if (!path) {
      setFeedback('请先填写目录路径')
      return false
    }

    isScanning.value = true
    try {
      const title = input.title.trim() || getDirectoryName(path)
      const maxDepth = normalizeMaxDepth(input.maxDepth)
      const scanResult = await scanPreviewDirectory(path, maxDepth)
      if (!scanResult.exists || !scanResult.isDirectory) {
        setFeedback(scanResult.message)
        return false
      }

      const source: DesktopFilePreviewSource = {
        id: createPreviewSourceId(path),
        title,
        path,
        maxDepth,
        message: scanResult.message,
        fileCount: scanResult.fileCount,
        tree: scanResult.tree,
        updatedAt: new Date().toISOString(),
      }
      sources.value = [
        source,
        ...sources.value.filter((item) => item.id !== source.id),
      ]
      persistState()
      restoreCurrentFile()
      revealCurrentFile()
      persistState()
      setFeedback(scanResult.message)
      return true
    } finally {
      isScanning.value = false
    }
  }

  async function updateSource(
    sourceId: string,
    input: { title: string; path: string; maxDepth: number },
  ) {
    const existing = sources.value.find((item) => item.id === sourceId)
    if (!existing) {
      return false
    }

    const nextTitle = input.title.trim() || getDirectoryName(input.path)
    const nextPath = input.path.trim()
    const nextMaxDepth = normalizeMaxDepth(input.maxDepth)
    const pathChanged = nextPath !== existing.path
    const depthChanged = nextMaxDepth !== existing.maxDepth
    const titleChanged = nextTitle !== existing.title

    if (!nextPath) {
      setFeedback('请先填写目录路径')
      return false
    }

    if (!pathChanged && !depthChanged) {
      if (!titleChanged) {
        return true
      }

      sources.value = sources.value.map((item) =>
        item.id === sourceId
          ? {
              ...item,
              title: nextTitle,
              updatedAt: new Date().toISOString(),
            }
          : item,
      )
      persistState()
      return true
    }

    isScanning.value = true
    try {
      const scanResult = await scanPreviewDirectory(nextPath, nextMaxDepth)
      if (!scanResult.exists || !scanResult.isDirectory) {
        setFeedback(scanResult.message)
        return false
      }

      const nextId = createPreviewSourceId(nextPath)
      sources.value = sources.value.map((item) => {
        if (item.id !== sourceId) {
          return item
        }

        return {
          id: nextId,
          title: nextTitle,
          path: nextPath,
          maxDepth: nextMaxDepth,
          message: scanResult.message,
          fileCount: scanResult.fileCount,
          tree: scanResult.tree,
          updatedAt: new Date().toISOString(),
        }
      })

      if (nextId !== sourceId) {
        expandedSourceIds.value = expandedSourceIds.value.map((id) =>
          id === sourceId ? nextId : id,
        )
        expandedFolderIds.value = expandedFolderIds.value.map((id) =>
          id.startsWith(`${sourceId}::`) ? id.replace(sourceId, nextId) : id,
        )
        if (currentFileId.value.startsWith(`${sourceId}::`)) {
          currentFileId.value = currentFileId.value.replace(sourceId, nextId)
        }
      }

      persistState()
      restoreCurrentFile()
      revealCurrentFile()
      persistState()
      setFeedback(scanResult.message)
      return true
    } finally {
      isScanning.value = false
    }
  }

  async function rescanSource(sourceId: string, restoreSelection = true) {
    const source = sources.value.find((item) => item.id === sourceId)
    if (!source) {
      return
    }

    const scanResult = await scanPreviewDirectory(source.path, source.maxDepth)
    sources.value = sources.value.map((item) =>
      item.id === sourceId
        ? {
            ...item,
            message: scanResult.message,
            fileCount: scanResult.fileCount,
            tree: scanResult.tree,
            updatedAt: new Date().toISOString(),
          }
        : item,
    )
    persistState()
    if (restoreSelection) {
      restoreCurrentFile()
      revealCurrentFile()
      persistState()
    }
  }

  async function rescanAllSources() {
    isScanning.value = true
    try {
      for (const source of sources.value) {
        await rescanSource(source.id, false)
      }
      restoreCurrentFile()
      revealCurrentFile()
      persistState()
    } finally {
      isScanning.value = false
    }
  }

  function selectFile(fileId: string) {
    if (!files.value.some((file) => file.id === fileId)) {
      return
    }

    currentFileId.value = fileId
    revealCurrentFile()
    persistState()
  }

  function toggleSource(sourceId: string) {
    expandedSourceIds.value = toggleStoredId(expandedSourceIds.value, sourceId)
    persistState()
  }

  function toggleFolder(folderId: string) {
    expandedFolderIds.value = toggleStoredId(expandedFolderIds.value, folderId)
    persistState()
  }

  function toggleLibrary() {
    isLibraryCollapsed.value = !isLibraryCollapsed.value
    persistState()
  }

  function removeSource(sourceId: string) {
    sources.value = sources.value.filter((source) => source.id !== sourceId)
    expandedSourceIds.value = expandedSourceIds.value.filter((id) => id !== sourceId)
    expandedFolderIds.value = expandedFolderIds.value.filter(
      (id) => !id.startsWith(`${sourceId}::`),
    )
    viewStateMemory.value = removeViewStateMemoryForSource(
      viewStateMemory.value,
      sourceId,
    )
    if (currentFile.value?.sourceId === sourceId) {
      currentFileId.value = ''
    }
    persistState()
    restoreCurrentFile()
  }

  function rememberViewState(fileId: string, state: FileViewerViewState) {
    if (!fileId || !state) {
      return
    }

    viewStateMemory.value = {
      ...viewStateMemory.value,
      [fileId]: {
        viewState: cloneViewState(state),
        updatedAt: new Date().toISOString(),
      },
    }
    persistState()
  }

  function setFeedback(message: string) {
    feedbackMessage.value = message
  }

  function restoreCurrentFile() {
    if (currentFile.value) {
      return
    }

    currentFileId.value = files.value[0]?.id ?? ''
    persistState()
  }

  function revealCurrentFile() {
    const file = currentFile.value
    if (!file) {
      return
    }

    const source = sources.value.find((item) => item.id === file.sourceId)
    if (!source) {
      return
    }

    const ancestorFolderIds = findPreviewAncestorFolderIds(
      source.tree,
      file.id,
      source.id,
    )
    expandedSourceIds.value = appendUniqueIds(expandedSourceIds.value, [source.id])
    expandedFolderIds.value = appendUniqueIds(
      expandedFolderIds.value,
      ancestorFolderIds,
    )
  }

  function persistState() {
    if (typeof window === 'undefined') {
      return
    }

    const payload: PersistedFilePreviewState = {
      currentFileId: currentFileId.value,
      expandedFolderIds: expandedFolderIds.value,
      expandedSourceIds: expandedSourceIds.value,
      isLibraryCollapsed: isLibraryCollapsed.value,
      viewStateMemory: viewStateMemory.value,
      sources: sources.value.map((source) => ({
        id: source.id,
        title: source.title,
        path: source.path,
        maxDepth: source.maxDepth,
        updatedAt: source.updatedAt,
      })),
    }
    window.localStorage.setItem(FILE_PREVIEW_STORAGE_KEY, JSON.stringify(payload))
  }

  return {
    currentFile,
    currentFileId,
    currentFileUrl,
    currentViewStateMemory,
    expandedFolderIds: readonly(expandedFolderIds),
    expandedSourceIds: readonly(expandedSourceIds),
    feedbackMessage: readonly(feedbackMessage),
    files,
    isLibraryCollapsed: readonly(isLibraryCollapsed),
    isScanning: readonly(isScanning),
    sources: readonly(sources),
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
  }
}

function readPersistedState(): PersistedFilePreviewState {
  if (typeof window === 'undefined') {
    return createEmptyPersistedState()
  }

  const rawValue = window.localStorage.getItem(FILE_PREVIEW_STORAGE_KEY)
  if (!rawValue) {
    return createEmptyPersistedState()
  }

  try {
    const parsed = JSON.parse(rawValue) as Partial<PersistedFilePreviewState>
    return {
      currentFileId: typeof parsed.currentFileId === 'string' ? parsed.currentFileId : '',
      expandedFolderIds: normalizeStringArray(parsed.expandedFolderIds),
      expandedSourceIds: normalizeStringArray(parsed.expandedSourceIds),
      isLibraryCollapsed: parsed.isLibraryCollapsed === true,
      viewStateMemory: normalizeViewStateMemory(parsed.viewStateMemory),
      sources: Array.isArray(parsed.sources)
        ? parsed.sources.flatMap((source) => normalizePersistedSource(source))
        : [],
    }
  } catch {
    return createEmptyPersistedState()
  }
}

function createEmptyPersistedState(): PersistedFilePreviewState {
  return {
    currentFileId: '',
    expandedFolderIds: [],
    expandedSourceIds: [],
    isLibraryCollapsed: false,
    viewStateMemory: {},
    sources: [],
  }
}

function normalizeViewStateMemory(value: unknown) {
  if (!value || typeof value !== 'object') {
    return {}
  }

  const entries = Object.entries(value as Record<string, unknown>)
  const memory: Record<string, PersistedPreviewViewStateMemory> = {}

  for (const [fileId, entry] of entries) {
    if (!fileId || !entry || typeof entry !== 'object') {
      continue
    }

    const record = entry as Partial<PersistedPreviewViewStateMemory>
    if (!record.viewState || typeof record.viewState !== 'object') {
      continue
    }

    memory[fileId] = {
      viewState: cloneViewState(record.viewState as FileViewerViewState),
      updatedAt:
        typeof record.updatedAt === 'string'
          ? record.updatedAt
          : new Date().toISOString(),
    }
  }

  return memory
}

function cloneViewState(state: FileViewerViewState): FileViewerViewState {
  return JSON.parse(JSON.stringify(state)) as FileViewerViewState
}

function removeViewStateMemoryForSource(
  memory: Record<string, PersistedPreviewViewStateMemory>,
  sourceId: string,
) {
  const prefix = `${sourceId}::`
  const next: Record<string, PersistedPreviewViewStateMemory> = {}

  for (const [fileId, entry] of Object.entries(memory)) {
    if (!fileId.startsWith(prefix)) {
      next[fileId] = entry
    }
  }

  return next
}

function normalizePersistedSource(source: unknown): PersistedFilePreviewSource[] {
  if (!source || typeof source !== 'object') {
    return []
  }

  const value = source as Partial<PersistedFilePreviewSource>
  if (typeof value.path !== 'string' || !value.path.trim()) {
    return []
  }

  return [
    {
      id:
        typeof value.id === 'string' && value.id
          ? value.id
          : createPreviewSourceId(value.path),
      title:
        typeof value.title === 'string' && value.title.trim()
          ? value.title.trim()
          : getDirectoryName(value.path),
      path: value.path,
      maxDepth: normalizeMaxDepth(value.maxDepth),
      updatedAt:
        typeof value.updatedAt === 'string' ? value.updatedAt : new Date().toISOString(),
    },
  ]
}

function normalizeMaxDepth(value: unknown) {
  const depth = typeof value === 'number' ? value : Number(value)
  if (!Number.isFinite(depth)) {
    return DEFAULT_PREVIEW_MAX_DEPTH
  }

  if (depth === 0) {
    return 0
  }

  return Math.min(10, Math.max(1, Math.round(depth)))
}

function flattenPreviewNodes(
  nodes: PreviewTreeNode[],
  source: Pick<DesktopFilePreviewSource, 'id' | 'title'>,
): DesktopPreviewFile[] {
  return nodes.flatMap((node) => {
    if (node.kind === 'file') {
      return [
        {
          ...node,
          id: `${source.id}::${node.id}`,
          sourceId: source.id,
          sourceTitle: source.title,
        },
      ]
    }

    return flattenPreviewNodes(node.children, source)
  })
}

function createPreviewSourceId(path: string) {
  return `preview-source:${path.trim().toLowerCase()}`
}

function getDirectoryName(path: string) {
  const normalized = path.trim().replace(/\\/g, '/')
  return normalized.split('/').filter(Boolean).at(-1) || '本地目录'
}

function normalizeStringArray(value: unknown) {
  return Array.isArray(value)
    ? [...new Set(value.filter((item): item is string => typeof item === 'string'))]
    : []
}

function toggleStoredId(ids: string[], targetId: string) {
  return ids.includes(targetId)
    ? ids.filter((id) => id !== targetId)
    : [...ids, targetId]
}

function appendUniqueIds(ids: string[], additions: string[]) {
  return [...new Set([...ids, ...additions])]
}

function findPreviewAncestorFolderIds(
  nodes: PreviewTreeNode[],
  fileId: string,
  sourceId: string,
  ancestors: string[] = [],
): string[] {
  for (const node of nodes) {
    if (node.kind === 'file') {
      if (`${sourceId}::${node.id}` === fileId) {
        return ancestors
      }
      continue
    }

    const folderId = `${sourceId}::${node.id}`
    const result = findPreviewAncestorFolderIds(
      node.children,
      fileId,
      sourceId,
      [...ancestors, folderId],
    )
    if (result.length > 0) {
      return result
    }
  }

  return []
}
