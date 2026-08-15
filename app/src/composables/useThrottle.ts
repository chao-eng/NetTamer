import { useThrottleStore } from '@/stores/throttleStore'

/** 限速策略相关便捷封装。 */
export function useThrottle() {
  const throttleStore = useThrottleStore()
  return {
    throttleStore,
    load: throttleStore.load,
    apply: throttleStore.apply,
    remove: throttleStore.remove,
  }
}
