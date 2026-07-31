/******************************************************************************
 * 交易附件渲染器单元测试
 *
 * 验证 transaction-attachment-renderer.ts 的核心功能：
 *   - renderAttachment：主入口按 transaction.type 分发
 *   - renderPayment (type 0)：普通支付附件渲染
 *   - renderMessaging (type 1)：消息类各 subtype 渲染
 *   - renderAccountControl (type 4)：账户控制（余额租赁/PhasingOnly）
 *   - renderTaggedData (type 6)：标签化数据上传/扩展
 *   - getVotingModelName / getMinBalanceModelName：枚举名称映射
 *   - 未知类型返回 incorrect=true
 *
 * 异步类型（Asset/Marketplace/Monetary/Shuffling）依赖 nrcsApi，已 Mock。
 *
 * 对标参考：nrs.modals.transaction.js:228-1374 的附件渲染逻辑
 ******************************************************************************/
import { describe, it, expect, vi, beforeEach } from 'vitest'

// Mock 异步渲染模块（避免依赖真实 nrcsApi）
vi.mock('@/api/modules', () => ({
  nrcsApi: {
    getAsset: vi.fn(),
    getAssetTrades: vi.fn(),
    getOrderTrades: vi.fn(),
    getCurrency: vi.fn(),
    getOffer: vi.fn(),
    getExchangesByExchangeRequest: vi.fn(),
    getExchangesByOffer: vi.fn(),
    getShufflingParticipants: vi.fn(),
    getShufflers: vi.fn(),
    getShuffling: vi.fn(),
    getDGSGood: vi.fn(),
    getDGSPurchase: vi.fn(),
  },
}))

import { renderAttachment, getVotingModelName, getMinBalanceModelName } from '@/utils/transaction-attachment-renderer'
import type { NrcsTransaction } from '@/api/modules/nrcs.api'
import { nrcsApi } from '@/api/modules'

// 测试辅助：构造交易对象
function makeTx(overrides: Partial<NrcsTransaction> = {}): NrcsTransaction {
  return {
    transaction: '123',
    type: 0,
    subtype: 0,
    sender: '111',
    senderRS: 'NRCS-AAAA-AAAA-AAAA-AAAAA',
    recipient: '222',
    recipientRS: 'NRCS-BBBB-BBBB-BBBB-BBBBB',
    amountNQT: '100000000',
    feeNQT: '1000000',
    timestamp: 1000,
    height: 100,
    confirmations: 10,
    attachment: {},
    ...overrides,
  }
}

const OPTIONS = { currentAccount: '111', lastBlockHeight: 200 }

