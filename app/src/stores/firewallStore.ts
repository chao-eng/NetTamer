import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { FirewallRule } from '@/types'
import { invokeSafe } from '@/lib/ipc'

const now = () => Math.floor(Date.now() / 1000)

const MOCK_RULES: FirewallRule[] = [
  {
    id: 'FR_mock_1',
    name: 'Block: steam.exe',
    processName: 'steam.exe',
    active: true,
    createdAt: now() - 600,
  },
]

export const useFirewallStore = defineStore('firewall', () => {
  const rules = ref<FirewallRule[]>([])

  async function load() {
    const list = await invokeSafe<FirewallRule[]>(
      'list_firewall_rules',
      undefined,
      MOCK_RULES,
    )
    rules.value = list ?? []
  }

  async function apply(rule: FirewallRule) {
    await invokeSafe('apply_firewall_rule', { rule })
    const idx = rules.value.findIndex((r) => r.id === rule.id)
    if (idx >= 0) {
      rules.value[idx] = rule
    } else {
      rules.value = [...rules.value, rule]
    }
  }

  async function blockProcess(processName: string) {
    const trimmed = processName.trim()
    if (!trimmed) return
    const rule: FirewallRule = {
      id: `FR_${trimmed}_${Date.now()}`,
      name: `Block: ${trimmed}`,
      processName: trimmed,
      active: true,
      createdAt: Math.floor(Date.now() / 1000),
    }
    await apply(rule)
  }

  async function remove(id: string) {
    await invokeSafe('remove_firewall_rule', { id })
    rules.value = rules.value.filter((r) => r.id !== id)
  }

  async function unblockProcess(processName: string) {
    const target = processName.trim().toLowerCase()
    const found = rules.value.find(
      (r) =>
        r.processName.toLowerCase() === target ||
        r.processName.toLowerCase() === `${target}.exe` ||
        `${r.processName.toLowerCase()}.exe` === target,
    )
    if (found) {
      await remove(found.id)
    }
  }

  function isBlocked(processName: string): boolean {
    const target = processName.trim().toLowerCase()
    return rules.value.some(
      (r) =>
        r.active &&
        (r.processName.toLowerCase() === target ||
          r.processName.toLowerCase() === `${target}.exe` ||
          `${r.processName.toLowerCase()}.exe` === target),
    )
  }

  return { rules, load, apply, blockProcess, remove, unblockProcess, isBlocked }
})
