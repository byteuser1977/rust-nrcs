/**
 * @deprecated Ethereum RESTful 客户端（Bearer token + /auth /transactions /node /contracts 端点）。
 *
 * 该客户端实现的是以太坊风格的后端契约（tx_hash / gas / wei / block_number / abi / bytecode），
 * 与 NRCS（Nxt 风格：requestType / NQT / RS 地址 / secretPhrase / transactionBytes）模型完全冲突。
 *
 * NRCS 前端的唯一推荐客户端是 `@/api/nrcs-client`（nrcsGet / nrcsPost / nrcsRequest），
 * 所有业务 API 通过 `@/api/modules/nrcs.api.ts` 的 `nrcsApi` 对象调用。
 *
 * 保留该文件仅为兼容尚未重写的以太坊占位视图（阶段 3.x 将逐步替换为 NRCS 风格）。
 * 新代码禁止使用本文件导出的 request/get/post/put/del。
 *
 * @see /Volumes/DATA/data/develop/git/nrcs/nrcs-main/html/www/ui/js/nrs.server.js 参考实现
 * @see @/api/nrcs-client.ts 替代实现
 */
import axios, { type AxiosInstance, type AxiosResponse, type InternalAxiosRequestConfig } from 'axios'
import type { RequestConfig, ApiError } from '@/types'
import { useAccountStore } from '@/stores/modules/account.store'
import { useUiStore } from '@/stores/modules/ui.store'
import { ElMessage } from 'element-plus'

interface Metadata {
  startTime: number
  requestId: string
}

interface ExtendedAxiosRequestConfig extends InternalAxiosRequestConfig {
  metadata?: Metadata
  retryCount?: number
  silent?: boolean
}

const createInstance = (): AxiosInstance => {
  const instance = axios.create({
    baseURL: import.meta.env.VITE_API_BASE_URL,
    timeout: 30000,
    headers: {
      'Content-Type': 'application/json',
      'X-Requested-With': 'XMLHttpRequest'
    },
    transformResponse: [(data) => {
      try {
        return JSON.parse(data as string)
      } catch {
        return data
      }
    }]
  })

  instance.interceptors.request.use(
    (config: ExtendedAxiosRequestConfig) => {
      const accountStore = useAccountStore()
      const startTime = Date.now()

      if (accountStore.accessToken) {
        config.headers.Authorization = `Bearer ${accountStore.accessToken}`
      }

      const requestId = crypto.randomUUID()
      config.headers['X-Request-Id'] = requestId

      if (import.meta.env.DEV) {
        console.group(`[API Request] ${config.method?.toUpperCase()} ${config.url}`)
        console.log('Request ID:', requestId)
        if (config.params) {
          console.log('Params:', config.params)
        }
        if (config.data && config.method !== 'get') {
          console.log('Body:', config.data)
        }
        console.groupEnd()
      }

      config.metadata = { startTime, requestId }

      return config
    },
    (error) => {
      console.error('[Request Error]', error)
      return Promise.reject(error)
    }
  )

  instance.interceptors.response.use(
    (response: AxiosResponse) => {
      const { config, data, status } = response
      const extendedConfig = config as ExtendedAxiosRequestConfig
      const startTime = extendedConfig.metadata?.startTime || 0
      const duration = Date.now() - startTime

      if (import.meta.env.DEV) {
        console.group(`[API Response] ${status} ${config.url}`)
        console.log('Request ID:', extendedConfig.metadata?.requestId)
        console.log('Duration:', `${duration}ms`)
        console.log('Data:', data)
        console.groupEnd()
      }

      if (data && data.code !== undefined && data.code !== 0) {
        const apiError: ApiError = {
          code: data.code,
          message: data.message || '请求失败',
          details: data.data
        }

        handleBusinessError(apiError, config as ExtendedAxiosRequestConfig)
        return Promise.reject(apiError)
      }

      return data.data !== undefined ? data.data : data
    },
    async (error) => {
      const { response, config } = error

      const extendedConfig = config as ExtendedAxiosRequestConfig | undefined
      const startTime = extendedConfig?.metadata?.startTime
      const duration = startTime ? Date.now() - startTime : 0

      if (import.meta.env.DEV) {
        console.error(`[API Error] ${config?.url} - ${duration}ms`, error)
      }

      if (!response) {
        ElMessage.error('网络连接失败，请检查网络')
        return Promise.reject({
          code: 0,
          message: 'NETWORK_ERROR',
          details: error.message
        })
      }

      const { status, data } = response

      switch (status) {
        case 401:
          handleUnauthorized()
          break
        case 403:
          ElMessage.error('权限不足')
          break
        case 404:
          ElMessage.error('请求的资源不存在')
          break
        case 429:
          ElMessage.error('请求过于频繁，请稍后再试')
          break
        case 500:
          ElMessage.error('服务器内部错误')
          break
        case 502:
        case 503:
          ElMessage.error('服务器暂时不可用')
          if (extendedConfig?.retryCount !== undefined && extendedConfig.retryCount < 3) {
            extendedConfig.retryCount = (extendedConfig.retryCount || 0) + 1
            return handleRetry(extendedConfig)
          }
          break
      }

      const errorMessage = data?.message || `HTTP ${status}`
      ElMessage.error(errorMessage)

      return Promise.reject({
        code: status,
        message: errorMessage,
        details: data
      })
    }
  )

  return instance
}

function handleBusinessError(error: ApiError, config?: ExtendedAxiosRequestConfig) {
  const uiStore = useUiStore()

  switch (error.code) {
    case 401:
      handleUnauthorized()
      break
    case 429:
      uiStore.addNotification({
        type: 'warning',
        title: '请求频繁',
        message: '请稍后再试',
        duration: 3000
      })
      break
    default:
      if (!config?.silent) {
        ElMessage.error(error.message)
      }
  }
}

function handleUnauthorized() {
  const accountStore = useAccountStore()
  accountStore.logout()

  const router = useRouter()
  router.replace({
    name: 'login',
    query: { redirect: router.currentRoute.value.fullPath }
  })
}

async function handleRetry(config: ExtendedAxiosRequestConfig): Promise<any> {
  const retryCount = config.retryCount || 0
  const maxRetries = 3
  const baseDelay = 1000

  if (retryCount >= maxRetries) {
    ElMessage.error('请求失败，请稍后再试')
    return Promise.reject(new Error('Max retries exceeded'))
  }

  const delay = baseDelay * Math.pow(2, retryCount)

  if (import.meta.env.DEV) {
    console.log(`[Retry] ${retryCount + 1}/${maxRetries} waiting ${delay}ms...`)
  }

  await new Promise(resolve => setTimeout(resolve, delay))

  const newConfig = { ...config, retryCount }
  return request(config.url || '', newConfig)
}

export const request = createInstance()

export default request

export function get<T = any>(url: string, config?: RequestConfig): Promise<T> {
  return request.get(url, config) as Promise<T>
}

export function post<T = any>(url: string, data?: any, config?: RequestConfig): Promise<T> {
  return request.post(url, data, config) as Promise<T>
}

export function put<T = any>(url: string, data?: any, config?: RequestConfig): Promise<T> {
  return request.put(url, data, config) as Promise<T>
}

export function del<T = any>(url: string, config?: RequestConfig): Promise<T> {
  return request.delete(url, config) as Promise<T>
}

export { request as axiosInstance }
