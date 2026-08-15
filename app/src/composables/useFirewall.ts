import { useFirewallStore } from '@/stores/firewallStore'

/** 进程联网控制与防火墙便捷封装。 */
export function useFirewall() {
  const firewallStore = useFirewallStore()
  return {
    firewallStore,
    load: firewallStore.load,
    apply: firewallStore.apply,
    blockProcess: firewallStore.blockProcess,
    unblockProcess: firewallStore.unblockProcess,
    remove: firewallStore.remove,
    isBlocked: firewallStore.isBlocked,
  }
}
