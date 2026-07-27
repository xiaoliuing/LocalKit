import { computed, shallowRef } from 'vue'
import type { DesktopSearchScope } from '@/composables/useDesktopDocsSearch'

const STORAGE_KEY = 'docs-atlas.desktop.reading-state.v1'
const FAVORITE_ENTRY_LIMIT = 120
const RECENT_ENTRY_LIMIT = 36

export type DesktopRecentDocEntry = {
  workspaceId: string
  slug: string
  openedAt: string
}

export type DesktopFavoriteDocEntry = {
  workspaceId: string
  slug: string
  savedAt: string
}

type DesktopSidebarState = {
  openBranchIds: string[]
  openSectionId: string | null
}

type DesktopReadingState = {
  currentWorkspaceId: string
  favoriteEntries: DesktopFavoriteDocEntry[]
  recentEntries: DesktopRecentDocEntry[]
  selectedDocByWorkspaceId: Record<string, string>
  searchScopeByWorkspaceId: Record<string, DesktopSearchScope>
  sidebarStateByWorkspaceId: Record<string, DesktopSidebarState>
  scrollTopByDocKey: Record<string, number>
}

const defaultState: DesktopReadingState = {
  currentWorkspaceId: '',
  favoriteEntries: [],
  recentEntries: [],
  scrollTopByDocKey: {},
  searchScopeByWorkspaceId: {},
  selectedDocByWorkspaceId: {},
  sidebarStateByWorkspaceId: {},
}

const readingState = shallowRef<DesktopReadingState>(cloneDefaultState())
let hasLoaded = false
let persistTimer: number | null = null

export function useDesktopReadingState() {
  ensureLoaded()

  return {
    currentWorkspaceId: computed(() => readingState.value.currentWorkspaceId),
    favoriteEntries: computed(() => readingState.value.favoriteEntries),
    flushPersist,
    getDocScrollTop,
    isDocFavorite,
    recentEntries: computed(() => readingState.value.recentEntries),
    getSearchScopeForWorkspace,
    getSelectedDocForWorkspace,
    getSidebarStateForWorkspace,
    removeFavoriteDoc,
    recordRecentDoc,
    setCurrentWorkspaceId,
    setDocScrollTop,
    setSearchScopeForWorkspace,
    setSelectedDocForWorkspace,
    setSidebarStateForWorkspace,
    toggleFavoriteDoc,
  }
}

function ensureLoaded() {
  if (hasLoaded) {
    return
  }

  hasLoaded = true

  if (typeof window === 'undefined') {
    return
  }

  try {
    const rawValue = window.localStorage.getItem(STORAGE_KEY)
    if (!rawValue) {
      return
    }

    readingState.value = normalizeState(JSON.parse(rawValue) as Partial<DesktopReadingState>)
  } catch {
    readingState.value = cloneDefaultState()
  }
}

function normalizeState(value: Partial<DesktopReadingState>): DesktopReadingState {
  return {
    currentWorkspaceId: typeof value.currentWorkspaceId === 'string' ? value.currentWorkspaceId : '',
    favoriteEntries: normalizeFavoriteEntries(value.favoriteEntries),
    recentEntries: normalizeRecentEntries(value.recentEntries),
    scrollTopByDocKey: normalizeStringNumberMap(value.scrollTopByDocKey),
    searchScopeByWorkspaceId: normalizeScopeMap(value.searchScopeByWorkspaceId),
    selectedDocByWorkspaceId: normalizeStringMap(value.selectedDocByWorkspaceId),
    sidebarStateByWorkspaceId: normalizeSidebarStateMap(value.sidebarStateByWorkspaceId),
  }
}

function setCurrentWorkspaceId(workspaceId: string) {
  if (!workspaceId || readingState.value.currentWorkspaceId === workspaceId) {
    return
  }

  readingState.value = {
    ...readingState.value,
    currentWorkspaceId: workspaceId,
  }
  schedulePersist()
}

function getSelectedDocForWorkspace(workspaceId: string) {
  return readingState.value.selectedDocByWorkspaceId[workspaceId] ?? ''
}

function setSelectedDocForWorkspace(workspaceId: string, slug: string) {
  if (!workspaceId || !slug || readingState.value.selectedDocByWorkspaceId[workspaceId] === slug) {
    return
  }

  readingState.value = {
    ...readingState.value,
    selectedDocByWorkspaceId: {
      ...readingState.value.selectedDocByWorkspaceId,
      [workspaceId]: slug,
    },
  }
  schedulePersist()
}

function recordRecentDoc(workspaceId: string, slug: string, openedAt = new Date().toISOString()) {
  if (!workspaceId || !slug) {
    return
  }

  const normalizedOpenedAt = normalizeRecentTimestamp(openedAt) ?? new Date().toISOString()
  const nextEntries = [
    { workspaceId, slug, openedAt: normalizedOpenedAt },
    ...readingState.value.recentEntries.filter((entry) => !(entry.workspaceId === workspaceId && entry.slug === slug)),
  ].slice(0, RECENT_ENTRY_LIMIT)

  if (recentEntriesEqual(readingState.value.recentEntries, nextEntries)) {
    return
  }

  readingState.value = {
    ...readingState.value,
    recentEntries: nextEntries,
  }
  schedulePersist()
}

