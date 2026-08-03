/**
 * 远程节点管理器入口模块。
 *
 * 端口自参考实现 `nrs.remote.nodes.js:23-52` 的 `NRS.updateRemoteNodes` 与 `NRS.initRemoteNodesMgr`。
 *
 * 职责：
 *   - 创建 `RemoteNodesManager` 单例并注入远程请求函数（`remoteTransport`）
 *   - 注入 `confirmResponse` 为 nrcs-client 的响应钩子，启用 GET 请求的多节点交叉验证
 *   - 注入 `ConfirmationDeps`（来自 constants store 与 mobileSettings）
 *   - 注册管理器到 `feature-detection`，使 `getRemoteNodeUrl` 可用
 *   - 按运行场景启动 bootstrap（移动端）或 `updateRemoteNodes`（API 代理端）
 *
 * 调用时机：app 启动完成 constants store 加载与 mobileSettings 读取后调用 `initRemoteNodesMgr`。
 */
import { RemoteNodesManager, getRemoteNodesManager, setRemoteNodesManager, pickRandomRemoteNode } from './remote-nodes-manager'
import {
  setConfirmationDeps,
  setConfirmationTransport,
  confirmResponse,
  resetConfirmations,
  type ConfirmationDeps,
} from './response-confirmation'
import { getMobileSettings } from './mobile-settings'
import { remoteTransport, setResponseHook } from '@/api/nrcs-client'
import {
  registerRemoteNodesMgr,
  resetRemoteNode,
  isMobileApp,
  isUpdateRemoteNodes,
  isRemoteNodeConnectionAllowed,
} from '@/utils/feature-detection'
import { useConstantsStore } from '@/stores/modules/constants.store'
import { ElMessage } from 'element-plus'
import { i18n } from '@/locales'

/**
 * 构造确认依赖。
 *
 * 对标 `nrs.remote.nodes.js:54` 的 `NRS.requestNeedsConfirmation` 对
 * `NRS.isRequirePost` / `NRS.isRequestForwardable` / `NRS.mobileSettings.validators_count` 的依赖。
 *
 * 使用 constants store 的运行时方法（基于服务端 REQUEST_TYPES），
 * 以及 mobileSettings 的 validators_count。
 */
function buildConfirmationDeps(): ConfirmationDeps {
  const constantsStore = useConstantsStore()
  return {
    isRequirePost: (rt: string) => constantsStore.isRequirePost(rt),
    isRequestForwardable: (rt: string) => constantsStore.isRequestForwardable(rt),
    getValidatorsCount: () => getMobileSettings().validators_count,
  }
}

/**
 * 初始化远程节点管理器。
 *
 * 对标 `nrs.remote.nodes.js:35` 的 `NRS.initRemoteNodesMgr`。
 *
 * 场景分支：
 *   - 移动端（isMobileApp）：
 *     - `remote_node_address` 为空 → `addBootstrapNodes`（从内置列表随机连接）
 *     - 否则 → `addBootstrapNode`（连接用户配置的单节点，失败回退到 addBootstrapNodes）
 *   - API 代理端（isUpdateRemoteNodes，即 apiProxy=true）：
 *     - 允许连接 → `updateRemoteNodes`（从本地节点 getPeers 拉取 CONNECTED 节点）
 *     - HTTPS 禁止 → 提示 `https_client_cannot_connect_remote_nodes`
 *   - Web 端直连：无操作（直连本地节点，无需远程节点）
 *
 * @param isTestnet - 是否测试网
 * @returns Promise，移动端 bootstrap 完成后 resolve；其他场景立即 resolve
 */
export async function initRemoteNodesMgr(isTestnet: boolean): Promise<void> {
  // 创建管理器单例（对标 nrs.remote.nodes.js:36-38）
  let mgr = getRemoteNodesManager()
  if (!mgr) {
    mgr = new RemoteNodesManager(isTestnet)
    setRemoteNodesManager(mgr)
  }

  // 注入远程请求函数（对标参考中 NRS.sendRequest 对 remoteNode 选项的支持）
  mgr.setRemoteTransport(remoteTransport)

  // 注入确认依赖与 transport（对标 confirmResponse 对 NRS.isRequirePost 等的依赖）
  setConfirmationDeps(buildConfirmationDeps())
  setConfirmationTransport(remoteTransport)

  // 注册 confirmResponse 为响应钩子（仅对 GET 请求触发，内部自判断 requestNeedsConfirmation）
  setResponseHook(({ requestType, data, response, remoteNode }) => {
    // fire-and-forget，不阻塞主响应
    confirmResponse(requestType, data, response, remoteNode).catch((e) => {
      console.log('[remote-nodes] confirmResponse error', e)
    })
  })

  // 注册到 feature-detection，使 getRemoteNodeUrl / getRemoteNode 可用
  registerRemoteNodesMgr(pickRandomRemoteNode)

  // 按场景启动
  if (isMobileApp()) {
    const mobileSettings = getMobileSettings()
    if (mobileSettings.remote_node_address === '') {
      await mgr.addBootstrapNodes()
    } else {
      await mgr.addBootstrapNode()
    }
  } else if (isUpdateRemoteNodes()) {
    if (isRemoteNodeConnectionAllowed()) {
      await updateRemoteNodes()
    } else {
      // 对标 nrs.remote.nodes.js:49 的 $.growl 提示
      const t = i18n.global.t.bind(i18n.global)
      ElMessage.warning(t('mobileSettings.httpsClientCannotConnectRemoteNodes'))
    }
  }
}

/**
 * 通过本地节点拉取 CONNECTED peers 更新远程节点列表。
 *
 * 对标 `nrs.remote.nodes.js:23` 的 `NRS.updateRemoteNodes`。
 * 调用 `getPeers`（走本地节点），清空并重建 manager.nodes。
 */
export async function updateRemoteNodes(): Promise<void> {
  const mgr = getRemoteNodesManager()
  if (!mgr) {
    console.log('[remote-nodes] Manager not initialized, skip updateRemoteNodes')
    return
  }
  console.log('[remote-nodes] Updating remote nodes')
  try {
    const response: any = await remoteTransport('getPeers', { state: 'CONNECTED', includePeerInfo: true })
    if (response.peers) {
      mgr.nodes = {}
      mgr.addRemoteNodes(response.peers)
    }
    console.log('[remote-nodes] remote nodes updated')
  } catch (e: any) {
    console.log(`[remote-nodes] updateRemoteNodes error: ${e?.message || e}`)
  }
}

/**
 * 重置远程节点管理器（登出 / 切换网络时调用）。
 *
 * 清空 manager 单例、确认历史、feature-detection 中的远程节点状态与响应钩子。
 */
export function teardownRemoteNodesMgr(): void {
  setRemoteNodesManager(null)
  resetConfirmations()
  resetRemoteNode(false)
  setResponseHook(null)
}

// 导出常用项，供外部使用
export { RemoteNodesManager, getRemoteNodesManager, pickRandomRemoteNode } from './remote-nodes-manager'
export { RemoteNode } from './remote-node'
export {
  confirmResponse,
  requestNeedsConfirmation,
  confirmations,
  confirmationRate,
  resetConfirmations,
  isPeerListSimilar,
  compareLedgerEntries,
  getComparableResponse,
  type ConfirmationReport,
  type ConfirmationRateState,
} from './response-confirmation'
export { getMobileSettings, setMobileSettings, type MobileSettings } from './mobile-settings'
export { isRemoteNodeConnectable } from './remote-nodes-manager'
export type { PeerData, RemoteNodeHandle, RemoteRequestOptions } from './types'
