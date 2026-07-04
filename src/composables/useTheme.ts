import { ref, onMounted } from "vue"

export type Theme = "light" | "dark"

const STORAGE_KEY = "transfer-theme"
const DATA_ATTR = "data-theme"

const current = ref<Theme>("light")

export function useTheme() {
  function apply(next: Theme) {
    current.value = next
    document.documentElement.setAttribute(DATA_ATTR, next)
    document.documentElement.style.colorScheme = next
  }

  function persist(next: Theme) {
    localStorage.setItem(STORAGE_KEY, next)
  }

  function setTheme(next: Theme) {
    apply(next)
    persist(next)
  }

  function toggle() {
    setTheme(current.value === "light" ? "dark" : "light")
  }

  function detectPrefersDark(): boolean {
    return window.matchMedia("(prefers-color-scheme: dark)").matches
  }

  function loadSaved(): Theme | null {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (raw === "light" || raw === "dark") return raw
    return null
  }

  // 首次挂载时从 localStorage / 系统偏好恢复
  onMounted(() => {
    const saved = loadSaved()
    const resolved = saved ?? (detectPrefersDark() ? "dark" : "light")
    apply(resolved)
    persist(resolved)
  })

  return {
    theme: current,
    toggle,
    setTheme,
  }
}
