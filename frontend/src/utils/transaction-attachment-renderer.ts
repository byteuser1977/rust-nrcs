/******************************************************************************
 * NRCS 交易附件渲染器 —— 主入口 + Payment / Messaging / AccountControl /
 * TaggedData 类型渲染。
 *
 * 对标 nrs.modals.transaction.js processTransactionModalData 中按 transaction.type
 * 分发的附件渲染逻辑：
 *   - type 0 (Payment): nrs.modals.transaction.js:233-251
 *   - type 1 (Messaging): nrs.modals.transaction.js:253-553
 *   - type 4 (AccountControl / BalanceLeasing / PhasingOnly): :991-1019
 *   - type 6 (TaggedData): :1194-1212
 *
 * 复杂类型（Asset/Marketplace/Monetary/Shuffling）在独立模块中实现：
 *   - transaction-attachment-asset.ts (type 2)
 *   - transaction-attachment-marketplace.ts (type 3)
 *   - transaction-attachment-monetary.ts (type 5)
 *   - transaction-attachment-shuffling.ts (type 7)
 ******************************************************************************/
import type { NrcsTransaction } from '@/api/modules/nrcs.api'
import type { InfoRow } from './transaction-info-table'
import {
  getTransactionLink,
  getAccountLink,
} from './transaction-links'
import { renderAssetAttachment } from './transaction-attachment-asset'
import { renderMarketplaceAttachment } from './transaction-attachment-marketplace'
import { renderMonetaryAttachment } from './transaction-attachment-monetary'
import { renderShufflingAttachment } from './transaction-attachment-shuffling'

// ============================================================================
// 类型定义
// ============================================================================

/** 渲染附件的选项 */
export interface RenderOptions {
  /** 当前登录账户 ID（用于判断 encryptToSelfMessage 可见性、sender==self 等） */
  currentAccount: string
  /** 当前区块高度（用于 phasing finishIn 计算） */
  lastBlockHeight: number
}

/** 渲染结果 */
export interface RenderResult {
  /** 信息表行（attachment 主体） */
  rows: InfoRow[]
  /** 是否为未知类型（对标参考中 incorrect=true） */
  incorrect: boolean
  /** 是否需要异步加载（如 getAsset/getCurrency 等附加 API 调用） */
  async: boolean
}

// ============================================================================
// renderAttachment —— 主入口（对标 nrs.modals.transaction.js:228-1374 的分发逻辑）
// ============================================================================

/**
 * 渲染交易附件为信息表行（对标 nrs.modals.transaction.js 中的 type/subtype 分发）。
 *
 * 按 transaction.type 分发到专用渲染器：
 *   - type 0 → renderPayment
 *   - type 1 → renderMessaging
 *   - type 2 → renderAssetAttachment（独立模块）
 *   - type 3 → renderMarketplaceAttachment（独立模块）
 *   - type 4 → renderAccountControl
 *   - type 5 → renderMonetaryAttachment（独立模块）
 *   - type 6 → renderTaggedData
 *   - type 7 → renderShufflingAttachment（独立模块）
 *   - 其他 → incorrect=true
 *
 * @param transaction 交易对象
 * @param options 渲染选项
 * @returns 渲染结果
 */
export async function renderAttachment(
  transaction: NrcsTransaction,
  options: RenderOptions,
): Promise<RenderResult> {
  const type = transaction.type ?? 0
  const subtype = transaction.subtype ?? 0

  switch (type) {
    case 0:
      return renderPayment(transaction, subtype)
    case 1:
      return renderMessaging(transaction, subtype)
    case 2:
      return renderAssetAttachment(transaction, subtype, options)
    case 3:
      return renderMarketplaceAttachment(transaction, subtype, options)
    case 4:
      return renderAccountControl(transaction, subtype)
    case 5:
      return renderMonetaryAttachment(transaction, subtype, options)
    case 6:
      return renderTaggedData(transaction, subtype)
    case 7:
      return renderShufflingAttachment(transaction, subtype, options)
    default:
      return { rows: [], incorrect: true, async: false }
  }
}

// ============================================================================
// renderPayment —— type 0: 普通支付（对标 nrs.modals.transaction.js:233-251）
// ============================================================================

/**
 * 渲染普通支付交易附件（type 0, subtype 0）。
 *
 * 对标参考中 data = { type, amount, fee, recipient, sender }。
 *
 * @param transaction 交易对象
 * @param subtype 子类型
 * @returns 渲染结果
 */
