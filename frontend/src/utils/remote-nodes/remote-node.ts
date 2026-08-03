/**
 * RemoteNode —— 单个远程节点的抽象。
 *
 * 端口自参考实现 `util/remotenodesmgr.js:17-39` 的 `RemoteNode` 构造函数与原型方法。
 *
 * 每个 RemoteNode 封装一个远端 API 节点的连接信息与黑名单状态，
 * 提供 `getUrl()` / `isBlacklisted()` / `blacklist()` 三个核心方法。
 * feature-detection 的 `RemoteNodeHandle` 接口由本类实现。
 */
import type { PeerData, RemoteNodeHandle } from './types'

/** 黑名单时长（毫秒，对标参考实现的 10 分钟） */
const BLACKLIST_DURATION_MS = 10 * 60 * 1000

/**
 * 远程节点。
 *
 * 对标 `util/remotenodesmgr.js:17` 的 `function RemoteNode(peerData, useAnnouncedAddress)`。
 *
 * 适配说明：参考使用原型链 + 全局 `NRS.logConsole`，本实现改为类方法 + 注入 logger，
 * 避免对全局 NRS 的依赖。
 */
export class RemoteNode implements RemoteNodeHandle {
  /** 节点 IP 或主机名 */
  public address: string
  /** 对外宣告地址 */
  public announcedAddress: string
  /** API 端口 */
  public port: number
  /** 是否使用 SSL */
  public isSsl: boolean
  /** 是否优先使用 announcedAddress 构造 URL（bootstrap 节点为 true） */
  public useAnnouncedAddress: boolean
  /** 服务列表（API/CORS/HALLMARK 等） */
  public services: string[]
  /** 软件版本 */
  public version: string
  /** 黑名单到期时间戳（毫秒），0 表示未拉黑 */
  public blacklistedUntil: number
  /** 连接建立时间（用于统计响应耗时） */
  public connectionTime: Date

  /**
   * 构造远程节点。
   *
   * @param peerData - peer 原始数据（来自 getPeers 或 bootstrap 列表）
   * @param useAnnouncedAddress - 是否优先使用 announcedAddress 构造 URL，默认 false
   */
  constructor(peerData: PeerData, useAnnouncedAddress = false) {
    this.address = peerData.address
    this.announcedAddress = peerData.announcedAddress || peerData.address
    this.port = peerData.apiPort || 0
    this.isSsl = !!peerData.isSsl
    this.useAnnouncedAddress = useAnnouncedAddress
    this.services = peerData.services || []
    this.version = peerData.version || ''
    this.blacklistedUntil = 0
    this.connectionTime = new Date()
  }

  /**
   * 获取节点完整 URL。
   *
   * 对标 `util/remotenodesmgr.js:27` 的 `RemoteNode.prototype.getUrl`。
   *
   * @returns 形如 `http://1.2.3.4:7876` 的 URL
   */
  getUrl(): string {
    const protocol = this.isSsl ? 'https://' : 'http://'
    const host = this.useAnnouncedAddress ? this.announcedAddress : this.address
    return `${protocol}${host}:${this.port}`
  }

  /**
   * 是否已被黑名单。
   *
   * 对标 `util/remotenodesmgr.js:31` 的 `RemoteNode.prototype.isBlacklisted`。
   *
   * @returns 当前时间在黑名单到期前返回 true
   */
  isBlacklisted(): boolean {
    return Date.now() < this.blacklistedUntil
  }

  /**
   * 将节点加入黑名单（10 分钟）。
   *
   * 对标 `util/remotenodesmgr.js:35` 的 `RemoteNode.prototype.blacklist`。
   */
  blacklist(): void {
    this.blacklistedUntil = Date.now() + BLACKLIST_DURATION_MS
    // 对标参考的 NRS.logConsole，此处用 console.log 保持简洁
    console.log(`[remote-nodes] Blacklist ${this.address} until ${new Date(this.blacklistedUntil).toISOString()}`)
  }
}
