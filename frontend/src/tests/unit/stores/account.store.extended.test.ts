/******************************************************************************
 * NRCS 账户 Store 扩展功能单元测试（阶段 1.2）
 *
 * 验证 account.store.ts 新增的核心功能（对标 nrs.js）：
 *   - getAccountInfo：完整账户信息拉取 + leasing/control/asset 状态更新（nrs.js:1101-1325）
 *   - updateAccountLeasingStatus：出租状态计算（nrs.js:1394-1489）
 *   - updateAccountControlStatus：Phasing Only 控制检测（nrs.js:1491-1531）
 *   - checkAssetDifferences：资产余额变化对比（nrs.js:1533-1645）
 *   - compareAndUpdateAssetBalances：IndexedDB 资产余额存储与对比
 *   - onAssetDifference：回调注册
 *
 * Mock 依赖：
 *   - @/api/modules：nrcsApi.getAccount/getAccountPublicKey/getBlockchainStatus/getPhasingOnlyControl
 *   - @/utils/nrcs-storage：storageSelect/storageInsert/storageUpdate
 *   - @/utils/mnemonic：passphraseToAccount/checkPassphraseStrength
 ******************************************************************************/
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'

// Mock nrcsApi
vi.mock('@/api/modules', () => ({
  nrcsApi: {
    getAccount: vi.fn(),
    getAccountPublicKey: vi.fn(),
    getBlockchainStatus: vi.fn(),
    getPhasingOnlyControl: vi.fn(),
  },
}))

// Mock storage
vi.mock('@/utils/nrcs-storage', () => ({
  storageSelect: vi.fn(),
  storageInsert: vi.fn(),
  storageUpdate: vi.fn(),
}))

// Mock mnemonic
vi.mock('@/utils/mnemonic', () => ({
  passphraseToAccount: vi.fn(),
  checkPassphraseStrength: vi.fn(),
}))

import { useAccountStore } from '@/stores/modules/account.store'
import { nrcsApi } from '@/api/modules'
import { storageSelect, storageInsert, storageUpdate } from '@/utils/nrcs-storage'
import { passphraseToAccount } from '@/utils/mnemonic'
import type { NrcsAccount, NrcsAssetBalance, NrcsPhasingOnlyControl } from '@/api/modules'

// ============================================================================
// 测试数据构造
// ============================================================================

const TEST_ACCOUNT_RS = 'NRCS-SM2H-LPVM-ES9M-94C92'
const TEST_ACCOUNT_ID = '1234567890'

/** 构造完整账户信息 */
function buildAccountInfo(overrides: Partial<NrcsAccount> = {}): NrcsAccount {
  return {
    account: TEST_ACCOUNT_ID,
    accountRS: TEST_ACCOUNT_RS,
    balanceNQT: '100000000',
    unconfirmedBalanceNQT: '100000000',
    forgedBalanceNQT: '0',
    guaranteedBalanceNQT: '0',
    effectiveBalanceNRCS: 1,
    name: 'test',
    description: '',
    ...overrides,
  } as NrcsAccount
}

/** 构造 Phasing Only 控制信息 */
function buildPhasingOnlyControl(
  overrides: Partial<NrcsPhasingOnlyControl> = {},
): NrcsPhasingOnlyControl {
  return {
    votingModel: 0,
    quorum: '100000000',
    phasingWhitelisted: ['123'],
    minBalanceModel: 0,
    minDuration: 1,
    maxDuration: 100,
    maxFees: '1000000',
    ...overrides,
  } as NrcsPhasingOnlyControl
}

// ============================================================================
// 测试用例
// ============================================================================

