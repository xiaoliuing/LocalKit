import { convertFileSrc, invoke } from '@tauri-apps/api/core'

export type VideoTreeNode = {
  id: string
  name: string
  path: string
  relativePath: string
  kind: 'folder' | 'file'
  size: number
  modifiedAt: number
  children: VideoTreeNode[]
}

export type VideoDirectoryScanResult = {
  exists: boolean
  isDirectory: boolean
  message: string
  videoCount: number
  tree: VideoTreeNode[]
}

export async function scanVideoDirectory(path: string): Promise<VideoDirectoryScanResult> {
  if (isTauriRuntime()) {
    return invoke<VideoDirectoryScanResult>('scan_video_directory', { path })
  }

  const trimmedPath = path.trim()
  return {
    exists: Boolean(trimmedPath),
    isDirectory: Boolean(trimmedPath),
    message: trimmedPath ? '浏览器预览模式不扫描本地视频' : '视频目录不能为空',
    videoCount: 0,
    tree: [],
  }
}

export function toVideoAssetUrl(path: string) {
  if (!path.trim()) {
    return ''
  }

  if (isTauriRuntime()) {
    return convertFileSrc(path)
  }

  return path
}

export function getVideoMimeType(path: string) {
  const extension = path.split('.').at(-1)?.toLowerCase()
  const mimeTypes: Record<string, string> = {
    avi: 'video/x-msvideo',
    m4v: 'video/x-m4v',
    mkv: 'video/x-matroska',
    mov: 'video/quicktime',
    mp4: 'video/mp4',
    ogv: 'video/ogg',
    webm: 'video/webm',
  }

  return extension ? mimeTypes[extension] ?? 'video/mp4' : 'video/mp4'
}

function isTauriRuntime() {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}
