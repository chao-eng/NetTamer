import { ref } from 'vue'

export type ToastType = 'info' | 'success' | 'error' | 'warning'

export interface ToastItem {
  id: number
  message: string
  type: ToastType
}

const toasts = ref<ToastItem[]>([])
let seq = 0

/** 推送一个 toast，3 秒后自动移除。 */
export function toast(message: string, type: ToastType = 'info') {
  const id = ++seq
  toasts.value.push({ id, message, type })
  setTimeout(() => {
    toasts.value = toasts.value.filter((t) => t.id !== id)
  }, 3000)
}

/** 供组件获取响应式 toast 列表。 */
export function useToasts() {
  return toasts
}
