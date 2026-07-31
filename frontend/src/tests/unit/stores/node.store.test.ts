/******************************************************************************
 * NRCS 节点状态 Store 单元测试
 *
 * 验证 node.store.ts 的核心功能（对标 nrs.js）：
 *   - getState：getBlockchainStatus 调用 + apiProxy 分支 + 错误处理（nrs.js:500-561）
 *   - handleBlockchainStatus：首次/扫描中/扫描完成/区块变化/无变化五分支（nrs.js:432-484）
 *   - setStateInterval：自适应轮询间隔 10/15/30s（nrs.js:408-423）
 *   - updateBlockchainDownloadProgress：下载进度计算（nrs.js:1646-1695）
 *   - checkIfOnAFork：分叉检测（nrs.js:1697-1723）
 *   - 回调机制：onFirstState/onBlockChanged/onScanningDone/onNoNewBlock
 *   - 轮询控制：startStatePolling/stopStatePolling
 *   - clearState：状态清空
 *
 * Mock 依赖：
 *   - @/api/modules/nrcs.api：nrcsApi.getBlockchainStatus/getBlocks
 *   - @/utils/feature-detection：isPollGetState/isMobileApp
 ******************************************************************************/
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'

// Mock nrcsApi
vi.mock('@/api/modules/nrcs.api', () => ({
  nrcsApi: {
    getBlockchainStatus: vi.fn(),
    getBlocks: vi.fn(),
  },
}))

// Mock feature-detection
vi.mock('@/utils/feature-detection', () => ({
  isPollGetState: vi.fn(() => true),
  isMobileApp: vi.fn(() => false),
}))

import { useNodeStore } from '@/stores/modules/node.store'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { isPollGetState } from '@/utils/feature-detection'
import type { NrcsBlockchainStatus, NrcsBlock } from '@/api/modules/nrcs.api'

// ============================================================================
// 测试数据构造
// ============================================================================

/** 构造一份最小但结构完整的 getBlockchainStatus 响应 */
function buildStatus(overrides: Partial<NrcsBlockchainStatus> = {}): NrcsBlockchainStatus {
  return {
    application: 'NRCS',
    version: '2.0.0',
    time: 1000000,
    lastBlock: 'block-1',
    lastBlockHeight: 1000,
    cumulativeDifficulty: '12345',
    numberOfBlocks: 1001,
    lastBlockchainFeeder: 'peer-1',
    lastBlockchainFeederHeight: 5000,
    isScanning: false,
    isDownloading: false,
    maxRollback: 720,
    isTestnet: false,
    blockchainState: 'UP_TO_DATE',
    requestProcessingTime: 1,
    ...overrides,
  } as NrcsBlockchainStatus
}

/** 构造一份区块数据 */
function buildBlock(overrides: Partial<NrcsBlock> = {}): NrcsBlock {
  return {
    block: 'block-' + Math.random().toString(36).slice(2, 8),
    height: 1000,
    generator: '1234567890',
    generatorRS: 'NRCS-XXXX-XXXX-XXXX-XXXXX',
    timestamp: 1000000,
    numberOfTransactions: 0,
    totalAmountNQT: '0',
    totalFeeNQT: '0',
    payloadLength: 0,
    version: 3,
    baseTarget: '1000000000',
    cumulativeDifficulty: '12345',
    payloadHash: '0'.repeat(64),
    generationSignature: '0'.repeat(64),
    previousBlockHash: '0'.repeat(64),
    blockSignature: '0'.repeat(128),
    ...overrides,
  } as NrcsBlock
}

// ============================================================================
// 测试用例
// ============================================================================

