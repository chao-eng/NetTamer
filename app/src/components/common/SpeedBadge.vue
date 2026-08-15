<script setup lang="ts">
import { computed } from 'vue'
import { formatSpeed } from '@/composables/useFormatters'
import { cn } from '@/lib/utils'

const props = withDefaults(
  defineProps<{ rate?: number; direction?: 'up' | 'down' }>(),
  { rate: 0, direction: 'up' },
)

const text = computed(() => formatSpeed(props.rate ?? 0))
const color = computed(() =>
  props.direction === 'up' ? 'text-warning' : 'text-download',
)
</script>

<template>
  <span :class="cn('tabular font-medium', color)">{{ text }}</span>
</template>
