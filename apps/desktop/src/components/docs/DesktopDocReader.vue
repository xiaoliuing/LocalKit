<script setup lang="ts">
import type { DesktopMarkdownThemeId } from '@/composables/useDesktopPreferences'
import type { DocDetail, DocMeta } from '@/types/docs'
import DesktopDocContent from './DesktopDocContent.vue'
import DesktopDocPager from './DesktopDocPager.vue'

const props = withDefaults(
  defineProps<{
    doc: DocDetail | null
    isLoading?: boolean
    highlightQuery: string
    isFavorite?: boolean
    markdownThemeId?: DesktopMarkdownThemeId
    nextDoc: DocMeta | null
    prevDoc: DocMeta | null
    restoreScrollTop?: number
    saveDoc: (absolutePath: string, markdown: string) => Promise<void>
  }>(),
  {
    isLoading: false,
    isFavorite: false,
    markdownThemeId: 'atlas',
    restoreScrollTop: 0,
  },
)

const emit = defineEmits<{
  selectDoc: [slug: string]
  scrollTopChange: [top: number]
  toggleFavorite: []
}>()
</script>

<template>
  <section
    v-if="props.isLoading"
    class="desktop-doc-reader__loading"
    aria-busy="true"
  >
    <div class="desktop-doc-reader__loading-shell">
      <article class="desktop-doc-reader__loading-article">
        <header class="desktop-doc-reader__loading-header">
          <div class="desktop-doc-reader__loading-header-copy">
            <div class="desktop-doc-reader__loading-header-row">
              <span class="desktop-doc-reader__loading-kicker" />
              <span class="desktop-doc-reader__loading-path-copy" />
            </div>
            <div class="desktop-doc-reader__loading-meta">
              <span class="desktop-doc-reader__loading-meta-label" />
              <span class="desktop-doc-reader__loading-meta-value" />
            </div>
          </div>

          <span class="desktop-doc-reader__loading-action" />
        </header>

        <div class="desktop-doc-reader__loading-body">
          <span class="desktop-doc-reader__loading-line desktop-doc-reader__loading-line--wide" />
          <span class="desktop-doc-reader__loading-line desktop-doc-reader__loading-line--mid" />
          <span class="desktop-doc-reader__loading-line desktop-doc-reader__loading-line--soft" />
          <span class="desktop-doc-reader__loading-block desktop-doc-reader__loading-block--code" />
          <span class="desktop-doc-reader__loading-line desktop-doc-reader__loading-line--wide" />
          <span class="desktop-doc-reader__loading-line" />
          <span class="desktop-doc-reader__loading-line desktop-doc-reader__loading-line--mid" />
          <span class="desktop-doc-reader__loading-block desktop-doc-reader__loading-block--paragraph" />
        </div>
      </article>

      <div class="desktop-doc-reader__loading-pager">
        <span class="desktop-doc-reader__loading-pager-card" />
        <span class="desktop-doc-reader__loading-pager-card" />
      </div>
    </div>
  </section>

  <section
    v-else-if="doc"
    class="desktop-doc-reader"
  >
    <DesktopDocContent
      :doc="doc"
      :is-favorite="props.isFavorite"
      :highlight-query="highlightQuery"
      :markdown-theme-id="props.markdownThemeId"
      :restore-scroll-top="props.restoreScrollTop"
      :save-doc="props.saveDoc"
      @scroll-top-change="emit('scrollTopChange', $event)"
      @toggle-favorite="emit('toggleFavorite')"
    />
    <DesktopDocPager
      :next-doc="nextDoc"
      :prev-doc="prevDoc"
      @select-doc="emit('selectDoc', $event)"
    />
  </section>

  <section
    v-else
    class="desktop-doc-reader__empty"
  >
    <h2>未加载到文档</h2>
    <p>请从左侧目录中选择一篇文档。</p>
  </section>
</template>

<style scoped>
.desktop-doc-reader {
  display: grid;
  grid-template-rows: minmax(0, 1fr) auto;
  gap: 0.72rem;
  min-width: 0;
  min-height: 0;
  height: 100%;
  overflow: hidden;
  padding: 0;
}

