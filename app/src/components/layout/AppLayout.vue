<script setup lang="ts">
import { onMounted } from 'vue'
import AppSidebar from '@/components/layout/AppSidebar.vue'
import AppHeader from '@/components/layout/AppHeader.vue'
import Toaster from '@/components/ui/toaster.vue'
import { useSettingsStore } from '@/stores/settingsStore'

const settings = useSettingsStore()

onMounted(() => {
  settings.load()
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
