/**
 * NRCS 通知系统集成模块
 *
 * 将 useNotifications 通知系统与 account.store / node.store 连接起来，
 * 对标 NRS 中 nrs.js 直接在 connectionError/checkIfOnAFork/checkAssetDifferences
 * 内调用 $.growl 的模式。
 *
 * 集成点：
 *   1. **account.store.onAssetDifference**：资产余额变化 → notifyAssetReceived/notifyAssetSold/notifyMultipleAssetDifferences
 *      （对标 nrs.js:1596-1644 checkAssetDifferences 内的 $.growl 调用）
 *   2. **node.store.forkWarning**：分叉检测 → notifyForkWarning/notifyForkWarningBaseTarget
 *      （对标 nrs.js:1713/1719 checkIfOnAFork 内的 $.growl 调用）
 *   3. **node.store.serverConnect**：连接断开 → notifyConnectionError
 *      （对标 nrs.js:493 connectionError 内的 $.growl 调用）
 *
 * 用法：在 App.vue 或 main.ts 中调用 `setupNotificationIntegration()`
 */
import { watch } from 'vue'
import { useNotifications } from '@/composables/useNotifications'
import { useAccountStore } from '@/stores/modules/account.store'
import { useNodeStore } from '@/stores/modules/node.store'
import type { AssetDifference } from '@/stores/modules/account.store'

/** 集成是否已初始化（避免重复注册） */
let integrated = false

/**
 * 处理资产余额变化，生成对应通知（对标 nrs.js:1596-1644）。
 *
 * - diff 数量 <= 3：逐个生成通知（收到/出售）
 * - diff 数量 > 3：生成"多个资产变动"通知
 *
 * @param differences 资产余额变化列表
 */
function handleAssetDifferences(differences: AssetDifference[]): void {
  const { notifyAssetReceived, notifyAssetSold, notifyMultipleAssetDifferences } =
    useNotifications()

  // 对标 nrs.js:1577-1644 diff 数量判断
  if (differences.length > 3) {
    notifyMultipleAssetDifferences()
    return
  }

  // 对标 nrs.js:1584-1638 逐个生成通知
  for (const diff of differences) {
    if (diff.difference.charAt(0) !== '-') {
      // 正数 → 收到资产（对标 nrs.js:1597-1616）
      // 注：NRS 在此处还查询资产名称（asset.name），此处简化使用 asset ID
      notifyAssetReceived(diff.asset, diff.difference)
    } else {
      // 负数 → 出售/转移资产（对标 nrs.js:1617-1637）
      notifyAssetSold(diff.asset, diff.difference.substring(1))
    }
  }
}

/**
 * 设置通知系统集成。
 *
 * 在应用启动时（App.vue setup 或 main.ts）调用一次，注册所有回调。
 * 重复调用安全（已初始化时跳过）。
 */
export function setupNotificationIntegration(): void {
  if (integrated) return
  integrated = true

  const accountStore = useAccountStore()
  const nodeStore = useNodeStore()

  // === 1. 资产余额变化通知（对标 nrs.js:1584-1638） ===
  accountStore.onAssetDifference(handleAssetDifferences)

  // === 2. 分叉警告通知（对标 nrs.js:1713/1719） ===
  // watch forkWarning 状态变化，触发对应通知
  watch(
    () => nodeStore.forkWarning,
    (newVal, oldVal) => {
      if (newVal === oldVal) return
      const { notifyForkWarning, notifyForkWarningBaseTarget } = useNotifications()
      if (newVal === 'fork_warning') {
        notifyForkWarning()
      } else if (newVal === 'fork_warning_base_target') {
        notifyForkWarningBaseTarget()
      }
    },
  )

  // === 3. 服务器连接错误通知（对标 nrs.js:493） ===
  // watch serverConnect 状态变化，false 时触发连接错误通知
  watch(
    () => nodeStore.serverConnect,
    (newVal, oldVal) => {
      if (newVal === false && oldVal !== false) {
        const { notifyConnectionError } = useNotifications()
        notifyConnectionError(
          typeof window !== 'undefined' ? window.location.origin : 'server',
          '连接已断开',
        )
      }
    },
  )
}

/**
 * 重置集成状态（仅用于测试）。
 */
export function resetNotificationIntegration(): void {
  integrated = false
}
