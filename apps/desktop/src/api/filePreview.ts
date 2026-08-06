import { convertFileSrc, invoke } from '@tauri-apps/api/core'

export type PreviewTreeNode = {
  id: string
  name: string
  path: string
  relativePath: string
  kind: 'folder' | 'file'
  size: number
  modifiedAt: number
  children: PreviewTreeNode[]
}

export type PreviewDirectoryScanResult = {
  exists: boolean
  isDirectory: boolean
  message: string
  fileCount: number
  tree: PreviewTreeNode[]
}

export type PreviewTextFileResult = {
  content: string
  encoding: string
}

const LEGACY_PLAIN_TEXT_EXTENSIONS = new Set([
  'bat',
  'cfg',
  'cmd',
  'conf',
  'ini',
  'log',
  'nfo',
  'properties',
  'txt',
])

export const DEFAULT_PREVIEW_MAX_DEPTH = 3

export const PREVIEW_MAX_DEPTH_OPTIONS = [
  { value: 1, label: '1 层' },
  { value: 2, label: '2 层' },
  { value: 3, label: '3 层（默认）' },
  { value: 4, label: '4 层' },
  { value: 5, label: '5 层' },
  { value: 10, label: '10 层' },
  { value: 0, label: '不限制' },
] as const

export async function scanPreviewDirectory(
  path: string,
  maxDepth = DEFAULT_PREVIEW_MAX_DEPTH,
): Promise<PreviewDirectoryScanResult> {
  if (isTauriRuntime()) {
    return invoke<PreviewDirectoryScanResult>('scan_preview_directory', {
      path,
      maxDepth,
    })
  }

  const trimmedPath = path.trim()
  return {
    exists: Boolean(trimmedPath),
    isDirectory: Boolean(trimmedPath),
    message: trimmedPath ? '浏览器预览模式不扫描本地目录' : '目录路径不能为空',
    fileCount: 0,
    tree: [],
  }
}

export function toPreviewAssetUrl(path: string) {
  if (!path.trim()) {
    return ''
  }

  if (isTauriRuntime()) {
    return convertFileSrc(path)
  }

  return path
}

export function isLegacyPlainTextPreview(filename: string) {
  const normalized = filename.trim().toLowerCase()
  const extension = normalized.includes('.')
    ? normalized.split('.').at(-1) ?? ''
    : ''
  return LEGACY_PLAIN_TEXT_EXTENSIONS.has(extension)
}

export async function readPreviewTextFile(
  path: string,
): Promise<PreviewTextFileResult | null> {
  if (!isTauriRuntime()) {
    return null
  }

  try {
    return await invoke<PreviewTextFileResult>('read_preview_text_file', { path })
  } catch {
    return null
  }
}

function isTauriRuntime() {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}
