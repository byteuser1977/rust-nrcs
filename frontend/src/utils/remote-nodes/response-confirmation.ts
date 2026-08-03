/**
 * 响应交叉验证模块。
 *
 * 端口自参考实现 `nrs.remote.nodes.js:54-335`。
 *
 * 在 API 代理（apiProxy）或移动端场景下，本地节点不可信，需对可转发的 GET 请求
 * 随机选取若干远程节点重复请求，比对响应一致性，避免被恶意节点欺骗。
 *
 * 核心流程（对标 `nrs.remote.nodes.js:175` 的 `confirmResponse`）：
 *   1. 主请求成功后，将响应规范化为可比较字符串（`getComparableResponse`）
 *   2. 随机选取 `validators_count` 个远程节点（排除主请求节点）重发请求
 *   3. 比对每个远程节点响应与主响应，记录到 `confirmationReport`
 *   4. 更新确认率指示器（绿/黄/红渐变）与确认历史表
 *
 * 适配说明（Vue3 与参考的差异）：
 * - 参考通过 jQuery 操作 `#confirmation_rate_indicator` 与 `#request_confirmations_info_table`，
 *   本实现改为维护响应式 `confirmations` 与 `confirmationRate`，由 Vue 组件订阅渲染。
 * - 参考依赖全局 `NRS.isRequirePost` / `NRS.isRequestForwardable` / `NRS.mobileSettings`，
 *   本实现改为 `ConfirmationDeps` 依赖注入，避免循环依赖。
 * - 参考的 `getRandomNodes` 调用存在 `processedAddresses.concat` 未赋值的 bug，已在 manager 中修正。
 */
import { ref, type Ref } from 'vue'
import type { RemoteNodeHandle, RemoteNodeRequestFn, RemoteRequestOptions } from './types'
import { getRemoteNodesManager } from './remote-nodes-manager'

/** 可 prunable 的附件类型（对标 `nrs.remote.nodes.js:65`） */
const PRUNABLE_ATTACHMENTS = [
  'PrunablePlainMessage',
  'PrunableEncryptedMessage',
  'UnencryptedPrunableEncryptedMessage',
  'ShufflingProcessing',
  'TaggedDataUpload',
]

/** 确认报告（对标 `nrs.remote.nodes.js:194` 的 `confirmationReport`） */
export interface ConfirmationReport {
  /** 待确认的节点地址列表 */
  processing: string[]
  /** 已确认（响应一致）的节点列表 */
  confirmingNodes: RemoteNodeHandle[]
  /** 已拒绝（响应不一致）的节点列表 */
  rejectingNodes: RemoteNodeHandle[]
  /** 请求类型 */
  requestType: string
  /** 请求时间 */
  requestTime: Date
}

/** 确认依赖（由 index.ts init 时注入，避免循环依赖 constants store） */
export interface ConfirmationDeps {
  /** 判断 requestType 是否必须 POST（对标 NRS.isRequirePost） */
  isRequirePost: (requestType: string) => boolean
  /** 判断 requestType 是否可转发（对标 NRS.isRequestForwardable） */
  isRequestForwardable: (requestType: string) => boolean
  /** 获取验证者数量（对标 NRS.mobileSettings.validators_count） */
  getValidatorsCount: () => number
}

/** 确认历史（对标 `nrs.remote.nodes.js:21` 的 `requestConfirmations` 数组） */
export const confirmations: Ref<ConfirmationReport[]> = ref([])

/** 确认率指示器状态（对标 `updateConfirmationsIndicator` 的颜色与 hasRejections） */
export interface ConfirmationRateState {
  /** 是否存在拒绝 */
  hasRejections: boolean
  /** 确认总数（主节点 + 各远程确认） */
  confirmations: number
  /** 拒绝总数 */
  rejections: number
  /** 指示器颜色（hex，如 `#3ea940`） */
  color: string
}

/** 确认率指示器响应式状态 */
export const confirmationRate: Ref<ConfirmationRateState> = ref({
  hasRejections: false,
  confirmations: 0,
  rejections: 0,
  color: '#3ea940',
})

/** 注入的依赖（由 setConfirmationDeps 设置） */
let deps: ConfirmationDeps | null = null

/** 注入的远程请求函数（由 setConfirmationTransport 设置） */
let remoteTransport: RemoteNodeRequestFn | null = null

