<script setup lang="ts">
import { onMounted, ref } from 'vue'
import type { Rule } from '@/types'
import { Direction, DIRECTION_OPTIONS } from '@/types'
import { useAlertStore } from '@/stores/alertStore'
import { toast } from '@/components/ui/toast'
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select } from '@/components/ui/select'
import { Switch } from '@/components/ui/switch'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Table, TableHeader, TableBody, TableRow, TableHead, TableCell } from '@/components/ui/table'
import { formatSpeed } from '@/composables/useFormatters'

const alertStore = useAlertStore()

const name = ref('')
const processName = ref('')
const thresholdKb = ref(512)
const direction = ref<Direction>(Direction.Upload)
const cooldown = ref(30)

async function createRule() {
  if (!processName.value.trim()) {
    toast('请填写进程名', 'error')
    return
  }
  const rule: Rule = {
    id: `R_${processName.value}_${Date.now()}`,
    name: name.value.trim() || `预警-${processName.value}`,
    processName: processName.value.trim(),
    threshold: Math.round(Number(thresholdKb.value) * 1024),
    direction: Number(direction.value),
    cooldownSec: Number(cooldown.value),
    enabled: true,
    createdAt: Math.floor(Date.now() / 1000),
  }
  await alertStore.createRule(rule)
  await alertStore.loadRules()
  toast('已创建预警规则', 'success')
  name.value = ''
  processName.value = ''
}

async function removeRule(id: string) {
  await alertStore.deleteRule(id)
  toast('已删除规则', 'success')
}

async function toggleEnabled(rule: Rule) {
  await alertStore.updateRule({ ...rule, enabled: !rule.enabled })
}

onMounted(async () => {
  await alertStore.loadRules()
  await alertStore.loadHistory()
})
</script>

<template>
  <div class="flex flex-col gap-4">
    <Card>
      <CardHeader>
        <CardTitle>新建预警规则</CardTitle>
        <CardDescription>当进程上传/下载速率超过阈值时触发通知。</CardDescription>
      </CardHeader>
      <CardContent class="flex flex-col gap-3">
        <div class="grid grid-cols-1 gap-3 md:grid-cols-2">
          <div>
            <Label for="rname">规则名称</Label>
            <Input id="rname" v-model="name" placeholder="可选" class="mt-1" />
          </div>
          <div>
            <Label for="rpname">进程名（支持 * 通配）</Label>
            <Input id="rpname" v-model="processName" placeholder="chrome.exe" class="mt-1" />
          </div>
          <div>
            <Label for="rthr">阈值 (KB/s)</Label>
            <Input id="rthr" v-model="thresholdKb" type="number" class="mt-1" />
          </div>
          <div>
            <Label for="rdir">方向</Label>
            <Select
              id="rdir"
              v-model="direction"
              :options="DIRECTION_OPTIONS"
              class="mt-1"
            />
          </div>
          <div>
            <Label for="rcd">冷却时间 (秒)</Label>
            <Input id="rcd" v-model="cooldown" type="number" class="mt-1" />
          </div>
        </div>
        <div class="flex justify-end">
          <Button @click="createRule">创建规则</Button>
        </div>
      </CardContent>
    </Card>

    <Card>
      <CardHeader>
        <CardTitle>预警规则 ({{ alertStore.rules.length }})</CardTitle>
      </CardHeader>
      <CardContent>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>名称</TableHead>
              <TableHead>进程</TableHead>
              <TableHead class="text-right">阈值</TableHead>
              <TableHead class="text-right">状态</TableHead>
              <TableHead class="text-right">操作</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableRow v-for="r in alertStore.rules" :key="r.id">
              <TableCell>{{ r.name }}</TableCell>
              <TableCell>{{ r.processName }}</TableCell>
              <TableCell class="text-right">{{ formatSpeed(r.threshold) }}</TableCell>
              <TableCell class="text-right">
                <Switch :model-value="r.enabled" @update:model-value="toggleEnabled(r)" />
              </TableCell>
              <TableCell class="text-right">
                <Button size="sm" variant="ghost" @click="removeRule(r.id)">删除</Button>
              </TableCell>
            </TableRow>
            <TableRow v-if="alertStore.rules.length === 0">
              <TableCell colspan="5" class="text-center text-muted-foreground">暂无规则</TableCell>
            </TableRow>
          </TableBody>
        </Table>
      </CardContent>
    </Card>

    <Card>
      <CardHeader>
        <CardTitle>预警历史 ({{ alertStore.history.length }})</CardTitle>
      </CardHeader>
      <CardContent>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>进程</TableHead>
              <TableHead class="text-right">触发速率</TableHead>
              <TableHead class="text-right">阈值</TableHead>
              <TableHead class="text-right">时间</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableRow v-for="h in alertStore.history" :key="h.id">
              <TableCell>{{ h.processName }}</TableCell>
              <TableCell class="text-right">{{ formatSpeed(h.currentRate) }}</TableCell>
              <TableCell class="text-right">{{ formatSpeed(h.threshold) }}</TableCell>
              <TableCell class="text-right text-muted-foreground">
                {{ new Date(h.triggeredAt * 1000).toLocaleTimeString() }}
              </TableCell>
            </TableRow>
            <TableRow v-if="alertStore.history.length === 0">
              <TableCell colspan="4" class="text-center text-muted-foreground">暂无记录</TableCell>
            </TableRow>
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  </div>
</template>
