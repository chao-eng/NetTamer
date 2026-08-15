import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invokeSafe } from '@/lib/ipc'
import {
  DEFAULT_CONFIG,
  CONFIG_KEYS,
  type ThemeMode,
} from '@/types'

function applyTheme(theme: string) {
  if (typeof document === 'undefined') return
  if (theme === 'dark') {
    document.documentElement.classList.add('dark')
  } else {
    document.documentElement.classList.remove('dark')
  }
}

export const useSettingsStore = defineStore('settings', () => {
  const config = ref<Record<string, string>>({ ...DEFAULT_CONFIG })

  async function load() {
    const remote = await invokeSafe<Record<string, string>>(
      'get_all_config',
      undefined,
      {},
    )
    config.value = { ...DEFAULT_CONFIG, ...(remote ?? {}) }
    applyTheme(config.value[CONFIG_KEYS.theme] ?? 'dark')
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

  return {
    config,
    load,
    set,
    setRefreshInterval,
    toggleTheme,
    toggleAutoStart,
    toggleMinimizeToTray,
  }
})
