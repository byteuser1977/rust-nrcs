/******************************************************************************
 * NRCS 资产类型交易附件渲染器（type 2）—— 对标 nrs.modals.transaction.js:554-726。
 *
 * subtype 列表：
 *   - 0: 资产发行（issueAsset）— 需要 getAsset 拉取 initialQuantity
 *   - 1: 资产转移（transferAsset）
 *   - 2: 卖单（placeAskOrder）— 需要 getAsset + getOrderTrades
 *   - 3: 买单（placeBidOrder）— 需要 getAsset + getOrderTrades
 *   - 4: 取消卖单（cancelAskOrder）
 *   - 5: 取消买单（cancelBidOrder）
 *   - 6: 股息支付（dividendPayment）
 *   - 7: 删除资产份额（deleteAssetShares）
 ******************************************************************************/
import { nrcsApi } from '@/api/modules/nrcs.api'
import type { NrcsTransaction } from '@/api/modules/nrcs.api'
import type { InfoRow } from './transaction-info-table'
import type { RenderOptions, RenderResult } from './transaction-attachment-renderer'
import {
  getTransactionLink,
  getAccountLink,
  getAssetLink,
} from './transaction-links'
import {
  formatAmount,
  formatQuantity,
  formatOrderPricePerWholeQNT,
  calculateOrderTotalNQT,
} from './format'

// ============================================================================
// renderAssetAttachment —— 主入口
// ============================================================================

/**
 * 渲染资产类型交易附件（对标 nrs.modals.transaction.js:554-726）。
 *
 * @param transaction 交易对象
 * @param subtype 子类型
 * @param options 渲染选项
 * @returns 渲染结果（部分 subtype 需要异步拉取资产信息）
 */
export async function renderAssetAttachment(
  transaction: NrcsTransaction,
  subtype: number,
  _options: RenderOptions,
): Promise<RenderResult> {
  const attachment = transaction.attachment || {}
  const senderLink = getAccountLink(transaction, 'sender')

  switch (subtype) {
    // subtype 0: 资产发行
    case 0: {
      const rows: InfoRow[] = [
        { label: 'Type', value: 'asset_issuance' },
        { label: 'Name', value: attachment.name ?? '' },
        { label: 'Decimals', value: String(attachment.decimals ?? 0) },
        { label: 'Description', value: attachment.description ?? '' },
      ]

      // 拉取资产信息获取 initialQuantity / 当前 quantity（对标 :557-575）
      try {
        const asset = await nrcsApi.getAsset(transaction.transaction ?? '')
        if (asset) {
          rows.push({
            label: 'Initial Quantity',
            value: { quantity: asset.quantityQNT, decimals: attachment.decimals ?? 0 },
          })
        }
      } catch {
        // 忽略错误，仅展示附件字段
      }

      rows.push({ label: 'Sender', value: senderLink })
      return { rows, incorrect: false, async: true }
    }

    // subtype 1: 资产转移
    case 1: {
      const rows: InfoRow[] = [
        { label: 'Type', value: 'asset_transfer' },
        { label: 'Asset', value: getAssetLink(attachment.asset ?? '0') },
        {
          label: 'Quantity',
          value: { quantity: attachment.quantityQNT ?? '0', decimals: 0 },
        },
        { label: 'Recipient', value: getAccountLink(transaction, 'recipient') },
        { label: 'Sender', value: senderLink },
      ]

      // 拉取资产精度（对标参考中 transferAsset 分支的 getAsset 调用）
      try {
        const asset = await nrcsApi.getAsset(attachment.asset ?? '')
        if (asset) {
          // 更新 Quantity 行的 decimals
          const qtyRow = rows.find((r) => r.label === 'Quantity')
          if (qtyRow && typeof qtyRow.value === 'object' && 'quantity' in qtyRow.value) {
            qtyRow.value = { quantity: attachment.quantityQNT ?? '0', decimals: asset.decimals }
          }
          // 更新 Asset 行的显示名
          const assetRow = rows.find((r) => r.label === 'Asset')
          if (assetRow) {
            assetRow.value = getAssetLink(attachment.asset ?? '0', asset.name)
          }
        }
      } catch {
        // 忽略
      }

      return { rows, incorrect: false, async: true }
    }

    // subtype 2/3: 卖单/买单
    case 2:
    case 3: {
      return renderAssetOrder(transaction, subtype)
    }

    // subtype 4/5: 取消卖单/买单
    case 4:
    case 5: {
      const rows: InfoRow[] = [
        {
          label: 'Type',
          value: subtype === 4 ? 'ask_order_cancellation' : 'bid_order_cancellation',
        },
        { label: 'Order', value: getTransactionLink(attachment.order ?? '0') },
        { label: 'Sender', value: senderLink },
      ]
      return { rows, incorrect: false, async: false }
    }

    // subtype 6: 股息支付
    case 6: {
      const rows: InfoRow[] = [
        { label: 'Type', value: 'dividend_payment' },
        { label: 'Asset', value: getAssetLink(attachment.asset ?? '0') },
        { label: 'Height', value: String(attachment.height ?? 0) },
        { label: 'Amount NQT Per QNT', value: attachment.amountNQTPerQNT ?? '0' },
        { label: 'Sender', value: senderLink },
      ]

      // 拉取资产精度
      try {
        const asset = await nrcsApi.getAsset(attachment.asset ?? '')
        if (asset) {
          const assetRow = rows.find((r) => r.label === 'Asset')
          if (assetRow) {
            assetRow.value = getAssetLink(attachment.asset ?? '0', asset.name)
          }
        }
      } catch {
        // 忽略
      }

      return { rows, incorrect: false, async: true }
    }

    // subtype 7: 删除资产份额
    case 7: {
      const rows: InfoRow[] = [
        { label: 'Type', value: 'delete_asset_shares' },
        { label: 'Asset', value: getAssetLink(attachment.asset ?? '0') },
        {
          label: 'Quantity',
          value: { quantity: attachment.quantityQNT ?? '0', decimals: 0 },
        },
        { label: 'Sender', value: senderLink },
      ]

      try {
        const asset = await nrcsApi.getAsset(attachment.asset ?? '')
        if (asset) {
          const qtyRow = rows.find((r) => r.label === 'Quantity')
          if (qtyRow && typeof qtyRow.value === 'object' && 'quantity' in qtyRow.value) {
            qtyRow.value = { quantity: attachment.quantityQNT ?? '0', decimals: asset.decimals }
          }
          const assetRow = rows.find((r) => r.label === 'Asset')
          if (assetRow) {
            assetRow.value = getAssetLink(attachment.asset ?? '0', asset.name)
          }
        }
      } catch {
        // 忽略
      }

      return { rows, incorrect: false, async: true }
    }

    default:
      return { rows: [], incorrect: true, async: false }
  }
}

