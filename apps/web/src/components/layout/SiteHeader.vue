<script setup lang="ts">
  import {
    computed,
    nextTick,
    onMounted,
    onUnmounted,
    shallowRef,
    useTemplateRef,
    watch,
  } from "vue";
  import DocsSearchPanel from "@/components/docs/DocsSearchPanel.vue";
  import type {
    ResolvedTheme,
    ThemeAccent,
    ThemeAccentId,
    ThemeMode,
  } from "@/composables/useTheme";
  import type { SearchResult } from "@/types/docs";

  const query = defineModel<string>({ default: "" });

  const props = defineProps<{
    isSearchOpen: boolean;
    pageLabel: string;
    resolvedTheme: ResolvedTheme;
    results: SearchResult[];
    selectedIndex: number;
    themeAccent: ThemeAccentId;
    themeAccents: ThemeAccent[];
    themeMode: ThemeMode;
    totalDocs: number;
    totalSections: number;
  }>();

  const emit = defineEmits<{
    closeSearch: [];
    moveSelection: [direction: 1 | -1];
    openSearch: [];
    selectThemeAccent: [accent: ThemeAccentId];
    selectThemeMode: [mode: ThemeMode];
    submitSearch: [routePath?: string];
    toggleSidebar: [];
  }>();

  const searchPanelRef =
    useTemplateRef<InstanceType<typeof DocsSearchPanel>>("searchPanel");
  const themeMenuRef = useTemplateRef<HTMLElement>("themeMenu");

  const isMobileSearchExpanded = shallowRef(false);
  const isThemeMenuOpen = shallowRef(false);

  const shouldShowMobileSearch = computed(() => isMobileSearchExpanded.value);
  const activeAccentLabel = computed(
    () =>
      props.themeAccents.find((accent) => accent.id === props.themeAccent)
        ?.label ?? "星图蓝",
  );

  const themeModeOptions: Array<{ id: ThemeMode; label: string }> = [
    { id: "system", label: "系统" },
    { id: "light", label: "浅色" },
    { id: "dark", label: "暗色" },
  ];

  watch(
    [() => query.value, () => props.isSearchOpen],
    ([currentQuery, searchOpen]) => {
      if (!currentQuery.trim() && !searchOpen) {
        isMobileSearchExpanded.value = false;
      }
    },
  );

  onMounted(() => {
    document.addEventListener("pointerdown", onPointerDown);
  });

  onUnmounted(() => {
    document.removeEventListener("pointerdown", onPointerDown);
  });

  function onPointerDown(event: PointerEvent) {
    const target = event.target;

    if (!(target instanceof Node)) {
      return;
    }

    if (!themeMenuRef.value?.contains(target)) {
      isThemeMenuOpen.value = false;
    }
  }

  function closeSearch() {
    emit("closeSearch");

    if (!query.value.trim()) {
      isMobileSearchExpanded.value = false;
    }
  }

  function openMobileSearch() {
    isMobileSearchExpanded.value = true;
    emit("openSearch");

    void nextTick(() => {
      searchPanelRef.value?.focusInput();
    });
  }

  function toggleMobileSearch() {
    if (isMobileSearchExpanded.value) {
      isMobileSearchExpanded.value = false;
      emit("closeSearch");
      return;
    }

    openMobileSearch();
  }

  function toggleThemeMenu() {
    isThemeMenuOpen.value = !isThemeMenuOpen.value;
  }

  function selectThemeMode(mode: ThemeMode) {
    emit("selectThemeMode", mode);
    isThemeMenuOpen.value = false;
  }

  function selectThemeAccent(accent: ThemeAccentId) {
    emit("selectThemeAccent", accent);
    isThemeMenuOpen.value = false;
  }
</script>

