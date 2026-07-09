<script setup lang="ts">
export type DesktopSettingsSection = 'appearance' | 'updates' | 'data'

const props = defineProps<{
  activeSection: DesktopSettingsSection
}>()

const emit = defineEmits<{
  select: [section: DesktopSettingsSection]
}>()

const items: Array<{
  id: DesktopSettingsSection
  title: string
}> = [
  { id: 'appearance', title: '外观' },
  { id: 'updates', title: '更新' },
  { id: 'data', title: '数据与日志' },
]
</script>

<template>
  <nav class="desktop-settings-nav" aria-label="设置分组">
    <button
      v-for="item in items"
      :key="item.id"
      :class="[
        'desktop-settings-nav__item',
        { 'desktop-settings-nav__item--active': props.activeSection === item.id },
      ]"
      type="button"
      @click="emit('select', item.id)"
    >
      {{ item.title }}
    </button>
  </nav>
</template>

<style scoped>
.desktop-settings-nav {
  display: grid;
  align-content: start;
  gap: 0;
}

.desktop-settings-nav__item {
  width: 100%;
  padding: 0.5rem 1rem;
  border: 0;
  border-left: 2px solid transparent;
  background: transparent;
  color: var(--desktop-muted);
  font-size: 0.8rem;
  font-weight: 500;
  text-align: left;
  cursor: pointer;
  transition: background-color 0.12s ease, color 0.12s ease, border-color 0.12s ease;
}

.desktop-settings-nav__item:hover {
  background: rgba(0, 0, 0, 0.04);
  color: var(--desktop-ink);
}

.desktop-settings-nav__item--active {
  border-left-color: var(--desktop-accent);
  background: rgba(var(--desktop-accent-rgb), 0.08);
  color: var(--desktop-accent);
  font-weight: 600;
}
</style>
