<script setup lang="ts">
import { computed, reactive, shallowRef, watch } from 'vue'
import type { WorkspaceDetail, WorkspaceSourceNode, WorkspaceSourceNodeInput } from '@docs-atlas/shared-types/workspace'
import type { DesktopAccentOption } from '@/composables/useDesktopPreferences'
import DesktopUiIcon from '@/components/ui/DesktopUiIcon.vue'
import DesktopSourceTreeDialog from './DesktopSourceTreeDialog.vue'

type WorkspaceForm = {
  name: string
  description: string
  color: string
  sources: WorkspaceSourceNodeInput[]
}

type CreateEntryMode = 'create' | 'import'

const isOpen = defineModel<boolean>('open', { default: false })

const props = defineProps<{
  accentOptions: DesktopAccentOption[]
  canDelete?: boolean
  docCount?: number
  isDeleting?: boolean
  isExporting?: boolean
  isImporting?: boolean
  isSaving: boolean
  mode: 'create' | 'edit'
  sourceCount?: number
  unhealthySourceCount?: number
  workspaceCount?: number
  workspace?: WorkspaceDetail | null
}>()

const emit = defineEmits<{
  close: []
  delete: []
  export: []
  import: []
  submit: [payload: WorkspaceForm]
}>()

const form = reactive<WorkspaceForm>({
  name: '',
  description: '',
  color: '#1f54d9',
  sources: [],
})
const deleteConfirmState = reactive({ value: false })
const createEntryMode = shallowRef<CreateEntryMode>('create')
const isSourceDialogOpen = shallowRef(false)

const isValid = computed(() => form.name.trim().length > 0)
const isCreateMode = computed(() => props.mode === 'create')
const isEditMode = computed(() => props.mode === 'edit')
const isImportMode = computed(() => isCreateMode.value && createEntryMode.value === 'import')
const dialogTitle = computed(() => (isEditMode.value ? '编辑文档仓库' : '新建文档仓库'))
const submitLabel = computed(() => {
  if (props.isSaving) {
    return isEditMode.value ? '保存中...' : '创建中...'
  }

  return isEditMode.value ? '保存文档仓库' : '创建文档仓库'
})
const canDeleteWorkspace = computed(() => isEditMode.value && Boolean(props.canDelete))
const canExportWorkspace = computed(() => isEditMode.value && Boolean(props.workspace))
const deleteLabel = computed(() => {
  if (props.isDeleting) {
    return '删除中...'
  }

  return deleteConfirmState.value ? '确认删除文档仓库' : '删除文档仓库'
})
const importLabel = computed(() => (props.isImporting ? '导入中...' : '导入文档仓库'))
const totalSourceCount = computed(() => countFolderSources(form.sources))
const totalGroupCount = computed(() => countGroupSources(form.sources))
const draftWorkspace = computed<WorkspaceDetail | null>(() => ({
  id: props.workspace?.id ?? 'workspace:draft',
  name: form.name.trim() || '未命名文档仓库',
  description: form.description.trim(),
  icon: props.workspace?.icon ?? 'folder',
  color: form.color,
  defaultSearchScope: props.workspace?.defaultSearchScope ?? 'global',
  sortOrder: props.workspace?.sortOrder ?? 0,
  createdAt: props.workspace?.createdAt ?? '',
  updatedAt: props.workspace?.updatedAt ?? '',
  lastOpenedAt: props.workspace?.lastOpenedAt ?? null,
  sources: toWorkspaceNodes(form.sources, props.workspace?.id ?? 'workspace:draft'),
}))

watch(
  () => [isOpen.value, props.mode, props.workspace?.id] as const,
  ([open]) => {
    if (open) {
      fillForm()
      return
    }

    resetForm()
  },
  { immediate: true },
)

function resetForm() {
  form.name = ''
  form.description = ''
  form.color = props.accentOptions[0]?.hex ?? '#1f54d9'
  form.sources = []
  deleteConfirmState.value = false
  createEntryMode.value = 'create'
  isSourceDialogOpen.value = false
}

function fillForm() {
  deleteConfirmState.value = false
  isSourceDialogOpen.value = false
  createEntryMode.value = 'create'

  if (isEditMode.value && props.workspace) {
    form.name = props.workspace.name
    form.description = props.workspace.description
    form.color = props.workspace.color
    form.sources = cloneSourceInputs(toSourceInputs(props.workspace.sources))
    return
  }

  resetForm()
}

