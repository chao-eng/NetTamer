import { defineStore } from 'pinia'
import { ref } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { invokeSafe, listenSafe } from '@/lib/ipc'
import {
  DEFAULT_CONFIG,
  CONFIG_KEYS,
  type ThemeMode,
} from '@/types'

function applyTheme(theme: string) {
  if (typeof document === 'undefined') return
  const isDark = theme === 'dark'
  if (isDark) {
    document.documentElement.classList.add('dark')
    document.documentElement.setAttribute('data-theme', 'dark')
  } else {
    document.documentElement.classList.remove('dark')
    document.documentElement.setAttribute('data-theme', 'light')
  }
  localStorage.setItem('app-theme', theme)

  try {
    getCurrentWindow()
      .setTheme(isDark ? 'dark' : 'light')
      .catch((e) => console.warn('Failed to set native window theme:', e))
  } catch {
    // Non-Tauri fallback
  }
}

// Immediately apply saved theme on module load to prevent titlebar flicker
const initialTheme = (typeof localStorage !== 'undefined' && localStorage.getItem('app-theme')) || 'light'
applyTheme(initialTheme)

export const useSettingsStore = defineStore('settings', () => {
  const config = ref<Record<string, string>>({ ...DEFAULT_CONFIG })

  // Listen to remote config sync events across all windows / tray menus
  listenSafe<{ key: string; value: string }>('config:sync', (data) => {
    if (data && data.key) {
      config.value[data.key] = data.value
      if (data.key === CONFIG_KEYS.theme) {
        applyTheme(data.value)
      }
    }
  }).catch(() => {})

  async function load() {
    const remote = await invokeSafe<Record<string, string>>(
      'get_all_config',
      undefined,
      {},
    )
    config.value = { ...DEFAULT_CONFIG, ...(remote ?? {}) }
    applyTheme(config.value[CONFIG_KEYS.theme] ?? 'light')
  }

  async function set(key: string, value: string) {
    config.value[key] = value
    await invokeSafe('set_config', { key, value })
  }

  async function setRefreshInterval(ms: number) {
    await set(CONFIG_KEYS.refreshInterval, String(ms))
  }

  async function toggleTheme() {
    const cur = config.value[CONFIG_KEYS.theme] ?? 'dark'
    const next: ThemeMode = cur === 'dark' ? 'light' : 'dark'
    await set(CONFIG_KEYS.theme, next)
    applyTheme(next)
  }

  async function toggleAutoStart() {
    const cur = config.value[CONFIG_KEYS.autoStart] === 'true'
    await set(CONFIG_KEYS.autoStart, String(!cur))
  }

  async function toggleMinimizeToTray() {
    const cur = config.value[CONFIG_KEYS.minimizeToTray] === 'true'
    await set(CONFIG_KEYS.minimizeToTray, String(!cur))
  }

  async function toggleIncludeKernelStats() {
    const cur = config.value[CONFIG_KEYS.includeKernelStats] === 'true'
    await set(CONFIG_KEYS.includeKernelStats, String(!cur))
  }

  async function toggleTaskbarSpeed() {
    const cur = config.value[CONFIG_KEYS.taskbarSpeed] === 'true'
    await set(CONFIG_KEYS.taskbarSpeed, String(!cur))
  }

  async function toggleFloatingSpeed() {
    const cur = config.value[CONFIG_KEYS.floatingSpeed] === 'true'
    await set(CONFIG_KEYS.floatingSpeed, String(!cur))
  }

  async function toggleFloatingClickThrough() {
    const cur = config.value[CONFIG_KEYS.floatingClickThrough] === 'true'
    await set(CONFIG_KEYS.floatingClickThrough, String(!cur))
  }

  async function setFloatingOpacity(op: number) {
    await set(CONFIG_KEYS.floatingOpacity, String(op))
  }

  const isImmersiveWindow = ref(false)

  function toggleImmersiveWindow(val?: boolean) {
    if (typeof val === 'boolean') {
      isImmersiveWindow.value = val
    } else {
      isImmersiveWindow.value = !isImmersiveWindow.value
    }
  }

  return {
    config,
    load,
    set,
    setRefreshInterval,
    toggleTheme,
    toggleAutoStart,
    toggleMinimizeToTray,
    toggleIncludeKernelStats,
    toggleTaskbarSpeed,
    toggleFloatingSpeed,
    toggleFloatingClickThrough,
    setFloatingOpacity,
    isImmersiveWindow,
    toggleImmersiveWindow,
  }
})
