<script setup lang="ts">
import { onMounted, onBeforeUnmount, watch } from 'vue'
import { useRoute } from 'vue-router'
import AppSidebar from '@/components/layout/AppSidebar.vue'
import Toaster from '@/components/ui/toaster.vue'
import { toast } from '@/components/ui/toast'
import { useSettingsStore } from '@/stores/settingsStore'
import { useAlertStore } from '@/stores/alertStore'
import { listenSafe, type UnlistenFn } from '@/lib/ipc'
import type { AlertEvent } from '@/types'
import { formatSpeed } from '@/composables/useFormatters'

import { useProcessStore } from '@/stores/processStore'
import { useFirewallStore } from '@/stores/firewallStore'

const route = useRoute()
const settings = useSettingsStore()
const alertStore = useAlertStore()
const processStore = useProcessStore()
const firewallStore = useFirewallStore()
let unlisten: UnlistenFn = () => {}

watch(
  () => route.path,
  (newPath) => {
    if (newPath !== '/visualizer' && settings.isImmersiveWindow) {
      settings.toggleImmersiveWindow(false)
    }
  },
)

onMounted(async () => {
  settings.load()
  firewallStore.load()
  processStore.start()
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
  <div v-if="route.path === '/taskbar-widget' || route.path === '/floating-widget'" class="h-screen w-screen bg-transparent overflow-hidden">
    <router-view />
  </div>
  <div v-else class="flex h-screen w-screen overflow-hidden bg-background text-foreground">
    <AppSidebar v-show="!settings.isImmersiveWindow" />
    <main :class="['relative flex-1 overflow-auto scrollbar-thin', settings.isImmersiveWindow || route.path === '/visualizer' ? 'p-0 h-full w-full' : 'p-6']">
      <router-view />
    </main>
    <Toaster />
  </div>
</template>