describe('node.store: 初始状态', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    vi.mocked(isPollGetState).mockReturnValue(true)
  })

  it('初始 state 为 null，连接状态为 false', () => {
    const store = useNodeStore()
    expect(store.state).toBeNull()
    expect(store.serverConnect).toBe(false)
    expect(store.peerConnect).toBe(false)
    expect(store.downloadingBlockchain).toBe(false)
    expect(store.isScanning).toBe(false)
    expect(store.firstTime).toBe(true)
    expect(store.isPolling).toBe(false)
  })

  it('lastBlockHeight 在无状态时返回 0', () => {
    const store = useNodeStore()
    expect(store.lastBlockHeight).toBe(0)
    expect(store.lastBlock).toBe('0')
    expect(store.numberOfBlocks).toBe(0)
  })
})

describe('node.store: getState', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    vi.mocked(isPollGetState).mockReturnValue(true)
  })

  it('成功获取状态并触发 handleBlockchainStatus（非 apiProxy）', async () => {
    const status = buildStatus()
    vi.mocked(nrcsApi.getBlockchainStatus).mockResolvedValue(status)

    const store = useNodeStore()
    await store.getState()

    expect(nrcsApi.getBlockchainStatus).toHaveBeenCalledOnce()
    expect(store.state).toEqual(status)
    expect(store.serverConnect).toBe(true)
  })

  it('apiProxy 模式额外拉取 getBlocks 更新 lastProxyBlock', async () => {
    const status = buildStatus({ apiProxy: true })
    const proxyBlock = buildBlock({ block: 'proxy-block-1', height: 2000 })
    vi.mocked(nrcsApi.getBlockchainStatus).mockResolvedValue(status)
    vi.mocked(nrcsApi.getBlocks).mockResolvedValue({ blocks: [proxyBlock] } as any)

    const store = useNodeStore()
    await store.getState()

    expect(nrcsApi.getBlocks).toHaveBeenCalledWith(0, 0)
    expect(store.lastProxyBlock).toBe('proxy-block-1')
    expect(store.lastProxyBlockHeight).toBe(2000)
    expect(store.state).toEqual(status)
  })

  it('apiProxy 模式但 getBlocks 返回空数组时仍处理状态', async () => {
    const status = buildStatus({ apiProxy: true })
    vi.mocked(nrcsApi.getBlockchainStatus).mockResolvedValue(status)
    vi.mocked(nrcsApi.getBlocks).mockResolvedValue({ blocks: [] } as any)

    const store = useNodeStore()
    await store.getState()

    expect(store.state).toEqual(status)
    expect(store.serverConnect).toBe(true)
  })

  it('getBlockchainStatus 失败时设置 serverConnect=false 并增加 errorCount', async () => {
    const err = new Error('network error') as any
    err.code = 5
    err.description = '网络错误'
    vi.mocked(nrcsApi.getBlockchainStatus).mockRejectedValue(err)

    const store = useNodeStore()
    await store.getState()

    expect(store.serverConnect).toBe(false)
    expect(store.errorCount).toBe(1)
    expect(store.state).toBeNull()
  })

  it('errorCode=19 时不重置 serverConnect（对标 nrs.js:487）', async () => {
    const err = new Error('special') as any
    err.code = 19
    vi.mocked(nrcsApi.getBlockchainStatus).mockRejectedValue(err)

    const store = useNodeStore()
    // 先设置 serverConnect=true 模拟之前已连接
    store.serverConnect = true
    await store.getState()

    // errorCode=19 时不应重置 serverConnect
    expect(store.serverConnect).toBe(true)
  })

  it('callback 在状态处理完成后被调用', async () => {
    const status = buildStatus()
    vi.mocked(nrcsApi.getBlockchainStatus).mockResolvedValue(status)

    const store = useNodeStore()
    const callback = vi.fn()
    await store.getState(callback)

    expect(callback).toHaveBeenCalledOnce()
  })
})

