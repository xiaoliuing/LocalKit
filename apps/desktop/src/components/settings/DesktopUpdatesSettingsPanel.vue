<script setup lang="ts">
import { computed } from 'vue'
import type { DesktopLatestRelease, DesktopReleaseUpdateStatus } from '@/composables/useDesktopReleaseUpdates'
import './desktop-settings-shared.css'

const props = defineProps<{
  currentVersion?: string
  lastCheckedAt?: string
  latestRelease: DesktopLatestRelease | null
  updateMessage?: string
  updateStatus: DesktopReleaseUpdateStatus
}>()

const emit = defineEmits<{
  checkUpdates: []
  installUpdate: []
  openLatestRelease: []
}>()

const updateActionLabel = computed(() => {
  if (props.updateStatus === 'checking') return '检查中…'
  if (props.updateStatus === 'available') return '下载并安装'
  if (props.updateStatus === 'downloading') return '下载中…'
  if (props.updateStatus === 'installing' || props.updateStatus === 'relaunching') return '安装中…'
  return '检查更新'
})

const updateBusy = computed(
  () =>
    props.updateStatus === 'checking' ||
    props.updateStatus === 'downloading' ||
    props.updateStatus === 'installing' ||
    props.updateStatus === 'relaunching',
)

const actionTitle = computed(() => {
  if (props.latestRelease?.version) {
    return `最新版本 ${props.latestRelease.version}`
  }

  return '检查最新稳定版'
})

const actionSubtitle = computed(() => {
  if (props.latestRelease?.name) {
    const publishedAt = props.latestRelease.publishedAt
      ? new Date(props.latestRelease.publishedAt)
      : null

    if (publishedAt && !Number.isNaN(publishedAt.getTime())) {
      const dateLabel = new Intl.DateTimeFormat('zh-CN', { dateStyle: 'medium' }).format(publishedAt)
      return `${props.latestRelease.name} · 发布于 ${dateLabel}`
    }

    return props.latestRelease.name
  }

  if (props.lastCheckedAt) {
    const date = new Date(props.lastCheckedAt)
    if (!Number.isNaN(date.getTime())) {
      return `最近检查：${new Intl.DateTimeFormat('zh-CN', {
        dateStyle: 'medium',
        timeStyle: 'short',
      }).format(date)}`
    }
  }

  return '点击按钮检查是否有新版本。'
})
</script>

<template>
  <section class="desktop-settings-page">
    <header class="desktop-settings-page__hero">
      <h3>应用更新</h3>
      <p>检查并安装最新稳定版。</p>
      <span class="desktop-settings-page__badge">当前版本 · {{ props.currentVersion || '读取中…' }}</span>
    </header>

    <div class="desktop-settings-page__stack">
      <div class="desktop-settings-page__action">
        <div class="desktop-settings-page__action-copy">
          <strong>{{ actionTitle }}</strong>
          <span>{{ actionSubtitle }}</span>
        </div>
        <button
          :class="[
            'desktop-settings-page__action-button',
            { 'desktop-settings-page__action-button--primary': props.updateStatus === 'available' },
          ]"
          :disabled="updateBusy"
          type="button"
          @click="props.updateStatus === 'available' ? emit('installUpdate') : emit('checkUpdates')"
        >
          {{ updateActionLabel }}
        </button>
      </div>

      <p
        v-if="props.updateMessage"
        :class="[
          'desktop-settings-page__feedback',
          { 'desktop-settings-page__feedback--error': props.updateStatus === 'error' },
        ]"
      >
        {{ props.updateMessage }}
      </p>

      <button class="desktop-settings-page__link" type="button" @click="emit('openLatestRelease')">
        在浏览器中打开 GitHub Release 页面
      </button>
    </div>
  </section>
</template>
