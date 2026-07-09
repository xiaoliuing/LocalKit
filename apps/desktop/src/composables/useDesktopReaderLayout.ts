import { computed, shallowRef } from 'vue'

const STORAGE_KEY = 'docs-atlas.desktop.reader-layout.v1'
const RAIL_WIDTH_PX = 64

const DEFAULT_SIDEBAR_PANEL_WIDTH = 248
const DEFAULT_TOC_WIDTH = 248
const MIN_SIDEBAR_PANEL_WIDTH = 196
const MAX_SIDEBAR_PANEL_WIDTH = 440
const MIN_TOC_WIDTH = 176
const MAX_TOC_WIDTH = 520

type DesktopReaderLayoutState = {
  sidebarPanelWidth: number
  tocWidth: number
}

const defaultState: DesktopReaderLayoutState = {
  sidebarPanelWidth: DEFAULT_SIDEBAR_PANEL_WIDTH,
  tocWidth: DEFAULT_TOC_WIDTH,
}

const sidebarPanelWidth = shallowRef(defaultState.sidebarPanelWidth)
const tocWidth = shallowRef(defaultState.tocWidth)
let hasLoaded = false

export function useDesktopReaderLayout() {
  ensureLoaded()

  const sidebarWidth = computed(() => RAIL_WIDTH_PX + sidebarPanelWidth.value)

  function clamp(value: number, min: number, max: number) {
    return Math.min(max, Math.max(min, Math.round(value)))
  }

  function persistLayout() {
    if (typeof window === 'undefined') {
      return
    }

    const payload: DesktopReaderLayoutState = {
      sidebarPanelWidth: sidebarPanelWidth.value,
      tocWidth: tocWidth.value,
    }

    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(payload))
  }

  function startSidebarResize(event: MouseEvent) {
    startPanelResize(event, {
      getWidth: () => sidebarPanelWidth.value,
      setWidth: (value) => {
        sidebarPanelWidth.value = value
      },
      min: MIN_SIDEBAR_PANEL_WIDTH,
      max: MAX_SIDEBAR_PANEL_WIDTH,
      invertDelta: false,
    })
  }

  function startTocResize(event: MouseEvent) {
    startPanelResize(event, {
      getWidth: () => tocWidth.value,
      setWidth: (value) => {
        tocWidth.value = value
      },
      min: MIN_TOC_WIDTH,
      max: MAX_TOC_WIDTH,
      invertDelta: true,
    })
  }

  return {
    MAX_SIDEBAR_PANEL_WIDTH,
    MAX_TOC_WIDTH,
    MIN_SIDEBAR_PANEL_WIDTH,
    MIN_TOC_WIDTH,
    sidebarPanelWidth,
    sidebarWidth,
    startSidebarResize,
    startTocResize,
    tocWidth,
  }
}

function ensureLoaded() {
  if (hasLoaded || typeof window === 'undefined') {
    return
  }

  hasLoaded = true

  try {
    const raw = window.localStorage.getItem(STORAGE_KEY)
    if (!raw) {
      return
    }

    const parsed = JSON.parse(raw) as Partial<DesktopReaderLayoutState>
    if (typeof parsed.sidebarPanelWidth === 'number') {
      sidebarPanelWidth.value = clampNumber(
        parsed.sidebarPanelWidth,
        MIN_SIDEBAR_PANEL_WIDTH,
        MAX_SIDEBAR_PANEL_WIDTH,
        DEFAULT_SIDEBAR_PANEL_WIDTH,
      )
    }

    if (typeof parsed.tocWidth === 'number') {
      tocWidth.value = clampNumber(
        parsed.tocWidth,
        MIN_TOC_WIDTH,
        MAX_TOC_WIDTH,
        DEFAULT_TOC_WIDTH,
      )
    }
  } catch {
    sidebarPanelWidth.value = defaultState.sidebarPanelWidth
    tocWidth.value = defaultState.tocWidth
  }
}

function clampNumber(
  value: number,
  min: number,
  max: number,
  fallback: number,
) {
  if (!Number.isFinite(value)) {
    return fallback
  }

  return Math.min(max, Math.max(min, Math.round(value)))
}

function startPanelResize(
  event: MouseEvent,
  options: {
    getWidth: () => number
    setWidth: (value: number) => void
    min: number
    max: number
    invertDelta: boolean
  },
) {
  if (event.button !== 0) {
    return
  }

  event.preventDefault()

  const startX = event.clientX
  const startWidth = options.getWidth()

  const handleMove = (moveEvent: MouseEvent) => {
    const delta = moveEvent.clientX - startX
    const nextWidth = options.invertDelta
      ? startWidth - delta
      : startWidth + delta

    options.setWidth(
      Math.min(options.max, Math.max(options.min, Math.round(nextWidth))),
    )
  }

  const handleUp = () => {
    window.removeEventListener('mousemove', handleMove)
    window.removeEventListener('mouseup', handleUp)
    document.body.style.cursor = ''
    document.body.style.userSelect = ''
    persistLayout()
  }

  document.body.style.cursor = 'col-resize'
  document.body.style.userSelect = 'none'
  window.addEventListener('mousemove', handleMove)
  window.addEventListener('mouseup', handleUp)
}