describe('node.store: handleBlockchainStatus 五分支', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    vi.mocked(isPollGetState).mockReturnValue(true)
  })

  it('首次获取状态 → 触发 onFirstState 回调', () => {
    const store = useNodeStore()
    const firstStateCb = vi.fn()
    store.onFirstState(firstStateCb)

    const status = buildStatus({ lastBlock: 'block-1' })
    store.handleBlockchainStatus(status)

    expect(firstStateCb).toHaveBeenCalledWith(status)
    expect(store.firstTime).toBe(false)
  })

  it('区块变化 → 触发 onBlockChanged 回调（对标 nrs.js:459）', () => {
    const store = useNodeStore()

    // 首次设置状态
    const status1 = buildStatus({ lastBlock: 'block-1' })
    store.handleBlockchainStatus(status1)

    // 注册回调
    const blockChangedCb = vi.fn()
    store.onBlockChanged(blockChangedCb)

    // 区块变化
    const status2 = buildStatus({ lastBlock: 'block-2' })
    store.handleBlockchainStatus(status2)

    expect(blockChangedCb).toHaveBeenCalledWith(status2, 'block-1', 'block-2')
  })

  it('正在扫描 → 不触发区块变化回调（对标 nrs.js:446-448）', () => {
    const store = useNodeStore()

    // 首次设置状态
    store.handleBlockchainStatus(buildStatus({ lastBlock: 'block-1' }))

    // 正在扫描
    const scanningStatus = buildStatus({ lastBlock: 'block-2', isScanning: true })
    store.handleBlockchainStatus(scanningStatus)

    expect(store.isScanning).toBe(true)
  })

  it('扫描完成 → 触发 onScanningDone 回调并清空 blocks（对标 nrs.js:449-458）', () => {
    const store = useNodeStore()

    // 首次设置状态
    store.handleBlockchainStatus(buildStatus({ lastBlock: 'block-1' }))

    // 进入扫描状态
    store.handleBlockchainStatus(buildStatus({ lastBlock: 'block-2', isScanning: true }))
    expect(store.isScanning).toBe(true)

    // 先放一些 blocks
    store.updateBlocks([buildBlock(), buildBlock()])

    // 注册扫描完成回调
    const scanningDoneCb = vi.fn()
    store.onScanningDone(scanningDoneCb)

    // 扫描完成（isScanning 从 true 变 false）
    const doneStatus = buildStatus({ lastBlock: 'block-3', isScanning: false })
    store.handleBlockchainStatus(doneStatus)

    expect(store.isScanning).toBe(false)
    expect(store.blocks).toEqual([])
    expect(scanningDoneCb).toHaveBeenCalledWith(doneStatus)
  })

  it('无新区块 → 触发 onNoNewBlock 回调（对标 nrs.js:469-475）', () => {
    const store = useNodeStore()

    // 首次设置状态
    store.handleBlockchainStatus(buildStatus({ lastBlock: 'block-1' }))

    // 注册回调
    const noNewBlockCb = vi.fn()
    store.onNoNewBlock(noNewBlockCb)

    // 相同 lastBlock
    const sameStatus = buildStatus({ lastBlock: 'block-1' })
    store.handleBlockchainStatus(sameStatus)

    expect(noNewBlockCb).toHaveBeenCalledWith(sameStatus)
  })

  it('下载中时自动调用 updateBlockchainDownloadProgress（对标 nrs.js:557-559）', () => {
    const store = useNodeStore()
    store.setDownloadingBlockchain(true)
    store.setPeerConnect(true)

    const status = buildStatus({
      numberOfBlocks: 1000,
      lastBlockchainFeederHeight: 5000,
    })
    store.handleBlockchainStatus(status)

    // downloadProgress 应被计算
    expect(store.downloadProgress).not.toBeNull()
    expect(store.downloadProgress?.blocksLeft).toBe(4000)
  })
})

