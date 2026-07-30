import { describe, it, expect, vi, beforeEach } from 'vitest'
import {
  isPrivateIP,
  isIndexedDBSupported,
  isExternalLinkVisible,
  isWebWalletLinkVisible,
  isMobileApp,
  isEnableMobileAppSimulation,
  isRequireCors,
  isPollGetState,
  isUpdateRemoteNodes,
  isRemoteNodeConnectionAllowed,
  isExportContactsAvailable,
  isFileEncryptionSupported,
  isShowDummyCheckbox,
  isDecodePeerHallmark,
  getRemoteNodeUrl,
  getRemoteNode,
  resetRemoteNode,
  getDownloadLink,
  openMobileBrowser,
  isCordovaScanningEnabled,
  isScanningAllowed,
  isCameraPermissionRequired,
  getShapeShiftUrl,
  getChangellyUrl,
  isForgingSupported,
  isFundingMonitorSupported,
  isShufflingSupported,
  isConfirmResponse,
  isDisplayOptionalDashboardTiles,
  isShowClientOptionsLink,
  getGeneratorAccuracyWarning,
  isInitializePlugins,
  isShowRemoteWarning,
  isForgingSafe,
  isPassphraseAtRisk,
  isWindowPrintSupported,
  isDisableScheduleRequest,
  getAdminPassword,
  setFeatureContext,
  getFeatureContext,
  resetFeatureContext,
  registerRemoteNodesMgr,
  type RemoteNodeHandle
} from '@/utils/feature-detection'

describe('feature-detection: isPrivateIP', () => {
  it('识别 10.x 私有地址', () => {
    expect(isPrivateIP('10.0.0.1')).toBe(true)
  })
  it('识别 127.x 回环地址', () => {
    expect(isPrivateIP('127.0.0.1')).toBe(true)
  })
  it('识别 172.16-31.x 私有地址', () => {
    expect(isPrivateIP('172.16.0.1')).toBe(true)
    expect(isPrivateIP('172.31.255.255')).toBe(true)
    expect(isPrivateIP('172.15.0.1')).toBe(false)
    expect(isPrivateIP('172.32.0.1')).toBe(false)
  })
  it('识别 192.168.x 私有地址', () => {
    expect(isPrivateIP('192.168.1.1')).toBe(true)
  })
  it('公网地址返回 false', () => {
    expect(isPrivateIP('8.8.8.8')).toBe(false)
  })
  it('非 IPv4 格式返回 false', () => {
    expect(isPrivateIP('localhost')).toBe(false)
    expect(isPrivateIP('::1')).toBe(false)
    expect(isPrivateIP('')).toBe(false)
  })
})

describe('feature-detection: 基础环境检测', () => {
  it('isIndexedDBSupported 检测 window.indexedDB 是否定义', () => {
    // jsdom 默认不提供 indexedDB，stub 后应返回 true
    vi.stubGlobal('indexedDB', {})
    expect(isIndexedDBSupported()).toBe(true)
    vi.unstubAllGlobals()
    expect(isIndexedDBSupported()).toBe(false)
  })
  it('isRemoteNodeConnectionAllowed 在 HTTP 页面返回 true', () => {
    // jsdom 默认 protocol 为 http:
    expect(isRemoteNodeConnectionAllowed()).toBe(true)
  })
  it('isEnableMobileAppSimulation 在非移动设备返回 true', () => {
    expect(isEnableMobileAppSimulation()).toBe(true)
  })
  it('isRequireCors 在非移动设备返回 true', () => {
    expect(isRequireCors()).toBe(true)
  })
  it('isDecodePeerHallmark 返回 true（Node 原生支持 Promise）', () => {
    expect(isDecodePeerHallmark()).toBe(true)
  })
  it('isCameraPermissionRequired 非 Cordova 返回 false', () => {
    expect(isCameraPermissionRequired()).toBe(false)
  })
  it('isCordovaScanningEnabled 非移动设备返回 false', () => {
    expect(isCordovaScanningEnabled()).toBe(false)
  })
  it('isWindowPrintSupported 非桌面非移动返回 true', () => {
    expect(isWindowPrintSupported()).toBe(true)
  })
  // jsdom hostname 为 localhost，故 isLocalHost=true
  it('isForgingSafe 本地主机返回 true', () => {
    expect(isForgingSafe()).toBe(true)
  })
  it('isShowRemoteWarning 本地主机返回 false', () => {
    expect(isShowRemoteWarning()).toBe(false)
  })
})

