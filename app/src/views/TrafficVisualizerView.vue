<script setup lang="ts">
import { ref, reactive, computed, onMounted, onBeforeUnmount, watch } from 'vue'
import { useProcessStore } from '@/stores/processStore'
import RiveCar from '@/components/cars/RiveCar.vue'
import {
  ArrowUpRight,
  ArrowDownRight,
  Car,
  Activity,
  RotateCcw,
} from 'lucide-vue-next'

const processStore = useProcessStore()

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

function getCarTypeForProcess(name: string, rateMBps: number): number {
  if (!name) return 0
  const lower = name.toLowerCase()

  // 1. Semantic process name matching
  if (lower.includes('steam') || lower.includes('download') || lower.includes('torrent') || lower.includes('netdisk') || lower.includes('baidu')) {
    return 0 // 🚚 Truck / 重型卡车
  }
  if (lower.includes('game') || lower.includes('epic') || lower.includes('lol') || lower.includes('genshin') || lower.includes('valorant')) {
    return 5 // 🏎️ Supercar / 超跑
  }
  if (lower.includes('chrome') || lower.includes('edge') || lower.includes('browser') || lower.includes('firefox')) {
    return 6 // 🚗 Coupe / 轿跑
  }
  if (lower.includes('wechat') || lower.includes('qq') || lower.includes('chat') || lower.includes('discord') || lower.includes('telegram')) {
    return 14 // 🚗 Micro / 微型 Smart
  }
  if (lower.includes('music') || lower.includes('spotify') || lower.includes('cloudmusic')) {
    return 2 // 🚗 Cabriolet / 敞篷车
  }
  if (lower.includes('git') || lower.includes('node') || lower.includes('code') || lower.includes('cmd')) {
    return 8 // 🛻 Pickup / 皮卡
  }

  // 2. High-bandwidth mapping
  if (rateMBps > 30) return 5 // Supercar
  if (rateMBps > 10) return 11 // SUV
  if (rateMBps > 2) return 13 // Sedan

  // 3. Consistent Hash mapping
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

onMounted(() => {
  processStore.fetchList()
  if (!processStore.isMonitoring) {
    processStore.start()
  }
  syncAllLanes()
  syncTimer = window.setInterval(syncAllLanes, 1000)
})

onBeforeUnmount(() => {
  if (syncTimer) {
    clearInterval(syncTimer)
    syncTimer = null
  }
})
</script>

<template>
  <div class="traffic-visualizer flex h-full flex-col overflow-hidden bg-slate-950 text-slate-100">
    <!-- Top HUD Navigation Bar -->
    <header class="z-30 flex shrink-0 items-center justify-between border-b border-slate-800/80 bg-slate-900/90 px-6 py-3 backdrop-blur-md">
      <div class="flex items-center gap-3">
        <div class="flex h-10 w-10 items-center justify-center rounded-xl bg-gradient-to-tr from-cyan-500 to-indigo-600 shadow-lg shadow-cyan-500/20">
          <Car class="h-5 w-5 text-white" />
        </div>
        <div>
          <h1 class="text-base font-bold tracking-tight text-white flex items-center gap-2">
            流量进程高架公路
            <span class="inline-flex items-center rounded-full bg-cyan-500/10 px-2 py-0.5 text-xs font-semibold text-cyan-400 border border-cyan-500/20">
              实际进程速率 · 0 速率离道
            </span>
          </h1>
          <p class="text-xs text-slate-400">小车专属映射活跃进程实时流量 · 速率为 0 自动清场离道</p>
        </div>
      </div>

      <!-- Live Bandwidth & Control Tools -->
      <div class="flex items-center gap-4">
        <!-- Bandwidth Overview HUD (Strictly Real-time) -->
        <div class="flex items-center gap-4 text-xs font-mono">
          <div class="flex items-center gap-1.5 text-amber-400">
            <ArrowUpRight class="h-4 w-4" />
            <span class="text-slate-400">上行总计:</span>
            <span class="font-bold">{{ (processStore.totalUploadRate / 1024 / 1024).toFixed(2) }} MB/s</span>
          </div>
          <div class="flex items-center gap-1.5 text-emerald-400">
            <ArrowDownRight class="h-4 w-4" />
            <span class="text-slate-400">下行总计:</span>
            <span class="font-bold">{{ (processStore.totalDownloadRate / 1024 / 1024).toFixed(2) }} MB/s</span>
          </div>
        </div>
      </div>
    </header>

    <!-- Main Highway Stage (Static Stage + Pure Active Process Cars) -->
    <div class="highway-stage relative flex-1 select-none overflow-hidden">
      <!-- 1. [Static Sky & Moon & Stars] -->
      <div class="sky-backdrop absolute inset-0 z-0 pointer-events-none">
        <div class="sky-gradient absolute inset-0"></div>

        <!-- Static Crescent Moon -->
        <div class="moon-box absolute right-14 top-6 flex items-center opacity-90">
          <div class="moon-glow relative h-14 w-14 rounded-full bg-amber-100 shadow-[0_0_45px_rgba(254,240,138,0.4)]">
            <div class="moon-crater absolute left-2 top-3 h-3 w-3 rounded-full bg-amber-200/40"></div>
            <div class="moon-crater absolute right-3 bottom-2 h-2 w-2 rounded-full bg-amber-200/40"></div>
          </div>
        </div>

        <!-- Static Twinkling Stars -->
        <div class="stars-box absolute inset-0">
          <span class="star s-1"></span>
          <span class="star s-2"></span>
          <span class="star s-3"></span>
          <span class="star s-4"></span>
          <span class="star s-5"></span>
          <span class="star s-6"></span>
          <span class="star s-7"></span>
        </div>

        <!-- Static Distant Cloud Silhouettes -->
        <div class="clouds-backdrop absolute inset-x-0 top-3 h-16 opacity-30">
          <div class="cloud-shape c1"></div>
          <div class="cloud-shape c2"></div>
          <div class="cloud-shape c3"></div>
        </div>
      </div>

      <!-- 2. [Static Background City Skyline Silhouette] -->
      <div class="city-skyline absolute inset-x-0 bottom-24 z-1 pointer-events-none">
        <div class="skyline-buildings"></div>
      </div>

      <!-- Empty Track Peaceful Hint (When 0 active cars on the whole highway) -->
      <div
        v-if="activeCarsCount === 0"
        class="absolute left-1/2 top-1/3 z-25 -translate-x-1/2 -translate-y-1/2 flex items-center gap-2 rounded-full border border-slate-700/80 bg-slate-900/80 px-4 py-1.5 text-xs text-slate-300 backdrop-blur-md shadow-xl"
      >
        <span class="h-2 w-2 rounded-full bg-slate-500 animate-pulse"></span>
        <span>当前无活跃网络流量进程 · 公路通畅待机中</span>
      </div>

      <!-- 3. [Static 4-Track Elevated Highway Infrastructure] -->
      <div class="tracks-container relative z-10 flex h-full flex-col justify-end pb-12">
        
        <!-- Track 1: Upload Lane 1 (⬅️ Moves Right-to-Left) -->
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
              class="moving-car-runner drive-pass-left absolute"
              :style="{ bottom: '-16px', '--drive-dur': `${laneVehicles.up1.durationSec}s` }"
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
              />
            </div>
          </div>

          <div class="static-road-surface relative h-4 w-full bg-slate-800/95 border-t-2 border-amber-500/50 shadow-md">
            <div class="static-road-dashes"></div>
          </div>
        </div>

        <!-- Track 2: Upload Lane 2 (⬅️ Moves Right-to-Left) -->
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
              class="moving-car-runner drive-pass-left absolute"
              :style="{ bottom: '-16px', '--drive-dur': `${laneVehicles.up2.durationSec}s` }"
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
              />
            </div>
          </div>

          <div class="static-road-surface relative h-4 w-full bg-slate-800/95 border-t border-amber-500/40">
            <div class="static-road-dashes"></div>
          </div>
        </div>

        <!-- ╞══════ Central Green Isolation Belt & Crash Barrier ══════╡ -->
        <div class="central-barrier-strip relative z-15 flex h-7 w-full items-center justify-between bg-gradient-to-r from-emerald-950 via-teal-950 to-emerald-950 px-6 border-y-2 border-emerald-500/50 shadow-[0_0_15px_rgba(16,185,129,0.35)]">
          <div class="flex items-center gap-2 text-[10px] font-bold uppercase tracking-wider text-emerald-400">
            <span class="h-2 w-2 rounded-full bg-emerald-400 animate-pulse"></span>
            <span>⮜⮜ 上行车道 (UPLOAD LANE)</span>
          </div>

          <div class="flex items-center gap-10 opacity-80">
            <span class="h-3 w-2 rounded-sm bg-amber-400 shadow-[0_0_8px_#f59e0b]"></span>
            <span class="h-3 w-2 rounded-sm bg-emerald-400 shadow-[0_0_8px_#10b981]"></span>
            <span class="h-3 w-2 rounded-sm bg-amber-400 shadow-[0_0_8px_#f59e0b]"></span>
            <span class="h-3 w-2 rounded-sm bg-emerald-400 shadow-[0_0_8px_#10b981]"></span>
            <span class="h-3 w-2 rounded-sm bg-amber-400 shadow-[0_0_8px_#f59e0b]"></span>
            <span class="h-3 w-2 rounded-sm bg-emerald-400 shadow-[0_0_8px_#10b981]"></span>
          </div>

          <div class="flex items-center gap-2 text-[10px] font-bold uppercase tracking-wider text-cyan-400">
            <span>下行车道 (DOWNLOAD LANE) ⮞⮞</span>
            <span class="h-2 w-2 rounded-full bg-cyan-400 animate-pulse"></span>
          </div>
        </div>

        <!-- Track 3: Download Lane 1 (➡️ Moves Left-to-Right) -->
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
              class="moving-car-runner drive-pass-right absolute"
              :style="{ bottom: '-16px', '--drive-dur': `${laneVehicles.down1.durationSec}s` }"
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
              />
            </div>
          </div>

          <div class="static-road-surface relative h-4 w-full bg-slate-800/95 border-t-2 border-cyan-500/50">
            <div class="static-road-dashes"></div>
          </div>
        </div>

        <!-- Track 4: Download Lane 2 (➡️ Moves Left-to-Right) -->
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
              class="moving-car-runner drive-pass-right absolute"
              :style="{ bottom: '-16px', '--drive-dur': `${laneVehicles.down2.durationSec}s` }"
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
              />
            </div>
          </div>

          <div class="static-road-surface relative h-4 w-full bg-slate-800/95 border-t border-cyan-500/40">
            <div class="static-road-dashes"></div>
          </div>
        </div>

        <!-- 4. [Clean Solid Subgrade Base] -->
        <div class="subgrade-base relative h-6 w-full border-t-2 border-slate-800 bg-slate-950 shadow-inner"></div>
      </div>
    </div>

    <!-- Bottom Traffic Control Panel -->
    <footer class="z-30 border-t border-slate-800 bg-slate-900/95 px-6 py-2.5 backdrop-blur-md">
      <div class="flex items-center justify-between gap-4">
        <div class="flex items-center gap-4 text-xs font-semibold text-slate-300">
          <div class="flex items-center gap-2">
            <Activity class="h-4 w-4 text-cyan-400" />
            <span>实际运行状态：</span>
          </div>

          <span class="text-slate-400">
            当前活跃进程车辆: <span class="font-mono font-bold text-cyan-300">{{ activeCarsCount }}</span> 辆
          </span>
        </div>

        <div class="flex items-center gap-2 text-xs font-mono text-slate-400">
          <span class="h-2 w-2 rounded-full bg-emerald-400 animate-pulse"></span>
          <span>严格绑定实际进程流量 · 0 速率自动离场</span>
        </div>

        <!-- Refresh / Reset Button -->
        <button
          @click="processStore.fetchList(); syncAllLanes();"
          class="flex items-center gap-1.5 rounded-lg border border-slate-700 bg-slate-800 px-3 py-1 text-xs font-medium text-slate-300 hover:bg-slate-700 transition"
        >
          <RotateCcw class="h-3.5 w-3.5" />
          <span>刷新进程</span>
        </button>
      </div>
    </footer>
  </div>
</template>

<style scoped>
/* Sky Gradient */
.sky-gradient {
  background: linear-gradient(180deg, #070a17 0%, #0f152d 45%, #181d3f 80%, #0d1222 100%);
}

/* Stars */
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

/* Distant Cloud Silhouettes */
.cloud-shape {
  position: absolute;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 50px;
}
.c1 { width: 140px; height: 28px; left: 10%; top: 10px; }
.c2 { width: 220px; height: 35px; left: 45%; top: 20px; }
.c3 { width: 160px; height: 25px; right: 12%; top: 15px; }

/* Static City Skyline Silhouette */
.skyline-buildings {
  height: 120px;
  background: 
    linear-gradient(180deg, transparent 0%, rgba(15, 23, 42, 0.75) 100%),
    repeating-linear-gradient(
      90deg,
      rgba(20, 28, 55, 0.5) 0px,
      rgba(20, 28, 55, 0.5) 24px,
      transparent 24px,
      transparent 32px,
      rgba(30, 41, 75, 0.7) 32px,
      rgba(30, 41, 75, 0.7) 60px,
      transparent 60px,
      transparent 70px,
      rgba(25, 34, 62, 0.6) 70px,
      rgba(25, 34, 62, 0.6) 110px,
      transparent 110px,
      transparent 125px
    );
  mask-image: linear-gradient(to top, black 50%, transparent 100%);
  -webkit-mask-image: linear-gradient(to top, black 50%, transparent 100%);
}

/* Static Road Track Dashes */
.static-road-dashes {
  position: absolute;
  inset: 0;
  background-image: repeating-linear-gradient(
    90deg,
    rgba(255, 255, 255, 0.6) 0px,
    rgba(255, 255, 255, 0.6) 24px,
    transparent 24px,
    transparent 54px
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
    #94a3b8 0px,
    #94a3b8 3px,
    transparent 3px,
    transparent 45px
  );
  border-top: 1.5px solid #64748b;
}

/* Static Street Lamps with Warm Glow Cones */
.static-lamp {
  position: absolute;
  bottom: 0;
  width: 3px;
  height: 38px;
  background: #cbd5e1;
  border-radius: 2px;
}
.static-lamp::after {
  content: '';
  position: absolute;
  top: 0;
  left: -4px;
  width: 11px;
  height: 4px;
  background: #f8fafc;
  border-radius: 3px;
}
.lamp-glow {
  position: absolute;
  top: 2px;
  left: -20px;
  width: 44px;
  height: 36px;
  background: radial-gradient(ellipse at 50% 0%, rgba(254, 240, 138, 0.25) 0%, rgba(254, 240, 138, 0) 75%);
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
