<script setup lang="ts">
import { cn } from '@/lib/utils'

const props = withDefaults(defineProps<{ active?: boolean; class?: string }>(), {
  active: false,
})
</script>

<template>
  <span
    :class="
      cn(
        'inline-block h-2.5 w-2.5 rounded-full',
        props.active ? 'bg-upload' : 'bg-muted-foreground/40',
        props.class,
      )
    "
  />
</template>
