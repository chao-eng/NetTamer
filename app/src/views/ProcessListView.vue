<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref, computed } from 'vue'
import type { ProcessStats, Policy, Rule } from '@/types'
import { Direction, DIRECTION_OPTIONS } from '@/types'
import { useProcessStore } from '@/stores/processStore'
import { useThrottleStore } from '@/stores/throttleStore'
import { useAlertStore } from '@/stores/alertStore'
import { toast } from '@/components/ui/toast'
import {
  Card,
  CardHeader,
  CardTitle,
  CardContent,
} from '@/components/ui/card'
import ProcessIcon from '@/components/common/ProcessIcon.vue'
import SpeedBadge from '@/components/common/SpeedBadge.vue'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select } from '@/components/ui/select'
import { Switch } from '@/components/ui/switch'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import {
  Table,
  TableHeader,
  TableBody,
  TableRow,
  TableHead,
  TableCell,
} from '@/components/ui/table'
import { Dialog } from '@/components/ui/dialog'

const processStore = useProcessStore()
const throttleStore = useThrottleStore()
const alertStore = useAlertStore()

const throttleOpen = ref(false)
const alertOpen = ref(false)
const selected = ref<ProcessStats | null>(null)

// 限速表单
const kbps = ref(512)
const limitUpload = ref(true)
const limitDownload = ref(true)
// 预警表单
const thresholdKb = ref(512)
const direction = ref<Direction>(Direction.Upload)
const cooldown = ref(30)

const sorted = computed(() => processStore.sortedProcesses)

function sortBy(field: 'uploadRate' | 'downloadRate' | 'name' | 'pid') {
  processStore.setSort(field)
}

function openThrottle(p: ProcessStats) {
  selected.value = p
  throttleOpen.value = true
}
function openAlert(p: ProcessStats) {
  selected.value = p
  alertOpen.value = true
}

async function applyThrottle() {
  if (!selected.value) return
  const policy: Policy = {
    id: `NT_${selected.value.name}_${Date.now()}`,
    name: `NT_${selected.value.name}`,
    processName: selected.value.name,
    rateLimitBps: Math.round(kbps.value * 1024 * 8),
    limitUpload: limitUpload.value,
    limitDownload: limitDownload.value,
    active: true,
    createdAt: Math.floor(Date.now() / 1000),
  }
  await throttleStore.apply(policy)
  await throttleStore.load()
  toast('已应用限速策略', 'success')
  throttleOpen.value = false
}

async function createAlert() {
  if (!selected.value) return
  const rule: Rule = {
    id: `R_${selected.value.name}_${Date.now()}`,
    name: `预警-${selected.value.name}`,
    processName: selected.value.name,
    threshold: Math.round(Number(thresholdKb.value) * 1024),
    direction: Number(direction.value),
    cooldownSec: Number(cooldown.value),
    enabled: true,
    createdAt: Math.floor(Date.now() / 1000),
  }
  await alertStore.createRule(rule)
  await alertStore.loadRules()
  toast('已创建预警规则', 'success')
  alertOpen.value = false
}

let unlisten: (() => void)[] = []

onMounted(async () => {
  await processStore.fetchList()
  unlisten = await processStore.bindEvents()
  if (!processStore.isMonitoring) {
    await processStore.start()
  }
})
onBeforeUnmount(() => {
  processStore.searchQuery = ''
  unlisten.forEach((fn) => fn())
})
</script>

<template>
  <div class="flex flex-col gap-4">
    <div class="flex items-center gap-3">
      <Input
        v-model="processStore.searchQuery"
        placeholder="搜索进程名..."
        class="max-w-sm"
      />
      <Badge variant="secondary">{{ sorted.length }} 个进程</Badge>
    </div>

    <Card>
      <CardContent class="pt-6">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>
                <button class="font-medium hover:underline" @click="sortBy('name')">进程</button>
              </TableHead>
              <TableHead>
                <button class="font-medium hover:underline" @click="sortBy('pid')">PID</button>
              </TableHead>
              <TableHead class="text-right">
                <button class="font-medium hover:underline" @click="sortBy('uploadRate')">上传</button>
              </TableHead>
              <TableHead class="text-right">
                <button class="font-medium hover:underline" @click="sortBy('downloadRate')">下载</button>
              </TableHead>
              <TableHead class="text-right">操作</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableRow v-for="p in sorted" :key="p.pid">
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
              <TableCell class="text-right">
                <div class="flex justify-end gap-2">
                  <Button size="sm" variant="outline" @click="openThrottle(p)">限速</Button>
                  <Button size="sm" variant="outline" @click="openAlert(p)">预警</Button>
                </div>
              </TableCell>
            </TableRow>
          </TableBody>
        </Table>
      </CardContent>
    </Card>

    <Dialog v-model:open="throttleOpen" title="进程限速">
      <div class="flex flex-col gap-3">
        <div>
          <Label>进程</Label>
          <div class="mt-1 text-sm font-medium">{{ selected?.name }}</div>
        </div>
        <div>
          <Label for="kbps">限速值 (KB/s)</Label>
          <Input id="kbps" v-model="kbps" type="number" class="mt-1" />
        </div>
        <div class="flex items-center justify-between">
          <Label>限制上传</Label>
          <Switch v-model="limitUpload" />
        </div>
        <div class="flex items-center justify-between">
          <Label>限制下载</Label>
          <Switch v-model="limitDownload" />
        </div>
      </div>
      <template #footer>
        <Button variant="ghost" @click="throttleOpen = false">取消</Button>
        <Button @click="applyThrottle">应用</Button>
      </template>
    </Dialog>

    <Dialog v-model:open="alertOpen" title="创建预警规则">
      <div class="flex flex-col gap-3">
        <div>
          <Label>进程</Label>
          <div class="mt-1 text-sm font-medium">{{ selected?.name }}</div>
        </div>
        <div>
          <Label for="thr">阈值 (KB/s)</Label>
          <Input id="thr" v-model="thresholdKb" type="number" class="mt-1" />
        </div>
        <div>
          <Label>方向</Label>
          <Select
            v-model="direction"
            :options="DIRECTION_OPTIONS"
            class="mt-1"
          />
        </div>
        <div>
          <Label for="cd">冷却时间 (秒)</Label>
          <Input id="cd" v-model="cooldown" type="number" class="mt-1" />
        </div>
      </div>
      <template #footer>
        <Button variant="ghost" @click="alertOpen = false">取消</Button>
        <Button @click="createAlert">创建</Button>
      </template>
    </Dialog>
  </div>
</template>
