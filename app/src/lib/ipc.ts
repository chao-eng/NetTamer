/**
 * Tauri IPC 安全封装。
 *
 * 前端既要跑在 Tauri WebView 里（真实后端），也要能在浏览器 `vite dev`
 * 下直接打开调试。因此所有 `invoke` / `listen` 都必须经过 {@link isTauri} 守卫：
 * 不在 Tauri 中时返回调用方给定的回退值，绝不抛异常。
 */
import { invoke } from '@tauri-apps/api/core'
import { listen, type Event, type UnlistenFn } from '@tauri-apps/api/event'
import { isTauri } from '@/types'

export type { UnlistenFn }

const warned = new Set<string>()

function warnOnce(key: string, message: string) {
  if (warned.has(key)) return
  warned.add(key)
  console.info(`[NetTamer] ${message}`)
}

/**
 * 调用后端命令。非 Tauri 环境直接返回 `fallback`。
 *
 * @param cmd 命令名，必须与 `tauri::generate_handler!` 中注册的一致
 * @param args 参数对象，键名使用 camelCase（Tauri 自动转 snake_case）
 * @param fallback 非 Tauri 环境下的回退值
 */
export async function invokeSafe<T>(
  cmd: string,
  args?: Record<string, unknown>,
  fallback?: T,
): Promise<T> {
  if (!isTauri()) {
    warnOnce(cmd, `非 Tauri 环境，命令 "${cmd}" 已跳过（使用本地模拟数据）`)
    return fallback as T
  }
  return (await invoke<T>(cmd, args)) as T
}

/**
 * 订阅后端事件。非 Tauri 环境返回一个空的 unlisten 函数。
 */
export async function listenSafe<T>(
  event: string,
  handler: (payload: T) => void,
): Promise<UnlistenFn> {
  if (!isTauri()) {
    warnOnce(`listen:${event}`, `非 Tauri 环境，事件 "${event}" 未订阅`)
    return () => {}
  }
  return listen<T>(event, (e: Event<T>) => handler(e.payload))
}

/** 批量取消订阅，并清空传入数组。 */
export function unlistenAll(unlisteners: UnlistenFn[]) {
  while (unlisteners.length) {
    const fn = unlisteners.pop()
    try {
      fn?.()
    } catch {
      /* 忽略取消订阅过程中的异常 */
    }
  }
}

/** 统一把后端错误（Rust 侧为 String）转成可展示文本。 */
export function toErrorMessage(err: unknown): string {
  if (typeof err === 'string') return err
  if (err instanceof Error) return err.message
  try {
    return JSON.stringify(err)
  } catch {
    return String(err)
  }
}