describe('feature-detection: 上下文相关功能支持', () => {
  beforeEach(() => {
    resetFeatureContext()
  })

  it('isMobileApp：默认非移动端，开启模拟后为移动端', () => {
    expect(isMobileApp()).toBe(false)
    setFeatureContext({ mobileSettings: { is_simulate_app: true } })
    expect(isMobileApp()).toBe(true)
  })

  it('isForgingSupported / isShufflingSupported / isFundingMonitorSupported：移动端或 API 代理时禁用', () => {
    expect(isForgingSupported()).toBe(true)
    expect(isShufflingSupported()).toBe(true)
    expect(isFundingMonitorSupported()).toBe(true)

    setFeatureContext({ apiProxy: true })
    expect(isForgingSupported()).toBe(false)
    expect(isShufflingSupported()).toBe(false)
    expect(isFundingMonitorSupported()).toBe(false)

    resetFeatureContext()
    setFeatureContext({ mobileSettings: { is_simulate_app: true } })
    expect(isForgingSupported()).toBe(false)
    expect(isShufflingSupported()).toBe(false)
  })

  it('isConfirmResponse：移动端或 API 代理时为 true', () => {
    expect(isConfirmResponse()).toBe(false)
    setFeatureContext({ apiProxy: true })
    expect(isConfirmResponse()).toBe(true)
    resetFeatureContext()
    setFeatureContext({ mobileSettings: { is_simulate_app: true } })
    expect(isConfirmResponse()).toBe(true)
  })

  it('isPollGetState：默认 true，API 代理时 true', () => {
    expect(isPollGetState()).toBe(true)
  })

  it('isUpdateRemoteNodes：仅 API 代理时 true', () => {
    expect(isUpdateRemoteNodes()).toBe(false)
    setFeatureContext({ apiProxy: true })
    expect(isUpdateRemoteNodes()).toBe(true)
  })

  it('isPassphraseAtRisk：本地+无代理+非移动 → false；任一条件触发 → true', () => {
    // jsdom localhost → isLocalHost=true
    expect(isPassphraseAtRisk()).toBe(false)
    setFeatureContext({ apiProxy: true })
    expect(isPassphraseAtRisk()).toBe(true)
    resetFeatureContext()
    setFeatureContext({ mobileSettings: { is_simulate_app: true } })
    expect(isPassphraseAtRisk()).toBe(true)
  })

  it('isDisableScheduleRequest：移动端或 API 代理时 true', () => {
    expect(isDisableScheduleRequest()).toBe(false)
    setFeatureContext({ apiProxy: true })
    expect(isDisableScheduleRequest()).toBe(true)
  })

  it('isDisplayOptionalDashboardTiles / isInitializePlugins：移动端禁用', () => {
    expect(isDisplayOptionalDashboardTiles()).toBe(true)
    expect(isInitializePlugins()).toBe(true)
    setFeatureContext({ mobileSettings: { is_simulate_app: true } })
    expect(isDisplayOptionalDashboardTiles()).toBe(false)
    expect(isInitializePlugins()).toBe(false)
  })

  it('isShowClientOptionsLink：移动端或 API 代理时显示', () => {
    expect(isShowClientOptionsLink()).toBe(false)
    setFeatureContext({ apiProxy: true })
    expect(isShowClientOptionsLink()).toBe(true)
  })

  it('isScanningAllowed：本地主机时允许', () => {
    expect(isScanningAllowed()).toBe(true)
    resetFeatureContext()
    setFeatureContext({ isTestNet: true })
    expect(isScanningAllowed()).toBe(true)
  })
})

describe('feature-detection: UI 链接与下载', () => {
  beforeEach(() => {
    resetFeatureContext()
  })

  it('isExternalLinkVisible / isWebWalletLinkVisible：移动端隐藏', () => {
    // 非桌面、非移动 → 外部链接可见，web 钱包链接不可见
    expect(isExternalLinkVisible()).toBe(true)
    expect(isWebWalletLinkVisible()).toBe(false)

    setFeatureContext({ mobileSettings: { is_simulate_app: true } })
    expect(isExternalLinkVisible()).toBe(false)
    expect(isWebWalletLinkVisible()).toBe(false)
  })

  it('isExportContactsAvailable / isFileEncryptionSupported：非桌面 true', () => {
    expect(isExportContactsAvailable()).toBe(true)
    expect(isFileEncryptionSupported()).toBe(true)
  })

  it('isShowDummyCheckbox：非桌面 false', () => {
    expect(isShowDummyCheckbox()).toBe(false)
  })

  it('getGeneratorAccuracyWarning：非桌面返回 i18n key', () => {
    expect(getGeneratorAccuracyWarning()).toBe('generator_timing_accuracy_warning')
  })

  it('getDownloadLink：桌面直接 href，移动端标记 external', () => {
    const desktop = getDownloadLink('https://example.com/file.zip')
    expect(desktop.href).toBe('https://example.com/file.zip')
    expect(desktop.external).toBe(false)

    setFeatureContext({ mobileSettings: { is_simulate_app: true } })
    const mobile = getDownloadLink('https://example.com/file.zip')
    expect(mobile.external).toBe(true)
  })

  it('getShapeShiftUrl / getChangellyUrl / getAdminPassword 从设置读取', () => {
    expect(getShapeShiftUrl()).toBe('')
    expect(getChangellyUrl()).toBe('')
    expect(getAdminPassword()).toBe('')

    setFeatureContext({
      settings: {
        shape_shift_url: 'https://shapeshift.io',
        changelly_url: 'https://changelly.com',
        admin_password: 'secret123'
      }
    })
    expect(getShapeShiftUrl()).toBe('https://shapeshift.io')
    expect(getChangellyUrl()).toBe('https://changelly.com')
    expect(getAdminPassword()).toBe('secret123')
  })
})

