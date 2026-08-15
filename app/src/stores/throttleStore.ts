import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Policy } from '@/types'
import { invokeSafe } from '@/lib/ipc'

const now = () => Math.floor(Date.now() / 1000)

const MOCK_POLICIES: Policy[] = [
  {
    id: 'NT_mock_1',
    name: 'NT_steam.exe',
    processName: 'steam.exe',
    rateLimitBps: 10 * 1000 * 1000,
    limitUpload: true,
    limitDownload: true,
    active: true,
    createdAt: now() - 600,
  },
]

export const useThrottleStore = defineStore('throttle', () => {
  const policies = ref<Policy[]>([])

  async function load() {
    const list = await invokeSafe<Policy[]>(
      'list_throttle_policies',
      undefined,
      MOCK_POLICIES,
    )
    policies.value = list ?? []
  }

  async function apply(policy: Policy) {
    await invokeSafe('apply_throttle_policy', { policy })
    const idx = policies.value.findIndex((p) => p.id === policy.id)
    if (idx >= 0) {
      policies.value[idx] = policy
    } else {
      policies.value = [...policies.value, policy]
    }
  }

  async function remove(id: string) {
    await invokeSafe('remove_throttle_policy', { id })
    policies.value = policies.value.filter((p) => p.id !== id)
  }

  return { policies, load, apply, remove }
})
