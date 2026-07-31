/**
 * NRCS 通知系统 composable
 *
 * 对标参考：
 *   - `nrs.notifications.js`：交易类型通知计数（notificationCount/notificationTS/lastKnownTransaction）
 *   - `nrs.js` 中的 `$.growl` 调用：Toast 弹窗通知（资产变化/分叉警告/连接错误等）
 *   - `3rdparty/growl.js`：Bootstrap Growl 库（type/message/delay/position）
 *
 * 设计：
 *   - **Toast 通知**：基于 ui.store 的 addNotification，提供 NRCS 风格的便捷方法
 *     （notifySuccess/notifyError/notifyWarning/notifyInfo + NRCS 专用方法）
 *   - **交易类型计数**：模块级 reactive state（单例），按 type/subtype 统计新收到交易数
 *     - notificationCount：未读数量
 *     - notificationTS：已读时间戳（后续交易 timestamp > 此值才计数）
 *     - lastKnownTransaction：该类型最近一笔交易
 *   - **IndexedDB 持久化**：notification_timestamps 存储已读时间戳，对标 nrs.notifications.js:97-122
 *
 * 复用：
 *   - ui.store.addNotification/removeNotification（Toast 渲染）
 *   - utils/nrcs-storage.ts（IndexedDB 持久化）
 *   - constants/transaction-types.ts（TRANSACTION_TYPES 类型定义）
 *   - api/modules/nrcs.api.ts（getBlockchainTransactions/getTime/getAccountPhasedTransactionCount）
 */
import { ref, readonly, computed } from 'vue'
import { useUiStore } from '@/stores/modules/ui.store'
import { storageSelect, storageInsert, storageUpdate } from '@/utils/nrcs-storage'
import { TRANSACTION_TYPES } from '@/constants/transaction-types'
import { nrcsApi } from '@/api/modules'
import type { NrcsTransaction } from '@/api/modules'

// ============================================================================
// 类型定义
// ============================================================================

/** Toast 通知类型（对标 growl.js:45-48 的 alert-* 类） */
export type ToastType = 'success' | 'warning' | 'info' | 'error'

/** Toast 通知选项 */
export interface ToastOptions {
  /** 通知类型（默认 info） */
  type?: ToastType
  /** 标题（可选） */
  title?: string
  /** 自动关闭延时 ms，0 表示不自动关闭（默认 5000，对标 growl.js:184 delay: 5000） */
  duration?: number
}

/** 交易子类型通知状态（对标 nrs.notifications.js 中 subTypeDict 的通知相关字段） */
export interface TransactionSubTypeNotifyState {
  /** 未读通知数（对标 subTypeDict.notificationCount） */
  notificationCount: number
  /** 已读时间戳（对标 subTypeDict.notificationTS，后续交易 timestamp > 此值才计数） */
  notificationTS: number
  /** 该类型最近一笔交易（对标 subTypeDict.lastKnownTransaction） */
  lastKnownTransaction?: NrcsTransaction
}

/** 交易类型通知状态（对标 nrs.notifications.js 中 typeDict 的通知相关字段） */
export interface TransactionTypeNotifyState {
  /** 该类型下所有子类型的通知状态 */
  subTypes: Record<number, TransactionSubTypeNotifyState>
  /** 该类型总未读数（对标 typeDict.notificationCount） */
  notificationCount: number
}

// ============================================================================
// 模块级单例状态（Toast 通知 + 交易类型计数）
// ============================================================================

/**
 * 交易类型通知计数表（模块级单例，对标 NRS.transactionTypes 中注入的 notification* 字段）。
 *
 * 结构：typeIndex → TransactionTypeNotifyState
 * 在 useNotifications() 首次调用时从 TRANSACTION_TYPES 初始化。
 */
const transactionNotifyStates = ref<Record<number, TransactionTypeNotifyState>>({})

/** 通知计数表是否已初始化 */
const initialized = ref(false)

/** 未确认交易数（对标 nrs.notifications.js:217-226 setUnconfirmedNotifications） */
const unconfirmedCount = ref(0)

/** Phasing 待审批交易数（对标 nrs.notifications.js:228-239 setPhasingNotifications） */
const phasingCount = ref(0)

/** Shuffling 进行中数（对标 nrs.notifications.js:241-252 setShufflingNotifications） */
const shufflingCount = ref(0)

// ============================================================================
// 内部工具
// ============================================================================

/**
 * 初始化交易类型通知计数表（对标 nrs.notifications.js:180-194 loadNotificationsFromTimestamps 的初始化部分）。
 *
 * 从 TRANSACTION_TYPES 构建空的计数表，所有 notificationCount=0。
 */
