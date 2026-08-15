/** 纯格式化工具，无副作用，可在任意组件中 import 使用。 */

export function formatBytes(n: number): string {
  if (!isFinite(n) || n <= 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let i = 0
  let v = n
  while (v >= 1024 && i < units.length - 1) {
    v /= 1024
    i++
  }
  return `${v.toFixed(2)} ${units[i]}`
}

export function formatSpeed(bytesPerSec: number): string {
  return `${formatBytes(bytesPerSec)}/s`
}

/** 网络速率常用 1000 进制：bps / Kbps / Mbps / Gbps。 */
export function formatBps(bitsPerSec: number): string {
  if (!isFinite(bitsPerSec) || bitsPerSec <= 0) return '0 bps'
  const units: Array<{ v: number; s: string }> = [
    { v: 1e9, s: 'Gbps' },
    { v: 1e6, s: 'Mbps' },
    { v: 1e3, s: 'Kbps' },
    { v: 1, s: 'bps' },
  ]
  for (const u of units) {
    if (bitsPerSec >= u.v) return `${(bitsPerSec / u.v).toFixed(2)} ${u.s}`
  }
  return `${bitsPerSec} bps`
}

export function formatRate(value: number, unit: 'bytes' | 'bits'): string {
  return unit === 'bytes' ? formatSpeed(value) : formatBps(value)
}
