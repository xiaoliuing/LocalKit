<script setup lang="ts">
import type { DocHeading } from '@/types/docs'

defineProps<{
  activeId: string
  headings: DocHeading[]
}>()

const emit = defineEmits<{
  select: [payload: { id: string; index: number }]
}>()
</script>

<template>
  <aside
    v-if="headings.length"
    class="doc-toc"
  >
    <div class="doc-toc__inner desktop-scroll">
      <p class="doc-toc__eyebrow">大纲</p>

      <button
        v-for="(heading, index) in headings"
        :key="heading.id"
        :class="[
          'doc-toc__link',
          `doc-toc__link--level-${heading.level}`,
          { 'doc-toc__link--active': heading.id === activeId },
        ]"
        type="button"
        @click="emit('select', { id: heading.id, index })"
      >
        {{ heading.text }}
      </button>
    </div>
  </aside>
</template>

<style scoped>
.doc-toc {
  height: 100%;
}

.doc-toc__inner {
  display: flex;
  flex-direction: column;
  align-items: stretch;
  justify-content: flex-start;
  gap: 0.18rem;
  height: 100%;
  overflow-y: auto;
  padding: 1rem 0.92rem 0.88rem;
}

.doc-toc__eyebrow {
  margin: 0 0 0.42rem;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  font-size: 0.68rem;
  font-weight: 700;
  color: var(--desktop-soft);
}

.doc-toc__link {
  flex: none;
  border: 0;
  padding: 0.36rem 0.22rem;
  border-radius: 0;
  border-left: 2px solid transparent;
  background: transparent;
  text-align: left;
  color: var(--desktop-muted);
  cursor: pointer;
  font-size: 0.76rem;
  line-height: 1.42;
}

.doc-toc__link--active {
  border-left-color: var(--desktop-accent);
  background: rgba(var(--desktop-accent-rgb), 0.05);
  color: var(--desktop-accent);
}

.doc-toc__link--level-3 {
  margin-left: 0.34rem;
  padding-left: 0.62rem;
  font-size: 0.72rem;
}
</style>
