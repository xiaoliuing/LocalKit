<script setup lang="ts">
import { computed } from 'vue'
import DesktopUiIcon from '@/components/ui/DesktopUiIcon.vue'
import type { DocsSourceGroup } from '@/types/docs'

const props = defineProps<{
  currentDocSlug: string | null
  currentSectionId: string | null
  currentSourceId: string | null
  depth: number
  node: DocsSourceGroup
  openBranchIds: string[]
  openSectionId: string | null
}>()

const emit = defineEmits<{
  selectDoc: [slug: string]
  toggleNode: [id: string, depth: number]
  toggleSection: [sectionId: string]
}>()

const isOpen = computed(() => props.openBranchIds[props.depth] === props.node.id)
const isSourceActive = computed(() => props.node.sourceId !== null && props.node.sourceId === props.currentSourceId)
const sectionItems = computed(() =>
  props.node.sections.map((section) => ({
    ...section,
    sectionIndexDoc: section.docs.find((doc) => doc.isSectionIndex) ?? null,
    tutorials: section.docs.filter((doc) => !doc.isSectionIndex),
  })),
)

function forwardToggleNode(id: string, depth: number) {
  emit('toggleNode', id, depth)
}

function forwardToggleSection(sectionId: string) {
  emit('toggleSection', sectionId)
}
</script>

<template>
  <section
    class="desktop-docs-sidebar-node"
    :style="{ '--desktop-sidebar-depth': depth }"
  >
    <button
      :class="[
        'desktop-docs-sidebar-node__toggle',
        {
          'desktop-docs-sidebar-node__toggle--active': isOpen || isSourceActive,
          'desktop-docs-sidebar-node__toggle--group': !node.isSource,
        },
      ]"
      type="button"
      @click="emit('toggleNode', node.id, depth)"
    >
      <span class="desktop-docs-sidebar-node__name">{{ node.name }}</span>
      <DesktopUiIcon
        name="chevron-down"
        :size="14"
        :class="[
          'desktop-docs-sidebar-node__toggle-icon',
          { 'desktop-docs-sidebar-node__toggle-icon--open': isOpen },
        ]"
      />
    </button>

    <div
      v-if="isOpen"
      class="desktop-docs-sidebar-node__body"
    >
      <template v-if="node.isSource">
        <div
          v-if="node.rootDocs.length > 0"
          class="desktop-docs-sidebar-node__docs"
        >
          <button
            v-for="doc in node.rootDocs"
            :key="doc.slug"
            :class="[
              'desktop-docs-sidebar-node__doc-link',
              { 'desktop-docs-sidebar-node__doc-link--active': doc.slug === currentDocSlug },
            ]"
            type="button"
            @click="emit('selectDoc', doc.slug)"
          >
            {{ doc.title }}
          </button>
        </div>

        <section
          v-for="section in sectionItems"
          :key="section.id"
          class="desktop-docs-sidebar-node__section"
        >
          <div
            :class="[
              'desktop-docs-sidebar-node__section-row',
              {
                'desktop-docs-sidebar-node__section-row--active':
                  currentSectionId === section.id || openSectionId === section.id,
              },
            ]"
          >
            <button
              class="desktop-docs-sidebar-node__section-title"
              type="button"
              @click="section.sectionIndexDoc && emit('selectDoc', section.sectionIndexDoc.slug)"
            >
              {{ section.title }}
            </button>

            <button
              v-if="section.tutorials.length > 0"
              class="desktop-docs-sidebar-node__section-toggle"
              type="button"
              @click="emit('toggleSection', section.id)"
            >
              <DesktopUiIcon
                name="chevron-down"
                :size="14"
                :class="[
                  'desktop-docs-sidebar-node__section-toggle-icon',
                  { 'desktop-docs-sidebar-node__section-toggle-icon--open': openSectionId === section.id },
                ]"
              />
            </button>
          </div>

          <div
            v-if="openSectionId === section.id && section.tutorials.length > 0"
            class="desktop-docs-sidebar-node__docs"
          >
            <button
              v-for="doc in section.tutorials"
              :key="doc.slug"
              :class="[
                'desktop-docs-sidebar-node__doc-link',
                { 'desktop-docs-sidebar-node__doc-link--active': doc.slug === currentDocSlug },
              ]"
              type="button"
              @click="emit('selectDoc', doc.slug)"
            >
              {{ doc.title }}
            </button>
          </div>
        </section>
      </template>

      <template v-else>
        <DesktopDocsSidebarNode
          v-for="child in node.children"
          :key="child.id"
          :current-doc-slug="currentDocSlug"
          :current-section-id="currentSectionId"
          :current-source-id="currentSourceId"
          :depth="depth + 1"
          :node="child"
          :open-branch-ids="openBranchIds"
          :open-section-id="openSectionId"
          @select-doc="emit('selectDoc', $event)"
          @toggle-node="forwardToggleNode"
          @toggle-section="forwardToggleSection"
        />
      </template>
    </div>
  </section>