function initTransactionNotifyStates(): void {
  if (initialized.value) return
  const states: Record<number, TransactionTypeNotifyState> = {}
  for (const typeIndex of Object.keys(TRANSACTION_TYPES)) {
    const typeDef = TRANSACTION_TYPES[Number(typeIndex)]
    const subTypes: Record<number, TransactionSubTypeNotifyState> = {}
    for (const subTypeIndex of Object.keys(typeDef.subTypes)) {
      subTypes[Number(subTypeIndex)] = {
        notificationCount: 0,
        notificationTS: 0,
      }
    }
    states[Number(typeIndex)] = {
      subTypes,
      notificationCount: 0,
    }
  }
  transactionNotifyStates.value = states
  initialized.value = true
}

/**
 * 从 IndexedDB 加载已读时间戳并初始化通知计数（对标 nrs.notifications.js:174-194 loadNotificationsFromTimestamps）。
 *
 * @param currentTime 服务端当前时间（对标 response.time）
 * @param tsDictString IndexedDB 中存储的时间戳 JSON
 * @param account 当前登录账户 ID（数字形式，对标 NRS.account）
 */
async function loadNotificationsFromTimestamps(
  currentTime: number,
  tsDictString: string,
  account: string,
): Promise<void> {
  initTransactionNotifyStates()

  let tsDict: Record<string, number> = {}
  if (tsDictString) {
    try {
      tsDict = JSON.parse(tsDictString)
    } catch {
      tsDict = {}
    }
  }

  // 对标 nrs.notifications.js:180-191 设置 notificationTS
  for (const typeIndex of Object.keys(transactionNotifyStates.value)) {
    const typeState = transactionNotifyStates.value[Number(typeIndex)]
    typeState.notificationCount = 0
    for (const subTypeIndex of Object.keys(typeState.subTypes)) {
      const tsKey = `ts_${typeIndex}_${subTypeIndex}`
      const subTypeState = typeState.subTypes[Number(subTypeIndex)]
      if (tsDict[tsKey] !== undefined) {
        subTypeState.notificationTS = tsDict[tsKey]
      } else {
        subTypeState.notificationTS = currentTime
      }
      subTypeState.notificationCount = 0
    }
  }

  // 对标 nrs.notifications.js:192 initNotificationCounts
  await initNotificationCounts(currentTime, account)
  // 对标 nrs.notifications.js:193 saveNotificationTimestamps
  await saveNotificationTimestamps()
}

/**
 * 从区块链交易初始化通知计数（对标 nrs.notifications.js:147-172 initNotificationCounts）。
 *
 * 拉取最近 14 天的 100 笔交易，统计 recipient == 当前账户 且 timestamp > notificationTS 的交易。
 *
 * @param time 服务端当前时间
 * @param account 当前登录账户 ID（数字形式，对标 NRS.account）
 */
async function initNotificationCounts(time: number, account: string): Promise<void> {
  if (!account) {
    console.warn('[notifications] initNotificationCounts: account 为空，跳过')
    return
  }
  // 对标 nrs.notifications.js:148 最近 14 天
  const fromTS = time - 60 * 60 * 24 * 14
  try {
    // 对标 nrs.notifications.js:149-154 getBlockchainTransactions+（timestamp + firstIndex=0 + lastIndex=99）
    const response = await nrcsApi.getBlockchainTransactions(
      account,
      0, // firstIndex
      99, // lastIndex
      undefined,
      undefined,
      { timestamp: fromTS },
    )

    if (response.transactions && response.transactions.length) {
      for (const t of response.transactions) {
        const typeIdx = t.type ?? 0
        const subTypeIdx = t.subtype ?? 0
        const typeState = transactionNotifyStates.value[typeIdx]
        if (!typeState) continue
        const subTypeState = typeState.subTypes[subTypeIdx]
        if (!subTypeState) continue

        // 对标 nrs.notifications.js:159 recipient == NRS.account 且 receiverPage 存在
        const subTypeDef = TRANSACTION_TYPES[typeIdx]?.subTypes[subTypeIdx]
        if (t.recipient && t.recipient === account && subTypeDef?.receiverPage) {
          // 对标 nrs.notifications.js:160-162 更新 lastKnownTransaction
          if (
            !subTypeState.lastKnownTransaction ||
            (subTypeState.lastKnownTransaction.timestamp ?? 0) < (t.timestamp ?? 0)
          ) {
            subTypeState.lastKnownTransaction = t
          }
          // 对标 nrs.notifications.js:163-166 timestamp > notificationTS 才计数
          if ((t.timestamp ?? 0) > subTypeState.notificationTS) {
            typeState.notificationCount += 1
            subTypeState.notificationCount += 1
          }
        }
      }
    }
  } catch (e) {
    console.warn('[notifications] initNotificationCounts 失败:', e)
  }
}

