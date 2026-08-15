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

const emit = defineEmits<{ ready: [] }>()

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

const canvasRef = ref<HTMLCanvasElement | null>(null)
let riveInstance: Rive | null = null

// State machine input references for real-time control
let speedInput: any = null
const triggerInputs = new Map<string, any>()

const isRiveLoaded = ref(false)
// Whether the target car has been selected (hides the 4x4 matrix flash)
const isCarSelected = ref(false)

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

// Tiered speed for wheel rotation:
//   < 512 KB/s (4 Mbps)  -> 1 (slow)
//   < 2 MB/s   (16 Mbps) -> 2 (medium)
//   >= 2 MB/s            -> 3 (fast)
const normalizedRiveSpeed = computed(() => {
  if (props.speedMbps <= 0) return 0
  if (props.speedMbps < 4) return 1
  if (props.speedMbps < 16) return 2
  return 3
})

// High speed threshold
const isHighSpeed = computed(() => props.speedMbps > 15 || props.flameIntensity > 0.4)
const isUltraSpeed = computed(() => props.speedMbps > 60 || props.flameIntensity > 0.8)

// Track whether we successfully entered the state-machine mode (with speed binding)
// or fell back to the simpler animations mode (single car, no speed control).
let useStateMachine = true
let fallbackTimer: ReturnType<typeof setTimeout> | null = null
let triggerRetryTimers: ReturnType<typeof setTimeout>[] = []
let triggerRAF: number | null = null

// Track the currently selected car name (updated via onStateChange)
let selectedCarName = ''
// Track whether move_wheels is active (for trigger retry logic)
let moveWheelsActive = false

// Generation counter to handle car type changes during async scan
let initGeneration = 0

// Cache: maps grid position "col,row" -> car name.
// Once we've scanned the 4x4 matrix once, we never need to scan again.
const gridCarMap = new Map<string, string>()

