<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink, useRoute } from 'vue-router'
import { LayoutDashboard, List, Bell, Gauge, Settings, Car, Sun, Moon, Minimize2, Github } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'
import { isTauri } from '@/types'
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

async function openGithub() {
  const url = 'https://github.com/chao-eng/NetTamer'
  if (isTauri()) {
    await invokeSafe('open_url', { url }, undefined)
  } else {
    window.open(url, '_blank')
  }
}
</script>

<template>
  <aside class="flex w-44 shrink-0 flex-col border-r bg-card p-2.5">
    <div class="mb-4 flex items-center gap-1.5 px-1.5 text-lg font-bold tracking-tight">
      <span class="text-xl">🐾</span>
      <span>网络驯兽师</span>
    </div>
    <nav class="flex flex-col gap-1">
      <RouterLink
        v-for="item in items"
        :key="item.to"
        :to="item.to"
        :class="
          cn(
            'flex items-center gap-2.5 rounded-md px-2.5 py-1.5 text-sm font-medium transition-colors',
            route.path === item.to
              ? 'bg-primary text-primary-foreground'
              : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground',
          )
        "
      >
        <component :is="item.icon" class="h-4 w-4 shrink-0" />
        <span>{{ item.label }}</span>
      </RouterLink>
    </nav>

    <div class="mt-auto flex flex-col gap-1 border-t pt-2.5">
      <a
        href="https://github.com/chao-eng/NetTamer"
        target="_blank"
        rel="noopener noreferrer"
        class="flex h-8 items-center gap-2.5 rounded-md px-2.5 text-xs text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
        @click.prevent="openGithub"
      >
        <Github class="h-3.5 w-3.5 shrink-0" />
        <span class="font-medium">GitHub 主页</span>
      </a>

      <Button
        variant="ghost"
        size="sm"
        class="h-8 justify-start gap-2.5 px-2.5 text-xs text-muted-foreground hover:text-foreground"
        :aria-label="'切换主题'"
        @click="settings.toggleTheme()"
      >
        <component :is="isDark ? Sun : Moon" class="h-3.5 w-3.5 shrink-0" />
        <span class="font-medium">{{ isDark ? '切换浅色' : '切换深色' }}</span>
      </Button>

      <Button
        variant="ghost"
        size="sm"
        class="h-8 justify-start gap-2.5 px-2.5 text-xs text-muted-foreground hover:text-foreground"
        aria-label="最小化到托盘"
        @click="minimizeToTray()"
      >
        <Minimize2 class="h-3.5 w-3.5 shrink-0" />
        <span class="font-medium">最小化到托盘</span>
      </Button>
    </div>
  </aside>
</template>