/**
 * 注入确认依赖。
 *
 * 由 `index.ts` 的 `initRemoteNodesMgr` 在 constants store 加载后调用。
 *
 * @param newDeps - 确认依赖
 */
export function setConfirmationDeps(newDeps: ConfirmationDeps): void {
  deps = newDeps
}

/**
 * 注入远程请求函数。
 *
 * @param fn - 远程请求函数（与 RemoteNodesManager 共用）
 */
export function setConfirmationTransport(fn: RemoteNodeRequestFn): void {
  remoteTransport = fn
}

/**
 * 判断请求是否需要远程节点确认。
 *
 * 对标 `nrs.remote.nodes.js:54` 的 `NRS.requestNeedsConfirmation`。
 *
 * 条件：
 *   1. 远程节点管理器已初始化
 *   2. 请求非 POST（可转发）
 *   3. 请求可转发（requireBlockchain 且非 requireFullClient 且不在 proxyNotForwarded 列表）
 *
   * @param requestType - 请求类型（支持 `xxx+schedule` 前缀剥离）
 * @returns 需要确认返回 true
 */
export function requestNeedsConfirmation(requestType: string): boolean {
  if (!getRemoteNodesManager()) return false
  if (!deps) return false
  const plusIndex = requestType.indexOf('+')
  const stripped = plusIndex > 0 ? requestType.substring(0, plusIndex) : requestType
  return !deps.isRequirePost(stripped) && deps.isRequestForwardable(stripped)
}

/**
 * 规范化 prunable 附件：仅保留 hash 字段，删除可变内容。
 *
 * 对标 `nrs.remote.nodes.js:69` 的 `normalizePrunableAttachment`。
 * prunable 附件的 hash 不可变，但内容可能被裁剪，故比较时只保留 hash。
 *
 * @param transaction - 交易对象（可能含 attachment）
 */
function normalizePrunableAttachment(transaction: any): void {
  const attachment = transaction?.attachment
  if (!attachment) return
  // 检测是否为 prunable 附件
  let isPrunableAttachment = false
  for (const key in attachment) {
    if (!Object.prototype.hasOwnProperty.call(attachment, key) || !key.startsWith('version.')) continue
    const strippedKey = key.substring('version.'.length)
    if (PRUNABLE_ATTACHMENTS.indexOf(strippedKey) >= 0) {
      isPrunableAttachment = true
    }
  }
  if (!isPrunableAttachment) return
  // 仅保留以 "hash" 结尾的字段
  for (const key in attachment) {
    if (!Object.prototype.hasOwnProperty.call(attachment, key)) continue
    if (key.length < 4 || key.substring(key.length - 4).toLowerCase() !== 'hash') {
      delete attachment[key]
    }
  }
}

/**
 * 统计两个数组的公共元素数。
 *
 * 对标 `nrs.remote.nodes.js:133` 的 `NRS.countCommonElements`。
 */
function countCommonElements(a1: any[], a2: any[]): number {
  let count = 0
  for (const el of a1) {
    if (a2.indexOf(el) >= 0) count++
  }
  return count
}

/**
 * 比较两个 peer 列表是否相似（公共元素占比 > 70%）。
 *
 * 对标 `nrs.remote.nodes.js:99` 的 `NRS.isPeerListSimilar`。
 *
 * @param peers1 - 第一个响应（含 peers 数组）
 * @param peers2 - 第二个响应（含 peers 数组）
 */
export function isPeerListSimilar(peers1: any, peers2: any): boolean {
  if (!peers1.peers && !peers2.peers) return true
  if (!peers1.peers || !peers2.peers) return false
  const shared = countCommonElements(peers1.peers, peers2.peers)
  return (100 * shared) / Math.min(peers1.peers.length, peers2.peers.length) > 70
}

/**
 * 比较两个 ledger 条目列表是否一致。
 *
 * 对标 `nrs.remote.nodes.js:113` 的 `NRS.compareLedgerEntries`。
 */
export function compareLedgerEntries(obj1: any, obj2: any): boolean {
  if (!obj1.entries && !obj2.entries) return true
  if (!obj1.entries || !obj2.entries) return false
  if (Array.isArray(obj1.entries) && Array.isArray(obj2.entries)) {
    for (let i = 0; i < obj1.entries.length && i < obj2.entries.length; i++) {
      if (JSON.stringify(obj1.entries[i]) !== JSON.stringify(obj2.entries[i])) return false
    }
    return true
  }
  return false
}

