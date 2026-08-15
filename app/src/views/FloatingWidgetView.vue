<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { listenSafe, type UnlistenFn, invokeSafe, unlistenAll } from '@/lib/ipc'
import { ArrowUp, ArrowDown } from 'lucide-vue-next'

const uploadStr = ref('0.0 K/s')
const downloadStr = ref('0.0 K/s')
const isDark = ref(false)
const opacity = ref(100)
const isClickThrough = ref(false)

function applyTheme(theme: string) {
  isDark.value = theme === 'dark'
  if (isDark.value) {
    document.documentElement.classList.add('dark')
    document.documentElement.setAttribute('data-theme', 'dark')
  } else {
    document.documentElement.classList.remove('dark')
    document.documentElement.setAttribute('data-theme', 'light')
  }
}

function formatSpeed(bytesPerSec: number): string {
  if (!bytesPerSec || bytesPerSec <= 0 || isNaN(bytesPerSec)) return '0.0 K/s'
  if (bytesPerSec < 1024) {
    return `${(bytesPerSec / 1024).toFixed(1)} K/s`
  }
  if (bytesPerSec < 1024 * 1024) {
    const kb = bytesPerSec / 1024
    return kb < 100 ? `${kb.toFixed(1)} K/s` : `${Math.round(kb)} K/s`
  }
  if (bytesPerSec < 1024 * 1024 * 1024) {
    const mb = bytesPerSec / (1024 * 1024)
    return `${mb.toFixed(1)} M/s`
  }
  const gb = bytesPerSec / (1024 * 1024 * 1024)
  return `${gb.toFixed(1)} G/s`
}

function updateRates(up: number, down: number) {
  uploadStr.value = formatSpeed(up)
  downloadStr.value = formatSpeed(down)
}

const unlisteners: UnlistenFn[] = []
let pollTimer: any = null

async function pollStats() {
  try {
    const s = await invokeSafe<any>('get_system_stats', undefined, undefined)
    if (s && (s.totalUploadRate !== undefined || s.total_upload_rate !== undefined)) {
      const up = Number(s.totalUploadRate ?? s.total_upload_rate ?? 0)
      const down = Number(s.totalDownloadRate ?? s.total_download_rate ?? 0)
      updateRates(up, down)
    } else {
      const list = await invokeSafe<any[]>('get_process_list', undefined, [])
      if (Array.isArray(list) && list.length > 0) {
        const up = list.reduce((sum, p) => sum + Number(p.uploadRate ?? p.upload_rate ?? 0), 0)
        const down = list.reduce((sum, p) => sum + Number(p.downloadRate ?? p.download_rate ?? 0), 0)
        updateRates(up, down)
      }
    }
  } catch {}
}

function handleMouseDown(e: MouseEvent) {
  // Left-click dragging
  if (e.button === 0 && !isClickThrough.value) {
    try {
      getCurrentWindow().startDragging().catch(() => {})
    } catch {}
  }
}

async function handleContextMenu(e: MouseEvent) {
  e.preventDefault()
  e.stopPropagation()
  await invokeSafe('show_floating_context_menu', undefined, undefined)
}

async function openDashboard() {
  await invokeSafe('show_main_window', undefined, undefined)
}

