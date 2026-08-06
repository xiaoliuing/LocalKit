<script setup lang="ts">
import { computed } from 'vue'
import { useDocsCatalog } from '@/composables/useDocsCatalog'

const { docs, sections } = useDocsCatalog()

const tutorialCount = computed(() =>
  docs.filter((doc) => !doc.isSectionIndex).length,
)

const sectionCards = computed(() =>
  sections.map((section) => ({
    id: section.id,
    routePath: section.routePath,
    sourceLabel: section.sourceLabel,
    title: section.title,
    count: section.docs.filter((doc) => !doc.isSectionIndex).length,
    intro: section.docs[0]?.summary ?? '该专题下暂时还没有摘要内容。',
  })),
)

const recentDocs = computed(() =>
  docs
    .filter((doc) => !doc.isSectionIndex)
    .slice(-6)
    .reverse(),
)

const startBrowseRoute = computed(() =>
  sectionCards.value[0]?.routePath
  ?? docs.find((doc) => doc.isSectionIndex)?.routePath
  ?? docs[0]?.routePath
  ?? '/',
)
</script>

<template>
  <section class="home-view">
    <header class="hero-card">
      <p class="hero-card__eyebrow">
        Docs From Local Markdown
      </p>
      <h1 class="hero-card__title">
        聚合本地 Markdown，统一阅读不同项目的设计文档。
      </h1>
      <p class="hero-card__body">
        `LocalKit` 直接读取本地文档目录，把 README 作为导读页，把后续文档整理成清晰的目录与阅读流。
      </p>
      <div class="hero-card__stats">
        <span class="hero-card__stat">{{ sections.length }} 个专题</span>
        <span class="hero-card__stat">{{ tutorialCount }} 篇教程</span>
        <span class="hero-card__stat">README 导读结构</span>
      </div>
      <div class="hero-card__actions">
        <RouterLink
          class="hero-card__primary"
          :to="startBrowseRoute"
        >
          开始浏览
        </RouterLink>
        <span class="hero-card__meta">先读导读页，再进入具体文档。</span>
      </div>
    </header>

    <section class="home-grid">
      <article class="home-panel">
        <div class="panel-heading">
          <p class="panel-heading__eyebrow">
            Sections
          </p>
          <h2 class="panel-heading__title">
            分类浏览
          </h2>
        </div>

        <div class="section-grid">
          <RouterLink
            v-for="section in sectionCards"
            :key="section.id"
            class="section-card"
            :to="section.routePath"
          >
            <span class="section-card__source">{{ section.sourceLabel }}</span>
            <span class="section-card__count">{{ section.count }} 篇教程</span>
            <strong class="section-card__title">{{ section.title }}</strong>
            <span class="section-card__intro">{{ section.intro }}</span>
          </RouterLink>
        </div>
      </article>

      <article class="home-panel">
        <div class="panel-heading">
          <p class="panel-heading__eyebrow">
            Recently Indexed
          </p>
          <h2 class="panel-heading__title">
            最近可读
          </h2>
        </div>

        <div class="recent-list">
          <RouterLink
            v-for="doc in recentDocs"
            :key="doc.slug"
            class="recent-list__item"
            :to="doc.routePath"
          >
            <span class="recent-list__section">{{ doc.sectionTitle ? `${doc.sourceLabel} / ${doc.sectionTitle}` : doc.sourceLabel }}</span>
            <strong>{{ doc.title }}</strong>
            <span class="recent-list__summary">{{ doc.summary }}</span>
          </RouterLink>
        </div>
      </article>
    </section>
  </section>
</template>

<style scoped>
.home-view {
  display: grid;
  gap: 1rem;
}

.hero-card,
.home-panel {
  padding: 1.25rem;
  border: 1px solid var(--color-line);
  border-radius: 24px;
  background: var(--surface-panel-alt);
  box-shadow: var(--shadow-panel);
}

.hero-card {
  display: grid;
  gap: 0.8rem;
  padding: 1.45rem;
}

