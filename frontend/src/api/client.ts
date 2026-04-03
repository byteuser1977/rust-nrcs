import axios, { type AxiosInstance, type AxiosRequestConfig, type AxiosResponse } from 'axios'
import type { ApiResponse, RequestConfig, ApiError } from '@/types'
import { useAccountStore } from '@/stores/modules/account.store'
import { useUiStore } from '@/stores/modules/ui.store'
import { ElMessage } from 'element-plus'

/**
 * 创建 Axios 实例
 */
const createInstance = (): AxiosInstance => {
  const instance = axios.create({
    baseURL: import.meta.env.VITE_API_BASE_URL,
    timeout: 30000,
    headers: {
      'Content-Type': 'application/json',
      'X-Requested-With': 'XMLHttpRequest'
    },
    transformResponse: [(data, headers) => {
      try {
        return JSON.parse(data as string)
      } catch {
        return data
      }
    }]
  })

  // Request Interceptor
  instance.interceptors.request.use(
    (config) => {
      const accountStore = useAccountStore()
      const uiStore = useUiStore()
      const startTime = Date.now()

      // 注入 Token
      if (accountStore.accessToken) {
        config.headers.Authorization = `Bearer ${accountStore.accessToken}`
      }

      // 设置请求 ID
      const requestId = crypto.randomUUID()
      config.headers['X-Request-Id'] = requestId

      // 开发环境日志
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

      // 存储开始时间（用于计算响应耗时）
      config.metadata = { startTime, requestId }

      return config
    },
    (error) => {
      console.error('[Request Error]', error)
      return Promise.reject(error)
    }
  )

  // Response Interceptor
  instance.interceptors.response.use(
    (response: AxiosResponse) => {
      const { config, data, status, headers } = response
      const startTime = (config as any).metadata?.startTime as number
      const duration = Date.now() - startTime

      // 开发环境日志
      if (import.meta.env.DEV) {
        console.group(`[API Response] ${status} ${config.url}`)
        console.log('Request ID:', (config as any).metadata?.requestId)
        console.log('Duration:', `${duration}ms`)
        console.log('Data:', data)
        console.groupEnd()
      }

      // 检查业务错误码（假设 code: 0 表示成功）
      if (data && data.code !== undefined && data.code !== 0) {
        const apiError: ApiError = {
          code: data.code,
          message: data.message || '请求失败',
          details: data.data
        }

        handleBusinessError(apiError, config as RequestConfig)
        return Promise.reject(apiError)
      }

      // 返回 data.data 或直接返回 data（取决于后端格式）
      return data.data !== undefined ? data.data : data
    },
    async (error) => {
      const { response, config } = error

      // 计算耗时
      const startTime = (config as any)?.metadata?.startTime
      const duration = startTime ? Date.now() - startTime : 0

      if (import.meta.env.DEV) {
        console.error(`[API Error] ${config?.url} - ${duration}ms`, error)
      }

      // 网络错误
      if (!response) {
        ElMessage.error('网络连接失败，请检查网络')
        return Promise.reject({
          code: 0,
          message: 'NETWORK_ERROR',
          details: error.message
        })
      }

      const { status, data } = response

      // HTTP 状态码处理
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
          // 尝试重试
          if ((config as RequestConfig)?.retryCount !== undefined && (config as RequestConfig).retryCount < 3) {
            (config as RequestConfig).retryCount = ((config as RequestConfig).retryCount || 0) + 1
            return handleRetry(config as RequestConfig)
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

/**
 * 处理业务错误
 */
function handleBusinessError(error: ApiError, config?: RequestConfig) {
  const accountStore = useAccountStore()
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
      // 显示错误消息（除非 silent）
      if (!config?.silent) {
        ElMessage.error(error.message)
      }
  }
}

/**
 * 处理未授权
 */
function handleUnauthorized() {
  const accountStore = useAccountStore()
  accountStore.logout()

  // 跳转到登录页（保留原路径）
  const router = useRouter()
  router.replace({
    name: 'login',
    query: { redirect: router.currentRoute.value.fullPath }
  })
}

/**
 * 重试逻辑（指数退避）
 */
async function handleRetry(config: RequestConfig): Promise<any> {
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

  // 克隆 config 避免修改原对象
  const newConfig = { ...config, retryCount }
  return request(config.url || '', newConfig)
}

// 导出单例
export const request = createInstance()

/**
 * 类型化请求方法
 */
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
