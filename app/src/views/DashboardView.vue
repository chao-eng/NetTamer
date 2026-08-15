<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref, computed, watch } from 'vue'
import type { SpeedPoint, UnlistenFn } from '@/types'
import { useProcessStore } from '@/stores/processStore'
import {
  Card,
  CardHeader,
  CardTitle,
  CardContent,
} from '@/components/ui/card'
import SpeedBadge from '@/components/common/SpeedBadge.vue'
import SpeedChart from '@/components/charts/SpeedChart.vue'
import ProcessIcon from '@/components/common/ProcessIcon.vue'
import StatusIndicator from '@/components/common/StatusIndicator.vue'
import { Button } from '@/components/ui/button'
import {
  Table,
  TableHeader,
  TableBody,
  TableRow,
  TableHead,
  TableCell,
} from '@/components/ui/table'

const processStore = useProcessStore()
const buffer = ref<SpeedPoint[]>([])
let unlisten: UnlistenFn[] = []

const topProcesses = computed(() => processStore.sortedProcesses.slice(0, 5))

watch(
  () => [processStore.totalUploadRate, processStore.totalDownloadRate] as const,
  ([up, down]) => {
    buffer.value.push({ t: Date.now(), up, down })
    if (buffer.value.length > 60) buffer.value.shift()
  },
)

onMounted(async () => {
  await processStore.fetchList()
  unlisten = await processStore.bindEvents()
  if (!processStore.isMonitoring) {
    await processStore.start()
  }
})

onBeforeUnmount(() => {
  unlisten.forEach((fn) => fn())
})

async function toggleMonitoring() {
  if (processStore.isMonitoring) {
    await processStore.stop()
  } else {
    await processStore.start()
  }
}
</script>

<template>
  <div class="flex flex-col gap-6">
    <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
      <Card>
        <CardHeader class="pb-2">
          <CardTitle class="text-sm text-muted-foreground">总上传速率</CardTitle>
        </CardHeader>
        <CardContent>
          <div class="text-2xl font-bold">
            <SpeedBadge :rate="processStore.totalUploadRate" direction="up" />
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardHeader class="pb-2">
          <CardTitle class="text-sm text-muted-foreground">总下载速率</CardTitle>
        </CardHeader>
        <CardContent>
          <div class="text-2xl font-bold">
            <SpeedBadge :rate="processStore.totalDownloadRate" direction="down" />
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardHeader class="pb-2">
          <CardTitle class="text-sm text-muted-foreground">监控状态</CardTitle>
        </CardHeader>
        <CardContent class="flex items-center gap-2">
          <StatusIndicator :active="processStore.isMonitoring" />
          <span class="text-sm">
            {{ processStore.isMonitoring ? '运行中' : '已停止' }}
          </span>
          <Button
            class="ml-auto"
            size="sm"
            :variant="processStore.isMonitoring ? 'secondary' : 'default'"
            @click="toggleMonitoring"
          >
            {{ processStore.isMonitoring ? '停止监控' : '开始监控' }}
          </Button>
        </CardContent>
      </Card>
    </div>

    <Card>
      <CardHeader>
        <CardTitle>实时速率</CardTitle>
      </CardHeader>
      <CardContent>
        <SpeedChart :data="buffer" />
      </CardContent>
    </Card>

    <Card>
      <CardHeader>
        <CardTitle>Top 5 进程（按上传速率）</CardTitle>
      </CardHeader>
      <CardContent>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>进程</TableHead>
              <TableHead>PID</TableHead>
              <TableHead class="text-right">上传</TableHead>
              <TableHead class="text-right">下载</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableRow v-for="p in topProcesses" :key="p.pid">
              <TableCell>
                <div class="flex items-center gap-2">
                  <ProcessIcon :icon-b64="p.iconB64" :name="p.name" />
                  <span class="truncate">{{ p.name }}</span>
                </div>
              </TableCell>
              <TableCell class="tabular">{{ p.pid }}</TableCell>
              <TableCell class="text-right">
                <SpeedBadge :rate="p.uploadRate" direction="up" />
              </TableCell>
              <TableCell class="text-right">
                <SpeedBadge :rate="p.downloadRate" direction="down" />
              </TableCell>
            </TableRow>
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  </div>
</template>