describe('account.store: getAccountInfo', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('成功拉取完整账户信息并更新基础字段', async () => {
    const info = buildAccountInfo({
      balanceNQT: '200000000',
      name: 'myaccount',
      description: 'my desc',
    })
    vi.mocked(nrcsApi.getAccount).mockResolvedValue(info)

    const store = useAccountStore()
    await store.getAccountInfo(TEST_ACCOUNT_RS)

    expect(nrcsApi.getAccount).toHaveBeenCalledWith(TEST_ACCOUNT_RS, {
      includeLessors: true,
      includeAssets: true,
      includeCurrencies: true,
      includeEffectiveBalance: true,
    })
    expect(store.accountInfo).toEqual(info)
    expect(store.balanceNQT).toBe('200000000')
    expect(store.name).toBe('myaccount')
    expect(store.description).toBe('my desc')
  })

  it('账户不存在(code=5)时清零余额并清除 accountInfo', async () => {
    const err = new Error('Unknown account') as any
    err.code = 5
    vi.mocked(nrcsApi.getAccount).mockRejectedValue(err)

    const store = useAccountStore()
    // 先设置一些状态
    store.balanceNQT = '100000000'
    store.accountInfo = buildAccountInfo()

    await store.getAccountInfo(TEST_ACCOUNT_RS)

    expect(store.balanceNQT).toBe('0')
    expect(store.accountInfo).toBeNull()
    expect(store.name).toBe('')
  })

  it('其他错误时降级为 fetchAccountInfo（简版）', async () => {
    // 第一次调用（getAccountInfo 内）拒绝 → 触发 catch → 降级调 fetchAccountInfo
    // 第二次调用（fetchAccountInfo 内）成功返回
    // 注意：mockResolvedValueOnce 优先级高于 mockRejectedValue，故第一次用 mockRejectedValueOnce
    const err = new Error('network error') as any
    vi.mocked(nrcsApi.getAccount)
      .mockRejectedValueOnce(err)
      .mockResolvedValueOnce(buildAccountInfo({ balanceNQT: '500000000' }))

    const store = useAccountStore()
    await store.getAccountInfo(TEST_ACCOUNT_RS)

    // 降级后应通过 fetchAccountInfo 设置余额
    expect(store.balanceNQT).toBe('500000000')
  })

  it('链上 RS 与本地不一致时采用链上版本（对标 nrs.js:1116-1121）', async () => {
    // mockReset 清除前一个测试可能残留的 mockResolvedValueOnce 队列（clearAllMocks 不清除一次性队列）
    vi.mocked(nrcsApi.getAccount).mockReset()
    const info = buildAccountInfo({ accountRS: 'NRCS-DIFF-RENT-ADDR-XXXXX' })
    vi.mocked(nrcsApi.getAccount).mockResolvedValue(info)
    vi.mocked(nrcsApi.getPhasingOnlyControl).mockResolvedValue(buildPhasingOnlyControl())

    const store = useAccountStore()
    store.accountRS = TEST_ACCOUNT_RS

    await store.getAccountInfo(TEST_ACCOUNT_RS)

    expect(store.accountRS).toBe('NRCS-DIFF-RENT-ADDR-XXXXX')
  })
})

describe('account.store: updateAccountLeasingStatus', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('无 accountInfo 时返回空状态', () => {
    const store = useAccountStore()
    store.updateAccountLeasingStatus(1000)

    expect(store.leasingStatus.label).toBe('')
    expect(store.leasingStatus.statusMessage).toBe('')
  })

  it('lastBlockHeight >= currentLeasingHeightFrom → leased_out（对标 nrs.js:1406-1413）', () => {
    const store = useAccountStore()
    store.accountInfo = buildAccountInfo({
      currentLeasingHeightFrom: 500,
      currentLeasingHeightTo: 2000,
      currentLesseeRS: 'NRCS-LESSEE-ADDR-XXXXX',
    })

    store.updateAccountLeasingStatus(1000)

    expect(store.leasingStatus.label).toBe('leased_out')
    expect(store.leasingStatus.statusMessage).toContain('已出租')
    expect(store.leasingStatus.statusMessage).toContain('1000') // 剩余 blocks = 2000-1000
  })

  it('lastBlockHeight < currentLeasingHeightTo → leased_soon（对标 nrs.js:1414-1422）', () => {
    const store = useAccountStore()
    store.accountInfo = buildAccountInfo({
      currentLeasingHeightFrom: 1500,
      currentLeasingHeightTo: 3000,
      currentLesseeRS: 'NRCS-LESSEE-ADDR-XXXXX',
    })

    store.updateAccountLeasingStatus(1000)

    expect(store.leasingStatus.label).toBe('leased_soon')
    expect(store.leasingStatus.statusMessage).toContain('即将')
    expect(store.leasingStatus.statusMessage).toContain('500') // 1500-1000=500 块后
  })

  it('无出租信息(from/to 均为 undefined) → 未出租（对标 nrs.js:1423-1425 else 分支）', () => {
    // NRS 原始逻辑：lastBlockHeight >= undefined → false, lastBlockHeight < undefined → false → else 分支
    // 即账户未设置 currentLeasingHeightFrom/To（无活跃租约）时进入 not_leased_out
    const store = useAccountStore()
    store.accountInfo = buildAccountInfo({
      // 不设置 currentLeasingHeightFrom/To
    })

    store.updateAccountLeasingStatus(500)

    expect(store.leasingStatus.label).toBe('')
    expect(store.leasingStatus.statusMessage).toContain('未出租')
  })

  it('nextLeasingHeightFrom < MAX_INT_JAVA 时显示下一承租方信息（对标 nrs.js:1398-1404）', () => {
    const store = useAccountStore()
    store.accountInfo = buildAccountInfo({
      currentLeasingHeightFrom: 100,
      currentLeasingHeightTo: 200,
      nextLeasingHeightFrom: 300,
      nextLeasingHeightTo: 500,
    })

    store.updateAccountLeasingStatus(500)

    expect(store.leasingStatus.nextLesseeStatus).toContain('300')
    expect(store.leasingStatus.nextLesseeStatus).toContain('500')
  })

  it('lessors 存在时 lessorCount 正确（对标 nrs.js:1433-1444）', () => {
    const store = useAccountStore()
    store.accountInfo = buildAccountInfo({
      lessors: ['111', '222', '333'],
    })

    store.updateAccountLeasingStatus(1000)

    expect(store.leasingStatus.lessorCount).toBe(3)
  })
})

