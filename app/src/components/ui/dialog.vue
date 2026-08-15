<script setup lang="ts">
import { cn } from '@/lib/utils'

const props = withDefaults(
  defineProps<{ open?: boolean; title?: string; class?: string }>(),
  { open: false, title: '' },
)

const emit = defineEmits<{ (e: 'update:open', v: boolean): void }>()

function close() {
  emit('update:open', false)
}
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="fixed inset-0 z-50 flex items-center justify-center">
      <div
        class="absolute inset-0 bg-black/50 animate-fade-in"
        @click="close"
      />
      <div
        :class="cn('relative z-10 w-full max-w-md rounded-lg border bg-card p-6 text-card-foreground shadow-lg animate-zoom-in', props.class)"
      >
        <h2 class="mb-4 text-lg font-semibold">{{ title }}</h2>
        <div class="text-sm">
          <slot />
        </div>
        <div class="mt-6 flex justify-end gap-2">
          <slot name="footer" :close="close" />
        </div>
      </div>
    </div>
  </Teleport>
</template>
