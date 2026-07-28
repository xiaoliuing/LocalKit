import { computed, shallowRef } from 'vue'
import { syncWindowBackgroundColor } from '@/api/system'

const STORAGE_KEY = 'docs-atlas.desktop.preferences.v1'
const DEFAULT_CUSTOM_ACCENT_COLOR = '#5B6FD6'

export type DesktopThemeMode = 'system' | 'light' | 'dark'
export type DesktopMarkdownThemeId = 'atlas' | 'github' | 'compact' | 'reading'
export type DesktopAccentId =
  | 'custom'
  | 'slate-indigo'
  | 'terracotta'
  | 'plum-orchid'
  | 'walnut-brown'
  | 'atlas-blue'
  | 'ocean-teal'
  | 'forest-green'
  | 'sunset-amber'
  | 'dusty-rose'

export type DesktopAccentOption = {
  id: DesktopAccentId
  label: string
  hex: string
  rgb: string
}

export type DesktopAccentSelection = {
  id: DesktopAccentId
  label: string
  hex: string
  rgb: string
}

export type DesktopMarkdownThemeOption = {
  id: DesktopMarkdownThemeId
  label: string
  description: string
}

type DesktopPreferences = {
  themeMode: DesktopThemeMode
  accentId: DesktopAccentId
  customAccentColor: string
  markdownThemeId: DesktopMarkdownThemeId
}

const accentOptions: DesktopAccentOption[] = [
  { id: 'slate-indigo', label: '靛云蓝', hex: '#5B6FD6', rgb: '91, 111, 214' },
  { id: 'terracotta', label: '赤陶橘', hex: '#C97059', rgb: '201, 112, 89' },
  { id: 'plum-orchid', label: '晚梅紫', hex: '#9A68B2', rgb: '154, 104, 178' },
  { id: 'walnut-brown', label: '榛木棕', hex: '#9B7653', rgb: '155, 118, 83' },
  { id: 'atlas-blue', label: '星图蓝', hex: '#1F54D9', rgb: '31, 84, 217' },
  { id: 'ocean-teal', label: '海雾青', hex: '#0F8C95', rgb: '15, 140, 149' },
  { id: 'forest-green', label: '森林绿', hex: '#1F8F63', rgb: '31, 143, 99' },
  { id: 'sunset-amber', label: '落日金', hex: '#C28A1A', rgb: '194, 138, 26' },
  { id: 'dusty-rose', label: '雾玫瑰', hex: '#C05F7F', rgb: '192, 95, 127' },
]

const markdownThemeOptions: DesktopMarkdownThemeOption[] = [
  { id: 'atlas', label: 'Atlas 默认', description: '平衡型文档排版，适合教程和设计文档。' },
  { id: 'github', label: 'GitHub', description: '接近 GitHub Markdown 的边界、表格和代码风格。' },
  { id: 'compact', label: '紧凑', description: '更小字号和间距，适合高信息密度文档。' },
  { id: 'reading', label: '长文阅读', description: '更宽松的行高和段落间距，适合长篇说明。' },
]

const defaultPreferences: DesktopPreferences = {
  themeMode: 'system',
  accentId: 'slate-indigo',
  customAccentColor: DEFAULT_CUSTOM_ACCENT_COLOR,
  markdownThemeId: 'atlas',
}

const darkTitlebarColors: Record<DesktopAccentId, string> = {
  custom: '#111528',
  'atlas-blue': '#1a2d57',
  'ocean-teal': '#163b3f',
  'forest-green': '#19392f',
  'sunset-amber': '#46361b',
  'dusty-rose': '#472433',
  'slate-indigo': '#2a3154',
  terracotta: '#4a2a23',
  'plum-orchid': '#402c49',
  'walnut-brown': '#3f3025',
}

const preferences = shallowRef<DesktopPreferences>(defaultPreferences)
let hasLoaded = false
let mediaQueryList: MediaQueryList | null = null
let cleanupMediaQueryListener: (() => void) | null = null

