/******************************************************************************
 * NRCS 货币系统附件渲染器（type 5）—— 对标 nrs.modals.transaction.js:1021-1193。
 *
 * subtype 列表：
 *   - 0: 货币发行（issueCurrency）
 *   - 1: 储备增加（currencyReserveIncrease）
 *   - 2: 储备提取（currencyReserveClaim）
 *   - 3: 货币转移（transferCurrency）
 *   - 4: 发布兑换报价（publishExchangeOffer）— 需要 getOffer + getExchangesByOffer
 *   - 5: 货币买入（currencyBuy）— 需要 getExchangesByExchangeRequest
 *   - 6: 货币卖出（currencySell）— 需要 getExchangesByExchangeRequest
 *   - 7: 铸造（currencyMint）
 *   - 8: 删除货币（deleteCurrency）
 ******************************************************************************/
import { nrcsApi } from '@/api/modules/nrcs.api'
import type { NrcsTransaction } from '@/api/modules/nrcs.api'
import type { InfoRow } from './transaction-info-table'
import type { RenderOptions, RenderResult } from './transaction-attachment-renderer'
import {
  getTransactionLink,
  getAccountLink,
  getCurrencyLink,
} from './transaction-links'
import {
  formatAmount,
  formatQuantity,
  formatOrderPricePerWholeQNT,
  calculateOrderTotalNQT,
} from './format'

// ============================================================================
// renderMonetaryAttachment —— 主入口
// ============================================================================

/**
 * 渲染货币系统交易附件（对标 nrs.modals.transaction.js:1021-1193）。
 *
 * @param transaction 交易对象
 * @param subtype 子类型
 * @param options 渲染选项
 * @returns 渲染结果
 */
export async function renderMonetaryAttachment(
  transaction: NrcsTransaction,
  subtype: number,
  _options: RenderOptions,
): Promise<RenderResult> {
  const attachment = transaction.attachment || {}
  const senderLink = getAccountLink(transaction, 'sender')
  const recipientLink = getAccountLink(transaction, 'recipient')

  switch (subtype) {
    // subtype 0: 货币发行
    case 0: {
      const rows: InfoRow[] = [
        { label: 'Type', value: 'currency_issuance' },
        { label: 'Name', value: attachment.name ?? '' },
        { label: 'Code', value: attachment.code ?? '' },
        { label: 'Description', value: attachment.description ?? '' },
        { label: 'Currency Type', value: String(attachment.type ?? 0) },
        { label: 'Initial Supply', value: String(attachment.initialSupply ?? 0) },
        { label: 'Reserve Supply', value: String(attachment.reserveSupply ?? 0) },
        { label: 'Max Supply', value: String(attachment.maxSupply ?? 0) },
        { label: 'Issuance Height', value: String(attachment.issuanceHeight ?? 0) },
        { label: 'Min Reserve Per Unit NQT', value: attachment.minReservePerUnitNQT ?? '0' },
        { label: 'Min Difficulty', value: String(attachment.minDifficulty ?? 0) },
        { label: 'Max Difficulty', value: String(attachment.maxDifficulty ?? 0) },
        { label: 'Ruleset', value: String(attachment.ruleset ?? 0) },
        { label: 'Algorithm', value: String(attachment.algorithm ?? 0) },
        { label: 'Decimals', value: String(attachment.decimals ?? 0) },
        { label: 'Sender', value: senderLink },
      ]
      return { rows, incorrect: false, async: false }
    }

    // subtype 1: 储备增加
    case 1: {
      const rows: InfoRow[] = [
        { label: 'Type', value: 'currency_reserve_increase' },
        { label: 'Currency', value: getCurrencyLink(attachment.currency ?? '0') },
        { label: 'Amount Per Unit NQT', value: attachment.amountPerUnitNQT ?? '0' },
        { label: 'Sender', value: senderLink },
      ]

      await enrichCurrencyRow(rows, attachment.currency)
      return { rows, incorrect: false, async: true }
    }

    // subtype 2: 储备提取
    case 2: {
      const rows: InfoRow[] = [
        { label: 'Type', value: 'currency_reserve_claim' },
        { label: 'Currency', value: getCurrencyLink(attachment.currency ?? '0') },
        { label: 'Units', value: String(attachment.units ?? 0) },
        { label: 'Sender', value: senderLink },
      ]

      await enrichCurrencyRow(rows, attachment.currency)
      return { rows, incorrect: false, async: true }
    }

    // subtype 3: 货币转移
    case 3: {
      const rows: InfoRow[] = [
        { label: 'Type', value: 'currency_transfer' },
        { label: 'Currency', value: getCurrencyLink(attachment.currency ?? '0') },
        { label: 'Units', value: String(attachment.units ?? 0) },
        { label: 'Recipient', value: recipientLink },
        { label: 'Sender', value: senderLink },
      ]

      await enrichCurrencyRow(rows, attachment.currency)
      return { rows, incorrect: false, async: true }
    }

    // subtype 4: 发布兑换报价
    case 4: {
      return renderPublishExchangeOffer(transaction)
    }

    // subtype 5/6: 货币买入/卖出
    case 5:
    case 6: {
      return renderCurrencyBuySell(transaction, subtype)
    }

    // subtype 7: 铸造
    case 7: {
      const rows: InfoRow[] = [
        { label: 'Type', value: 'currency_mint' },
        { label: 'Currency', value: getCurrencyLink(attachment.currency ?? '0') },
        { label: 'Nonce', value: String(attachment.nonce ?? 0) },
        { label: 'Units', value: String(attachment.units ?? 0) },
        { label: 'Counter', value: String(attachment.counter ?? 0) },
        { label: 'Sender', value: senderLink },
      ]

      await enrichCurrencyRow(rows, attachment.currency)
      return { rows, incorrect: false, async: true }
    }

    // subtype 8: 删除货币
    case 8: {
      const rows: InfoRow[] = [
        { label: 'Type', value: 'delete_currency' },
        { label: 'Currency', value: getCurrencyLink(attachment.currency ?? '0') },
        { label: 'Sender', value: senderLink },
      ]

      await enrichCurrencyRow(rows, attachment.currency)
      return { rows, incorrect: false, async: true }
    }

    default:
      return { rows: [], incorrect: true, async: false }
  }
}

