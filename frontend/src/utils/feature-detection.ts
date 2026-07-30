/**
 * NRCS 特性检测模块。
 *
 * 端口自参考实现 `nrs.feature.detection.js`（253 行）。
 * 提供 30+ 个运行时环境特性检测函数，影响 forging / shuffling / 远程节点 / 移动端等运行时决策。
 *
 * 适配说明（Vue3 与参考的差异）：
 * - 参考使用全局 `NRS.state` / `NRS.settings` / `NRS.mobileSettings` / `NRS.isTestNet`，
 *   本模块改用可设置的 `FeatureContext`（`setFeatureContext`），由 app 在状态加载后注入，
 *   避免与 Pinia store 循环依赖。
 * - `window.java`（JavaFX 桥接）与 `isNode`（Node 服务端包装）分支不适用于 Vue3 Web 端，已省略。
 * - `getDownloadLink` 不再返回 HTML 字符串或操作 jQuery，改为返回 `{ href, external }` 描述符，
 *   由 Vue 组件自行渲染。
 * - 远程节点管理器（`remoteNodesMgr`）在阶段 5.4 实现，本模块通过 `registerRemoteNodesMgr`
 *   预留注入点；未注册时 `getRemoteNodeUrl` 返回空字符串。
 */

// ============================================================================
// 静态环境标识（模块加载时计算一次）
// ============================================================================

/** 是否为 JavaFX 桌面应用（通过 User-Agent 检测） */
const isDesktopApplication =
  typeof navigator !== 'undefined' && navigator.userAgent.indexOf('JavaFX') >= 0

/** 是否原生支持 Promise */
const isPromiseSupported =
  typeof Promise !== 'undefined' && Promise.toString().indexOf('[native code]') !== -1

/** 是否为 Cordova 移动设备 */
const isMobileDevice =
  typeof window !== 'undefined' && (window as unknown as { cordova?: unknown }).cordova !== undefined

/** 页面是否通过 HTTPS 加载 */
const isLoadedOverHttps =
  typeof window !== 'undefined' && window.location?.protocol === 'https:'

// ============================================================================
// 运行时上下文（由 app 在 getState / settings 加载后注入）
// ============================================================================

/** 远程节点句柄接口（阶段 5.4 remote-nodes-mgr 实现后注入） */
export interface RemoteNodeHandle {
  /** 获取远程节点 URL */
  getUrl(): string
  /** 将该节点加入黑名单 */
  blacklist(): void
}

/** 特性检测运行时上下文 */
export interface FeatureContext {
  /** 节点是否为 API 代理（来自 getState 响应的 apiProxy 字段） */
  apiProxy?: boolean
  /** 是否测试网 */
  isTestNet?: boolean
  /** 移动端设置（含 is_simulate_app 模拟开关） */
  mobileSettings?: { is_simulate_app?: boolean }
  /** 用户设置（含交易所 URL、管理员密码等） */
  settings?: {
    shape_shift_url?: string
    changelly_url?: string
    admin_password?: string
  }
}

/** 当前运行时上下文 */
let featureContext: FeatureContext = {}

/** 当前远程节点句柄 */
let remoteNode: RemoteNodeHandle | null = null

/** 远程节点管理器获取函数（阶段 5.4 注入） */
let remoteNodesMgrGetter: (() => RemoteNodeHandle | null) | null = null

// ============================================================================
// 上下文与注册 API
// ============================================================================

/**
 * 设置特性检测运行时上下文。
 * App 在 `getState` / 设置加载完成后调用，将 apiProxy / isTestNet / settings 注入。
 *
 * @param ctx - 运行时上下文（部分字段可省略，仅更新传入字段）
 */
export function setFeatureContext(ctx: FeatureContext): void {
  featureContext = { ...featureContext, ...ctx }
}

/**
 * 获取当前运行时上下文（主要用于测试与调试）。
 */
export function getFeatureContext(): FeatureContext {
  return featureContext
}

/**
 * 重置运行时上下文与远程节点状态（登出 / 切换网络时调用）。
 */
export function resetFeatureContext(): void {
  featureContext = {}
  remoteNode = null
}

/**
 * 注册远程节点管理器获取函数（阶段 5.4 `remote-nodes-mgr` 实现后调用）。
 *
 * @param getter - 返回一个随机可用远程节点句柄，或 null
 */
export function registerRemoteNodesMgr(getter: () => RemoteNodeHandle | null): void {
  remoteNodesMgrGetter = getter
}

// ============================================================================
// 基础检测函数（端口自 nrs.feature.detection.js）
// ============================================================================