/**
 * 提取可比较的响应字符串。
 *
 * 对标 `nrs.remote.nodes.js:143` 的 `NRS.getComparableResponse`。
 * 删除可变字段（requestProcessingTime/confirmations/nextBlock/ledgerId），
 * 规范化 prunable 附件，返回 JSON 字符串用于一致性比对。
 *
 * @param origResponse - 原始响应对象（会被修改）
 * @param requestType - 请求类型
 * @returns 可比较的 JSON 字符串
 */
export function getComparableResponse(origResponse: any, requestType: string): string {
  if (requestType === 'getBlockchainStatus') {
    return JSON.stringify({
      application: origResponse.application,
      isTestnet: origResponse.isTestnet,
    })
  }
  if (requestType === 'getState') {
    // getState 无需比较
    return requestType
  }

  delete origResponse.requestProcessingTime
  delete origResponse.confirmations
  if (requestType === 'getBlock') {
    delete origResponse.nextBlock
  } else if (origResponse.transactions) {
    for (const transaction of origResponse.transactions) {
      delete transaction.confirmations
      normalizePrunableAttachment(transaction)
    }
  } else if (requestType === 'getAccountLedger' && origResponse.entries) {
    for (const entry of origResponse.entries) {
      delete entry.ledgerId
    }
  }
  return JSON.stringify(origResponse)
}

/**
 * 对主请求的响应发起远程节点交叉验证。
 *
 * 对标 `nrs.remote.nodes.js:175` 的 `NRS.confirmResponse`。
 *
 * 流程：
 *   1. 规范化主响应为可比较字符串
 *   2. 随机选取 `validators_count` 个远程节点（排除主请求节点）
 *   3. 向每个节点重发请求，比对响应
 *   4. 维护 confirmationReport 与确认率指示器
 *
 * @param requestType - 请求类型
 * @param data - 原始请求数据
 * @param expectedResponse - 主请求的响应（作为期望值）
 * @param requestRemoteNode - 主请求使用的远程节点（若有，排除其避免重复验证）
 */
export async function confirmResponse(
  requestType: string,
  data: Record<string, any>,
  expectedResponse: any,
  requestRemoteNode?: RemoteNodeHandle | null,
): Promise<void> {
  if (!requestNeedsConfirmation(requestType)) return
  if (!deps || !remoteTransport) return
  const mgr = getRemoteNodesManager()
  if (!mgr) return

  // 克隆并规范化主响应
  let expectedResponseStr: string
  try {
    const cloned = JSON.parse(JSON.stringify(expectedResponse))
    expectedResponseStr = getComparableResponse(cloned, requestType)
  } catch (e) {
    console.log(`[remote-nodes] Cannot parse JSON response for request ${requestType}`)
    return
  }

  const ignoredAddresses: string[] = []
  if (requestRemoteNode?.address) {
    ignoredAddresses.push(requestRemoteNode.address)
  }
  const validatorsCount = deps.getValidatorsCount()
  const nodes = mgr.getRandomNodes(validatorsCount, ignoredAddresses)
  const now = new Date()
  const confirmationReport: ConfirmationReport = {
    processing: [],
    confirmingNodes: [],
    rejectingNodes: [],
    requestType,
    requestTime: now,
  }

  confirmations.value.unshift(confirmationReport)

  // 清理过期历史（保留最近 1 分 15 秒，对标 nrs.remote.nodes.js:201-214）
  const minRequestTime = new Date(now)
  minRequestTime.setMinutes(minRequestTime.getMinutes() - 1)
  minRequestTime.setSeconds(minRequestTime.getSeconds() - 15)
  for (let idx = confirmations.value.length - 1; idx > 0; idx--) {
    if (minRequestTime > confirmations.value[idx].requestTime) {
      confirmations.value.splice(idx, 1)
    } else {
      break
    }
  }
  // 上限 50 条（对标 nrs.remote.nodes.js:216）
  if (confirmations.value.length > 50) {
    confirmations.value.splice(confirmations.value.length - 1, 1)
  }

  for (const node of nodes) {
    if (node.isBlacklisted()) continue
    confirmationReport.processing.push(node.announcedAddress || node.address)
    ignoredAddresses.push(node.address)

    remoteTransport(requestType, { ...data }, {
      noProxy: true,
      remoteNode: node,
    } as RemoteRequestOptions)
      .then((response: any) => {
        const idx = confirmationReport.processing.indexOf(node.announcedAddress || node.address)
        if (idx >= 0) confirmationReport.processing.splice(idx, 1)

        if (response.errorCode) {
          console.log(`[remote-nodes] Confirm request error ${response.errorDescription}`)
          return
        }

        const responseStr = getComparableResponse(response, requestType)
        const isSimilar =
          responseStr === expectedResponseStr ||
          (requestType === 'getPeers' && isPeerListSimilar(response, expectedResponse)) ||
          (requestType === 'getAccountLedger' && compareLedgerEntries(response, expectedResponse))

        if (isSimilar) {
          confirmationReport.confirmingNodes.push(node)
        } else {
          console.log(
            `[remote-nodes] ${node.announcedAddress || node.address} response defers from ${requestRemoteNode?.announcedAddress || 'primary'} response for ${requestType}`,
          )
          confirmationReport.rejectingNodes.push(node)
          updateConfirmationRate()
        }

        if (confirmationReport.processing.length === 0) {
          console.log(
            `[remote-nodes] onConfirmation:Request ${requestType} confirmations ${confirmationReport.confirmingNodes.length} rejections ${confirmationReport.rejectingNodes.length}`,
          )
          updateConfirmationRate()
        }
      })
      .catch((err: any) => {
        const idx = confirmationReport.processing.indexOf(node.announcedAddress || node.address)
        if (idx >= 0) confirmationReport.processing.splice(idx, 1)
        console.log(`[remote-nodes] Confirm request exception ${err?.message || err}`)
      })
  }
}

