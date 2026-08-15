<script setup lang="ts">
import { cn } from '@/lib/utils'

const props = withDefaults(
  defineProps<{
    modelValue?: string | number
    class?: string
    type?: string
    placeholder?: string
    disabled?: boolean
  }>(),
  { type: 'text' },
)

const emit = defineEmits<{
  (e: 'update:modelValue', v: string | number): void
}>()

function onInput(e: Event) {
  const el = e.target as HTMLInputElement
  const value =
    props.type === 'number' ? (el.value === '' ? '' : el.valueAsNumber || 0) : el.value
  emit('update:modelValue', value)
}
</script>

<template>
  <input
    :type="props.type"
    :value="modelValue"
    :placeholder="placeholder"
    :disabled="disabled"
    @input="onInput"
    :class="cn('flex h-9 w-full rounded-md border border-input bg-background px-3 py-1 text-sm shadow-sm transition-colors placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50', props.class)"
  />
</template>
