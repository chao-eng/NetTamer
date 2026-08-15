<script setup lang="ts">
import { ref, reactive, computed, onMounted, onBeforeUnmount, onActivated, onDeactivated, watch } from 'vue'
import type { UnlistenFn } from '@/types'
import { useProcessStore } from '@/stores/processStore'
import { useSettingsStore } from '@/stores/settingsStore'
import RiveCar from '@/components/cars/RiveCar.vue'
import {
  ArrowUpRight,
  ArrowDownRight,
  Activity,
  RotateCcw,
  Maximize2,
  Minimize2,
} from 'lucide-vue-next'

const processStore = useProcessStore()
const settings = useSettingsStore()

function bytesToMBps(bytesPerSec: number): number {
  return bytesPerSec / (1024 * 1024)
}

function formatRate(mbps: number): string {
  if (mbps <= 0) return '0.0 KB/s'
  if (mbps < 1) return `${(mbps * 1024).toFixed(1)} KB/s`
  if (mbps >= 1024) return `${(mbps / 1024).toFixed(2)} GB/s`
  return `${mbps.toFixed(1)} MB/s`
}

function calculateDriveDuration(rateMBps: number): number {
  if (rateMBps <= 0.001) return 0
  const duration = 16.5 / (Math.pow(rateMBps, 0.45) + 0.6)
  return Math.max(1.3, Math.min(16, Number(duration.toFixed(2))))
}

interface LaneVehicle {
  id: string
  laneKey: 'up1' | 'up2' | 'down1' | 'down2'
  carType: number
  name: string
  icon: string
  rateMBps: number
  speedMbps: number
  durationSec: number
  isDriving: boolean
  rateFormatted: string
  runKey: number
}

// 4 Tracks vehicle models (strictly real-time dynamic)
const laneVehicles = reactive<Record<'up1' | 'up2' | 'down1' | 'down2', LaneVehicle>>({
  up1: {
    id: 'car-up1',
    laneKey: 'up1',
    carType: 0, // Truck
    name: '',
    icon: '📤',
    rateMBps: 0,
    speedMbps: 0,
    durationSec: 3.5,
    isDriving: false,
    rateFormatted: '0.0 KB/s',
    runKey: 1,
  },
  up2: {
    id: 'car-up2',
    laneKey: 'up2',
    carType: 13, // Sedan
    name: '',
    icon: '📤',
    rateMBps: 0,
    speedMbps: 0,
    durationSec: 5.0,
    isDriving: false,
    rateFormatted: '0.0 KB/s',
    runKey: 2,
  },
  down1: {
    id: 'car-down1',
    laneKey: 'down1',
    carType: 5, // Supercar
    name: '',
    icon: '📥',
    rateMBps: 0,
    speedMbps: 0,
    durationSec: 2.0,
    isDriving: false,
    rateFormatted: '0.0 KB/s',
    runKey: 3,
  },
  down2: {
    id: 'car-down2',
    laneKey: 'down2',
    carType: 11, // SUV
    name: '',
    icon: '📥',
    rateMBps: 0,
    speedMbps: 0,
    durationSec: 4.5,
    isDriving: false,
    rateFormatted: '0.0 KB/s',
    runKey: 4,
  },
})

// Per-lane readiness flag: RiveCar emits 'ready' when the car is fully
// loaded. The CSS drive animation only starts after this becomes true,
// so the car is always a single car (never emoji/matrix) when it enters
// the visible area.
const carReady = reactive<Record<'up1' | 'up2' | 'down1' | 'down2', boolean>>({
  up1: false,
  up2: false,
  down1: false,
  down2: false,
})

// Query strictly real-time ETW active process for each track
function fetchProcessForLane(laneKey: 'up1' | 'up2' | 'down1' | 'down2'): {
  rate: number
  name: string
  icon: string
} {
  if (laneKey === 'up1' || laneKey === 'up2') {
    const activeUploads = processStore.processes
      .filter((p) => p.uploadRate > 0)
      .sort((a, b) => b.uploadRate - a.uploadRate)

    const target = laneKey === 'up1' ? activeUploads[0] : activeUploads[1]
    if (target && target.uploadRate > 0) {
      return {
        rate: bytesToMBps(target.uploadRate),
        name: target.name,
        icon: '📤',
      }
    }
    return { rate: 0, name: '', icon: '' }
  } else {
    const activeDownloads = processStore.processes
      .filter((p) => p.downloadRate > 0)
      .sort((a, b) => b.downloadRate - a.downloadRate)

    const target = laneKey === 'down1' ? activeDownloads[0] : activeDownloads[1]
    if (target && target.downloadRate > 0) {
      return {
        rate: bytesToMBps(target.downloadRate),
        name: target.name,
        icon: '📥',
      }
    }
    return { rate: 0, name: '', icon: '' }
  }
}

