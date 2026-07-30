/**
 * NRCS 动态常量 Store。
 *
 * 端口自参考实现 `nrs.constants.js`：
 * - 启动时调用 `nrcsApi.getConstants()` 拉取服务端常量
 * - `processConstants` 将响应注入到各运行时容器（VOTING_MODELS / HASH_ALGORITHMS /
 *   REQUEST_TYPES / API_TAGS / SHUFFLING_STAGES / PEER_STATES / CURRENCY_TYPES 等）
 * - `loadTransactionTypeConstants` 将服务端交易类型常量合并进 `TRANSACTION_TYPES`
 * - 提供 `isApiEnabled` / `isRequestTypeEnabled` / `isRequireBlockchain` 等运行时判定辅助函数
 *
 * 与静态常量的关系：
 * - 编译期固定常量（DB_VERSION / MAX_*_JAVA / PL_* 等）见 `@/constants/server-constants.ts`
 * - 运行时容器（加载前为空）由本 Store 在应用启动时填充
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { convertNumericToRSAccountFormat } from '@/utils/converters'
import {
  TRANSACTION_TYPES,
  loadTransactionTypeConstants,
  type TransactionTypeDef
} from '@/constants/transaction-types'
import {
  LAST_KNOWN_BLOCK,
  LAST_KNOWN_TESTNET_BLOCK,
  isRuntimeMapEmpty,
  type ApiTagConfig,
  type RequestTypeConfig,
  type ServerConstantsResponse,
  type TransactionSubTypeServerInfo
} from '@/constants/server-constants'

/** 加载前 / 加载失败的回退默认值 */
const DEFAULT_MAX_TAGGED_DATA_DATA_LENGTH = 0

/** getECBlock 返回的回退区块结构 */
interface ECBlockFallback {
  id: string
  height: string
}

