import { shallowRef } from 'vue'
import { getVersion } from '@tauri-apps/api/app'
import { relaunch } from '@tauri-apps/plugin-process'
import { check } from '@tauri-apps/plugin-updater'
import { openExternalUrl } from '@/api/system'

const RELEASES_URL = 'https://github.com/xiaoliuing/docs-atlas/releases'
const UPDATER_MANIFEST_URLS = [
  'https://raw.githubusercontent.com/xiaoliuing/docs-atlas/main/desktop-updater/latest.json',
  'https://cdn.jsdelivr.net/gh/xiaoliuing/docs-atlas@main/desktop-updater/latest.json',
]

export type DesktopReleaseUpdateStatus =
  | 'idle'
  | 'checking'
  | 'available'
  | 'downloading'
  | 'installing'
  | 'relaunching'
  | 'up-to-date'
  | 'error'
  | 'unsupported'

export type DesktopLatestRelease = {
  version: string
  name: string
  htmlUrl: string
  publishedAt: string
}

type DesktopReleaseSummary = {
  version: string
  name: string
  htmlUrl: string
  publishedAt: string
}

export function useDesktopReleaseUpdates() {
  const currentVersion = shallowRef('')
  const latestRelease = shallowRef<DesktopLatestRelease | null>(null)
  const lastCheckedAt = shallowRef('')
  const status = shallowRef<DesktopReleaseUpdateStatus>('idle')
  const message = shallowRef('')

  async function loadCurrentVersion() {
    if (!isTauriRuntime()) {
      currentVersion.value = ''
      status.value = 'unsupported'
      message.value = '仅桌面应用支持检查更新。'
      return
    }

    try {
      currentVersion.value = await getVersion()
      if (status.value === 'unsupported') {
        status.value = 'idle'
        message.value = ''
      }
    } catch (error) {
      status.value = 'error'
      message.value = error instanceof Error ? error.message : '无法读取当前版本'
    }
  }

  async function checkForUpdates() {
    if (!isTauriRuntime()) {
      status.value = 'unsupported'
      message.value = '仅桌面应用支持检查更新。'
      return
    }

    if (!currentVersion.value) {
      await loadCurrentVersion()
      if (!currentVersion.value) {
        status.value = 'error'
        message.value = '无法读取当前版本'
        return
      }
    }

    status.value = 'checking'
    message.value = '正在检查更新…'

    try {
      const update = await checkForUpdatesWithRetry()
      lastCheckedAt.value = new Date().toISOString()

      if (update) {
        latestRelease.value = toLatestRelease({
          version: update.version,
          name: `LocalKit ${update.version}`,
          htmlUrl: buildReleaseUrl(update.version),
          publishedAt: update.date ?? '',
        })
        status.value = 'available'
        message.value = `发现新版本 ${update.version}`
        return
      }

      status.value = 'up-to-date'
      message.value = `当前已是最新版本 ${currentVersion.value}`
      latestRelease.value = null
    } catch (error) {
      const fallbackRelease = await fetchLatestReleaseSummary()

      if (fallbackRelease) {
        lastCheckedAt.value = new Date().toISOString()
        latestRelease.value = toLatestRelease(fallbackRelease)

        if (compareVersions(fallbackRelease.version, currentVersion.value) > 0) {
          status.value = 'error'
          message.value = `已检测到新版本 ${fallbackRelease.version}，但应用内检查通道不可用，请先从 GitHub Release 手动更新。`
          return
        }

        status.value = 'up-to-date'
        message.value = `当前已是最新版本 ${currentVersion.value}`
        return
      }

      status.value = 'error'
      message.value = normalizeUnknownError(error, '检查更新失败，请稍后重试')
    }
  }

  async function installUpdate() {
    if (!isTauriRuntime()) {
      status.value = 'unsupported'
      message.value = '仅桌面应用支持安装更新。'
      return
    }

    status.value = 'checking'
    message.value = '正在准备更新包…'

    try {
      const update = await checkForUpdatesWithRetry()

      if (!update) {
        status.value = 'up-to-date'
        message.value = `当前已是最新版本 ${currentVersion.value}`
        latestRelease.value = null
        return
      }

      latestRelease.value = toLatestRelease({
        version: update.version,
        name: `LocalKit ${update.version}`,
        htmlUrl: buildReleaseUrl(update.version),
        publishedAt: update.date ?? '',
      })
      lastCheckedAt.value = new Date().toISOString()

      let downloadedBytes = 0
      let contentLength: number | null = null
      status.value = 'downloading'
      message.value = `正在下载更新 ${update.version}…`

      await retry(async () => {
        downloadedBytes = 0
        contentLength = null

        await update.downloadAndInstall((event) => {
          if (event.event === 'Started') {
            contentLength = event.data.contentLength ?? null
            status.value = 'downloading'
            message.value = contentLength
              ? `正在下载更新… 0%`
              : `正在下载更新 ${update.version}…`
            return
          }

          if (event.event === 'Progress') {
            downloadedBytes += event.data.chunkLength
            status.value = 'downloading'

            const percent = contentLength
              ? Math.min(100, Math.round((downloadedBytes / contentLength) * 100))
              : null

            message.value =
              percent === null
                ? `正在下载更新… ${(downloadedBytes / 1024 / 1024).toFixed(1)} MB`
                : `正在下载更新… ${percent}%`
            return
          }

          status.value = 'installing'
          message.value = '正在安装更新…'
        })
      }, 2)

      status.value = 'relaunching'
      message.value = '更新已安装，正在重启应用…'
      await relaunch()
    } catch (error) {
      status.value = 'error'
      message.value = normalizeUnknownError(error, '安装更新失败，请稍后重试')
    }
  }

  async function openLatestRelease() {
    const targetUrl = latestRelease.value?.htmlUrl || RELEASES_URL
    await openExternalUrl(targetUrl)
  }

  return {
    checkForUpdates,
    currentVersion,
    installUpdate,
    lastCheckedAt,
    latestRelease,
    loadCurrentVersion,
    message,
    openLatestRelease,
    status,
  }
}