/**
 * 判断 IP 是否为私有地址（10.x / 127.x / 172.16-31.x / 192.168.x）。
 * 端口自 `nrs.feature.detection.js:28` 的 `NRS.isPrivateIP`。
 *
 * @param ip - IPv4 地址字符串
 * @returns 是私有 IP 返回 true
 */
export function isPrivateIP(ip: string): boolean {
  if (!/^\d+\.\d+\.\d+\.\d+$/.test(ip)) {
    return false
  }
  const parts = ip.split('.')
  const p0 = parts[0]
  const p1 = parseInt(parts[1], 10)
  return (
    p0 === '10' ||
    p0 === '127' ||
    (p0 === '172' && p1 >= 16 && p1 <= 31) ||
    (p0 === '192' && p1 === 168)
  )
}

/**
 * 判断当前页面是否加载在本地主机（localhost / 127.0.0.1 / 私有 IP）。
 * 模块加载时计算一次，导出为函数以保持与参考一致的调用形式。
 */
function computeIsLocalHost(): boolean {
  if (typeof window === 'undefined' || !window.location?.hostname) {
    return false
  }
  const hostName = window.location.hostname.toLowerCase()
  return hostName === 'localhost' || hostName === '127.0.0.1' || isPrivateIP(hostName)
}

/** 当前是否为本地主机 */
const isLocalHost = computeIsLocalHost()

/**
 * 浏览器是否支持 IndexedDB。
 * 端口自 `nrs.feature.detection.js:41` 的 `NRS.isIndexedDBSupported`。
 */
export function isIndexedDBSupported(): boolean {
  return typeof window !== 'undefined' && window.indexedDB !== undefined
}

/**
 * 是否显示外部链接。
 * 端口自 `nrs.feature.detection.js:45`：移动端不显示；JavaFX 桌面端在 Linux 上不显示。
 */
export function isExternalLinkVisible(): boolean {
  if (isMobileApp()) {
    return false
  }
  return !(isDesktopApplication && navigator.userAgent.indexOf('Linux') >= 0)
}

/**
 * 是否显示 Web 钱包链接（仅 JavaFX 桌面端且非 Linux 显示）。
 * 端口自 `nrs.feature.detection.js:53`。
 */
export function isWebWalletLinkVisible(): boolean {
  if (isMobileApp()) {
    return false
  }
  return isDesktopApplication && navigator.userAgent.indexOf('Linux') === -1
}

/**
 * 是否为移动端应用（Cordova 设备或开启了移动端模拟）。
 * 端口自 `nrs.feature.detection.js:60` 的 `NRS.isMobileApp`。
 */
export function isMobileApp(): boolean {
  return isMobileDevice || !!featureContext.mobileSettings?.is_simulate_app
}

/**
 * 是否允许启用移动端模拟（非真实移动设备时才允许）。
 * 端口自 `nrs.feature.detection.js:64`。
 */
export function isEnableMobileAppSimulation(): boolean {
  return !isMobileDevice
}

/**
 * 是否需要 CORS（非移动设备时需要）。
 * 端口自 `nrs.feature.detection.js:68`。
 */
export function isRequireCors(): boolean {
  return !isMobileDevice
}

/**
 * 是否轮询 getState（JavaFX 桌面端仅在作为代理时轮询）。
 * 端口自 `nrs.feature.detection.js:72`。
 */
export function isPollGetState(): boolean {
  return !isDesktopApplication || !!featureContext.apiProxy
}

/**
 * 是否更新远程节点列表（仅作为 API 代理时）。
 * 端口自 `nrs.feature.detection.js:77`。
 */
export function isUpdateRemoteNodes(): boolean {
  return !!featureContext.apiProxy
}

/**
 * 是否允许连接远程节点（HTTPS 页面禁止连接 HTTP 节点，避免混合内容错误）。
 * 端口自 `nrs.feature.detection.js:81`。
 */
export function isRemoteNodeConnectionAllowed(): boolean {
  return !isLoadedOverHttps
}

/**
 * 是否支持联系人导出（JavaFX 桌面端不支持）。
 * 端口自 `nrs.feature.detection.js:89`。
 */
export function isExportContactsAvailable(): boolean {
  return !isDesktopApplication
}

/**
 * 是否支持文件加密（JavaFX 桌面端无法读取文件）。
 * 端口自 `nrs.feature.detection.js:93`。
 */
export function isFileEncryptionSupported(): boolean {
  return !isDesktopApplication
}

/**
 * 是否显示占位 checkbox（仅 JavaFX 桌面端 Linux 上修正渲染问题）。
 * 端口自 `nrs.feature.detection.js:97`。
 */