function getCarTypeForProcess(name: string, _rateMBps: number): number {
  if (!name) return 0
  // Consistent hash: same process name always maps to the same car type (0-14)
  let hash = 0
  for (let i = 0; i < name.length; i++) {
    hash = name.charCodeAt(i) + ((hash << 5) - hash)
  }
  return Math.abs(hash) % 15
}

/**
 * Sync active driving state strictly to real process traffic:
 * If a process has rate > 0 and is not driving yet, start its run.
 * If rate is 0, let any current pass finish and do not spawn next run.
 */
function syncLane(laneKey: 'up1' | 'up2' | 'down1' | 'down2') {
  const v = laneVehicles[laneKey]
  const p = fetchProcessForLane(laneKey)

  if (p.rate > 0.001) {
    if (!v.isDriving) {
      // Start driving on the highway
      carReady[laneKey] = false
      v.rateMBps = p.rate
      v.speedMbps = p.rate * 8
      v.durationSec = calculateDriveDuration(p.rate)
      v.rateFormatted = formatRate(p.rate)
      v.name = p.name
      v.icon = p.icon
      v.carType = getCarTypeForProcess(p.name, p.rate)
      v.runKey++
      v.isDriving = true
    }
  } else {
    // 0 rate: if not currently mid-run, ensure isDriving is false
    if (!v.isDriving) {
      v.rateMBps = 0
      v.speedMbps = 0
      v.name = ''
    }
  }
}

function syncAllLanes() {
  syncLane('up1')
  syncLane('up2')
  syncLane('down1')
  syncLane('down2')
}

watch(() => processStore.processes, syncAllLanes, { deep: true })

/**
 * Single pass completed (Car is now 100% offscreen):
 * If process rate is still > 0, refresh speed and vehicle for the next run.
 * If process rate has dropped to 0, car exits the track completely.
 */
function handlePassCompleted(laneKey: 'up1' | 'up2' | 'down1' | 'down2') {
  const v = laneVehicles[laneKey]
  if (!v) return

  // Query latest rate for this process
  const p = fetchProcessForLane(laneKey)

  if (p.rate <= 0.001) {
    // Process stopped / rate is 0 -> Disappear from highway
    v.isDriving = false
    v.rateMBps = 0
    v.speedMbps = 0
    v.name = ''
    return
  }

  // Rate is still > 0 -> Ready for next pass
  v.isDriving = false
  v.carType = getCarTypeForProcess(p.name, p.rate)
  v.rateMBps = p.rate
  v.speedMbps = p.rate * 8
  v.durationSec = calculateDriveDuration(p.rate)
  v.rateFormatted = formatRate(p.rate)
  v.name = p.name
  v.icon = p.icon

  // Restart next run with brief natural re-entry stagger (200ms - 600ms)
  const staggerWait = 200 + Math.random() * 400
  setTimeout(() => {
    if (fetchProcessForLane(laneKey).rate > 0.001) {
      carReady[laneKey] = false
      v.runKey++
      v.isDriving = true
    }
  }, staggerWait)
}

// Total active cars currently driving
const activeCarsCount = computed(() => {
  let count = 0
  if (laneVehicles.up1.isDriving) count++
  if (laneVehicles.up2.isDriving) count++
  if (laneVehicles.down1.isDriving) count++
  if (laneVehicles.down2.isDriving) count++
  return count
})

let syncTimer: number | null = null
let unlisten: UnlistenFn[] = []

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape' && settings.isImmersiveWindow) {
    settings.toggleImmersiveWindow(false)
  }
}

onMounted(async () => {
  window.addEventListener('keydown', handleKeydown)
  await processStore.fetchList()
  unlisten = await processStore.bindEvents()
  if (!processStore.isMonitoring) {
    await processStore.start()
  }
  syncAllLanes()
  syncTimer = window.setInterval(syncAllLanes, 1000)
})

onActivated(() => {
  syncAllLanes()
})

