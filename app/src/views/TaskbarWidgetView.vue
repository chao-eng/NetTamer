<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue'
import { listenSafe, type UnlistenFn, invokeSafe, unlistenAll } from '@/lib/ipc'

const uploadStr = ref('0.0 K/s')
const downloadStr = ref('0.0 K/s')

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

async function initStats() {
  try {
    const s = await invokeSafe<any>('get_system_stats', undefined, undefined)
    if (s) {
      const up = Number(s.totalUploadRate ?? s.total_upload_rate ?? 0)
      const down = Number(s.totalDownloadRate ?? s.total_download_rate ?? 0)
      updateRates(up, down)
    }
  } catch {}
}

onMounted(async () => {
  // Initial fetch
  initStats()

  // Listen to lightweight system:stats event
  unlisteners.push(
    await listenSafe<any>('system:stats', (s) => {
      if (s) {
        const up = Number(s.totalUploadRate ?? s.total_upload_rate ?? 0)
        const down = Number(s.totalDownloadRate ?? s.total_download_rate ?? 0)
        updateRates(up, down)
      }
    }),
  )
})

onBeforeUnmount(() => {
  unlistenAll(unlisteners)
})
</script>

<template>
  <div
    class="pointer-events-none flex h-full w-full select-none items-center justify-center gap-3 whitespace-nowrap flex-nowrap px-1 font-mono text-[12px] font-bold leading-none text-slate-100 bg-transparent cursor-default overflow-hidden drop-shadow-[0_1px_2px_rgba(0,0,0,0.95)]"
  >
    <div class="inline-flex items-center gap-1 shrink-0">
      <span class="text-sky-400 font-extrabold">↑:</span>
      <span class="text-slate-100 tracking-tight font-bold">{{ uploadStr }}</span>
    </div>
    <div class="inline-flex items-center gap-1 shrink-0">
      <span class="text-amber-400 font-extrabold">↓:</span>
      <span class="text-slate-100 tracking-tight font-bold">{{ downloadStr }}</span>
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
