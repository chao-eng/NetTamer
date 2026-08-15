import { useProcessStore } from '@/stores/processStore'

/** 监控相关便捷封装。 */
export function useProcessMonitor() {
  const processStore = useProcessStore()
  return {
    processStore,
    fetchList: processStore.fetchList,
    start: processStore.start,
    stop: processStore.stop,
    bindEvents: processStore.bindEvents,
    toggleMonitor: async () => {
      if (processStore.isMonitoring) await processStore.stop()
      else await processStore.start()
    },
  }
}
