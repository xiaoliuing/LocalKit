<script setup lang="ts">
import { computed, shallowRef, watch } from 'vue'
import DesktopUiIcon from '@/components/ui/DesktopUiIcon.vue'

const props = defineProps<{
  feedbackMessage: string
  isOpen: boolean
  isSaving: boolean
}>()

const emit = defineEmits<{
  cancel: []
  chooseFolder: []
  save: [input: { title: string; path: string }]
}>()

const title = shallowRef('')
const path = shallowRef('')
const canSave = computed(() => path.value.trim().length > 0 && !props.isSaving)

watch(() => props.isOpen, (isOpen) => {
  if (!isOpen) {
    title.value = ''
    path.value = ''
  }
})

function setPath(nextPath: string) {
  path.value = nextPath
  if (!title.value.trim()) {
    title.value = getDirectoryName(nextPath)
  }
}

function submit() {
  if (!canSave.value) {
    return
  }

  emit('save', { title: title.value, path: path.value })
}

function getDirectoryName(value: string) {
  const normalized = value.trim().replace(/\\/g, '/')
  return normalized.split('/').filter(Boolean).at(-1) || '视频目录'
}

defineExpose({ setPath })
</script>

<template>
  <Teleport to="body">
    <div
      v-if="isOpen"
      class="desktop-video-source-dialog"
      @mousedown.self="emit('cancel')"
    >
      <form class="desktop-video-source-dialog__panel" @submit.prevent="submit">
        <header class="desktop-video-source-dialog__header">
          <div>
            <h2>添加视频目录</h2>
            <p>选择本地目录，子文件夹会自动生成目录树。</p>
          </div>
          <button type="button" title="关闭" @click="emit('cancel')">
            <DesktopUiIcon name="close" :size="17" />
          </button>
        </header>

        <div class="desktop-video-source-dialog__body">
          <label class="desktop-video-source-dialog__field">
            <span>显示名称 <small>可选</small></span>
            <input v-model="title" type="text" placeholder="例如：产品课程" />
          </label>

          <label class="desktop-video-source-dialog__field">
            <span>目录路径</span>
            <div class="desktop-video-source-dialog__path">
              <input v-model="path" type="text" placeholder="选择目录或粘贴本地路径" />
              <button type="button" @click="emit('chooseFolder')">选择目录</button>
            </div>
          </label>

          <p v-if="feedbackMessage" class="desktop-video-source-dialog__feedback">
            {{ feedbackMessage }}
          </p>
        </div>

        <footer class="desktop-video-source-dialog__footer">
          <button type="button" @click="emit('cancel')">取消</button>
          <button class="desktop-video-source-dialog__save" type="submit" :disabled="!canSave">
            {{ isSaving ? '正在扫描…' : '添加目录' }}
          </button>
        </footer>
      </form>
    </div>
  </Teleport>
</template>

<style scoped>
.desktop-video-source-dialog {
  position: fixed;
  z-index: 420;
  inset: 0;
  display: grid;
  place-items: center;
  padding: 1.5rem;
  background: rgba(10, 16, 28, 0.46);
  backdrop-filter: blur(8px);
}

.desktop-video-source-dialog__panel {
  width: min(31rem, 100%);
  overflow: hidden;
  border: 1px solid color-mix(in srgb, var(--desktop-line-strong) 76%, var(--desktop-accent));
  border-radius: 16px;
  background: var(--desktop-surface-strong);
  box-shadow: 0 24px 80px rgba(var(--desktop-shadow), 0.34);
}

.desktop-video-source-dialog__header,
.desktop-video-source-dialog__footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  padding: 1rem 1.1rem;
}

.desktop-video-source-dialog__header {
  border-bottom: 1px solid var(--desktop-line);
}

.desktop-video-source-dialog__header h2,
.desktop-video-source-dialog__header p {
  margin: 0;
}

.desktop-video-source-dialog__header h2 {
  color: var(--desktop-ink);
  font-size: 0.96rem;
}

.desktop-video-source-dialog__header p {
  margin-top: 0.2rem;
  color: var(--desktop-muted);
  font-size: 0.72rem;
}

.desktop-video-source-dialog button {
  border: 0;
  font: inherit;
  cursor: pointer;
}

.desktop-video-source-dialog__header button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 2rem;
  height: 2rem;
  border-radius: 9px;
  background: transparent;
  color: var(--desktop-muted);
}

.desktop-video-source-dialog__header button:hover {
  background: rgba(var(--desktop-accent-rgb), 0.08);
  color: var(--desktop-accent);
}

.desktop-video-source-dialog__body {
  display: grid;
  gap: 0.9rem;
  padding: 1.15rem;
}

.desktop-video-source-dialog__field {
  display: grid;
  gap: 0.4rem;
}

.desktop-video-source-dialog__field > span {
  color: var(--desktop-ink);
  font-size: 0.74rem;
  font-weight: 720;
}

.desktop-video-source-dialog__field small {
  color: var(--desktop-soft);
  font-weight: 500;
}

.desktop-video-source-dialog__field input {
  min-width: 0;
  height: 2.35rem;
  padding: 0 0.72rem;
  border: 1px solid var(--desktop-line-strong);
  border-radius: 9px;
  outline: none;
  background: var(--desktop-input-bg, var(--desktop-surface));
  color: var(--desktop-ink);
  font: inherit;
  font-size: 0.76rem;
}

.desktop-video-source-dialog__field input:focus {
  border-color: rgba(var(--desktop-accent-rgb), 0.64);
  box-shadow: 0 0 0 3px rgba(var(--desktop-accent-rgb), 0.1);
}

.desktop-video-source-dialog__path {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 0.5rem;
}

.desktop-video-source-dialog__path button,
.desktop-video-source-dialog__footer button {
  min-height: 2.35rem;
  padding: 0 0.85rem;
  border-radius: 9px;
  background: rgba(var(--desktop-accent-rgb), 0.08);
  color: var(--desktop-accent);
  font-size: 0.74rem;
  font-weight: 720;
}

.desktop-video-source-dialog__feedback {
  margin: 0;
  color: var(--desktop-muted);
  font-size: 0.72rem;
}

.desktop-video-source-dialog__footer {
  justify-content: flex-end;
  border-top: 1px solid var(--desktop-line);
}

.desktop-video-source-dialog__footer .desktop-video-source-dialog__save {
  background: var(--desktop-accent);
  color: #fff;
}

.desktop-video-source-dialog__footer button:disabled {
  opacity: 0.48;
  cursor: default;
}
</style>