describe('account.store: updateAccountControlStatus', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('accountControls 包含 PHASING_ONLY → 调 getPhasingOnlyControl 并存储（对标 nrs.js:1497-1505）', async () => {
    const control = buildPhasingOnlyControl({ votingModel: 1 })
    vi.mocked(nrcsApi.getPhasingOnlyControl).mockResolvedValue(control)

    const store = useAccountStore()
    store.accountInfo = buildAccountInfo({
      accountControls: ['PHASING_ONLY'],
    })
    store.accountId = TEST_ACCOUNT_ID

    await store.updateAccountControlStatus()

    expect(nrcsApi.getPhasingOnlyControl).toHaveBeenCalledWith(TEST_ACCOUNT_ID)
    expect(store.phasingOnlyControl).toEqual(control)
    expect(store.hasPhasingOnlyControl).toBe(true)
    expect(store.accountInfo?.phasingOnly).toEqual(control)
  })

  it('accountControls 不包含 PHASING_ONLY → 清除 phasingOnlyControl（对标 nrs.js:1528-1530）', async () => {
    const store = useAccountStore()
    store.accountInfo = buildAccountInfo({
      accountControls: [],
    })
    // 先设置一些 phasingOnly 状态
    store.phasingOnlyControl = buildPhasingOnlyControl()
    store.hasPhasingOnlyControl = true

    await store.updateAccountControlStatus()

    expect(store.phasingOnlyControl).toBeNull()
    expect(store.hasPhasingOnlyControl).toBe(false)
  })

  it('getPhasingOnlyControl 返回 votingModel < 0 → 清除（对标 nrs.js:1501/1524-1526）', async () => {
    vi.mocked(nrcsApi.getPhasingOnlyControl).mockResolvedValue(
      buildPhasingOnlyControl({ votingModel: -1 }),
    )

    const store = useAccountStore()
    store.accountInfo = buildAccountInfo({
      accountControls: ['PHASING_ONLY'],
    })
    store.accountId = TEST_ACCOUNT_ID

    await store.updateAccountControlStatus()

    expect(store.phasingOnlyControl).toBeNull()
    expect(store.hasPhasingOnlyControl).toBe(false)
  })

  it('getPhasingOnlyControl 失败 → 清除并记录警告', async () => {
    vi.mocked(nrcsApi.getPhasingOnlyControl).mockRejectedValue(new Error('network'))

    const store = useAccountStore()
    store.accountInfo = buildAccountInfo({
      accountControls: ['PHASING_ONLY'],
    })
    store.accountId = TEST_ACCOUNT_ID

    await store.updateAccountControlStatus()

    expect(store.phasingOnlyControl).toBeNull()
    expect(store.hasPhasingOnlyControl).toBe(false)
  })

  it('无 accountInfo → 清除 phasingOnlyControl', async () => {
    const store = useAccountStore()
    store.accountInfo = null

    await store.updateAccountControlStatus()

    expect(store.phasingOnlyControl).toBeNull()
    expect(store.hasPhasingOnlyControl).toBe(false)
  })
})

