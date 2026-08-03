/**
 * RemoteNodesManager —— 远程节点管理器。
 *
 * 端口自参考实现 `util/remotenodesmgr.js:41-293` 的 `RemoteNodesManager` 类。
 *
 * 职责：
 *   - 维护当前可用的远程节点集合（`nodes` map，按 address 索引）
 *   - 移动端：从内置 bootstrap 列表连接初始节点（`addBootstrapNodes` / `addBootstrapNode`）
 *   - API 代理端：从 `getPeers` 拉取 CONNECTED 节点（`addRemoteNodes`，由 `updateRemoteNodes` 调用）
 *   - 随机选取非黑名单节点用于请求转发与响应交叉验证（`getRandomNode` / `getRandomNodes`）
 *   - 周期性通过已知节点发现更多节点（`findMoreNodes`）
 *
 * 适配说明（Vue3 与参考的差异）：
 * - 参考使用回调式 `resolve/reject` + 全局 `NRS.sendRequest`，本实现改为 Promise + 注入 `remoteTransport`，
 *   避免对 nrcs-client 的硬依赖，便于测试。
 * - 参考通过 `jQuery.getScript` 动态加载 bootstrap 数据，本实现改为静态 JSON 导入（Vite 原生支持）。
 * - 参考依赖全局 `NRS.isMobileApp` / `NRS.isRequireCors` / `NRS.isRemoteNodeConnectionAllowed`，
 *   本实现改为从 `feature-detection` 导入同名函数。
 */
import { RemoteNode } from './remote-node'
import type { PeerData, RemoteNodeHandle, RemoteNodeRequestFn, RemoteRequestOptions } from './types'
import { getMobileSettings } from './mobile-settings'
import { isMobileApp, isRequireCors, isRemoteNodeConnectionAllowed } from '@/utils/feature-detection'
import mainnetBootstrap from './data/bootstrap-mainnet.json'
import testnetBootstrap from './data/bootstrap-testnet.json'

/** bootstrap 连接计数器（对标 `util/remotenodesmgr.js:45-52` 的 `this.bc`） */
interface BootstrapCounters {
  /** 成功连接数 */
  success: number
  /** 失败连接数 */
  fail: number
  /** 总尝试数 */
  counter: number
  /** 目标成功数 */
  target: number
  /** 下一个待尝试索引 */
  index: number
  /** bootstrap 是否已完成 */
  bootstrapComplete: boolean
}

/**
 * 判断节点版本是否过旧。
 *
 * 对标 `util/remotenodesmgr.js:56` 的 `isOldVersion`。
 * 版本格式 x.y.z：x<1 或 (x>=1 且 y<10) 视为旧版本。
 *
 * @param version - 版本字符串
 * @returns 旧版本返回 true
 */
function isOldVersion(version: string | undefined): boolean {
  if (!version) return true
  const parts = String(version).split('.')
  if (parts.length !== 3) return true
  const major = parseInt(parts[0], 10)
  if (major < 1) return true
  return parseInt(parts[1], 10) < 10
}

/**
 * 判断节点是否可连接。
 *
 * 对标 `util/remotenodesmgr.js:68` 的 `isRemoteNodeConnectable`。
 *
 * 条件：
 *   1. services 是数组且包含 'API'（或 isSslAllowed 时包含 'API_SSL'）
 *   2. 不需要 CORS（移动端）或 services 包含 'CORS'
 *   3. 版本不过旧
 *
 * @param nodeData - peer 数据（也兼容 getBlockchainStatus 响应，其 services/version 字段来自 peer）
 * @param isSslAllowed - 是否允许 SSL 节点
 */
export function isRemoteNodeConnectable(nodeData: PeerData, isSslAllowed: boolean): boolean {
  const services = nodeData.services
  if (!Array.isArray(services)) return false
  const hasApi = services.indexOf('API') >= 0 || (isSslAllowed && services.indexOf('API_SSL') >= 0)
  if (!hasApi) return false
  if (isRequireCors() && services.indexOf('CORS') < 0) return false
  return !isOldVersion(nodeData.version)
}

/**
 * Fisher-Yates 随机打乱数组（原地修改）。
 *
 * 对标参考 `nrs.util.js` 的 `NRS.getRandomPermutation`。
 *
 * @param arr - 待打乱数组
 * @returns 打乱后的同一数组引用
 */
function shuffle<T>(arr: T[]): T[] {
  for (let i = arr.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1))
    const tmp = arr[i]
    arr[i] = arr[j]
    arr[j] = tmp
  }
  return arr
}

/**
 * 远程节点管理器。
 *
 * 对标 `util/remotenodesmgr.js:41` 的 `function RemoteNodesManager(isTestnet)`。
 */
