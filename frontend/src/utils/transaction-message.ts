/******************************************************************************
 * NRCS 交易消息解析 —— 对标 nrs.messages.js / nrs.modals.transaction.js 中的
 * 消息字段处理逻辑。
 *
 * 参考：
 *   - nrs.modals.transaction.js:256-321 公开消息解析（含 legacy hex）
 *   - nrs.modals.transaction.js:1325-1374 底部消息区
 *   - nrs.messages.js:111 isTextMessage
 *   - nrs.util.js convertFromHex8 / convertFromHex16
 ******************************************************************************/
import { hexStringToString } from './converters'
import type { NrcsTransaction } from '@/api/modules/nrcs.api'

// ============================================================================
// 常量
// ============================================================================

/** UTF-16 BOM 前缀（对标参考中 `"feff"` 检测） */
const UTF16_BOM_PREFIX = 'feff'

// ============================================================================
// 类型定义
// ============================================================================

/** 解析后的公开消息 */
export interface ParsedPublicMessage {
  /** 消息文本（已解码，可能为二进制数据占位） */
  text: string
  /** 是否为二进制数据（无法解码为文本） */
  isBinary: boolean
  /** 是否为 legacy UTF-16 编码 */
  isLegacyUtf16: boolean
}

// ============================================================================
// isTextMessage —— 对标 NRS.isTextMessage(transaction)
// ============================================================================

/**
 * 判断交易附件中的消息是否为文本（对标 NRS.isTextMessage）。
 *
 * 参考实现（nrs.messages.js:111）：
 *   - 若 attachment.messageIsText === true → true
 *   - 若 attachment.messageIsPrunable 且 attachment."version.PrunablePlainMessage" 存在
 *     且 messageIsText === false → false（prunable 二进制）
 *   - 否则按 messageIsText 字段
 *
 * @param transaction 交易对象
 * @returns true 表示是文本消息
 */
export function isTextMessage(transaction: NrcsTransaction): boolean {
  const attachment = transaction.attachment
  if (!attachment) return false

  // 显式标记
  if (attachment.messageIsText === true) return true
  if (attachment.messageIsText === false) return false

  // prunable 消息：检查 version.PrunablePlainMessage
  if (attachment.messageIsPrunable) {
    if (attachment['version.PrunablePlainMessage'] !== undefined) {
      return attachment.messageIsText !== false
    }
  }

  // 默认按 false 处理（参考行为）
  return false
}

// ============================================================================
// parsePublicMessage —— 对标 nrs.modals.transaction.js:256-276 的消息解析
// ============================================================================

/**
 * 解析交易附件中的公开消息（对标 nrs.modals.transaction.js:256-276）。
 *
 * 参考实现分三种情况：
 *   1. 无 version.Message 且无 version.PrunablePlainMessage（legacy）：
 *      - 尝试 hexStringToString
 *      - 失败时：若以 "feff" 开头 → convertFromHex16（UTF-16）
 *      - 否则 → convertFromHex8（UTF-8 legacy）
 *   2. 有 version.Message 或 version.PrunablePlainMessage：
 *      - messageIsText=true → 直接 String(message)
 *      - messageIsText=false → "binary_data" 占位
 *
 * @param attachment 交易附件
 * @returns 解析结果
 */
export function parsePublicMessage(
  attachment: Record<string, any> | undefined,
): ParsedPublicMessage | null {
  if (!attachment || !attachment.message) {
    return null
  }

  const hasVersionMessage =
    attachment['version.Message'] !== undefined ||
    attachment['version.PrunablePlainMessage'] !== undefined

  // 情况 1：legacy 消息（无版本标记）
  if (!hasVersionMessage) {
    try {
      const text = hexStringToString(attachment.message)
      return {
        text,
        isBinary: false,
        isLegacyUtf16: false,
      }
    } catch {
      // legacy UTF-16 检测
      if (typeof attachment.message === 'string' && attachment.message.startsWith(UTF16_BOM_PREFIX)) {
        const text = convertFromHex16(attachment.message)
        return {
          text,
          isBinary: false,
          isLegacyUtf16: true,
        }
      }
      // legacy UTF-8
      const text = convertFromHex8(attachment.message)
      return {
        text,
        isBinary: false,
        isLegacyUtf16: false,
      }
    }
  }

  // 情况 2：新版本消息
  if (attachment.messageIsText) {
    return {
      text: String(attachment.message),
      isBinary: false,
      isLegacyUtf16: false,
    }
  }

  // 二进制消息
  return {
    text: 'binary_data',
    isBinary: true,
    isLegacyUtf16: false,
  }
}

