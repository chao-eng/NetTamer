<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch, computed, nextTick } from 'vue'
import { Rive, RuntimeLoader, Layout, Fit, Alignment } from '@rive-app/canvas'
import riveWasmUrl from '@rive-app/canvas/rive.wasm?url'

// Set local WASM URL resolved by Vite
try {
  RuntimeLoader.setWasmUrl(riveWasmUrl)
} catch (e) {
  console.warn('Set WASM url warning:', e)
}

// Global cached Promise for car-types.riv to prevent concurrent fetch race conditions
let bufferPromise: Promise<ArrayBuffer> | null = null

function getRivBuffer(): Promise<ArrayBuffer> {
  if (!bufferPromise) {
    bufferPromise = fetch('/animations/car-types.riv')
      .then((res) => {
        if (!res.ok) throw new Error(`HTTP ${res.status}`)
        return res.arrayBuffer()
      })
      .catch((err) => {
        console.warn('Failed to fetch /animations/car-types.riv, trying relative fallback...', err)
        return fetch('./animations/car-types.riv')
          .then((res2) => res2.arrayBuffer())
          .catch((err2) => {
            bufferPromise = null // Allow retry if both fail
            throw err2
          })
      })
  }
  return bufferPromise
}

export interface RiveCarProps {
  carType?: number | string
  speedMbps?: number // in Mbps or speed factor
  direction?: 'left' | 'right' // 'left' = upload, 'right' = download
  label?: string
  subLabel?: string
  icon?: string
  width?: number
  height?: number
  showFlame?: boolean
  flameIntensity?: number // 0 to 1
}

const props = withDefaults(defineProps<RiveCarProps>(), {
  carType: 0,
  speedMbps: 0,
  direction: 'right',
  label: '',
  subLabel: '',
  icon: '',
  width: 170,
  height: 85,
  showFlame: true,
  flameIntensity: 0,
})

const CAR_NAMES = [
  'truck',
  'minivan',
  'cabriolet',
  'minitruck',
  'campervan',
  'supercar',
  'coupe',
  'van',
  'pickup',
  'roadster',
  'hatchback',
  'suv',
  'cuv',
  'sedan',
  'micro',
]

const canvasRef = ref<HTMLCanvasElement | null>(null)
let riveInstance: Rive | null = null
let smInputs: any[] = []
const isRiveLoaded = ref(false)

// Map car type string to index (0-14 from car-types.riv)
const CAR_TYPE_MAP: Record<string, number> = {
  truck: 0,
  minivan: 1,
  cabriolet: 2,
  minitruck: 3,
  campervan: 4,
  supercar: 5,
  coupe: 6,
  van: 7,
  pickup: 8,
  roadster: 9,
  hatchback: 10,
  suv: 11,
  cuv: 12,
  sedan: 13,
  micro: 14,
}

const numericCarType = computed<number>(() => {
  if (typeof props.carType === 'number') {
    return Math.max(0, Math.min(14, Math.floor(props.carType)))
  }
  const key = String(props.carType).toLowerCase().trim()
  return CAR_TYPE_MAP[key] ?? 0
})

const currentCarName = computed<string>(() => {
  return CAR_NAMES[numericCarType.value] || 'truck'
})

// Normalised speed for wheel rotation (0 - 100)
const normalizedRiveSpeed = computed(() => {
  if (props.speedMbps <= 0) return 0
  return Math.min(100, Math.max(10, Math.round(Math.log10(props.speedMbps + 1) * 35)))
})

// High speed threshold
const isHighSpeed = computed(() => props.speedMbps > 15 || props.flameIntensity > 0.4)
const isUltraSpeed = computed(() => props.speedMbps > 60 || props.flameIntensity > 0.8)

async function initRive() {
  if (!canvasRef.value) return

  if (riveInstance) {
    try {
      riveInstance.cleanup()
    } catch {
      // ignore
    }
    riveInstance = null
  }

  try {
    const carName = currentCarName.value // 'truck', 'minivan', 'cabriolet', 'minitruck', 'campervan', 'supercar', 'coupe', 'van', 'pickup', 'roadster', 'hatchback', 'suv', 'cuv', 'sedan', 'micro'

    // Empirically verified: 'src' with 'animations: carName' isolates that specific vehicle and completely eliminates the 4x4 matrix
    riveInstance = new Rive({
      src: '/animations/car-types.riv',
      canvas: canvasRef.value,
      artboard: 'car_types_artboard',
      animations: carName,
      autoplay: true,
      layout: new Layout({
        fit: Fit.Contain,
        alignment: Alignment.BottomCenter,
      }),
      onLoad: () => {
        isRiveLoaded.value = true
        riveInstance?.resizeDrawingSurfaceToCanvas()
      },
      onLoadError: (err) => {
        console.error('[RiveCar] Failed to load Rive animation:', err)
      },
    })
  } catch (err) {
    console.warn('Rive initialization failed:', err)
  }
}

watch(currentCarName, () => {
  initRive()
})

onMounted(async () => {
  await nextTick()
  initRive()
})

