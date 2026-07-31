/******************************************************************************
 * NRCS 交易详情 composable —— 对标 nrs.modals.transaction.js。
 *
 * 核心函数：
 *   - showTransactionModal(transaction | id, sharedKey?)  入口
 *   - getTransaction(transactionId)                       通过 API 拉取交易
 *   - processTransactionModalData(transaction, sharedKey?) 处理交易数据
 *   - getPhasingDetails(phasingParams)                    phasing 详情
 *
 * 处理流程（对标 nrs.modals.transaction.js:128-1395）：
 *   1. showTransactionModal 接受交易对象或 ID
 *   2. 若是 ID → 调用 getTransaction API 拉取（includePhasingResult=true）
 *   3. processTransactionModalData：
 *      a. 构造 transactionDetails（去掉 attachment/transaction，格式化字段）
 *      b. 调用 renderAttachment 渲染附件主体
 *      c. 调用 getPhasingDetails 渲染 phasing 详情
 *      d. 解析公开消息（parsePublicMessage）
 *      e. 识别加密消息字段（getDecryptionFields）
 *
 * 解耦设计：composable 不直接渲染 UI，仅返回结构化数据，由 TransactionDetailPanel.vue 渲染。
 ******************************************************************************/
import { ref, computed, type Ref } from 'vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import type { NrcsTransaction } from '@/api/modules/nrcs.api'
import { useAccountStore } from '@/stores/modules/account.store'
import { useNodeStore } from '@/stores/modules/node.store'
import { formatAmount, formatTimestamp } from '@/utils/format'
import { getBlockLink, getTransactionLink, getAccountLink } from '@/utils/transaction-links'
import { createInfoTable, type InfoRow } from '@/utils/transaction-info-table'
import {
  parsePublicMessage,
  getDecryptionFields,
  getMessageHash,
  type ParsedPublicMessage,
} from '@/utils/transaction-message'
import { renderAttachment, getVotingModelName, getMinBalanceModelName } from '@/utils/transaction-attachment-renderer'
import { useTransactionDecryption, type DecryptionResult } from './useTransactionDecryption'

// ============================================================================
// 常量
// ============================================================================

/** Java 有符号 int 最大值（用于 height == MAX_INT_JAVA → "unknown"） */
const MAX_INT_JAVA = 2147483647

// ============================================================================
// 类型定义
// ============================================================================

/** 交易详情 modal 的完整状态 */
export interface TransactionDetailState {
  /** 是否显示 modal */
  visible: boolean
  /** 是否正在加载（对标 NRS.fetchingModalData） */
  loading: boolean
  /** 当前交易对象 */
  transaction: NrcsTransaction | null
  /** 交易基本信息表行（对标 transactionDetails → createInfoTable） */
  transactionDetails: InfoRow[]
  /** 附件信息表行（对标 infoTable → createInfoTable） */
  attachmentRows: InfoRow[]
  /** phasing 详情表行（对标 phasingDetails → createInfoTable） */
  phasingDetails: InfoRow[]
  /** 公开消息（顶部，对标 transaction_info_output_top） */
  topMessage: ParsedPublicMessage | null
  /** 底部消息（对标 transaction_info_output_bottom） */
  bottomMessage: ParsedPublicMessage | null
  /** 消息哈希（对标 hashRow） */
  messageHash: string | undefined
  /** 加密消息解密结果 */
  decryptionResults: DecryptionResult[]
  /** 是否需要解密（有待解密的加密消息） */
  hasEncryptedMessage: boolean
  /** 是否需要 sharedKey 输入 */
  needsSharedKey: boolean
  /** 是否为未知交易类型（对标 incorrect=true） */
  incorrect: boolean
  /** 错误信息 */
  error: string | null
}

/** 初始状态 */
const INITIAL_STATE: TransactionDetailState = {
  visible: false,
  loading: false,
  transaction: null,
  transactionDetails: [],
  attachmentRows: [],
  phasingDetails: [],
  topMessage: null,
  bottomMessage: null,
  messageHash: undefined,
  decryptionResults: [],
  hasEncryptedMessage: false,
  needsSharedKey: false,
  incorrect: false,
  error: null,
}

