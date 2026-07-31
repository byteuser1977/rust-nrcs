/******************************************************************************
 * useForging composable —— 锻造状态机（对标 nrs.modals.forging.js）。
 *
 * 移植 nrs.modals.forging.js（204 行）的核心逻辑：
 *   - updateForgingStatus(secretPhrase?)：6 种错误状态判定 + getForging 查询
 *   - getForgingTooltip(data)：根据锻造者信息生成 tooltip
 *   - checkForgingPreconditions()：forgingIndicator.click 前置校验（6 条件）
 *   - startForging/stopForging：封装 API 调用 + 状态更新
 *
 * 状态机分支（对标 nrs.modals.forging.js:131-191）：
 *   1. !isForgingSupported → 隐藏指示器
 *   2. isLightClient → NOT_FORGING（error_forging_light_client）
 *   3. !publicKey → NOT_FORGING（error_forging_no_public_key）
 *   4. isLeased → NOT_FORGING（error_forging_lease）
 *   5. effectiveBalance == 0 → NOT_FORGING（error_forging_effective_balance）
 *   6. downloadingBlockchain → NOT_FORGING（error_forging_blockchain_downloading）
 *   7. isScanning → NOT_FORGING（error_forging_blockchain_rescanning）
 *   8. needsAdminPassword && !adminPassword && (!secretPhrase || !isForgingSafe) → 不变
 *   9. else → getForging 查询，解析 account/generators/errorDescription
 *
 * 使用模块级单例状态，确保 Header 指示器 / Generators 页 / ForgingModal 共享同一状态。
 ******************************************************************************/
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useAccountStore } from '@/stores/modules/account.store'
import { useNodeStore } from '@/stores/modules/node.store'
import { isForgingSupported, isForgingSafe, getAdminPassword } from '@/utils/feature-detection'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { formatAmount } from '@/utils/format'

// ============================================================================
// 锻造状态常量（对标 NRS.constants.FORGING / NOT_FORGING / UNKNOWN）
// ============================================================================

export type ForgingStatus = 'forging' | 'not_forging' | 'unknown'

// ============================================================================
// 模块级单例状态（跨组件共享）
// ============================================================================

/** 当前锻造状态（对标 NRS.forgingStatus） */
const forgingStatus = ref<ForgingStatus>('not_forging')
/** 当前锻造 tooltip 文本（对标 NRS.updateForgingTooltip） */
const forgingTooltip = ref<string>('')
/** 当前账户是否正在锻造（对标 NRS.isAccountForging） */
const isAccountForging = ref<boolean>(false)
/** 是否正在加载锻造状态 */
const isLoading = ref<boolean>(false)
/** 锻造前置校验错误（非空时表示无法锻造的原因） */
const forgingError = ref<string>('')

// ============================================================================
// composable
// ============================================================================

/**
 * 锻造状态机 composable（对标 nrs.modals.forging.js）。
 *
 * @returns 锻造状态、tooltip、更新方法、start/stop 方法
 */
