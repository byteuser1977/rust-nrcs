import axios, { type AxiosInstance } from 'axios'
import { ElMessage } from 'element-plus'

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
    if (data && data.errorCode !== undefined && data.errorCode !== 0) {
      const error = new Error(data.errorDescription || '请求失败') as any
      error.code = data.errorCode
      error.description = data.errorDescription
      return Promise.reject(error)
    }
    return data
  },
  (error) => {
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
  'setPhasingOnlyControl', 'publishExchangeOffer',
  'startFundingMonitor', 'stopFundingMonitor',
  'dividendPayment', 'increaseAssetShares', 'deleteAssetShares'
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