/**
 * 保存已读时间戳到 IndexedDB（对标 nrs.notifications.js:97-122 saveNotificationTimestamps）。
 */
async function saveNotificationTimestamps(): Promise<void> {
  const tsDict: Record<string, number> = {}
  for (const typeIndex of Object.keys(transactionNotifyStates.value)) {
    const typeState = transactionNotifyStates.value[Number(typeIndex)]
    for (const subTypeIndex of Object.keys(typeState.subTypes)) {
      const tsKey = `ts_${typeIndex}_${subTypeIndex}`
      tsDict[tsKey] = typeState.subTypes[Number(subTypeIndex)].notificationTS
    }
  }
  const tsDictString = JSON.stringify(tsDict)

  try {
    // 对标 nrs.notifications.js:106-121 storageSelect + storageUpdate/storageInsert
    const result = await storageSelect<{ id: string; contents: string }>('data', [
      { id: 'notification_timestamps' },
    ])
    if (result && result.length > 0) {
      await storageUpdate('data', { contents: tsDictString }, [{ id: 'notification_timestamps' }])
    } else {
      await storageInsert('data', 'id', {
        id: 'notification_timestamps',
        contents: tsDictString,
      })
    }
  } catch (e) {
    console.warn('[notifications] saveNotificationTimestamps 失败:', e)
  }
}

// ============================================================================
// 主 composable
// ============================================================================

/**
 * NRCS 通知系统 composable。
 *
 * 用法：
 * ```ts
 * const { notifySuccess, notifyAssetReceived, updateNotifications } = useNotifications()
 * notifySuccess('操作成功')
 * await updateNotifications()
 * ```
 *
 * Toast 通知底层使用 ui.store（addNotification/removeNotification），
 * 交易类型计数使用模块级单例 state。
 */