onDeactivated(() => {
  settings.toggleImmersiveWindow(false)
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleKeydown)
  settings.toggleImmersiveWindow(false)
  if (syncTimer) {
    clearInterval(syncTimer)
    syncTimer = null
  }
  unlisten.forEach((fn) => fn())
  unlisten = []
})
</script>

<template>
  <div class="traffic-visualizer flex h-full w-full flex-col overflow-hidden bg-slate-100 text-slate-900 dark:bg-slate-950 dark:text-slate-100">
    <!-- Top HUD Navigation Bar (Hidden in Immersive Window mode) -->
    <header
      v-if="!settings.isImmersiveWindow"
      class="z-30 flex shrink-0 items-center justify-end border-b border-slate-200/80 bg-white/90 px-6 py-3 backdrop-blur-md dark:border-slate-800/80 dark:bg-slate-900/90"
    >
      <!-- Live Bandwidth & Control Tools -->
      <div class="flex items-center gap-4">
        <!-- Bandwidth Overview HUD (Strictly Real-time) -->
        <div class="flex items-center gap-4 text-xs font-mono">
          <div class="flex items-center gap-1.5 text-amber-600 dark:text-amber-400">
            <ArrowUpRight class="h-4 w-4" />
            <span class="text-slate-500 dark:text-slate-400">上行总计:</span>
            <span class="font-bold">{{ (processStore.totalUploadRate / 1024 / 1024).toFixed(2) }} MB/s</span>
          </div>
          <div class="flex items-center gap-1.5 text-emerald-600 dark:text-emerald-400">
            <ArrowDownRight class="h-4 w-4" />
            <span class="text-slate-500 dark:text-slate-400">下行总计:</span>
            <span class="font-bold">{{ (processStore.totalDownloadRate / 1024 / 1024).toFixed(2) }} MB/s</span>
          </div>
        </div>
      </div>
    </header>

    <!-- Main Highway Stage (Static Stage + Pure Active Process Cars) -->
    <div class="highway-stage relative flex-1 select-none overflow-hidden">
      <!-- Floating Exit Immersive Button (appears in immersive window mode) -->
      <button
        v-if="settings.isImmersiveWindow"
        @click="settings.toggleImmersiveWindow(false)"
        class="absolute right-4 bottom-4 z-40 flex items-center gap-1.5 rounded-lg border border-slate-300/80 bg-white/90 px-3 py-1.5 text-xs font-medium text-slate-700 backdrop-blur-md shadow-lg transition hover:bg-white dark:border-slate-700/80 dark:bg-slate-900/90 dark:text-slate-200 dark:hover:bg-slate-900"
      >
        <Minimize2 class="h-3.5 w-3.5 text-cyan-600 dark:text-cyan-400" />
        <span>退出铺满 (Esc)</span>
      </button>

      <!-- 1. [Sky & Celestial Body & Stars] -->
      <div class="sky-backdrop absolute inset-0 z-0 pointer-events-none">
        <div class="sky-gradient absolute inset-0"></div>

        <!-- Glowing Daylight Sun (light mode only) -->
        <div class="sun-box absolute right-16 top-6 flex items-center justify-center pointer-events-none dark:hidden">
          <div class="sun-outer-halo absolute h-28 w-28 rounded-full bg-amber-300/25 blur-2xl"></div>
          <div class="sun-inner-halo absolute h-18 w-18 rounded-full bg-amber-400/35 blur-lg"></div>
          <div class="sun-core relative h-12 w-12 rounded-full bg-gradient-to-tr from-amber-400 via-amber-300 to-yellow-100 shadow-[0_0_35px_rgba(251,191,36,0.65)]"></div>
        </div>
        <div class="moon-box absolute right-14 top-6 hidden items-center opacity-90 dark:flex pointer-events-none">
          <div class="moon-glow relative h-14 w-14 rounded-full bg-amber-100 shadow-[0_0_45px_rgba(254,240,138,0.4)]">
            <div class="moon-crater absolute left-2 top-3 h-3 w-3 rounded-full bg-amber-200/40"></div>
            <div class="moon-crater absolute right-3 bottom-2 h-2 w-2 rounded-full bg-amber-200/40"></div>
          </div>
        </div>

        <!-- Static Twinkling Stars (dark mode only) -->
        <div class="stars-box absolute inset-0 hidden dark:block">
          <span class="star s-1"></span>
          <span class="star s-2"></span>
          <span class="star s-3"></span>
          <span class="star s-4"></span>
          <span class="star s-5"></span>
          <span class="star s-6"></span>
          <span class="star s-7"></span>
        </div>
      </div>

      <!-- Empty Track Peaceful Hint (When 0 active cars on the whole highway) -->
      <div
        v-if="activeCarsCount === 0"
        class="absolute left-1/2 top-1/3 z-25 -translate-x-1/2 -translate-y-1/2 flex items-center gap-2 rounded-full border border-slate-300/80 bg-white/90 px-4 py-1.5 text-xs text-slate-700 backdrop-blur-md shadow-xl dark:border-slate-700/80 dark:bg-slate-900/80 dark:text-slate-300"
      >
        <span class="h-2 w-2 rounded-full bg-cyan-500 animate-pulse dark:bg-slate-500"></span>
        <span>当前无活跃网络流量进程 · 公路通畅待机中</span>
      </div>

      <!-- 3. [Static 4-Track Elevated Highway Infrastructure] -->
      <div class="tracks-container relative z-10 flex h-full flex-col justify-end pb-12">
        
        <!-- Track 1: Upload Lane 1 (Moves Right-to-Left) -->
        <div class="track-stage-lane relative flex flex-col justify-end" style="height: 105px;">
          <!-- Static Roadside Props (Street Lamps, Guardrail Posts) -->
          <div class="static-lane-props pointer-events-none absolute inset-x-0 bottom-2 h-14">
            <div class="static-lamp lamp-pos-1"><div class="lamp-glow"></div></div>
            <div class="static-lamp lamp-pos-2"><div class="lamp-glow"></div></div>
            <div class="static-lamp lamp-pos-3"><div class="lamp-glow"></div></div>
            <div class="static-lamp lamp-pos-4"><div class="lamp-glow"></div></div>
            <div class="static-guardrail-bar"></div>
          </div>

          <!-- Active Process Car on Track 1 (Hidden if rate is 0) -->
          <div class="car-runner-layer pointer-events-none absolute inset-0 z-20 flex items-end pb-0">
            <div
              v-if="laneVehicles.up1.isDriving"
              :key="`up1-${laneVehicles.up1.runKey}`"
              :class="['moving-car-runner absolute', carReady.up1 ? 'drive-pass-left' : '']"
              :style="{ bottom: '-16px', left: carReady.up1 ? undefined : 'calc(100% + 60px)', '--drive-dur': `${laneVehicles.up1.durationSec}s` }"
              @animationend="handlePassCompleted('up1')"
            >
              <RiveCar
                :car-type="laneVehicles.up1.carType"
                :speed-mbps="laneVehicles.up1.speedMbps"
                direction="left"
                :label="laneVehicles.up1.name"
                :sub-label="laneVehicles.up1.rateFormatted"
                :icon="laneVehicles.up1.icon"
                :width="160"
                :height="80"
                @ready="carReady.up1 = true"
              />
            </div>
          </div>

          <div class="static-road-surface relative h-4 w-full border-t-2 border-amber-500/80 shadow-md" style="background-color: var(--road-asphalt);">
            <div class="static-road-dashes"></div>
          </div>
        </div>

        <!-- Track 2: Upload Lane 2 (Moves Right-to-Left) -->
        <div class="track-stage-lane relative flex flex-col justify-end" style="height: 105px;">
          <div class="static-lane-props pointer-events-none absolute inset-x-0 bottom-2 h-14">
            <div class="static-lamp lamp-pos-alt-1"><div class="lamp-glow"></div></div>
            <div class="static-lamp lamp-pos-alt-2"><div class="lamp-glow"></div></div>
            <div class="static-guardrail-bar"></div>
          </div>

          <!-- Active Process Car on Track 2 (Hidden if rate is 0) -->
          <div class="car-runner-layer pointer-events-none absolute inset-0 z-20 flex items-end pb-0">
            <div
              v-if="laneVehicles.up2.isDriving"
              :key="`up2-${laneVehicles.up2.runKey}`"
              :class="['moving-car-runner absolute', carReady.up2 ? 'drive-pass-left' : '']"
              :style="{ bottom: '-16px', left: carReady.up2 ? undefined : 'calc(100% + 60px)', '--drive-dur': `${laneVehicles.up2.durationSec}s` }"
              @animationend="handlePassCompleted('up2')"
            >
              <RiveCar
                :car-type="laneVehicles.up2.carType"
                :speed-mbps="laneVehicles.up2.speedMbps"
                direction="left"
                :label="laneVehicles.up2.name"
                :sub-label="laneVehicles.up2.rateFormatted"
                :icon="laneVehicles.up2.icon"
                :width="160"
                :height="80"
                @ready="carReady.up2 = true"
              />
            </div>
          </div>

          <div class="static-road-surface relative h-4 w-full border-t border-amber-500/60 shadow-sm" style="background-color: var(--road-asphalt);">
            <div class="static-road-dashes"></div>
          </div>
        </div>

        <!-- Central Green Isolation Belt & Crash Barrier -->
        <div class="central-barrier-strip relative z-15 flex h-7 w-full items-center justify-between bg-gradient-to-r from-emerald-100/90 via-teal-100/90 to-emerald-100/90 px-6 border-y-2 border-emerald-600/40 shadow-[0_0_12px_rgba(16,185,129,0.18)] dark:from-emerald-950 dark:via-teal-950 dark:to-emerald-950 dark:border-emerald-500/50 dark:shadow-[0_0_15px_rgba(16,185,129,0.35)]">
          <div class="flex items-center gap-2 text-xs font-bold tracking-wide text-emerald-700 dark:text-emerald-400">
            <span class="h-2 w-2 rounded-full bg-emerald-500 animate-pulse dark:bg-emerald-400"></span>
            <span>上行流量 ←</span>
          </div>

          <div class="flex items-center gap-2 text-xs font-bold tracking-wide text-cyan-700 dark:text-cyan-400">
            <span>下行流量 →</span>
            <span class="h-2 w-2 rounded-full bg-cyan-500 animate-pulse dark:bg-cyan-400"></span>
          </div>
        </div>

        <!-- Track 3: Download Lane 1 (Moves Left-to-Right) -->
        <div class="track-stage-lane relative flex flex-col justify-end" style="height: 105px;">
          <div class="static-lane-props pointer-events-none absolute inset-x-0 bottom-2 h-14">
            <div class="static-lamp lamp-pos-1"><div class="lamp-glow"></div></div>
            <div class="static-lamp lamp-pos-2"><div class="lamp-glow"></div></div>
            <div class="static-lamp lamp-pos-3"><div class="lamp-glow"></div></div>
            <div class="static-lamp lamp-pos-4"><div class="lamp-glow"></div></div>
            <div class="static-guardrail-bar"></div>
          </div>

          <!-- Active Process Car on Track 3 (Hidden if rate is 0) -->
          <div class="car-runner-layer pointer-events-none absolute inset-0 z-20 flex items-end pb-0">
            <div
              v-if="laneVehicles.down1.isDriving"
              :key="`down1-${laneVehicles.down1.runKey}`"
              :class="['moving-car-runner absolute', carReady.down1 ? 'drive-pass-right' : '']"
              :style="{ bottom: '-16px', left: carReady.down1 ? undefined : '-190px', '--drive-dur': `${laneVehicles.down1.durationSec}s` }"
              @animationend="handlePassCompleted('down1')"
            >
              <RiveCar
                :car-type="laneVehicles.down1.carType"
                :speed-mbps="laneVehicles.down1.speedMbps"
                direction="right"
                :label="laneVehicles.down1.name"
                :sub-label="laneVehicles.down1.rateFormatted"
                :icon="laneVehicles.down1.icon"
                :width="160"
                :height="80"
                @ready="carReady.down1 = true"
              />
            </div>
          </div>

          <div class="static-road-surface relative h-4 w-full border-t-2 border-cyan-500/80 shadow-md" style="background-color: var(--road-asphalt);">
            <div class="static-road-dashes"></div>
          </div>
        </div>

        <!-- Track 4: Download Lane 2 (Moves Left-to-Right) -->
        <div class="track-stage-lane relative flex flex-col justify-end" style="height: 105px;">
          <div class="static-lane-props pointer-events-none absolute inset-x-0 bottom-2 h-14">
            <div class="static-lamp lamp-pos-alt-1"><div class="lamp-glow"></div></div>
            <div class="static-lamp lamp-pos-alt-2"><div class="lamp-glow"></div></div>
            <div class="static-guardrail-bar"></div>
          </div>

          <!-- Active Process Car on Track 4 (Hidden if rate is 0) -->
          <div class="car-runner-layer pointer-events-none absolute inset-0 z-20 flex items-end pb-0">
            <div
              v-if="laneVehicles.down2.isDriving"
              :key="`down2-${laneVehicles.down2.runKey}`"
              :class="['moving-car-runner absolute', carReady.down2 ? 'drive-pass-right' : '']"
              :style="{ bottom: '-16px', left: carReady.down2 ? undefined : '-190px', '--drive-dur': `${laneVehicles.down2.durationSec}s` }"
              @animationend="handlePassCompleted('down2')"
            >
              <RiveCar
                :car-type="laneVehicles.down2.carType"
                :speed-mbps="laneVehicles.down2.speedMbps"
                direction="right"
                :label="laneVehicles.down2.name"
                :sub-label="laneVehicles.down2.rateFormatted"
                :icon="laneVehicles.down2.icon"
                :width="160"
                :height="80"
                @ready="carReady.down2 = true"
              />
            </div>
          </div>

          <div class="static-road-surface relative h-4 w-full border-t border-cyan-500/60 shadow-sm" style="background-color: var(--road-asphalt);">
            <div class="static-road-dashes"></div>
          </div>
        </div>

        <!-- 4. [Clean Solid Subgrade Base] -->
        <div class="subgrade-base relative h-7 w-full border-t-2 border-slate-300/80 bg-gradient-to-b from-slate-200 to-slate-100 shadow-inner dark:border-slate-800 dark:from-slate-900 dark:to-slate-950"></div>
      </div>
    </div>

    <!-- Bottom Traffic Control Panel (Hidden in Immersive Window mode) -->
    <footer
      v-if="!settings.isImmersiveWindow"
      class="z-30 border-t border-slate-200 bg-white/95 px-6 py-2.5 backdrop-blur-md dark:border-slate-800 dark:bg-slate-900/95"
    >
      <div class="flex items-center justify-between gap-4">
        <div class="flex items-center gap-2 text-xs text-slate-600 dark:text-slate-400">
          <Activity class="h-3.5 w-3.5 text-cyan-600 dark:text-cyan-400" />
          <span>
            当前活跃进程车辆: <span class="font-mono font-bold text-cyan-600 dark:text-cyan-300">{{ activeCarsCount }}</span> 辆
          </span>
        </div>

        <!-- Control Tools -->
        <div class="flex items-center gap-2">
          <!-- Full Window View Toggle Button -->
          <button
            @click="settings.toggleImmersiveWindow(true)"
            class="flex items-center gap-1.5 rounded-lg border border-slate-300 bg-slate-100 px-3 py-1 text-xs font-medium text-slate-700 hover:bg-slate-200 transition dark:border-slate-700 dark:bg-slate-800 dark:text-slate-300 dark:hover:bg-slate-700"
          >
            <Maximize2 class="h-3.5 w-3.5 text-cyan-600 dark:text-cyan-400" />
            <span>铺满窗口</span>
          </button>

          <!-- Refresh / Reset Button -->
          <button
            @click="processStore.fetchList(); syncAllLanes();"
            class="flex items-center gap-1.5 rounded-lg border border-slate-300 bg-slate-100 px-3 py-1 text-xs font-medium text-slate-700 hover:bg-slate-200 transition dark:border-slate-700 dark:bg-slate-800 dark:text-slate-300 dark:hover:bg-slate-700"
          >
            <RotateCcw class="h-3.5 w-3.5" />
            <span>刷新进程</span>
          </button>
        </div>
      </div>
    </footer>
  </div>