<template>
  <header class="site-header">
    <div class="site-header__inner">
      <div class="site-header__toolbar">
        <div class="site-header__brand">
          <button
            aria-label="打开目录"
            class="site-header__menu"
            type="button"
            @click="emit('toggleSidebar')"
          >
            目录
          </button>

          <RouterLink class="site-header__title" to="/">
            <span class="site-header__title-mark" />
            <span class="site-header__title-text">LocalKit</span>
          </RouterLink>

          <p class="site-header__subtitle">
            {{ props.pageLabel }}
          </p>
        </div>

        <DocsSearchPanel
          :class="[
            'site-header__search',
            { 'site-header__search--collapsed': !shouldShowMobileSearch },
          ]"
          ref="searchPanel"
          :is-open="props.isSearchOpen"
          :results="props.results"
          :selected-index="props.selectedIndex"
          v-model="query"
          @close="closeSearch"
          @move-selection="emit('moveSelection', $event)"
          @open="emit('openSearch')"
          @submit="emit('submitSearch', $event)"
        />

        <div ref="themeMenu" class="site-header__controls">
          <a
            aria-label="打开 LocalKit GitHub 仓库"
            class="site-header__github-link"
            href="https://github.com/xiaoliuing/docs-atlas"
            rel="noreferrer"
            target="_blank"
          >
            <svg
              aria-hidden="true"
              class="site-header__github-icon"
              viewBox="0 0 24 24"
            >
              <path
                fill="currentColor"
                d="M12 2C6.48 2 2 6.59 2 12.24c0 4.52 2.87 8.36 6.84 9.71.5.1.68-.22.68-.49 0-.24-.01-1.04-.01-1.89-2.78.62-3.37-1.21-3.37-1.21-.46-1.18-1.11-1.49-1.11-1.49-.91-.64.07-.63.07-.63 1 .07 1.53 1.05 1.53 1.05.9 1.57 2.35 1.12 2.92.86.09-.67.35-1.12.63-1.38-2.22-.26-4.56-1.14-4.56-5.05 0-1.11.39-2.01 1.03-2.72-.1-.26-.45-1.31.1-2.74 0 0 .84-.28 2.75 1.04A9.32 9.32 0 0 1 12 6.84c.85 0 1.71.12 2.51.36 1.91-1.32 2.75-1.04 2.75-1.04.55 1.43.2 2.48.1 2.74.64.71 1.03 1.61 1.03 2.72 0 3.92-2.34 4.79-4.57 5.04.36.32.68.95.68 1.92 0 1.39-.01 2.5-.01 2.84 0 .27.18.59.69.49A10.25 10.25 0 0 0 22 12.24C22 6.59 17.52 2 12 2Z"
              />
            </svg>
          </a>

          <button
            :aria-label="shouldShowMobileSearch ? '收起搜索' : '打开搜索'"
            class="site-header__search-trigger"
            type="button"
            @click="toggleMobileSearch"
          >
            <svg
              v-if="!shouldShowMobileSearch"
              aria-hidden="true"
              class="site-header__search-icon"
              fill="none"
              viewBox="0 0 24 24"
            >
              <circle
                cx="11"
                cy="11"
                r="5.75"
                stroke="currentColor"
                stroke-linecap="round"
                stroke-width="1.85"
              />
              <path
                d="M15.35 15.35L19 19"
                stroke="currentColor"
                stroke-linecap="round"
                stroke-width="1.85"
              />
            </svg>
            <svg
              v-else
              aria-hidden="true"
              class="site-header__search-icon"
              fill="none"
              viewBox="0 0 24 24"
            >
              <path
                d="M7 7L17 17"
                stroke="currentColor"
                stroke-linecap="round"
                stroke-width="1.85"
              />
              <path
                d="M17 7L7 17"
                stroke="currentColor"
                stroke-linecap="round"
                stroke-width="1.85"
              />
            </svg>
          </button>

          <button
            :aria-expanded="isThemeMenuOpen"
            aria-label="打开主题设置"
            class="site-header__theme-trigger"
            type="button"
            @click="toggleThemeMenu"
          >
            <span
              :class="[
                'site-header__theme-mark',
                {
                  'site-header__theme-mark--dark':
                    props.resolvedTheme === 'dark',
                },
              ]"
            />
            <span class="site-header__theme-label">{{
              activeAccentLabel
            }}</span>
          </button>

          <div v-if="isThemeMenuOpen" class="site-header__theme-popover">
            <div class="site-header__theme-section">
              <p class="site-header__theme-heading">外观</p>
              <div class="site-header__mode-list">
                <button
                  v-for="option in themeModeOptions"
                  :key="option.id"
                  :class="[
                    'site-header__mode-button',
                    {
                      'site-header__mode-button--active':
                        option.id === props.themeMode,
                    },
                  ]"
                  type="button"
                  @click="selectThemeMode(option.id)"
                >
                  {{ option.label }}
                </button>
              </div>
            </div>

            <div class="site-header__theme-section">
              <div class="site-header__theme-heading-row">
                <p class="site-header__theme-heading">主题色</p>
                <span class="site-header__theme-current">{{
                  activeAccentLabel
                }}</span>
              </div>
              <div class="site-header__palette-grid">
                <button
                  v-for="accent in props.themeAccents"
                  :key="accent.id"
                  :aria-label="`切换到${accent.label}`"
                  :class="[
                    'site-header__palette-option',
                    {
                      'site-header__palette-option--active':
                        accent.id === props.themeAccent,
                    },
                  ]"
                  :style="{ '--theme-swatch': accent.color }"
                  type="button"
                  @click="selectThemeAccent(accent.id)"
                >
                  <span class="site-header__palette-swatch" />
                  <span class="site-header__palette-name">{{
                    accent.label
                  }}</span>
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </header>
</template>