// ============================================================================
// renderAssetOrder —— 卖单/买单渲染（对标 NRS.formatAssetOrder :1397-1452）
// ============================================================================

/**
 * 渲染资产订单交易（ask/bid placement，对标 NRS.formatAssetOrder）。
 *
 * 拉取资产信息和订单成交记录，展示：
 *   - 资产名称 / 数量 / 单价 / 总价
 *   - 成交记录表格（trades）
 *   - 已成交数量 / 已成交总额
 *
 * @param transaction 交易对象
 * @param subtype 2=ask, 3=bid
 * @returns 渲染结果
 */
async function renderAssetOrder(
  transaction: NrcsTransaction,
  subtype: number,
): Promise<RenderResult> {
  const attachment = transaction.attachment || {}
  const senderLink = getAccountLink(transaction, 'sender')

  const rows: InfoRow[] = [
    {
      label: 'Type',
      value: subtype === 2 ? 'ask_order_placement' : 'bid_order_placement',
    },
    { label: 'Asset', value: getAssetLink(attachment.asset ?? '0') },
    {
      label: 'Quantity',
      value: { quantity: attachment.quantityQNT ?? '0', decimals: 0 },
    },
    { label: 'Sender', value: senderLink },
  ]

  // 拉取资产精度
  let decimals = 0
  let assetName: string | undefined
  try {
    const asset = await nrcsApi.getAsset(attachment.asset ?? '')
    if (asset) {
      decimals = asset.decimals
      assetName = asset.name
      // 更新 Asset 行显示名
      const assetRow = rows.find((r) => r.label === 'Asset')
      if (assetRow) {
        assetRow.value = getAssetLink(attachment.asset ?? '0', asset.name)
      }
      // 更新 Quantity 行精度
      const qtyRow = rows.find((r) => r.label === 'Quantity')
      if (qtyRow && typeof qtyRow.value === 'object' && 'quantity' in qtyRow.value) {
        qtyRow.value = { quantity: attachment.quantityQNT ?? '0', decimals }
      }
      // 添加单价和总价（对标 :1402-1404）
      const priceFormatted = formatOrderPricePerWholeQNT(attachment.priceNQT ?? '0', decimals)
      rows.splice(3, 0, { label: 'Price', value: `${priceFormatted} NRC` })
      const totalNQT = calculateOrderTotalNQT(attachment.quantityQNT ?? '0', attachment.priceNQT ?? '0')
      rows.splice(4, 0, { label: 'Total', value: `${formatAmount(totalNQT)} NRC` })
    }
  } catch {
    // 忽略
  }

  // 拉取订单成交记录（对标 :1414-1443）
  try {
    const isAsk = subtype === 2
    const tradesResp = await nrcsApi.getOrderTrades(
      isAsk ? transaction.transaction : undefined,
      isAsk ? undefined : transaction.transaction,
    )
    if (tradesResp.trades && tradesResp.trades.length > 0) {
      // 简化：展示成交数量和总额（完整表格由组件层渲染）
      let totalQty = BigInt(0)
      let totalNQTTrade = BigInt(0)
      for (const trade of tradesResp.trades) {
        totalQty += BigInt(trade.quantityQNT)
        totalNQTTrade += BigInt(trade.quantityQNT) * BigInt(trade.priceNQT)
      }
      rows.push({
        label: 'Quantity Traded',
        value: { quantity: totalQty.toString(), decimals },
      })
      rows.push({
        label: 'Total Traded',
        value: `${formatAmount(totalNQTTrade.toString())} NRC`,
      })
    } else {
      rows.push({ label: 'Trades', value: 'no_matching_trade' })
    }
  } catch {
    // 忽略
  }

  return { rows, incorrect: false, async: true }
}

// ============================================================================
// formatQuantity 辅助导出（供组件层使用）
// ============================================================================

export { formatQuantity }
