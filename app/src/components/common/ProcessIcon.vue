<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{ iconB64?: string; name?: string }>(),
  { iconB64: '', name: '' },
)

const letter = computed(() => (props.name || '?').charAt(0).toUpperCase())
const src = computed(() =>
  props.iconB64 ? `data:image/png;base64,${props.iconB64}` : '',
)
</script>

<template>
  <div class="flex h-6 w-6 items-center justify-center overflow-hidden rounded bg-muted">
    <img v-if="src" :src="src" alt="" class="h-full w-full object-contain" />
    <span v-else class="text-xs font-bold text-muted-foreground">{{ letter }}</span>
  </div>
</template>
