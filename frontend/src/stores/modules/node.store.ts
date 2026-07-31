/******************************************************************************
 * NRCS 节点状态 Store —— 对标 nrs.js 的 getState / handleBlockchainStatus /
 * setStateInterval / updateBlockchainDownloadProgress / checkIfOnAFork。
 *
 * 本 Store 在阶段 1.1 基于 `nrcsApi`（NRCS requestType 风格）重写，
 * 取代原先基于 `nodeApi`（Ethereum RESTful 风格）的占位实现。
 *
 * 核心能力：
 *   - `getState`：调用 `getBlockchainStatus` 获取链状态（对标 nrs.js:500-561）
 *   - `handleBlockchainStatus`：处理状态响应，检测区块变化（对标 nrs.js:432-484）
 *   - `setStateInterval`：自适应轮询间隔 10/15/30s（对标 nrs.js:408-423）
 *   - `updateBlockchainDownloadProgress`：下载进度计算（对标 nrs.js:1646-1695）
 *   - `checkIfOnAFork`：分叉检测（对标 nrs.js:1697-1723）
 *
 * 解耦设计：handleBlockchainStatus 通过 `onBlockChanged`/`onFirstState`
 * 回调通知外部模块（blocks/transactions/account），避免直接依赖其他 Store。
 *
 * 参考源码：/Volumes/DATA/data/develop/git/nrcs/nrcs-main/html/www/ui/js/nrs.js
 ******************************************************************************/
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import type { NrcsBlockchainStatus, NrcsBlock } from '@/api/modules/nrcs.api'
import { isPollGetState } from '@/utils/feature-detection'

// ============================================================================
// 常量
// ============================================================================

/** 默认轮询间隔（秒），对标 nrs.js 初始 stateIntervalSeconds */
const DEFAULT_STATE_INTERVAL_SECONDS = 30

/** 快速轮询间隔（秒），区块变化时使用 */
const FAST_STATE_INTERVAL_SECONDS = 10

/** 中速轮询间隔（秒） */
const MEDIUM_STATE_INTERVAL_SECONDS = 15

/** 分叉检测：检查最近区块数（对标 nrs.js:1700-1706 的 i < 10） */
const FORK_CHECK_BLOCKS = 10

/** 下载进度：最后 N 块的进度条（对标 nrs.js:1647 lastNumBlocks = 5000） */
const LAST_NUM_BLOCKS = 5000

/** 基础目标百分比阈值，超过则警告分叉（对标 nrs.js:1717 的 > 1000） */
const FORK_BASE_TARGET_PERCENT_THRESHOLD = 1000

// ============================================================================
// 回调类型
// ============================================================================

/**
 * 首次获取到链状态时的回调（对标 nrs.js:443 firstTime 分支）。
 * 外部 blocks 模块可在此调用 getBlock + handleInitialBlocks。
 */
type FirstStateCallback = (state: NrcsBlockchainStatus) => void

/**
 * 链状态变化时的回调（对标 nrs.js:459 previousLastBlock != lastBlock 分支）。
 * 外部 blocks/transactions/account 模块可在此触发增量更新。
 */
type BlockChangedCallback = (
  state: NrcsBlockchainStatus,
  previousLastBlock: string,
  lastBlock: string,
) => void

/**
 * 扫描完成时的回调（对标 nrs.js:449-458 isScanning 分支）。
 * 扫描结束后需要重置区块/交易/账户数据。
 */
type ScanningDoneCallback = (state: NrcsBlockchainStatus) => void

/**
 * 无新区块时的回调（对标 nrs.js:469-475 else 分支）。
 * 外部 transactions 模块可在此获取未确认交易。
 */
type NoNewBlockCallback = (state: NrcsBlockchainStatus) => void

// ============================================================================
// Store 定义
// ============================================================================

