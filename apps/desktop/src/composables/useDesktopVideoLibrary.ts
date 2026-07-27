import { computed, readonly, shallowRef } from 'vue'
import { pickFolderPath } from '@/api/workspaces'
import { scanVideoDirectory, toVideoAssetUrl, type VideoTreeNode } from '@/api/videos'

const VIDEO_LIBRARY_STORAGE_KEY = 'docs-atlas.desktop.video-library.v1'

export type DesktopVideoSource = {
  id: string
  title: string
  path: string
  message: string
  videoCount: number
  tree: VideoTreeNode[]
  updatedAt: string
}

export type DesktopVideoFile = VideoTreeNode & {
  sourceId: string
  sourceTitle: string
}

type PersistedVideoSource = {
  id: string
  title: string
  path: string
  updatedAt: string
}

type PersistedPlaybackMemory = {
  videoId: string
  sourceId: string
  position: number
  duration: number
  updatedAt: string
}

type PersistedVideoLibraryState = {
  sources: PersistedVideoSource[]
  currentVideoId: string
  playbackMemory: Record<string, PersistedPlaybackMemory>
  expandedSourceIds: string[]
  expandedFolderIds: string[]
}

export function useDesktopVideoLibrary() {
  const persistedState = readPersistedState()
  const sources = shallowRef<DesktopVideoSource[]>(
    persistedState.sources.map((source) => ({
      ...source,
      message: '等待扫描',
      videoCount: 0,
      tree: [],
    })),
  )
  const currentVideoId = shallowRef(persistedState.currentVideoId)
  const playbackMemory = shallowRef<Record<string, PersistedPlaybackMemory>>({
    ...persistedState.playbackMemory,
  })
  const expandedSourceIds = shallowRef([...persistedState.expandedSourceIds])
  const expandedFolderIds = shallowRef([...persistedState.expandedFolderIds])
  const isScanning = shallowRef(false)
  const feedbackMessage = shallowRef('')

  const videos = computed<DesktopVideoFile[]>(() =>
    sources.value.flatMap((source) => flattenVideoNodes(source.tree, source)),
  )
  const currentVideo = computed(() =>
    videos.value.find((video) => video.id === currentVideoId.value) ?? null,
  )
  const currentVideoUrl = computed(() =>
    currentVideo.value ? toVideoAssetUrl(currentVideo.value.path) : '',
  )
  const currentPlaybackMemory = computed(() =>
    currentVideo.value ? playbackMemory.value[currentVideo.value.id] ?? null : null,
  )
  const lastPlaybackMemory = computed(() =>
    Object.values(playbackMemory.value).sort((left, right) =>
      right.updatedAt.localeCompare(left.updatedAt),
    )[0] ?? null,
  )
  const lastPlaybackVideo = computed(() => {
    const memory = lastPlaybackMemory.value
    if (!memory) {
      return null
    }

    return videos.value.find((video) => video.id === memory.videoId) ?? null
  })

  async function restoreSources() {
    if (sources.value.length === 0) {
      return
    }

    await rescanAllSources()
    restoreCurrentVideo()
    revealCurrentVideo()
    persistState()
  }

  async function chooseFolderPath() {
    return pickFolderPath()
  }

  async function addSource(input: { title: string; path: string }) {
    const path = input.path.trim()
    if (!path) {
      setFeedback('请先填写视频目录路径')
      return false
    }

    const title = input.title.trim() || getDirectoryName(path)
    const scanResult = await scanVideoDirectory(path)
    if (!scanResult.exists || !scanResult.isDirectory) {
      setFeedback(scanResult.message)
      return false
    }

    const source: DesktopVideoSource = {
      id: createVideoSourceId(path),
      title,
      path,
      message: scanResult.message,
      videoCount: scanResult.videoCount,
      tree: scanResult.tree,
      updatedAt: new Date().toISOString(),
    }
    const nextSources = [
      source,
      ...sources.value.filter((item) => item.id !== source.id),
    ]

    sources.value = nextSources
    persistState()
    restoreCurrentVideo()
    revealCurrentVideo()
    persistState()
    setFeedback(scanResult.message)
    return true
  }

  async function rescanSource(sourceId: string, restoreSelection = true) {
    const source = sources.value.find((item) => item.id === sourceId)
    if (!source) {
      return
    }

    const scanResult = await scanVideoDirectory(source.path)
    sources.value = sources.value.map((item) =>
      item.id === sourceId
        ? {
            ...item,
            message: scanResult.message,
            videoCount: scanResult.videoCount,
            tree: scanResult.tree,
            updatedAt: new Date().toISOString(),
          }
        : item,
    )
    persistState()
    if (restoreSelection) {
      restoreCurrentVideo()
      revealCurrentVideo()
      persistState()
    }
  }

  async function rescanAllSources() {
    isScanning.value = true
    try {
      for (const source of sources.value) {
        await rescanSource(source.id, false)
      }
      restoreCurrentVideo()
      revealCurrentVideo()
      persistState()
    } finally {
      isScanning.value = false
    }
  }

  function selectVideo(videoId: string) {
    if (!videos.value.some((video) => video.id === videoId)) {
      return
    }

    currentVideoId.value = videoId
    revealCurrentVideo()
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

  function rememberPlayback(position: number, duration: number) {
    const video = currentVideo.value
    if (!video) {
      return
    }

    playbackMemory.value = {
      ...playbackMemory.value,
      [video.id]: {
        videoId: video.id,
        sourceId: video.sourceId,
        position: Math.max(0, Math.round(position)),
        duration: Math.max(0, Math.round(duration || 0)),
        updatedAt: new Date().toISOString(),
      },
    }
    persistState()
  }

  function resumeLastVideo() {
    const video = lastPlaybackVideo.value
    if (!video) {
      restoreCurrentVideo()
      return
    }

    selectVideo(video.id)
  }

  function removeSource(sourceId: string) {
    sources.value = sources.value.filter((source) => source.id !== sourceId)
    expandedSourceIds.value = expandedSourceIds.value.filter((id) => id !== sourceId)
    expandedFolderIds.value = expandedFolderIds.value.filter(
      (id) => !id.startsWith(`${sourceId}::`),
    )
    if (currentVideo.value?.sourceId === sourceId) {
      currentVideoId.value = ''
    }
    persistState()
    restoreCurrentVideo()
  }

  function setFeedback(message: string) {
    feedbackMessage.value = message
  }

  function restoreCurrentVideo() {
    if (currentVideo.value) {
      return
    }

    const lastVideo = lastPlaybackVideo.value
    currentVideoId.value = lastVideo?.id ?? videos.value[0]?.id ?? ''
    persistState()
  }

  function revealCurrentVideo() {
    const video = currentVideo.value
    if (!video) {
      return
    }

    const source = sources.value.find((item) => item.id === video.sourceId)
    if (!source) {
      return
    }

    const ancestorFolderIds = findVideoAncestorFolderIds(
      source.tree,
      video.id,
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

    const payload: PersistedVideoLibraryState = {
      currentVideoId: currentVideoId.value,
      expandedFolderIds: expandedFolderIds.value,
      expandedSourceIds: expandedSourceIds.value,
      playbackMemory: playbackMemory.value,
      sources: sources.value.map((source) => ({
        id: source.id,
        title: source.title,
        path: source.path,
        updatedAt: source.updatedAt,
      })),
    }
    window.localStorage.setItem(VIDEO_LIBRARY_STORAGE_KEY, JSON.stringify(payload))
  }

  return {
    currentPlaybackMemory,
    currentVideo,
    currentVideoId,
    currentVideoUrl,
    expandedFolderIds: readonly(expandedFolderIds),
    expandedSourceIds: readonly(expandedSourceIds),
    feedbackMessage: readonly(feedbackMessage),
    isScanning: readonly(isScanning),
    lastPlaybackMemory,
    lastPlaybackVideo,
    sources: readonly(sources),
    videos,
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
    toggleFolder,
    toggleSource,
  }
}

function readPersistedState(): PersistedVideoLibraryState {
  if (typeof window === 'undefined') {
    return createEmptyPersistedState()
  }

  const rawValue = window.localStorage.getItem(VIDEO_LIBRARY_STORAGE_KEY)
  if (!rawValue) {
    return createEmptyPersistedState()
  }

  try {
    const parsed = JSON.parse(rawValue) as Partial<PersistedVideoLibraryState>
    return {
      currentVideoId: typeof parsed.currentVideoId === 'string' ? parsed.currentVideoId : '',
      expandedFolderIds: normalizeStringArray(parsed.expandedFolderIds),
      expandedSourceIds: normalizeStringArray(parsed.expandedSourceIds),
      playbackMemory: isRecord(parsed.playbackMemory) ? parsed.playbackMemory : {},
      sources: Array.isArray(parsed.sources)
        ? parsed.sources.flatMap((source) => normalizePersistedSource(source))
        : [],
    }
  } catch {
    return createEmptyPersistedState()
  }
}

function createEmptyPersistedState(): PersistedVideoLibraryState {
  return {
    currentVideoId: '',
    expandedFolderIds: [],
    expandedSourceIds: [],
    playbackMemory: {},
    sources: [],
  }
}

function normalizePersistedSource(source: unknown): PersistedVideoSource[] {
  if (!source || typeof source !== 'object') {
    return []
  }

  const value = source as Partial<PersistedVideoSource>
  if (typeof value.path !== 'string' || !value.path.trim()) {
    return []
  }

  return [
    {
      id: typeof value.id === 'string' && value.id ? value.id : createVideoSourceId(value.path),
      title: typeof value.title === 'string' && value.title.trim()
        ? value.title.trim()
        : getDirectoryName(value.path),
      path: value.path,
      updatedAt: typeof value.updatedAt === 'string' ? value.updatedAt : new Date().toISOString(),
    },
  ]
}

function isRecord(value: unknown): value is Record<string, PersistedPlaybackMemory> {
  return Boolean(value && typeof value === 'object' && !Array.isArray(value))
}

function flattenVideoNodes(
  nodes: VideoTreeNode[],
  source: Pick<DesktopVideoSource, 'id' | 'title'>,
): DesktopVideoFile[] {
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

    return flattenVideoNodes(node.children, source)
  })
}

function createVideoSourceId(path: string) {
  return `video-source:${path.trim().toLowerCase()}`
}

function getDirectoryName(path: string) {
  const normalized = path.trim().replace(/\\/g, '/')
  return normalized.split('/').filter(Boolean).at(-1) || '视频目录'
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

function findVideoAncestorFolderIds(
  nodes: VideoTreeNode[],
  videoId: string,
  sourceId: string,
  ancestors: string[] = [],
): string[] {
  for (const node of nodes) {
    if (node.kind === 'file') {
      if (`${sourceId}::${node.id}` === videoId) {
        return ancestors
      }
      continue
    }

    const folderId = `${sourceId}::${node.id}`
    const result = findVideoAncestorFolderIds(
      node.children,
      videoId,
      sourceId,
      [...ancestors, folderId],
    )
    if (result.length > 0) {
      return result
    }
  }

  return []
}