// ============================================================================
// convertFromHex8 / convertFromHex16 —— 对标 nrs.util.js
// ============================================================================

/**
 * 将 hex 字符串按 UTF-8 解码（legacy 路径，对标 NRS.convertFromHex8）。
 *
 * @param hex hex 字符串
 * @returns 解码后的字符串
 */
function convertFromHex8(hex: string): string {
  // 等价于 hexStringToString 的回退路径
  try {
    return hexStringToString(hex)
  } catch {
    return hex
  }
}

/**
 * 将 hex 字符串按 UTF-16 解码（legacy 路径，对标 NRS.convertFromHex16）。
 *
 * 输入通常以 "feff"（BOM）开头。
 *
 * @param hex hex 字符串
 * @returns 解码后的字符串
 */
function convertFromHex16(hex: string): string {
  // 去掉 BOM 前缀
  let h = hex
  if (h.startsWith(UTF16_BOM_PREFIX)) {
    h = h.substring(UTF16_BOM_PREFIX.length)
  }
  // 每两个字节（4 个 hex 字符）组成一个 UTF-16 码元
  let result = ''
  for (let i = 0; i + 3 < h.length; i += 4) {
    const code = parseInt(h.substring(i, i + 4), 16)
    result += String.fromCharCode(code)
  }
  return result
}

// ============================================================================
// getDecryptionFields —— 对标 nrs.modals.transaction.js 中的 fieldsToDecrypt 构造
// ============================================================================

/** 待解密字段描述 */
export interface DecryptField {
  /** 附件字段名（如 'encryptedMessage' / 'encryptToSelfMessage'） */
  fieldName: string
  /** 展示标签（如「加密消息」/「给自己的备注」） */
  label: string
}

/**
 * 从交易附件中识别需要解密的字段（对标 nrs.modals.transaction.js:279-289, 1351-1365）。
 *
 * 规则：
 *   - attachment.encryptedMessage 存在 → 添加 encryptedMessage 字段
 *   - attachment.encryptToSelfMessage 存在且当前账户是发送方 → 添加 encryptToSelfMessage 字段
 *
 * @param transaction 交易对象
 * @param currentAccount 当前登录账户 ID
 * @returns 待解密字段列表
 */
export function getDecryptionFields(
  transaction: NrcsTransaction,
  currentAccount: string,
): DecryptField[] {
  const fields: DecryptField[] = []
  const attachment = transaction.attachment
  if (!attachment) return fields

  if (attachment.encryptedMessage) {
    fields.push({
      fieldName: 'encryptedMessage',
      label: 'encrypted_message',
    })
  }

  if (attachment.encryptToSelfMessage && transaction.sender === currentAccount) {
    fields.push({
      fieldName: 'encryptToSelfMessage',
      label: 'note_to_self',
    })
  }

  return fields
}

// ============================================================================
// getMessageHash —— 对标 nrs.modals.transaction.js:309
// ============================================================================

/**
 * 获取消息哈希（对标 nrs.modals.transaction.js:309）。
 *
 * 优先取 messageHash，回退到 encryptedMessageHash。
 *
 * @param attachment 交易附件
 * @returns 哈希字符串或 undefined
 */
export function getMessageHash(
  attachment: Record<string, any> | undefined,
): string | undefined {
  if (!attachment) return undefined
  return attachment.messageHash || attachment.encryptedMessageHash
}