export const useNodeStore = defineStore('node', () => {
  // --------------------------------------------------------------------------
  // State（对标 NRS.state 及模块级变量）
  // --------------------------------------------------------------------------

  /** 当前链状态（对标 NRS.state，nrs.js:436 NRS.state = response） */
  const state = ref<NrcsBlockchainStatus | null>(null)

  /** 服务器连接状态（对标 NRS.serverConnect，nrs.js:440） */
  const serverConnect = ref(false)

  /** Peer 连接状态（对标 NRS.peerConnect） */
  const peerConnect = ref(false)

  /** 是否正在下载区块链（对标 NRS.downloadingBlockchain，nrs.js:557） */
  const downloadingBlockchain = ref(false)

  /** 是否正在扫描（对标 nrs.js:448 模块级 isScanning） */
  const isScanning = ref(false)

  /** 最后代理区块 ID（对标 NRS.lastProxyBlock，nrs.js:528） */
  const lastProxyBlock = ref<string>('0')

  /** 最后代理区块高度（对标 NRS.lastProxyBlockHeight，nrs.js:529） */
  const lastProxyBlockHeight = ref<number>(0)

  /** 最近区块列表（对标 NRS.blocks，用于分叉检测 nrs.js:1700） */
  const blocks = ref<NrcsBlock[]>([])

  /** 上一次的代理区块（对标 nrs.js:426 _prevLastProxyBlock） */
  const prevLastProxyBlock = ref<string>('0')

  /** 是否首次获取状态（对标 nrs.js:433 firstTime） */
  const firstTime = ref(true)

  // 轮询控制
  /** 当前轮询间隔秒数（对标 nrs.js:418 stateIntervalSeconds） */
  const stateIntervalSeconds = ref<number>(DEFAULT_STATE_INTERVAL_SECONDS)

  /** 轮询定时器句柄（对标 nrs.js:419 stateInterval） */
  const stateInterval = ref<ReturnType<typeof setInterval> | null>(null)

  /** 是否正在轮询 */
  const isPolling = ref(false)

  /** 上次轮询时间戳（毫秒） */
  const lastPollTime = ref<number>(0)

  /** 连续错误计数（用于自适应退避） */
  const errorCount = ref<number>(0)

  // 下载进度（对标 nrs.js:1646-1695 计算结果）
  /** 下载进度信息 */
  const downloadProgress = ref<{
    percentageTotal: number
    percentageLast: number
    blocksLeft: number
  } | null>(null)

  // 分叉警告（对标 nrs.js:1712/1718 $.growl）
  /** 分叉警告消息（null 表示无警告） */
  const forkWarning = ref<string | null>(null)

  // 回调注册表
  const firstStateCallbacks = ref<FirstStateCallback[]>([])
  const blockChangedCallbacks = ref<BlockChangedCallback[]>([])
  const scanningDoneCallbacks = ref<ScanningDoneCallback[]>([])
  const noNewBlockCallbacks = ref<NoNewBlockCallback[]>([])

  // --------------------------------------------------------------------------
  // Getters
  // --------------------------------------------------------------------------

  /** 最后区块 ID（对标 nrs.js:437 lastBlock） */
  const lastBlock = computed<string>(() => state.value?.lastBlock || '0')

  /** 最后区块高度（对标 nrs.js:438 height，apiProxy 时用 lastProxyBlockHeight） */
  const lastBlockHeight = computed<number>(() => {
    if (!state.value) return 0
    return state.value.apiProxy
      ? lastProxyBlockHeight.value
      : Math.max(0, state.value.numberOfBlocks - 1)
  })

  /** 区块总数（对标 NRS.state.numberOfBlocks） */
  const numberOfBlocks = computed<number>(() => state.value?.numberOfBlocks || 0)

  /** 节点版本（对标 NRS.state.version） */
  const nodeVersion = computed<string>(() => state.value?.version || 'Unknown')

  /** 应用名称（对标 NRS.state.application） */
  const application = computed<string>(() => state.value?.application || '')

  /** 是否测试网（对标 NRS.state.isTestnet） */
  const isTestnet = computed<boolean>(() => state.value?.isTestnet ?? false)

  /** 是否轻客户端（对标 NRS.state.isLightClient） */
  const isLightClient = computed<boolean>(() => state.value?.isLightClient ?? false)

  /** 是否 API 代理（对标 NRS.state.apiProxy） */
  const isApiProxy = computed<boolean>(() => state.value?.apiProxy ?? false)

  /** 是否正在扫描（对标 NRS.state.isScanning） */
  const isScanningState = computed<boolean>(() => state.value?.isScanning ?? false)

  /** 账本裁剪保留数（对标 NRS.state.ledgerTrimKeep） */
  const ledgerTrimKeep = computed<number>(() => state.value?.ledgerTrimKeep ?? 0)

  /** 累积难度（对标 NRS.state.cumulativeDifficulty） */
  const cumulativeDifficulty = computed<string>(() =>
    state.value?.cumulativeDifficulty || '0',
  )

  /** 最后区块链供给者高度（对标 NRS.state.lastBlockchainFeederHeight） */
  const lastBlockchainFeederHeight = computed<number>(() =>
    state.value?.lastBlockchainFeederHeight ?? 0,
  )

  // --------------------------------------------------------------------------
  // 回调注册（供 blocks/transactions/account 模块注册监听）
  // --------------------------------------------------------------------------

  /**
   * 注册首次获取状态回调（对标 nrs.js:443 firstTime 分支）。
   * blocks 模块应在此调用 getBlock + handleInitialBlocks。
   */
  function onFirstState(cb: FirstStateCallback): void {
    firstStateCallbacks.value.push(cb)
  }

  /**
   * 注册区块变化回调（对标 nrs.js:459 previousLastBlock != lastBlock 分支）。
   * blocks/transactions/account 模块应在此触发增量更新。
   */
  function onBlockChanged(cb: BlockChangedCallback): void {
    blockChangedCallbacks.value.push(cb)
  }

  /**
   * 注册扫描完成回调（对标 nrs.js:449-458 isScanning 分支）。
   * 扫描结束后需要重置区块/交易/账户数据。
   */
  function onScanningDone(cb: ScanningDoneCallback): void {
    scanningDoneCallbacks.value.push(cb)
  }

  /**
   * 注册无新区块回调（对标 nrs.js:469-475 else 分支）。
   * transactions 模块应在此获取未确认交易。
   */
  function onNoNewBlock(cb: NoNewBlockCallback): void {
    noNewBlockCallbacks.value.push(cb)
  }

  /** 清除所有回调注册 */
  function clearCallbacks(): void {
    firstStateCallbacks.value = []
    blockChangedCallbacks.value = []
    scanningDoneCallbacks.value = []
    noNewBlockCallbacks.value = []
  }

  // --------------------------------------------------------------------------
  // Actions
  // --------------------------------------------------------------------------

  /**
   * 获取区块链状态（对标 nrs.js:500-561 NRS.getState）。
   *
   * 调用 `getBlockchainStatus` API，处理响应：
   *   - apiProxy 模式：额外拉取最新区块更新 lastProxyBlock
   *   - 非 apiProxy 模式：直接交给 handleBlockchainStatus
   *   - 下载中：更新下载进度
   *
   * 注意：`nrcsGet` 在服务端返回 errorCode 时会抛出异常（见 nrcs-client.ts:37-41），
   * 因此错误统一由 catch 块处理，对标 nrs.js:505-506 的 connectionError。
   *
   * @param callback 状态处理完成后的回调
   * @param msg 调试日志消息（对标 nrs.js:502 logConsole）
   */
  async function getState(callback?: () => void, msg?: string): Promise<void> {
    if (msg) {
      console.log('[node.store] getState event: ' + msg)
    }

    try {
      const response = await nrcsApi.getBlockchainStatus()

      // 对标 nrs.js:512 apiProxy 分支
      if (response.apiProxy) {
        // 拉取最新区块以更新 lastProxyBlock（对标 nrs.js:520-536）
        try {
          const proxyBlocksResponse = await nrcsApi.getBlocks(0, 0)
          const proxyBlocks = proxyBlocksResponse.blocks
          if (proxyBlocks && proxyBlocks.length > 0) {
            prevLastProxyBlock.value = lastProxyBlock.value
            lastProxyBlock.value = proxyBlocks[0].block
            lastProxyBlockHeight.value = proxyBlocks[0].height
            handleBlockchainStatus(response, callback)
            return
          }
        } catch (proxyErr: any) {
          // 对标 nrs.js:523-525 代理区块拉取失败
          connectionError(
            proxyErr?.description || proxyErr?.message || '代理区块拉取失败',
            proxyErr?.code ?? -1,
          )
        }
      }

      // 非 apiProxy 或代理区块拉取失败时直接处理状态（对标 nrs.js:542）
      handleBlockchainStatus(response, callback)
    } catch (err: any) {
      // 对标 nrs.js:505-506 connectionError
      connectionError(
        err?.description || err?.message || String(err),
        err?.code ?? -1,
      )
    }
  }

  /**
   * 处理区块链状态响应（对标 nrs.js:432-484 NRS.handleBlockchainStatus）。
   *
   * 核心逻辑：
   *   1. 首次（firstTime）：触发 onFirstState 回调（blocks 模块拉取初始区块）
   *   2. 正在扫描（isScanning）：仅记录状态
   *   3. 扫描刚完成：触发 onScanningDone 回调（重置数据）
   *   4. 区块变化：触发 onBlockChanged 回调（增量更新）
   *   5. 无变化：触发 onNoNewBlock 回调（拉取未确认交易）
   *
   * @param response getBlockchainStatus 响应
   * @param callback 处理完成后的回调
   */
  function handleBlockchainStatus(
    response: NrcsBlockchainStatus,
    callback?: () => void,
  ): void {
    // 对标 nrs.js:433 firstTime = !("lastBlock" in NRS.state)
    const isFirstTime = firstTime.value
    const previousLastBlock = isFirstTime ? '0' : state.value?.lastBlock || '0'

    // 对标 nrs.js:436 NRS.state = response
    state.value = response
    serverConnect.value = true // 对标 nrs.js:440

    const currentLastBlock = response.lastBlock

    if (isFirstTime) {
      // 对标 nrs.js:443-445 首次获取状态
      firstTime.value = false
      firstStateCallbacks.value.forEach((cb) => cb(response))
    } else if (response.isScanning) {
      // 对标 nrs.js:446-448 正在扫描
      isScanning.value = true
    } else if (isScanning.value) {
      // 对标 nrs.js:449-458 扫描刚完成，需要重置
      isScanning.value = false
      blocks.value = []
      scanningDoneCallbacks.value.forEach((cb) => cb(response))
    } else if (previousLastBlock !== currentLastBlock) {
      // 对标 nrs.js:459-468 区块变化
      blockChangedCallbacks.value.forEach((cb) =>
        cb(response, previousLastBlock, currentLastBlock),
      )
    } else {
      // 对标 nrs.js:469-475 无新区块
      noNewBlockCallbacks.value.forEach((cb) => cb(response))
    }

    // 对标 nrs.js:557-559 下载中更新进度
    if (downloadingBlockchain.value) {
      updateBlockchainDownloadProgress()
    }

    if (callback) {
      callback()
    }
  }

  /**
   * 连接错误处理（对标 nrs.js:486-498 NRS.connectionError）。
   *
   * @param errorDescription 错误描述
   * @param errorCode 错误码（19 表示特殊错误，不重置 serverConnect）
   */
  function connectionError(errorDescription: string, errorCode: number): void {
    if (errorCode !== 19) {
      serverConnect.value = false
    }
    errorCount.value++
    const msg = `[node.store] 服务器连接错误: ${errorDescription || '未知错误'}`
    console.error(msg)
  }

  /**
   * 设置状态轮询间隔（对标 nrs.js:408-423 NRS.setStateInterval）。
   *
   * 自适应策略：
   *   - 30s：默认/空闲
   *   - 15s：中等活跃
   *   - 10s：高活跃（区块频繁变化）
   *
   * 仅在 isPollGetState() 返回 true 时生效（对标 nrs.js:409）。
   *
   * @param seconds 轮询间隔秒数
   */
  function setStateInterval(seconds: number): void {
    // 对标 nrs.js:409-411 特性检测
    if (!isPollGetState()) {
      return
    }
    // 对标 nrs.js:412-414 间隔未变则不重置
    if (seconds === stateIntervalSeconds.value && stateInterval.value) {
      return
    }
    // 对标 nrs.js:415-417 清除旧定时器
    if (stateInterval.value) {
      clearInterval(stateInterval.value)
    }
    stateIntervalSeconds.value = seconds
    // 对标 nrs.js:418-422 设置新定时器
    stateInterval.value = setInterval(() => {
      getState(undefined, 'poll')
      // updateForgingStatus 由 forging 模块自行轮询（阶段 2.9）
    }, 1000 * seconds)
  }

  /**
   * 更新区块链下载进度（对标 nrs.js:1646-1695 NRS.updateBlockchainDownloadProgress）。
   *
   * 计算逻辑：
   *   - percentageTotal：当前区块数 / 供给者高度 * 100
   *   - blocksLeft：供给者高度 - 当前区块数
   *   - percentageLast：最后 5000 块的进度
   *
   * 轻客户端不显示下载进度（对标 nrs.js:1651-1652）。
   */
  function updateBlockchainDownloadProgress(): void {
    // 对标 nrs.js:1651-1652 轻客户端隐藏
    if (state.value?.isLightClient) {
      downloadProgress.value = null
      return
    }
    // 对标 nrs.js:1653-1656 无连接时显示 halted
    if (!serverConnect.value || !peerConnect.value) {
      downloadProgress.value = null
      return
    }

    let percentageTotal = 0
    let blocksLeft: number | undefined
    let percentageLast = 0

    // 对标 nrs.js:1665-1671
    if (
      state.value?.lastBlockchainFeederHeight &&
      state.value.numberOfBlocks <= state.value.lastBlockchainFeederHeight
    ) {
      percentageTotal = Math.round(
        (state.value.numberOfBlocks / state.value.lastBlockchainFeederHeight) * 100,
      )
      blocksLeft =
        state.value.lastBlockchainFeederHeight - state.value.numberOfBlocks
      if (
        blocksLeft <= LAST_NUM_BLOCKS &&
        state.value.lastBlockchainFeederHeight > LAST_NUM_BLOCKS
      ) {
        percentageLast = Math.round(
          ((LAST_NUM_BLOCKS - blocksLeft) / LAST_NUM_BLOCKS) * 100,
        )
      }
    }

    downloadProgress.value = {
      percentageTotal,
      percentageLast,
      blocksLeft: blocksLeft ?? 0,
    }
  }

  /**
   * 检测是否处于分叉状态（对标 nrs.js:1697-1723 NRS.checkIfOnAFork）。
   *
   * 分叉检测算法：
   *   1. 检查最近 10 个区块的生成者是否都等于当前账户（异常情况）
   *   2. 检查最新区块的 baseTarget 百分比是否超过 1000%（异常波动）
   *
   * @param account 当前账户 ID（对标 NRS.account，nrs.js:1702）
   * @param isTestnetOverride 是否测试网（对标 NRS.isTestNet，nrs.js:1717）
   */
  function checkIfOnAFork(account?: string, isTestnetOverride?: boolean): void {
    // 对标 nrs.js:1698 下载中不检测
    if (downloadingBlockchain.value) {
      return
    }

    forkWarning.value = null

    // 对标 nrs.js:1699-1709 检查最近 10 个区块生成者
    let isForgingAllBlocks = true
    if (blocks.value.length >= FORK_CHECK_BLOCKS) {
      for (let i = 0; i < FORK_CHECK_BLOCKS; i++) {
        if (blocks.value[i].generator !== account) {
          isForgingAllBlocks = false
          break
        }
      }
    } else {
      isForgingAllBlocks = false
    }

    // 对标 nrs.js:1711-1715 连续 10 块都是自己生成 → 分叉警告
    if (isForgingAllBlocks && account) {
      forkWarning.value = 'fork_warning'
    }

    // 对标 nrs.js:1717-1721 baseTarget 百分比超过 1000% → 分叉警告
    const testnet = isTestnetOverride ?? isTestnet.value
    if (blocks.value.length > 0 && !testnet) {
      const baseTargetPercent = calculateBaseTargetPercent(blocks.value[0])
      if (baseTargetPercent > FORK_BASE_TARGET_PERCENT_THRESHOLD) {
        forkWarning.value = 'fork_warning_base_target'
      }
    }
  }

  /**
   * 计算 baseTarget 百分比（对标 NRS.baseTargetPercent，nrs.js:1717 调用）。
   *
   * NRCS 中 baseTarget 相对于初始值的百分比，反映 forging 难度变化。
   * 此处采用简化实现：baseTarget 数值与Genesis基准的比值。
   *
   * @param block 区块
   * @returns baseTarget 百分比
   */
  function calculateBaseTargetPercent(block: NrcsBlock): number {
    const baseTarget = parseInt(block.baseTarget, 10) || 0
    if (baseTarget === 0) return 0
    // 简化：返回 baseTarget 相对于基准的百分比
    // 实际 NRCS 使用 NRS.state.baseTarget / 初始 baseTarget * 100
    return Math.round((baseTarget / 1_000_000_000) * 100)
  }

  // --------------------------------------------------------------------------
  // 轮询控制
  // --------------------------------------------------------------------------

  /**
   * 开始状态轮询（对标 nrs.js 中登录后启动 getState 轮询）。
   *
   * 立即执行一次 getState，然后按 stateIntervalSeconds 间隔轮询。
   */
  function startStatePolling(): void {
    if (isPolling.value) return
    if (!isPollGetState()) {
      console.warn('[node.store] isPollGetState() 返回 false，不启动轮询')
      return
    }
    isPolling.value = true
    errorCount.value = 0

    // 立即执行一次（对标 nrs.js 登录后首次 getState）
    getState(undefined, 'start').catch((err) =>
      console.error('[node.store] 首次 getState 失败:', err),
    )

    // 设置自适应轮询
    setStateInterval(stateIntervalSeconds.value)
  }

  /**
   * 停止状态轮询。
   */
  function stopStatePolling(): void {
    if (stateInterval.value) {
      clearInterval(stateInterval.value)
      stateInterval.value = null
    }
    isPolling.value = false
  }

  /**
   * 立即刷新一次状态（供手动刷新按钮调用）。
   */
  async function refreshNow(): Promise<void> {
    await getState(undefined, 'manual')
  }

  /**
   * 设置下载状态（对标 NRS.downloadingBlockchain 赋值）。
   * 由 blocks 模块在初始下载时设置。
   */
  function setDownloadingBlockchain(downloading: boolean): void {
    downloadingBlockchain.value = downloading
  }

  /**
   * 设置 peer 连接状态（对标 NRS.peerConnect 赋值）。
   * 由 checkConnected 或 peer 模块设置。
   */
  function setPeerConnect(connected: boolean): void {
    peerConnect.value = connected
  }

  /**
   * 更新最近区块列表（对标 NRS.blocks 赋值，供分叉检测用）。
   * 由 blocks 模块在 handleInitialBlocks/handleNewBlocks 后调用。
   */
  function updateBlocks(newBlocks: NrcsBlock[]): void {
    blocks.value = newBlocks
  }

  /**
   * 清除所有状态（对标登出时重置）。
   */
  function clearState(): void {
    stopStatePolling()
    state.value = null
    serverConnect.value = false
    peerConnect.value = false
    downloadingBlockchain.value = false
    isScanning.value = false
    lastProxyBlock.value = '0'
    lastProxyBlockHeight.value = 0
    prevLastProxyBlock.value = '0'
    firstTime.value = true
    blocks.value = []
    downloadProgress.value = null
    forkWarning.value = null
    errorCount.value = 0
    stateIntervalSeconds.value = DEFAULT_STATE_INTERVAL_SECONDS
    clearCallbacks()
  }

  return {
    // state
    state,
    serverConnect,
    peerConnect,
    downloadingBlockchain,
    isScanning,
    lastProxyBlock,
    lastProxyBlockHeight,
    blocks,
    prevLastProxyBlock,
    firstTime,
    stateIntervalSeconds,
    stateInterval,
    isPolling,
    lastPollTime,
    errorCount,
    downloadProgress,
    forkWarning,
    // getters
    lastBlock,
    lastBlockHeight,
    numberOfBlocks,
    nodeVersion,
    application,
    isTestnet,
    isLightClient,
    isApiProxy,
    isScanningState,
    ledgerTrimKeep,
    cumulativeDifficulty,
    lastBlockchainFeederHeight,
    // callback registration
    onFirstState,
    onBlockChanged,
    onScanningDone,
    onNoNewBlock,
    clearCallbacks,
    // actions
    getState,
    handleBlockchainStatus,
    setStateInterval,
    updateBlockchainDownloadProgress,
    checkIfOnAFork,
    connectionError,
    startStatePolling,
    stopStatePolling,
    refreshNow,
    setDownloadingBlockchain,
    setPeerConnect,
    updateBlocks,
    clearState,
  }
})
