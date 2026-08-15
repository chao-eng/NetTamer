import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { ProcessStats, SortField, SortOrder, SystemStats } from '@/types'
import { invokeSafe, listenSafe, type UnlistenFn } from '@/lib/ipc'

const MOCK_PROCESSES: ProcessStats[] = [
  {
    pid: 4,
    name: 'Windows 系统网络 (System)',
    path: '[Windows Kernel]',
    category: 'kernel',
    iconB64: '',
    uploadRate: 512,
    downloadRate: 1024,
    totalUpload: 1048576,
    totalDownload: 10485760,
  },
  {
    pid: 1024,
    name: 'svchost.exe',
    path: 'C:\\Windows\\System32\\svchost.exe',
    category: 'windowsService',
    iconB64: '',
    uploadRate: 2048,
    downloadRate: 16384,
    totalUpload: 524288,
    totalDownload: 4194304,
  },
  {
    pid: 4172,
    name: 'chrome.exe',
    path: 'C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe',
    category: 'userApp',
    iconB64: '',
    uploadRate: 18234,
    downloadRate: 842133,
    totalUpload: 2345678,
    totalDownload: 142345678,
  },
  {
    pid: 8810,
    name: 'steam.exe',
    path: 'C:\\Program Files (x86)\\Steam\\steam.exe',
    category: 'userApp',
    iconB64: '',
    uploadRate: 4096,
    downloadRate: 2310444,
    totalUpload: 512000,
    totalDownload: 523400000,
  },
  {
    pid: 2231,
    name: 'discord.exe',
    path: 'C:\\Users\\admin\\AppData\\Local\\Discord\\discord.exe',
    category: 'userApp',
    iconB64: '',
    uploadRate: 9211,
    downloadRate: 55321,
    totalUpload: 980123,
    totalDownload: 4321098,
  },
]

export const useProcessStore = defineStore('process', () => {
  const processes = ref<ProcessStats[]>([])
  const totalUploadRate = ref(0)
  const totalDownloadRate = ref(0)
  const isMonitoring = ref(false)
  const refreshInterval = ref(1000)
  const sortField = ref<SortField>('uploadRate')
  const sortOrder = ref<SortOrder>('desc')
  const searchQuery = ref('')

  const sortedProcesses = computed<ProcessStats[]>(() => {
    const q = searchQuery.value.trim().toLowerCase()
    let list = processes.value
    if (q) {
      list = list.filter((p) => p.name.toLowerCase().includes(q))
    }
    const field = sortField.value
    const dir = sortOrder.value === 'asc' ? 1 : -1
    return [...list].sort((a, b) => {
      if (field === 'name') return a.name.localeCompare(b.name) * dir
      if (field === 'pid') return (a.pid - b.pid) * dir
      const av = (a as unknown as Record<string, number>)[field] ?? 0
      const bv = (b as unknown as Record<string, number>)[field] ?? 0
      return (av - bv) * dir
    })
  })

  async function fetchList() {
    const list = await invokeSafe<ProcessStats[]>(
      'get_process_list',
      undefined,
      MOCK_PROCESSES,
    )
    processes.value = list ?? []
  }

  async function setRefreshInterval(ms: number) {
    refreshInterval.value = ms
    await invokeSafe('set_refresh_interval', { ms })
  }

  function setSort(field: SortField) {
    if (sortField.value === field) {
      sortOrder.value = sortOrder.value === 'asc' ? 'desc' : 'asc'
    } else {
      sortField.value = field
      sortOrder.value = 'desc'
    }
  }

  async function start() {
    await invokeSafe('start_monitoring')
    await invokeSafe('set_refresh_interval', { ms: refreshInterval.value })
    isMonitoring.value = true
  }

  async function stop() {
    await invokeSafe('stop_monitoring')
    isMonitoring.value = false
  }

  async function bindEvents(): Promise<UnlistenFn[]> {
    const unlisteners: UnlistenFn[] = []
    unlisteners.push(
      await listenSafe<ProcessStats[]>('speed:update', (list) => {
        if (list) processes.value = list
      }),
    )
    unlisteners.push(
      await listenSafe<SystemStats>('system:stats', (s) => {
        if (s) {
          totalUploadRate.value = s.totalUploadRate
          totalDownloadRate.value = s.totalDownloadRate
        }
      }),
    )
    return unlisteners
  }

  return {
    processes,
    totalUploadRate,
    totalDownloadRate,
    isMonitoring,
    refreshInterval,
    sortField,
    sortOrder,
    searchQuery,
    sortedProcesses,
    fetchList,
    setRefreshInterval,
    setSort,
    start,
    stop,
    bindEvents,
  }
})