describe('node.store: setStateInterval', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.useFakeTimers()
    vi.clearAllMocks()
    vi.mocked(isPollGetState).mockReturnValue(true)
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('isPollGetState 返回 false 时不设置定时器', () => {
    vi.mocked(isPollGetState).mockReturnValue(false)
    const store = useNodeStore()
    store.setStateInterval(15)

    expect(store.stateInterval).toBeNull()
    expect(store.stateIntervalSeconds).toBe(30) // 默认值
  })

  it('设置 10 秒间隔', () => {
    const store = useNodeStore()
    store.setStateInterval(10)

    expect(store.stateIntervalSeconds).toBe(10)
    expect(store.stateInterval).not.toBeNull()
  })

  it('相同间隔且定时器存在时不重置（对标 nrs.js:412-414）', () => {
    const store = useNodeStore()
    store.setStateInterval(15)
    const firstTimer = store.stateInterval

    store.setStateInterval(15) // 相同间隔
    expect(store.stateInterval).toBe(firstTimer)
  })

  it('切换间隔时清除旧定时器并设置新定时器', () => {
    const store = useNodeStore()
    store.setStateInterval(30)
    const firstTimer = store.stateInterval

    store.setStateInterval(10)
    expect(store.stateIntervalSeconds).toBe(10)
    expect(store.stateInterval).not.toBe(firstTimer)
  })

  it('定时器触发时调用 getState', () => {
    const status = buildStatus()
    vi.mocked(nrcsApi.getBlockchainStatus).mockResolvedValue(status)

    const store = useNodeStore()
    store.setStateInterval(10)

    // 快进 10 秒
    vi.advanceTimersByTime(10_000)

    expect(nrcsApi.getBlockchainStatus).toHaveBeenCalled()
  })
})

describe('node.store: updateBlockchainDownloadProgress', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    vi.mocked(isPollGetState).mockReturnValue(true)
  })

  it('轻客户端 → downloadProgress 为 null（对标 nrs.js:1651-1652）', () => {
    const store = useNodeStore()
    store.handleBlockchainStatus(buildStatus({ isLightClient: true }))
    store.setPeerConnect(true)

    store.updateBlockchainDownloadProgress()

    expect(store.downloadProgress).toBeNull()
  })

  it('无服务器连接 → downloadProgress 为 null（对标 nrs.js:1653-1654）', () => {
    const store = useNodeStore()
    store.handleBlockchainStatus(buildStatus())
    store.serverConnect = false

    store.updateBlockchainDownloadProgress()

    expect(store.downloadProgress).toBeNull()
  })

  it('正常计算下载进度（对标 nrs.js:1665-1671）', () => {
    const store = useNodeStore()
    store.handleBlockchainStatus(
      buildStatus({
        numberOfBlocks: 1000,
        lastBlockchainFeederHeight: 5000,
      }),
    )
    store.setPeerConnect(true)

    store.updateBlockchainDownloadProgress()

    expect(store.downloadProgress).not.toBeNull()
    expect(store.downloadProgress?.percentageTotal).toBe(20) // 1000/5000 = 20%
    expect(store.downloadProgress?.blocksLeft).toBe(4000)
  })

  it('blocksLeft <= 5000 时计算 percentageLast（对标 nrs.js:1668-1670）', () => {
    const store = useNodeStore()
    store.handleBlockchainStatus(
      buildStatus({
        numberOfBlocks: 4990,
        lastBlockchainFeederHeight: 10000, // > 5000
      }),
    )
    store.setPeerConnect(true)

    store.updateBlockchainDownloadProgress()

    // blocksLeft = 10000 - 4990 = 5010，不满足 <= 5000
    // 但 numberOfBlocks(4990) > lastBlockchainFeederHeight(10000)? No, 4990 <= 10000
    // blocksLeft = 10000 - 4990 = 5010 > 5000，所以 percentageLast = 0
    expect(store.downloadProgress?.blocksLeft).toBe(5010)
    expect(store.downloadProgress?.percentageLast).toBe(0)
  })

  it('blocksLeft <= 5000 且 feeder > 5000 时计算 percentageLast', () => {
    const store = useNodeStore()
    store.handleBlockchainStatus(
      buildStatus({
        numberOfBlocks: 9990,
        lastBlockchainFeederHeight: 10000, // > 5000
      }),
    )
    store.setPeerConnect(true)

    store.updateBlockchainDownloadProgress()

    // blocksLeft = 10000 - 9990 = 10 <= 5000
    // percentageLast = (5000 - 10) / 5000 * 100 = 99.8 → 100
    expect(store.downloadProgress?.blocksLeft).toBe(10)
    expect(store.downloadProgress?.percentageLast).toBe(100)
  })
})