function renderPayment(
  transaction: NrcsTransaction,
  subtype: number,
): RenderResult {
  if (subtype !== 0) {
    return { rows: [], incorrect: true, async: false }
  }

  const rows: InfoRow[] = [
    { label: 'Type', value: 'ordinary_payment' },
    { label: 'Amount', value: transaction.amountNQT ?? '0' },
    { label: 'Fee', value: transaction.feeNQT ?? '0' },
    {
      label: 'Recipient',
      value: getAccountLink(transaction, 'recipient'),
    },
    {
      label: 'Sender',
      value: getAccountLink(transaction, 'sender'),
    },
  ]

  return { rows, incorrect: false, async: false }
}

// ============================================================================
// renderMessaging —— type 1: 消息类交易（对标 nrs.modals.transaction.js:253-553）
// ============================================================================

/**
 * 渲染消息类交易附件（type 1）。
 *
 * 按 subtype 分发：
 *   - 0: 普通消息（消息内容在 output 区，附件仅 sender/recipient）
 *   - 1: 别名分配（alias_assignment）
 *   - 2: 投票创建（poll_creation）
 *   - 3: 投票（vote_casting）
 *   - 4: Hub 公告（hub_announcement）
 *   - 5: 账户信息（account_info）
 *   - 6: 别名出售/转让/取消（alias_sale / alias_transfer / alias_sale_cancellation）
 *   - 7: 别名购买（alias_buy）
 *   - 8: 别名删除（alias_deletion）
 *   - 9: 交易审批（transaction_approval）
 *   - 10: 设置账户属性（set_account_property）
 *   - 11: 删除账户属性（delete_account_property）
 *
 * @param transaction 交易对象
 * @param subtype 子类型
 * @returns 渲染结果
 */