</template>

<style scoped>
.desktop-docs-sidebar-node {
  display: grid;
  gap: 0.18rem;
}

.desktop-docs-sidebar-node__toggle,
.desktop-docs-sidebar-node__section-row,
.desktop-docs-sidebar-node__doc-link {
  transition: background-color 0.18s ease, border-color 0.18s ease, color 0.18s ease;
}

.desktop-docs-sidebar-node__toggle,
.desktop-docs-sidebar-node__section-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  width: 100%;
  min-height: 28px;
  padding: 0.24rem 0.38rem;
  border: 0;
  border-left: 2px solid transparent;
  border-radius: 0;
  background: transparent;
  color: var(--desktop-muted);
  font-size: 0.78rem;
}

.desktop-docs-sidebar-node__toggle {
  padding-inline-start: calc(0.44rem + var(--desktop-sidebar-depth, 0) * 0.3rem);
}

.desktop-docs-sidebar-node__toggle--group {
  color: var(--desktop-ink);
}

.desktop-docs-sidebar-node__toggle--active,
.desktop-docs-sidebar-node__section-row--active {
  background: rgba(var(--desktop-accent-rgb), 0.08);
  color: var(--desktop-accent);
  border-radius: 0.2rem;
}

.desktop-docs-sidebar-node__name {
  min-width: 0;
  font-weight: 640;
  text-align: left;
  overflow-wrap: anywhere;
}

.desktop-docs-sidebar-node__toggle-icon,
.desktop-docs-sidebar-node__section-toggle-icon {
  display: block;
  transform: rotate(-90deg);
  transform-origin: center;
  transition: transform 0.18s ease, opacity 0.18s ease;
  flex: none;
  opacity: 0.88;
}

.desktop-docs-sidebar-node__toggle-icon--open,
.desktop-docs-sidebar-node__section-toggle-icon--open {
  transform: rotate(0deg);
}

.desktop-docs-sidebar-node__body {
  display: grid;
  gap: 0.26rem;
  margin-top: 0.08rem;
  margin-left: 0.22rem;
  padding-left: 0.52rem;
  border-left: 1px solid rgba(var(--desktop-accent-rgb), 0.12);
}

.desktop-docs-sidebar-node__section {
  display: grid;
  gap: 0.14rem;
  padding-left: 0.08rem;
}

.desktop-docs-sidebar-node__section-title {
  border: 0;
  background: transparent;
  min-width: 0;
  color: inherit;
  text-align: left;
  font-size: 0.77rem;
  font-weight: 620;
  cursor: pointer;
}

.desktop-docs-sidebar-node__section-toggle {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  padding: 0;
  border: 0;
  border-radius: 5px;
  background: rgba(var(--desktop-accent-rgb), 0.05);
  color: var(--desktop-accent);
  cursor: pointer;
  line-height: 0;
  flex: none;
}

.desktop-docs-sidebar-node__docs {
  display: grid;
  gap: 0.08rem;
}

.desktop-docs-sidebar-node__doc-link {
  display: block;
  width: 100%;
  padding: 0.28rem 0.42rem;
  border: 0;
  border-left: 2px solid transparent;
  border-radius: 0;
  background: transparent;
  color: var(--desktop-muted);
  text-align: left;
  cursor: pointer;
  font-size: 0.76rem;
  line-height: 1.4;
}

.desktop-docs-sidebar-node__doc-link:hover {
  color: var(--desktop-ink);
  background: rgba(0, 0, 0, 0.04);
}

.desktop-docs-sidebar-node__doc-link--active {
  color: var(--desktop-accent);
  border-left-color: var(--desktop-accent);
  background: rgba(var(--desktop-accent-rgb), 0.08);
  font-weight: 600;
  border-radius: 0.2rem;
}
</style>