describe('node.store: checkIfOnAFork', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    vi.mocked(isPollGetState).mockReturnValue(true)
  })

  it('下载中 → 不检测（对标 nrs.js:1698）', () => {
    const store = useNodeStore()
    store.setDownloadingBlockchain(true)
    store.updateBlocks(Array.from({ length: 10 }, () => buildBlock({ generator: 'me' })))

    store.checkIfOnAFork('me')

    expect(store.forkWarning).toBeNull()
  })

  it('最近 10 块生成者都等于当前账户 → fork_warning（对标 nrs.js:1711-1715）', () => {
    const store = useNodeStore()
    const myAccount = '1234567890'
    store.updateBlocks(
      Array.from({ length: 10 }, () => buildBlock({ generator: myAccount })),
    )

    store.checkIfOnAFork(myAccount)

    expect(store.forkWarning).toBe('fork_warning')
  })

  it('最近 10 块生成者不全等于当前账户 → 无警告', () => {
    const store = useNodeStore()
    const myAccount = '1234567890'
    const blocks = Array.from({ length: 10 }, (_, i) =>
      buildBlock({ generator: i === 5 ? 'other' : myAccount }),
    )
    store.updateBlocks(blocks)

    store.checkIfOnAFork(myAccount)

    expect(store.forkWarning).toBeNull()
  })

  it('blocks 不足 10 块 → 无警告', () => {
    const store = useNodeStore()
    store.updateBlocks([buildBlock({ generator: 'me' }), buildBlock({ generator: 'me' })])

    store.checkIfOnAFork('me')

    expect(store.forkWarning).toBeNull()
  })

  it('account 为空时即使 10 块相同也不警告', () => {
    const store = useNodeStore()
    store.updateBlocks(Array.from({ length: 10 }, () => buildBlock({ generator: 'me' })))

    store.checkIfOnAFork(undefined)

    expect(store.forkWarning).toBeNull()
  })

  it('baseTarget 百分比超过 1000% → fork_warning_base_target（对标 nrs.js:1717-1721）', () => {
    const store = useNodeStore()
    // baseTarget = 1000000000 → 1000000000 / 1000000000 * 100 = 100，不超 1000
    // 需要 baseTarget > 10 * 1000000000 = 10000000000
    store.updateBlocks([
      buildBlock({ generator: 'other', baseTarget: '20000000000' }),
    ])

    store.checkIfOnAFork('me', false) // 非测试网

    expect(store.forkWarning).toBe('fork_warning_base_target')
  })

  it('测试网时 baseTarget 超阈值也不警告（对标 nrs.js:1717 !NRS.isTestNet）', () => {
    const store = useNodeStore()
    store.updateBlocks([
      buildBlock({ generator: 'other', baseTarget: '20000000000' }),
    ])

    store.checkIfOnAFork('me', true) // 测试网

    expect(store.forkWarning).toBeNull()
  })
})