function clearAllTimers() {
  if (fallbackTimer) { clearTimeout(fallbackTimer); fallbackTimer = null }
  triggerRetryTimers.forEach(t => clearTimeout(t))
  triggerRetryTimers = []
  if (triggerRAF !== null) { cancelAnimationFrame(triggerRAF); triggerRAF = null }
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

/**
 * Setup state machine input references.
 * Must be called after onLoad and after each instance recreation.
 */
function setupInputs() {
  if (!riveInstance) return

  const inputs = riveInstance.stateMachineInputs('Machine') || []

  speedInput = inputs.find((i: any) => i.name === 'speed') || null
  triggerInputs.clear()
  for (const input of inputs) {
    if (typeof input.fire === 'function') {
      triggerInputs.set(input.name.toLowerCase(), input)
    }
  }
}

/**
 * Handle state machine state changes.
 * Tracks the currently selected car and triggers onCarSelected when target is found.
 */
function handleStateChange(event: any) {
  const states: string[] = event?.data || []

  moveWheelsActive = states.includes('move_wheels')

  for (const s of states) {
    if (CAR_NAMES.includes(s)) {
      selectedCarName = s
      if (s === currentCarName.value && !isCarSelected.value) {
        onCarSelected(s)
      }
    }
  }
}

/**
 * Called when the target car is successfully selected.
 * Reveals canvas, fires trigger for wheel rotation, sets speed.
 *
 * CRITICAL: speed must be 0 when clicking (so the state machine can select
 * the car). After selection, set speed > 0 to start move_wheels.
 * The trigger is fired as a backup to start wheel rotation.
 */
function onCarSelected(carName: string) {
  if (isCarSelected.value) return
  isCarSelected.value = true
  clearAllTimers()

  // Reset moveWheelsActive - it may be stale from the Overview state.
  moveWheelsActive = false

  // Set speed IMMEDIATELY - in the car's detail view, speed > 0 may
  // automatically transition to move_wheels (wheel rotation).
  if (speedInput) {
    speedInput.value = normalizedRiveSpeed.value || 5
  }

  // Helper: fire trigger + re-set speed
  const fireTriggerWithSpeed = () => {
    const trigger = triggerInputs.get(carName.toLowerCase())
    if (trigger) {
      trigger.fire()
    }
    if (speedInput) {
      speedInput.value = normalizedRiveSpeed.value || 5
    }
  }

  // Fire 1: Next animation frame (after current advance cycle completes)
  triggerRAF = requestAnimationFrame(() => {
    triggerRAF = null
    if (!isCarSelected.value) return
    fireTriggerWithSpeed()
  })

  // Fire 2: 500ms - if move_wheels still not active, retry
  triggerRetryTimers.push(setTimeout(() => {
    if (!isCarSelected.value || moveWheelsActive) return
    fireTriggerWithSpeed()
  }, 500))

  // Fire 3: 1000ms - last resort
  triggerRetryTimers.push(setTimeout(() => {
    if (!isCarSelected.value || moveWheelsActive) return
    fireTriggerWithSpeed()
  }, 1000))
}

/**
 * Simulate a mouse click on the Rive canvas at the given relative coordinates.
 *
 * CRITICAL: Rive SDK v2.40 registers listeners for 'mousedown'/'mouseup'
 * (NOT 'pointerdown'/'pointerup'). See rive.js lines 2618-2619.
 */
function simulateMouseClick(
  canvas: HTMLCanvasElement,
  xPercent: number,
  yPercent: number,
) {
  const rect = canvas.getBoundingClientRect()
  const x = rect.left + xPercent * rect.width
  const y = rect.top + yPercent * rect.height

  const opts: MouseEventInit = {
    bubbles: true,
    cancelable: true,
    clientX: x,
    clientY: y,
  }

  canvas.dispatchEvent(new MouseEvent('mousedown', opts))
  setTimeout(() => {
    canvas.dispatchEvent(new MouseEvent('mouseup', opts))
  }, 30)
}

/**
 * Generate 16 grid positions (4x4) in canvas percentage coordinates.
 * Accounts for Fit.Contain + BottomCenter alignment.
 */
function generateGridPositions(
  canvas: HTMLCanvasElement,
  abWidth: number,
  abHeight: number,
): { x: number; y: number; label: string }[] {
  const rect = canvas.getBoundingClientRect()
  const scale = Math.min(rect.width / abWidth, rect.height / abHeight)
  const renderedW = abWidth * scale
  const renderedH = abHeight * scale
  const offsetX = (rect.width - renderedW) / 2 // CenterX
  const offsetY = rect.height - renderedH       // BottomY

  const positions: { x: number; y: number; label: string }[] = []
  for (let row = 0; row < 4; row++) {
    for (let col = 0; col < 4; col++) {
      const artX = (col + 0.5) * (abWidth / 4)
      const artY = (row + 0.5) * (abHeight / 4)
      const canvasX = offsetX + artX * scale
      const canvasY = offsetY + artY * scale
      positions.push({
        x: canvasX / rect.width,
        y: canvasY / rect.height,
        label: 'grid(' + col + ',' + row + ')',
      })
    }
  }
  return positions
}

/**
 * Recreate the Rive instance from scratch using the cached buffer.
 *
 * This is used instead of riveInstance.reset() because reset() does NOT
 * properly restore click handling. After reset(), canvas clicks no longer
 * select cars (state changes show ['move_wheels', 'idle'] instead of car names).
 *
 * cleanup() + new Rive() goes through the full initialization flow, including
 * event listener registration and artboard hit-testing setup, which ensures
 * clicks work reliably.
 */
async function recreateRiveInstance(myGen: number): Promise<void> {
  // Cleanup existing instance (removes event listeners + state machine)
  if (riveInstance) {
    try { riveInstance.cleanup() } catch { /* ignore */ }
    riveInstance = null
  }

  clearAllTimers()
  speedInput = null
  triggerInputs.clear()
  selectedCarName = ''
  moveWheelsActive = false

  const buffer = await getRivBuffer()
  if (myGen !== initGeneration || !canvasRef.value) return

  return new Promise((resolve) => {
    riveInstance = new Rive({
      buffer: buffer.slice(0),
      canvas: canvasRef.value,
      artboard: 'car_types_artboard',
      stateMachines: 'Machine',
      autoplay: true,
      layout: new Layout({
        fit: Fit.Contain,
        alignment: Alignment.BottomCenter,
      }),
      onLoad: () => {
        if (myGen !== initGeneration) { resolve(); return }
        riveInstance?.resizeDrawingSurfaceToCanvas()
        setupInputs()
        isRiveLoaded.value = true
        resolve()
      },
      onStateChange: handleStateChange,
      onLoadError: () => { resolve() },
    })
  })
}

/**
 * Click scan: find the target car by clicking grid positions.
 *
 * KEY INSIGHT: riveInstance.reset() does NOT restore click handling.
 * After reset(), clicks produce ['move_wheels', 'idle'] instead of car names.
 * The fix: use cleanup() + new Rive() (via recreateRiveInstance) for each
 * position. This is slower but actually works.
 *
 * OPTIMIZATION: Results are cached in gridCarMap. Once we've found a car's
 * position, subsequent lookups skip the scan entirely.
 */
async function startClickScan(targetCarName: string, myGen: number) {
  if (!canvasRef.value) return

  // Check cache: do we already know this car's position?
  for (const [key, carName] of gridCarMap) {
    if (carName === targetCarName) {
      const [col, row] = key.split(',').map(Number)
      await recreateRiveInstance(myGen)
      if (myGen !== initGeneration || isCarSelected.value) return
      await sleep(150)

      selectedCarName = ''
      const riveAny = riveInstance as any
      const abWidth = riveAny?.artboard?.width || 5000
      const abHeight = riveAny?.artboard?.height || 4778
      const positions = generateGridPositions(canvasRef.value, abWidth, abHeight)
      const idx = row * 4 + col
      if (idx < positions.length) {
        const pos = positions[idx]
        simulateMouseClick(canvasRef.value, pos.x, pos.y)
        await sleep(200)
        if (myGen !== initGeneration || isCarSelected.value) return

        if (selectedCarName === targetCarName) {
          onCarSelected(targetCarName)
          return
        }
        // Cache stale - fall through to full scan
        gridCarMap.delete(key)
      }
      break
    }
  }

  // Full scan: try all 16 positions
  for (let row = 0; row < 4; row++) {
    for (let col = 0; col < 4; col++) {
      if (myGen !== initGeneration || isCarSelected.value) return

      const key = col + ',' + row
      // Skip positions we already know aren't the target
      if (gridCarMap.has(key) && gridCarMap.get(key) !== targetCarName) {
        continue
      }

      // For first position, the instance from initRive() is still fresh.
      // For subsequent positions, recreate the instance.
      if (row > 0 || col > 0) {
        await recreateRiveInstance(myGen)
        if (myGen !== initGeneration || isCarSelected.value) return
        await sleep(150)
      }

      if (!canvasRef.value || !riveInstance) continue

      const riveAny = riveInstance as any
      const abWidth = riveAny?.artboard?.width || 5000
      const abHeight = riveAny?.artboard?.height || 4778
      const positions = generateGridPositions(canvasRef.value, abWidth, abHeight)
      const idx = row * 4 + col
      if (idx >= positions.length) continue

      const pos = positions[idx]

      selectedCarName = ''
      simulateMouseClick(canvasRef.value, pos.x, pos.y)

      await sleep(200)
      if (myGen !== initGeneration || isCarSelected.value) return

      // Cache the result
      if (selectedCarName) {
        gridCarMap.set(key, selectedCarName)
      }

      if (selectedCarName === targetCarName) {
        onCarSelected(targetCarName)
        return
      }
    }
  }

  console.warn('[RiveCar] "' + targetCarName + '" not found in 16 positions. Falling back to animations mode.')
  initRiveWithAnimations()
}

/**
 * Initialise Rive with the 'Machine' state machine.
 *
 * STRATEGY:
 * 1. Load with buffer (cached) + autoplay: true + stateMachines: 'Machine'.
 * 2. Canvas hidden (opacity 0) - user sees emoji fallback, not matrix.
 * 3. Skip trigger (doesn't select cars) and go straight to click scan.
 * 4. Click scan uses cleanup() + new Rive() between positions (NOT reset()).
 * 5. Results cached in gridCarMap for instant lookup on subsequent cars.
 * 6. If no match after 16 positions - fallback to animations mode.
 */
async function initRive() {
  if (!canvasRef.value) return

  // Cleanup existing instance
  if (riveInstance) {
    try { riveInstance.cleanup() } catch { /* ignore */ }
    riveInstance = null
  }

  clearAllTimers()

  // Reset state
  initGeneration++
  const myGen = initGeneration
  speedInput = null
  triggerInputs.clear()
  selectedCarName = ''
  isRiveLoaded.value = false
  isCarSelected.value = false
  useStateMachine = true

  const carName = currentCarName.value

  try {
    const buffer = await getRivBuffer()
    if (myGen !== initGeneration) return

    riveInstance = new Rive({
      buffer: buffer.slice(0),
      canvas: canvasRef.value,
      artboard: 'car_types_artboard',
      stateMachines: 'Machine',
      autoplay: true,
      layout: new Layout({
        fit: Fit.Contain,
        alignment: Alignment.BottomCenter,
      }),
      onLoad: () => {
        if (myGen !== initGeneration) return
        riveInstance?.resizeDrawingSurfaceToCanvas()
        setupInputs()
        isRiveLoaded.value = true

        // CRITICAL: Do NOT set speed here!
        // Speed must be 0 when clicking, otherwise the state machine enters
        // move_wheels in the Overview context and car selection fails.
        // Speed is set in onCarSelected() AFTER the car is selected.

        // Skip trigger (doesn't select cars) and go straight to click scan
        startClickScan(carName, myGen)
      },
      onStateChange: handleStateChange,
      onLoadError: (err) => {
        console.error('[RiveCar] Load error:', err)
        initRiveWithAnimations()
      },
    })

    // Fallback: if car not selected after 15s, use animations mode
    // (increased from 6s because recreateRiveInstance is slower than reset)
    fallbackTimer = setTimeout(() => {
      if (myGen !== initGeneration) return
      if (!isCarSelected.value) {
        console.warn('[RiveCar] Car not selected after 15s. Falling back to animations mode.')
        initRiveWithAnimations()
      }
    }, 15000)
  } catch (err) {
    console.warn('Rive initialization failed:', err)
    initRiveWithAnimations()
  }
}

/**
 * Fallback: load with animations: carName (directly plays the single-car animation).
 * This bypasses the state machine entirely - no 4x4 matrix, but also no speed binding.
 */
function initRiveWithAnimations() {
  if (!canvasRef.value) return

  clearAllTimers()

  if (riveInstance) {
    try { riveInstance.cleanup() } catch { /* ignore */ }
    riveInstance = null
  }

  // In animations mode we can't bind speed
  speedInput = null
  triggerInputs.clear()
  useStateMachine = false

  try {
    const carName = currentCarName.value

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
        riveInstance?.resizeDrawingSurfaceToCanvas()
        isRiveLoaded.value = true
        isCarSelected.value = true
      },
      onLoadError: (err) => {
        console.error('[RiveCar] Fallback animations load error:', err)
        isCarSelected.value = true
      },
    })
  } catch (err) {
    console.warn('[RiveCar] Fallback animations failed:', err)
    isCarSelected.value = true
  }
}