.desktop-doc-reader__empty {
  display: grid;
  gap: 0.45rem;
  align-content: start;
  padding: 1.2rem 1.2rem;
  border: 0;
  border-radius: 0;
  background: transparent;
}

.desktop-doc-reader__loading {
  min-width: 0;
  min-height: 0;
  height: 100%;
}

.desktop-doc-reader__loading-shell {
  display: grid;
  grid-template-rows: minmax(0, 1fr) auto;
  gap: 0.72rem;
  min-height: 100%;
  padding: 0;
}

.desktop-doc-reader__loading-article {
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  min-height: 0;
  overflow: hidden;
  border: 0;
  border-radius: 0;
  background: transparent;
  box-shadow: none;
}

.desktop-doc-reader__loading-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 0.85rem;
  padding: 0.72rem 1.25rem 0.68rem;
  border-bottom: 1px solid
    color-mix(in srgb, var(--desktop-line-strong) 44%, var(--desktop-line));
  background: var(--desktop-surface-strong);
}

.desktop-doc-reader__loading-header-copy {
  display: grid;
  gap: 0.28rem;
  min-width: 0;
  flex: 1 1 auto;
}

.desktop-doc-reader__loading-header-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.desktop-doc-reader__loading-meta {
  display: flex;
  align-items: center;
  gap: 0.72rem;
}

.desktop-doc-reader__loading-body,
.desktop-doc-reader__loading-pager {
  display: grid;
  gap: 0.72rem;
}

.desktop-doc-reader__loading-path-copy,
.desktop-doc-reader__loading-kicker,
.desktop-doc-reader__loading-action,
.desktop-doc-reader__loading-meta-label,
.desktop-doc-reader__loading-meta-value,
.desktop-doc-reader__loading-line,
.desktop-doc-reader__loading-block,
.desktop-doc-reader__loading-pager-card {
  display: block;
  border-radius: 999px;
  background: linear-gradient(
    90deg,
    rgba(var(--desktop-accent-rgb), 0.08),
    rgba(var(--desktop-accent-rgb), 0.16),
    rgba(var(--desktop-accent-rgb), 0.08)
  );
  background-size: 220% 100%;
  animation: desktop-doc-reader-loading 1.25s linear infinite;
}

.desktop-doc-reader__loading-kicker {
  width: 10rem;
  height: 0.7rem;
  flex: 1 1 auto;
}

.desktop-doc-reader__loading-path-copy {
  width: 1.72rem;
  height: 1.72rem;
  border-radius: 0.42rem;
  flex-shrink: 0;
}

.desktop-doc-reader__loading-action {
  width: 4.8rem;
  height: 1.62rem;
  border-radius: var(--desktop-radius-sm);
  flex-shrink: 0;
}

.desktop-doc-reader__loading-meta-label {
  width: 3.2rem;
  height: 0.68rem;
}

.desktop-doc-reader__loading-meta-value {
  width: 7.5rem;
  height: 0.68rem;
}

.desktop-doc-reader__loading-line {
  width: 100%;
  height: 0.82rem;
}

.desktop-doc-reader__loading-line--wide {
  width: 92%;
}

.desktop-doc-reader__loading-line--mid {
  width: 84%;
}

.desktop-doc-reader__loading-line--soft {
  width: 78%;
}

.desktop-doc-reader__loading-body {
  align-content: start;
  min-height: 0;
  padding: 1rem;
}

.desktop-doc-reader__loading-block {
  width: 100%;
  border-radius: 1rem;
}

.desktop-doc-reader__loading-block--code {
  height: 10rem;
}

.desktop-doc-reader__loading-block--paragraph {
  height: 8rem;
}

.desktop-doc-reader__loading-pager {
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.88rem;
}

.desktop-doc-reader__loading-pager-card {
  height: 5.6rem;
  border-radius: 1rem;
}

@keyframes desktop-doc-reader-loading {
  0% {
    background-position: 200% 0;
  }

  100% {
    background-position: -40% 0;
  }
}

.desktop-doc-reader__empty h2,
.desktop-doc-reader__empty p {
  margin: 0;
}

.desktop-doc-reader__empty p {
  color: var(--desktop-muted);
}
</style>
