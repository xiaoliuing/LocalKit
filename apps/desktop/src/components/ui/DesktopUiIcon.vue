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
    :viewBox="name === 'settings' || name === 'folder' || name === 'video' ? '0 0 1024 1024' : '0 0 24 24'"
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
        d="M381.632 138.688h47.36c68.48 0 123.008 0 165.76 5.76 44.224 5.952 80.64 18.56 109.44 47.36 28.8 28.8 41.408 65.216 47.36 109.44 0.64 4.672 1.216 9.472 1.728 14.464 25.408-20.48 47.552-37.376 66.944-49.088 29.184-17.664 62.72-30.208 96.384-13.44 33.216 16.448 44.352 50.432 49.216 84.608 4.864 34.304 4.864 80.832 4.864 138.112v72.192c0 57.28 0 103.872-4.864 138.112-4.864 34.176-16 68.224-49.28 84.672-33.6 16.64-67.136 4.16-96.32-13.44-19.392-11.776-41.536-28.608-66.944-49.152a443.136 443.136 0 0 1-1.728 14.528c-5.952 44.16-18.56 80.64-47.36 109.44-28.8 28.8-65.216 41.408-109.44 47.36-42.752 5.76-97.28 5.76-165.76 5.76h-47.36c-68.416 0-122.944 0-165.76-5.76-44.16-5.952-80.64-18.56-109.44-47.36-28.8-28.8-41.408-65.28-47.36-109.44-5.76-42.816-5.76-97.408-5.76-165.76V466.944c0-68.352 0-122.88 5.76-165.76 5.952-44.16 18.56-80.64 47.36-109.44 28.8-28.8 65.28-41.344 109.44-47.36 42.816-5.76 97.344-5.76 165.76-5.76z m375.36 489.984c42.88 35.328 72.832 59.712 96.32 73.92 25.024 15.104 32.256 12.224 34.88 10.88 3.008-1.472 9.984-6.272 14.272-36.224 4.096-29.248 4.224-71.232 4.224-131.84V478.592c0-60.672-0.128-102.656-4.224-131.904-4.288-29.952-11.264-34.752-14.272-36.288-2.56-1.28-9.856-4.16-34.88 10.88-23.488 14.272-53.504 38.72-96.32 73.984 0.32 21.952 0.32 45.824 0.32 71.68v90.048c0 25.792 0 49.664-0.32 71.68zM224.384 207.872c-37.12 4.992-57.728 14.208-72.704 29.184-14.912 14.912-24.192 35.584-29.184 72.704-5.12 38.016-5.12 88.32-5.12 159.552v85.376c0 71.296 0 121.6 5.12 159.552 4.992 37.12 14.272 57.792 29.184 72.704 14.976 14.976 35.584 24.256 72.704 29.184 38.08 5.12 88.32 5.184 159.616 5.184h42.688c71.296 0 121.536 0 159.552-5.12 37.12-5.056 57.792-14.272 72.704-29.248 14.976-14.912 24.192-35.584 29.184-72.704 5.12-38.016 5.184-88.32 5.184-159.552V469.312c0-71.296 0-121.536-5.12-159.552-5.056-37.12-14.272-57.792-29.248-72.704-14.912-14.976-35.584-24.192-72.704-29.184-38.016-5.12-88.32-5.184-159.552-5.184H384c-71.296 0-121.6 0-159.616 5.12z"
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