export function useNotifications() {
  const uiStore = useUiStore()

  // --- Toast 通知（对标 $.growl） ---

  /**
   * 添加 Toast 通知（对标 $.growl(message, options)）。
   *
   * @param message 消息内容
   * @param options 通知选项（type/title/duration）
   * @returns 通知 ID（用于手动关闭）
   */
  function notify(message: string, options?: ToastOptions): number {
    return uiStore.addNotification({
      type: options?.type ?? 'info',
      title: options?.title ?? '',
      message,
      duration: options?.duration ?? 5000,
    })
  }

  /**
   * 成功通知（对标 $.growl(msg, { type: "success" })）。
   */
  function notifySuccess(message: string, options?: Omit<ToastOptions, 'type'>): number {
    return notify(message, { ...options, type: 'success' })
  }

  /**
   * 错误通知（对标 $.growl(msg, { type: "danger" })）。
   */
  function notifyError(message: string, options?: Omit<ToastOptions, 'type'>): number {
    return notify(message, { ...options, type: 'error' })
  }

  /**
   * 警告通知（对标 $.growl(msg, { type: "warning" })）。
   */
  function notifyWarning(message: string, options?: Omit<ToastOptions, 'type'>): number {
    return notify(message, { ...options, type: 'warning' })
  }

  /**
   * 信息通知（对标 $.growl(msg, { type: "info" })）。
   */
  function notifyInfo(message: string, options?: Omit<ToastOptions, 'type'>): number {
    return notify(message, { ...options, type: 'info' })
  }

  /**
   * 移除指定 Toast 通知。
   */
  function dismissNotification(id: number): void {
    uiStore.removeNotification(id)
  }

  /**
   * 清除所有 Toast 通知。
   */
  function clearAllToasts(): void {
    uiStore.clearAllNotifications()
  }

  // --- NRCS 专用通知（对标 nrs.js 中的 $.growl 调用） ---

  /**
   * 资产收到通知（对标 nrs.js:1602-1614 you_received_assets）。
   *
   * @param name 资产名称
   * @param count 收到数量（格式化后）
   */
  function notifyAssetReceived(name: string, count: string): void {
    if (count === '1') {
      notifySuccess(`您收到 1 个 ${name} 资产`)
    } else {
      notifySuccess(`您收到 ${count} 个 ${name} 资产`)
    }
  }

  /**
   * 资产出售/转移通知（对标 nrs.js:1622-1634 you_sold_assets）。
   *
   * @param name 资产名称
   * @param count 出售数量（格式化后）
   */
  function notifyAssetSold(name: string, count: string): void {
    if (count === '1') {
      notifySuccess(`您出售、转移或删除 1 个 ${name} 资产`)
    } else {
      notifySuccess(`您出售、转移或删除 ${count} 个 ${name} 资产`)
    }
  }

  /**
   * 多资产变动通知（对标 nrs.js:1641 multiple_assets_differences）。
   */
  function notifyMultipleAssetDifferences(): void {
    notifySuccess('多个不同的资产已经被出售或购买。')
  }

  /**
   * 分叉警告 - 连续锻造 10 块（对标 nrs.js:1713 fork_warning）。
   */
  function notifyForkWarning(): void {
    notifyError('警告：您很有可能处于分叉（你已经锻造了最近的10个区块）。', { duration: 0 })
  }

  /**
   * 分叉警告 - baseTarget 异常（对标 nrs.js:1719 fork_warning_base_target）。
   */
  function notifyForkWarningBaseTarget(): void {
    notifyError('警告：您很有可能处于分叉（基础目标值非常高）。', { duration: 0 })
  }

  /**
   * 服务器连接错误（对标 nrs.js:493 error_server_connect）。
   *
   * @param url 请求 URL
   * @param description 错误描述
   */
  function notifyConnectionError(url: string, description?: string): void {
    const msg = `无法连接至 ${url}` + (description ? ` ${description}` : '')
    notifyError(msg, { duration: 0 })
  }

  /**
   * 剪贴板复制成功（对标 nrs.js:377 success_clipboard_copy）。
   */
  function notifyClipboardCopy(): void {
    notifyInfo('已成功复制到剪贴板。')
  }

  // --- 交易类型通知计数（对标 nrs.notifications.js） ---

  /**
   * 总未读通知数（对标 nrs.notifications.js:33 totalCount）。
   */
  const totalNotificationCount = computed(() => {
    let total = 0
    for (const typeState of Object.values(transactionNotifyStates.value)) {
      total += typeState.notificationCount
    }
    return total
  })

  /**
   * 有未读通知的子类型数（对标 nrs.notifications.js:23 subTypeCount）。
   */
  const subTypeNotificationCount = computed(() => {
    let count = 0
    for (const typeState of Object.values(transactionNotifyStates.value)) {
      for (const subTypeState of Object.values(typeState.subTypes)) {
        if (subTypeState.notificationCount > 0) count++
      }
    }
    return count
  })

  /**
   * 更新通知系统（对标 nrs.notifications.js:196-215 updateNotifications）。
   *
   * 从服务端获取当前时间，加载 IndexedDB 中的时间戳，重新初始化计数。
   *
   * @param account 当前登录账户 ID（数字形式，对标 NRS.account）
   */
  async function updateNotifications(account: string): Promise<void> {
    if (!account) {
      console.warn('[notifications] updateNotifications: account 为空，跳过')
      return
    }
    try {
      const timeResponse = await nrcsApi.getTime()
      if (timeResponse.time === undefined || timeResponse.time === null) return

      const result = await storageSelect<{ id: string; contents: string }>('data', [
        { id: 'notification_timestamps' },
      ])
      if (result && result.length > 0) {
        await loadNotificationsFromTimestamps(timeResponse.time, result[0].contents, account)
      } else {
        await loadNotificationsFromTimestamps(timeResponse.time, '', account)
      }
    } catch (e) {
      console.warn('[notifications] updateNotifications 失败:', e)
    }
  }

  /**
   * 重置通知状态（对标 nrs.notifications.js:124-145 resetNotificationState）。
   *
   * 将指定页面（或全部）的通知计数清零，更新 notificationTS。
   *
   * @param page 可选，指定要重置的 receiverPage；不传则重置全部
   */
  async function resetNotificationState(page?: string): Promise<void> {
    try {
      const timeResponse = await nrcsApi.getTime()
      if (timeResponse.time === undefined || timeResponse.time === null) return

      for (const typeIndex of Object.keys(transactionNotifyStates.value)) {
        const typeState = transactionNotifyStates.value[Number(typeIndex)]
        for (const subTypeIndex of Object.keys(typeState.subTypes)) {
          const subTypeState = typeState.subTypes[Number(subTypeIndex)]
          const subTypeDef = TRANSACTION_TYPES[Number(typeIndex)]?.subTypes[Number(subTypeIndex)]
          if (!page || subTypeDef?.receiverPage === page) {
            const countBefore = subTypeState.notificationCount
            // 对标 nrs.notifications.js:131-135 lastKnownTransaction 存在时用其 timestamp+1
            if (subTypeState.lastKnownTransaction) {
              subTypeState.notificationTS =
                (subTypeState.lastKnownTransaction.timestamp ?? 0) + 1
            } else {
              subTypeState.notificationTS = timeResponse.time
            }
            subTypeState.notificationCount = 0
            typeState.notificationCount -= countBefore
          }
        }
      }

      await saveNotificationTimestamps()
    } catch (e) {
      console.warn('[notifications] resetNotificationState 失败:', e)
    }
  }

  /**
   * 设置未确认交易通知数（对标 nrs.notifications.js:217-226 setUnconfirmedNotifications）。
   *
   * @param count 未确认交易数
   * @param itemsPerPage 每页条数（超过时显示 "N+"，对标 nrs.notifications.js:219-223）
   */
  function setUnconfirmedNotifications(count: number, itemsPerPage: number = 15): void {
    if (count > itemsPerPage) {
      unconfirmedCount.value = itemsPerPage // UI 层显示 "N+"
    } else {
      unconfirmedCount.value = count
    }
  }

  /**
   * 设置 Phasing 待审批交易通知数（对标 nrs.notifications.js:228-239 setPhasingNotifications）。
   *
   * @param account 当前账户 RS 或数字 ID
   */
  async function setPhasingNotifications(account: string): Promise<void> {
    try {
      // 对标 nrs.notifications.js:229 getAccountPhasedTransactionCount
      const response = await nrcsApi.getAccountPhasedTransactionCount(account)
      if (response?.numberOfPhasedTransactions !== undefined) {
        phasingCount.value = response.numberOfPhasedTransactions
      }
    } catch (e) {
      console.warn('[notifications] setPhasingNotifications 失败:', e)
    }
  }

  /**
   * 设置 Shuffling 通知数（对标 nrs.notifications.js:241-252 setShufflingNotifications）。
   */
  async function setShufflingNotifications(): Promise<void> {
    try {
      // 对标 nrs.notifications.js:242 getAllShufflings
      const response = await nrcsApi.getAllShufflings()
      if (response?.shufflings) {
        shufflingCount.value = response.shufflings.length
      } else {
        shufflingCount.value = 0
      }
    } catch (e) {
      console.warn('[notifications] setShufflingNotifications 失败:', e)
    }
  }

  /**
   * 获取有未读通知的子类型列表（供 UI 渲染通知菜单，对标 nrs.notifications.js:29-53）。
   */
  const notifySubTypes = computed(() => {
    const list: Array<{
      type: number
      subtype: number
      count: number
      title: string
      receiverPage?: string
    }> = []
    for (const typeIndex of Object.keys(transactionNotifyStates.value)) {
      const typeState = transactionNotifyStates.value[Number(typeIndex)]
      for (const subTypeIndex of Object.keys(typeState.subTypes)) {
        const subTypeState = typeState.subTypes[Number(subTypeIndex)]
        if (subTypeState.notificationCount > 0) {
          const subTypeDef = TRANSACTION_TYPES[Number(typeIndex)]?.subTypes[Number(subTypeIndex)]
          list.push({
            type: Number(typeIndex),
            subtype: Number(subTypeIndex),
            count: subTypeState.notificationCount,
            title: subTypeDef?.title ?? `Type ${typeIndex}.${subTypeIndex}`,
            receiverPage: subTypeDef?.receiverPage,
          })
        }
      }
    }
    return list
  })

  return {
    // --- Toast 通知 ---
    notify,
    notifySuccess,
    notifyError,
    notifyWarning,
    notifyInfo,
    dismissNotification,
    clearAllToasts,
    // --- NRCS 专用通知 ---
    notifyAssetReceived,
    notifyAssetSold,
    notifyMultipleAssetDifferences,
    notifyForkWarning,
    notifyForkWarningBaseTarget,
    notifyConnectionError,
    notifyClipboardCopy,
    // --- 交易类型计数（只读 state） ---
    transactionNotifyStates: readonly(transactionNotifyStates),
    totalNotificationCount,
    subTypeNotificationCount,
    notifySubTypes,
    unconfirmedCount: readonly(unconfirmedCount),
    phasingCount: readonly(phasingCount),
    shufflingCount: readonly(shufflingCount),
    // --- 交易类型计数（actions） ---
    updateNotifications,
    resetNotificationState,
    setUnconfirmedNotifications,
    setPhasingNotifications,
    setShufflingNotifications,
  }
}