describe('account.store: checkAssetDifferences', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('无变化时不触发回调', () => {
    const store = useAccountStore()
    const cb = vi.fn()
    store.onAssetDifference(cb)

    const balances: NrcsAssetBalance[] = [
      { asset: 'asset1', balanceQNT: '100' },
      { asset: 'asset2', balanceQNT: '200' },
    ]

    store.checkAssetDifferences(balances, balances)

    expect(cb).not.toHaveBeenCalled()
  })

  it('新增资产 → diff 为正值（对标 nrs.js:1572-1573）', () => {
    const store = useAccountStore()
    const cb = vi.fn()
    store.onAssetDifference(cb)

    const current: NrcsAssetBalance[] = [
      { asset: 'asset1', balanceQNT: '100' },
      { asset: 'asset2', balanceQNT: '200' },
    ]
    const previous: NrcsAssetBalance[] = [
      { asset: 'asset1', balanceQNT: '100' },
    ]

    store.checkAssetDifferences(current, previous)

    expect(cb).toHaveBeenCalledWith([
      { asset: 'asset2', difference: '200' },
    ])
  })

  it('资产被移除 → diff 为负值（对标 nrs.js:1561-1562）', () => {
    const store = useAccountStore()
    const cb = vi.fn()
    store.onAssetDifference(cb)

    const current: NrcsAssetBalance[] = [
      { asset: 'asset1', balanceQNT: '100' },
    ]
    const previous: NrcsAssetBalance[] = [
      { asset: 'asset1', balanceQNT: '100' },
      { asset: 'asset2', balanceQNT: '200' },
    ]

    store.checkAssetDifferences(current, previous)

    expect(cb).toHaveBeenCalledWith([
      { asset: 'asset2', difference: '-200' },
    ])
  })

  it('资产余额变化 → diff 为差值（对标 nrs.js:1563-1564）', () => {
    const store = useAccountStore()
    const cb = vi.fn()
    store.onAssetDifference(cb)

    const current: NrcsAssetBalance[] = [
      { asset: 'asset1', balanceQNT: '300' },
    ]
    const previous: NrcsAssetBalance[] = [
      { asset: 'asset1', balanceQNT: '100' },
    ]

    store.checkAssetDifferences(current, previous)

    expect(cb).toHaveBeenCalledWith([
      { asset: 'asset1', difference: '200' },
    ])
  })

  it('资产余额减少 → diff 为负差值', () => {
    const store = useAccountStore()
    const cb = vi.fn()
    store.onAssetDifference(cb)

    const current: NrcsAssetBalance[] = [
      { asset: 'asset1', balanceQNT: '50' },
    ]
    const previous: NrcsAssetBalance[] = [
      { asset: 'asset1', balanceQNT: '200' },
    ]

    store.checkAssetDifferences(current, previous)

    expect(cb).toHaveBeenCalledWith([
      { asset: 'asset1', difference: '-150' },
    ])
  })

  it('多个资产变化 → 全部包含在 diff 中', () => {
    const store = useAccountStore()
    const cb = vi.fn()
    store.onAssetDifference(cb)

    const current: NrcsAssetBalance[] = [
      { asset: 'asset1', balanceQNT: '150' }, // 变化
      { asset: 'asset2', balanceQNT: '100' }, // 新增
    ]
    const previous: NrcsAssetBalance[] = [
      { asset: 'asset1', balanceQNT: '100' },
      { asset: 'asset3', balanceQNT: '50' }, // 被移除
    ]

    store.checkAssetDifferences(current, previous)

    expect(cb).toHaveBeenCalledOnce()
    const diff = cb.mock.calls[0][0]
    expect(diff).toHaveLength(3)
    // asset1 变化 +50
    expect(diff.find((d: any) => d.asset === 'asset1').difference).toBe('50')
    // asset3 被移除 -50
    expect(diff.find((d: any) => d.asset === 'asset3').difference).toBe('-50')
    // asset2 新增 100
    expect(diff.find((d: any) => d.asset === 'asset2').difference).toBe('100')
  })

  it('大数减法正确处理（对标 nrs.js:1564 BigInteger.subtract）', () => {
    const store = useAccountStore()
    const cb = vi.fn()
    store.onAssetDifference(cb)

    const current: NrcsAssetBalance[] = [
      { asset: 'asset1', balanceQNT: '9007199254740992' }, // 超过 Number.MAX_SAFE_INTEGER
    ]
    const previous: NrcsAssetBalance[] = [
      { asset: 'asset1', balanceQNT: '9007199254740990' },
    ]

    store.checkAssetDifferences(current, previous)

    expect(cb).toHaveBeenCalledWith([
      { asset: 'asset1', difference: '2' },
    ])
  })
})