export class RemoteNodesManager {
  /** 是否测试网 */
  public isTestnet: boolean
  /** 节点集合（key=address, value=RemoteNode） */
  public nodes: Record<string, RemoteNode> = {}
  /** bootstrap 计数器 */
  public bc: BootstrapCounters = {
    success: 0,
    fail: 0,
    counter: 0,
    target: 0,
    index: 0,
    bootstrapComplete: false,
  }
  /** 注入的远程请求函数（由 nrcs-client 注册） */
  private remoteTransport: RemoteNodeRequestFn | null = null

  /**
   * 构造管理器。
   *
   * 对标 `util/remotenodesmgr.js:41-54`：初始化 nodes/bc 并调用 init。
   * 参考在 init 中通过 `jQuery.getScript` 加载 bootstrap 数据，本实现改为静态导入，
   * 因此 init 在 Web 端为空操作，bootstrap 数据直接通过 `getBootstrapData()` 读取。
   *
   * @param isTestnet - 是否测试网
   */
  constructor(isTestnet: boolean) {
    this.isTestnet = isTestnet
  }

  /**
   * 注册远程请求函数。
   *
   * 由 `nrcs-client.ts` 在模块初始化时调用，将支持 `remoteNode` 选项的请求函数注入本管理器。
   * 未注册时 `addBootstrapNode` / `addBootstrapNodes` / `findMoreNodes` 将无法发起请求。
   *
   * @param fn - 远程请求函数
   */
  setRemoteTransport(fn: RemoteNodeRequestFn): void {
    this.remoteTransport = fn
  }

  /**
   * 获取 bootstrap 节点数据。
   *
   * 对标参考 `RemoteNodesManager.prototype.REMOTE_NODES_BOOTSTRAP`（由 `remotenodesbootstrap.*.js` 赋值）。
   */
  getBootstrapData(): PeerData[] {
    return (this.isTestnet ? testnetBootstrap : mainnetBootstrap) as unknown as PeerData[]
  }

  /**
   * 从 getPeers 响应批量添加远程节点。
   *
   * 对标 `util/remotenodesmgr.js:77` 的 `addRemoteNodes`。
   * 仅添加可连接且非旧版本的节点；保留已存在节点的黑名单状态。
   *
   * @param peersData - getPeers 响应中的 peers 数组
   */
  addRemoteNodes(peersData: PeerData[]): void {
    for (const peerData of peersData) {
      if (!isRemoteNodeConnectable(peerData, false)) continue
      const oldNode = this.nodes[peerData.address]
      const newNode = new RemoteNode(peerData)
      if (oldNode) {
        newNode.blacklistedUntil = oldNode.blacklistedUntil
      }
      this.nodes[peerData.address] = newNode
      console.log(`[remote-nodes] Found remote node ${peerData.address} blacklisted ${newNode.isBlacklisted()}`)
    }
  }

  /**
   * 连接用户手动配置的单个 bootstrap 节点。
   *
   * 对标 `util/remotenodesmgr.js:92` 的 `addBootstrapNode`。
   * 连接失败时回退到 `addBootstrapNodes`（随机 bootstrap）。
   *
   * @returns Promise，连接成功 resolve，失败 reject
   */
  async addBootstrapNode(): Promise<void> {
    const mobileSettings = getMobileSettings()
    const node = new RemoteNode({
      address: mobileSettings.remote_node_address,
      announcedAddress: mobileSettings.remote_node_address,
      apiPort: mobileSettings.remote_node_port,
      isSsl: mobileSettings.is_remote_node_ssl,
    })
    if (!this.remoteTransport) {
      console.log('[remote-nodes] Remote transport not registered, skip addBootstrapNode')
      return
    }
    console.log(`[remote-nodes] Connecting to configured address ${node.address} on port ${node.port} using ssl ${node.isSsl}`)
    try {
      const response: any = await this.remoteTransport('getBlockchainStatus', {}, {
        noProxy: true,
        remoteNode: node,
      } as RemoteRequestOptions)
      if (
        (response.blockchainState && response.blockchainState !== 'UP_TO_DATE') ||
        response.isDownloading
      ) {
        console.log(`[remote-nodes] Warning: Bootstrap node blockchain state is ${response.blockchainState}`)
      }
      if (response.errorCode || !isRemoteNodeConnectable(response, true)) {
        if (response.errorCode) {
          console.log(`[remote-nodes] Bootstrap node cannot be used ${response.errorDescription}`)
        } else {
          console.log('[remote-nodes] Bootstrap node does not provide the required services')
        }
        console.log('[remote-nodes] Cannot connect to configured node, connecting to a random node')
        return this.addBootstrapNodes()
      }
      console.log(`[remote-nodes] Adding bootstrap node ${node.address}`)
      this.nodes[node.address] = node
    } catch (e: any) {
      console.log(`[remote-nodes] Bootstrap node error: ${e?.message || e}`)
      return this.addBootstrapNodes()
    }
  }

