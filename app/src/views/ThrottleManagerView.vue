<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref } from 'vue'
import type { Policy } from '@/types'
import { useThrottleStore } from '@/stores/throttleStore'
import { toast } from '@/components/ui/toast'
import { listenSafe, type UnlistenFn } from '@/lib/ipc'
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Table, TableHeader, TableBody, TableRow, TableHead, TableCell } from '@/components/ui/table'
import { formatSpeed } from '@/composables/useFormatters'

const throttleStore = useThrottleStore()

const processName = ref('')
const kbps = ref(512)
const limitUpload = ref(true)
const limitDownload = ref(false)

let unlisten: UnlistenFn = () => {}

async function applyThrottle() {
  if (!processName.value.trim()) {
    toast('请填写进程名', 'error')
    return
  }
  if (!limitUpload.value && !limitDownload.value) {
    toast('请至少选择一个限速方向（上传或下载）', 'error')
    return
  }
  const policy: Policy = {
    id: `NT_${processName.value}_${Date.now()}`,
    name: `NT_${processName.value}`,
    processName: processName.value.trim(),
    rateLimitBps: Math.round(kbps.value * 1024 * 8),
    limitUpload: limitUpload.value,
    limitDownload: limitDownload.value,
    active: true,
    createdAt: Math.floor(Date.now() / 1000),
  }
  await throttleStore.apply(policy)
  await throttleStore.load()
  toast('已应用限速策略', 'success')
  processName.value = ''
}

async function removePolicy(id: string) {
  await throttleStore.remove(id)
  toast('已移除限速策略', 'success')
}

onMounted(async () => {
  await throttleStore.load()
  unlisten = await listenSafe<Policy>('throttle:changed', () => {
    throttleStore.load()
  })
})
onBeforeUnmount(() => unlisten())
</script>

<template>
  <div class="flex flex-col gap-4">
    <Card>
      <CardHeader>
        <CardTitle>新建限速策略</CardTitle>
        <CardDescription>基于 WinDivert 对指定进程进行上传 / 下载限速。</CardDescription>
      </CardHeader>
      <CardContent class="flex flex-col gap-3">
        <div class="grid grid-cols-1 gap-3 md:grid-cols-3">
          <div>
            <Label for="tpname">进程名</Label>
            <Input id="tpname" v-model="processName" placeholder="steam.exe" class="mt-1" />
          </div>
          <div>
            <Label for="tkbps">限速值 (KB/s)</Label>
            <Input id="tkbps" v-model="kbps" type="number" class="mt-1" />
          </div>
          <div class="flex flex-col justify-end gap-2">
            <div class="flex items-center justify-between">
              <Label>限制上传</Label>
              <Switch v-model="limitUpload" />
            </div>
            <div class="flex items-center justify-between">
              <Label>限制下载</Label>
              <Switch v-model="limitDownload" />
            </div>
          </div>
        </div>
        <div class="flex justify-end">
          <Button @click="applyThrottle">应用策略</Button>
        </div>
      </CardContent>
    </Card>

    <Card>
      <CardHeader>
        <CardTitle>当前限速策略 ({{ throttleStore.policies.length }})</CardTitle>
      </CardHeader>
      <CardContent>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>进程</TableHead>
              <TableHead class="text-right">限速</TableHead>
              <TableHead class="text-right">方向</TableHead>
              <TableHead class="text-right">操作</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableRow v-for="p in throttleStore.policies" :key="p.id">
              <TableCell>{{ p.processName }}</TableCell>
              <TableCell class="text-right">{{ formatSpeed(p.rateLimitBps / 8) }}</TableCell>
              <TableCell class="text-right">
                <div class="flex justify-end gap-1">
                  <Badge v-if="p.limitUpload" variant="secondary">上</Badge>
                  <Badge v-if="p.limitDownload" variant="secondary">下</Badge>
                </div>
              </TableCell>
              <TableCell class="text-right">
                <Button size="sm" variant="ghost" @click="removePolicy(p.id)">移除</Button>
              </TableCell>
            </TableRow>
            <TableRow v-if="throttleStore.policies.length === 0">
              <TableCell colspan="4" class="text-center text-muted-foreground">暂无策略</TableCell>
            </TableRow>
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  </div>
</template>
