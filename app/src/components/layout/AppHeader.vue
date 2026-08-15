<script setup lang="ts">
import { computed } from 'vue'
import { useProcessStore } from '@/stores/processStore'
import { useSettingsStore } from '@/stores/settingsStore'
import SpeedBadge from '@/components/common/SpeedBadge.vue'
import { Button } from '@/components/ui/button'
import { Sun, Moon } from 'lucide-vue-next'

const processStore = useProcessStore()
const settings = useSettingsStore()

const isDark = computed(() => (settings.config.theme ?? 'dark') === 'dark')
</script>

<template>
  <header class="flex h-14 shrink-0 items-center justify-between border-b bg-card px-6">
    <div class="flex items-center gap-6">
      <div class="flex items-center gap-2">
        <span class="text-xs text-muted-foreground">上传</span>
        <SpeedBadge :rate="processStore.totalUploadRate" direction="up" />
      </div>
      <div class="flex items-center gap-2">
        <span class="text-xs text-muted-foreground">下载</span>
        <SpeedBadge :rate="processStore.totalDownloadRate" direction="down" />
      </div>
    </div>
    <div class="flex items-center gap-4">
      <span class="hidden text-xs text-muted-foreground md:inline">最小化到托盘</span>
      <Button variant="ghost" size="icon" :aria-label="'切换主题'" @click="settings.toggleTheme()">
        <component :is="isDark ? Sun : Moon" class="h-4 w-4" />
      </Button>
    </div>
  </header>
</template>
