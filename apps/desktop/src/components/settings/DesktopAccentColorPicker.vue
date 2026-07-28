<script setup lang="ts">
import Pickr from '@simonwep/pickr'
import '@simonwep/pickr/dist/themes/nano.min.css'
import { onBeforeUnmount, onMounted, shallowRef, useTemplateRef, watch } from 'vue'

const props = defineProps<{
  color: string
  active: boolean
}>()

const emit = defineEmits<{
  activate: []
  updateColor: [color: string]
}>()

const pickerButton = useTemplateRef<HTMLButtonElement>('pickerButton')
const picker = shallowRef<Pickr | null>(null)

onMounted(() => {
  if (!pickerButton.value) {
    return
  }

  picker.value = Pickr.create({
    el: pickerButton.value,
    container: document.body,
    theme: 'nano',
    useAsButton: true,
    default: props.color,
    closeOnScroll: true,
    position: 'bottom-start',
    defaultRepresentation: 'HEXA',
    appClass: 'desktop-accent-picker__popper',
    swatches: [
      '#1B203E',
      '#5B6FD6',
      '#1F54D9',
      '#0F8C95',
      '#1F8F63',
      '#C28A1A',
      '#C05F7F',
      '#9A68B2',
    ],
    components: {
      palette: true,
      preview: true,
      opacity: false,
      hue: true,
      interaction: {
        hex: false,
        rgba: false,
        hsla: false,
        hsva: false,
        cmyk: false,
        input: true,
        cancel: false,
        clear: false,
        save: false,
      },
    },
  })

  picker.value
    .on('init', (instance) => {
      instance.setColor(props.color, true)
    })
    .on('show', () => {
      picker.value?.setColor(props.color, true)
    })
    .on('change', (color) => {
      const nextColor = normalizeColorString(color?.toHEXA().toString() ?? props.color)
      emit('updateColor', nextColor)
    })
    .on('save', (color) => {
      if (!color) {
        return
      }

      emit('updateColor', normalizeColorString(color.toHEXA().toString()))
      picker.value?.hide()
    })
})

watch(
  () => props.color,
  (color) => {
    picker.value?.setColor(color, true)
  },
)

onBeforeUnmount(() => {
  picker.value?.destroy()
  picker.value = null
})

function normalizeColorString(value: string) {
  const trimmed = value.trim()
  if (/^#[0-9a-fA-F]{6}$/.test(trimmed)) {
    return trimmed.toUpperCase()
  }

  if (/^#[0-9a-fA-F]{8}$/.test(trimmed)) {
    return `#${trimmed.slice(1, 7)}`.toUpperCase()
  }

  return props.color
}
</script>

<template>
  <button
    ref="pickerButton"
    :class="[
      'desktop-settings-page__color',
      'desktop-accent-picker',
      { 'desktop-settings-page__color--active': props.active },
    ]"
    :style="{ '--accent-color': props.color }"
    type="button"
    @click="emit('activate')"
  >
    <span class="desktop-settings-page__color-swatch desktop-accent-picker__swatch" />
    <span class="desktop-accent-picker__name">自定义</span>
    <span class="desktop-accent-picker__value">{{ props.color }}</span>
  </button>
</template>

<style scoped>
.desktop-accent-picker {
  position: relative;
  min-height: 4.4rem;
}

.desktop-accent-picker__swatch {
  width: 1.55rem;
  height: 1.55rem;
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.2);
}

.desktop-accent-picker__name {
  color: var(--desktop-ink);
  font-size: 0.68rem;
  font-weight: 600;
}

.desktop-accent-picker__value {
  color: var(--desktop-soft);
  font-size: 0.58rem;
  font-variant-numeric: tabular-nums;
  letter-spacing: 0.02em;
  text-transform: uppercase;
}

:global(.desktop-accent-picker__popper) {
  z-index: 4000;
}

:global(.desktop-accent-picker__popper .pcr-interaction) {
  gap: 0;
  padding-top: 0.45rem;
}

:global(.desktop-accent-picker__popper .pcr-interaction > :not(.pcr-result)) {
  display: none;
}

:global(.desktop-accent-picker__popper .pcr-interaction .pcr-result) {
  width: 100%;
}
</style>
