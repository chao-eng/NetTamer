<script setup lang="ts">
import { computed } from 'vue'
import { cn } from '@/lib/utils'

type Option = string | { label: string; value: string | number }

const props = withDefaults(
  defineProps<{
    modelValue?: string | number
    options?: Option[]
    class?: string
    disabled?: boolean
  }>(),
  { options: () => [] },
)

const emit = defineEmits<{ (e: 'update:modelValue', v: string | number): void }>()

const normalized = computed(() =>
  (props.options ?? []).map((o) =>
    typeof o === 'string' ? { label: o, value: o } : o,
  ),
)

function onChange(e: Event) {
  const el = e.target as HTMLSelectElement
  const found = normalized.value.find((o) => String(o.value) === el.value)
  emit('update:modelValue', found ? found.value : el.value)
}
</script>

<template>
  <select
    :value="modelValue"
    :disabled="disabled"
    @change="onChange"
    :class="cn('flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50', props.class)"
  >
    <option v-for="opt in normalized" :key="String(opt.value)" :value="opt.value">
      {{ opt.label }}
    </option>
  </select>
</template>