.hero-card__eyebrow,
.panel-heading__eyebrow {
  margin: 0;
  font-size: 0.78rem;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--color-soft);
}

.hero-card__title,
.panel-heading__title {
  margin: 0;
  font-family: var(--font-display);
}

.hero-card__title {
  font-size: clamp(1.85rem, 3.4vw, 3rem);
  line-height: 1.06;
  max-width: 18ch;
}

.hero-card__body {
  margin: 0;
  max-width: 46rem;
  color: var(--color-muted);
  font-size: 0.96rem;
  line-height: 1.7;
}

.hero-card__stats {
  display: flex;
  flex-wrap: wrap;
  gap: 0.55rem;
}

.hero-card__stat {
  min-height: 34px;
  display: inline-flex;
  align-items: center;
  padding: 0.34rem 0.68rem;
  border-radius: 999px;
  background: var(--surface-chip);
  color: var(--color-ink);
  border: 1px solid var(--color-line);
  font-size: 0.84rem;
}

.hero-card__actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.8rem;
  align-items: center;
}

.hero-card__primary {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 40px;
  padding: 0.72rem 1rem;
  border-radius: 999px;
  background: var(--color-accent);
  color: var(--text-on-contrast);
  text-decoration: none;
  box-shadow: var(--shadow-emphasis);
  font-size: 0.92rem;
  font-weight: 600;
}

.hero-card__meta {
  color: var(--color-soft);
  font-size: 0.86rem;
}

.home-grid {
  display: grid;
  gap: 1rem;
}

.panel-heading {
  display: grid;
  gap: 0.22rem;
  margin-bottom: 0.85rem;
}

.panel-heading__title {
  font-size: 1.08rem;
  line-height: 1.2;
}

.section-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 0.8rem;
}

.section-card,
.recent-list__item {
  display: grid;
  gap: 0.35rem;
  min-height: 144px;
  align-content: start;
  padding: 0.9rem;
  border: 1px solid var(--color-line);
  border-radius: 18px;
  text-decoration: none;
  color: var(--color-ink);
  background: var(--surface-card);
  transition:
    transform 0.18s ease,
    background-color 0.18s ease,
    border-color 0.18s ease,
    box-shadow 0.18s ease;
}

.section-card__source,
.recent-list__section {
  color: var(--color-soft);
  font-size: 0.74rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.section-card__title,
.recent-list__item strong {
  font-size: 0.96rem;
  line-height: 1.4;
}

.section-card:hover,
.recent-list__item:hover {
  transform: translateY(-2px);
  border-color: rgba(var(--color-accent-rgb), 0.28);
  background: var(--surface-card-hover);
  box-shadow: var(--shadow-panel);
}

.section-card__count {
  color: var(--color-soft);
  font-size: 0.78rem;
}

.section-card__intro,
.recent-list__summary {
  color: var(--color-muted);
  font-size: 0.86rem;
  line-height: 1.6;
}

.recent-list {
  display: grid;
  gap: 0.75rem;
}

@media (max-width: 960px) {
  .hero-card,
  .home-panel {
    padding: 1.05rem;
    border-radius: 20px;
  }

  .hero-card {
    gap: 0.7rem;
  }

  .hero-card__title {
    max-width: 100%;
    font-size: clamp(1.6rem, 8vw, 2.35rem);
  }

  .hero-card__body {
    font-size: 0.92rem;
  }

  .section-card,
  .recent-list__item {
    min-height: auto;
  }
}

@media (max-width: 640px) {
  .home-view {
    gap: 1rem;
  }

  .hero-card,
  .home-panel {
    padding: 0.92rem;
    border-radius: 18px;
  }

  .hero-card__stats,
  .hero-card__actions {
    flex-direction: column;
    align-items: stretch;
  }

  .hero-card__primary {
    width: 100%;
  }

  .section-grid {
    grid-template-columns: 1fr;
  }
}
</style>