export function isShowDummyCheckbox(): boolean {
  return isDesktopApplication && navigator.userAgent.indexOf('Linux') >= 0
}

/**
 * 是否解码节点 hallmark（需原生 Promise 支持）。
 * 端口自 `nrs.feature.detection.js:101`。
 */
export function isDecodePeerHallmark(): boolean {
  return isPromiseSupported
}

// ============================================================================
// 远程节点管理（阶段 5.4 remote-nodes-mgr 实现后完整可用）
// ============================================================================

/**
 * 获取远程节点 URL。
 * 端口自 `nrs.feature.detection.js:105` 的 `NRS.getRemoteNodeUrl`。
 *
 * - 非移动端：Web 端返回空字符串（直连本地节点）
 * - 移动端：从远程节点管理器随机取一个节点；无可用节点时返回空字符串
 *
 * @returns 远程节点 URL，或空字符串
 */
export function getRemoteNodeUrl(): string {
  if (!isMobileApp()) {
    // Web 端直连本地节点，无远程节点
    return ''
  }
  if (remoteNode) {
    return remoteNode.getUrl()
  }
  if (remoteNodesMgrGetter) {
    remoteNode = remoteNodesMgrGetter()
  }
  if (remoteNode) {
    const url = remoteNode.getUrl()
    console.log('[feature-detection] Remote node url: ' + url)
    return url
  }
  console.log('[feature-detection] No available remote nodes')
  return ''
}

/**
 * 获取当前远程节点句柄。
 * 端口自 `nrs.feature.detection.js:126`。
 */
export function getRemoteNode(): RemoteNodeHandle | null {
  return remoteNode
}

/**
 * 重置远程节点（可选加入黑名单）。
 * 端口自 `nrs.feature.detection.js:130`。
 *
 * @param blacklist - 是否将当前节点加入黑名单
 */
export function resetRemoteNode(blacklist = false): void {
  if (remoteNode && blacklist) {
    remoteNode.blacklist()
  }
  remoteNode = null
}

// ============================================================================
// 下载与浏览器跳转
// ============================================================================

/**
 * 下载链接描述符（替代参考中返回 HTML 字符串的实现）。
 * 移动端通过外部浏览器打开，桌面端直接使用 href。
 */
export interface DownloadLinkDescriptor {
  /** 链接地址 */
  href: string
  /** 是否需要在外部浏览器打开（移动端） */
  external: boolean
}

/**
 * 获取下载链接描述符。
 * 端口自 `nrs.feature.detection.js:137` 的 `NRS.getDownloadLink`，
 * 改为返回结构化描述符供 Vue 组件渲染。
 *
 * @param url - 下载地址
 * @returns `{ href, external }`
 */
export function getDownloadLink(url: string): DownloadLinkDescriptor {
  return {
    href: url,
    external: isMobileApp()
  }
}

/**
 * 在移动端外部浏览器打开 URL（Cordova InAppBrowser）。
 * 端口自 `nrs.feature.detection.js:154`。
 *
 * @param url - 要打开的 URL
 */
export function openMobileBrowser(url: string): void {
  try {
    const cordova = (window as unknown as {
      cordova?: { InAppBrowser?: { open: (url: string, target: string) => void } }
    }).cordova
    cordova?.InAppBrowser?.open(url, '_system')
  } catch (e) {
    console.log('[feature-detection] openMobileBrowser error: ' + (e as Error).message)
  }
}

// ============================================================================
// 扫码与相机
// ============================================================================

/**
 * 是否启用 Cordova 扫码（仅移动设备）。
 * 端口自 `nrs.feature.detection.js:163`。
 */
export function isCordovaScanningEnabled(): boolean {
  return isMobileDevice
}

/**
 * 是否允许扫码（移动设备 / 本地主机 / 测试网）。
 * 端口自 `nrs.feature.detection.js:167`。
 */
export function isScanningAllowed(): boolean {
  return isMobileDevice || isLocalHost || !!featureContext.isTestNet
}

/**
 * 是否需要相机权限（Android 6.0+）。
 * 端口自 `nrs.feature.detection.js:171`。
 * 依赖 Cordova device 插件，非移动端返回 false。
 */
export function isCameraPermissionRequired(): boolean {
  const device = (window as unknown as {
    device?: { platform?: string; version?: string }
  }).device
  return !!device && device.platform === 'Android' && !!device.version && device.version >= '6.0.0'
}

// ============================================================================
// 交易所 URL
// ============================================================================

/**
 * 获取 ShapeShift 交易所 URL。
 * 端口自 `nrs.feature.detection.js:175`。
 */
export function getShapeShiftUrl(): string {
  return featureContext.settings?.shape_shift_url || ''
}