describe('account.store: compareAndUpdateAssetBalances', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('首次存储（IndexedDB 无记录）→ storageInsert（对标 nrs.js:1157-1162）', async () => {
    vi.mocked(storageSelect).mockResolvedValue([])
    vi.mocked(storageInsert).mockResolvedValue([])

    const store = useAccountStore()
    const balances: NrcsAssetBalance[] = [{ asset: 'asset1', balanceQNT: '100' }]

    // 直接调用内部方法（通过 getAccountInfo 间接触发）
    store.accountInfo = buildAccountInfo({ assetBalances: balances })
    vi.mocked(nrcsApi.getAccount).mockResolvedValue(store.accountInfo)
    vi.mocked(nrcsApi.getPhasingOnlyControl).mockResolvedValue(buildPhasingOnlyControl())

    await store.getAccountInfo(TEST_ACCOUNT_RS)

    expect(storageInsert).toHaveBeenCalledWith('data', 'id', {
      id: 'asset_balances',
      contents: JSON.stringify(balances),
    })
  })

  it('有变化时 → storageUpdate + checkAssetDifferences（对标 nrs.js:1142-1155）', async () => {
    const previousBalances: NrcsAssetBalance[] = [{ asset: 'asset1', balanceQNT: '100' }]
    const currentBalances: NrcsAssetBalance[] = [{ asset: 'asset1', balanceQNT: '200' }]

    vi.mocked(storageSelect).mockResolvedValue([
      { id: 'asset_balances', contents: JSON.stringify(previousBalances) },
    ])
    vi.mocked(storageUpdate).mockResolvedValue([])
    vi.mocked(nrcsApi.getAccount).mockResolvedValue(
      buildAccountInfo({ assetBalances: currentBalances }),
    )
    vi.mocked(nrcsApi.getPhasingOnlyControl).mockResolvedValue(buildPhasingOnlyControl())

    const store = useAccountStore()
    const cb = vi.fn()
    store.onAssetDifference(cb)

    await store.getAccountInfo(TEST_ACCOUNT_RS)

    expect(storageUpdate).toHaveBeenCalledWith(
      'data',
      { contents: JSON.stringify(currentBalances) },
      [{ id: 'asset_balances' }],
    )
    expect(cb).toHaveBeenCalled()
  })

  it('无变化时 → 不触发 storageUpdate 和 checkAssetDifferences', async () => {
    const balances: NrcsAssetBalance[] = [{ asset: 'asset1', balanceQNT: '100' }]

    vi.mocked(storageSelect).mockResolvedValue([
      { id: 'asset_balances', contents: JSON.stringify(balances) },
    ])
    vi.mocked(nrcsApi.getAccount).mockResolvedValue(
      buildAccountInfo({ assetBalances: balances }),
    )
    vi.mocked(nrcsApi.getPhasingOnlyControl).mockResolvedValue(buildPhasingOnlyControl())

    const store = useAccountStore()
    const cb = vi.fn()
    store.onAssetDifference(cb)

    await store.getAccountInfo(TEST_ACCOUNT_RS)

    expect(storageUpdate).not.toHaveBeenCalled()
    expect(cb).not.toHaveBeenCalled()
  })
})

describe('account.store: logout 清除新 state', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('logout 后 accountInfo/leasingStatus/phasingOnlyControl 全部清空', () => {
    const store = useAccountStore()
    store.accountInfo = buildAccountInfo()
    store.leasingStatus = {
      label: 'leased_out',
      statusMessage: 'test',
      nextLesseeStatus: 'test',
      lessorCount: 2,
    }
    store.phasingOnlyControl = buildPhasingOnlyControl()
    store.hasPhasingOnlyControl = true
    store.previousAssetBalances = [{ asset: 'a', balanceQNT: '1' }]

    store.logout()

    expect(store.accountInfo).toBeNull()
    expect(store.leasingStatus.label).toBe('')
    expect(store.phasingOnlyControl).toBeNull()
    expect(store.hasPhasingOnlyControl).toBe(false)
    expect(store.previousAssetBalances).toEqual([])
  })
})
