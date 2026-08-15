<script setup lang="ts">
import { useToasts, type ToastType } from '@/components/ui/toast'
import { cn } from '@/lib/utils'

const toasts = useToasts()

const styleMap: Record<ToastType, string> = {
  info: 'bg-secondary text-secondary-foreground',
  success: 'bg-upload text-upload-foreground',
  error: 'bg-destructive text-destructive-foreground',
  warning: 'bg-warning text-warning-foreground',
}
</script>

<template>
  <div class="fixed bottom-4 right-4 z-[60] flex flex-col gap-2">
    <div
      v-for="t in toasts"
      :key="t.id"
      :class="cn('rounded-md px-4 py-2 text-sm shadow-lg animate-slide-in-right', styleMap[t.type])"
    >
      {{ t.message }}
    </div>
  </div>
</template>