function handleSubmit() {
  if (!isValid.value || props.isSaving || isImportMode.value) {
    return
  }

  emit('submit', {
    name: form.name.trim(),
    description: form.description.trim(),
    color: form.color,
    sources: cloneSourceInputs(form.sources),
  })
}

function handleClose() {
  isOpen.value = false
  emit('close')
}

function handleDelete() {
  if (!canDeleteWorkspace.value || props.isDeleting) {
    return
  }

  if (!deleteConfirmState.value) {
    deleteConfirmState.value = true
    return
  }

  emit('delete')
}

function handleImport() {
  if (props.isImporting) {
    return
  }

  emit('import')
}

function handleExport() {
  if (!canExportWorkspace.value || props.isExporting) {
    return
  }

  emit('export')
}

function handleSourceSave(sources: WorkspaceSourceNodeInput[]) {
  form.sources = cloneSourceInputs(sources)
  isSourceDialogOpen.value = false
}

function countFolderSources(nodes: WorkspaceSourceNodeInput[]) {
  return nodes.reduce((count, node) => {
    const selfCount = node.kind === 'folder' ? 1 : 0
    return count + selfCount + countFolderSources(node.children ?? [])
  }, 0)
}

function countGroupSources(nodes: WorkspaceSourceNodeInput[]) {
  return nodes.reduce((count, node) => {
    const selfCount = node.kind === 'group' ? 1 : 0
    return count + selfCount + countGroupSources(node.children ?? [])
  }, 0)
}

function cloneSourceInputs(nodes: WorkspaceSourceNodeInput[]): WorkspaceSourceNodeInput[] {
  return nodes.map((node) => ({
    id: node.id,
    parentId: node.parentId ?? null,
    kind: node.kind,
    name: node.name,
    path: node.path ?? '',
    enabled: node.enabled ?? true,
    position: node.position ?? 0,
    children: cloneSourceInputs(node.children ?? []),
  }))
}

function toSourceInputs(nodes: WorkspaceSourceNode[]): WorkspaceSourceNodeInput[] {
  return nodes.map((node) => ({
    id: node.id,
    parentId: node.parentId,
    kind: node.kind,
    name: node.name,
    path: node.path,
    enabled: node.enabled,
    position: node.position,
    children: toSourceInputs(node.children),
  }))
}

function toWorkspaceNodes(nodes: WorkspaceSourceNodeInput[], workspaceId: string): WorkspaceSourceNode[] {
  return nodes.map((node, index) => ({
    id: node.id,
    workspaceId,
    parentId: node.parentId ?? null,
    kind: node.kind,
    name: node.name,
    path: node.path ?? '',
    enabled: node.enabled ?? true,
    position: node.position ?? index,
    children: toWorkspaceNodes(node.children ?? [], workspaceId),
  }))
}
</script>