  /**
   * 从内置 bootstrap 列表批量连接节点。
   *
   * 对标 `util/remotenodesmgr.js:122` 的 `addBootstrapNodes`。
   * 分批并发连接（每批 3*target 个），达到 target 个成功后 resolve；
   * 全部尝试完仍未达标则 reject。
   *
   * @returns Promise，达标 resolve，未达标 reject
   */
  async addBootstrapNodes(): Promise<void> {
    if (!isRemoteNodeConnectionAllowed()) {
      console.log('[remote-nodes] HTTPS client cannot connect to remote nodes')
      return Promise.reject(new Error('https_client_cannot_connect_remote_nodes'))
    }
    if (!this.remoteTransport) {
      console.log('[remote-nodes] Remote transport not registered, skip addBootstrapNodes')
      return Promise.reject(new Error('remote_transport_not_registered'))
    }

    const peersData = shuffle([...this.getBootstrapData()])
    const mobileSettings = getMobileSettings()
    this.bc.target = mobileSettings.is_testnet ? 2 : mobileSettings.bootstrap_nodes_count
    const batchSize = Math.min(peersData.length, 3 * this.bc.target)

    return new Promise<void>((resolve, reject) => {
      // 对标 `util/remotenodesmgr.js:140` 的 checkCounters
      const checkCounters = (): boolean => {
        if (this.bc.bootstrapComplete) {
          console.log('[remote-nodes] Ignore: bootstrap is complete')
          return false
        }
        if (this.bc.success >= this.bc.target) {
          console.log(`[remote-nodes] Resolve: found ${this.bc.target} nodes, start client`)
          resolve()
          this.bc.bootstrapComplete = true
          return false
        }
        if (this.bc.counter >= peersData.length) {
          console.log(
            `[remote-nodes] Connection failed, connected only to ${this.bc.success} nodes in ${this.bc.counter} attempts. Target is ${this.bc.target}`,
          )
          reject(new Error('bootstrap_failed'))
          this.bc.bootstrapComplete = true
          return false
        }
        return this.bc.counter === this.bc.index
      }

      // 对标 `util/remotenodesmgr.js:160` 的 startNextBatch
      const startNextBatch = (): void => {
        const batch: RemoteNode[] = []
        for (; this.bc.index < peersData.length && batch.length < batchSize; this.bc.index++) {
          const peerData = peersData[this.bc.index]
          if (!isRemoteNodeConnectable(peerData, false)) {
            console.log(
              `[remote-nodes] Reject: bootstrap node ${peerData.address} required services not available` +
                (peerData.services ? `, node services ${peerData.services}` : ''),
            )
            this.bc.counter++
            this.bc.fail++
            continue
          }
          const node = new RemoteNode(peerData, true)
          if (!node.port) {
            console.log(`[remote-nodes] Reject: bootstrap node ${node.address}, api port undefined`)
            this.bc.counter++
            this.bc.fail++
            continue
          }
          batch.push(node)
        }

        for (const node of batch) {
          console.log(`[remote-nodes] Connecting to bootstrap node ${node.address} port ${node.port}`)
          this.remoteTransport!('getBlockchainStatus', {}, {
            noProxy: true,
            remoteNode: node,
            timeout: 5000,
          } as RemoteRequestOptions)
            .then((response: any) => {
              this.bc.counter++
              if (response.errorCode) {
                console.log(`[remote-nodes] Reject: bootstrap node returned error ${response.errorDescription}`)
                this.bc.fail++
                if (checkCounters()) startNextBatch()
                return
              }
              if (
                (response.blockchainState && response.blockchainState !== 'UP_TO_DATE') ||
                response.isDownloading
              ) {
                console.log(`[remote-nodes] Reject: bootstrap node ${node.address} blockchain state is ${response.blockchainState}`)
                this.bc.fail++
                if (checkCounters()) startNextBatch()
                return
              }
              if (!isRemoteNodeConnectable(response, false)) {
                console.log(`[remote-nodes] Reject: bootstrap node ${node.address} required service not available, node services ${node.services}`)
                this.bc.fail++
                if (checkCounters()) startNextBatch()
                return
              }
              const elapsed = Date.now() - node.connectionTime.getTime()
              console.log(`[remote-nodes] Accept: adding bootstrap node ${node.address} response time ${elapsed} ms`)
              this.nodes[node.address] = node
              this.bc.success++
              if (checkCounters()) startNextBatch()
            })
            .catch((err: any) => {
              this.bc.counter++
              console.log(`[remote-nodes] Reject: bootstrap node ${node.address} request error ${err?.message || err}`)
              this.bc.fail++
              if (checkCounters()) startNextBatch()
            })
        }
      }

      startNextBatch()
    })
  }

