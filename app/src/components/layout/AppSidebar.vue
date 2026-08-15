<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink, useRoute } from 'vue-router'
import { LayoutDashboard, List, Bell, Gauge, Settings, Car, Sun, Moon, Minimize2 } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'
import { useSettingsStore } from '@/stores/settingsStore'
import { invokeSafe } from '@/lib/ipc'

const items = [
  { to: '/', label: '仪表盘', icon: LayoutDashboard },
  { to: '/processes', label: '进程', icon: List },
  { to: '/visualizer', label: '流量公路', icon: Car },
  { to: '/alerts', label: '预警', icon: Bell },
  { to: '/throttle', label: '限速', icon: Gauge },
  { to: '/settings', label: '设置', icon: Settings },
]

const route = useRoute()
const settings = useSettingsStore()

const isDark = computed(() => (settings.config.theme ?? 'dark') === 'dark')

async function minimizeToTray() {
  await invokeSafe('minimize_to_tray', undefined, undefined)
}
</script>

<template>
  <aside class="flex w-56 shrink-0 flex-col border-r bg-card p-3">
    <div class="mb-6 flex items-center gap-2 px-2 text-xl font-bold tracking-tight">
      <span class="text-2xl">🐾</span>
      <span>NetTamer</span>
    </div>
    <nav class="flex flex-col gap-1">
      <RouterLink
        v-for="item in items"
        :key="item.to"
        :to="item.to"
        :class="
          cn(
            'flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium transition-colors',
            route.path === item.to
              ? 'bg-primary text-primary-foreground'
              : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground',
          )
        "
      >
        <component :is="item.icon" class="h-4 w-4" />
        <span>{{ item.label }}</span>
      </RouterLink>
    </nav>

    <div class="mt-auto flex flex-col gap-1 border-t pt-3">
      <Button
        variant="ghost"
        size="sm"
        class="justify-start gap-3 px-3 text-muted-foreground hover:text-foreground"
        :aria-label="'切换主题'"
        @click="settings.toggleTheme()"
      >
        <component :is="isDark ? Sun : Moon" class="h-4 w-4" />
        <span class="text-sm font-medium">{{ isDark ? '切换浅色' : '切换深色' }}</span>
      </Button>

      <Button
        variant="ghost"
        size="sm"
        class="justify-start gap-3 px-3 text-muted-foreground hover:text-foreground"
        aria-label="最小化到托盘"
        @click="minimizeToTray()"
      >
        <Minimize2 class="h-4 w-4" />
        <span class="text-sm font-medium">最小化到托盘</span>
      </Button>
    </div>
  </aside>
</template>