<template>
  <div
    v-if="isOpen"
    class="desktop-workspace-dialog"
    @click="handleClose"
  >
    <section
      class="desktop-workspace-dialog__panel"
      @click.stop
    >
      <header class="desktop-workspace-dialog__header">
        <div class="desktop-workspace-dialog__header-copy">
          <h2 class="desktop-workspace-dialog__title">{{ dialogTitle }}</h2>
          <p v-if="isEditMode" class="desktop-workspace-dialog__summary">
            修改文档仓库的基础信息和文档源配置。
          </p>
          <p v-else-if="!isImportMode" class="desktop-workspace-dialog__summary">
            创建新的文档仓库，用于聚合不同项目的本地文档。
          </p>
        </div>

        <button
          :aria-label="isEditMode ? '关闭编辑文档仓库对话框' : '关闭新建文档仓库对话框'"
          class="desktop-workspace-dialog__close"
          type="button"
          @click="handleClose"
        >
          <DesktopUiIcon name="close" :size="16" />
        </button>
      </header>

      <div v-if="isCreateMode" class="desktop-workspace-dialog__tabs">
        <button
          :class="[
            'desktop-workspace-dialog__tab',
            { 'desktop-workspace-dialog__tab--active': createEntryMode === 'create' },
          ]"
          type="button"
          @click="createEntryMode = 'create'"
        >
          创建文档仓库
        </button>
        <button
          :class="[
            'desktop-workspace-dialog__tab',
            { 'desktop-workspace-dialog__tab--active': createEntryMode === 'import' },
          ]"
          type="button"
          @click="createEntryMode = 'import'"
        >
          导入配置
        </button>
      </div>

      <div v-if="isImportMode" class="desktop-workspace-dialog__body">
        <div class="desktop-workspace-dialog__import">
          <h3 class="desktop-workspace-dialog__import-title">从配置文件导入</h3>
          <p class="desktop-workspace-dialog__import-copy">
            适合直接接管已有的文档仓库定义，导入后会创建一个新的文档仓库实例。
          </p>
          <button
            :disabled="props.isImporting"
            class="desktop-workspace-dialog__primary"
            type="button"
            @click="handleImport"
          >
            {{ importLabel }}
          </button>
        </div>
      </div>

      <div v-else class="desktop-workspace-dialog__body">
        <section class="desktop-workspace-dialog__section">
          <div class="desktop-workspace-dialog__section-head">
            <h3 class="desktop-workspace-dialog__section-title">基础信息</h3>
            <button
              v-if="canExportWorkspace"
              :disabled="props.isExporting"
              class="desktop-workspace-dialog__section-action"
              type="button"
              @click="handleExport"
            >
              {{ props.isExporting ? '导出中...' : '导出配置' }}
            </button>
          </div>

          <label class="desktop-workspace-dialog__field">
            <span>名称</span>
            <input
              v-model="form.name"
              class="desktop-workspace-dialog__input"
              maxlength="48"
              placeholder="例如：后端设计 / 个人知识库"
              type="text"
            />
          </label>

          <label class="desktop-workspace-dialog__field">
            <span>描述</span>
            <textarea
              v-model="form.description"
              class="desktop-workspace-dialog__textarea"
              maxlength="160"
              placeholder="一句话说明这个文档仓库的用途。"
              rows="3"
            />
          </label>

          <div class="desktop-workspace-dialog__field">
            <span>标识色</span>
            <div class="desktop-workspace-dialog__colors">
              <button
                v-for="accent in props.accentOptions"
                :key="accent.id"
                :aria-label="`选择主题色 ${accent.label}`"
                :class="[
                  'desktop-workspace-dialog__color',
                  { 'desktop-workspace-dialog__color--active': form.color === accent.hex },
                ]"
                :style="{ '--workspace-color': accent.hex }"
                type="button"
                @click="form.color = accent.hex"
              />
            </div>
            <p class="desktop-workspace-dialog__field-hint">
              仅用于文档仓库圆点标记，不会修改应用全局主题。
            </p>
          </div>
        </section>

        <section class="desktop-workspace-dialog__section">
          <div class="desktop-workspace-dialog__section-head">
            <h3 class="desktop-workspace-dialog__section-title">文档源</h3>
            <button
              class="desktop-workspace-dialog__section-action"
              type="button"
              @click="isSourceDialogOpen = true"
            >
              {{ totalSourceCount > 0 || totalGroupCount > 0 ? '编辑文档源' : '设置文档源' }}
            </button>
          </div>

          <div class="desktop-workspace-dialog__stats">
            <span class="desktop-workspace-dialog__stat">{{ `${totalSourceCount} 个文档源` }}</span>
            <span class="desktop-workspace-dialog__stat">{{ `${totalGroupCount} 个分组` }}</span>
            <span
              v-if="isEditMode"
              class="desktop-workspace-dialog__stat"
            >
              {{ `${props.docCount ?? 0} 篇文档` }}
            </span>
            <span
              v-if="isEditMode && (props.unhealthySourceCount ?? 0) > 0"
              class="desktop-workspace-dialog__stat desktop-workspace-dialog__stat--warning"
            >
              {{ `${props.unhealthySourceCount} 个异常` }}
            </span>
          </div>
          <p class="desktop-workspace-dialog__field-hint">
            文档源非必填，可先创建仓库，后续再补充目录结构。
          </p>
        </section>

        <div
          v-if="canDeleteWorkspace"
          class="desktop-workspace-dialog__danger"
        >
          <h3 class="desktop-workspace-dialog__danger-title">删除当前文档仓库</h3>
          <p v-if="props.workspaceCount && props.workspaceCount > 1">
            删除后移除该仓库及其文档源配置，不会删除原始文档目录。
          </p>
          <p v-else>
            至少保留一个文档仓库，当前文档仓库不可删除。
          </p>

          <button
            :disabled="props.isDeleting || !props.canDelete"
            :class="[
              'desktop-workspace-dialog__danger-button',
              { 'desktop-workspace-dialog__danger-button--confirming': deleteConfirmState.value },
            ]"
            type="button"
            @click="handleDelete"
          >
            {{ deleteLabel }}
          </button>
        </div>
      </div>

      <footer class="desktop-workspace-dialog__footer">
        <button
          class="desktop-workspace-dialog__ghost"
          type="button"
          @click="handleClose"
        >
          取消
        </button>
        <button
          v-if="!isImportMode"
          :disabled="!isValid || props.isSaving"
          class="desktop-workspace-dialog__primary"
          type="button"
          @click="handleSubmit"
        >
          {{ submitLabel }}
        </button>
      </footer>

      <DesktopSourceTreeDialog
        v-model:open="isSourceDialogOpen"
        :is-saving="false"
        :is-scanning="false"
        :runtime-source-statuses-by-node-id="{}"
        :workspace="draftWorkspace"
        @close="isSourceDialogOpen = false"
        @save="handleSourceSave"
      />
    </section>
  </div>