  /**
   * 随机选取一个非黑名单节点。
   *
   * 对标 `util/remotenodesmgr.js:225` 的 `getRandomNode`。
   *
   * @param ignoredAddresses - 需跳过的地址列表
   * @returns 可用节点，或 null
   */
  getRandomNode(ignoredAddresses: string[] = []): RemoteNode | null {
    const addresses = Object.keys(this.nodes)
    if (addresses.length === 0) {
      console.log('[remote-nodes] Cannot get random node. No nodes available')
      return null
    }
    let index = Math.floor(Math.random() * addresses.length)
    const startIndex = index
    let node: RemoteNode | null = null
    do {
      const address = addresses[index]
      if (ignoredAddresses.indexOf(address) >= 0) {
        node = null
      } else {
        node = this.nodes[address]
        if (node && node.isBlacklisted()) {
          node = null
        }
      }
      index = (index + 1) % addresses.length
    } while (node === null && index !== startIndex)
    return node
  }

  /**
   * 随机选取多个互不重复的非黑名单节点。
   *
   * 对标 `util/remotenodesmgr.js:250` 的 `getRandomNodes`。
   * 注意参考实现中 `processedAddresses.concat(ignoredAddresses)` 是 bug（concat 返回新数组未赋值），
   * 此处修正为 push 已选地址。
   *
   * @param count - 需要的节点数
   * @param ignoredAddresses - 需跳过的地址列表
   * @returns 节点数组（可能少于 count）
   */
  getRandomNodes(count: number, ignoredAddresses: string[] = []): RemoteNode[] {
    const processedAddresses = [...ignoredAddresses]
    const result: RemoteNode[] = []
    for (let i = 0; i < count; i++) {
      const node = this.getRandomNode(processedAddresses)
      if (node) {
        processedAddresses.push(node.address)
        result.push(node)
      }
    }
    return result
  }

  /**
   * 通过已知节点发现更多节点。
   *
   * 对标 `util/remotenodesmgr.js:267` 的 `findMoreNodes`。
   * 随机选一个已知节点，调用 getPeers 拉取其连接的 peers，合并到 nodes。
   * 支持周期性重调度（isReschedule=true 时 30s 后再次执行）。
   *
   * @param isReschedule - 是否周期性重调度
   */
  findMoreNodes(isReschedule = false): void {
    if (!this.remoteTransport) return
    const node = this.getRandomNode()
    if (!node) return
    this.remoteTransport('getPeers', { state: 'CONNECTED', includePeerInfo: true }, {
      noProxy: true,
      remoteNode: node,
    } as RemoteRequestOptions)
      .then((response: any) => {
        if (response.peers) {
          this.addRemoteNodes(response.peers as PeerData[])
        }
        if (isReschedule) {
          setTimeout(() => this.findMoreNodes(true), 30000)
        }
      })
      .catch((err: any) => {
        console.log(`[remote-nodes] findMoreNodes error: ${err?.message || err}`)
      })
  }
}

/**
 * 当前管理器单例（对标全局 `NRS.remoteNodesMgr`）。
 * 由 `initRemoteNodesMgr` 创建，供 feature-detection 与 response-confirmation 共享。
 */
let currentManager: RemoteNodesManager | null = null

/**
 * 获取当前远程节点管理器单例。
 *
 * @returns 当前管理器，未初始化返回 null
 */
export function getRemoteNodesManager(): RemoteNodesManager | null {
  return currentManager
}

/**
 * 设置当前远程节点管理器单例（供 init 与测试使用）。
 *
 * @param mgr - 管理器实例
 */
export function setRemoteNodesManager(mgr: RemoteNodesManager | null): void {
  currentManager = mgr
}

/**
 * 兼容 feature-detection 的 getter：随机返回一个可用节点句柄。
 *
 * 对标 `nrs.feature.detection.js:105` 的 `NRS.getRemoteNodeUrl` 中
 * `NRS.remoteNodesMgr.getRandomNode()` 的调用。
 * 管理器未初始化时返回 null。
 */
export function pickRandomRemoteNode(): RemoteNodeHandle | null {
  if (!currentManager) return null
  return currentManager.getRandomNode()
}