</template>

<!-- Non-scoped: defines CSS variables for light/dark theme -->
<style>
.traffic-visualizer {
  --sky-c1: #cde8fe;
  --sky-c2: #e0f2fe;
  --sky-c3: #eef2ff;
  --sky-c4: #f8fafc;
  --road-asphalt: #334155;
  --road-dash: rgba(255, 255, 255, 0.75);
  --guardrail-bar: #94a3b8;
  --guardrail-border: #64748b;
  --lamp-pole: #64748b;
  --lamp-head: #1e293b;
  --lamp-glow-bg: radial-gradient(ellipse at 50% 0%, rgba(254, 240, 138, 0.3) 0%, rgba(254, 240, 138, 0) 75%);
}
.dark .traffic-visualizer {
  --sky-c1: #060913;
  --sky-c2: #0b1124;
  --sky-c3: #111836;
  --sky-c4: #0a0f1d;
  --road-asphalt: #1e293b;
  --road-dash: rgba(255, 255, 255, 0.65);
  --guardrail-bar: #64748b;
  --guardrail-border: #475569;
  --lamp-pole: #cbd5e1;
  --lamp-head: #f8fafc;
  --lamp-glow-bg: radial-gradient(ellipse at 50% 0%, rgba(254, 240, 138, 0.25) 0%, rgba(254, 240, 138, 0) 75%);
}
</style>

