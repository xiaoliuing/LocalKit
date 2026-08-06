<script setup lang="ts">
import DesktopUiIcon from '@/components/ui/DesktopUiIcon.vue'
import {
  countSupportedPreviewExtensions,
  FILE_PREVIEW_SUPPORTED_GROUPS,
  FILE_PREVIEW_UNSUPPORTED_GROUPS,
} from '@/constants/filePreviewFormats'

defineProps<{
  isOpen: boolean
}>()

const emit = defineEmits<{
  close: []
}>()

const supportedExtensionCount = countSupportedPreviewExtensions()
</script>

<template>
  <Teleport to="body">
    <div
      v-if="isOpen"
      class="desktop-file-preview-format-dialog"
      @mousedown.self="emit('close')"
    >
      <section class="desktop-file-preview-format-dialog__panel">
        <header class="desktop-file-preview-format-dialog__header">
          <div>
            <h2>文件预览格式说明</h2>
            <p>
              当前已接入 {{ supportedExtensionCount }} 种扩展名，覆盖文档、图片、代码、压缩包、工程文件等。
            </p>
          </div>
          <button type="button" title="关闭" @click="emit('close')">
            <DesktopUiIcon name="close" :size="17" />
          </button>
        </header>

        <div class="desktop-file-preview-format-dialog__body desktop-scroll">
          <section class="desktop-file-preview-format-dialog__section">
            <h3>已支持预览</h3>
            <div
              v-for="group in FILE_PREVIEW_SUPPORTED_GROUPS"
              :key="group.id"
              class="desktop-file-preview-format-dialog__group"
            >
              <div class="desktop-file-preview-format-dialog__group-head">
                <strong>{{ group.label }}</strong>
                <span>{{ group.extensions.length }} 种</span>
              </div>
              <p class="desktop-file-preview-format-dialog__extensions">
                {{ group.extensions.map((item) => `.${item}`).join(' · ') }}
              </p>
              <p v-if="group.note" class="desktop-file-preview-format-dialog__note">
                {{ group.note }}
              </p>
            </div>
          </section>

          <section class="desktop-file-preview-format-dialog__section">
            <h3>暂不支持 / 仅下载</h3>
            <div
              v-for="group in FILE_PREVIEW_UNSUPPORTED_GROUPS"
              :key="group.label"
              class="desktop-file-preview-format-dialog__group desktop-file-preview-format-dialog__group--unsupported"
            >
              <div class="desktop-file-preview-format-dialog__group-head">
                <strong>{{ group.label }}</strong>
              </div>
              <p class="desktop-file-preview-format-dialog__extensions">
                {{ group.examples.map((item) => (item.startsWith('.') ? item : `.${item}`)).join(' · ') }}
              </p>
              <p class="desktop-file-preview-format-dialog__note">
                {{ group.reason }}
              </p>
            </div>
          </section>
        </div>
      </section>
    </div>
  </Teleport>
</template>

<style scoped>
.desktop-file-preview-format-dialog {
  position: fixed;
  z-index: 430;
  inset: 0;
  display: grid;
  place-items: center;
  padding: 1.5rem;
  background: rgba(10, 16, 28, 0.46);
  backdrop-filter: blur(8px);
}

.desktop-file-preview-format-dialog__panel {
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  width: min(42rem, 100%);
  max-height: min(80vh, 44rem);
  overflow: hidden;
  border: 1px solid
    color-mix(in srgb, var(--desktop-line-strong) 76%, var(--desktop-accent));
  border-radius: 16px;
  background: var(--desktop-surface-strong);
  box-shadow: 0 24px 80px rgba(var(--desktop-shadow), 0.34);
}

.desktop-file-preview-format-dialog__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
  padding: 1rem 1.1rem;
  border-bottom: 1px solid var(--desktop-line);
}

.desktop-file-preview-format-dialog__header h2,
.desktop-file-preview-format-dialog__header p {
  margin: 0;
}

.desktop-file-preview-format-dialog__header h2 {
  color: var(--desktop-ink);
  font-size: 0.96rem;
}

.desktop-file-preview-format-dialog__header p {
  margin-top: 0.25rem;
  color: var(--desktop-muted);
  font-size: 0.72rem;
  line-height: 1.5;
}

.desktop-file-preview-format-dialog__header button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 2rem;
  height: 2rem;
  border: 0;
  border-radius: 9px;
  background: transparent;
  color: var(--desktop-muted);
  cursor: pointer;
}

.desktop-file-preview-format-dialog__header button:hover {
  background: rgba(var(--desktop-accent-rgb), 0.08);
  color: var(--desktop-accent);
}

.desktop-file-preview-format-dialog__body {
  overflow: auto;
  padding: 1rem 1.1rem 1.15rem;
}

.desktop-file-preview-format-dialog__section + .desktop-file-preview-format-dialog__section {
  margin-top: 1.15rem;
  padding-top: 1.15rem;
  border-top: 1px solid var(--desktop-line);
}

.desktop-file-preview-format-dialog__section h3 {
  margin: 0 0 0.75rem;
  color: var(--desktop-ink);
  font-size: 0.82rem;
  font-weight: 720;
}

.desktop-file-preview-format-dialog__group + .desktop-file-preview-format-dialog__group {
  margin-top: 0.75rem;
}

.desktop-file-preview-format-dialog__group {
  padding: 0.72rem 0.78rem;
  border: 1px solid rgba(var(--desktop-accent-rgb), 0.16);
  border-radius: 10px;
  background: rgba(var(--desktop-accent-rgb), 0.04);
}

.desktop-file-preview-format-dialog__group--unsupported {
  border-color: var(--desktop-line);
  background: var(--desktop-surface);
}

.desktop-file-preview-format-dialog__group-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 0.75rem;
  margin-bottom: 0.35rem;
}

.desktop-file-preview-format-dialog__group-head strong {
  color: var(--desktop-ink);
  font-size: 0.76rem;
}

.desktop-file-preview-format-dialog__group-head span {
  flex: none;
  color: var(--desktop-soft);
  font-size: 0.64rem;
}

.desktop-file-preview-format-dialog__extensions {
  margin: 0;
  color: var(--desktop-muted);
  font-size: 0.68rem;
  line-height: 1.65;
  overflow-wrap: anywhere;
}

.desktop-file-preview-format-dialog__note {
  margin: 0.42rem 0 0;
  color: var(--desktop-soft);
  font-size: 0.64rem;
  line-height: 1.55;
}
</style>
