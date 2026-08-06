<script setup lang="ts">
withDefaults(
  defineProps<{
    name:
      | 'atlas'
      | 'search'
      | 'settings'
      | 'grip'
      | 'plus'
      | 'close'
      | 'chevron-down'
      | 'chevron-left'
      | 'chevron-right'
      | 'bookmark'
      | 'zoom-in'
      | 'zoom-out'
      | 'rotate-left'
      | 'rotate-right'
      | 'reset-view'
      | 'history'
      | 'file'
      | 'copy'
      | 'folder'
      | 'reader'
      | 'recent'
      | 'tools'
      | 'video'
      | 'audio'
      | 'file-preview'
      | 'agent-sessions'
      | 'lightbulb'
    size?: number | string
    strokeWidth?: number
  }>(),
  {
    size: 20,
    strokeWidth: 1.8,
  },
)
</script>

<template>
  <svg
    class="desktop-ui-icon"
    :style="{ width: typeof size === 'number' ? `${size}px` : size, height: typeof size === 'number' ? `${size}px` : size }"
    :viewBox="
      name === 'settings' ||
      name === 'folder' ||
      name === 'video' ||
      name === 'file-preview' ||
      name === 'agent-sessions'
        ? '0 0 1024 1024'
        : '0 0 24 24'
    "
    fill="none"
    aria-hidden="true"
  >
    <template v-if="name === 'atlas'">
      <path
        d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"
        :stroke-width="strokeWidth"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
      <path
        d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"
        :stroke-width="strokeWidth"
        stroke-linejoin="round"
      />
    </template>

    <template v-else-if="name === 'reader'">
      <path d="M12 7v14" :stroke-width="strokeWidth" stroke-linecap="round" stroke-linejoin="round" />
      <path
        d="M3 18a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1h5a4 4 0 0 1 4 4 4 4 0 0 1 4-4h5a1 1 0 0 1 1 1v13a1 1 0 0 1-1 1h-6a3 3 0 0 0-3 3 3 3 0 0 0-3-3z"
        :stroke-width="strokeWidth"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </template>

    <template v-else-if="name === 'search'">
      <circle cx="11" cy="11" r="6.5" :stroke-width="strokeWidth" />
      <path d="M16 16L20.5 20.5" :stroke-width="strokeWidth" stroke-linecap="round" />
    </template>

    <template v-else-if="name === 'settings'">
      <path
        d="M581.818182 930.909091h-139.636364c-44.218182 0-81.454545-37.236364-81.454545-81.454546v-55.854545c0-6.981818-4.654545-9.309091-6.981818-9.309091-2.327273-2.327273-6.981818-2.327273-11.636364 0l-48.872727 27.927273c-39.563636 23.272727-88.436364 9.309091-111.709091-30.254546l-69.818182-121.018181c-11.636364-18.618182-13.963636-41.890909-6.981818-62.836364s18.618182-39.563636 37.236363-48.872727l48.872728-27.927273c4.654545-2.327273 6.981818-6.981818 6.981818-9.309091 0-2.327273 0-6.981818-6.981818-9.309091l-48.872728-27.927273c-18.618182-11.636364-32.581818-27.927273-37.236363-48.872727-4.654545-20.945455-2.327273-41.890909 6.981818-62.836364l69.818182-121.018181c23.272727-39.563636 72.145455-51.2 111.709091-30.254546l48.872727 27.927273c4.654545 2.327273 9.309091 2.327273 11.636364 0 2.327273-2.327273 6.981818-4.654545 6.981818-9.309091V174.545455C360.727273 130.327273 397.963636 93.090909 442.181818 93.090909h139.636364c44.218182 0 81.454545 37.236364 81.454545 81.454546v55.854545c0 6.981818 4.654545 9.309091 6.981818 9.309091 2.327273 2.327273 6.981818 2.327273 11.636364 0l48.872727-27.927273c39.563636-23.272727 88.436364-9.309091 111.709091 30.254546l69.818182 121.018181c23.272727 39.563636 9.309091 88.436364-30.254545 111.709091l-48.872728 27.927273c-4.654545 2.327273-6.981818 6.981818-6.981818 9.309091 0 2.327273 0 6.981818 6.981818 9.309091l48.872728 27.927273c39.563636 23.272727 51.2 72.145455 30.254545 111.709091l-69.818182 121.018181c-23.272727 39.563636-72.145455 51.2-111.709091 30.254546l-48.872727-27.927273c-4.654545-2.327273-9.309091-2.327273-11.636364 0-2.327273 2.327273-6.981818 4.654545-6.981818 9.309091v55.854545c0 44.218182-37.236364 81.454545-81.454545 81.454546z m-232.727273-218.763636c13.963636 0 27.927273 4.654545 39.563636 11.636363 25.6 13.963636 41.890909 41.890909 41.89091 69.818182v55.854545c0 6.981818 4.654545 11.636364 11.636363 11.636364h139.636364c6.981818 0 11.636364-4.654545 11.636363-11.636364v-55.854545c0-30.254545 16.290909-55.854545 41.89091-69.818182 25.6-13.963636 55.854545-13.963636 81.454545 0l48.872727 27.927273c4.654545 2.327273 11.636364 2.327273 16.290909-4.654546l69.818182-121.018181c2.327273-4.654545 2.327273-11.636364-4.654545-16.290909L798.254545 581.818182c-25.6-13.963636-41.890909-41.890909-41.890909-69.818182 0-30.254545 16.290909-55.854545 41.890909-69.818182l48.872728-27.927273c4.654545-2.327273 6.981818-9.309091 4.654545-16.290909l-69.818182-121.018181c-2.327273-4.654545-9.309091-6.981818-16.290909-4.654546l-48.872727 27.927273c-25.6 13.963636-55.854545 13.963636-81.454545 0s-41.890909-41.890909-41.89091-69.818182V174.545455c0-6.981818-4.654545-11.636364-11.636363-11.636364h-139.636364c-6.981818 0-11.636364 4.654545-11.636363 11.636364v55.854545c0 30.254545-16.290909 55.854545-41.89091 69.818182-25.6 13.963636-55.854545 13.963636-81.454545 0l-48.872727-27.927273c-4.654545-2.327273-11.636364-2.327273-16.290909 4.654546l-69.818182 121.018181c-2.327273 4.654545-2.327273 6.981818 0 9.309091 0 2.327273 2.327273 4.654545 4.654545 6.981818L228.072727 442.181818c25.6 13.963636 41.890909 41.890909 41.890909 69.818182 0 30.254545-16.290909 55.854545-41.890909 69.818182l-48.872727 27.927273c-4.654545 2.327273-4.654545 4.654545-4.654545 6.981818v9.309091l69.818181 121.018181c2.327273 4.654545 9.309091 6.981818 16.290909 4.654546l48.872728-27.927273c11.636364-6.981818 25.6-11.636364 39.563636-11.636363z"
        fill="currentColor"
        stroke="none"
      />
      <path
        d="M512 663.272727c-83.781818 0-151.272727-67.490909-151.272727-151.272727s67.490909-151.272727 151.272727-151.272727 151.272727 67.490909 151.272727 151.272727-67.490909 151.272727-151.272727 151.272727z m0-232.727272c-44.218182 0-81.454545 37.236364-81.454545 81.454545s37.236364 81.454545 81.454545 81.454545 81.454545-37.236364 81.454545-81.454545-37.236364-81.454545-81.454545-81.454545z"
        fill="currentColor"
        stroke="none"
      />
    </template>

    <template v-else-if="name === 'grip'">
      <circle cx="9" cy="8" r="1.2" fill="currentColor" stroke="none" />
      <circle cx="15" cy="8" r="1.2" fill="currentColor" stroke="none" />
      <circle cx="9" cy="12" r="1.2" fill="currentColor" stroke="none" />
      <circle cx="15" cy="12" r="1.2" fill="currentColor" stroke="none" />
      <circle cx="9" cy="16" r="1.2" fill="currentColor" stroke="none" />
      <circle cx="15" cy="16" r="1.2" fill="currentColor" stroke="none" />
    </template>

    <template v-else-if="name === 'plus'">
      <path d="M12 6V18" :stroke-width="strokeWidth" stroke-linecap="round" />
      <path d="M6 12H18" :stroke-width="strokeWidth" stroke-linecap="round" />
    </template>

    <template v-else-if="name === 'close'">
      <path d="M6.75 6.75L17.25 17.25" :stroke-width="strokeWidth" stroke-linecap="round" />
      <path d="M17.25 6.75L6.75 17.25" :stroke-width="strokeWidth" stroke-linecap="round" />
    </template>

    <template v-else-if="name === 'chevron-down'">
      <path d="M7 10L12 15L17 10" :stroke-width="strokeWidth" stroke-linecap="round" stroke-linejoin="round" />
    </template>

    <template v-else-if="name === 'chevron-left'">
      <path d="M14.75 6.5L9.25 12L14.75 17.5" :stroke-width="strokeWidth" stroke-linecap="round" stroke-linejoin="round" />
    </template>

    <template v-else-if="name === 'chevron-right'">
      <path d="M9.25 6.5L14.75 12L9.25 17.5" :stroke-width="strokeWidth" stroke-linecap="round" stroke-linejoin="round" />
    </template>

    <template v-else-if="name === 'bookmark'">
      <path
        d="M7.25 5.5H16.75C17.44 5.5 18 6.06 18 6.75V19.25L12 15.55L6 19.25V6.75C6 6.06 6.56 5.5 7.25 5.5Z"
        :stroke-width="strokeWidth"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </template>

    <template v-else-if="name === 'zoom-in'">
      <circle cx="10.5" cy="10.5" r="5.5" :stroke-width="strokeWidth" />
      <path d="M14.7 14.7L19 19" :stroke-width="strokeWidth" stroke-linecap="round" />
      <path d="M10.5 8.2V12.8" :stroke-width="1.6" stroke-linecap="round" />
      <path d="M8.2 10.5H12.8" :stroke-width="1.6" stroke-linecap="round" />
    </template>

    <template v-else-if="name === 'zoom-out'">
      <circle cx="10.5" cy="10.5" r="5.5" :stroke-width="strokeWidth" />
      <path d="M14.7 14.7L19 19" :stroke-width="strokeWidth" stroke-linecap="round" />
      <path d="M8.2 10.5H12.8" :stroke-width="1.6" stroke-linecap="round" />
    </template>

    <template v-else-if="name === 'rotate-left'">
      <path d="M8 7H4.75V10.25" :stroke-width="strokeWidth" stroke-linecap="round" stroke-linejoin="round" />
      <path d="M5 9.5A7 7 0 1 1 6.4 15.9" :stroke-width="strokeWidth" stroke-linecap="round" />
    </template>

    <template v-else-if="name === 'rotate-right'">
      <path d="M16 7H19.25V10.25" :stroke-width="strokeWidth" stroke-linecap="round" stroke-linejoin="round" />
      <path d="M19 9.5A7 7 0 1 0 17.6 15.9" :stroke-width="strokeWidth" stroke-linecap="round" />
    </template>

    <template v-else-if="name === 'reset-view'">
      <path d="M8 5.5H5.5V8" :stroke-width="strokeWidth" stroke-linecap="round" stroke-linejoin="round" />
      <path d="M16 5.5H18.5V8" :stroke-width="strokeWidth" stroke-linecap="round" stroke-linejoin="round" />
      <path d="M8 18.5H5.5V16" :stroke-width="strokeWidth" stroke-linecap="round" stroke-linejoin="round" />
      <path d="M16 18.5H18.5V16" :stroke-width="strokeWidth" stroke-linecap="round" stroke-linejoin="round" />
      <circle cx="12" cy="12" r="1.5" :stroke-width="1.5" />
    </template>

    <template v-else-if="name === 'history'">
      <path d="M5.5 7.5V4.75H8.25" :stroke-width="strokeWidth" stroke-linecap="round" stroke-linejoin="round" />
      <path d="M5.8 6.2A7 7 0 1 1 4.95 12" :stroke-width="strokeWidth" stroke-linecap="round" />
      <path d="M12 8.25V12.2L14.85 13.9" :stroke-width="strokeWidth" stroke-linecap="round" stroke-linejoin="round" />
    </template>

    <template v-else-if="name === 'recent'">
      <circle cx="5" cy="7" r="1.2" fill="currentColor" stroke="none" />
      <circle cx="5" cy="12" r="1.2" fill="currentColor" stroke="none" />
      <circle cx="5" cy="17" r="1.2" fill="currentColor" stroke="none" />
      <path d="M9 7h10" :stroke-width="strokeWidth" stroke-linecap="round" />
      <path d="M9 12h7" :stroke-width="strokeWidth" stroke-linecap="round" />
      <path d="M9 17h4" :stroke-width="strokeWidth" stroke-linecap="round" />
    </template>

    <template v-else-if="name === 'tools'">
      <path
        d="M8.4 4.8 6.1 7.1l2.2 2.2 2.3-2.3-2.2-2.2ZM5.1 8.7l2.2 2.2-2 2-2.2-2.2 2-2ZM13.6 4.8l2.2 2.2-2.3 2.3-2.2-2.2 2.3-2.3ZM18.9 8.7l2 2-2.2 2.2-2-2 2.2-2.2ZM11.9 11.8l1.4 1.4-5.8 5.8a1 1 0 0 1-.7.3H4.4v-2.4a1 1 0 0 1 .3-.7l5.8-5.8 1.4 1.4Z"
        :stroke-width="strokeWidth"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
      <path
        d="M13.5 11.9 19 17.4"
        :stroke-width="strokeWidth"
        stroke-linecap="round"
      />
    </template>

    <template v-else-if="name === 'video'">
      <path
        d="M374.6 636.5c4.4 0 8.5-1.2 12.1-3.3l171.7-100c8-3.6 13.6-11.9 13.6-21.5 0-8.8-4.8-16.6-11.9-20.7l-167.8-97.8c-4.3-5-10.7-8.1-17.7-8.1-13.1 0-23.6 10.7-23.6 23.8v1.3l-0.3 0.2 0.4 199.8c-0.1 0.8-0.1 1.6-0.1 2.5 0 13.2 10.6 23.8 23.6 23.8z"
        fill="currentColor"
        stroke="none"
      />
      <path
        d="M64.7 586.3a32.2 32.1 0 1 0 64.4 0 32.2 32.1 0 1 0-64.4 0Z"
        fill="currentColor"
        stroke="none"
      />
      <path
        d="M960 398.3c0.1-1.6 0.2-3.2 0.2-4.8 0-35-28.5-63.3-63.6-63.3-11.7 0-22.7 3.2-32.2 8.7l-0.5-0.3-31.5 18.2v-64.7c-0.1-73.1-59.9-133-133.1-133H197.4c-73.1 0-133 59.8-133 133v165.8h0.2c0 17.7 14.4 32.1 32.2 32.1s32.2-14.4 32.2-32.1h0.2V287c0-35.2 28.8-64 64-64h510.2c35.2 0 64 28.8 64 64v448.9c0 35.2-28.8 64-64 64H193.3c-35.2 0-64-28.8-64-64v-21.4c0-17.7-14.4-32.1-32.2-32.1-17.8 0-32.2 14.4-32.2 32.1h-0.4v15.3c0 73.2 59.9 133 133 133h501.9c73.2 0 133-59.8 133-133v-64.1l33.1 19.1 0.1-0.1c9.2 5.1 19.8 8 31 8 35.1 0 63.6-28.4 63.6-63.3 0-1.6-0.1-3.2-0.2-4.8V398.3z m-63.6 205.1c-0.3 7.8-6.9 14.1-15 14.1-2.7 0-5.3-0.7-7.5-2l-41.5-23.7V430.1l40.9-23.2c2.3-1.5 5.1-2.3 8.1-2.3 8.3 0 15 6.6 15 14.6v184.2z"
        fill="currentColor"
        stroke="none"
      />
    </template>

    <template v-else-if="name === 'folder'">
      <path
        d="M418.688 133.333333a122.666667 122.666667 0 0 1 93.013333 42.666667H785.066667a186.666667 186.666667 0 0 1 186.581333 181.162667l0.085333 5.504v17.92h-0.256c0.170667 1.429333 0.256 2.88 0.256 4.373333V789.333333a186.666667 186.666667 0 0 1-186.666666 186.666667H238.933333A186.666667 186.666667 0 0 1 52.266667 789.333333V320a186.666667 186.666667 0 0 1 186.666666-186.666667z m0 74.666667H238.933333A112 112 0 0 0 126.933333 320v469.333333c0 61.866667 50.133333 112 112 112h546.133334c61.866667 0 112-50.133333 112-112V422.272H615.04a122.666667 122.666667 0 0 1-113.834667-76.992l-1.834666-4.842667-35.413334-100.416a48 48 0 0 0-45.269333-32.021333zM448 688a37.333333 37.333333 0 0 1 3.072 74.538667L448 762.666667h-170.666667a37.333333 37.333333 0 0 1-3.072-74.538667L277.333333 688h170.666667z m337.066667-437.333333H546.88l22.912 64.917333a48 48 0 0 0 41.685333 31.914667l3.562667 0.128 281.024-0.021334a112.021333 112.021333 0 0 0-106.389333-96.853333l-4.608-0.085333z"
        fill="currentColor"
        stroke="none"
      />
    </template>

    <template v-else-if="name === 'audio'">
      <path d="M6.5 14.2V9.8" :stroke-width="strokeWidth" stroke-linecap="round" />
      <path d="M9 16.2V7.8" :stroke-width="strokeWidth" stroke-linecap="round" />
      <path d="M11.5 18V6" :stroke-width="strokeWidth" stroke-linecap="round" />
      <path d="M14 15.2V8.8" :stroke-width="strokeWidth" stroke-linecap="round" />
      <path d="M16.5 13V11" :stroke-width="strokeWidth" stroke-linecap="round" />
    </template>

    <template v-else-if="name === 'file'">
      <path
        d="M8.25 5.5H13.1L18 10.4V18.25C18 18.94 17.44 19.5 16.75 19.5H7.25C6.56 19.5 6 18.94 6 18.25V6.75C6 6.06 6.56 5.5 7.25 5.5H8.25Z"
        :stroke-width="strokeWidth"
        stroke-linejoin="round"
      />
      <path d="M13 5.5V10.25H17.75" :stroke-width="strokeWidth" stroke-linecap="round" stroke-linejoin="round" />
    </template>

    <template v-else-if="name === 'file-preview'">
      <path
        d="M581.568 64c43.904 0 85.76 18.688 115.008 51.392l149.824 167.232a32 32 0 0 1 8.192 21.376v187.2a32 32 0 1 1-64 0l-0.064-159.168h-140.8a62.592 62.592 0 0 1-62.592-62.592l-0.064-141.184L581.568 128H282.432C232.512 128 192 168.512 192 218.432v587.136C192 855.488 232.512 896 282.432 896h167.424a32 32 0 1 1 0 64H282.432A154.432 154.432 0 0 1 128 805.568V218.432A154.432 154.432 0 0 1 282.432 64h299.136z m96.064 526.528c54.72 0 106.176 27.328 153.664 71.872a439.04 439.04 0 0 1 59.392 68.864 32 32 0 0 1 0 35.264 439.04 439.04 0 0 1-59.392 68.928c-47.488 44.48-98.944 71.808-153.6 71.808-54.784 0-106.24-27.328-153.728-71.808a439.04 439.04 0 0 1-59.392-68.928 32 32 0 0 1 0-35.2 439.04 439.04 0 0 1 59.392-68.928c47.552-44.544 98.944-71.872 153.664-71.872z m0 64c-35.328 0-73.088 20.096-109.888 54.528a366.08 366.08 0 0 0-25.408 26.432l-11.264 13.376 11.264 13.44c3.968 4.544 8.064 9.024 12.288 13.44l13.12 12.992c36.8 34.496 74.56 54.528 109.888 54.528s73.088-20.032 109.888-54.528a375.68 375.68 0 0 0 36.8-40l-11.392-13.248a378.112 378.112 0 0 0-12.224-13.44l-13.184-12.992c-36.8-34.432-74.56-54.528-109.888-54.528z m0 52.288c22.912 0 41.408 18.816 41.408 42.112 0 23.232-18.56 42.112-41.408 42.112a41.792 41.792 0 0 1-41.408-42.112c0-23.296 18.56-42.112 41.408-42.112z m-26.56-546.304v107.52h96.384l-96.384-107.52z"
        fill="currentColor"
        stroke="none"
      />
    </template>

    <template v-else-if="name === 'agent-sessions'">
      <path
        d="M921.173333 905.728l-56.021333-330.922667h18.901333a27.733333 27.733333 0 0 0 27.52-28.16V338.773333a27.733333 27.733333 0 0 0-27.52-28.16h-260.010666V113.493333A27.733333 27.733333 0 0 0 596.565333 85.333333h-169.130666a27.733333 27.733333 0 0 0-27.477334 28.16v197.12H139.904a27.733333 27.733333 0 0 0-27.477333 28.16v207.872c0 15.616 12.288 28.16 27.477333 28.16h18.944L102.826667 905.728a27.733333 27.733333 0 0 0 27.050666 32.938667h764.245334a27.989333 27.989333 0 0 0 27.050666-32.938667zM186.453333 386.389333h287.488v-225.28h76.117334v225.28h287.488v112.64H186.453333v-112.64z m494.677334 476.458667v-168.917333a8.576 8.576 0 0 0-8.448-8.661334h-50.773334a8.576 8.576 0 0 0-8.448 8.661334v168.96H410.538667v-168.96a8.576 8.576 0 0 0-8.490667-8.661334H351.317333a8.576 8.576 0 0 0-8.448 8.661334v168.96H185.173333l47.658667-281.6H791.04l47.701333 281.6h-157.610666z"
        fill="currentColor"
        stroke="none"
      />
    </template>

    <template v-else-if="name === 'lightbulb'">
      <path
        d="M15 14c.2-1 .7-1.7 1.5-2.5 1-.9 1.5-2.2 1.5-3.5A6 6 0 0 0 6 8c0 1 .2 2.2 1.5 3.5.7.7 1.3 1.5 1.5 2.5"
        :stroke-width="strokeWidth"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
      <path d="M9 18h6" :stroke-width="strokeWidth" stroke-linecap="round" />
      <path d="M10 22h4" :stroke-width="strokeWidth" stroke-linecap="round" />
    </template>

    <template v-else-if="name === 'copy'">
      <rect
        x="9"
        y="9"
        width="13"
        height="13"
        rx="2"
        ry="2"
        :stroke-width="strokeWidth"
      />
      <path
        d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"
        :stroke-width="strokeWidth"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </template>
  </svg>
</template>

<style scoped>
.desktop-ui-icon {
  display: block;
  flex: none;
  stroke: currentColor;
  stroke-linecap: round;
  stroke-linejoin: round;
}
</style>
