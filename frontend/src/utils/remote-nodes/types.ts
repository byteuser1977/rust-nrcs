/**
 * 远程节点模块类型定义。
 *
 * 端口自参考实现 `util/remotenodesmgr.js` 与 `nrs.remote.nodes.js` 中隐式使用的 peer 数据结构。
 * NRCS 节点通过 `getPeers` 返回的 peer 对象，以及内置的 bootstrap 节点列表，均遵循此结构。
 */

/**
 * Peer / 远程节点原始数据结构。
 *
 * 对标 `getPeers` 响应中的单个 peer 对象，以及 `remotenodesbootstrap.*.js` 中的节点条目。
 * 字段命名与 NRCS 后端响应保持一致（camelCase）。
 */
export interface PeerData {
  /** 节点 IP 或主机名 */
  address: string
  /** 节点对外宣告的地址（通常等于 address，也可能是域名） */
  announcedAddress?: string
  /** API 端口 */
  apiPort?: number
  /** 是否使用 SSL（仅用户手动配置的节点允许 SSL，需可信证书） */
  isSsl?: boolean
  /** 节点提供的服务列表（HALLMARK/API/CORS/API_SSL 等） */
  services?: string[]
  /** 节点软件版本（格式 x.y.z） */
  version?: string
  /** 区块链状态（UP_TO_DATE/UPLOADING/DOWNLOADING 等） */
  blockchainState?: string
  /** 是否正在下载区块 */
  isDownloading?: boolean
  /** 连接状态（1=CONNECTED） */
  state?: number
  /** 平台标识 */
  platform?: string
  /** 应用名（NRS） */
  application?: string
  /** P2P 端口 */
  port?: number
  /** 是否分享地址 */
  shareAddress?: boolean
  /** Hallmark 签名（hex） */
  hallmark?: string
  /** 权重 */
  weight?: number
  /** 下行流量 */
  downloadedVolume?: number
  /** 上行流量 */
  uploadedVolume?: number
  /** 是否入站连接 */
  inbound?: boolean
  /** 是否入站 WebSocket */
  inboundWebSocket?: boolean
  /** 是否出站 WebSocket */
  outboundWebSocket?: boolean
  /** 最后更新时间（区块时间戳） */
  lastUpdated?: number
  /** 是否被黑名单 */
  blacklisted?: boolean
  /** 最后连接尝试时间 */
  lastConnectAttempt?: number
  /** 允许扩展字段 */
  [key: string]: any
}

/**
 * 远程节点请求选项。
 *
 * 对标参考 `nrs.server.js` 中 `NRS.sendRequest` 的 options 参数中与远程节点相关的字段。
 * 用于在 `nrcsGet`/`nrcsPost` 中支持远程节点转发与代理跳过。
 */
export interface RemoteRequestOptions {
  /** 指定远程节点发起请求（绕过本地节点） */
  remoteNode?: RemoteNodeHandle
  /** 是否跳过本地代理（直接请求 remoteNode 或本地节点） */
  noProxy?: boolean
  /** 请求超时（毫秒） */
  timeout?: number
  /** 是否对参数做 URL 编码（参考 doNotEscape，默认 false 即编码） */
  doNotEscape?: boolean
}

/**
 * 远程节点句柄接口（与 `feature-detection.ts` 的 RemoteNodeHandle 对齐）。
 *
 * 由 `RemoteNode` 类实现，供 feature-detection 注入。
 */
export interface RemoteNodeHandle {
  /** 节点地址（IP 或主机名） */
  address: string
  /** 对外宣告地址 */
  announcedAddress?: string
  /** API 端口 */
  port?: number
  /** 是否 SSL */
  isSsl?: boolean
  /** 获取节点完整 URL（含协议与端口） */
  getUrl(): string
  /** 是否已被黑名单 */
  isBlacklisted(): boolean
  /** 加入黑名单（默认 10 分钟） */
  blacklist(): void
}

/**
 * 远程节点请求函数类型。
 *
 * 由 `nrcs-client.ts` 实现 `remoteTransport` 并注入到本模块，
 * 使本模块不直接依赖 axios，便于测试与解耦。
 *
 * @param requestType - NRCS requestType
 * @param data - 请求参数
 * @param options - 远程节点请求选项
 * @returns 响应数据（已剥离 errorCode 包装）
 */
export type RemoteNodeRequestFn = (
  requestType: string,
  data: Record<string, any>,
  options?: RemoteRequestOptions,
) => Promise<any>