<style scoped>
  .site-header {
    position: sticky;
    top: 0.65rem;
    z-index: 40;
    padding: 0 1rem;
  }

  .site-header__inner {
    padding: 0.9rem 1.1rem;
    margin: 0 auto;
    max-width: 1560px;
    border: 1px solid var(--color-line);
    border-radius: 24px;
    backdrop-filter: blur(22px);
    background: var(--surface-panel-alt);
    box-shadow: var(--shadow-panel);
  }

  .site-header__toolbar {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(280px, 420px) auto;
    gap: 0.9rem;
    align-items: center;
  }

  .site-header__brand {
    display: flex;
    align-items: center;
    gap: 0.85rem;
    min-width: 0;
  }

  .site-header__menu {
    display: none;
    border: 0;
    border-radius: 999px;
    min-height: 44px;
    padding: 0.65rem 0.95rem;
    background: rgba(var(--color-accent-rgb), 0.12);
    color: var(--color-ink);
    font: inherit;
    cursor: pointer;
  }

  .site-header__title {
    display: inline-flex;
    align-items: center;
    gap: 0.7rem;
    min-width: 0;
    color: var(--color-ink);
    font-family: var(--font-display);
    font-size: 1.35rem;
    font-weight: 700;
    text-decoration: none;
    letter-spacing: 0.02em;
  }

  .site-header__title-mark {
    width: 0.88rem;
    height: 0.88rem;
    border-radius: 999px;
    background: radial-gradient(
      circle at 30% 30%,
      #ffffff,
      var(--color-accent) 58%,
      var(--color-accent-deep) 100%
    );
    box-shadow: 0 0 0 6px rgba(var(--color-accent-rgb), 0.12);
  }

  .site-header__title-text,
  .site-header__subtitle {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .site-header__subtitle {
    margin: 0;
    padding-left: 0.85rem;
    border-left: 1px solid var(--color-line);
    color: var(--color-muted);
  }

  .site-header__search {
    min-width: 0;
  }

  .site-header__controls {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 0.55rem;
  }

  .site-header__search-trigger,
  .site-header__github-link,
  .site-header__theme-trigger {
    min-width: 42px;
    min-height: 42px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 0.55rem;
    padding: 0.45rem 0.8rem;
    border: 1px solid var(--color-line);
    border-radius: 999px;
    background: var(--surface-chip);
    color: var(--color-ink);
    cursor: pointer;
    text-decoration: none;
    transition:
      background-color 0.18s ease,
      border-color 0.18s ease,
      transform 0.18s ease;
  }

  .site-header__search-trigger {
    display: none;
    padding: 0;
  }

  .site-header__search-trigger:hover,
  .site-header__github-link:hover,
  .site-header__theme-trigger:hover,
  .site-header__mode-button:hover,
  .site-header__palette-option:hover {
    background: rgba(var(--color-accent-rgb), 0.1);
    border-color: rgba(var(--color-accent-rgb), 0.28);
    transform: translateY(-1px);
  }

  .site-header__search-icon {
    width: 1.05rem;
    height: 1.05rem;
  }

  .site-header__github-link {
    padding: 0;
  }

  .site-header__github-icon {
    width: 1.08rem;
    height: 1.08rem;
  }

  .site-header__theme-mark {
    width: 0.82rem;
    height: 0.82rem;
    border-radius: 999px;
    background: linear-gradient(135deg, #fff4ba, #f2c145);
    box-shadow: 0 0 0 4px rgba(var(--color-accent-rgb), 0.14);
  }

  .site-header__theme-mark--dark {
    background: linear-gradient(135deg, #d7e4ff, #6d82c9 65%, #293969 100%);
  }

  .site-header__theme-label {
    max-width: 6rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--color-muted);
    font-size: 0.9rem;
  }

  .site-header__theme-popover {
    position: absolute;
    top: calc(100% + 0.7rem);
    right: 0;
    width: min(21rem, calc(100vw - 2rem));
    display: grid;
    gap: 0.9rem;
    padding: 0.95rem;
    border: 1px solid var(--color-line);
    border-radius: 22px;
    background: var(--surface-panel);
    box-shadow: var(--shadow-panel);
  }

  .site-header__theme-section {
    display: grid;
    gap: 0.65rem;
  }

  .site-header__theme-heading,
  .site-header__theme-current {
    margin: 0;
    font-size: 0.8rem;
    color: var(--color-soft);
  }

  .site-header__theme-heading {
    letter-spacing: 0.12em;
    text-transform: uppercase;
  }

  .site-header__theme-heading-row {
    display: flex;
    justify-content: space-between;
    gap: 0.6rem;
    align-items: center;
  }

  .site-header__mode-list {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 0.45rem;
  }

  .site-header__mode-button,
  .site-header__palette-option {
    border: 1px solid var(--color-line);
    border-radius: 16px;
    background: var(--surface-card);
    color: var(--color-ink);
    cursor: pointer;
    transition:
      background-color 0.18s ease,
      border-color 0.18s ease,
      transform 0.18s ease;
  }

  .site-header__mode-button {
    min-height: 40px;
    padding: 0.65rem 0.7rem;
  }

  .site-header__mode-button--active,
  .site-header__palette-option--active {
    border-color: rgba(var(--color-accent-rgb), 0.38);
    background: rgba(var(--color-accent-rgb), 0.12);
  }

  .site-header__palette-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 0.45rem;
  }

  .site-header__palette-option {
    display: flex;
    align-items: center;
    gap: 0.7rem;
    min-height: 44px;
    padding: 0.65rem 0.75rem;
    text-align: left;
  }

  .site-header__palette-swatch {
    width: 1rem;
    height: 1rem;
    flex: 0 0 auto;
    border-radius: 999px;
    background: var(--theme-swatch);
    box-shadow:
      inset 0 0 0 1px rgba(255, 255, 255, 0.56),
      0 0 0 1px rgba(9, 18, 40, 0.08);
  }

  .site-header__palette-name {
    color: var(--color-muted);
  }

  @media (max-width: 1180px) {
    .site-header__toolbar {
      grid-template-columns: minmax(0, 1fr) minmax(240px, 360px) auto;
    }
  }

  @media (max-width: 960px) {
    .site-header {
      top: 0;
      padding: 0;
    }

    .site-header__inner {
      padding: 0.75rem 0.8rem 0.85rem;
      max-width: none;
      border-radius: 0;
      border-top: 0;
      border-right: 0;
      border-left: 0;
      box-shadow: 0 10px 24px rgba(var(--color-accent-rgb), 0.12);
    }

    .site-header__toolbar {
      grid-template-columns: minmax(0, 1fr) auto;
      gap: 0.7rem;
    }

    .site-header__menu {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      min-width: 44px;
      padding: 0.65rem 0.85rem;
    }

    .site-header__subtitle,
    .site-header__search--collapsed {
      display: none;
    }

    .site-header__search {
      grid-column: 1 / -1;
      order: 3;
    }

    .site-header__search-trigger {
      display: inline-flex;
    }
  }

  @media (max-width: 640px) {
    .site-header__inner {
      padding: 0.7rem;
      border-radius: 0 0 16px 16px;
    }

    .site-header__brand {
      gap: 0.6rem;
    }

    .site-header__title {
      font-size: 1.06rem;
    }

    .site-header__theme-label {
      display: none;
    }

    .site-header__theme-trigger {
      padding-inline: 0;
    }
  }

  @media (max-width: 520px) {
    .site-header__inner {
      padding: 0.65rem 0.7rem 0.8rem;
    }

    .site-header__toolbar {
      gap: 0.55rem;
    }

    .site-header__menu,
    .site-header__github-link,
    .site-header__search-trigger,
    .site-header__theme-trigger {
      min-width: 40px;
      min-height: 40px;
    }

    .site-header__title {
      font-size: 0.98rem;
    }

    .site-header__theme-popover {
      width: min(19rem, calc(100vw - 1rem));
      right: -0.15rem;
      padding: 0.8rem;
    }

    .site-header__palette-option {
      min-height: 42px;
    }
  }
</style>