/**
 * 获取 Changelly 交易所 URL。
 * 端口自 `nrs.feature.detection.js:179`。
 */
export function getChangellyUrl(): string {
  return featureContext.settings?.changelly_url || ''
}

// ============================================================================
// 功能支持判定（forging / shuffling / funding monitor）
// ============================================================================

/**
 * 是否支持 forging（非移动端且非 API 代理）。
 * 端口自 `nrs.feature.detection.js:183`。
 */
export function isForgingSupported(): boolean {
  return !isMobileApp() && !featureContext.apiProxy
}

/**
 * 是否支持 funding monitor（非移动端且非 API 代理）。
 * 端口自 `nrs.feature.detection.js:187`。
 */
export function isFundingMonitorSupported(): boolean {
  return !isMobileApp() && !featureContext.apiProxy
}

/**
 * 是否支持 shuffling（非移动端且非 API 代理）。
 * 端口自 `nrs.feature.detection.js:191`。
 */
export function isShufflingSupported(): boolean {
  return !isMobileApp() && !featureContext.apiProxy
}

/**
 * 是否需要确认响应（移动端或 API 代理时，因远程节点不可信需多节点交叉验证）。
 * 端口自 `nrs.feature.detection.js:195`。
 */
export function isConfirmResponse(): boolean {
  return isMobileApp() || !!featureContext.apiProxy
}

// ============================================================================
// UI 显示判定
// ============================================================================

/**
 * 是否显示仪表盘可选磁贴（非移动端）。
 * 端口自 `nrs.feature.detection.js:199`。
 */
export function isDisplayOptionalDashboardTiles(): boolean {
  return !isMobileApp()
}

/**
 * 是否显示客户端选项链接（移动端或 API 代理）。
 * 端口自 `nrs.feature.detection.js:203`。
 */
export function isShowClientOptionsLink(): boolean {
  return isMobileApp() || !!featureContext.apiProxy
}

/**
 * 获取生成器计时准确性警告的 i18n key。
 * 端口自 `nrs.feature.detection.js:207`：桌面端返回空字符串，否则返回 i18n key。
 *
 * @returns i18n key `generator_timing_accuracy_warning`，或空字符串
 */
export function getGeneratorAccuracyWarning(): string {
  if (isDesktopApplication) {
    return ''
  }
  return 'generator_timing_accuracy_warning'
}

/**
 * 是否初始化插件（非移动端）。
 * 端口自 `nrs.feature.detection.js:214`。
 */
export function isInitializePlugins(): boolean {
  return !isMobileApp()
}

/**
 * 是否显示远程节点警告（非本地主机）。
 * 端口自 `nrs.feature.detection.js:218`。
 */
export function isShowRemoteWarning(): boolean {
  return !isLocalHost
}

// ============================================================================
// 安全判定
// ============================================================================

/**
 * forging 是否安全（仅本地主机安全）。
 * 端口自 `nrs.feature.detection.js:222`。
 */
export function isForgingSafe(): boolean {
  return isLocalHost
}

/**
 * secretPhrase 是否处于风险中（非本地、或 API 代理、或移动端）。
 * 端口自 `nrs.feature.detection.js:226`。
 */
export function isPassphraseAtRisk(): boolean {
  return !isLocalHost || !!featureContext.apiProxy || isMobileApp()
}

// ============================================================================
// 其他
// ============================================================================

/**
 * 是否支持窗口打印（非桌面、非移动）。
 * 端口自 `nrs.feature.detection.js:230`。
 */
export function isWindowPrintSupported(): boolean {
  return !isDesktopApplication && !isMobileDevice
}

/**
 * 是否禁用定时交易请求（移动端或 API 代理）。
 * 端口自 `nrs.feature.detection.js:234`。
 */
export function isDisableScheduleRequest(): boolean {
  return isMobileApp() || !!featureContext.apiProxy
}

/**
 * 获取管理员密码。
 * 端口自 `nrs.feature.detection.js:238`：Web 端从用户设置读取（JavaFX / Node 分支不适用）。
 *
 * @returns 管理员密码，未设置返回空字符串
 */
export function getAdminPassword(): string {
  return featureContext.settings?.admin_password || ''
}

// ============================================================================
// 调试导出（仅供测试与日志，不在业务代码中使用）
// ============================================================================

/**
 * 暴露静态环境标识（仅供测试与调试）。
 * 业务代码应使用上述检测函数，而非直接读取这些原始标识。
 */
export const __envFlags = {
  isDesktopApplication,
  isMobileDevice,
  isLocalHost,
  isLoadedOverHttps,
  isPromiseSupported
}
