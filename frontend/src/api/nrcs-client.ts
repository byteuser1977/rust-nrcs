/**
 * NRCS 前端唯一推荐的 HTTP 客户端。
 *
 * 实现 NRCS（Nxt 风格）后端契约：
 * - baseURL `/nrcs`，所有请求通过 `requestType` 参数区分接口
 * - GET 查询、POST 写入（form-urlencoded）
 * - 响应统一为 `{ errorCode, errorDescription, ...data }`，errorCode !== 0 视为业务错误
 * - `isRequirePost` 按 requestType 与敏感字段（secretPhrase/doNotSign/adminPassword）判定是否必须 POST
 *
 * 对应参考实现：`/Volumes/DATA/data/develop/git/nrcs/nrcs-main/html/www/ui/js/nrs.server.js`。
 * 所有业务调用应通过 `@/api/modules/nrcs.api.ts` 的 `nrcsApi` 对象，而非直接调用本文件。
 */
import axios, { type AxiosInstance } from 'axios'
import { ElMessage } from 'element-plus'
import { logger } from '@/utils/logger'
import type { RemoteNodeHandle, RemoteNodeRequestFn, RemoteRequestOptions } from '@/utils/remote-nodes/types'

const NRCS_BASE_URL = '/nrcs'

/**
 * 响应钩子（供 response-confirmation 的 confirmResponse 接入，避免循环依赖）。
 *
 * 仅对 GET 请求触发（POST 请求无需确认）。钩子内部会判断 requestNeedsConfirmation，
 * 无需确认时直接返回。
 */
export interface ResponseHookInfo {
  /** 请求类型 */
  requestType: string
  /** 请求参数（GET 为 params，POST 为解析后的 data） */
  data: Record<string, any>
  /** 主请求的响应数据 */
  response: any
  /** 主请求使用的远程节点（若有，用于排除） */
  remoteNode?: RemoteNodeHandle
}

type ResponseHook = (info: ResponseHookInfo) => void

let responseHook: ResponseHook | null = null

/**
 * 注册响应钩子。
 *
 * 由 `utils/remote-nodes/index.ts` 的 `initRemoteNodesMgr` 调用，
 * 将 `confirmResponse` 注入到所有 GET 请求的成功响应链路。
 *
 * @param fn - 钩子函数，传 null 清除
 */
export function setResponseHook(fn: ResponseHook | null): void {
  responseHook = fn
}

const nrcsClient: AxiosInstance = axios.create({
  baseURL: NRCS_BASE_URL,
  timeout: 30000,
  headers: {
    'Content-Type': 'application/x-www-form-urlencoded'
  }
})

nrcsClient.interceptors.request.use((config) => {
  if (config.method === 'get') {
    config.params = config.params || {}
    config.params.random = Math.random()
  }
  return config
})

nrcsClient.interceptors.response.use(
  (response) => {
    const data = response.data
    // 记录到调试控制台（对标 nrs.server.js 的 NRS.addToConsole 调用）
    logger.addToConsole(
      response.config.url || '',
      response.config.method?.toUpperCase() || 'GET',
      response.config.params || response.config.data || null,
      data,
      !!(data && data.errorCode !== undefined && data.errorCode !== 0)
    )

    if (data && data.errorCode !== undefined && data.errorCode !== 0) {
      const error = new Error(data.errorDescription || '请求失败') as any
      error.code = data.errorCode
      error.description = data.errorDescription
      return Promise.reject(error)
    }
    // 触发响应钩子（confirmResponse 在此接入，fire-and-forget，不阻塞主响应）
    if (responseHook) {
      const config = response.config
      const requestType = config.params?.requestType
      if (requestType) {
        try {
          responseHook({
            requestType,
            data: { ...(config.params || {}) },
            response: data,
            remoteNode: (config as any)._remoteNodeHandle,
          })
        } catch (e) {
          console.log('[nrcs-client] response hook error', e)
        }
      }
    }
    return data
  },
  (error) => {
    // 记录错误响应到调试控制台
    const config = error.config
    if (config) {
      logger.addToConsole(
        config.url || '',
        config.method?.toUpperCase() || 'GET',
        config.params || config.data || null,
        error.response?.data || error.message,
        true
      )
    }
    // 远程节点请求（_silent 标记）失败时不弹全局提示，避免 bootstrap 阶段刷屏
    if (!error.response && !config?._silent) {
      ElMessage.error('网络连接失败')
    }
    return Promise.reject(error)
  }
)

export interface NrcsRequestParams {
  requestType: string
  [key: string]: any
}