function renderMessaging(
  transaction: NrcsTransaction,
  subtype: number,
): RenderResult {
  const attachment = transaction.attachment || {}
  const senderLink = getAccountLink(transaction, 'sender')

  switch (subtype) {
    // subtype 0: 普通消息（消息内容在 output 区渲染，此处仅返回空 rows）
    case 0:
      return { rows: [], incorrect: false, async: false }

    // subtype 1: 别名分配
    case 1: {
      const rows: InfoRow[] = [
        { label: 'Type', value: 'alias_assignment' },
        { label: 'Alias', value: attachment.alias ?? '' },
        { label: 'Data', value: attachment.uri ?? '' },
        { label: 'Sender', value: senderLink },
      ]
      return { rows, incorrect: false, async: false }
    }

    // subtype 2: 投票创建
    case 2: {
      const rows: InfoRow[] = [
        { label: 'Type', value: 'poll_creation' },
        { label: 'Name', value: attachment.name ?? '' },
        { label: 'Description', value: attachment.description ?? '' },
        { label: 'Finish Height', value: String(attachment.finishHeight ?? 0) },
        { label: 'Min Number Of Options', value: String(attachment.minNumberOfOptions ?? 0) },
        { label: 'Max Number Of Options', value: String(attachment.maxNumberOfOptions ?? 0) },
        { label: 'Min Range Value', value: String(attachment.minRangeValue ?? 0) },
        { label: 'Max Range Value', value: String(attachment.maxRangeValue ?? 0) },
        { label: 'Min Balance', value: String(attachment.minBalance ?? 0) },
        { label: 'Min Balance Model', value: String(attachment.minBalanceModel ?? 0) },
      ]

      // votingModel 映射（对标 :348-366）
      const votingModelName = getVotingModelName(attachment.votingModel)
      rows.push({ label: 'Voting Model', value: votingModelName })
      if (attachment.votingModel === 2) {
        rows.push({ label: 'Asset ID', value: getTransactionLink(attachment.holding ?? '0') })
      } else if (attachment.votingModel === 3) {
        rows.push({ label: 'Currency ID', value: getTransactionLink(attachment.holding ?? '0') })
      }

      // 选项列表（对标 :369-371）
      if (Array.isArray(attachment.options)) {
        attachment.options.forEach((opt: string, i: number) => {
          rows.push({ label: `Option ${i + 1}`, value: opt })
        })
      }

      rows.push({ label: 'Sender', value: senderLink })
      return { rows, incorrect: false, async: false }
    }

    // subtype 3: 投票
    case 3: {
      const votes = Array.isArray(attachment.vote) ? attachment.vote : []
      const voteStr = votes
        .map((v: number) => (v === -128 ? 'N/A' : String(v)))
        .join(' , ')

      const rows: InfoRow[] = [
        { label: 'Type', value: 'vote_casting' },
        { label: 'Poll', value: getTransactionLink(attachment.poll ?? '0') },
        { label: 'Vote', value: voteStr },
        { label: 'Sender', value: senderLink },
      ]
      return { rows, incorrect: false, async: false }
    }

    // subtype 4: Hub 公告
    case 4: {
      const rows: InfoRow[] = [
        { label: 'Type', value: 'hub_announcement' },
        { label: 'Sender', value: senderLink },
      ]
      return { rows, incorrect: false, async: false }
    }

    // subtype 5: 账户信息
    case 5: {
      const rows: InfoRow[] = [
        { label: 'Type', value: 'account_info' },
        { label: 'Name', value: attachment.name ?? '' },
        { label: 'Description', value: attachment.description ?? '' },
      ]
      return { rows, incorrect: false, async: false }
    }

    // subtype 6: 别名出售/转让/取消
    case 6: {
      let typeLabel = 'alias_sale'
      if (attachment.priceNQT === '0') {
        if (transaction.sender === transaction.recipient) {
          typeLabel = 'alias_sale_cancellation'
        } else {
          typeLabel = 'alias_transfer'
        }
      }

      const rows: InfoRow[] = [
        { label: 'Type', value: typeLabel },
        { label: 'Alias Name', value: attachment.alias ?? '' },
      ]

      if (typeLabel === 'alias_sale') {
        rows.push({ label: 'Price', value: attachment.priceNQT ?? '0' })
      }
      if (typeLabel !== 'alias_sale_cancellation') {
        rows.push({
          label: 'Recipient',
          value: getAccountLink(transaction, 'recipient'),
        })
      }
      rows.push({ label: 'Sender', value: senderLink })

      return { rows, incorrect: false, async: false }
    }

    // subtype 7: 别名购买
    case 7: {
      const rows: InfoRow[] = [
        { label: 'Type', value: 'alias_buy' },
        { label: 'Alias Name', value: attachment.alias ?? '' },
        { label: 'Price', value: transaction.amountNQT ?? '0' },
        { label: 'Recipient', value: getAccountLink(transaction, 'recipient') },
        { label: 'Sender', value: senderLink },
      ]
      return { rows, incorrect: false, async: false }
    }

    // subtype 8: 别名删除
    case 8: {
      const rows: InfoRow[] = [
        { label: 'Type', value: 'alias_deletion' },
        { label: 'Alias Name', value: attachment.alias ?? '' },
        { label: 'Sender', value: senderLink },
      ]
      return { rows, incorrect: false, async: false }
    }

    // subtype 9: 交易审批
    case 9: {
      const rows: InfoRow[] = [{ label: 'Type', value: 'transaction_approval' }]
      const fullHashes = Array.isArray(attachment.transactionFullHashes)
        ? attachment.transactionFullHashes
        : []
      fullHashes.forEach((_: string, i: number) => {
        // 参考实现：byteArrayToBigInteger(hexStringToByteArray(fullHash)) → transactionId
        // 简化：直接用 fullHash 作为链接（实际应转换为 numeric ID）
        rows.push({
          label: `Transaction ${i + 1}`,
          value: getTransactionLink(fullHashes[i]),
        })
      })
      rows.push({ label: 'Sender', value: senderLink })
      return { rows, incorrect: false, async: false }
    }

    // subtype 10: 设置账户属性
    case 10: {
      const rows: InfoRow[] = [
        { label: 'Type', value: 'set_account_property' },
        { label: 'Property', value: attachment.property ?? '' },
        { label: 'Value', value: attachment.value ?? '' },
      ]
      return { rows, incorrect: false, async: false }
    }

    // subtype 11: 删除账户属性
    case 11: {
      const rows: InfoRow[] = [
        { label: 'Type', value: 'delete_account_property' },
        { label: 'Property', value: getTransactionLink(attachment.property ?? '0') },
      ]
      return { rows, incorrect: false, async: false }
    }

    default:
      return { rows: [], incorrect: true, async: false }
  }
}

// ============================================================================
// renderAccountControl —— type 4: 账户控制（对标 nrs.modals.transaction.js:991-1019）
// ============================================================================

/**
 * 渲染账户控制类交易附件（type 4）。
 *
 * subtype：
 *   - 0: 余额租赁（leaseBalance）
 *   - 1: 设置 Phasing Only 控制（setPhasingOnlyControl）
 *
 * @param transaction 交易对象
 * @param subtype 子类型
 * @returns 渲染结果
 */
