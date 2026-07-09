<script setup lang="ts">
import { computed } from 'vue'
import type {
  DesktopAccentId,
  DesktopAccentOption,
  DesktopMarkdownThemeId,
  DesktopMarkdownThemeOption,
  DesktopThemeMode,
} from '@/composables/useDesktopPreferences'
import './desktop-settings-shared.css'

const props = defineProps<{
  accentId: DesktopAccentId
  accentOptions: DesktopAccentOption[]
  markdownThemeId: DesktopMarkdownThemeId
  markdownThemeOptions: DesktopMarkdownThemeOption[]
  themeMode: DesktopThemeMode
}>()

const emit = defineEmits<{
  updateAccent: [accentId: DesktopAccentId]
  updateMarkdownTheme: [themeId: DesktopMarkdownThemeId]
  updateThemeMode: [themeMode: DesktopThemeMode]
}>()

const currentAccentLabel = computed(
  () => props.accentOptions.find((item) => item.id === props.accentId)?.label ?? '默认',
)

const themeModeOptions: Array<{
  value: DesktopThemeMode
  label: string
}> = [
  { value: 'system', label: '跟随系统' },
  { value: 'light', label: '浅色' },
  { value: 'dark', label: '深色' },
]
</script>

<template>
  <section class="desktop-settings-page">
    <header class="desktop-settings-page__hero">
      <h3>外观</h3>
      <p>调整主题模式、主题色和正文排版风格。</p>
      <span class="desktop-settings-page__badge">当前主题 · {{ currentAccentLabel }}</span>
    </header>

    <div class="desktop-settings-page__body">
      <div class="desktop-settings-page__column">
        <section class="desktop-settings-page__section">
          <h4>主题模式</h4>
          <div class="desktop-settings-page__options desktop-settings-page__options--inline">
            <button
              v-for="option in themeModeOptions"
              :key="option.value"
              :class="[
                'desktop-settings-page__option',
                { 'desktop-settings-page__option--active': props.themeMode === option.value },
              ]"
              type="button"
              @click="emit('updateThemeMode', option.value)"
            >
              <span class="desktop-settings-page__option-copy">
                <strong>{{ option.label }}</strong>
              </span>
            </button>
          </div>
        </section>

        <section class="desktop-settings-page__section">
          <h4>主题色</h4>
          <div class="desktop-settings-page__colors">
            <button
              v-for="option in props.accentOptions"
              :key="option.id"
              :aria-label="`切换主题色 ${option.label}`"
              :class="[
                'desktop-settings-page__color',
                { 'desktop-settings-page__color--active': props.accentId === option.id },
              ]"
              :style="{ '--accent-color': option.hex }"
              type="button"
              @click="emit('updateAccent', option.id)"
            >
              <span class="desktop-settings-page__color-swatch" />
              <span>{{ option.label }}</span>
            </button>
          </div>
        </section>
      </div>

      <div class="desktop-settings-page__column">
        <section class="desktop-settings-page__section">
          <h4>Markdown 阅读主题</h4>
          <div class="desktop-settings-page__themes">
            <button
              v-for="option in props.markdownThemeOptions"
              :key="option.id"
              :class="[
                'desktop-settings-page__theme',
                { 'desktop-settings-page__theme--active': props.markdownThemeId === option.id },
              ]"
              type="button"
              @click="emit('updateMarkdownTheme', option.id)"
            >
              <strong>{{ option.label }}</strong>
              <span>{{ option.description }}</span>
            </button>
          </div>
        </section>
      </div>
    </div>
  </section>
</template>