onBeforeUnmount(() => {
  if (riveInstance) {
    try {
      riveInstance.cleanup()
    } catch {
      // ignore
    }
    riveInstance = null
  }
})

// SVG fallback car icons for high-contrast / fallback rendering
const FALLBACK_CARS = [
  '🚗', // sedan
  '🚙', // suv
  '🚚', // truck
  '🚐', // minivan
  '🚌', // campervan
  '🏎️', // supercar
  '🚐', // van
  '🛻', // minitruck
]
</script>

<template>
  <div
    class="rive-car-container relative select-none"
    :class="[
      direction === 'left' ? 'is-upload' : 'is-download',
      { 'is-high-speed': isHighSpeed, 'is-ultra-speed': isUltraSpeed },
    ]"
    :style="{ width: `${width}px`, height: `${height}px` }"
  >
    <!-- Floating Tag / Bubble (Small process name & speed badge above the car) -->
    <div
      v-if="label || subLabel"
      class="car-tag absolute -top-7 left-1/2 z-50 flex -translate-x-1/2 items-center gap-1.5 whitespace-nowrap rounded-md border border-slate-700/90 bg-slate-950/95 px-2.5 py-0.5 shadow-2xl backdrop-blur-md"
    >
      <span v-if="icon" class="text-xs leading-none">{{ icon }}</span>
      <!-- Process Name (Crisp small font) -->
      <span class="text-xs font-mono font-bold tracking-tight text-white">{{ label }}</span>
      <!-- Speed Rate (Mini glowing pill) -->
      <span
        v-if="subLabel"
        class="rounded px-1.5 py-0.2 font-mono text-[10px] font-bold"
        :class="direction === 'left' ? 'bg-amber-500/20 text-amber-300 border border-amber-500/40' : 'bg-cyan-500/20 text-cyan-300 border border-cyan-500/40'"
      >
        {{ subLabel }}
      </span>
      <!-- Pointer Triangle -->
      <div class="pointer-triangle absolute left-1/2 -bottom-1 h-1.5 w-1.5 -translate-x-1/2 rotate-45 border-b border-r border-slate-700/90 bg-slate-950"></div>
    </div>

    <!-- Outer Direction Flip Box (Dedicated element so scaleX is NEVER overwritten by suspension keyframes) -->
    <div
      class="car-flip-box relative h-full w-full"
      :style="{ transform: direction === 'right' ? 'scaleX(-1)' : 'scaleX(1)' }"
    >
      <!-- Inner Suspension Bobbing Wrapper (Stays perfectly flat/level with road) -->
      <div
        class="car-suspension-layer relative h-full w-full"
        :class="speedMbps > 0 ? 'suspension-bobbing' : 'suspension-idle'"
      >
        <!-- Rive Canvas -->
        <canvas
          ref="canvasRef"
          :width="width"
          :height="height"
          class="relative z-10 block h-full w-full object-contain"
        />

        <!-- Fallback when canvas/rive is still loading -->
        <div
          v-if="!isRiveLoaded"
          class="absolute inset-0 z-0 flex items-center justify-center text-4xl opacity-90 transition-opacity"
        >
          <span>{{ FALLBACK_CARS[numericCarType] || '🚗' }}</span>
        </div>
      </div>
    </div>

    <!-- Realistic Ground Shadow (Tied to chassis contact) -->
    <div
      class="car-ground-shadow absolute -bottom-1 left-1/2 h-2.5 w-[75%] -translate-x-1/2 rounded-full"
      :class="speedMbps > 0 ? 'shadow-bob' : ''"
    ></div>
  </div>
</template>

<style scoped>
.rive-car-container {
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

/* Tag flip correction for upload */
.upload-tag {
  border-color: rgba(245, 158, 11, 0.4);
  box-shadow: 0 4px 14px rgba(245, 158, 11, 0.25);
}
.download-tag {
  border-color: rgba(16, 185, 129, 0.4);
  box-shadow: 0 4px 14px rgba(16, 185, 129, 0.25);
}

/* Suspension Bobbing & Idle Dynamics */
.suspension-idle {
  animation: carIdle 2.6s ease-in-out infinite;
}

.suspension-bobbing {
  animation: carDriveBob 0.35s ease-in-out infinite;
}

@keyframes carIdle {
  0%, 100% {
    transform: translateY(0px);
  }
  50% {
    transform: translateY(-2px);
  }
}

@keyframes carDriveBob {
  0%, 100% {
    transform: translateY(0px);
  }
  50% {
    transform: translateY(-1.5px);
  }
}

/* Ground Shadow */
.car-ground-shadow {
  background: radial-gradient(ellipse at center, rgba(0, 0, 0, 0.65) 0%, rgba(0, 0, 0, 0.25) 50%, rgba(0, 0, 0, 0) 75%);
  filter: blur(1.5px);
  pointer-events: none;
}

.shadow-bob {
  animation: shadowPulse 0.35s ease-in-out infinite;
}

@keyframes shadowPulse {
  0%, 100% {
    transform: translateX(-50%) scale(1);
    opacity: 0.85;
  }
  50% {
    transform: translateX(-50%) scale(0.92, 0.85);
    opacity: 0.6;
  }
}
</style>
