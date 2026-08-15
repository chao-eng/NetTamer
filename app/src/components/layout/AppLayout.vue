<script setup lang="ts">
import { onMounted, onBeforeUnmount } from 'vue'
import AppSidebar from '@/components/layout/AppSidebar.vue'
import AppHeader from '@/components/layout/AppHeader.vue'
import Toaster from '@/components/ui/toaster.vue'
import { toast } from '@/components/ui/toast'
import { useSettingsStore } from '@/stores/settingsStore'
import { useAlertStore } from '@/stores/alertStore'
import { listenSafe, type UnlistenFn } from '@/lib/ipc'
import type { AlertEvent } from '@/types'
import { formatSpeed } from '@/composables/useFormatters'

const settings = useSettingsStore()
const alertStore = useAlertStore()
let unlisten: UnlistenFn = () => {}

onMounted(async () => {
  settings.load()
  unlisten = await listenSafe<AlertEvent>('alert:triggered', (ev) => {
    if (ev) {
      toast(`⚠️ 进程「${ev.processName}」速率达到 ${formatSpeed(ev.currentRate)}，已触发预警！`, 'warning')
      alertStore.loadHistory()
    }
  })
})

onBeforeUnmount(() => {
  unlisten()
})
</script>

<template>
  <div class="flex h-screen w-screen overflow-hidden bg-background text-foreground">
    <AppSidebar />
    <div class="flex flex-1 flex-col overflow-hidden">
      <AppHeader />
      <main class="flex-1 overflow-auto scrollbar-thin p-6">
        <router-view />
      </main>
    </div>
    <Toaster />
  </div>
</template>