// ============================================================================
// renderPublishExchangeOffer —— 对标 NRS.formatCurrencyOffer :1498-1561
// ============================================================================

/**
 * 渲染发布兑换报价交易（对标 NRS.formatCurrencyOffer）。
 *
 * 拉取 getOffer 获取买/卖报价余额，拉取 getExchangesByOffer 获取成交记录。
 *
 * @param transaction 交易对象
 * @returns 渲染结果
 */
async function renderPublishExchangeOffer(
  transaction: NrcsTransaction,
): Promise<RenderResult> {
  const attachment = transaction.attachment || {}
  const senderLink = getAccountLink(transaction, 'sender')

  const rows: InfoRow[] = [
    { label: 'Type', value: 'exchange_offer' },
    { label: 'Currency', value: getCurrencyLink(attachment.currency ?? '0') },
    { label: 'Expiration Height', value: String(attachment.expirationHeight ?? 0) },
    { label: 'Sender', value: senderLink },
  ]

  let decimals = 0
  let code: string | undefined

  // 拉取货币精度和代码
  try {
    const currency = await nrcsApi.getCurrency(attachment.currency ?? '')
    if (currency) {
      decimals = currency.decimals
      code = currency.code
      const curRow = rows.find((r) => r.label === 'Currency')
      if (curRow) {
        curRow.value = getCurrencyLink(attachment.currency ?? '0', currency.code)
      }
    }
  } catch {
    // 忽略
  }

  // 买入/卖出报价信息（对标 :1513-1520）
  try {
    const offerResp = await nrcsApi.getOffer(transaction.transaction ?? '')
    if (offerResp.buyOffer && offerResp.sellOffer) {
      const rateUnitsStr = ` [ ${code ?? 'XXX'} / NRC ]`
      rows.push({
        label: 'Buy Supply',
        value: `${formatQuantity(offerResp.buyOffer.supply, decimals)} (initial: ${formatQuantity(attachment.initialBuySupply ?? '0', decimals)})`,
      })
      rows.push({
        label: 'Buy Limit',
        value: `${formatQuantity(offerResp.buyOffer.limit, decimals)} (initial: ${formatQuantity(attachment.totalBuyLimit ?? '0', decimals)})`,
      })
      rows.push({
        label: 'Buy Rate',
        value: formatOrderPricePerWholeQNT(attachment.buyRateNQT ?? '0', decimals) + rateUnitsStr,
      })
      rows.push({
        label: 'Sell Supply',
        value: `${formatQuantity(offerResp.sellOffer.supply, decimals)} (initial: ${formatQuantity(attachment.initialSellSupply ?? '0', decimals)})`,
      })
      rows.push({
        label: 'Sell Limit',
        value: `${formatQuantity(offerResp.sellOffer.limit, decimals)} (initial: ${formatQuantity(attachment.totalSellLimit ?? '0', decimals)})`,
      })
      rows.push({
        label: 'Sell Rate',
        value: formatOrderPricePerWholeQNT(attachment.sellRateNQT ?? '0', decimals) + rateUnitsStr,
      })
    }
  } catch {
    // 忽略
  }

  // 成交记录（对标 :1522-1559）
  try {
    const exchangesResp = await nrcsApi.getExchangesByOffer(transaction.transaction ?? '')
    if (exchangesResp.exchanges && exchangesResp.exchanges.length > 0) {
      let exchangedUnits = BigInt(0)
      let exchangedTotal = BigInt(0)
      for (const ex of exchangesResp.exchanges) {
        exchangedUnits += BigInt(ex.units)
        exchangedTotal += BigInt(ex.units) * BigInt(ex.rateNQT)
      }
      rows.push({
        label: 'Units Exchanged',
        value: { quantity: exchangedUnits.toString(), decimals },
      })
      rows.push({
        label: 'Total Exchanged',
        value: `${formatAmount(exchangedTotal.toString())} [NRC]`,
      })
    } else {
      rows.push({ label: 'Exchanges', value: 'no_matching_exchange_request' })
    }
  } catch {
    // 忽略
  }

  return { rows, incorrect: false, async: true }
}