function isTauriRuntime() {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

function toLatestRelease(release: DesktopReleaseSummary): DesktopLatestRelease {
  return {
    version: release.version,
    name: release.name,
    htmlUrl: release.htmlUrl,
    publishedAt: release.publishedAt,
  }
}

function buildReleaseUrl(version: string) {
  return `${RELEASES_URL}/tag/desktop-v${version}`
}

async function fetchLatestReleaseSummary(): Promise<DesktopReleaseSummary | null> {
  for (const endpoint of UPDATER_MANIFEST_URLS) {
    try {
      const response = await fetch(endpoint, {
        cache: 'no-store',
      })

      if (!response.ok) {
        continue
      }

      const payload = (await response.json()) as {
        version?: string
        notes?: string
        pub_date?: string
      }
      const version = String(payload.version ?? '').trim()

      if (!version) {
        continue
      }

      return {
        version,
        name: `LocalKit ${version}`,
        htmlUrl: buildReleaseUrl(version),
        publishedAt: payload.pub_date?.trim() || '',
      }
    } catch {
      continue
    }
  }

  return null
}

async function checkForUpdatesWithRetry() {
  return retry(() => check(), 2)
}

async function retry<T>(task: () => Promise<T>, attempts: number): Promise<T> {
  let lastError: unknown

  for (let index = 0; index < attempts; index += 1) {
    try {
      return await task()
    } catch (error) {
      lastError = error

      if (index < attempts - 1) {
        await wait(800 * (index + 1))
      }
    }
  }

  throw lastError
}

function wait(durationMs: number) {
  return new Promise((resolve) => {
    window.setTimeout(resolve, durationMs)
  })
}

function compareVersions(left: string, right: string) {
  const leftParts = splitVersion(left)
  const rightParts = splitVersion(right)
  const size = Math.max(leftParts.length, rightParts.length)

  for (let index = 0; index < size; index += 1) {
    const leftValue = leftParts[index] ?? 0
    const rightValue = rightParts[index] ?? 0

    if (leftValue > rightValue) {
      return 1
    }

    if (leftValue < rightValue) {
      return -1
    }
  }

  return 0
}

function splitVersion(version: string) {
  return version
    .split('.')
    .map((part) => Number.parseInt(part, 10))
    .filter((part) => Number.isFinite(part))
}

function normalizeUnknownError(error: unknown, fallbackMessage: string) {
  if (error instanceof Error && error.message.trim()) {
    return error.message
  }

  if (typeof error === 'string' && error.trim()) {
    return error
  }

  if (error && typeof error === 'object') {
    const message = Reflect.get(error, 'message')
    if (typeof message === 'string' && message.trim()) {
      return message
    }

    try {
      return JSON.stringify(error)
    } catch {
      return fallbackMessage
    }
  }

  return fallbackMessage
}
