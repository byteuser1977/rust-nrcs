/******************************************************************************
 * NRCS 交易详情链接工具 —— 对标 nrs.util.js / nrs.js 中的链接与账户展示辅助。
 *
 * 在参考 jQuery 实现中，链接通过拼接 `<a href='#' data-xxx='...'>` HTML 字符串实现。
 * Vue3 环境下我们改用「结构化数据」返回：LinkRef 描述目标（type + value + text），
 * 由组件层渲染为 router-link / clickable 元素，避免在工具层拼 HTML。
 *
 * 参考源码：
 *   - nrs.js: NRS.getTransactionLink / NRS.getAccountLink / NRS.getBlockLink /
 *             NRS.getAccountTitle / NRS.getAccountForDecryption
 *   - nrs.modals.transaction.js: 多处调用上述函数
 ******************************************************************************/
import { convertNumericToRSAccountFormat } from './converters'

// ============================================================================
// 类型定义
// ============================================================================

/** 链接目标类型（对标参考中 data-* 属性的语义） */
export type LinkType =
  | 'transaction' // data-transaction
  | 'account' // data-user / data-account
  | 'block' // data-block / goto-block
  | 'asset' // data-goto-asset
  | 'currency' // data-goto-currency
  | 'poll' // data-poll
  | 'alias' // data-alias
  | 'goods' // data-goods
  | 'tagged-data' // data-tagged-data
  | 'shuffling' // data-shuffling

/** 结构化链接引用，由组件层渲染为可点击元素 */
export interface LinkRef {
  /** 链接目标类型 */
  type: LinkType
  /** 目标 ID（数字字符串或 RS 地址） */
  value: string
  /** 显示文本（默认为 value） */
  text?: string
}

/** 信息表行的值类型：纯文本 / HTML 字符串 / 链接 / 链接数组 / 数量（含精度） */
export type InfoValue =
  | string
  | number
  | { html: string } // 已转义的 HTML 字符串（如表格片段）
  | LinkRef
  | LinkRef[]
  | { quantity: string; decimals: number } // [quantityQNT, decimals] 二元组，对标 NRS.formatQuantity 入参
  | { amount: string; decimals?: number } // [amountNQT, decimals] 二元组

// ============================================================================
// 链接构造函数
// ============================================================================

/**
 * 构造交易链接引用（对标 NRS.getTransactionLink(transaction, text, isChainTransaction)）。
 *
 * @param transaction 交易 ID（数字字符串）
 * @param text 可选显示文本（默认为交易 ID）
 * @returns 结构化链接引用，供组件渲染为可点击的交易详情入口
 */
export function getTransactionLink(
  transaction: string | number,
  text?: string,
): LinkRef {
  const value = String(transaction)
  return {
    type: 'transaction',
    value,
    text: text ?? value,
  }
}

/**
 * 构造账户链接引用（对标 NRS.getAccountLink(accountObj, field, useAccountRSForText)）。
 *
 * 接受以下三种输入：
 *   - 字符串（RS 地址或数字 ID）
 *   - 交易/账户对象（含 sender/senderRS 或 recipient/recipientRS 字段）
 *   - 含 account/accountRS 字段的对象
 *
 * @param account 字符串或账户对象
 * @param field 取值字段：'sender' / 'recipient' / 'account' / undefined
 * @returns 结构化链接引用
 */
export function getAccountLink(
  account: string | Record<string, any> | undefined,
  field?: 'sender' | 'recipient' | 'account',
): LinkRef | string {
  if (!account) return '-'

  // 字符串输入：直接构造
  if (typeof account === 'string') {
    const rs = account.includes('-') ? account : convertNumericToRSAccountFormat(account)
    return {
      type: 'account',
      value: rs,
      text: rs,
    }
  }

  // 对象输入：按 field 取相应字段
  let rawId: string | undefined
  let rawRs: string | undefined
  if (field === 'sender') {
    rawId = account.sender
    rawRs = account.senderRS
  } else if (field === 'recipient') {
    rawId = account.recipient
    rawRs = account.recipientRS
  } else {
    // field === 'account' 或 undefined
    rawId = account.account
    rawRs = account.accountRS
  }

  // 两者都没有：返回占位
  if (!rawId && !rawRs) return '-'

  // 优先使用 RS 地址作为显示文本
  const value = rawRs ?? convertNumericToRSAccountFormat(rawId!)
  return {
    type: 'account',
    value,
    text: value,
  }
}