// ============================================================================
// renderCurrencyBuySell —— 对标 NRS.formatCurrencyExchange :1454-1496
// ============================================================================

/**
 * 渲染货币买入/卖出交易（对标 NRS.formatCurrencyExchange）。
 *
 * 拉取 getExchangesByExchangeRequest 获取成交记录。
 *
 * @param transaction 交易对象
 * @param subtype 5=buy, 6=sell
 * @returns 渲染结果
 */
async function renderCurrencyBuySell(
  transaction: NrcsTransaction,
  subtype: number,
): Promise<RenderResult> {
  const attachment = transaction.attachment || {}
  const senderLink = getAccountLink(transaction, 'sender')

  const rows: InfoRow[] = [
    {
      label: 'Type',
      value: subtype === 5 ? 'buy_currency' : 'sell_currency',
    },
    { label: 'Currency', value: getCurrencyLink(attachment.currency ?? '0') },
    { label: 'Sender', value: senderLink },
  ]

  let decimals = 0
  let code: string | undefined

  // 拉取货币精度和代码
  try {
    const currency = await nrcsApi.getCurrency(attachment.currency ?? '')
    if (currency) {
      decimals = currency.decimals
      code = currency.code
      const curRow = rows.find((r) => r.label === 'Currency')
      if (curRow) {
        curRow.value = getCurrencyLink(attachment.currency ?? '0', currency.code)
      }
      // 添加单位和汇率（对标 :1458-1461）
      const rateUnitsStr = ` [ ${code} / NRC ]`
      rows.splice(2, 0, {
        label: 'Units',
        value: { quantity: String(attachment.units ?? '0'), decimals },
      })
      rows.splice(3, 0, {
        label: 'Rate',
        value: formatOrderPricePerWholeQNT(attachment.rateNQT ?? '0', decimals) + rateUnitsStr,
      })
    }
  } catch {
    // 忽略
  }

  // 成交记录（对标 :1463-1494）
  try {
    const exchangesResp = await nrcsApi.getExchangesByExchangeRequest(transaction.transaction ?? '')
    if (exchangesResp.exchanges && exchangesResp.exchanges.length > 0) {
      let exchangedUnits = BigInt(0)
      let exchangedTotal = BigInt(0)
      for (const ex of exchangesResp.exchanges) {
        exchangedUnits += BigInt(ex.units)
        exchangedTotal += BigInt(ex.units) * BigInt(ex.rateNQT)
      }
      rows.push({
        label: 'Units Exchanged',
        value: { quantity: exchangedUnits.toString(), decimals },
      })
      rows.push({
        label: 'Total Exchanged',
        value: `${formatAmount(exchangedTotal.toString())} [NRC]`,
      })
    } else {
      rows.push({ label: 'Exchanges', value: 'no_matching_exchange_offer' })
    }
  } catch {
    // 忽略
  }

  return { rows, incorrect: false, async: true }
}

// ============================================================================
// 辅助函数
// ============================================================================

/**
 * 富化货币行：拉取货币信息，更新 Currency 行显示代码。
 *
 * @param rows 信息表行数组
 * @param currencyId 货币 ID
 */
async function enrichCurrencyRow(
  rows: InfoRow[],
  currencyId: string | undefined,
): Promise<void> {
  if (!currencyId) return
  try {
    const currency = await nrcsApi.getCurrency(currencyId)
    if (currency) {
      const curRow = rows.find((r) => r.label === 'Currency')
      if (curRow) {
        curRow.value = getCurrencyLink(currencyId, currency.code)
      }
    }
  } catch {
    // 货币可能已删除（对标 NRS.getUnknownCurrencyData :1563-1572）
    const curRow = rows.find((r) => r.label === 'Currency')
    if (curRow) {
      curRow.value = `${currencyId} (Currency Deleted or not Issued)`
    }
  }
}
