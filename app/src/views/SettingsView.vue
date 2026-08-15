<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { CONFIG_KEYS } from '@/types'
import { useSettingsStore } from '@/stores/settingsStore'
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { Button } from '@/components/ui/button'
import { formatSpeed } from '@/composables/useFormatters'

const settingsStore = useSettingsStore()
const refreshMs = ref(1000)

onMounted(async () => {
  await settingsStore.load()
  const v = Number(settingsStore.config[CONFIG_KEYS.refreshInterval])
  if (!Number.isNaN(v)) refreshMs.value = v
})

async function onRefreshChange() {
  const ms = Math.max(250, Math.round(refreshMs.value))
  refreshMs.value = ms
  await settingsStore.setRefreshInterval(ms)
}
</script>

<template>
  <div class="flex flex-col gap-4">
    <Card>
      <CardHeader>
        <CardTitle>全局设置</CardTitle>
        <CardDescription>刷新频率、主题与开机行为。</CardDescription>
      </CardHeader>
      <CardContent class="flex flex-col gap-4">
        <div class="flex items-center justify-between">
          <div>
            <Label for="ri">刷新间隔 (毫秒)</Label>
            <p class="text-xs text-muted-foreground">过小会增加系统开销，建议 ≥ 250ms</p>
          </div>
          <Input id="ri" v-model="refreshMs" type="number" class="w-28" @change="onRefreshChange" />
        </div>

        <div class="flex items-center justify-between">
          <div>
            <Label>深色主题</Label>
            <p class="text-xs text-muted-foreground">当前: {{ settingsStore.config[CONFIG_KEYS.theme] }}</p>
          </div>
          <Button variant="outline" @click="settingsStore.toggleTheme()">切换主题</Button>
        </div>

        <div class="flex items-center justify-between">
          <div>
            <Label>开机自启</Label>
            <p class="text-xs text-muted-foreground">需管理员权限，依赖 Tauri autostart 插件</p>
          </div>
          <Switch
            :model-value="settingsStore.config[CONFIG_KEYS.autoStart] === 'true'"
            @update:model-value="settingsStore.toggleAutoStart()"
          />
        </div>

        <div class="flex items-center justify-between">
          <div>
            <Label>最小化到托盘</Label>
            <p class="text-xs text-muted-foreground">关闭窗口时保留后台监控</p>
          </div>
          <Switch
            :model-value="settingsStore.config[CONFIG_KEYS.minimizeToTray] === 'true'"
            @update:model-value="settingsStore.toggleMinimizeToTray()"
          />
        </div>

        <div class="flex items-center justify-between">
          <div>
            <Label>任务栏网速显示</Label>
            <p class="text-xs text-muted-foreground">在系统任务栏托盘实时显示上传与下载速度（↑ / ↓）</p>
          </div>
          <Switch
            :model-value="settingsStore.config[CONFIG_KEYS.taskbarSpeed] === 'true'"
            @update:model-value="settingsStore.toggleTaskbarSpeed()"
          />
        </div>
      </CardContent>
    </Card>

    <Card>
      <CardHeader>
        <CardTitle>关于</CardTitle>
      </CardHeader>
      <CardContent class="text-sm text-muted-foreground">
        <p>NetTamer — 进程级网络监控与流量整形工具</p>
        <p>技术栈：Tauri 2.0 · Vue 3 · shadcn-vue · Tailwind CSS · windows-rs ETW · WinDivert</p>
      </CardContent>
    </Card>
  </div>
</template>