export const useConstantsStore = defineStore('constants', () => {
  // ==========================================================================
  // State（运行时容器，加载前为空 / 默认值）
  // ==========================================================================

  /** 完整 `getConstants` 响应（对标 NRS.constants.SERVER） */
  const server = ref<ServerConstantsResponse | null>(null)

  /** 投票模型：name → code */
  const votingModels = ref<Record<string, number>>({})
  /** 最低余额模型：name → code */
  const minBalanceModels = ref<Record<string, number>>({})
  /** 持有类型：name → code */
  const holdingTypes = ref<Record<string, number>>({})
  /** 哈希算法：name → code */
  const hashAlgorithms = ref<Record<string, number>>({})
  /** phasing 哈希算法：name → code */
  const phasingHashAlgorithms = ref<Record<string, number>>({})
  /** 铸币哈希算法：name → code */
  const mintingHashAlgorithms = ref<Record<string, number>>({})
  /** requestType → 配置 */
  const requestTypes = ref<Record<string, RequestTypeConfig>>({})
  /** API 标签：tag → { name, enabled } */
  const apiTags = ref<Record<string, ApiTagConfig>>({})
  /** 混币阶段：name → code */
  const shufflingStages = ref<Record<string, number>>({})
  /** 混币参与者状态：name → code */
  const shufflingParticipantStates = ref<Record<string, number>>({})
  /** 节点状态：name → code */
  const peerStates = ref<Record<string, number>>({})
  /** 货币类型：name → code */
  const currencyTypes = ref<Record<string, number>>({})
  /** 被禁用的 API 列表 */
  const disabledAPIs = ref<string[] | string[][]>([])
  /** 被禁用的 API 标签列表 */
  const disabledAPITags = ref<string[]>([])
  /** 代理节点不转发的请求列表 */
  const proxyNotForwardedRequests = ref<string[]>([])

  /** 合并服务端常量后的交易类型表（含 serverConstants） */
  const transactionTypes = ref<Record<number, TransactionTypeDef>>({ ...TRANSACTION_TYPES })
  /** 服务端子类型映射（按名称索引，对标 NRS.subtype） */
  const subtypeMap = ref<Record<string, TransactionSubTypeServerInfo>>({})

  /** 创世账户 ID（数字字符串） */
  const genesis = ref<string>('')
  /** 创世账户 RS 地址 */
  const genesisRS = ref<string>('')
  /** NRCS 纪元起点（毫秒） */
  const epochBeginning = ref<number>(0)
  /** 标记数据最大长度 */
  const maxTaggedDataDataLength = ref<number>(DEFAULT_MAX_TAGGED_DATA_DATA_LENGTH)
  /** 可修剪消息最大长度 */
  const maxPrunableMessageLength = ref<number>(0)
  /** 创世区块 ID */
  const genesisBlockId = ref<string>('')

  /** 是否已成功加载 */
  const loaded = ref<boolean>(false)
  /** 是否正在加载 */
  const loading = ref<boolean>(false)
  /** 最近一次加载错误 */
  const loadError = ref<string>('')

  // ==========================================================================
  // Getters
  // ==========================================================================

  /** 常量是否就绪 */
  const isLoaded = computed(() => loaded.value)

  /** REQUEST_TYPES 是否为空（未加载时隐式放行所有请求，对标参考） */
  const isRequestTypesEmpty = computed(() => isRuntimeMapEmpty(requestTypes.value))

  // ==========================================================================
  // Actions
  // ==========================================================================

  /**
   * 从节点拉取 `getConstants` 并注入运行时容器。
   *
   * 端口自 `nrs.constants.js` 的 `NRS.loadServerConstants` + `NRS.processConstants`。
   * Web 端始终走网络拉取（mobile app 离线模式由特性检测层处理，见阶段 0.8）。
   *
   * @param isTestnet - 是否测试网（影响 getECBlock 回退值，此处仅记录）
   */
  async function loadServerConstants(isTestnet = false): Promise<void> {
    if (loading.value) return
    loading.value = true
    loadError.value = ''
    try {
      const response = (await nrcsApi.getConstants()) as ServerConstantsResponse
      processConstants(response)
      console.log('[constants] done loading server constants')
    } catch (err) {
      loadError.value = err instanceof Error ? err.message : String(err)
      console.error('[constants] failed to load server constants:', err)
    } finally {
      loading.value = false
    }
  }

  /**
   * 将 `getConstants` 响应注入到各运行时容器。
   * 端口自 `nrs.constants.js:92` 的 `NRS.processConstants`。
   *
   * @param response - `getConstants` 响应
   */
  function processConstants(response: ServerConstantsResponse): void {
    if (!response?.genesisAccountId) {
      return
    }

    server.value = response
    votingModels.value = response.votingModels || {}
    minBalanceModels.value = response.minBalanceModels || {}
    holdingTypes.value = response.holdingTypes || {}
    hashAlgorithms.value = response.hashAlgorithms || {}
    // 兼容服务端字段名 phasingHashFunctions / phasingHashAlgorithms
    phasingHashAlgorithms.value =
      (response as unknown as { phasingHashFunctions?: Record<string, number> })
        .phasingHashFunctions || response.phasingHashAlgorithms || {}
    mintingHashAlgorithms.value = response.mintingHashAlgorithms || {}
    maxTaggedDataDataLength.value = response.maxTaggedDataDataLength || 0
    maxPrunableMessageLength.value = response.maxPrunableMessageLength || 0
    genesis.value = response.genesisAccountId
    genesisRS.value = convertNumericToRSAccountFormat(response.genesisAccountId)
    epochBeginning.value = response.epochBeginning
    genesisBlockId.value = response.genesisBlockId
    requestTypes.value = response.requestTypes || {}
    apiTags.value = response.apiTags || {}
    shufflingStages.value = response.shufflingStages || {}
    shufflingParticipantStates.value = response.shufflingParticipantStates || {}
    disabledAPIs.value = response.disabledAPIs || []
    disabledAPITags.value = normalizeDisabledList(response.disabledAPITags)
    proxyNotForwardedRequests.value = response.proxyNotForwardedRequests || []
    peerStates.value = response.peerStates || {}
    currencyTypes.value = response.currencyTypes || {}

    // 合并交易类型常量并更新子类型映射
    transactionTypes.value = loadTransactionTypeConstants(response, TRANSACTION_TYPES)
    subtypeMap.value = response.transactionSubTypes || {}

    loaded.value = true
  }

  /**
   * 重置所有运行时常量到未加载状态（登出 / 切换网络时调用）。
   */
  function reset(): void {
    server.value = null
    votingModels.value = {}
    minBalanceModels.value = {}
    holdingTypes.value = {}
    hashAlgorithms.value = {}
    phasingHashAlgorithms.value = {}
    mintingHashAlgorithms.value = {}
    requestTypes.value = {}
    apiTags.value = {}
    shufflingStages.value = {}
    shufflingParticipantStates.value = {}
    peerStates.value = {}
    currencyTypes.value = {}
    disabledAPIs.value = []
    disabledAPITags.value = []
    proxyNotForwardedRequests.value = []
    transactionTypes.value = { ...TRANSACTION_TYPES }
    subtypeMap.value = {}
    genesis.value = ''
    genesisRS.value = ''
    epochBeginning.value = 0
    maxTaggedDataDataLength.value = 0
    maxPrunableMessageLength.value = 0
    genesisBlockId.value = ''
    loaded.value = false
    loadError.value = ''
  }

  // ==========================================================================
  // 辅助函数（端口自 nrs.constants.js 的 NRS.* 判定函数）
  // ==========================================================================

  /**
   * 在映射表中按值反查键名。
   * 端口自 `nrs.constants.js:141` 的 `getKeyByValue`。
   *
   * @param map - name → code 映射
   * @param value - code 值
   * @returns 匹配的 name，未找到返回 null
   */
  function getKeyByValue<T extends string | number>(
    map: Record<string, T>,
    value: T
  ): string | null {
    for (const key of Object.keys(map)) {
      if (map[key] === value) {
        return key
      }
    }
    return null
  }

  /** 获取投票模型名称（code → name） */
  function getVotingModelName(code: number): string | null {
    return getKeyByValue(votingModels.value, code)
  }

  /** 获取投票模型代码（name → code） */
  function getVotingModelCode(name: string): number | undefined {
    return votingModels.value[name]
  }

  /** 获取最低余额模型名称（code → name） */
  function getMinBalanceModelName(code: number): string | null {
    return getKeyByValue(minBalanceModels.value, code)
  }

  /** 获取最低余额模型代码（name → code） */
  function getMinBalanceModelCode(name: string): number | undefined {
    return minBalanceModels.value[name]
  }

  /** 获取哈希算法名称（code → name） */
  function getHashAlgorithm(code: number): string | null {
    return getKeyByValue(hashAlgorithms.value, code)
  }

  /** 获取混币阶段名称（code → name） */
  function getShufflingStage(code: number): string | null {
    return getKeyByValue(shufflingStages.value, code)
  }

  /** 获取混币参与者状态名称（code → name） */
  function getShufflingParticipantState(code: number): string | null {
    return getKeyByValue(shufflingParticipantStates.value, code)
  }

  /** 获取节点状态名称（code → name） */
  function getPeerState(code: number): string | null {
    return getKeyByValue(peerStates.value, code)
  }

  /** 获取货币类型名称（code → name） */
  function getCurrencyType(code: number): string | null {
    return getKeyByValue(currencyTypes.value, code)
  }

  /**
   * 获取 EC 回退区块（节点不可达时使用）。
   * 端口自 `nrs.constants.js:184` 的 `NRS.getECBlock`。
   *
   * @param isTestnet - 是否测试网
   * @returns 主网或测试网的最后已知区块
   */
  function getECBlock(isTestnet = false): ECBlockFallback {
    return isTestnet ? LAST_KNOWN_TESTNET_BLOCK : LAST_KNOWN_BLOCK
  }

  /**
   * 判断 requestType 是否需要区块链已就绪。
   * 端口自 `nrs.constants.js:188` 的 `NRS.isRequireBlockchain`。
   * 未加载时隐式返回 false（对标参考：加载前不阻断）。
   *
   * @param requestType - 请求类型
   */
  function isRequireBlockchain(requestType: string): boolean {
    const cfg = requestTypes.value[requestType]
    if (!cfg) return false
    return cfg.requireBlockchain === true
  }

  /**
   * 判断 requestType 是否需要全节点。
   * 端口自 `nrs.constants.js:197` 的 `NRS.isRequireFullClient`。
   */
  function isRequireFullClient(requestType: string): boolean {
    const cfg = requestTypes.value[requestType]
    if (!cfg) return false
    return cfg.requireFullClient === true
  }

  /**
   * 判断 requestType 是否可被代理节点转发。
   * 端口自 `nrs.constants.js:206` 的 `NRS.isRequestForwardable`。
   */
  function isRequestForwardable(requestType: string): boolean {
    return (
      isRequireBlockchain(requestType) &&
      !isRequireFullClient(requestType) &&
      !proxyNotForwardedRequests.value.includes(requestType)
    )
  }

  /**
   * 判断 requestType 是否必须 POST。
   * 端口自 `nrs.constants.js:213` 的 `NRS.isRequirePost`。
   * 注意：`@/api/nrcs-client.ts` 中的 `isRequirePost` 基于静态规则，本函数基于服务端 REQUEST_TYPES，
   * 二者可并存；服务端常量加载后应优先使用本函数。
   */
  function isRequirePost(requestType: string): boolean {
    const cfg = requestTypes.value[requestType]
    if (!cfg) return false
    return cfg.requirePost === true
  }

  /**
   * 判断 requestType 是否启用。
   * 端口自 `nrs.constants.js:222` 的 `NRS.isRequestTypeEnabled`。
   * REQUEST_TYPES 为空时隐式放行；支持 "+schedule" 前缀剥离。
   */
  function isRequestTypeEnabled(requestType: string): boolean {
    if (isRuntimeMapEmpty(requestTypes.value)) {
      return true
    }
    const stripped = requestType.indexOf('+') > 0
      ? requestType.substring(0, requestType.indexOf('+'))
      : requestType
    return !!requestTypes.value[stripped]
  }

  /**
   * 判断 requestType 是否需要提交 secretPhrase。
   * 端口自 `nrs.constants.js:232` 的 `NRS.isSubmitPassphrase`。
   */
  function isSubmitPassphrase(requestType: string): boolean {
    return (
      requestType === 'startForging' ||
      requestType === 'stopForging' ||
      requestType === 'startShuffler' ||
      requestType === 'getForging' ||
      requestType === 'markHost' ||
      requestType === 'startFundingMonitor'
    )
  }

  /**
   * 判断 requestType 是否为定时交易请求（schedule 前缀）。
   * 端口自 `nrs.constants.js:241` 的 `NRS.isScheduleRequest`。
   */
  function isScheduleRequest(requestType: string): boolean {
    const keyword = 'schedule'
    return (
      !!requestType &&
      requestType.length >= keyword.length &&
      requestType.substring(0, keyword.length) === keyword
    )
  }

  /**
   * 获取文件上传类 requestType 的上传配置。
   * 端口自 `nrs.constants.js:246` 的 `NRS.getFileUploadConfig`。
   * 选择器（selector）在前端实现中由调用方视图自行绑定，此处仅返回参数名与上限。
   *
   * @param requestType - 请求类型
   * @param data - 表单数据（用于判断 sendMessage 是否加密）
   */
  function getFileUploadConfig(
    requestType: string,
    data?: Record<string, unknown>
  ): { requestParam: string; maxSize: number; errorDescription: string } | null {
    if (requestType === 'uploadTaggedData') {
      return {
        requestParam: 'file',
        maxSize: maxTaggedDataDataLength.value,
        errorDescription: 'error_file_too_big'
      }
    }
    if (requestType === 'dgsListing') {
      return {
        requestParam: 'messageFile',
        maxSize: maxPrunableMessageLength.value,
        errorDescription: 'error_image_too_big'
      }
    }
    if (requestType === 'sendMessage') {
      const encrypted = !!data?.encrypt_message
      return {
        requestParam: encrypted ? 'encryptedMessageFile' : 'messageFile',
        maxSize: maxPrunableMessageLength.value,
        errorDescription: 'error_message_too_big'
      }
    }
    return null
  }

  /**
   * 判断依赖（tags/apis）是否全部启用。
   * 端口自 `nrs.constants.js:274` 的 `NRS.isApiEnabled`。
   *
   * @param depends - 依赖描述 { tags?, apis? }，每项含 enabled 字段
   */
  function isApiEnabled(
    depends?: {
      tags?: (ApiTagConfig | null | undefined)[]
      apis?: (RequestTypeConfig | null | undefined)[]
    }
  ): boolean {
    if (!depends) return true
    const tags = depends.tags
    if (tags) {
      for (const tag of tags) {
        if (tag && !tag.enabled) return false
      }
    }
    const apis = depends.apis
    if (apis) {
      for (const api of apis) {
        if (api && !api.enabled) return false
      }
    }
    return true
  }

  /**
   * 判断单个 requestType 是否启用（isRequestTypeEnabled 的语义别名）。
   * 对标参考中 `NRS.isRequestEnabled` 的用途。
   */
  function isRequestEnabled(requestType: string): boolean {
    return isRequestTypeEnabled(requestType)
  }

  /**
   * 向 select 元素填充哈希算法选项。
   * 端口自 `nrs.constants.js:78` 的 `NRS.loadAlgorithmList`。
   * Vue3 中由调用方将返回的选项列表绑定到 <el-select>，而非直接操作 DOM。
   *
   * @param isPhasingHash - 是否取 phasing 哈希算法表
   * @returns 选项数组 { label, value }
   */
  function getAlgorithmOptions(
    isPhasingHash = false
  ): Array<{ label: string; value: number }> {
    const map = isPhasingHash ? phasingHashAlgorithms.value : hashAlgorithms.value
    return Object.keys(map).map((key) => ({ label: key, value: map[key] }))
  }

  return {
    // state
    server,
    votingModels,
    minBalanceModels,
    holdingTypes,
    hashAlgorithms,
    phasingHashAlgorithms,
    mintingHashAlgorithms,
    requestTypes,
    apiTags,
    shufflingStages,
    shufflingParticipantStates,
    peerStates,
    currencyTypes,
    disabledAPIs,
    disabledAPITags,
    proxyNotForwardedRequests,
    transactionTypes,
    subtypeMap,
    genesis,
    genesisRS,
    epochBeginning,
    maxTaggedDataDataLength,
    maxPrunableMessageLength,
    genesisBlockId,
    loaded,
    loading,
    loadError,
    // getters
    isLoaded,
    isRequestTypesEmpty,
    // actions
    loadServerConstants,
    processConstants,
    reset,
    // helpers
    getKeyByValue,
    getVotingModelName,
    getVotingModelCode,
    getMinBalanceModelName,
    getMinBalanceModelCode,
    getHashAlgorithm,
    getShufflingStage,
    getShufflingParticipantState,
    getPeerState,
    getCurrencyType,
    getECBlock,
    isRequireBlockchain,
    isRequireFullClient,
    isRequestForwardable,
    isRequirePost,
    isRequestTypeEnabled,
    isSubmitPassphrase,
    isScheduleRequest,
    getFileUploadConfig,
    isApiEnabled,
    isRequestEnabled,
    getAlgorithmOptions
  }
})

// ============================================================================
// 内部工具
// ============================================================================

/**
 * 规范化被禁用列表：服务端 `disabledAPITags` / `disabledAPIs` 可能是 `string[]` 或 `string[][]`，
 * 统一拍平为 `string[]`。
 */
function normalizeDisabledList(list: string[] | string[][] | undefined): string[] {
  if (!list || !Array.isArray(list)) return []
  const flat: string[] = []
  for (const item of list) {
    if (Array.isArray(item)) {
      flat.push(...item)
    } else {
      flat.push(item)
    }
  }
  return flat
}
