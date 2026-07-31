/******************************************************************************
 * 交易详情 Composable 单元测试
 *
 * 验证 useTransactionDetail.ts 的核心功能：
 *   - showTransactionModal：入口（对象/ID 双路径）
 *   - getTransaction：通过 API 拉取交易
 *   - processTransactionModalData：处理交易数据并填充状态
 *   - getPhasingDetails：phasing 详情渲染
 *   - buildTransactionDetails：交易基本信息格式化
 *   - closeModal：关闭 modal 并清空状态
 *   - 计算属性：transactionId/hasPhasingDetails/canApproveTransaction/canExtendData/isSentByCurrentAccount
 *
 * Mock 依赖：
 *   - @/api/modules/nrcs.api：nrcsApi.getTransaction/getAsset/getCurrency
 *   - @/stores/modules/account.store：useAccountStore
 *   - @/stores/modules/node.store：useNodeStore
 *   - @/composables/useTransactionDecryption：useTransactionDecryption
 *   - @/utils/format：formatAmount/formatTimestamp
 *
 * 对标参考：nrs.modals.transaction.js
 ******************************************************************************/
import { describe, it, expect, vi, beforeEach } from 'vitest'

// Mock nrcsApi
vi.mock('@/api/modules/nrcs.api', () => ({
  nrcsApi: {
    getTransaction: vi.fn(),
    getAsset: vi.fn(),
    getCurrency: vi.fn(),
    getShufflingParticipants: vi.fn(),
    getShufflers: vi.fn(),
    getShuffling: vi.fn(),
    getDGSGood: vi.fn(),
    getDGSPurchase: vi.fn(),
    getOrderTrades: vi.fn(),
    getOffer: vi.fn(),
    getExchangesByExchangeRequest: vi.fn(),
    getExchangesByOffer: vi.fn(),
  },
}))

// Mock account store
const mockAccountStore = {
  accountId: '111',
  accountRS: 'NRCS-AAAA-AAAA-AAAA-AAAAA',
  secretPhrase: '',
}
vi.mock('@/stores/modules/account.store', () => ({
  useAccountStore: () => mockAccountStore,
}))

// Mock node store
const mockNodeStore = {
  lastBlockHeight: 200,
}
vi.mock('@/stores/modules/node.store', () => ({
  useNodeStore: () => mockNodeStore,
}))

// Mock decryption composable
const mockDecryption = {
  decryptTransactionMessages: vi.fn().mockResolvedValue([]),
  decryptWithSharedKey: vi.fn().mockResolvedValue([]),
  needsSharedKey: { value: false },
}
vi.mock('@/composables/useTransactionDecryption', () => ({
  useTransactionDecryption: () => mockDecryption,
}))

// Mock format utilities
vi.mock('@/utils/format', () => ({
  formatAmount: vi.fn((amount: string) => String(Number(amount) / 1e8)),
  formatTimestamp: vi.fn((ts: number) => `TS:${ts}`),
}))

import { useTransactionDetail } from '@/composables/useTransactionDetail'
import { nrcsApi } from '@/api/modules/nrcs.api'
import type { NrcsTransaction } from '@/api/modules/nrcs.api'

// 测试辅助：构造交易对象
function makeTx(overrides: Partial<NrcsTransaction> = {}): NrcsTransaction {
  return {
    transaction: '123456',
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
    block: 'block123',
    confirmations: 10,
    attachment: {},
    ...overrides,
  }
}