onMounted(async () => {
  // 1. Initial theme & settings load
  try {
    const cfg = await invokeSafe<Record<string, string>>('get_all_config', undefined, {})
    if (cfg) {
      if (cfg.theme) applyTheme(cfg.theme)
      if (cfg.floating_opacity) opacity.value = Number(cfg.floating_opacity) || 100
      if (cfg.floating_click_through === 'true') {
        isClickThrough.value = true
        await invokeSafe('set_floating_click_through', { enabled: true })
      }
    } else {
      const savedTheme = localStorage.getItem('app-theme') || 'light'
      applyTheme(savedTheme)
      const savedOp = localStorage.getItem('floating_opacity')
      if (savedOp) opacity.value = Number(savedOp) || 100
    }
  } catch {
    applyTheme('light')
  }

  // 2. Listen to theme & opacity & click-through sync events
  unlisteners.push(
    await listenSafe<string>('theme:sync', (theme) => {
      if (theme) applyTheme(theme)
    }),
  )

  unlisteners.push(
    await listenSafe<number>('floating:opacity', (val) => {
      if (typeof val === 'number') opacity.value = val
    }),
  )

  unlisteners.push(
    await listenSafe<boolean>('floating:click-through', (val) => {
      if (typeof val === 'boolean') isClickThrough.value = val
    }),
  )

  // 3. Start ETW monitor & stats polling
  await invokeSafe('start_monitoring')
  pollStats()
  pollTimer = setInterval(pollStats, 1000)

  // 4. Listen to system:stats and speed:update
  unlisteners.push(
    await listenSafe<any>('system:stats', (s) => {
      if (s) {
        const up = Number(s.totalUploadRate ?? s.total_upload_rate ?? 0)
        const down = Number(s.totalDownloadRate ?? s.total_download_rate ?? 0)
        updateRates(up, down)
      }
    }),
  )

  unlisteners.push(
    await listenSafe<any[]>('speed:update', (list) => {
      if (Array.isArray(list)) {
        const up = list.reduce((sum, p) => sum + Number(p.uploadRate ?? p.upload_rate ?? 0), 0)
        const down = list.reduce((sum, p) => sum + Number(p.downloadRate ?? p.download_rate ?? 0), 0)
        updateRates(up, down)
      }
    }),
  )
})

onBeforeUnmount(() => {
  if (pollTimer) clearInterval(pollTimer)
  unlistenAll(unlisteners)
})
</script>

<template>
  <div
    class="flex h-screen w-screen select-none items-center justify-between px-3 font-mono text-[13px] font-bold leading-none cursor-move transition-colors duration-200 overflow-hidden shadow-lg border-2 rounded-md"
    :style="{ opacity: opacity / 100 }"
    :class="[
      isDark
        ? 'bg-slate-900/95 text-slate-100 border-sky-400/80 shadow-[0_0_15px_rgba(56,189,248,0.3)]'
        : 'bg-slate-100/95 text-slate-900 border-blue-500/80 shadow-[0_2px_10px_rgba(59,130,246,0.25)]',
    ]"
    title="左键按住拖拽 · 右键设置透明度与穿透 · 双击打开主界面"
    @mousedown="handleMouseDown"
    @contextmenu.prevent="handleContextMenu"
    @dblclick="openDashboard"
  >
    <!-- Upload indicator -->
    <div class="inline-flex items-center gap-1.5 shrink-0 pointer-events-none">
      <ArrowUp class="h-4 w-4 stroke-[3] text-orange-500 dark:text-orange-400 shrink-0" />
      <span
        :class="isDark ? 'text-slate-100' : 'text-blue-950'"
        class="tracking-tight font-extrabold"
      >
        {{ uploadStr }}
      </span>
    </div>

    <!-- Download indicator -->
    <div class="inline-flex items-center gap-1.5 shrink-0 ml-2 pointer-events-none">
      <ArrowDown class="h-4 w-4 stroke-[3] text-emerald-600 dark:text-emerald-400 shrink-0" />
      <span
        :class="isDark ? 'text-slate-100' : 'text-blue-950'"
        class="tracking-tight font-extrabold"
      >
        {{ downloadStr }}
      </span>
    </div>
  </div>
</template>

<style>
html,
body,
#app {
  background: transparent !important;
  background-color: transparent !important;
  overflow: hidden !important;
  margin: 0 !important;
  padding: 0 !important;
  user-select: none !important;
}
</style>

<style>
html,
body,
#app {
  background: transparent !important;
  background-color: transparent !important;
  overflow: visible !important;
  margin: 0 !important;
  padding: 0 !important;
  user-select: none !important;
}
</style>
