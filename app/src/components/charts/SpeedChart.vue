<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount } from 'vue'
import type { SpeedPoint } from '@/types'

const props = defineProps<{ data?: SpeedPoint[] }>()
const canvasRef = ref<HTMLCanvasElement | null>(null)
let resizeObserver: ResizeObserver | null = null

function draw() {
  const canvas = canvasRef.value
  if (!canvas) return
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  const dpr = window.devicePixelRatio || 1
  const w = canvas.clientWidth
  const h = canvas.clientHeight
  if (w === 0 || h === 0) return
  canvas.width = w * dpr
  canvas.height = h * dpr
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
  ctx.clearRect(0, 0, w, h)

  // 网格
  ctx.strokeStyle = 'rgba(128,128,128,0.18)'
  ctx.lineWidth = 1
  for (let i = 0; i <= 4; i++) {
    const y = (h / 4) * i
    ctx.beginPath()
    ctx.moveTo(0, y)
    ctx.lineTo(w, y)
    ctx.stroke()
  }

  const list = props.data ?? []
  if (list.length === 0) return

  const maxV = Math.max(1, ...list.map((p) => Math.max(p.up, p.down)))
  const n = list.length
  const stepX = n > 1 ? w / (n - 1) : w

  const drawLine = (key: 'up' | 'down', color: string) => {
    ctx.strokeStyle = color
    ctx.lineWidth = 2
    ctx.beginPath()
    list.forEach((p, idx) => {
      const x = idx * stepX
      const y = h - (p[key] / maxV) * (h - 12) - 6
      if (idx === 0) ctx.moveTo(x, y)
      else ctx.lineTo(x, y)
    })
    ctx.stroke()
  }

  drawLine('up', '#f59e0b')
  drawLine('down', '#3b82f6')
}

onMounted(() => {
  draw()
  if (canvasRef.value && typeof ResizeObserver !== 'undefined') {
    resizeObserver = new ResizeObserver(() => draw())
    resizeObserver.observe(canvasRef.value)
  }
})

onBeforeUnmount(() => {
  resizeObserver?.disconnect()
})

watch(() => props.data, draw, { deep: true })
</script>

<template>
  <canvas ref="canvasRef" class="h-48 w-full"></canvas>
</template>