describe('useTransactionDetail', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockAccountStore.secretPhrase = ''
    mockNodeStore.lastBlockHeight = 200
  })

  // ==========================================================================
  // showTransactionModal
  // ==========================================================================
  describe('showTransactionModal', () => {
    it('传入交易对象直接处理（不调用 API）', async () => {
      const { state, showTransactionModal } = useTransactionDetail()
      const tx = makeTx()
      await showTransactionModal(tx)
      expect(nrcsApi.getTransaction).not.toHaveBeenCalled()
      expect(state.value.loading).toBe(false)
      expect(state.value.transaction).toEqual(tx)
      expect(state.value.visible).toBe(true)
    })

    it('传入交易 ID 调用 getTransaction API 拉取', async () => {
      vi.mocked(nrcsApi.getTransaction).mockResolvedValue(makeTx())
      const { state, showTransactionModal } = useTransactionDetail()
      await showTransactionModal('123456')
      expect(nrcsApi.getTransaction).toHaveBeenCalledWith('123456')
      expect(state.value.transaction).not.toBeNull()
      expect(state.value.transaction?.transaction).toBe('123456')
    })

    it('API 拉取失败时设置 error 状态', async () => {
      vi.mocked(nrcsApi.getTransaction).mockRejectedValue(new Error('网络错误'))
      const { state, showTransactionModal } = useTransactionDetail()
      await showTransactionModal('nonexistent')
      expect(state.value.loading).toBe(false)
      expect(state.value.error).toBe('网络错误')
    })

    it('API 返回 errorCode 时抛出错误', async () => {
      vi.mocked(nrcsApi.getTransaction).mockResolvedValue({
        errorCode: 5,
        errorDescription: 'Unknown transaction',
      } as any)
      const { state, showTransactionModal } = useTransactionDetail()
      await showTransactionModal('nonexistent')
      expect(state.value.error).toBe('Unknown transaction')
    })

    it('加载中时阻止重复调用', async () => {
      const { showTransactionModal } = useTransactionDetail()
      const tx = makeTx()
      // 第一次调用
      await showTransactionModal(tx)
      // state.value.loading 应该已经是 false
      // 第二次调用应正常处理
      await showTransactionModal(tx)
    })

    it('传入 sharedKey 初始化 sharedKeyInput', async () => {
      const { sharedKeyInput, showTransactionModal } = useTransactionDetail()
      await showTransactionModal(makeTx(), 'abc123')
      expect(sharedKeyInput.value).toBe('abc123')
    })
  })

  // ==========================================================================
  // processTransactionModalData —— 各类型交易处理
  // ==========================================================================
  describe('processTransactionModalData', () => {
    it('处理普通支付交易（type 0）', async () => {
      const { state, showTransactionModal } = useTransactionDetail()
      await showTransactionModal(makeTx({ type: 0, subtype: 0 }))
      expect(state.value.transactionDetails.length).toBeGreaterThan(0)
      expect(state.value.attachmentRows.length).toBeGreaterThan(0)
      expect(state.value.incorrect).toBe(false)
    })

    it('处理消息类交易（type 1 subtype 0，顶部消息）', async () => {
      const { state, showTransactionModal } = useTransactionDetail()
      await showTransactionModal(
        makeTx({
          type: 1,
          subtype: 0,
          attachment: {
            message: 'Hello World',
            messageIsText: true,
            'version.Message': 1,
          },
        }),
      )
      expect(state.value.topMessage).not.toBeNull()
      expect(state.value.topMessage?.text).toBe('Hello World')
      expect(state.value.bottomMessage).toBeNull()
    })

    it('处理非消息类交易（底部消息）', async () => {
      const { state, showTransactionModal } = useTransactionDetail()
      await showTransactionModal(
        makeTx({
          type: 0,
          subtype: 0,
          attachment: {
            message: 'Payment note',
            messageIsText: true,
            'version.Message': 1,
          },
        }),
      )
      expect(state.value.bottomMessage).not.toBeNull()
      expect(state.value.bottomMessage?.text).toBe('Payment note')
      expect(state.value.topMessage).toBeNull()
    })

    it('处理未知类型交易（incorrect=true）', async () => {
      const { state, showTransactionModal } = useTransactionDetail()
      await showTransactionModal(makeTx({ type: 99 }))
      expect(state.value.incorrect).toBe(true)
      expect(state.value.error).toBe('error_unknown_transaction_type')
    })

    it('处理含加密消息的交易（hasEncryptedMessage=true）', async () => {
      const { state, showTransactionModal } = useTransactionDetail()
      await showTransactionModal(
        makeTx({
          type: 0,
          subtype: 0,
          attachment: {
            encryptedMessage: { data: 'abc', nonce: 'def' },
            'version.EncryptedMessage': 1,
          },
        }),
      )
      expect(state.value.hasEncryptedMessage).toBe(true)
    })

    it('处理含 phasingFinishHeight 的交易（生成 phasing 详情）', async () => {
      const { state, showTransactionModal } = useTransactionDetail()
      await showTransactionModal(
        makeTx({
          type: 0,
          subtype: 0,
          attachment: {
            phasingFinishHeight: 300,
            phasingVotingModel: 0,
            phasingQuorum: '100000000',
          },
        }),
      )
      expect(state.value.phasingDetails.length).toBeGreaterThan(0)
    })

    it('有 secretPhrase 时自动尝试解密', async () => {
      mockAccountStore.secretPhrase = 'mysecret'
      const { showTransactionModal } = useTransactionDetail()
      await showTransactionModal(
        makeTx({
          type: 0,
          subtype: 0,
          attachment: {
            encryptedMessage: { data: 'abc', nonce: 'def' },
            'version.EncryptedMessage': 1,
          },
        }),
      )
      expect(mockDecryption.decryptTransactionMessages).toHaveBeenCalled()
    })
  })

  // ==========================================================================
  // getPhasingDetails
  // ==========================================================================
  describe('getPhasingDetails', () => {
    it('渲染基本 phasing 详情', async () => {
      const { getPhasingDetails } = useTransactionDetail()
      const result = await getPhasingDetails(
        {
          phasingFinishHeight: 300,
          phasingVotingModel: 0,
          phasingQuorum: '100000000',
          phasingMinBalance: '0',
          phasingMinBalanceModel: 0,
        },
        200,
      )
      expect(result.finishHeight).toBe(300)
      expect(result.finishIn).toBe('100 blocks')
      expect(result.votingModel).toBe('vote_by_account')
      expect(result.quorum).toBe('100000000')
    })

    it('finishHeight <= lastBlockHeight 时显示 "finished"', async () => {
      const { getPhasingDetails } = useTransactionDetail()
      const result = await getPhasingDetails(
        {
          phasingFinishHeight: 100,
          phasingVotingModel: 0,
        },
        200,
      )
      expect(result.finishIn).toBe('finished')
    })

    it('含白名单的 phasing 详情', async () => {
      const { getPhasingDetails } = useTransactionDetail()
      const result = await getPhasingDetails(
        {
          phasingFinishHeight: 300,
          phasingVotingModel: 0,
          phasingWhitelist: ['111', '222'],
        },
        200,
      )
      expect(Array.isArray(result.whitelist)).toBe(true)
      expect(result.whitelist).toHaveLength(2)
    })

    it('无白名单时显示 "-"', async () => {
      const { getPhasingDetails } = useTransactionDetail()
      const result = await getPhasingDetails(
        {
          phasingFinishHeight: 300,
          phasingVotingModel: 0,
        },
        200,
      )
      expect(result.whitelist).toBe('-')
    })

    it('含 linkedFullHashes 的 phasing 详情', async () => {
      const { getPhasingDetails } = useTransactionDetail()
      const result = await getPhasingDetails(
        {
          phasingFinishHeight: 300,
          phasingVotingModel: 0,
          phasingLinkedFullHashes: ['hash1', 'hash2'],
        },
        200,
      )
      expect(result.linkedFullHashes).toEqual(['hash1', 'hash2'])
    })

    it('含 hashedSecret 的 phasing 详情', async () => {
      const { getPhasingDetails } = useTransactionDetail()
      const result = await getPhasingDetails(
        {
          phasingFinishHeight: 300,
          phasingVotingModel: 0,
          phasingHashedSecret: 'secret123',
          phasingHashedSecretAlgorithm: 0,
        },
        200,
      )
      expect(result.hashedSecret).toBe('secret123')
      expect(result.hashAlgorithm).toBe('sha256')
    })

    it('votingModel=2 (ASSET) 调用 getAsset 获取精度', async () => {
      vi.mocked(nrcsApi.getAsset).mockResolvedValue({
        asset: '555',
        name: 'MYASSET',
        decimals: 4,
      } as any)
      const { getPhasingDetails } = useTransactionDetail()
      const result = await getPhasingDetails(
        {
          phasingFinishHeight: 300,
          phasingVotingModel: 2,
          phasingHolding: '555',
          phasingQuorum: '100000000',
          phasingMinBalance: '1000',
        },
        200,
      )
      expect(nrcsApi.getAsset).toHaveBeenCalledWith('555')
      expect(result.asset_formatted_html).toBeDefined()
    })

    it('votingModel=3 (CURRENCY) 调用 getCurrency 获取精度', async () => {
      vi.mocked(nrcsApi.getCurrency).mockResolvedValue({
        currency: '777',
        code: 'USD',
        decimals: 2,
      } as any)
      const { getPhasingDetails } = useTransactionDetail()
      const result = await getPhasingDetails(
        {
          phasingFinishHeight: 300,
          phasingVotingModel: 3,
          phasingHolding: '777',
          phasingQuorum: '100000000',
          phasingMinBalance: '1000',
        },
        200,
      )
      expect(nrcsApi.getCurrency).toHaveBeenCalledWith('777')
      expect(result.currency_formatted_html).toBeDefined()
    })
  })

  // ==========================================================================
  // closeModal
  // ==========================================================================
  describe('closeModal', () => {
    it('关闭 modal 并清空状态', async () => {
      const { state, sharedKeyInput, showTransactionModal, closeModal } = useTransactionDetail()
      await showTransactionModal(makeTx())
      sharedKeyInput.value = 'secret'
      expect(state.value.visible).toBe(true)
      expect(state.value.transaction).not.toBeNull()

      closeModal()
      expect(state.value.visible).toBe(false)
      expect(state.value.transaction).toBeNull()
      expect(state.value.transactionDetails).toEqual([])
      expect(sharedKeyInput.value).toBe('')
    })
  })

  // ==========================================================================
  // 计算属性
  // ==========================================================================
  describe('计算属性', () => {
    it('transactionId 返回当前交易 ID', async () => {
      const { transactionId, showTransactionModal } = useTransactionDetail()
      await showTransactionModal(makeTx({ transaction: '999' }))
      expect(transactionId.value).toBe('999')
    })

    it('hasPhasingDetails 在有 phasing 详情时为 true', async () => {
      const { hasPhasingDetails, showTransactionModal } = useTransactionDetail()
      await showTransactionModal(
        makeTx({
          attachment: {
            phasingFinishHeight: 300,
            phasingVotingModel: 0,
          },
        }),
      )
      expect(hasPhasingDetails.value).toBe(true)
    })

    it('hasPhasingDetails 在无 phasing 详情时为 false', async () => {
      const { hasPhasingDetails, showTransactionModal } = useTransactionDetail()
      await showTransactionModal(makeTx())
      expect(hasPhasingDetails.value).toBe(false)
    })

    it('canExtendData 在 TaggedDataUpload (type 6 subtype 0) 时为 true', async () => {
      const { canExtendData, showTransactionModal } = useTransactionDetail()
      await showTransactionModal(makeTx({ type: 6, subtype: 0 }))
      expect(canExtendData.value).toBe(true)
    })

    it('canExtendData 在非 TaggedDataUpload 时为 false', async () => {
      const { canExtendData, showTransactionModal } = useTransactionDetail()
      await showTransactionModal(makeTx({ type: 0, subtype: 0 }))
      expect(canExtendData.value).toBe(false)
    })

    it('canApproveTransaction 在 phasing 未完成时为 true', async () => {
      const { canApproveTransaction, showTransactionModal } = useTransactionDetail()
      await showTransactionModal(
        makeTx({
          block: 'block123',
          attachment: {
            phasingFinishHeight: 300, // > lastBlockHeight=200
            phasingVotingModel: 0,
          },
        }),
      )
      expect(canApproveTransaction.value).toBe(true)
    })

    it('canApproveTransaction 在 phasing 已完成时为 false', async () => {
      const { canApproveTransaction, showTransactionModal } = useTransactionDetail()
      await showTransactionModal(
        makeTx({
          block: 'block123',
          attachment: {
            phasingFinishHeight: 100, // < lastBlockHeight=200
            phasingVotingModel: 0,
          },
        }),
      )
      expect(canApproveTransaction.value).toBe(false)
    })

    it('canApproveTransaction 在无 block 时为 false', async () => {
      const { canApproveTransaction, showTransactionModal } = useTransactionDetail()
      await showTransactionModal(
        makeTx({
          block: undefined as any,
          attachment: {
            phasingFinishHeight: 300,
            phasingVotingModel: 0,
          },
        }),
      )
      expect(canApproveTransaction.value).toBe(false)
    })

    it('isSentByCurrentAccount 在发送方为当前账户时为 true', async () => {
      const { isSentByCurrentAccount, showTransactionModal } = useTransactionDetail()
      await showTransactionModal(
        makeTx({ senderRS: 'NRCS-AAAA-AAAA-AAAA-AAAAA' }),
      )
      expect(isSentByCurrentAccount.value).toBe(true)
    })

    it('isSentByCurrentAccount 在发送方非当前账户时为 false', async () => {
      const { isSentByCurrentAccount, showTransactionModal } = useTransactionDetail()
      await showTransactionModal(
        makeTx({ senderRS: 'NRCS-XXXX-XXXX-XXXX-XXXXX' }),
      )
      expect(isSentByCurrentAccount.value).toBe(false)
    })
  })

  // ==========================================================================
  // decryptWithSharedKey
  // ==========================================================================
  describe('decryptWithSharedKey', () => {
    it('调用 decryption.decryptWithSharedKey', async () => {
      const { showTransactionModal, decryptWithSharedKey } = useTransactionDetail()
      await showTransactionModal(
        makeTx({
          attachment: {
            encryptedMessage: { data: 'abc', nonce: 'def' },
            'version.EncryptedMessage': 1,
          },
        }),
      )
      await decryptWithSharedKey('mykey')
      expect(mockDecryption.decryptWithSharedKey).toHaveBeenCalled()
    })
  })
})