/**
 * 更新确认率指示器。
 *
 * 对标 `nrs.remote.nodes.js:275` 的 `NRS.updateConfirmationsIndicator`。
 *
 * 颜色梯度：
 *   - 无拒绝：绿色 `#3ea940`
 *   - 有拒绝：从黄 `#eccc31` 到红 `#a94442` 按 rejectionsRatio 线性插值
 *
 * Vue3 适配：参考通过 jQuery 操作 DOM，本实现改为更新响应式 `confirmationRate`，
 * 由 `ConfirmationIndicator.vue` 组件订阅渲染。
 */
export function updateConfirmationRate(): void {
  const green = 0x3ea940
  let rejections = 0
  let confirmationsCount = 0
  let hasRejections = false

  for (const c of confirmations.value) {
    confirmationsCount++ // 主远程节点计 1 票
    confirmationsCount += c.confirmingNodes.length
    rejections += c.rejectingNodes.length
  }

  let color = green
  if (confirmationsCount > 0) {
    let rejectionsRatio = (rejections * 2) / confirmationsCount // 最差 1:1
    if (rejectionsRatio > 1) rejectionsRatio = 1
    if (rejectionsRatio > 0) {
      const gradientStart = 0xeccc31
      const gradientEnd = 0xa94442
      const r = ((gradientStart >> 16) & 0xff) * (1 - rejectionsRatio) + ((gradientEnd >> 16) & 0xff) * rejectionsRatio
      const g = ((gradientStart >> 8) & 0xff) * (1 - rejectionsRatio) + ((gradientEnd >> 8) & 0xff) * rejectionsRatio
      const b = (gradientStart & 0xff) * (1 - rejectionsRatio) + (gradientEnd & 0xff) * rejectionsRatio
      const rHex = Math.round(r).toString(16).padStart(2, '0')
      const gHex = Math.round(g).toString(16).padStart(2, '0')
      const bHex = Math.round(b).toString(16).padStart(2, '0')
      color = parseInt(`${rHex}${gHex}${bHex}`, 16)
      hasRejections = true
    }
  }

  const hexStr = '#' + color.toString(16).padStart(6, '0')
  confirmationRate.value = {
    hasRejections,
    confirmations: confirmationsCount,
    rejections,
    color: hexStr,
  }
}

/**
 * 清空确认历史与指示器（登出 / 切换网络时调用）。
 */
export function resetConfirmations(): void {
  confirmations.value = []
  confirmationRate.value = {
    hasRejections: false,
    confirmations: 0,
    rejections: 0,
    color: '#3ea940',
  }
}
