<script setup lang="ts">
import { computed } from 'vue'
import { formatSpeed } from '@/composables/useFormatters'

const props = withDefaults(
  defineProps<{ value?: number; label?: string; max?: number; unit?: 'bytes' | 'bits' }>(),
  { value: 0, label: '', max: 100, unit: 'bytes' },
)

const max = computed(() => props.max || 1)
const pct = computed(() => Math.min(1, Math.max(0, (props.value ?? 0) / max.value)))

const r = 60
const cx = 80
const cy = 80
const circumference = Math.PI * r
const dash = computed(() => `${pct.value * circumference} ${circumference}`)

const display = computed(() => formatSpeed(props.value ?? 0))
</script>

<template>
  <div class="flex flex-col items-center">
    <svg viewBox="0 0 160 95" class="w-40">
      <path
        :d="`M ${cx - r} ${cy} A ${r} ${r} 0 0 1 ${cx + r} ${cy}`"
        fill="none"
        stroke="hsl(var(--border))"
        stroke-width="10"
        stroke-linecap="round"
      />
      <path
        :d="`M ${cx - r} ${cy} A ${r} ${r} 0 0 1 ${cx + r} ${cy}`"
        fill="none"
        stroke="hsl(var(--primary))"
        stroke-width="10"
        stroke-linecap="round"
        :stroke-dasharray="dash"
      />
      <text :x="cx" :y="cy - 12" text-anchor="middle" class="fill-foreground text-sm font-semibold">
        {{ display }}
      </text>
    </svg>
    <div class="mt-1 text-xs text-muted-foreground">{{ label }}</div>
  </div>
</template>