</template>

<style scoped>
.desktop-workspace-dialog {
  position: fixed;
  inset: 0;
  z-index: 80;
  display: grid;
  place-items: center;
  padding: 1.25rem;
  overflow: auto;
  background: rgba(7, 13, 24, 0.2);
}

.desktop-workspace-dialog__panel {
  width: min(32rem, calc(100vw - 2rem));
  max-height: min(86vh, 52rem);
  display: flex;
  flex-direction: column;
  border: 1px solid var(--desktop-line);
  border-radius: var(--desktop-radius-md);
  background: var(--desktop-surface-strong);
  box-shadow: 0 16px 48px rgba(var(--desktop-shadow), 0.18);
  overflow: hidden;
}

.desktop-workspace-dialog__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 0.85rem;
  padding: 1.1rem 1.2rem 0.9rem;
  border-bottom: 1px solid var(--desktop-line-subtle, var(--desktop-line));
}

.desktop-workspace-dialog__header-copy {
  display: grid;
  gap: 0.28rem;
  min-width: 0;
}

.desktop-workspace-dialog__title {
  margin: 0;
  color: var(--desktop-ink);
  font-size: 1.02rem;
  font-weight: 600;
  line-height: 1.3;
}

.desktop-workspace-dialog__summary,
.desktop-workspace-dialog__field-hint,
.desktop-workspace-dialog__import-copy,
.desktop-workspace-dialog__danger p {
  margin: 0;
  color: var(--desktop-soft);
  font-size: 0.76rem;
  line-height: 1.55;
}

.desktop-workspace-dialog__close {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.75rem;
  height: 1.75rem;
  border: 0;
  border-radius: var(--desktop-radius-sm);
  background: transparent;
  color: var(--desktop-soft);
  cursor: pointer;
  flex-shrink: 0;
}

.desktop-workspace-dialog__close:hover {
  background: rgba(0, 0, 0, 0.04);
  color: var(--desktop-muted);
}

.desktop-workspace-dialog__tabs {
  display: flex;
  gap: 0;
  padding: 0 1.2rem;
  border-bottom: 1px solid var(--desktop-line-subtle, var(--desktop-line));
}

.desktop-workspace-dialog__tab {
  margin-bottom: -1px;
  padding: 0.62rem 0.9rem;
  border: 0;
  border-bottom: 2px solid transparent;
  background: transparent;
  color: var(--desktop-soft);
  font: inherit;
  font-size: 0.78rem;
  font-weight: 500;
  cursor: pointer;
}

.desktop-workspace-dialog__tab--active {
  color: var(--desktop-accent);
  border-bottom-color: var(--desktop-accent);
}

.desktop-workspace-dialog__body {
  display: grid;
  gap: 1.15rem;
  min-height: 0;
  overflow-y: auto;
  padding: 1rem 1.2rem 1.1rem;
}

.desktop-workspace-dialog__import {
  display: grid;
  gap: 0.65rem;
  justify-items: start;
}

.desktop-workspace-dialog__import-title,
.desktop-workspace-dialog__section-title {
  margin: 0;
  color: var(--desktop-ink);
  font-size: 0.8rem;
  font-weight: 600;
}

.desktop-workspace-dialog__danger-title {
  margin: 0;
  color: #dc2626;
  font-size: 0.8rem;
  font-weight: 600;
}