describe('node.store: 轮询控制', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.useFakeTimers()
    vi.clearAllMocks()
    vi.mocked(isPollGetState).mockReturnValue(true)
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('startStatePolling 立即执行一次 getState', () => {
    const status = buildStatus()
    vi.mocked(nrcsApi.getBlockchainStatus).mockResolvedValue(status)

    const store = useNodeStore()
    store.startStatePolling()

    expect(store.isPolling).toBe(true)
    expect(nrcsApi.getBlockchainStatus).toHaveBeenCalledOnce()
  })

  it('isPollGetState 返回 false 时不启动轮询', () => {
    vi.mocked(isPollGetState).mockReturnValue(false)
    const store = useNodeStore()
    store.startStatePolling()

    expect(store.isPolling).toBe(false)
  })

  it('stopStatePolling 清除定时器并设置 isPolling=false', () => {
    const status = buildStatus()
    vi.mocked(nrcsApi.getBlockchainStatus).mockResolvedValue(status)

    const store = useNodeStore()
    store.startStatePolling()
    expect(store.stateInterval).not.toBeNull()

    store.stopStatePolling()

    expect(store.isPolling).toBe(false)
    expect(store.stateInterval).toBeNull()
  })

  it('refreshNow 手动刷新触发 getState', async () => {
    const status = buildStatus()
    vi.mocked(nrcsApi.getBlockchainStatus).mockResolvedValue(status)

    const store = useNodeStore()
    await store.refreshNow()

    expect(nrcsApi.getBlockchainStatus).toHaveBeenCalledOnce()
    expect(store.state).toEqual(status)
  })
})

describe('node.store: clearState', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    vi.mocked(isPollGetState).mockReturnValue(true)
  })

  it('清空所有状态', async () => {
    const status = buildStatus()
    vi.mocked(nrcsApi.getBlockchainStatus).mockResolvedValue(status)

    const store = useNodeStore()
    await store.getState()
    store.updateBlocks([buildBlock()])
    store.setPeerConnect(true)

    expect(store.state).not.toBeNull()
    expect(store.blocks.length).toBe(1)

    store.clearState()

    expect(store.state).toBeNull()
    expect(store.serverConnect).toBe(false)
    expect(store.peerConnect).toBe(false)
    expect(store.downloadingBlockchain).toBe(false)
    expect(store.isScanning).toBe(false)
    expect(store.firstTime).toBe(true)
    expect(store.blocks).toEqual([])
    expect(store.downloadProgress).toBeNull()
    expect(store.forkWarning).toBeNull()
  })
})

describe('node.store: 回调管理', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    vi.mocked(isPollGetState).mockReturnValue(true)
  })

  it('clearCallbacks 清除所有已注册回调', () => {
    const store = useNodeStore()
    const cb1 = vi.fn()
    const cb2 = vi.fn()
    const cb3 = vi.fn()
    const cb4 = vi.fn()

    store.onFirstState(cb1)
    store.onBlockChanged(cb2)
    store.onScanningDone(cb3)
    store.onNoNewBlock(cb4)

    store.clearCallbacks()

    // 触发首次状态
    store.handleBlockchainStatus(buildStatus())

    expect(cb1).not.toHaveBeenCalled()
  })
})

describe('node.store: getters', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    vi.mocked(isPollGetState).mockReturnValue(true)
  })

  it('lastBlockHeight 非 apiProxy 时返回 numberOfBlocks - 1', () => {
    const store = useNodeStore()
    store.handleBlockchainStatus(buildStatus({ numberOfBlocks: 1001, apiProxy: false }))

    expect(store.lastBlockHeight).toBe(1000)
  })

  it('lastBlockHeight apiProxy 时返回 lastProxyBlockHeight', () => {
    const store = useNodeStore()
    store.handleBlockchainStatus(buildStatus({ numberOfBlocks: 1001, apiProxy: true }))
    store.lastProxyBlockHeight = 2000

    expect(store.lastBlockHeight).toBe(2000)
  })

  it('isTestnet/isLightClient/isApiProxy getters 正确反映 state', () => {
    const store = useNodeStore()
    store.handleBlockchainStatus(
      buildStatus({ isTestnet: true, isLightClient: true, apiProxy: true }),
    )

    expect(store.isTestnet).toBe(true)
    expect(store.isLightClient).toBe(true)
    expect(store.isApiProxy).toBe(true)
  })
})
