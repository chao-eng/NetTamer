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
            <p class="text-xs text-muted-foreground">在系统任务栏托盘左侧显示紧凑实时网速（↑ / ↓）</p>
          </div>
          <Switch
            :model-value="settingsStore.config[CONFIG_KEYS.taskbarSpeed] === 'true'"
            @update:model-value="settingsStore.toggleTaskbarSpeed()"
          />
        </div>

        <div class="space-y-3">
          <div class="flex items-center justify-between">
            <div>
              <Label>桌面置顶网速</Label>
              <p class="text-xs text-muted-foreground">在桌面显示置顶迷你网速悬浮窗，支持拖拽移动与主题自适应</p>
            </div>
            <Switch
              :model-value="settingsStore.config[CONFIG_KEYS.floatingSpeed] === 'true'"
              @update:model-value="settingsStore.toggleFloatingSpeed()"
            />
          </div>

          <!-- Sub-options for floating widget -->
          <div
            v-if="settingsStore.config[CONFIG_KEYS.floatingSpeed] === 'true'"
            class="ml-3 pl-3 border-l-2 border-primary/20 space-y-3 py-1 animate-in fade-in duration-200"
          >
            <div class="flex items-center justify-between">
              <div>
                <Label class="text-xs">悬浮窗鼠标穿透</Label>
                <p class="text-[11px] text-muted-foreground">开启后点击穿透到底层窗口；关闭可恢复拖拽与右键</p>
              </div>
              <Switch
                :model-value="settingsStore.config[CONFIG_KEYS.floatingClickThrough] === 'true'"
                @update:model-value="settingsStore.toggleFloatingClickThrough()"
              />
            </div>

            <div class="flex items-center justify-between">
              <div>
                <Label class="text-xs">悬浮窗透明度</Label>
                <p class="text-[11px] text-muted-foreground">设置悬浮窗的不透明度</p>
              </div>
              <div class="flex items-center gap-1">
                <button
                  v-for="op in [100, 80, 60, 40]"
                  :key="op"
                  type="button"
                  @click="settingsStore.setFloatingOpacity(op)"
                  class="px-2 py-0.5 text-xs font-mono rounded border transition-colors"
                  :class="
                    (settingsStore.config[CONFIG_KEYS.floatingOpacity] || '100') === String(op)
                      ? 'bg-primary text-primary-foreground border-primary font-bold shadow-sm'
                      : 'bg-background hover:bg-muted text-muted-foreground'
                  "
                >
                  {{ op }}%
                </button>
              </div>
            </div>
          </div>
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