function isDocFavorite(workspaceId: string, slug: string) {
  return readingState.value.favoriteEntries.some((entry) => entry.workspaceId === workspaceId && entry.slug === slug)
}

function toggleFavoriteDoc(workspaceId: string, slug: string, savedAt = new Date().toISOString()) {
  if (!workspaceId || !slug) {
    return
  }

  if (isDocFavorite(workspaceId, slug)) {
    removeFavoriteDoc(workspaceId, slug)
    return
  }

  const normalizedSavedAt = normalizeRecentTimestamp(savedAt) ?? new Date().toISOString()
  const nextEntries = [
    { workspaceId, slug, savedAt: normalizedSavedAt },
    ...readingState.value.favoriteEntries.filter((entry) => !(entry.workspaceId === workspaceId && entry.slug === slug)),
  ].slice(0, FAVORITE_ENTRY_LIMIT)

  readingState.value = {
    ...readingState.value,
    favoriteEntries: nextEntries,
  }
  schedulePersist()
}

function removeFavoriteDoc(workspaceId: string, slug: string) {
  if (!workspaceId || !slug) {
    return
  }

  const nextEntries = readingState.value.favoriteEntries.filter(
    (entry) => !(entry.workspaceId === workspaceId && entry.slug === slug),
  )

  if (nextEntries.length === readingState.value.favoriteEntries.length) {
    return
  }

  readingState.value = {
    ...readingState.value,
    favoriteEntries: nextEntries,
  }
  schedulePersist()
}

function getSearchScopeForWorkspace(workspaceId: string, fallback: DesktopSearchScope): DesktopSearchScope {
  const scope = readingState.value.searchScopeByWorkspaceId[workspaceId]
  return scope === 'workspace' || scope === 'global' ? scope : fallback
}

function setSearchScopeForWorkspace(workspaceId: string, scope: DesktopSearchScope) {
  if (!workspaceId || readingState.value.searchScopeByWorkspaceId[workspaceId] === scope) {
    return
  }

  readingState.value = {
    ...readingState.value,
    searchScopeByWorkspaceId: {
      ...readingState.value.searchScopeByWorkspaceId,
      [workspaceId]: scope,
    },
  }
  schedulePersist()
}

function getSidebarStateForWorkspace(workspaceId: string): DesktopSidebarState | null {
  const value = readingState.value.sidebarStateByWorkspaceId[workspaceId]
  if (!value) {
    return null
  }

  return {
    openBranchIds: [...value.openBranchIds],
    openSectionId: value.openSectionId,
  }
}

function setSidebarStateForWorkspace(workspaceId: string, value: DesktopSidebarState) {
  if (!workspaceId) {
    return
  }

  const nextValue: DesktopSidebarState = {
    openBranchIds: [...value.openBranchIds],
    openSectionId: value.openSectionId,
  }
  const currentValue = readingState.value.sidebarStateByWorkspaceId[workspaceId]

  if (
    currentValue &&
    currentValue.openSectionId === nextValue.openSectionId &&
    currentValue.openBranchIds.join('::') === nextValue.openBranchIds.join('::')
  ) {
    return
  }

  readingState.value = {
    ...readingState.value,
    sidebarStateByWorkspaceId: {
      ...readingState.value.sidebarStateByWorkspaceId,
      [workspaceId]: nextValue,
    },
  }
  schedulePersist()
}

function getDocScrollTop(workspaceId: string, slug: string) {
  const key = createDocKey(workspaceId, slug)
  return Math.max(0, readingState.value.scrollTopByDocKey[key] ?? 0)
}

function setDocScrollTop(workspaceId: string, slug: string, top: number) {
  const key = createDocKey(workspaceId, slug)
  const normalizedTop = Number.isFinite(top) ? Math.max(0, Math.round(top)) : 0

  if (!key || readingState.value.scrollTopByDocKey[key] === normalizedTop) {
    return
  }

  readingState.value = {
    ...readingState.value,
    scrollTopByDocKey: {
      ...readingState.value.scrollTopByDocKey,
      [key]: normalizedTop,
    },
  }
  schedulePersist()
}

function createDocKey(workspaceId: string, slug: string) {
  if (!workspaceId || !slug) {
    return ''
  }

  return `${workspaceId}::${slug}`
}

function schedulePersist() {
  if (typeof window === 'undefined') {
    return
  }

  if (persistTimer) {
    window.clearTimeout(persistTimer)
  }

  persistTimer = window.setTimeout(() => {
    flushPersist()
  }, 80)
}

function flushPersist() {
  if (typeof window === 'undefined') {
    return
  }

  if (persistTimer) {
    window.clearTimeout(persistTimer)
    persistTimer = null
  }

  window.localStorage.setItem(STORAGE_KEY, JSON.stringify(readingState.value))
}

function normalizeStringMap(value: unknown) {
  if (!value || typeof value !== 'object') {
    return {}
  }

  return Object.fromEntries(
    Object.entries(value).filter((entry): entry is [string, string] => typeof entry[1] === 'string' && entry[1].trim()),
  )
}