// ============================================================================
// composable
// ============================================================================

/**
 * 交易详情 composable。
 *
 * 提供：
 *   - showTransactionModal：显示交易详情 modal
 *   - processTransactionModalData：处理交易数据并填充状态
 *   - getPhasingDetails：获取 phasing 详情
 *   - decryptEncryptedMessages：解密加密消息
 *   - state：响应式状态
 *
 * @returns composable API
 */
export function useTransactionDetail() {
  /** 响应式状态 */
  const state = ref<TransactionDetailState>({ ...INITIAL_STATE })

  /** 共享密钥（用户输入，用于解密失败时重试） */
  const sharedKeyInput = ref<string>('')

  const accountStore = useAccountStore()
  const nodeStore = useNodeStore()

  /** 解密 composable */
  const decryption = useTransactionDecryption()

  // ----------------------------------------------------------------
  // showTransactionModal —— 入口（对标 nrs.modals.transaction.js:39-68）
  // ----------------------------------------------------------------

  /**
   * 显示交易详情 modal。
   *
   * 接受交易对象或交易 ID：
   *   - 对象 → 直接调 processTransactionModalData
   *   - ID（字符串）→ 调 getTransaction API 拉取后再处理
   *
   * @param transaction 交易对象或 ID
   * @param sharedKey 可选共享密钥（用于解密）
   */
  async function showTransactionModal(
    transaction: NrcsTransaction | string,
    sharedKey?: string,
  ): Promise<void> {
    if (state.value.loading) return

    state.value = { ...INITIAL_STATE, visible: true, loading: true }
    sharedKeyInput.value = sharedKey ?? ''

    try {
      if (typeof transaction === 'string') {
        // 通过 ID 拉取（对标 :54-60 includePhasingResult: true）
        const tx = await getTransaction(transaction)
        await processTransactionModalData(tx, sharedKey)
      } else {
        await processTransactionModalData(transaction, sharedKey)
      }
    } catch (e) {
      const error = e instanceof Error ? e.message : String(e)
      state.value.loading = false
      state.value.error = error
      console.error('[useTransactionDetail] showTransactionModal failed:', e)
    }
  }

  // ----------------------------------------------------------------
  // getTransaction —— 通过 API 拉取交易（对标 :54-60）
  // ----------------------------------------------------------------

  /**
   * 通过交易 ID 拉取交易详情（对标 nrs.modals.transaction.js:54-60）。
   *
   * @param transactionId 交易 ID
   * @returns 交易对象（含 attachment）
   */
  async function getTransaction(transactionId: string): Promise<NrcsTransaction> {
    const tx = await nrcsApi.getTransaction(transactionId)
    if (!tx || (tx as any).errorCode) {
      throw new Error((tx as any)?.errorDescription || `交易 ${transactionId} 不存在`)
    }
    // 注入 transaction 字段（对标 :58 response.transaction = input.transaction）
    tx.transaction = transactionId
    return tx
  }

  // ----------------------------------------------------------------
  // processTransactionModalData —— 处理交易数据（对标 :128-1395）
  // ----------------------------------------------------------------

  /**
   * 处理交易数据并填充 modal 状态（对标 nrs.modals.transaction.js:128-1395）。
   *
   * 步骤：
   *   1. 构造 transactionDetails（格式化 confirmations/block/timestamp/height）
   *   2. 调用 renderAttachment 渲染附件主体
   *   3. 调用 getPhasingDetails 渲染 phasing 详情（若有 phasingFinishHeight）
   *   4. 解析公开消息（顶部 + 底部）
   *   5. 识别加密消息字段
   *
   * @param transaction 交易对象
   * @param sharedKey 可选共享密钥
   */
  async function processTransactionModalData(
    transaction: NrcsTransaction,
    sharedKey?: string,
  ): Promise<void> {
    const currentAccount = accountStore.accountId || ''
    const lastBlockHeight = nodeStore.lastBlockHeight

    // === 1. 构造 transactionDetails（对标 :138-167） ===
    const transactionDetails = buildTransactionDetails(transaction, lastBlockHeight)
    const detailsRows = createInfoTable(transactionDetails, false)

    // === 2. 渲染附件主体 ===
    const renderResult = await renderAttachment(transaction, {
      currentAccount,
      lastBlockHeight,
    })

    // === 3. Phasing 详情（对标 :217-227） ===
    let phasingRows: InfoRow[] = []
    const attachment = transaction.attachment
    if (attachment && attachment.phasingFinishHeight !== undefined) {
      const phasingDetails = await getPhasingDetails(attachment, lastBlockHeight)
      phasingRows = createInfoTable(phasingDetails, false)
    }

    // === 4. 公开消息（对标 :256-321 顶部 / :1325-1374 底部） ===
    const isMessagingType = transaction.type === 1 && transaction.subtype === 0
    let topMessage: ParsedPublicMessage | null = null
    let bottomMessage: ParsedPublicMessage | null = null

    if (isMessagingType) {
      // type 1 subtype 0：消息显示在顶部
      topMessage = parsePublicMessage(attachment)
    } else if (attachment) {
      // 其他类型：消息显示在底部
      bottomMessage = parsePublicMessage(attachment)
    }

    // 消息哈希（对标 :309）
    const messageHash = getMessageHash(attachment)

    // === 5. 加密消息字段识别（对标 :279-289, 1351-1365） ===
    const decryptFields = getDecryptionFields(transaction, currentAccount)
    const hasEncryptedMessage = decryptFields.length > 0

    // === 6. 更新状态 ===
    state.value = {
      visible: true,
      loading: false,
      transaction,
      transactionDetails: detailsRows,
      attachmentRows: renderResult.rows,
      phasingDetails: phasingRows,
      topMessage,
      bottomMessage,
      messageHash,
      decryptionResults: [],
      hasEncryptedMessage,
      needsSharedKey: false,
      incorrect: renderResult.incorrect,
      error: renderResult.incorrect ? 'error_unknown_transaction_type' : null,
    }

    // === 7. 自动尝试解密（若有 secretPhrase） ===
    if (hasEncryptedMessage && accountStore.secretPhrase) {
      await decryptEncryptedMessages(sharedKey)
    }
  }

  // ----------------------------------------------------------------
  // buildTransactionDetails —— 构造交易基本信息（对标 :138-167）
  // ----------------------------------------------------------------

  /**
   * 构造交易基本信息对象（对标 nrs.modals.transaction.js:138-167）。
   *
   * 去掉 attachment / transaction 字段，格式化：
   *   - confirmations: 空 → "/"
   *   - block: 空 → "unconfirmed"
   *   - timestamp → formatTimestamp
   *   - blockTimestamp → formatTimestamp
   *   - height == MAX_INT_JAVA → "unknown"，否则 getBlockLink
   *   - executionHeight → getBlockLink
   *
   * @param transaction 交易对象
   * @param lastBlockHeight 当前区块高度
   * @returns 交易基本信息对象
   */
  function buildTransactionDetails(
    transaction: NrcsTransaction,
    _lastBlockHeight: number,
  ): Record<string, any> {
    // 浅拷贝并删除 attachment / transaction
    const details: Record<string, any> = { ...transaction, attachment: undefined }
    delete details.attachment
    delete details.transaction

    // referencedTransaction == "0" 时删除
    if (details.referencedTransaction === '0') {
      delete details.referencedTransaction
    }

    // confirmations
    if (!details.confirmations) {
      details.confirmations = '/'
    }

    // block
    if (!details.block) {
      details.block = 'unconfirmed'
    }

    // timestamp → transactionTime（对标 :151-153）
    if (details.timestamp !== undefined) {
      details.transactionTime = formatTimestamp(details.timestamp)
    }

    // blockTimestamp → blockGenerationTime（对标 :154-156）
    if (details.blockTimestamp !== undefined) {
      details.blockGenerationTime = formatTimestamp(details.blockTimestamp)
    }

    // height（对标 :157-162）
    if (details.height === MAX_INT_JAVA) {
      details.height = 'unknown'
    } else if (details.height !== undefined) {
      details.height_formatted_html = getBlockLink(details.height)
      delete details.height
    }

    // executionHeight（对标 :163-166）
    if (details.executionHeight !== undefined) {
      details.execution_height_formatted_html = getBlockLink(details.executionHeight)
      delete details.executionHeight
    }

    return details
  }

  // ----------------------------------------------------------------
  // getPhasingDetails —— 对标 nrs.modals.transaction.js:70-126
  // ----------------------------------------------------------------

  /**
   * 获取 phasing 详情（对标 nrs.modals.transaction.js:70-126 NRS.getPhasingDetails）。
   *
   * 字段：
   *   - finishHeight / finishIn（距完成还有多少块）
   *   - votingModel（投票模型名称）
   *   - quorum / minBalance（根据 holding 类型转换精度）
   *   - minBalanceModel
   *   - whitelist（白名单账户列表）
   *   - linkedFullHashes（关联交易全哈希列表）
   *   - hashedSecret / hashAlgorithm（哈希密钥）
   *
   * @param phasingParams phasing 参数（attachment 对象）
   * @param lastBlockHeight 当前区块高度
   * @returns phasing 详情对象
   */
  async function getPhasingDetails(
    phasingParams: Record<string, any>,
    lastBlockHeight: number,
  ): Promise<Record<string, any>> {
    const details: Record<string, any> = {}

    const finishHeight = phasingParams.phasingFinishHeight
    details.finishHeight = finishHeight
    const blocksLeft = finishHeight - lastBlockHeight
    details.finishIn = blocksLeft > 0 ? `${blocksLeft} blocks` : 'finished'

    // 投票模型（对标 :71-72）
    const votingModel = phasingParams.phasingVotingModel
    details.votingModel = getVotingModelName(votingModel)

    // quorum / minBalance 精度转换（对标 :73-89）
    // ASSET / CURRENCY 类型需拉取 decimals
    let decimals = 0
    if (votingModel === 2) {
      // ASSET
      try {
        const asset = await nrcsApi.getAsset(phasingParams.phasingHolding)
        if (asset) decimals = asset.decimals
        details.asset_formatted_html = getTransactionLink(phasingParams.phasingHolding)
      } catch {
        // 忽略
      }
    } else if (votingModel === 3) {
      // CURRENCY
      try {
        const currency = await nrcsApi.getCurrency(phasingParams.phasingHolding)
        if (currency) decimals = currency.decimals
        details.currency_formatted_html = getTransactionLink(phasingParams.phasingHolding)
      } catch {
        // 忽略
      }
    }

    // quorum / minBalance 转换为可读格式
    if (decimals > 0) {
      details.quorum = formatAmount(phasingParams.phasingQuorum, false, undefined, true)
      details.minBalance = formatAmount(phasingParams.phasingMinBalance, false, undefined, true)
    } else {
      details.quorum = String(phasingParams.phasingQuorum ?? 0)
      details.minBalance = String(phasingParams.phasingMinBalance ?? 0)
    }

    // minBalanceModel（对标 :96-97）
    details.minBalanceModel = getMinBalanceModelName(phasingParams.phasingMinBalanceModel)

    // whitelist（对标 :99-111）
    if (Array.isArray(phasingParams.phasingWhitelist) && phasingParams.phasingWhitelist.length > 0) {
      details.whitelist = phasingParams.phasingWhitelist.map((id: string | number) =>
        getAccountLink(String(id)),
      )
    } else {
      details.whitelist = '-'
    }

    // linkedFullHashes（对标 :112-121）
    if (
      Array.isArray(phasingParams.phasingLinkedFullHashes) &&
      phasingParams.phasingLinkedFullHashes.length > 0
    ) {
      details.linkedFullHashes = phasingParams.phasingLinkedFullHashes
    } else {
      details.linkedFullHashes = '-'
    }

    // hashedSecret + hashAlgorithm（对标 :122-125）
    if (phasingParams.phasingHashedSecret) {
      details.hashedSecret = phasingParams.phasingHashedSecret
      details.hashAlgorithm = getHashAlgorithmName(phasingParams.phasingHashedSecretAlgorithm)
    }

    return details
  }

  // ----------------------------------------------------------------
  // decryptEncryptedMessages —— 解密加密消息
  // ----------------------------------------------------------------

  /**
   * 解密交易中的加密消息（对标 nrs.modals.transaction.js:298 NRS.tryToDecrypt 调用）。
   *
   * 使用当前账户的 secretPhrase 解密，失败时提示用户输入 sharedKey。
   *
   * @param sharedKey 可选共享密钥
   */
  async function decryptEncryptedMessages(sharedKey?: string): Promise<void> {
    if (!state.value.transaction || !state.value.hasEncryptedMessage) return

    const secretPhrase = accountStore.secretPhrase || ''
    const currentAccount = accountStore.accountId || ''

    const results = await decryption.decryptTransactionMessages(
      state.value.transaction,
      secretPhrase,
      currentAccount,
      sharedKey ? { sharedKey } : {},
    )

    state.value.decryptionResults = results
    state.value.needsSharedKey = decryption.needsSharedKey.value
  }

  /**
   * 用用户输入的 sharedKey 重新解密。
   *
   * @param sharedKey 用户输入的共享密钥（hex）
   */
  async function decryptWithSharedKey(sharedKey: string): Promise<void> {
    if (!state.value.transaction) return

    const currentAccount = accountStore.accountId || ''
    const results = await decryption.decryptWithSharedKey(
      state.value.transaction,
      sharedKey,
      currentAccount,
    )

    state.value.decryptionResults = results
    state.value.needsSharedKey = decryption.needsSharedKey.value
  }

  // ----------------------------------------------------------------
  // closeModal —— 关闭 modal
  // ----------------------------------------------------------------

  /**
   * 关闭交易详情 modal（对标 :1646-1649 onHide）。
   *
   * 清空所有状态，包括解密结果。
   */
  function closeModal(): void {
    state.value = { ...INITIAL_STATE }
    sharedKeyInput.value = ''
  }

  // ----------------------------------------------------------------
  // 计算属性
  // ----------------------------------------------------------------

  /** 当前交易 ID */
  const transactionId = computed<string>(() => state.value.transaction?.transaction ?? '')

  /** 是否有 phasing 详情 */
  const hasPhasingDetails = computed<boolean>(() => state.value.phasingDetails.length > 0)

  /** 是否为消息类交易（type 1 subtype 0） */
  const isMessagingTransaction = computed<boolean>(() => {
    const tx = state.value.transaction
    return tx?.type === 1 && tx?.subtype === 0
  })

  /** 是否可以审批该交易（对标 :193-205） */
  const canApproveTransaction = computed<boolean>(() => {
    const tx = state.value.transaction
    if (!tx || !tx.attachment || !tx.block) return false
    const finishHeight = tx.attachment.phasingFinishHeight
    if (finishHeight === undefined) return false
    return finishHeight > nodeStore.lastBlockHeight
  })

  /** 是否可以扩展数据（对标 :206-212，仅 TaggedDataUpload） */
  const canExtendData = computed<boolean>(() => {
    const tx = state.value.transaction
    return tx?.type === 6 && tx?.subtype === 0
  })

  /** 是否为当前账户发送的交易 */
  const isSentByCurrentAccount = computed<boolean>(() => {
    const tx = state.value.transaction
    return tx?.senderRS === accountStore.accountRS
  })

  return {
    state,
    sharedKeyInput,
    transactionId,
    hasPhasingDetails,
    isMessagingTransaction,
    canApproveTransaction,
    canExtendData,
    isSentByCurrentAccount,
    showTransactionModal,
    getTransaction,
    processTransactionModalData,
    getPhasingDetails,
    decryptEncryptedMessages,
    decryptWithSharedKey,
    closeModal,
  }
}

// ============================================================================
// 辅助函数
// ============================================================================

/**
 * 获取哈希算法名称（对标 NRS.getHashAlgorithm）。
 *
 * @param algorithm 算法编号
 * @returns 算法名称
 */
function getHashAlgorithmName(algorithm?: number): string {
  switch (algorithm) {
    case 0:
      return 'sha256'
    case 1:
      return 'sha3_256'
    case 2:
      return 'keccak_256'
    case 3:
      return 'ripemd_160'
    case 4:
      return 'ripemd_160_sha256'
    case 5:
      return 'keccak_256_sha256'
    default:
      return String(algorithm ?? 0)
  }
}