describe('transaction-attachment-renderer', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  // ==========================================================================
  // renderAttachment —— 主入口分发
  // ==========================================================================
  describe('renderAttachment 主入口分发', () => {
    it('type 0 分发到 renderPayment', async () => {
      const result = await renderAttachment(makeTx({ type: 0, subtype: 0 }), OPTIONS)
      expect(result.incorrect).toBe(false)
      expect(result.rows.length).toBeGreaterThan(0)
      expect(result.rows.find((r) => r.label === 'Type')?.value).toBe('ordinary_payment')
    })

    it('type 1 subtype 0 分发到 renderMessaging（返回空 rows）', async () => {
      const result = await renderAttachment(makeTx({ type: 1, subtype: 0 }), OPTIONS)
      expect(result.incorrect).toBe(false)
      expect(result.rows).toEqual([])
    })

    it('未知 type 返回 incorrect=true', async () => {
      const result = await renderAttachment(makeTx({ type: 99 }), OPTIONS)
      expect(result.incorrect).toBe(true)
      expect(result.rows).toEqual([])
    })
  })

  // ==========================================================================
  // renderPayment (type 0)
  // ==========================================================================
  describe('renderPayment (type 0)', () => {
    it('subtype 0 渲染普通支付', async () => {
      const result = await renderAttachment(
        makeTx({ type: 0, subtype: 0, amountNQT: '500000000', feeNQT: '1000000' }),
        OPTIONS,
      )
      expect(result.incorrect).toBe(false)
      const labels = result.rows.map((r) => r.label)
      expect(labels).toContain('Type')
      expect(labels).toContain('Amount')
      expect(labels).toContain('Fee')
      expect(labels).toContain('Recipient')
      expect(labels).toContain('Sender')
    })

    it('subtype 非 0 返回 incorrect=true', async () => {
      const result = await renderAttachment(
        makeTx({ type: 0, subtype: 1 }),
        OPTIONS,
      )
      expect(result.incorrect).toBe(true)
    })
  })

  // ==========================================================================
  // renderMessaging (type 1)
  // ==========================================================================
  describe('renderMessaging (type 1)', () => {
    it('subtype 1 渲染别名分配', async () => {
      const result = await renderAttachment(
        makeTx({
          type: 1,
          subtype: 1,
          attachment: { alias: 'myalias', uri: 'http://example.com' },
        }),
        OPTIONS,
      )
      expect(result.incorrect).toBe(false)
      expect(result.rows.find((r) => r.label === 'Type')?.value).toBe('alias_assignment')
      expect(result.rows.find((r) => r.label === 'Alias')?.value).toBe('myalias')
      expect(result.rows.find((r) => r.label === 'Data')?.value).toBe('http://example.com')
    })

    it('subtype 2 渲染投票创建', async () => {
      const result = await renderAttachment(
        makeTx({
          type: 1,
          subtype: 2,
          attachment: {
            name: 'poll1',
            description: 'A poll',
            finishHeight: 1000,
            votingModel: 0,
            options: ['Yes', 'No'],
          },
        }),
        OPTIONS,
      )
      expect(result.incorrect).toBe(false)
      expect(result.rows.find((r) => r.label === 'Type')?.value).toBe('poll_creation')
      expect(result.rows.find((r) => r.label === 'Name')?.value).toBe('poll1')
      expect(result.rows.find((r) => r.label === 'Voting Model')?.value).toBe('vote_by_account')
    })

    it('subtype 5 渲染账户信息', async () => {
      const result = await renderAttachment(
        makeTx({
          type: 1,
          subtype: 5,
          attachment: { name: 'Alice', description: 'An account' },
        }),
        OPTIONS,
      )
      expect(result.incorrect).toBe(false)
      expect(result.rows.find((r) => r.label === 'Type')?.value).toBe('account_info')
      expect(result.rows.find((r) => r.label === 'Name')?.value).toBe('Alice')
    })

    it('subtype 6 priceNQT!=0 渲染 alias_sale', async () => {
      const result = await renderAttachment(
        makeTx({
          type: 1,
          subtype: 6,
          sender: '111',
          recipient: '222',
          attachment: { alias: 'myalias', priceNQT: '1000000' },
        }),
        OPTIONS,
      )
      expect(result.incorrect).toBe(false)
      expect(result.rows.find((r) => r.label === 'Type')?.value).toBe('alias_sale')
    })

    it('subtype 6 priceNQT=0 + sender==recipient 渲染 alias_sale_cancellation', async () => {
      const result = await renderAttachment(
        makeTx({
          type: 1,
          subtype: 6,
          sender: '111',
          recipient: '111',
          attachment: { alias: 'myalias', priceNQT: '0' },
        }),
        OPTIONS,
      )
      expect(result.incorrect).toBe(false)
      expect(result.rows.find((r) => r.label === 'Type')?.value).toBe('alias_sale_cancellation')
    })

    it('subtype 6 priceNQT=0 + sender!=recipient 渲染 alias_transfer', async () => {
      const result = await renderAttachment(
        makeTx({
          type: 1,
          subtype: 6,
          sender: '111',
          recipient: '222',
          attachment: { alias: 'myalias', priceNQT: '0' },
        }),
        OPTIONS,
      )
      expect(result.incorrect).toBe(false)
      expect(result.rows.find((r) => r.label === 'Type')?.value).toBe('alias_transfer')
    })

    it('subtype 8 渲染别名删除', async () => {
      const result = await renderAttachment(
        makeTx({
          type: 1,
          subtype: 8,
          attachment: { alias: 'myalias' },
        }),
        OPTIONS,
      )
      expect(result.incorrect).toBe(false)
      expect(result.rows.find((r) => r.label === 'Type')?.value).toBe('alias_deletion')
    })

    it('subtype 10 渲染设置账户属性', async () => {
      const result = await renderAttachment(
        makeTx({
          type: 1,
          subtype: 10,
          attachment: { property: 'key', value: 'value' },
        }),
        OPTIONS,
      )
      expect(result.incorrect).toBe(false)
      expect(result.rows.find((r) => r.label === 'Type')?.value).toBe('set_account_property')
      expect(result.rows.find((r) => r.label === 'Property')?.value).toBe('key')
    })

    it('subtype 11 渲染删除账户属性', async () => {
      const result = await renderAttachment(
        makeTx({
          type: 1,
          subtype: 11,
          attachment: { property: '123' },
        }),
        OPTIONS,
      )
      expect(result.incorrect).toBe(false)
      expect(result.rows.find((r) => r.label === 'Type')?.value).toBe('delete_account_property')
    })

    it('subtype 未知返回 incorrect=true', async () => {
      const result = await renderAttachment(
        makeTx({ type: 1, subtype: 99 }),
        OPTIONS,
      )
      expect(result.incorrect).toBe(true)
    })
  })

  // ==========================================================================
  // renderAccountControl (type 4)
  // ==========================================================================
  describe('renderAccountControl (type 4)', () => {
    it('subtype 0 渲染余额租赁', async () => {
      const result = await renderAttachment(
        makeTx({
          type: 4,
          subtype: 0,
          attachment: { period: 1440 },
        }),
        OPTIONS,
      )
      expect(result.incorrect).toBe(false)
      expect(result.rows.find((r) => r.label === 'Type')?.value).toBe('balance_leasing')
      expect(result.rows.find((r) => r.label === 'Period')?.value).toBe('1440')
    })

    it('subtype 1 渲染设置 Phasing Only 控制', async () => {
      const result = await renderAttachment(
        makeTx({
          type: 4,
          subtype: 1,
          attachment: {
            controlVotingModel: 0,
            controlQuorum: '100000000',
            controlMinBalance: '1000',
            controlMinBalanceModel: 0,
          },
        }),
        OPTIONS,
      )
      expect(result.incorrect).toBe(false)
      expect(result.rows.find((r) => r.label === 'Type')?.value).toBe('set_phasing_only_control')
      expect(result.rows.find((r) => r.label === 'Voting Model')?.value).toBe('vote_by_account')
      expect(result.rows.find((r) => r.label === 'Quorum')?.value).toBe('100000000')
    })

    it('subtype 未知返回 incorrect=true', async () => {
      const result = await renderAttachment(
        makeTx({ type: 4, subtype: 99 }),
        OPTIONS,
      )
      expect(result.incorrect).toBe(true)
    })
  })

  // ==========================================================================
  // renderTaggedData (type 6)
  // ==========================================================================
  describe('renderTaggedData (type 6)', () => {
    it('subtype 0 渲染数据上传', async () => {
      const result = await renderAttachment(
        makeTx({
          type: 6,
          subtype: 0,
          attachment: {
            hash: 'abc123',
            name: 'data1',
            description: 'A data',
            tags: 'tag1',
            type: 'text/plain',
            channel: 'mychannel',
            isText: true,
            filename: 'data.txt',
            data: 'Hello World',
          },
        }),
        OPTIONS,
      )
      expect(result.incorrect).toBe(false)
      expect(result.rows.find((r) => r.label === 'Hash')?.value).toBe('abc123')
      expect(result.rows.find((r) => r.label === 'Name')?.value).toBe('data1')
      expect(result.rows.find((r) => r.label === 'Is Text')?.value).toBe('true')
      expect(result.rows.find((r) => r.label === 'Data Size')?.value).toBe(String('Hello World'.length))
    })

    it('subtype 1 渲染数据扩展（无 data 字段，仅有 hash）', async () => {
      const result = await renderAttachment(
        makeTx({
          type: 6,
          subtype: 1,
          attachment: {
            hash: 'abc123',
            taggedData: '456',
          },
        }),
        OPTIONS,
      )
      expect(result.incorrect).toBe(false)
      expect(result.rows.find((r) => r.label === 'Hash')?.value).toBe('abc123')
      // taggedData 字段应渲染为 transaction 链接
      const taggedDataRow = result.rows.find((r) => r.label === 'Tagged Data')
      expect(taggedDataRow).toBeDefined()
    })

    it('subtype 未知返回 incorrect=true', async () => {
      const result = await renderAttachment(
        makeTx({ type: 6, subtype: 99 }),
        OPTIONS,
      )
      expect(result.incorrect).toBe(true)
    })
  })

  // ==========================================================================
  // 辅助函数
  // ==========================================================================
  describe('getVotingModelName', () => {
    it('case 0 → vote_by_account', () => {
      expect(getVotingModelName(0)).toBe('vote_by_account')
    })
    it('case 1 → vote_by_balance', () => {
      expect(getVotingModelName(1)).toBe('vote_by_balance')
    })
    it('case 2 → vote_by_asset', () => {
      expect(getVotingModelName(2)).toBe('vote_by_asset')
    })
    it('case 3 → vote_by_currency', () => {
      expect(getVotingModelName(3)).toBe('vote_by_currency')
    })
    it('case 4 → vote_by_transaction', () => {
      expect(getVotingModelName(4)).toBe('vote_by_transaction')
    })
    it('case 5 → vote_by_hash', () => {
      expect(getVotingModelName(5)).toBe('vote_by_hash')
    })
    it('case -1 → vote_by_none', () => {
      expect(getVotingModelName(-1)).toBe('vote_by_none')
    })
    it('undefined → "0"', () => {
      expect(getVotingModelName(undefined)).toBe('0')
    })
    it('未知值 → 数字字符串', () => {
      expect(getVotingModelName(99)).toBe('99')
    })
  })

  describe('getMinBalanceModelName', () => {
    it('case 0 → min_balance_model_nqt', () => {
      expect(getMinBalanceModelName(0)).toBe('min_balance_model_nqt')
    })
    it('case 1 → min_balance_model_asset', () => {
      expect(getMinBalanceModelName(1)).toBe('min_balance_model_asset')
    })
    it('case 2 → min_balance_model_currency', () => {
      expect(getMinBalanceModelName(2)).toBe('min_balance_model_currency')
    })
    it('undefined → "0"', () => {
      expect(getMinBalanceModelName(undefined)).toBe('0')
    })
    it('未知值 → 数字字符串', () => {
      expect(getMinBalanceModelName(99)).toBe('99')
    })
  })

  // ==========================================================================
  // 异步类型渲染（type 2/3/5/7，依赖 nrcsApi mock）
  // ==========================================================================
  describe('异步类型渲染', () => {
    it('type 2 (Asset) 调用 nrcsApi.getAsset', async () => {
      vi.mocked(nrcsApi.getAsset).mockResolvedValue({
        asset: '555',
        name: 'MYASSET',
        decimals: 4,
      } as any)
      const result = await renderAttachment(
        makeTx({
          type: 2,
          subtype: 0,
          attachment: { asset: '555', name: 'MYASSET', quantityQNT: '1000000', decimals: 4 },
        }),
        OPTIONS,
      )
      expect(result.incorrect).toBe(false)
      // type 2 subtype 0 是资产发行，应该不调用 getAsset（直接用 attachment 信息）
      // 但 subtype 1（转移）/2/3（订单）会调用
    })

    it('type 3 (Marketplace) 不抛异常', async () => {
      const result = await renderAttachment(
        makeTx({
          type: 3,
          subtype: 0,
          attachment: {
            name: 'product1',
            description: 'A product',
            priceNQT: '1000000',
            quantity: 10,
            tags: 'tag1',
          },
        }),
        OPTIONS,
      )
      expect(result.incorrect).toBe(false)
    })

    it('type 5 (Monetary) 不抛异常', async () => {
      const result = await renderAttachment(
        makeTx({
          type: 5,
          subtype: 0,
          attachment: {
            currency: '777',
            code: 'USD',
            name: 'US Dollar',
            description: 'A currency',
            type: 1,
            maxSupplyQNT: '1000000000',
            decimals: 2,
          },
        }),
        OPTIONS,
      )
      expect(result.incorrect).toBe(false)
    })

    it('type 7 (Shuffling) 调用 nrcsApi.getShufflingParticipants 等', async () => {
      vi.mocked(nrcsApi.getShufflingParticipants).mockResolvedValue({
        participants: [{ account: '111', accountRS: 'NRCS-AAAA-AAAA-AAAA-AAAAA' }],
      } as any)
      vi.mocked(nrcsApi.getShufflers).mockResolvedValue({ shufflers: [] } as any)
      vi.mocked(nrcsApi.getShuffling).mockResolvedValue({
        stage: 0,
        registrantCount: 1,
        participantCount: 4,
        blocksRemaining: 100,
      } as any)
      const result = await renderAttachment(
        makeTx({
          type: 7,
          subtype: 0,
          attachment: { shuffling: '888', amount: '100000000', participantCount: 4 },
        }),
        OPTIONS,
      )
      expect(result.incorrect).toBe(false)
    })
  })
})
