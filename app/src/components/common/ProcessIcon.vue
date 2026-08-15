<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{ iconB64?: string; name?: string }>(),
  { iconB64: '', name: '' },
)

const letter = computed(() => (props.name || '?').charAt(0).toUpperCase())
const src = computed(() => {
  if (!props.iconB64) return ''
  return props.iconB64.startsWith('data:')
    ? props.iconB64
    : `data:image/png;base64,${props.iconB64}`
})
</script>

<template>
  <div class="flex h-5 w-5 shrink-0 items-center justify-center">
    <img
      v-if="src"
      :src="src"
      alt=""
      class="h-5 w-5 object-contain select-none transition-transform"
      style="image-rendering: -webkit-optimize-contrast;"
    />
    <div
      v-else
      class="flex h-5 w-5 items-center justify-center rounded bg-muted/80 text-[10px] font-bold text-muted-foreground uppercase"
    >
      {{ letter }}
    </div>
  </div>
</template>
