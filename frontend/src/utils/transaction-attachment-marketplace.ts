/******************************************************************************
 * NRCS 数字商品市场附件渲染器（type 3）—— 对标 nrs.modals.transaction.js:727-990。
 *
 * subtype 列表：
 *   - 0: 商品上架（dgsListing）
 *   - 1: 商品下架（dgsDelisting）
 *   - 2: 价格变动（dgsPriceChange）
 *   - 3: 数量变动（dgsQuantityChange）
 *   - 4: 购买（dgsPurchase）
 *   - 5: 交付（dgsDelivery）
 *   - 6: 反馈（dgsFeedback）
 *   - 7: 退款（dgsRefund）
 ******************************************************************************/
import type { NrcsTransaction } from '@/api/modules/nrcs.api'
import type { InfoRow } from './transaction-info-table'
import type { RenderOptions, RenderResult } from './transaction-attachment-renderer'
import {
  getTransactionLink,
  getAccountLink,
} from './transaction-links'

// ============================================================================
// renderMarketplaceAttachment —— 主入口
// ============================================================================

/**
 * 渲染数字商品市场交易附件（对标 nrs.modals.transaction.js:727-990）。
 *
 * @param transaction 交易对象
 * @param subtype 子类型
 * @param options 渲染选项
 * @returns 渲染结果
 */
export async function renderMarketplaceAttachment(
  transaction: NrcsTransaction,
  subtype: number,
  _options: RenderOptions,
): Promise<RenderResult> {
  const attachment = transaction.attachment || {}
  const senderLink = getAccountLink(transaction, 'sender')
  const recipientLink = getAccountLink(transaction, 'recipient')

  switch (subtype) {
    // subtype 0: 商品上架
    case 0: {
      const rows: InfoRow[] = [
        { label: 'Type', value: 'dgs_listing' },
        { label: 'Name', value: attachment.name ?? '' },
        { label: 'Description', value: attachment.description ?? '' },
        { label: 'Tags', value: attachment.tags ?? '' },
        { label: 'Quantity', value: String(attachment.quantity ?? 0) },
        { label: 'Price', value: attachment.priceNQT ?? '0' },
        { label: 'Sender', value: senderLink },
      ]
      return { rows, incorrect: false, async: false }
    }

    // subtype 1: 商品下架
    case 1: {
      const rows: InfoRow[] = [
        { label: 'Type', value: 'dgs_delisting' },
        { label: 'Goods', value: getTransactionLink(attachment.goods ?? '0') },
        { label: 'Sender', value: senderLink },
      ]
      return { rows, incorrect: false, async: false }
    }

    // subtype 2: 价格变动
    case 2: {
      const rows: InfoRow[] = [
        { label: 'Type', value: 'dgs_price_change' },
        { label: 'Goods', value: getTransactionLink(attachment.goods ?? '0') },
        { label: 'Price', value: attachment.priceNQT ?? '0' },
        { label: 'Sender', value: senderLink },
      ]
      return { rows, incorrect: false, async: false }
    }

    // subtype 3: 数量变动
    case 3: {
      const rows: InfoRow[] = [
        { label: 'Type', value: 'dgs_quantity_change' },
        { label: 'Goods', value: getTransactionLink(attachment.goods ?? '0') },
        { label: 'Delta Quantity', value: String(attachment.deltaQuantity ?? 0) },
        { label: 'Sender', value: senderLink },
      ]
      return { rows, incorrect: false, async: false }
    }

    // subtype 4: 购买
    case 4: {
      const rows: InfoRow[] = [
        { label: 'Type', value: 'dgs_purchase' },
        { label: 'Goods', value: getTransactionLink(attachment.goods ?? '0') },
        { label: 'Quantity', value: String(attachment.quantity ?? 0) },
        { label: 'Price', value: attachment.priceNQT ?? '0' },
        { label: 'Delivery Deadline', value: String(attachment.deliveryDeadlineTimestamp ?? 0) },
        { label: 'Recipient', value: recipientLink },
        { label: 'Sender', value: senderLink },
      ]
      return { rows, incorrect: false, async: false }
    }

    // subtype 5: 交付
    case 5: {
      const rows: InfoRow[] = [
        { label: 'Type', value: 'dgs_delivery' },
        { label: 'Purchase', value: getTransactionLink(attachment.purchase ?? '0') },
        { label: 'Goods Data', value: attachment.goodsData ?? '' },
        { label: 'Goods Nonce', value: attachment.goodsNonce ?? '' },
        { label: 'Goods Is Text', value: attachment.goodsIsText ? 'true' : 'false' },
        { label: 'Discount', value: attachment.discountNQT ?? '0' },
        { label: 'Recipient', value: recipientLink },
        { label: 'Sender', value: senderLink },
      ]
      return { rows, incorrect: false, async: false }
    }

    // subtype 6: 反馈
    case 6: {
      const rows: InfoRow[] = [
        { label: 'Type', value: 'dgs_feedback' },
        { label: 'Purchase', value: getTransactionLink(attachment.purchase ?? '0') },
        { label: 'Recipient', value: recipientLink },
        { label: 'Sender', value: senderLink },
      ]
      return { rows, incorrect: false, async: false }
    }

    // subtype 7: 退款
    case 7: {
      const rows: InfoRow[] = [
        { label: 'Type', value: 'dgs_refund' },
        { label: 'Purchase', value: getTransactionLink(attachment.purchase ?? '0') },
        { label: 'Refund', value: attachment.refundNQT ?? '0' },
        { label: 'Recipient', value: recipientLink },
        { label: 'Sender', value: senderLink },
      ]
      return { rows, incorrect: false, async: false }
    }

    default:
      return { rows: [], incorrect: true, async: false }
  }
}