function buildFormData(params: Record<string, any>): string {
  const parts: string[] = []
  for (const [key, value] of Object.entries(params)) {
    if (value !== undefined && value !== null && value !== '') {
      parts.push(encodeURIComponent(key) + '=' + encodeURIComponent(String(value)))
    }
  }
  return parts.join('&')
}

const REQUIRE_POST_TYPES = new Set([
  'sendMoney', 'sendMessage', 'setAccountInfo', 'setAccountProperty',
  'deleteAccountProperty', 'setAlias', 'deleteAlias', 'sellAlias', 'buyAlias',
  'issueAsset', 'transferAsset', 'placeAskOrder', 'placeBidOrder',
  'cancelAskOrder', 'cancelBidOrder', 'issueCurrency', 'currencyBuy',
  'currencySell', 'currencyMint', 'transferCurrency', 'deleteCurrency',
  'createPoll', 'castVote', 'startForging', 'stopForging',
  'dgsListing', 'dgsDelisting', 'dgsPriceChange', 'dgsQuantityChange',
  'dgsPurchase', 'dgsDelivery', 'dgsFeedback', 'dgsRefund',
  'uploadTaggedData', 'extendTaggedData', 'generateToken',
  'broadcastTransaction', 'signTransaction', 'approveTransaction',
  'addPeer', 'blacklistPeer', 'leaseBalance',
  'shufflingCreate', 'shufflingProcess', 'shufflingRegister',
  'shufflingVerify', 'shufflingCancel',
  'startShuffler', 'stopShuffler',
  'setPhasingOnlyControl', 'publishExchangeOffer',
  'startFundingMonitor', 'stopFundingMonitor',
  'dividendPayment', 'increaseAssetShares', 'deleteAssetShares',
  'currencyReserveIncrease', 'currencyReserveClaim',
  'deleteScheduledTransaction', 'markHost'
])

export function isRequirePost(requestType: string, data?: Record<string, any>): boolean {
  if (REQUIRE_POST_TYPES.has(requestType)) return true
  if (data && ('secretPhrase' in data || 'doNotSign' in data || 'adminPassword' in data)) return true
  return false
}

/**
 * 将 RemoteRequestOptions 转换为 axios config 片段。
 *
 * - `remoteNode` 存在时覆盖 baseURL 为远程节点 URL，并标记 `_silent` 抑制全局错误提示
 * - `timeout` 覆盖默认超时
 * - `noProxy` 当前仅作语义标记（Web 端直连本地节点即非代理），不影响 axios 行为
 */
function applyRemoteOptions(options?: RemoteRequestOptions): Record<string, any> {
  const config: Record<string, any> = {}
  if (!options) return config
  if (options.remoteNode) {
    config.baseURL = options.remoteNode.getUrl()
    config._silent = true
    // 保留 handle 引用，供响应钩子（confirmResponse）排除主请求节点
    config._remoteNodeHandle = options.remoteNode
  }
  if (options.timeout) {
    config.timeout = options.timeout
  }
  return config
}

export async function nrcsGet<T = any>(
  requestType: string,
  params?: Record<string, any>,
  options?: RemoteRequestOptions,
): Promise<T> {
  const response = await nrcsClient.get('', {
    params: {
      requestType,
      ...params
    },
    ...applyRemoteOptions(options)
  })
  return response as T
}

export async function nrcsPost<T = any>(
  requestType: string,
  data?: Record<string, any>,
  options?: RemoteRequestOptions,
): Promise<T> {
  const formData = buildFormData({ requestType, ...data })
  const response = await nrcsClient.post('', formData, {
    headers: {
      'Content-Type': 'application/x-www-form-urlencoded'
    },
    ...applyRemoteOptions(options)
  })
  return response as T
}

export async function nrcsRequest<T = any>(
  requestType: string,
  params?: Record<string, any>,
  options?: RemoteRequestOptions,
): Promise<T> {
  if (isRequirePost(requestType, params)) {
    return nrcsPost<T>(requestType, params, options)
  }
  return nrcsGet<T>(requestType, params, options)
}

/**
 * 远程节点请求函数（供 RemoteNodesManager 与 response-confirmation 使用）。
 *
 * 对标参考 `nrs.server.js` 的 `NRS.sendRequest(requestType, data, callback, options)` 中
 * 支持 `remoteNode` / `noProxy` / `timeout` 选项的能力。
 * 自动按 `isRequirePost` 选择 GET/POST，剥离 errorCode 包装后返回 data。
 */
export const remoteTransport: RemoteNodeRequestFn = async (
  requestType: string,
  data: Record<string, any>,
  options?: RemoteRequestOptions,
): Promise<any> => {
  return nrcsRequest<any>(requestType, data, options)
}

export default nrcsClient
