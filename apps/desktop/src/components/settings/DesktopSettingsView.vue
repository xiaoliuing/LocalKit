<script setup lang="ts">
import type {
  DesktopAccentId,
  DesktopAccentOption,
  DesktopMarkdownThemeId,
  DesktopMarkdownThemeOption,
  DesktopThemeMode,
} from '@/composables/useDesktopPreferences'
import type { DesktopLatestRelease, DesktopReleaseUpdateStatus } from '@/composables/useDesktopReleaseUpdates'
import DesktopUiIcon from '@/components/ui/DesktopUiIcon.vue'
import DesktopAppearanceSettingsPanel from '@/components/settings/DesktopAppearanceSettingsPanel.vue'
import DesktopDataSettingsPanel from '@/components/settings/DesktopDataSettingsPanel.vue'
import DesktopSettingsNav, { type DesktopSettingsSection } from '@/components/settings/DesktopSettingsNav.vue'
import DesktopUpdatesSettingsPanel from '@/components/settings/DesktopUpdatesSettingsPanel.vue'

const props = defineProps<{
  accentId: DesktopAccentId
  accentOptions: DesktopAccentOption[]
  actionMessage?: string
  activeSection: DesktopSettingsSection
  busyAction?: 'app-data' | 'logs' | 'export' | null
  currentVersion?: string
  lastCheckedAt?: string
  latestRelease: DesktopLatestRelease | null
  markdownThemeId: DesktopMarkdownThemeId
  markdownThemeOptions: DesktopMarkdownThemeOption[]
  themeMode: DesktopThemeMode
  updateMessage?: string
  updateStatus: DesktopReleaseUpdateStatus
}>()

const emit = defineEmits<{
  checkUpdates: []
  close: []
  exportLogs: []
  installUpdate: []
  openLatestRelease: []
  openAppDataDirectory: []
  openLogsDirectory: []
  selectSection: [section: DesktopSettingsSection]
  updateAccent: [accentId: DesktopAccentId]
  updateMarkdownTheme: [themeId: DesktopMarkdownThemeId]
  updateThemeMode: [themeMode: DesktopThemeMode]
}>()
</script>

<template>
  <section class="desktop-settings-view">
    <header class="desktop-settings-view__header">
      <div>
        <h2 class="desktop-settings-view__title">应用设置</h2>
        <p class="desktop-settings-view__summary">主题、更新、数据与日志。</p>
      </div>

      <button class="desktop-settings-view__back" type="button" @click="emit('close')">
        <DesktopUiIcon name="chevron-left" :size="16" />
        <span>返回阅读</span>
      </button>
    </header>

    <div class="desktop-settings-view__shell">
      <aside class="desktop-settings-view__nav">
        <DesktopSettingsNav :active-section="props.activeSection" @select="emit('selectSection', $event)" />
      </aside>

      <div class="desktop-settings-view__content desktop-scroll">
        <DesktopAppearanceSettingsPanel
          v-if="props.activeSection === 'appearance'"
          :accent-id="props.accentId"
          :accent-options="props.accentOptions"
          :markdown-theme-id="props.markdownThemeId"
          :markdown-theme-options="props.markdownThemeOptions"
          :theme-mode="props.themeMode"
          @update-accent="emit('updateAccent', $event)"
          @update-markdown-theme="emit('updateMarkdownTheme', $event)"
          @update-theme-mode="emit('updateThemeMode', $event)"
        />

        <DesktopUpdatesSettingsPanel
          v-else-if="props.activeSection === 'updates'"
          :current-version="props.currentVersion"
          :last-checked-at="props.lastCheckedAt"
          :latest-release="props.latestRelease"
          :update-message="props.updateMessage"
          :update-status="props.updateStatus"
          @check-updates="emit('checkUpdates')"
          @install-update="emit('installUpdate')"
          @open-latest-release="emit('openLatestRelease')"
        />

        <DesktopDataSettingsPanel
          v-else
          :action-message="props.actionMessage"
          :busy-action="props.busyAction"
          @export-logs="emit('exportLogs')"
          @open-app-data-directory="emit('openAppDataDirectory')"
          @open-logs-directory="emit('openLogsDirectory')"
        />
      </div>
    </div>
  </section>
</template>

<style scoped>
.desktop-settings-view {
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  gap: 0;
  min-height: 0;
  height: 100%;
  width: 100%;
}

.desktop-settings-view__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  padding: 1.15rem 1.5rem 0.85rem;
}

.desktop-settings-view__title {
  margin: 0;
  color: var(--desktop-ink);
  font-size: 1.05rem;
  font-weight: 600;
}

.desktop-settings-view__summary {
  margin: 0.2rem 0 0;
  color: var(--desktop-soft);
  font-size: 0.74rem;
}

.desktop-settings-view__back {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.42rem 0.62rem;
  border: 1px solid var(--desktop-line);
  border-radius: var(--desktop-radius-sm);
  background: transparent;
  color: var(--desktop-ink);
  cursor: pointer;
  transition: background-color 0.12s ease;
}

.desktop-settings-view__back:hover {
  background: rgba(var(--desktop-accent-rgb), 0.06);
}

.desktop-settings-view__back span {
  font-size: 0.74rem;
  font-weight: 600;
}

.desktop-settings-view__shell {
  display: grid;
  grid-template-columns: 13.5rem minmax(0, 1fr);
  gap: 0;
  min-height: 0;
  height: 100%;
  overflow: hidden;
  background: var(--desktop-surface-strong);
}

.desktop-settings-view__nav {
  min-height: 0;
  padding: 0.65rem 0.5rem;
  border-right: 1px solid var(--desktop-line);
  background: var(--desktop-surface);
}

.desktop-settings-view__content {
  min-height: 0;
  min-width: 0;
  overflow: auto;
  padding: 1.15rem 1.5rem 1.35rem;
  background: var(--desktop-surface-strong);
}

@media (max-width: 1180px) {
  .desktop-settings-view__shell {
    grid-template-columns: 1fr;
  }

  .desktop-settings-view__nav {
    padding: 0.5rem 0.75rem 0;
    border-right: 0;
    border-bottom: 1px solid var(--desktop-line);
  }
}
</style>
