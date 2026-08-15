import { useAlertStore } from '@/stores/alertStore'

/** 预警规则相关便捷封装。 */
export function useAlertRules() {
  const alertStore = useAlertStore()
  return {
    alertStore,
    loadRules: alertStore.loadRules,
    createRule: alertStore.createRule,
    updateRule: alertStore.updateRule,
    deleteRule: alertStore.deleteRule,
    loadHistory: alertStore.loadHistory,
  }
}
