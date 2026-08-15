export type { UnlistenFn } from '@tauri-apps/api/event'


/** 单个进程的网络统计快照（事件 `speed:update` / 命令 `get_process_list`）。 */
export interface ProcessStats {
  pid: number
  name: string
  path: string
  /** Base64 编码的进程图标（可能为空字符串） */
  iconB64: string
  /** bytes/sec */
  uploadRate: number
  /** bytes/sec */
  downloadRate: number
  /** 累计字节数 */
  totalUpload: number
  /** 累计字节数 */
  totalDownload: number
}

/** 预警规则。 */
export interface Rule {
  id: string
  name: string
  /** 进程名匹配模式，支持通配符 `*` */
  processName: string
  /** 阈值，bytes/sec */
  threshold: number
  /** 0=Upload, 1=Download, 2=Both，见 {@link Direction} */
  direction: number
  cooldownSec: number
  enabled: boolean
  /** unix 时间戳（秒） */
  createdAt: number
}

/** 预警触发事件（事件 `alert:triggered`）。 */
export interface AlertEvent {
  id: string
  ruleId: string
  processName: string
  pid: number
  /** 触发预警的方向：0 = 上传, 1 = 下载, 2 = 双向 */
  direction?: number
  /** 触发时的实际速率 bytes/sec */
  currentRate: number
  /** 规则阈值 bytes/sec */
  threshold: number
  /** unix 时间戳（秒） */
  triggeredAt: number
}

/** 限速策略（事件 `throttle:changed`）。 */
export interface Policy {
  id: string
  /** 策略名，约定前缀 `NT_` */
  name: string
  processName: string
  /** 限速值，bits/sec，0 表示不限速 */
  rateLimitBps: number
  limitUpload: boolean
  limitDownload: boolean
  active: boolean
  /** unix 时间戳（秒） */
  createdAt: number
}

/** 预警历史查询过滤条件（命令 `get_alert_history`）。 */
export interface AlertHistoryFilter {
  ruleId?: string
  since?: number
  limit?: number
}

/** 系统级总速率（事件 `system:stats`）。 */
export interface SystemStats {
  /** bytes/sec */
  totalUploadRate: number
  /** bytes/sec */
  totalDownloadRate: number
}

/** 流量方向，与后端 `direction: i32` 对应。 */
export enum Direction {
  Upload = 0,
  Download = 1,
  Both = 2,
}

export const DIRECTION_OPTIONS: Array<{ label: string; value: Direction }> = [
  { label: '上传', value: Direction.Upload },
  { label: '下载', value: Direction.Download },
  { label: '上传 + 下载', value: Direction.Both },
]

/** 将 `direction` 数值转为中文标签。 */
export function directionLabel(direction: number): string {
  switch (direction) {
    case Direction.Upload:
      return '上传'
    case Direction.Download:
      return '下载'
    case Direction.Both:
      return '上传 + 下载'
    default:
      return '未知'
  }
}

/** 进程列表可排序字段。 */
export type SortField = 'uploadRate' | 'downloadRate' | 'name' | 'pid'

export type SortOrder = 'asc' | 'desc'

/** 速率折线图的采样点。 */
export interface SpeedPoint {
  /** 采样时刻（毫秒时间戳） */
  t: number
  /** 上传 bytes/sec */
  up: number
  /** 下载 bytes/sec */
  down: number
}

export type ThemeMode = 'light' | 'dark'

/** SQLite `config` 表中的键名（见 doc/architecture.md §10.1）。 */
export const CONFIG_KEYS = {
  refreshInterval: 'refresh_interval_ms',
  theme: 'theme',
  autoStart: 'auto_start',
  minimizeToTray: 'minimize_to_tray',
  alertSound: 'alert_sound',
  taskbarSpeed: 'taskbar_speed',
  floatingSpeed: 'floating_speed',
  floatingClickThrough: 'floating_click_through',
  floatingOpacity: 'floating_opacity',
} as const

export type ConfigKey = (typeof CONFIG_KEYS)[keyof typeof CONFIG_KEYS]

/** 默认配置，浏览器（非 Tauri）环境下作为回退值。 */
export const DEFAULT_CONFIG: Record<string, string> = {
  [CONFIG_KEYS.refreshInterval]: '1000',
  [CONFIG_KEYS.theme]: 'light',
  [CONFIG_KEYS.autoStart]: 'false',
  [CONFIG_KEYS.minimizeToTray]: 'true',
  [CONFIG_KEYS.alertSound]: 'true',
  [CONFIG_KEYS.taskbarSpeed]: 'false',
  [CONFIG_KEYS.floatingSpeed]: 'false',
  [CONFIG_KEYS.floatingClickThrough]: 'false',
  [CONFIG_KEYS.floatingOpacity]: '100',
}

/**
 * 是否运行在 Tauri WebView 中。
 * 非 Tauri（普通浏览器 `vite dev`）环境下所有 invoke/listen 均需跳过。
 */
export const isTauri = () =>
  typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
