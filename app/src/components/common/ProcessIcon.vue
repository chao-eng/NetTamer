<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{ iconB64?: string; name?: string }>(),
  { iconB64: '', name: '' },
)

const isSystem = computed(() => {
  const n = (props.name || '').toLowerCase()
  return n.includes('system') || n.includes('windows 系统') || n === 'ntoskrnl.exe'
})

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
    <svg
      v-else-if="isSystem"
      class="h-4 w-4 shrink-0"
      viewBox="0 0 88 88"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <path d="M0 12.402l35.687-4.86.016 34.423-35.67.203zm35.67 33.529l.028 34.453L0 75.48V46.08zm4.326-39.525L87.914 0v41.525l-47.918.275zm47.918 43.657L40.024 88V46.52l47.894.272z" fill="#0078D4"/>
    </svg>
    <div
      v-else
      class="flex h-5 w-5 items-center justify-center rounded bg-muted/80 text-[10px] font-bold text-muted-foreground uppercase"
    >
      {{ letter }}
    </div>
  </div>
</template>