function normalizeScopeMap(value: unknown) {
  if (!value || typeof value !== 'object') {
    return {}
  }

  return Object.fromEntries(
    Object.entries(value).filter(
      (entry): entry is [string, DesktopSearchScope] => entry[1] === 'global' || entry[1] === 'workspace',
    ),
  )
}

function normalizeSidebarStateMap(value: unknown) {
  if (!value || typeof value !== 'object') {
    return {}
  }

  return Object.fromEntries(
    Object.entries(value).flatMap(([workspaceId, sidebarState]) => {
      if (!sidebarState || typeof sidebarState !== 'object') {
        return []
      }

      const openBranchIds = Array.isArray((sidebarState as DesktopSidebarState).openBranchIds)
        ? (sidebarState as DesktopSidebarState).openBranchIds.filter((item): item is string => typeof item === 'string' && item.trim())
        : []
      const openSectionId =
        typeof (sidebarState as DesktopSidebarState).openSectionId === 'string'
          ? (sidebarState as DesktopSidebarState).openSectionId
          : null

      return [[workspaceId, { openBranchIds, openSectionId } satisfies DesktopSidebarState]]
    }),
  )
}

function normalizeStringNumberMap(value: unknown) {
  if (!value || typeof value !== 'object') {
    return {}
  }

  return Object.fromEntries(
    Object.entries(value).flatMap(([key, top]) => {
      if (typeof top !== 'number' || !Number.isFinite(top)) {
        return []
      }

      return [[key, Math.max(0, Math.round(top))]]
    }),
  )
}

function normalizeRecentEntries(value: unknown) {
  if (!Array.isArray(value)) {
    return []
  }

  return value
    .flatMap((entry) => {
      if (!entry || typeof entry !== 'object') {
        return []
      }

      const workspaceId = typeof (entry as DesktopRecentDocEntry).workspaceId === 'string'
        ? (entry as DesktopRecentDocEntry).workspaceId.trim()
        : ''
      const slug = typeof (entry as DesktopRecentDocEntry).slug === 'string'
        ? (entry as DesktopRecentDocEntry).slug.trim()
        : ''
      const openedAt = normalizeRecentTimestamp((entry as DesktopRecentDocEntry).openedAt)

      if (!workspaceId || !slug || !openedAt) {
        return []
      }

      return [{ workspaceId, slug, openedAt } satisfies DesktopRecentDocEntry]
    })
    .filter((entry, index, array) => array.findIndex((candidate) => candidate.workspaceId === entry.workspaceId && candidate.slug === entry.slug) === index)
    .slice(0, RECENT_ENTRY_LIMIT)
}

function normalizeFavoriteEntries(value: unknown) {
  if (!Array.isArray(value)) {
    return []
  }

  return value
    .flatMap((entry) => {
      if (!entry || typeof entry !== 'object') {
        return []
      }

      const workspaceId = typeof (entry as DesktopFavoriteDocEntry).workspaceId === 'string'
        ? (entry as DesktopFavoriteDocEntry).workspaceId.trim()
        : ''
      const slug = typeof (entry as DesktopFavoriteDocEntry).slug === 'string'
        ? (entry as DesktopFavoriteDocEntry).slug.trim()
        : ''
      const savedAt = normalizeRecentTimestamp((entry as DesktopFavoriteDocEntry).savedAt)

      if (!workspaceId || !slug || !savedAt) {
        return []
      }

      return [{ workspaceId, slug, savedAt } satisfies DesktopFavoriteDocEntry]
    })
    .filter((entry, index, array) => array.findIndex((candidate) => candidate.workspaceId === entry.workspaceId && candidate.slug === entry.slug) === index)
    .slice(0, FAVORITE_ENTRY_LIMIT)
}

function normalizeRecentTimestamp(value: unknown) {
  if (typeof value !== 'string' || !value.trim()) {
    return null
  }

  const date = new Date(value)
  if (Number.isNaN(date.getTime())) {
    return null
  }

  return date.toISOString()
}

function recentEntriesEqual(left: DesktopRecentDocEntry[], right: DesktopRecentDocEntry[]) {
  if (left.length !== right.length) {
    return false
  }

  return left.every((entry, index) => {
    const candidate = right[index]
    return Boolean(
      candidate &&
      candidate.workspaceId === entry.workspaceId &&
      candidate.slug === entry.slug &&
      candidate.openedAt === entry.openedAt,
    )
  })
}

function cloneDefaultState(): DesktopReadingState {
  return {
    currentWorkspaceId: defaultState.currentWorkspaceId,
    favoriteEntries: [...defaultState.favoriteEntries],
    recentEntries: [...defaultState.recentEntries],
    scrollTopByDocKey: { ...defaultState.scrollTopByDocKey },
    searchScopeByWorkspaceId: { ...defaultState.searchScopeByWorkspaceId },
    selectedDocByWorkspaceId: { ...defaultState.selectedDocByWorkspaceId },
    sidebarStateByWorkspaceId: { ...defaultState.sidebarStateByWorkspaceId },
  }
}