// Car type change: always re-init so the new car is selected from the matrix.
watch(currentCarName, (newCarName, oldCarName) => {
  if (newCarName === oldCarName) return
  if (useStateMachine) {
    initRive()
  } else {
    initRiveWithAnimations()
  }
})

// Real-time speed binding: update the Rive 'speed' input whenever
// the normalized speed changes, so wheels spin at the correct rate.
watch(normalizedRiveSpeed, (newSpeed) => {
  if (speedInput && isRiveLoaded.value) {
    speedInput.value = newSpeed
  }
})

// Emit 'ready' when the car is fully selected.
watch(isCarSelected, (val) => {
  if (val) emit('ready')
})

onMounted(async () => {
  await nextTick()
  initRive()
})

onBeforeUnmount(() => {
  clearAllTimers()
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
  '\u{1F697}', // car
  '\u{1F699}', // suv
  '\u{1F69A}', // truck
  '\u{1F690}', // minivan
  '\u{1F68C}', // campervan
  '\u{1F3CE}\uFE0F', // supercar
  '\u{1F690}', // van
  '\u{1F6FB}', // minitruck
]
</script>

<template>
  <div
    class="rive-car-container relative select-none"
    :class="[
      direction === 'left' ? 'is-upload' : 'is-download',
      { 'is-high-speed': isHighSpeed, 'is-ultra-speed': isUltraSpeed },
    ]"
    :style="{ width: width + 'px', height: height + 'px' }"
  >
    <!-- Floating Tag / Bubble -->
    <div
      v-if="label || subLabel"
      class="car-tag absolute -top-7 left-1/2 z-50 flex -translate-x-1/2 items-center gap-1.5 whitespace-nowrap rounded-md border border-slate-700/90 bg-slate-950/95 px-2.5 py-0.5 shadow-2xl backdrop-blur-md"
    >
      <span v-if="icon" class="text-xs leading-none">{{ icon }}</span>
      <span class="text-xs font-mono font-bold tracking-tight text-white">{{ label }}</span>
      <span
        v-if="subLabel"
        class="rounded px-1.5 py-0.2 font-mono text-[10px] font-bold"
        :class="direction === 'left' ? 'bg-amber-500/20 text-amber-300 border border-amber-500/40' : 'bg-cyan-500/20 text-cyan-300 border border-cyan-500/40'"
      >
        {{ subLabel }}
      </span>
      <div class="pointer-triangle absolute left-1/2 -bottom-1 h-1.5 w-1.5 -translate-x-1/2 rotate-45 border-b border-r border-slate-700/90 bg-slate-950"></div>
    </div>

    <!-- Outer Direction Flip Box -->
    <div
      class="car-flip-box relative h-full w-full"
      :style="{ transform: direction === 'right' ? 'scaleX(-1)' : 'scaleX(1)' }"
    >
      <!-- Inner Suspension Bobbing Wrapper -->
      <div
        class="car-suspension-layer relative h-full w-full"
        :class="speedMbps > 0 ? 'suspension-bobbing' : 'suspension-idle'"
      >
        <!-- Rive Canvas (hidden until target car is selected) -->
        <canvas
          ref="canvasRef"
          :width="width"
          :height="height"
          class="relative z-10 block h-full w-full object-contain transition-opacity duration-200"
          :class="isCarSelected ? 'opacity-100' : 'opacity-0'"
        />

        <!-- Fallback emoji when canvas/rive is loading or car not yet selected -->
        <div
          v-if="!isCarSelected"
          class="absolute inset-0 z-0 flex items-center justify-center text-4xl opacity-90 transition-opacity"
        >
          <span>{{ FALLBACK_CARS[numericCarType] || '\u{1F697}' }}</span>
        </div>
      </div>
    </div>

    <!-- Realistic Ground Shadow -->
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
