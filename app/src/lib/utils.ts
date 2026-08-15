import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'

/** 合并 Tailwind class，冲突时后者生效（shadcn-vue 约定工具）。 */
export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

/** 将数值限制在 [min, max] 区间内。 */
export function clamp(value: number, min: number, max: number) {
  return Math.min(Math.max(value, min), max)
}
