<script setup lang="ts">
import { onMounted, ref, computed } from 'vue'
import { useFirewallStore } from '@/stores/firewallStore'
import { useProcessStore } from '@/stores/processStore'
import { toast } from '@/components/ui/toast'
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Table, TableHeader, TableBody, TableRow, TableHead, TableCell } from '@/components/ui/table'
import { ShieldAlert, ShieldCheck, ShieldBan, Plus, Trash2, Globe, Activity } from 'lucide-vue-next'

const firewallStore = useFirewallStore()
const processStore = useProcessStore()

const processName = ref('')

const runningProcesses = computed(() => {
  return processStore.processes
    .filter((p) => p.uploadRate > 0 || p.downloadRate > 0 || p.totalUpload > 0 || p.totalDownload > 0)
    .slice(0, 10)
})

async function addBlockRule() {
  const name = processName.value.trim()
  if (!name) {
    toast('请输入需要禁止联网的进程名或路径', 'error')
    return
  }

  await firewallStore.blockProcess(name)
  await firewallStore.load()
  toast(`已成功对「${name}」开启联网阻断`, 'success')
  processName.value = ''
}

function quickSelect(name: string) {
  processName.value = name
}

async function removeRule(id: string, name: string) {
  await firewallStore.remove(id)
  toast(`已解除「${name}」的联网阻断`, 'success')
}

onMounted(async () => {
  await firewallStore.load()
})
</script>

<template>
  <div class="flex flex-col gap-5">
    <!-- Top banner & add card -->
    <Card class="border-border/60 bg-card/80 backdrop-blur-sm">
      <CardHeader>
        <div class="flex items-center gap-2">
          <div class="flex h-9 w-9 items-center justify-center rounded-lg bg-red-500/10 text-red-500">
            <ShieldBan class="h-5 w-5" />
          </div>
          <div>
            <CardTitle>添加进程联网阻断</CardTitle>
            <CardDescription class="mt-0.5">
              基于 Windows Filtering Platform (WFP) 内核 ALE 层直接阻断指定程序的所有入站与出站连接。
            </CardDescription>
          </div>
        </div>
      </CardHeader>
      <CardContent class="flex flex-col gap-4">
        <div class="flex flex-col gap-2 sm:flex-row sm:items-end">
          <div class="flex-1">
            <Label for="fpname" class="text-xs font-medium text-muted-foreground">进程文件名或全路径</Label>
            <Input
              id="fpname"
              v-model="processName"
              placeholder="例如: steam.exe, chrome.exe 或完整路径"
              class="mt-1"
              @keydown.enter="addBlockRule"
            />
          </div>
          <Button class="gap-1.5 bg-red-600 hover:bg-red-700 text-white" @click="addBlockRule">
            <Plus class="h-4 w-4" />
            一键禁止联网
          </Button>
        </div>

        <!-- Quick selection from active network processes -->
        <div v-if="runningProcesses.length > 0" class="flex flex-wrap items-center gap-1.5 pt-1">
          <span class="text-xs text-muted-foreground flex items-center gap-1">
            <Activity class="h-3.5 w-3.5 text-primary" /> 活跃流量进程:
          </span>
          <Badge
            v-for="p in runningProcesses"
            :key="p.pid"
            variant="outline"
            class="cursor-pointer transition-all hover:bg-primary/10 hover:border-primary/40"
            @click="quickSelect(p.name)"
          >
            {{ p.name }}
          </Badge>
        </div>
      </CardContent>
    </Card>

    <!-- Blocked rules list -->
    <Card class="border-border/60 bg-card/80 backdrop-blur-sm">
      <CardHeader class="flex flex-row items-center justify-between pb-3">
        <div>
          <CardTitle class="flex items-center gap-2 text-base">
            <ShieldAlert class="h-4 w-4 text-red-500" />
            已阻断进程列表
            <Badge variant="secondary" class="ml-1 font-mono text-xs">
              {{ firewallStore.rules.length }}
            </Badge>
          </CardTitle>
          <CardDescription class="mt-1 text-xs">
            下列进程的所有 TCP / UDP 网络请求已在内核层面被静默拦截。
          </CardDescription>
        </div>
      </CardHeader>
      <CardContent>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>目标进程</TableHead>
              <TableHead>拦截状态</TableHead>
              <TableHead>拦截策略</TableHead>
              <TableHead class="text-right">操作</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableRow v-for="r in firewallStore.rules" :key="r.id" class="group">
              <TableCell class="font-medium">
                <div class="flex items-center gap-2">
                  <div class="flex h-7 w-7 items-center justify-center rounded-md bg-muted/60 text-muted-foreground">
                    <Globe class="h-4 w-4" />
                  </div>
                  <span class="font-mono text-sm">{{ r.processName }}</span>
                </div>
              </TableCell>
              <TableCell>
                <Badge variant="destructive" class="gap-1 bg-red-500/15 text-red-600 dark:text-red-400 border-red-500/20">
                  <ShieldBan class="h-3 w-3" />
                  已禁止联网
                </Badge>
              </TableCell>
              <TableCell class="text-xs text-muted-foreground">
                双向拦截 (ALE IPv4 + IPv6)
              </TableCell>
              <TableCell class="text-right">
                <Button
                  size="sm"
                  variant="outline"
                  class="gap-1 text-xs hover:bg-green-500/10 hover:text-green-600 hover:border-green-500/30"
                  @click="removeRule(r.id, r.processName)"
                >
                  <ShieldCheck class="h-3.5 w-3.5 text-green-500" />
                  解除阻断
                </Button>
              </TableCell>
            </TableRow>
            <TableRow v-if="firewallStore.rules.length === 0">
              <TableCell colspan="4" class="h-32 text-center text-muted-foreground">
                <div class="flex flex-col items-center justify-center gap-1.5">
                  <ShieldCheck class="h-8 w-8 text-muted-foreground/40" />
                  <span>当前暂无被阻断联网的进程</span>
                  <span class="text-xs text-muted-foreground/60">所有进程均可正常访问网络</span>
                </div>
              </TableCell>
            </TableRow>
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  </div>
</template>
