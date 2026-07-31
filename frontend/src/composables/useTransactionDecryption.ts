/******************************************************************************
 * NRCS 交易加密消息解密 composable —— 对标 nrs.encryption.js NRS.tryToDecrypt /
 * NRS.tryToDecryptMessage。
 *
 * 参考：
 *   - nrs.modals.transaction.js:284-298 调用 NRS.tryToDecrypt 解密 encryptedMessage / encryptToSelfMessage
 *   - nrs.encryption.js NRS.tryToDecrypt(transaction, fieldsToDecrypt, account, options)
 *   - nrs.encryption.js NRS.decryptNote / NRS.tryToDecryptMessage
 *
 * 安全模型：解密在客户端本地完成，secretPhrase 不出客户端。
 * 解密流程：
 *   1. 从 account.store 获取当前账户的 secretPhrase（内存暂存）
 *   2. 通过 transaction.senderPublicKey 或 transaction.recipientPublicKey 派生对方公钥
 *   3. 调用 decryptNote(message, { nonce, publicKey, privateKey }) 解密
 *   4. 失败时提示用户输入 sharedKey 或 secretPhrase
 ******************************************************************************/
import { ref, type Ref } from 'vue'
import { decryptNote, getPrivateKey, getPublicKey } from '@/utils/nrcs-crypto'
import { hexStringToByteArray } from '@/utils/converters'
import { getAccountForDecryption } from '@/utils/transaction-links'
import { getDecryptionFields, type DecryptField } from '@/utils/transaction-message'
import type { NrcsTransaction } from '@/api/modules/nrcs.api'

// ============================================================================
// 类型定义
// ============================================================================

/** 解密结果 */
export interface DecryptionResult {
  /** 字段名（如 'encryptedMessage'） */
  fieldName: string
  /** 展示标签 */
  label: string
  /** 解密后的消息文本 */
  message: string
  /** 共享密钥（hex，用于显示或后续解密） */
  sharedKey: string
  /** 是否解密成功 */
  success: boolean
  /** 失败原因（失败时） */
  error?: string
}

/** 解密选项 */
export interface DecryptOptions {
  /** 共享密钥（hex，可选，用于无法用账户密钥解密的场景） */
  sharedKey?: string
  /** 是否为文本消息（默认 true） */
  isText?: boolean
  /** 是否已压缩（默认 true，新版交易默认压缩） */
  isCompressed?: boolean
}

// ============================================================================
// composable
// ============================================================================

/**
 * 交易加密消息解密 composable。
 *
 * 提供：
 *   - decryptTransactionMessages：批量解密交易中的加密消息字段
 *   - decryptField：解密单个字段
 *   - needsDecryption：判断交易是否有待解密的加密消息
 *
 * @returns 解密相关状态与方法
 */