export function useDesktopPreferences() {
  ensurePreferencesLoaded()

  const currentAccent = computed(() => resolveAccentSelection(preferences.value))

  function setThemeMode(themeMode: DesktopThemeMode) {
    preferences.value = {
      ...preferences.value,
      themeMode,
    }
    persistPreferences()
    applyPreferences(preferences.value)
  }

  function setAccent(accentId: DesktopAccentId) {
    preferences.value = {
      ...preferences.value,
      accentId,
    }
    persistPreferences()
    applyPreferences(preferences.value)
  }

  function setCustomAccentColor(customAccentColor: string) {
    preferences.value = {
      ...preferences.value,
      accentId: 'custom',
      customAccentColor: normalizeHexColor(customAccentColor),
    }
    persistPreferences()
    applyPreferences(preferences.value)
  }

  function setMarkdownTheme(markdownThemeId: DesktopMarkdownThemeId) {
    preferences.value = {
      ...preferences.value,
      markdownThemeId,
    }
    persistPreferences()
    applyPreferences(preferences.value)
  }

  return {
    accentOptions,
    currentAccent,
    markdownThemeOptions,
    preferences,
    setAccent,
    setCustomAccentColor,
    setMarkdownTheme,
    setThemeMode,
  }
}

function ensurePreferencesLoaded() {
  if (hasLoaded) {
    return
  }

  hasLoaded = true

  if (typeof window === 'undefined') {
    return
  }

  try {
    const rawValue = window.localStorage.getItem(STORAGE_KEY)
    if (rawValue) {
      const parsed = JSON.parse(rawValue) as Partial<DesktopPreferences>
      preferences.value = {
        themeMode: isThemeMode(parsed.themeMode) ? parsed.themeMode : defaultPreferences.themeMode,
        accentId: resolveAccentId(parsed.accentId),
        customAccentColor: normalizeHexColor(
          typeof parsed.customAccentColor === 'string' ? parsed.customAccentColor : defaultPreferences.customAccentColor,
        ),
        markdownThemeId: isMarkdownThemeId(parsed.markdownThemeId)
          ? parsed.markdownThemeId
          : defaultPreferences.markdownThemeId,
      }
    }
  } catch {
    preferences.value = defaultPreferences
  }

  bindSystemThemeListener()
  applyPreferences(preferences.value)
}

function persistPreferences() {
  if (typeof window === 'undefined') {
    return
  }

  window.localStorage.setItem(STORAGE_KEY, JSON.stringify(preferences.value))
}

function applyPreferences(value: DesktopPreferences) {
  if (typeof document === 'undefined') {
    return
  }

  const root = document.documentElement
  const accent = resolveAccentSelection(value)
  const resolvedTheme = value.themeMode === 'system' ? getSystemTheme() : value.themeMode

  root.dataset.themeMode = value.themeMode
  root.dataset.theme = resolvedTheme
  root.dataset.themeAccent = accent.id
  root.dataset.markdownTheme = value.markdownThemeId
  root.style.setProperty('color-scheme', resolvedTheme)
  root.style.setProperty('--desktop-accent', accent.hex)
  root.style.setProperty('--desktop-accent-rgb', accent.rgb)
  root.style.setProperty('--desktop-titlebar-bg-runtime', resolveTitlebarColor(accent, resolvedTheme))
  root.style.setProperty('--desktop-scrollbar-thumb', resolveScrollbarThumb(accent.rgb, resolvedTheme))
  root.style.setProperty('--desktop-scrollbar-thumb-hover', resolveScrollbarThumbHover(accent.rgb, resolvedTheme))
  root.style.setProperty('--desktop-scrollbar-track', resolveScrollbarTrack(accent.rgb, resolvedTheme))

  void syncNativeWindowChrome(resolveTitlebarColor(accent, resolvedTheme))
}

function isThemeMode(value: unknown): value is DesktopThemeMode {
  return value === 'system' || value === 'light' || value === 'dark'
}

function isAccentId(value: unknown): value is DesktopAccentId {
  return value === 'custom' || accentOptions.some((option) => option.id === value)
}

function resolveAccentId(value: unknown): DesktopAccentId {
  if (value === 'custom') {
    return 'custom'
  }

  if (value === 'pure-white') {
    return 'slate-indigo'
  }

  return isAccentId(value) ? value : defaultPreferences.accentId
}

function isMarkdownThemeId(value: unknown): value is DesktopMarkdownThemeId {
  return markdownThemeOptions.some((option) => option.id === value)
}

