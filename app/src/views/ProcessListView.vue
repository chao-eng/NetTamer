<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref, computed } from 'vue'
import type { ProcessStats, Rule } from '@/types'
import { useProcessStore } from '@/stores/processStore'
import { useFirewallStore } from '@/stores/firewallStore'
import { useAlertStore } from '@/stores/alertStore'
import { toast } from '@/components/ui/toast'
import {
  Card,
  CardContent,
} from '@/components/ui/card'
import ProcessIcon from '@/components/common/ProcessIcon.vue'
import SpeedBadge from '@/components/common/SpeedBadge.vue'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
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
import { ShieldBan, ShieldCheck, Bell } from 'lucide-vue-next'

const processStore = useProcessStore()
const firewallStore = useFirewallStore()
const alertStore = useAlertStore()

const alertOpen = ref(false)
const blockConfirmOpen = ref(false)
const selected = ref<ProcessStats | null>(null)

// 预警表单
const thresholdKb = ref(512)
const alertUpload = ref(true)
const alertDownload = ref(false)
const cooldown = ref(30)

const sorted = computed(() => processStore.sortedProcesses)

function sortBy(field: 'uploadRate' | 'downloadRate' | 'name' | 'pid') {
  processStore.setSort(field)
}

function promptBlock(p: ProcessStats) {
  selected.value = p
  blockConfirmOpen.value = true
}

async function confirmBlock() {
  if (!selected.value) return
  await firewallStore.blockProcess(selected.value.name)
  await firewallStore.load()
  toast(`已禁止「${selected.value.name}」联网`, 'success')
  blockConfirmOpen.value = false
}

async function toggleUnblock(p: ProcessStats) {
  await firewallStore.unblockProcess(p.name)
  await firewallStore.load()
  toast(`已恢复「${p.name}」的网络连接`, 'success')
}

function openAlert(p: ProcessStats) {
  selected.value = p
  alertUpload.value = true
  alertDownload.value = false
  alertOpen.value = true
}

async function createAlert() {
  if (!selected.value) return
  if (!alertUpload.value && !alertDownload.value) {
    toast('请至少选择一个预警方向（上传或下载）', 'error')
    return
  }
  const dir = alertUpload.value && alertDownload.value ? 2 : alertUpload.value ? 0 : 1
  const rule: Rule = {
    id: `R_${selected.value.name}_${Date.now()}`,
    name: `预警-${selected.value.name}`,
    processName: selected.value.name,
    threshold: Math.round(Number(thresholdKb.value) * 1024),
    direction: dir,
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
  await firewallStore.load()
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
                  <Badge
                    v-if="p.category === 'kernel'"
                    variant="outline"
                    class="text-[10px] px-1.5 py-0 h-4 bg-muted/80 text-muted-foreground border-border/80"
                  >
                    系统内核
                  </Badge>
                  <Badge
                    v-else-if="p.category === 'windowsService'"
                    variant="outline"
                    class="text-[10px] px-1.5 py-0 h-4 bg-amber-500/10 text-amber-600 dark:text-amber-400 border-amber-500/20"
                  >
                    系统服务
                  </Badge>
                  <Badge
                    v-if="firewallStore.isBlocked(p.name)"
                    variant="destructive"
                    class="ml-1 text-[10px] px-1.5 py-0 h-4 bg-red-500/15 text-red-500 border-red-500/20"
                  >
                    已断网
                  </Badge>
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
                <div class="flex justify-end gap-1.5">
                  <!-- Kernel process cannot be blocked -->
                  <Badge
                    v-if="p.category === 'kernel'"
                    variant="secondary"
                    class="h-7 text-xs font-normal text-muted-foreground/70 bg-muted/40 cursor-not-allowed"
                    title="Windows 系统内核核心流量，受系统保护无法阻断"
                  >
                    内核保护
                  </Badge>
                  <Button
                    v-else-if="firewallStore.isBlocked(p.name)"
                    size="sm"
                    variant="outline"
                    class="h-7 gap-1 text-xs text-green-600 border-green-500/30 hover:bg-green-500/10"
                    @click="toggleUnblock(p)"
                  >
                    <ShieldCheck class="h-3.5 w-3.5" />
                    放行
                  </Button>
                  <Button
                    v-else
                    size="sm"
                    variant="outline"
                    class="h-7 gap-1 text-xs text-red-500 border-red-500/20 hover:bg-red-500/10"
                    @click="promptBlock(p)"
                  >
                    <ShieldBan class="h-3.5 w-3.5" />
                    断网
                  </Button>
                  <Button
                    size="sm"
                    variant="ghost"
                    class="h-7 gap-1 text-xs"
                    @click="openAlert(p)"
                  >
                    <Bell class="h-3.5 w-3.5" />
                    预警
                  </Button>
                </div>
              </TableCell>
            </TableRow>
          </TableBody>
        </Table>
      </CardContent>
    </Card>

    <!-- Block Confirmation Dialog -->
    <Dialog v-model:open="blockConfirmOpen" title="禁止进程联网">
      <div class="flex flex-col gap-3 py-2">
        <div class="flex items-center gap-3">
          <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-red-500/10 text-red-500">
            <ShieldBan class="h-5 w-5" />
          </div>
          <div>
            <div class="text-sm font-semibold">确认对「{{ selected?.name }}」执行断网？</div>
            <div v-if="selected?.category === 'windowsService'" class="text-xs text-amber-600 dark:text-amber-400 mt-1 font-medium bg-amber-500/10 p-2 rounded border border-amber-500/20">
              ⚠️ 注意：该进程为 Windows 系统后台服务组件（Session 0），阻断联网可能影响部分系统网络功能。
            </div>
            <div v-else class="text-xs text-muted-foreground mt-0.5">
              启用后，该进程发起的全部网络连接请求将在 Windows 内核 ALE 与防火墙层被立即拦截。
            </div>
          </div>
        </div>
      </div>
      <template #footer>
        <Button variant="ghost" @click="blockConfirmOpen = false">取消</Button>
        <Button class="bg-red-600 hover:bg-red-700 text-white" @click="confirmBlock">
          确认禁止联网
        </Button>
      </template>
    </Dialog>

    <!-- Alert Rule Dialog -->
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
        <div class="flex items-center justify-between">
          <Label>预警上传流量</Label>
          <Switch v-model="alertUpload" />
        </div>
        <div class="flex items-center justify-between">
          <Label>预警下载流量</Label>
          <Switch v-model="alertDownload" />
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
