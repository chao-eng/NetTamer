import { useSettingsStore } from '@/stores/settingsStore'

/** 配置相关便捷封装。 */
export function useConfig() {
  const settingsStore = useSettingsStore()
  return {
    settingsStore,
    load: settingsStore.load,
    set: settingsStore.set,
    setRefreshInterval: settingsStore.setRefreshInterval,
    toggleTheme: settingsStore.toggleTheme,
    toggleAutoStart: settingsStore.toggleAutoStart,
    toggleMinimizeToTray: settingsStore.toggleMinimizeToTray,
  }
}
