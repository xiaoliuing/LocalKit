import { shallowRef } from "vue";

export type ResolvedTheme = "light" | "dark";
export type ThemeMode = "system" | "light" | "dark";
export type ThemeAccentId =
  | "atlas-blue"
  | "ocean-teal"
  | "forest-green"
  | "sunset-amber"
  | "dusty-rose"
  | "slate-indigo"
  | "sage-olive"
  | "terracotta"
  | "plum-orchid"
  | "walnut-brown";

export type ThemeAccent = {
  id: ThemeAccentId;
  label: string;
  color: string;
};

const MODE_STORAGE_KEY = "docs-atlas-theme-mode";
const ACCENT_STORAGE_KEY = "docs-atlas-theme-accent";

const themeMode = shallowRef<ThemeMode>("system");
const resolvedTheme = shallowRef<ResolvedTheme>("light");
const themeAccent = shallowRef<ThemeAccentId>("slate-indigo");

const themeAccents: ThemeAccent[] = [
  { id: "slate-indigo", label: "靛云蓝", color: "#5B6FD6" },
  // { id: "sage-olive", label: "鼠尾草", color: "#6F9478" },
  { id: "terracotta", label: "赤陶橘", color: "#C97059" },
  { id: "plum-orchid", label: "晚梅紫", color: "#9A68B2" },
  { id: "walnut-brown", label: "榛木棕", color: "#9B7653" },
  { id: "atlas-blue", label: "星图蓝", color: "#1F54D9" },
  { id: "ocean-teal", label: "海雾青", color: "#0F8C95" },
  { id: "forest-green", label: "森林绿", color: "#1F8F63" },
  { id: "sunset-amber", label: "落日金", color: "#C28A1A" },
  { id: "dusty-rose", label: "雾玫瑰", color: "#C05F7F" },
];

let initialized = false;
let cleanupSystemListener: (() => void) | null = null;

export function useTheme() {
  function initializeTheme() {
    if (initialized || typeof window === "undefined") {
      return;
    }

    initialized = true;

    const savedMode = window.localStorage.getItem(MODE_STORAGE_KEY);
    const savedAccent = window.localStorage.getItem(ACCENT_STORAGE_KEY);

    themeMode.value = isThemeMode(savedMode) ? savedMode : "system";
    themeAccent.value = resolveThemeAccent(savedAccent);

    applyTheme();
    bindSystemThemeListener();
  }

  function cycleThemeMode() {
    const nextMode = getNextThemeMode(themeMode.value);
    setThemeMode(nextMode);
  }

  function setThemeMode(nextMode: ThemeMode) {
    themeMode.value = nextMode;

    if (typeof window !== "undefined") {
      window.localStorage.setItem(MODE_STORAGE_KEY, nextMode);
    }

    applyTheme();
  }

  function setThemeAccent(nextAccent: ThemeAccentId) {
    themeAccent.value = nextAccent;

    if (typeof window !== "undefined") {
      window.localStorage.setItem(ACCENT_STORAGE_KEY, nextAccent);
    }

    applyTheme();
  }

  return {
    cycleThemeMode,
    initializeTheme,
    resolvedTheme,
    setThemeAccent,
    setThemeMode,
    themeAccent,
    themeAccents,
    themeMode,
  };
}

function bindSystemThemeListener() {
  if (cleanupSystemListener || typeof window === "undefined") {
    return;
  }

  const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
  const listener = () => {
    if (themeMode.value === "system") {
      applyTheme();
    }
  };

  mediaQuery.addEventListener("change", listener);
  cleanupSystemListener = () => {
    mediaQuery.removeEventListener("change", listener);
  };
}

function getSystemTheme(): ResolvedTheme {
  if (typeof window === "undefined") {
    return "light";
  }

  return window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
}

function applyTheme() {
  resolvedTheme.value =
    themeMode.value === "system" ? getSystemTheme() : themeMode.value;

  if (typeof document === "undefined") {
    return;
  }

  document.documentElement.dataset.theme = resolvedTheme.value;
  document.documentElement.dataset.themeMode = themeMode.value;
  document.documentElement.dataset.accent = themeAccent.value;
  document.documentElement.style.colorScheme = resolvedTheme.value;
}

function getNextThemeMode(currentMode: ThemeMode): ThemeMode {
  if (currentMode === "system") {
    return "light";
  }

  if (currentMode === "light") {
    return "dark";
  }

  return "system";
}

function isThemeMode(value: string | null): value is ThemeMode {
  return value === "system" || value === "light" || value === "dark";
}

function isThemeAccent(value: string | null): value is ThemeAccentId {
  return themeAccents.some((accent) => accent.id === value);
}

function resolveThemeAccent(value: string | null): ThemeAccentId {
  if (value === "pure-white") {
    return "slate-indigo";
  }

  return isThemeAccent(value) ? value : "slate-indigo";
}