<style scoped>
/* Sky Gradient */
.sky-gradient {
  background: linear-gradient(180deg, var(--sky-c1) 0%, var(--sky-c2) 45%, var(--sky-c3) 80%, var(--sky-c4) 100%);
}

/* Stars (dark mode only) */
.star {
  position: absolute;
  width: 2px;
  height: 2px;
  background: #fff;
  border-radius: 50%;
  box-shadow: 0 0 4px #fff;
  animation: starTwinkle 2s infinite ease-in-out alternate;
}
.s-1 { top: 12%; left: 15%; animation-delay: 0.1s; }
.s-2 { top: 22%; left: 35%; animation-delay: 0.7s; }
.s-3 { top: 8%; left: 55%; animation-delay: 1.2s; }
.s-4 { top: 18%; left: 75%; animation-delay: 0.4s; }
.s-5 { top: 28%; left: 88%; animation-delay: 1.5s; }
.s-6 { top: 10%; left: 42%; animation-delay: 0.9s; }
.s-7 { top: 32%; left: 20%; animation-delay: 1.8s; }

@keyframes starTwinkle {
  0% { opacity: 0.25; transform: scale(0.85); }
  100% { opacity: 1; transform: scale(1.3); }
}

/* Static Road Track Dashes (Thin elegant centerline) */
.static-road-dashes {
  position: absolute;
  top: 50%;
  left: 0;
  right: 0;
  height: 2px;
  transform: translateY(-50%);
  background-image: repeating-linear-gradient(
    90deg,
    var(--road-dash) 0px,
    var(--road-dash) 22px,
    transparent 22px,
    transparent 44px
  );
}