export function useForging() {
  const { t } = useI18n()
  const accountStore = useAccountStore()
  const nodeStore = useNodeStore()

  // --------------------------------------------------------------------------
  // 计算属性
  // --------------------------------------------------------------------------

  /** 锻造状态 i18n 标签（对标 forgingIndicator.find("span").html($.t(status))） */
  const forgingStatusLabel = computed<string>(() => {
    switch (forgingStatus.value) {
      case 'forging':
        return t('forging.forging')
      case 'not_forging':
        return t('forging.notForging')
      default:
        return t('forging.unknown')
    }
  })

  /** 是否支持锻造（对标 NRS.isForgingSupported()） */
  const canForge = computed<boolean>(() => isForgingSupported())

  // --------------------------------------------------------------------------
  // 辅助函数
  // --------------------------------------------------------------------------

  /**
   * 判断账户余额是否已出租（对标 NRS.isLeased）。
   *
   * 当 leasingStatus.label === 'leased_out' 时表示余额已出租给其他账户。
   *
   * @returns 是否已出租
   */
  function isLeased(): boolean {
    return accountStore.leasingStatus.label === 'leased_out'
  }

  /**
   * 根据锻造者信息生成 tooltip（对标 nrs.modals.forging.js:103-109 getForgingTooltip）。
   *
   * @param data 锻造者信息（含 account/accountRS），为空时表示当前账户
   * @returns tooltip 文本
   */
  function getForgingTooltip(data: { account?: string; accountRS?: string } | null | undefined): string {
    if (!data || !data.account || data.account === accountStore.accountId) {
      isAccountForging.value = true
      return t('forging.tooltipForging', { balance: formatAmount(accountStore.effectiveBalance) })
    }
    return t('forging.tooltipAnotherAccount', { accountRS: data.accountRS || data.account })
  }

  // --------------------------------------------------------------------------
  // 核心状态机
  // --------------------------------------------------------------------------

  /**
   * 更新锻造状态（对标 nrs.modals.forging.js:123-201 updateForgingStatus）。
   *
   * 按顺序检查 6 种错误条件，若全部通过则调用 getForging 查询实际状态。
   *
   * @param secretPhrase 可选密码短语（needsAdminPassword 且无 adminPassword 时使用）
   */
  async function updateForgingStatus(secretPhrase?: string): Promise<void> {
    // 1. 不支持锻造 → 隐藏指示器（对标 :125-128）
    if (!isForgingSupported()) {
      forgingStatus.value = 'not_forging'
      forgingTooltip.value = ''
      return
    }

    let status: ForgingStatus = forgingStatus.value
    let tooltip = forgingTooltip.value

    // 2. 轻客户端（对标 :131-133）
    if (nodeStore.isLightClient) {
      status = 'not_forging'
      tooltip = t('forging.errorLightClient')
    }
    // 3. 无公钥（对标 :134-136）
    else if (!accountStore.publicKey) {
      status = 'not_forging'
      tooltip = t('forging.errorNoPublicKey')
    }
    // 4. 余额已出租（对标 :137-139）
    else if (isLeased()) {
      status = 'not_forging'
      tooltip = t('forging.errorLease')
    }
    // 5. 有效余额为零（对标 :140-142）
    else if (accountStore.effectiveBalance === 0) {
      status = 'not_forging'
      tooltip = t('forging.errorEffectiveBalance')
    }
    // 6. 区块链下载中（对标 :143-145）
    else if (nodeStore.downloadingBlockchain) {
      status = 'not_forging'
      tooltip = t('forging.errorBlockchainDownloading')
    }
    // 7. 正在扫描（对标 :146-148）
    else if (nodeStore.isScanning) {
      status = 'not_forging'
      tooltip = t('forging.errorBlockchainRescanning')
    }
    // 8. 需要 adminPassword 但未提供且不安全（对标 :149-150，不变状态）
    else {
      // 9. 调用 getForging 查询实际状态（对标 :152-191）
      try {
        isLoading.value = true
        const params: Record<string, any> = {}
        const adminPwd = getAdminPassword()
        if (adminPwd) {
          params.adminPassword = adminPwd
        }
        if (secretPhrase && !adminPwd) {
          params.secretPhrase = secretPhrase
        }

        const response: any = await nrcsApi.getForging()
        isAccountForging.value = false

        if (response && 'account' in response) {
          // 单账户锻造中（对标 :161-164）
          status = 'forging'
          tooltip = getForgingTooltip(response)
          isAccountForging.value = true
        } else if (response && 'generators' in response) {
          // 多账户锻造者列表（对标 :165-185）
          const generators: any[] = response.generators || []
          if (generators.length === 0) {
            status = 'not_forging'
            tooltip = t('forging.tooltipNotStarted')
          } else {
            status = 'forging'
            if (generators.length === 1) {
              tooltip = getForgingTooltip(generators[0])
            } else {
              tooltip = t('forging.tooltipMultipleAccounts', { generators: generators.length })
              for (const g of generators) {
                if (g.account === accountStore.accountId) {
                  isAccountForging.value = true
                }
              }
              tooltip += isAccountForging.value
                ? ', ' + t('forging.currentAccountForging')
                : ', ' + t('forging.currentAccountNotForging')
            }
          }
        } else {
          // 未知错误（对标 :187-190）
          status = 'unknown'
          tooltip = response?.errorDescription || t('forging.tooltipUnknown')
        }
      } catch (e: any) {
        status = 'unknown'
        tooltip = e?.errorDescription || e?.message || t('forging.tooltipUnknown')
      } finally {
        isLoading.value = false
      }
    }

    // 应用状态（对标 :193-200）
    forgingStatus.value = status
    if (status === 'not_forging') {
      isAccountForging.value = false
    }
    forgingTooltip.value = tooltip
  }

  /**
   * 锻造前置校验（对标 nrs.modals.forging.js:60-95 forgingIndicator.click）。
   *
   * 检查 6 种无法锻造的条件，返回错误消息或 null（可锻造）。
   * 用于 Header 指示器点击和 Generators 页按钮。
   *
   * @returns 错误消息（无法锻造时）或 null（可继续显示 start/stop modal）
   */
  function checkForgingPreconditions(): string | null {
    // 1. 轻客户端（对标 :64-67）
    if (nodeStore.isLightClient) {
      return t('forging.errorLightClient')
    }
    // 2. 区块链下载中（对标 :68-71）
    if (nodeStore.downloadingBlockchain) {
      return t('forging.errorBlockchainDownloading')
    }
    // 3. 正在扫描（对标 :72-75）
    if (nodeStore.isScanning) {
      return t('forging.errorBlockchainRescanning')
    }
    // 4. 无公钥（对标 :76-79）
    if (!accountStore.publicKey) {
      return t('forging.errorNoPublicKey')
    }
    // 5. 有效余额为零（对标 :80-89）
    if (accountStore.effectiveBalance === 0) {
      if (isLeased()) {
        return t('forging.errorLease')
      }
      return t('forging.errorEffectiveBalance')
    }
    return null
  }

  // --------------------------------------------------------------------------
  // start/stop forging
  // --------------------------------------------------------------------------

  /**
   * 开始锻造（对标 nrs.modals.forging.js:22-37 startForgingComplete）。
   *
   * @param secretPhrase 密码短语
   * @returns 是否成功
   */
  async function startForging(secretPhrase: string): Promise<boolean> {
    try {
      const response: any = await nrcsApi.startForging(secretPhrase)
      if (response && 'deadline' in response) {
        forgingStatus.value = 'forging'
        isAccountForging.value = true
        forgingTooltip.value = getForgingTooltip(null)
        await updateForgingStatus()
        return true
      }
      return false
    } catch (e: any) {
      throw new Error(e?.errorDescription || e?.message || t('forging.startError'))
    }
  }

  /**
   * 停止锻造（对标 nrs.modals.forging.js:39-58 stopForgingComplete）。
   *
   * @param secretPhrase 可选密码短语
   * @returns 是否成功
   */
  async function stopForging(secretPhrase?: string): Promise<boolean> {
    try {
      const response: any = await nrcsApi.stopForging(secretPhrase)
      if (response && (response.foundAndStopped || (response.stopped && response.stopped > 0))) {
        isAccountForging.value = false
        if (!response.forgersCount || response.forgersCount === 0) {
          forgingStatus.value = 'not_forging'
        }
        await updateForgingStatus()
        return true
      }
      return false
    } catch (e: any) {
      throw new Error(e?.errorDescription || e?.message || t('forging.stopError'))
    }
  }

  /**
   * 重置锻造状态（登出时调用）。
   */
  function resetForgingState(): void {
    forgingStatus.value = 'not_forging'
    forgingTooltip.value = ''
    isAccountForging.value = false
    isLoading.value = false
    forgingError.value = ''
  }

  return {
    // 状态
    forgingStatus,
    forgingStatusLabel,
    forgingTooltip,
    isAccountForging,
    isLoading,
    canForge,
    // 方法
    updateForgingStatus,
    checkForgingPreconditions,
    getForgingTooltip,
    startForging,
    stopForging,
    resetForgingState,
  }
}
