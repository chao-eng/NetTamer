import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Rule, AlertEvent, AlertHistoryFilter } from '@/types'
import { invokeSafe } from '@/lib/ipc'

const now = () => Math.floor(Date.now() / 1000)

const MOCK_RULES: Rule[] = [
  {
    id: 'R_mock_1',
    name: 'Chrome 上传预警',
    processName: 'chrome.exe',
    threshold: 512 * 1024,
    direction: 0,
    cooldownSec: 30,
    enabled: true,
    createdAt: now() - 3600,
  },
  {
    id: 'R_mock_2',
    name: 'Steam 下载预警',
    processName: 'steam.exe',
    threshold: 5 * 1024 * 1024,
    direction: 1,
    cooldownSec: 60,
    enabled: false,
    createdAt: now() - 1800,
  },
]

const MOCK_HISTORY: AlertEvent[] = [
  {
    id: 'A_mock_1',
    ruleId: 'R_mock_1',
    processName: 'chrome.exe',
    pid: 4172,
    currentRate: 780 * 1024,
    threshold: 512 * 1024,
    triggeredAt: now() - 120,
  },
]

export const useAlertStore = defineStore('alert', () => {
  const rules = ref<Rule[]>([])
  const history = ref<AlertEvent[]>([])

  async function loadRules() {
    const list = await invokeSafe<Rule[]>('list_alert_rules', undefined, MOCK_RULES)
    rules.value = list ?? []
  }

  async function createRule(rule: Rule) {
    await invokeSafe('create_alert_rule', { rule })
    rules.value = [...rules.value, rule]
  }

  async function updateRule(rule: Rule) {
    await invokeSafe('update_alert_rule', { rule })
    rules.value = rules.value.map((r) => (r.id === rule.id ? rule : r))
  }

  async function deleteRule(id: string) {
    await invokeSafe('delete_alert_rule', { id })
    rules.value = rules.value.filter((r) => r.id !== id)
  }

  async function loadHistory(filter?: AlertHistoryFilter) {
    const args = filter ? { filter } : undefined
    const list = await invokeSafe<AlertEvent[]>(
      'get_alert_history',
      args,
      MOCK_HISTORY,
    )
    history.value = list ?? []
  }

  return {
    rules,
    history,
    loadRules,
    createRule,
    updateRule,
    deleteRule,
    loadHistory,
  }
})
