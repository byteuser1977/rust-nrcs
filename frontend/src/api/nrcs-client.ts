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

const NRCS_BASE_URL = '/nrcs'

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
    if (!error.response) {
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

export async function nrcsGet<T = any>(requestType: string, params?: Record<string, any>): Promise<T> {
  const response = await nrcsClient.get('', {
    params: {
      requestType,
      ...params
    }
  })
  return response as T
}

export async function nrcsPost<T = any>(requestType: string, data?: Record<string, any>): Promise<T> {
  const formData = buildFormData({ requestType, ...data })
  const response = await nrcsClient.post('', formData, {
    headers: {
      'Content-Type': 'application/x-www-form-urlencoded'
    }
  })
  return response as T
}

export async function nrcsRequest<T = any>(requestType: string, params?: Record<string, any>): Promise<T> {
  if (isRequirePost(requestType, params)) {
    return nrcsPost<T>(requestType, params)
  }
  return nrcsGet<T>(requestType, params)
}

export default nrcsClient
