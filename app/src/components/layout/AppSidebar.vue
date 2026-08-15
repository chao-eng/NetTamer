<script setup lang="ts">
import { RouterLink, useRoute } from 'vue-router'
import { LayoutDashboard, List, Bell, Gauge, Settings } from 'lucide-vue-next'
import { cn } from '@/lib/utils'

const items = [
  { to: '/', label: '仪表盘', icon: LayoutDashboard },
  { to: '/processes', label: '进程', icon: List },
  { to: '/alerts', label: '预警', icon: Bell },
  { to: '/throttle', label: '限速', icon: Gauge },
  { to: '/settings', label: '设置', icon: Settings },
]

const route = useRoute()
</script>

<template>
  <aside class="flex w-56 shrink-0 flex-col border-r bg-card p-3">
    <div class="mb-6 px-2 text-xl font-bold tracking-tight">NetTamer</div>
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
  </aside>
</template>