export function useTransactionDecryption() {
  /** 解密结果列表（响应式） */
  const decryptionResults: Ref<DecryptionResult[]> = ref([])

  /** 是否需要 sharedKey 输入（账户密钥解密失败时） */
  const needsSharedKey = ref(false)

  /** 是否正在解密 */
  const decrypting = ref(false)

  // ----------------------------------------------------------------
  // needsDecryption —— 判断交易是否有待解密的加密消息
  // ----------------------------------------------------------------

  /**
   * 判断交易是否有待解密的加密消息字段。
   *
   * @param transaction 交易对象
   * @param currentAccount 当前账户 ID
   * @returns true 表示有加密消息需要解密
   */
  function needsDecryption(
    transaction: NrcsTransaction,
    currentAccount: string,
  ): boolean {
    return getDecryptionFields(transaction, currentAccount).length > 0
  }

  // ----------------------------------------------------------------
  // decryptField —— 解密单个字段
  // ----------------------------------------------------------------

  /**
   * 解密单个加密消息字段（对标 NRS.decryptNote 的调用）。
   *
   * @param transaction 交易对象
   * @param field 字段描述（fieldName + label）
   * @param secretPhrase 当前账户的 secretPhrase（来自 account.store 内存暂存）
   * @param options 解密选项（sharedKey / isText / isCompressed）
   * @returns 解密结果
   */
  function decryptField(
    transaction: NrcsTransaction,
    field: DecryptField,
    secretPhrase: string,
    options: DecryptOptions = {},
  ): DecryptionResult {
    const attachment = transaction.attachment
    if (!attachment || !attachment[field.fieldName]) {
      return {
        fieldName: field.fieldName,
        label: field.label,
        message: '',
        sharedKey: '',
        success: false,
        error: '字段不存在',
      }
    }

    const encryptedMsg = attachment[field.fieldName]
    // 加密消息结构：{ data, nonce, isText, isCompressed, isPrunable }
    const dataHex = encryptedMsg.data ?? encryptedMsg.encryptedMessageData
    const nonceHex = encryptedMsg.nonce ?? encryptedMsg.encryptedMessageNonce
    const isText = options.isText ?? (encryptedMsg.isText !== false)
    const isCompressed = options.isCompressed ?? (encryptedMsg.isCompressed !== false)

    if (!dataHex || !nonceHex) {
      return {
        fieldName: field.fieldName,
        label: field.label,
        message: '',
        sharedKey: '',
        success: false,
        error: '加密数据缺失',
      }
    }

    try {
      // 决定解密用的对方公钥
      // encryptedMessage：用对方公钥（发送方用接收方公钥加密，接收方用发送方公钥解密）
      // encryptToSelfMessage：用自己公钥（自己加密给自己）
      const isEncryptToSelf = field.fieldName === 'encryptToSelfMessage'
      const otherPublicKey = isEncryptToSelf
        ? getPublicKey(secretPhrase) // 自己加密给自己 → 解密也用自己公钥派生共享密钥
        : transaction.senderPublicKey || transaction.recipientPublicKey

      if (!otherPublicKey && !options.sharedKey) {
        return {
          fieldName: field.fieldName,
          label: field.label,
          message: '',
          sharedKey: '',
          success: false,
          error: '缺少对方公钥，需要 sharedKey',
        }
      }

      // decryptNote 期望 Uint8Array（hexStringToByteArray 转换）
      const result = decryptNote(dataHex, {
        nonce: hexStringToByteArray(nonceHex),
        publicKey: otherPublicKey ? hexStringToByteArray(otherPublicKey) : undefined,
        privateKey: hexStringToByteArray(getPrivateKey(secretPhrase)),
        sharedKey: options.sharedKey
          ? hexStringToByteArray(options.sharedKey)
          : undefined,
        isText,
        isCompressed,
      }, secretPhrase)

      return {
        fieldName: field.fieldName,
        label: field.label,
        message: result.message,
        sharedKey: result.sharedKey,
        success: true,
      }
    } catch (e) {
      const error = e instanceof Error ? e.message : String(e)
      return {
        fieldName: field.fieldName,
        label: field.label,
        message: '',
        sharedKey: '',
        success: false,
        error,
      }
    }
  }

  // ----------------------------------------------------------------
  // decryptTransactionMessages —— 批量解密
  // ----------------------------------------------------------------

  /**
   * 批量解密交易中的所有加密消息字段（对标 nrs.modals.transaction.js 中的 NRS.tryToDecrypt 调用）。
   *
   * @param transaction 交易对象
   * @param secretPhrase 当前账户的 secretPhrase
   * @param currentAccount 当前账户 ID（用于判断 encryptToSelfMessage 是否可见）
   * @param options 解密选项
   * @returns 解密结果列表
   */
  async function decryptTransactionMessages(
    transaction: NrcsTransaction,
    secretPhrase: string,
    currentAccount: string,
    options: DecryptOptions = {},
  ): Promise<DecryptionResult[]> {
    decrypting.value = true
    needsSharedKey.value = false
    decryptionResults.value = []

    try {
      const fields = getDecryptionFields(transaction, currentAccount)
      const results: DecryptionResult[] = []

      for (const field of fields) {
        const result = decryptField(transaction, field, secretPhrase, options)
        results.push(result)
        // 若因缺少公钥失败 → 标记需要 sharedKey 输入
        if (!result.success && result.error?.includes('sharedKey')) {
          needsSharedKey.value = true
        }
      }

      decryptionResults.value = results
      return results
    } finally {
      decrypting.value = false
    }
  }

  // ----------------------------------------------------------------
  // decryptWithSharedKey —— 用 sharedKey 重新解密
  // ----------------------------------------------------------------

  /**
   * 用用户提供的 sharedKey 重新解密（对标参考中 sharedKey 输入框提交后的行为）。
   *
   * @param transaction 交易对象
   * @param sharedKey 用户输入的共享密钥（hex）
   * @param currentAccount 当前账户 ID
   * @returns 解密结果列表
   */
  async function decryptWithSharedKey(
    transaction: NrcsTransaction,
    sharedKey: string,
    currentAccount: string,
  ): Promise<DecryptionResult[]> {
    return decryptTransactionMessages(transaction, '', currentAccount, { sharedKey })
  }

  return {
    decryptionResults,
    needsSharedKey,
    decrypting,
    needsDecryption,
    decryptField,
    decryptTransactionMessages,
    decryptWithSharedKey,
  }
}