/* Static Guardrail Posts & Bars */
.static-guardrail-bar {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 10px;
  background-image: repeating-linear-gradient(
    90deg,
    var(--guardrail-bar) 0px,
    var(--guardrail-bar) 2.5px,
    transparent 2.5px,
    transparent 48px
  );
  border-top: 1.5px solid var(--guardrail-border);
}

/* Static Street Lamps with Warm Glow Cones */
.static-lamp {
  position: absolute;
  bottom: 0;
  width: 3px;
  height: 38px;
  background: var(--lamp-pole);
  border-radius: 2px;
}
.static-lamp::after {
  content: '';
  position: absolute;
  top: 0;
  left: -4px;
  width: 11px;
  height: 4px;
  background: var(--lamp-head);
  border-radius: 3px;
}
.lamp-glow {
  position: absolute;
  top: 2px;
  left: -20px;
  width: 44px;
  height: 36px;
  background: var(--lamp-glow-bg);
  pointer-events: none;
}

.lamp-pos-1 { left: 12%; }
.lamp-pos-2 { left: 38%; }
.lamp-pos-3 { left: 64%; }
.lamp-pos-4 { left: 88%; }

.lamp-pos-alt-1 { left: 24%; }
.lamp-pos-alt-2 { left: 74%; }

/* Single Full-Pass Screen-Crossing Animations (Clean Offscreen-to-Offscreen) */
.moving-car-runner {
  will-change: left;
  pointer-events: auto;
}

/* Pause vehicle movement when mouse hovers over it */
.moving-car-runner:hover {
  animation-play-state: paused;
}

/* Download: Single Pass from Left (-190px) to Right (100% + 60px) */
.drive-pass-right {
  animation: drivePassRight var(--drive-dur, 3.5s) linear forwards;
}

@keyframes drivePassRight {
  0% {
    left: -190px;
  }
  100% {
    left: calc(100% + 60px);
  }
}

/* Upload: Single Pass from Right (100% + 60px) to Left (-190px) */
.drive-pass-left {
  animation: drivePassLeft var(--drive-dur, 3.5s) linear forwards;
}

@keyframes drivePassLeft {
  0% {
    left: calc(100% + 60px);
  }
  100% {
    left: -190px;
  }
}
</style>
