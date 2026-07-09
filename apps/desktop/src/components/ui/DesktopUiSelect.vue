<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, shallowRef, useTemplateRef, watch } from 'vue'
import DesktopUiIcon from '@/components/ui/DesktopUiIcon.vue'

export type DesktopUiSelectOption = {
  value: string
  label: string
  disabled?: boolean
}

const props = withDefaults(
  defineProps<{
    options: DesktopUiSelectOption[]
    disabled?: boolean
    size?: 'sm' | 'md'
    placeholder?: string
  }>(),
  {
    disabled: false,
    size: 'md',
    placeholder: '请选择',
  },
)

const model = defineModel<string>({ default: '' })

const isOpen = shallowRef(false)
const rootRef = useTemplateRef<HTMLElement>('root')
const triggerRef = useTemplateRef<HTMLButtonElement>('trigger')
const menuRef = useTemplateRef<HTMLElement>('menu')
const menuStyle = shallowRef<Record<string, string>>({})

const selectedLabel = computed(() => {
  const match = props.options.find((option) => option.value === model.value)
  return match?.label ?? props.placeholder
})

function updateMenuPosition() {
  const trigger = triggerRef.value
  if (!trigger) {
    return
  }

  const rect = trigger.getBoundingClientRect()
  menuStyle.value = {
    top: `${rect.bottom + 6}px`,
    left: `${rect.left}px`,
    width: `${rect.width}px`,
  }
}

function toggleOpen() {
  if (props.disabled) {
    return
  }

  isOpen.value = !isOpen.value
}

function closeMenu() {
  isOpen.value = false
}

function selectOption(value: string) {
  model.value = value
  closeMenu()
}

function onDocumentPointerDown(event: PointerEvent) {
  if (!isOpen.value) {
    return
  }

  const target = event.target
  if (!(target instanceof Node)) {
    closeMenu()
    return
  }

  const root = rootRef.value
  const menu = menuRef.value
  if (root?.contains(target) || menu?.contains(target)) {
    return
  }

  closeMenu()
}

function onDocumentKeydown(event: KeyboardEvent) {
  if (!isOpen.value) {
    return
  }

  if (event.key === 'Escape') {
    closeMenu()
  }
}

function onWindowChange() {
  if (isOpen.value) {
    closeMenu()
  }
}

watch(isOpen, async (open) => {
  if (!open) {
    return
  }

  await nextTick()
  updateMenuPosition()
})

onMounted(() => {
  document.addEventListener('pointerdown', onDocumentPointerDown, true)
  document.addEventListener('keydown', onDocumentKeydown)
  window.addEventListener('resize', onWindowChange)
  window.addEventListener('scroll', onWindowChange, true)
})

onUnmounted(() => {
  document.removeEventListener('pointerdown', onDocumentPointerDown, true)
  document.removeEventListener('keydown', onDocumentKeydown)
  window.removeEventListener('resize', onWindowChange)
  window.removeEventListener('scroll', onWindowChange, true)
})
</script>

<template>
  <div
    ref="root"
    :class="[
      'desktop-select',
      `desktop-select--${size}`,
      {
        'desktop-select--open': isOpen,
        'desktop-select--disabled': disabled,
      },
    ]"
  >
    <button
      ref="trigger"
      class="desktop-select__trigger"
      type="button"
      :disabled="disabled"
      :aria-expanded="isOpen"
      aria-haspopup="listbox"
      @mousedown.stop
      @click.stop="toggleOpen"
    >
      <span class="desktop-select__value">{{ selectedLabel }}</span>
      <DesktopUiIcon
        class="desktop-select__chevron"
        name="chevron-down"
        :size="size === 'sm' ? 12 : 14"
      />
    </button>

    <Teleport to="body">
      <div
        v-if="isOpen"
        ref="menu"
        class="desktop-select__menu desktop-scroll"
        :class="`desktop-select__menu--${size}`"
        :style="menuStyle"
        role="listbox"
        @mousedown.stop
        @click.stop
      >
        <button
          v-for="option in options"
          :key="option.value"
          :class="[
            'desktop-select__option',
            { 'desktop-select__option--active': model === option.value },
          ]"
          type="button"
          role="option"
          :aria-selected="model === option.value"
          :disabled="option.disabled"
          @click="selectOption(option.value)"
        >
          {{ option.label }}
        </button>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.desktop-select {
  position: relative;
  width: 100%;
  min-width: 0;
}

.desktop-select__trigger {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.55rem;
  width: 100%;
  min-width: 0;
  height: 2rem;
  padding: 0 0.58rem 0 0.72rem;
  border: 1px solid var(--desktop-line);
  border-radius: var(--desktop-radius-sm);
  background: var(--desktop-surface-strong);
  color: var(--desktop-ink);
  font: inherit;
  font-size: 0.76rem;
  line-height: 1.4;
  text-align: left;
  cursor: pointer;
  transition:
    border-color 0.15s ease,
    background-color 0.15s ease,
    box-shadow 0.15s ease;
}

.desktop-select--sm .desktop-select__trigger {
  height: 1.75rem;
  padding: 0 0.45rem 0 0.55rem;
  font-size: 0.69rem;
}

.desktop-select__trigger:hover:not(:disabled) {
  border-color: var(--desktop-line-strong);
  background: var(--desktop-field-bg);
}

.desktop-select--open .desktop-select__trigger,
.desktop-select__trigger:focus-visible {
  border-color: var(--desktop-accent);
  box-shadow: var(--desktop-focus-ring);
  outline: none;
}

.desktop-select--disabled .desktop-select__trigger {
  opacity: 0.62;
  cursor: not-allowed;
}

.desktop-select__value {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.desktop-select__chevron {
  flex: none;
  color: var(--desktop-soft);
  transition: transform 0.15s ease;
}

.desktop-select--open .desktop-select__chevron {
  transform: rotate(180deg);
}
</style>

<style>
.desktop-select__menu {
  position: fixed;
  z-index: 80;
  display: grid;
  gap: 0.12rem;
  max-height: 14rem;
  padding: 0.28rem;
  border: 1px solid var(--desktop-line);
  border-radius: var(--desktop-radius-sm);
  background: var(--desktop-surface-strong);
  box-shadow: 0 10px 28px rgba(var(--desktop-shadow), 0.16);
  overflow-y: auto;
}

.desktop-select__option {
  display: block;
  width: 100%;
  padding: 0.48rem 0.55rem;
  border: 0;
  border-radius: calc(var(--desktop-radius-sm) - 2px);
  background: transparent;
  color: var(--desktop-ink);
  font: inherit;
  font-size: 0.74rem;
  line-height: 1.4;
  text-align: left;
  cursor: pointer;
  transition: background-color 0.12s ease, color 0.12s ease;
}

.desktop-select__menu--sm .desktop-select__option {
  font-size: 0.69rem;
}

.desktop-select__option:hover:not(:disabled) {
  background: rgba(var(--desktop-accent-rgb), 0.08);
}

.desktop-select__option--active {
  background: rgba(var(--desktop-accent-rgb), 0.12);
  color: var(--desktop-accent);
  font-weight: 600;
}

.desktop-select__option:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}
</style>