function bindSystemThemeListener() {
  if (typeof window === 'undefined' || cleanupMediaQueryListener) {
    return
  }

  mediaQueryList = window.matchMedia('(prefers-color-scheme: dark)')
  const handleChange = () => {
    if (preferences.value.themeMode === 'system') {
      applyPreferences(preferences.value)
    }
  }

  mediaQueryList.addEventListener('change', handleChange)
  cleanupMediaQueryListener = () => {
    mediaQueryList?.removeEventListener('change', handleChange)
  }
}

function getSystemTheme(): 'light' | 'dark' {
  if (typeof window === 'undefined') {
    return 'light'
  }

  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
}

function resolveTitlebarColor(accent: DesktopAccentSelection, theme: 'light' | 'dark') {
  if (accent.id === 'custom') {
    return theme === 'dark' ? mixHexColor(accent.hex, '#101722', 0.36) : accent.hex
  }

  if (theme === 'dark') {
    return darkTitlebarColors[accent.id]
  }

  return accent.hex
}

async function syncNativeWindowChrome(color: string) {
  await syncWindowBackgroundColor(color)
}

function resolveScrollbarThumb(rgb: string, theme: 'light' | 'dark') {
  return `rgba(${rgb}, ${theme === 'dark' ? '0.48' : '0.30'})`
}

function resolveScrollbarThumbHover(rgb: string, theme: 'light' | 'dark') {
  return `rgba(${rgb}, ${theme === 'dark' ? '0.68' : '0.46'})`
}

function resolveScrollbarTrack(rgb: string, theme: 'light' | 'dark') {
  return `rgba(${rgb}, ${theme === 'dark' ? '0.16' : '0.08'})`
}

function resolveAccentSelection(value: DesktopPreferences): DesktopAccentSelection {
  if (value.accentId === 'custom') {
    const hex = normalizeHexColor(value.customAccentColor)
    return {
      id: 'custom',
      label: '自定义',
      hex,
      rgb: hexToRgbString(hex),
    }
  }

  return accentOptions.find((option) => option.id === value.accentId) ?? accentOptions[0]
}

function normalizeHexColor(value: string) {
  const trimmed = value.trim()
  if (/^#[0-9a-fA-F]{6}$/.test(trimmed)) {
    return trimmed.toUpperCase()
  }

  if (/^#[0-9a-fA-F]{3}$/.test(trimmed)) {
    const [r, g, b] = trimmed.slice(1).split('')
    return `#${r}${r}${g}${g}${b}${b}`.toUpperCase()
  }

  return DEFAULT_CUSTOM_ACCENT_COLOR
}

function hexToRgbString(hex: string) {
  const normalizedHex = normalizeHexColor(hex)
  const red = Number.parseInt(normalizedHex.slice(1, 3), 16)
  const green = Number.parseInt(normalizedHex.slice(3, 5), 16)
  const blue = Number.parseInt(normalizedHex.slice(5, 7), 16)
  return `${red}, ${green}, ${blue}`
}

function mixHexColor(foregroundHex: string, backgroundHex: string, foregroundWeight: number) {
  const fg = hexToRgbTuple(foregroundHex)
  const bg = hexToRgbTuple(backgroundHex)
  const weight = Math.min(1, Math.max(0, foregroundWeight))
  const inverseWeight = 1 - weight
  const red = Math.round(fg[0] * weight + bg[0] * inverseWeight)
  const green = Math.round(fg[1] * weight + bg[1] * inverseWeight)
  const blue = Math.round(fg[2] * weight + bg[2] * inverseWeight)
  return rgbTupleToHex([red, green, blue])
}

function hexToRgbTuple(hex: string): [number, number, number] {
  const normalizedHex = normalizeHexColor(hex)
  return [
    Number.parseInt(normalizedHex.slice(1, 3), 16),
    Number.parseInt(normalizedHex.slice(3, 5), 16),
    Number.parseInt(normalizedHex.slice(5, 7), 16),
  ]
}

function rgbTupleToHex([red, green, blue]: [number, number, number]) {
  const clamp = (value: number) => Math.min(255, Math.max(0, value))
  return `#${[clamp(red), clamp(green), clamp(blue)]
    .map((value) => value.toString(16).padStart(2, '0'))
    .join('')}`.toUpperCase()
}