function renderAccountControl(
  transaction: NrcsTransaction,
  subtype: number,
): RenderResult {
  const attachment = transaction.attachment || {}
  const senderLink = getAccountLink(transaction, 'sender')

  switch (subtype) {
    // subtype 0: 余额租赁
    case 0: {
      const rows: InfoRow[] = [
        { label: 'Type', value: 'balance_leasing' },
        { label: 'Period', value: String(attachment.period ?? 0) },
        { label: 'Sender', value: senderLink },
      ]
      return { rows, incorrect: false, async: false }
    }

    // subtype 1: 设置 Phasing Only 控制
    case 1: {
      const rows: InfoRow[] = [
        { label: 'Type', value: 'set_phasing_only_control' },
        { label: 'Voting Model', value: getVotingModelName(attachment.controlVotingModel) },
        { label: 'Quorum', value: String(attachment.controlQuorum ?? 0) },
        { label: 'Min Balance', value: String(attachment.controlMinBalance ?? 0) },
        { label: 'Min Balance Model', value: getMinBalanceModelName(attachment.controlMinBalanceModel) },
        { label: 'Sender', value: senderLink },
      ]
      return { rows, incorrect: false, async: false }
    }

    default:
      return { rows: [], incorrect: true, async: false }
  }
}

// ============================================================================
// renderTaggedData —— type 6: 标签化数据（对标 nrs.modals.transaction.js:1194-1212 + getTaggedData）
// ============================================================================

/**
 * 渲染标签化数据交易附件（type 6）。
 *
 * 对标 NRS.getTaggedData(attachment, subtype, transaction)。
 *
 * subtype：
 *   - 0: 上传（uploadTaggedData）
 *   - 1: 扩展（extendTaggedData）
 *
 * @param transaction 交易对象
 * @param subtype 子类型
 * @returns 渲染结果
 */
function renderTaggedData(
  transaction: NrcsTransaction,
  subtype: number,
): RenderResult {
  const attachment = transaction.attachment || {}

  switch (subtype) {
    case 0:
    case 1: {
      const rows: InfoRow[] = []

      // 哈希
      if (attachment.hash) {
        rows.push({ label: 'Hash', value: attachment.hash })
      }

      // 关联的 tagged data（subtype 1: extend）
      if (attachment.taggedData) {
        rows.push({
          label: 'Tagged Data',
          value: getTransactionLink(attachment.taggedData),
        })
      }

      // 数据字段（subtype 0: upload）
      if (attachment.data) {
        rows.push({ label: 'Name', value: attachment.name ?? '' })
        rows.push({ label: 'Description', value: attachment.description ?? '' })
        rows.push({ label: 'Tags', value: attachment.tags ?? '' })
        rows.push({ label: 'Mime Type', value: attachment.type ?? '' })
        rows.push({ label: 'Channel', value: attachment.channel ?? '' })
        rows.push({ label: 'Is Text', value: attachment.isText ? 'true' : 'false' })
        rows.push({ label: 'Filename', value: attachment.filename ?? '' })

        // 数据大小（对标 :1594-1597）
        const dataSize = attachment.isText
          ? new TextEncoder().encode(attachment.data).length
          : Math.floor((attachment.data || '').length / 2)
        rows.push({ label: 'Data Size', value: String(dataSize) })
      }

      return { rows, incorrect: false, async: false }
    }

    default:
      return { rows: [], incorrect: true, async: false }
  }
}

// ============================================================================
// 辅助函数
// ============================================================================

/**
 * 获取投票模型名称（对标 NRS.getVotingModelName）。
 *
 * @param votingModel 投票模型编号
 * @returns 名称字符串
 */
export function getVotingModelName(votingModel?: number): string {
  switch (votingModel) {
    case 0:
      return 'vote_by_account'
    case 1:
      return 'vote_by_balance'
    case 2:
      return 'vote_by_asset'
    case 3:
      return 'vote_by_currency'
    case 4:
      return 'vote_by_transaction'
    case 5:
      return 'vote_by_hash'
    case -1:
      return 'vote_by_none'
    default:
      return String(votingModel ?? 0)
  }
}

/**
 * 获取最低余额模型名称（对标 NRS.getMinBalanceModelName）。
 *
 * @param minBalanceModel 最低余额模型编号
 * @returns 名称字符串
 */
export function getMinBalanceModelName(minBalanceModel?: number): string {
  switch (minBalanceModel) {
    case 0:
      return 'min_balance_model_nqt'
    case 1:
      return 'min_balance_model_asset'
    case 2:
      return 'min_balance_model_currency'
    default:
      return String(minBalanceModel ?? 0)
  }
}