.desktop-workspace-dialog__section {
  display: grid;
  gap: 0.65rem;
}

.desktop-workspace-dialog__section-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
}

.desktop-workspace-dialog__section-action,
.desktop-workspace-dialog__ghost,
.desktop-workspace-dialog__primary,
.desktop-workspace-dialog__danger-button {
  min-height: 1.88rem;
  padding: 0 0.72rem;
  border-radius: var(--desktop-radius-sm);
  font: inherit;
  font-size: 0.76rem;
  font-weight: 500;
  cursor: pointer;
}

.desktop-workspace-dialog__section-action,
.desktop-workspace-dialog__ghost {
  border: 1px solid var(--desktop-line);
  background: transparent;
  color: var(--desktop-muted);
}

.desktop-workspace-dialog__section-action:hover:not(:disabled),
.desktop-workspace-dialog__ghost:hover:not(:disabled) {
  background: rgba(0, 0, 0, 0.03);
  color: var(--desktop-ink);
}

.desktop-workspace-dialog__primary {
  border: 1px solid var(--desktop-accent);
  background: var(--desktop-accent);
  color: #fff;
}

.desktop-workspace-dialog__primary:disabled,
.desktop-workspace-dialog__danger-button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.desktop-workspace-dialog__field {
  display: grid;
  gap: 0.38rem;
}

.desktop-workspace-dialog__field > span {
  color: var(--desktop-muted);
  font-size: 0.76rem;
  font-weight: 500;
}

.desktop-workspace-dialog__input,
.desktop-workspace-dialog__textarea {
  width: 100%;
  border: 1px solid var(--desktop-line);
  border-radius: var(--desktop-radius-sm);
  background: var(--desktop-surface-strong);
  color: var(--desktop-ink);
  font: inherit;
  font-size: 0.8rem;
  padding: 0.58rem 0.68rem;
  resize: vertical;
}

.desktop-workspace-dialog__input:focus,
.desktop-workspace-dialog__textarea:focus {
  outline: none;
  border-color: rgba(var(--desktop-accent-rgb), 0.42);
}

.desktop-workspace-dialog__colors {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.desktop-workspace-dialog__color {
  width: 1.42rem;
  height: 1.42rem;
  padding: 0;
  border: 2px solid transparent;
  border-radius: 999px;
  background: var(--workspace-color);
  cursor: pointer;
}

.desktop-workspace-dialog__color--active {
  border-color: var(--desktop-ink);
  box-shadow: 0 0 0 2px var(--desktop-surface-strong);
}

.desktop-workspace-dialog__stats {
  display: flex;
  flex-wrap: wrap;
  gap: 0.34rem;
}

.desktop-workspace-dialog__stat {
  display: inline-flex;
  align-items: center;
  min-height: 1.38rem;
  padding: 0 0.48rem;
  border: 1px solid var(--desktop-line-subtle, var(--desktop-line));
  border-radius: 999px;
  color: var(--desktop-soft);
  font-size: 0.68rem;
  font-weight: 500;
}

.desktop-workspace-dialog__stat--warning {
  border-color: rgba(217, 131, 40, 0.22);
  color: #b56a1f;
}

.desktop-workspace-dialog__danger {
  display: grid;
  gap: 0.55rem;
  margin-top: 0.2rem;
  padding: 0.85rem 0.9rem;
  border: 1px solid rgba(220, 38, 38, 0.16);
  border-radius: var(--desktop-radius-sm);
  background: rgba(220, 38, 38, 0.06);
}

.desktop-workspace-dialog__danger p {
  color: var(--desktop-soft);
}

.desktop-workspace-dialog__danger-button {
  width: fit-content;
  border: 1px solid rgba(220, 38, 38, 0.28);
  background: rgba(220, 38, 38, 0.08);
  color: #dc2626;
}

.desktop-workspace-dialog__danger-button:hover:not(:disabled) {
  background: rgba(220, 38, 38, 0.12);
}

.desktop-workspace-dialog__danger-button--confirming {
  background: rgba(220, 38, 38, 0.14);
  border-color: rgba(220, 38, 38, 0.38);
}

.desktop-workspace-dialog__footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 0.5rem;
  padding: 0.72rem 1.2rem;
  border-top: 1px solid var(--desktop-line-subtle, var(--desktop-line));
}
</style>
