<script setup lang="ts">
import './desktop-settings-shared.css'

const props = defineProps<{
  actionMessage?: string
  busyAction?: 'app-data' | 'logs' | 'export' | null
}>()

const emit = defineEmits<{
  exportLogs: []
  openAppDataDirectory: []
  openLogsDirectory: []
}>()
</script>

<template>
  <section class="desktop-settings-page">
    <header class="desktop-settings-page__hero">
      <h3>数据与日志</h3>
      <p>打开本地目录或导出日志，不会影响当前阅读位置。</p>
      <span class="desktop-settings-page__badge">系统级操作</span>
    </header>

    <div class="desktop-settings-page__body">
      <div class="desktop-settings-page__column">
        <div class="desktop-settings-page__action">
          <div class="desktop-settings-page__action-copy">
            <strong>打开数据目录</strong>
            <span>数据库、缓存和导入导出文件。</span>
          </div>
          <button
            class="desktop-settings-page__action-button"
            :disabled="props.busyAction !== null"
            type="button"
            @click="emit('openAppDataDirectory')"
          >
            {{ props.busyAction === 'app-data' ? '处理中…' : '打开' }}
          </button>
        </div>

        <div class="desktop-settings-page__action">
          <div class="desktop-settings-page__action-copy">
            <strong>打开日志目录</strong>
            <span>定位本地运行日志，便于排查问题。</span>
          </div>
          <button
            class="desktop-settings-page__action-button"
            :disabled="props.busyAction !== null"
            type="button"
            @click="emit('openLogsDirectory')"
          >
            {{ props.busyAction === 'logs' ? '处理中…' : '打开' }}
          </button>
        </div>
      </div>

      <div class="desktop-settings-page__column">
        <div class="desktop-settings-page__action">
          <div class="desktop-settings-page__action-copy">
            <strong>导出日志文件</strong>
            <span>导出当前日志用于反馈和诊断。</span>
          </div>
          <button
            class="desktop-settings-page__action-button"
            :disabled="props.busyAction !== null"
            type="button"
            @click="emit('exportLogs')"
          >
            {{ props.busyAction === 'export' ? '处理中…' : '导出' }}
          </button>
        </div>

        <p v-if="props.actionMessage" class="desktop-settings-page__feedback">
          {{ props.actionMessage }}
        </p>
      </div>
    </div>
  </section>
</template>