/**
 * 构造区块链接引用（对标 NRS.getBlockLink(height, text)）。
 *
 * @param height 区块高度
 * @param text 可选显示文本（默认为高度数字）
 * @returns 结构化链接引用
 */
export function getBlockLink(height: number | string, text?: string): LinkRef {
  const value = String(height)
  return {
    type: 'block',
    value,
    text: text ?? value,
  }
}

/**
 * 构造资产链接引用（对标 data-goto-asset）。
 *
 * @param asset 资产 ID
 * @param name 可选显示名称
 */
export function getAssetLink(asset: string, name?: string): LinkRef {
  return {
    type: 'asset',
    value: asset,
    text: name ?? asset,
  }
}

/**
 * 构造货币链接引用（对标 data-goto-currency）。
 *
 * @param currency 货币 ID
 * @param code 可选显示代码
 */
export function getCurrencyLink(currency: string, code?: string): LinkRef {
  return {
    type: 'currency',
    value: currency,
    text: code ?? currency,
  }
}

// ============================================================================
// 账户展示辅助
// ============================================================================

/**
 * 获取账户展示标题（对标 NRS.getAccountTitle(accountObj, field)）。
 *
 * 优先使用 RS 地址；如果是当前账户，则使用联系人名（此处简化为返回 RS）。
 *
 * @param account 账户对象或字符串
 * @param field 取值字段
 * @returns 展示用字符串（RS 地址或占位符）
 */
export function getAccountTitle(
  account: string | Record<string, any> | undefined,
  field?: 'sender' | 'recipient' | 'account',
): string {
  const link = getAccountLink(account, field)
  if (typeof link === 'string') return link
  return link.text ?? link.value
}

/**
 * 决定用于解密消息的账户（对标 NRS.getAccountForDecryption(transaction)）。
 *
 * 规则：
 *   - 若当前账户是发送方 → 用接收方公钥解密（对方加密给我的）
 *   - 若当前账户是接收方 → 用发送方公钥解密（对方加密给我的）
 *   - 默认 → 用发送方
 *
 * @param transaction 交易对象
 * @param currentAccount 当前登录账户的数字 ID
 * @returns 用于解密的账户 ID（数字字符串）
 */
export function getAccountForDecryption(
  transaction: { sender?: string; recipient?: string },
  currentAccount: string,
): string {
  // 加密消息的解密方总是接收方（对方用我的公钥加密，我用私钥解密）
  // 但如果是 encryptToSelfMessage，则发送方就是解密方
  // 参考实现：默认返回 transaction.sender（用于 encryptedMessage 解密时取对方公钥）
  // 这里返回交易对手方账户：如果我是发送方，返回接收方；否则返回发送方
  if (transaction.sender === currentAccount) {
    return transaction.recipient ?? transaction.sender ?? '0'
  }
  return transaction.sender ?? '0'
}

/**
 * 将数字账户 ID 转换为 RS 地址格式（对标 NRS.convertNumericToRSAccountFormat）。
 *
 * @param accountId 数字账户 ID（字符串或数字）
 * @returns RS 地址（NRCS-XXXX-XXXX-XXXX-XXXXX），已是 RS 格式则原样返回
 */
export function convertNumericToRSAccount(
  accountId: string | number,
): string {
  const id = String(accountId)
  if (id.includes('-')) return id
  return convertNumericToRSAccountFormat(id)
}