describe('feature-detection: 远程节点管理', () => {
  beforeEach(() => {
    resetFeatureContext()
    resetRemoteNode(false)
    registerRemoteNodesMgr(() => null)
  })

  it('非移动端 getRemoteNodeUrl 返回空字符串', () => {
    expect(getRemoteNodeUrl()).toBe('')
  })

  it('移动端从远程节点管理器获取节点 URL', () => {
    const mockNode: RemoteNodeHandle = {
      getUrl: vi.fn(() => 'http://remote.node/nrs'),
      blacklist: vi.fn()
    }
    registerRemoteNodesMgr(() => mockNode)
    setFeatureContext({ mobileSettings: { is_simulate_app: true } })

    expect(getRemoteNodeUrl()).toBe('http://remote.node/nrs')
    expect(mockNode.getUrl).toHaveBeenCalled()
    // 第二次调用复用已缓存的节点
    getRemoteNodeUrl()
    expect(mockNode.getUrl).toHaveBeenCalledTimes(2)
  })

  it('移动端无可用节点返回空字符串', () => {
    registerRemoteNodesMgr(() => null)
    setFeatureContext({ mobileSettings: { is_simulate_app: true } })
    expect(getRemoteNodeUrl()).toBe('')
  })

  it('resetRemoteNode(blacklist) 调用 blacklist 并清空缓存', () => {
    const mockNode: RemoteNodeHandle = {
      getUrl: vi.fn(() => 'http://remote.node/nrs'),
      blacklist: vi.fn()
    }
    registerRemoteNodesMgr(() => mockNode)
    setFeatureContext({ mobileSettings: { is_simulate_app: true } })
    getRemoteNodeUrl()
    expect(getRemoteNode()).toBe(mockNode)

    resetRemoteNode(true)
    expect(mockNode.blacklist).toHaveBeenCalled()
    expect(getRemoteNode()).toBeNull()
  })

  it('resetRemoteNode(false) 不调用 blacklist', () => {
    const mockNode: RemoteNodeHandle = {
      getUrl: vi.fn(() => 'http://remote.node/nrs'),
      blacklist: vi.fn()
    }
    registerRemoteNodesMgr(() => mockNode)
    setFeatureContext({ mobileSettings: { is_simulate_app: true } })
    getRemoteNodeUrl()
    resetRemoteNode(false)
    expect(mockNode.blacklist).not.toHaveBeenCalled()
  })
})

describe('feature-detection: openMobileBrowser', () => {
  it('cordova 不可用时静默失败', () => {
    expect(() => openMobileBrowser('https://example.com')).not.toThrow()
  })

  it('cordova 可用时调用 InAppBrowser.open', () => {
    const open = vi.fn()
    vi.stubGlobal('cordova', { InAppBrowser: { open } })
    openMobileBrowser('https://example.com')
    expect(open).toHaveBeenCalledWith('https://example.com', '_system')
    vi.unstubAllGlobals()
  })
})

describe('feature-detection: 上下文管理', () => {
  beforeEach(() => {
    resetFeatureContext()
  })

  it('setFeatureContext 合并而非覆盖', () => {
    setFeatureContext({ apiProxy: true })
    setFeatureContext({ isTestNet: true })
    expect(getFeatureContext()).toEqual({ apiProxy: true, isTestNet: true })
  })

  it('resetFeatureContext 清空上下文', () => {
    setFeatureContext({ apiProxy: true, isTestNet: true })
    resetFeatureContext()
    expect(getFeatureContext()).toEqual({})
    expect(isForgingSupported()).toBe(true)
  })
})
