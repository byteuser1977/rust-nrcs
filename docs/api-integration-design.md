# NRCS 区块链平台 - API 集成层设计

## 1. 设计原则

1. **类型安全**：所有 API 响应必须定义 TypeScript 类型
2. **统一错误处理**：所有错误经过拦截器统一处理并转换为用户友好信息
3. **可追溯**：请求日志（请求/响应时间、参数、状态码）
4. **可重试**：网络错误自动重试（最多 3 次）
5. **可取消**：请求可取消（组件卸载时自动取消）
6. **可配置**：baseURL、timeout、headers 可配置

## 2. 整体架构

```
┌─────────────────────────────────────────────────────────────┐
│                   业务调用层（业务代码）                     │
│   accountApi.login({ username, password })               │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                请求拦截器链 (Request Interceptors)         │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ 1. 注入认证头 Authorization: Bearer {token}        │  │
│  │ 2. 添加请求 ID X-Request-Id（日志追踪）            │  │
│  │ 3. 记录请求开始时间（用于耗时统计）                 │  │
│  │ 4. 序列化请求体（JSON.stringify）                   │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                   Axios 适配层（HTTP 传输）                │
│  发送 HTTP 请求（支持 Fetch/Adapter 切换）               │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                响应拦截器链 (Response Interceptors)        │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ 1. 检查 HTTP 状态码（非 2xx 抛出错误）             │  │
│  │ 2. 提取业务错误码（code 字段）                      │  │
│  │ 3. 自动登出检测（401 处理）                         │  │
│  │ 4. 记录响应耗时、状态码到日志                       │  │
│  │ 5. 返回 response.data（解包）                       │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                   错误处理中间件 (Error Handler)           │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ - 网络错误 → 自动重试（最多 3 次）                  │  │
│  │ - 401 → 跳转登录页                                  │  │
│  │ - 429 → 友好提示"请求频率过高"                      │  │
│  │ - 5xx → "服务器繁忙，请稍后再试"                    │  │
│  │ - 其他 → 提取 message 字段显示                     │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
                 业务代码接收
          { code: 0, message: 'success', data: T }
                   或抛出 ApiError
```

## 3. 核心类型定义

```typescript
// types/api.ts

/**
 * API 统一响应格式
 * NRCS 后端遵循 OpenAPI 3.0 规范
 */
export interface ApiResponse<T = any> {
  code: number           // 业务错误码，0 表示成功
  message: string        // 可读的错误/成功消息
  data: T                // 响应体（成功时）
  request_id?: string   // 请求追踪 ID（可选，用于日志）
}

/**
 * 分页响应
 */
export interface PageResult<T> {
  total: number         // 总记录数
  list: T[]            // 数据列表
  page: number         // 当前页码（从 1 开始）
  size: number         // 每页数量
}

/**
 * API 错误信息（从响应中提取）
 */
export interface ApiError {
  code: number          // 业务错误码
  message: string       // 用户友好消息
  details?: any         // 额外错误详情（可选）
}

/**
 * 请求配置扩展
 */
export interface RequestConfig extends AxiosRequestConfig {
  // 是否显示加载提示
  showLoading?: boolean
  // 是否忽略错误（不抛出异常）
  silent?: boolean
  // 重试次数（默认 0 表示不重试）
  retryCount?: number
}
```

## 4. HTTP 客户端封装

### 4.1 Axios 实例配置

```typescript
// api/client.ts
import axios, { type AxiosInstance, type AxiosRequestConfig, type AxiosResponse } from 'axios'
import { message } from '@/utils/message'  // 假设有全局消息提示
import { router } from '@/router'
import { useAccountStore } from '@/stores/modules/account.store'
import { ErrorCode, type ApiError } from '@/types'
import { handleRetry } from './retry'

// 创建基础实例
const createInstance = (): AxiosInstance => {
  const instance = axios.create({
    baseURL: import.meta.env.VITE_API_BASE_URL,
    timeout: 30000,  // 30 秒
    headers: {
      'Content-Type': 'application/json',
      'X-Requested-With': 'XMLHttpRequest'
    },
    // 允许获取 response 全部数据（用于日志）
    transformResponse: [(data, headers) => {
      try {
        return JSON.parse(data as string)
      } catch {
        return data
      }
    }]
  })

  // ──────────────────────────────────────────────
  // Request Interceptor
  // ──────────────────────────────────────────────
  instance.interceptors.request.use(
    (config) => {
      // 1. 注入 Token
      const accountStore = useAccountStore()
      if (accountStore.accessToken) {
        config.headers.Authorization = `Bearer ${accountStore.accessToken}`
      }

      // 2. 添加请求 ID（日志追踪）
      const requestId = crypto.randomUUID()
      config.headers['X-Request-Id'] = requestId

      // 3. 记录请求日志（开发环境）
      if (import.meta.env.DEV) {
        console.log(`[API Request] ${config.method?.toUpperCase()} ${config.url}`, {
          params: config.params,
          data: config.data,
          requestId
        })
      }

      return config
    },
    (error) => {
      console.error('[Request Error]', error)
      return Promise.reject(error)
    }
  )

  // ──────────────────────────────────────────────
  // Response Interceptor
  // ──────────────────────────────────────────────
  instance.interceptors.response.use(
    (response: AxiosResponse) => {
      const { config, data, status, headers } = response

      // 1. 记录响应日志（开发环境）
      if (import.meta.env.DEV) {
        const duration = Date.now() - (config.metadata?.startTime as number)
        console.log(`[API Response] ${status} ${config.url}`, {
          requestId: config.headers['X-Request-Id'],
          duration: `${duration}ms`,
          data
        })
      }

      // 2. 检查业务错误码（假设后端 code: 0 表示成功）
      if (data.code !== 0) {
        const apiError: ApiError = {
          code: data.code,
          message: data.message || '请求失败',
          details: data.data
        }

        // 特殊码处理
        if (data.code === 401) {
          handleUnauthorized()
        } else if (data.code === 429) {
          message.warning('请求过于频繁，请稍后再试')
        } else {
          message.error(apiError.message)
        }

        return Promise.reject(apiError)
      }

      // 3. 返回 data 字段（业务数据）
      return data.data
    },
    async (error) => {
      const { response, config } = error

      // 网络错误（无 response）
      if (!response) {
        message.error('网络连接失败，请检查网络')
        return Promise.reject({
          code: ErrorCode.NETWORK_ERROR,
          message: '网络连接失败'
        })
      }

      const { status, data } = response

      // HTTP 状态码处理
      switch (status) {
        case 401:
          handleUnauthorized()
          break
        case 403:
          message.error('权限不足')
          break
        case 404:
          message.error('请求的资源不存在')
          break
        case 429:
          message.error('请求过于频繁，请稍后再试')
          break
        case 500:
          message.error('服务器内部错误')
          break
        case 502:
        case 503:
          message.error('服务器暂时不可用')
          // 502/503 尝试重试
          if (config.retryCount !== undefined && config.retryCount < 3) {
            config.retryCount = (config.retryCount || 0) + 1
            return handleRetry(config)
          }
          break
      }

      // 提取后端 message
      const errorMessage = data?.message || `HTTP ${status}`
      message.error(errorMessage)

      return Promise.reject({
        code: status,
        message: errorMessage,
        details: data
      })
    }
  )

  return instance
}

// 处理未授权
function handleUnauthorized(): void {
  const accountStore = useAccountStore()
  accountStore.logout()
  router.replace({ name: 'login', query: { redirect: router.currentRoute.value.fullPath } })
}

// 导出单例
export const request = createInstance()

// 类型化请求方法（方便使用）
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
```

### 4.2 重试机制

```typescript
// api/retry.ts
import { AxiosRequestConfig } from 'axios'
import { request } from './client'

export async function handleRetry(config: AxiosRequestConfig): Promise<any> {
  const retryCount = config.retryCount || 0
  const maxRetries = 3
  const baseDelay = 1000  // 1 秒

  if (retryCount >= maxRetries) {
    throw new Error('重试次数已达上限')
  }

  // 指数退避：1s、2s、4s
  const delay = baseDelay * Math.pow(2, retryCount)

  console.log(`[Retry] ${retryCount + 1}/${maxRetries} 等待 ${delay}ms 后重试`)

  await new Promise(resolve => setTimeout(resolve, delay))

  // 重新发起请求（克隆 config，避免修改原对象）
  const newConfig = { ...config, retryCount }
  return request(config.url, newConfig)
}
```

### 4.3 请求取消

```typescript
// composables/useRequest.ts
import { onUnmounted } from 'vue'
import axios from 'axios'

export function useRequest() {
  const cancelTokenSource = axios.CancelToken.source()

  onUnmounted(() => {
    // 组件卸载时取消所有挂起请求
    cancelTokenSource.cancel('Component unmounted')
  })

  return {
    cancelToken: cancelTokenSource.token
  }
}

// 使用示例
// const { cancelToken } = useRequest()
// request.get('/api/data', { cancelToken })
```

## 5. API 模块设计

### 5.1 模块划分原则

- **按业务域划分**：account、transaction、contract、node
- **每个模块一个文件**：`account.api.ts`
- **统一导出**：`api/modules/index.ts`
- **类型集中管理**：`types/business.ts`

### 5.2 Account API 示例

```typescript
// api/modules/account.api.ts
import type { ApiResponse, User, LoginCredentials, RegisterCredentials } from '@/types'
import { post, get } from './client'

export const accountApi = {
  /**
   * 用户登录
   * @param credentials 登录凭证（钱包签名）
   */
  login(credentials: LoginCredentials) {
    return post<ApiResponse<User>>('/api/v1/auth/login', credentials)
  },

  /**
   * 用户注册
   */
  register(data: RegisterCredentials) {
    return post<ApiResponse<User>>('/api/v1/auth/register', data)
  },

  /**
   * 获取当前用户信息
   */
  getUserInfo() {
    return get<ApiResponse<User>>('/api/v1/auth/userinfo')
  },

  /**
   * 登出
   */
  logout() {
    return post<ApiResponse<null>>('/api/v1/auth/logout')
  },

  /**
   * 刷新访问令牌
   */
  refreshToken(refreshToken: string) {
    return post<ApiResponse<{ access_token: string; refresh_token: string }>>(
      '/api/v1/auth/refresh',
      { refresh_token: refreshToken }
    )
  }
}
```

### 5.3 Transaction API 示例

```typescript
// api/modules/transaction.api.ts
import type { ApiResponse, PageResult, Transaction, SendTxParams } from '@/types'
import { get, post } from './client'

export const transactionApi = {
  /**
   * 查询交易列表（分页）
   */
  getTransactions(params: {
    page?: number
    size?: number
    status?: 'pending' | 'confirming' | 'success' | 'failed'
    address?: string  // 过滤地址（发送方/接收方）
  }) {
    return get<ApiResponse<PageResult<Transaction>>>('/api/v1/transactions', { params })
  },

  /**
   * 获取交易详情
   */
  getTransaction(hash: string) {
    return get<ApiResponse<Transaction>>(`/api/v1/transactions/${hash}`)
  },

  /**
   * 发送交易
   */
  sendTransaction(data: SendTxParams) {
    return post<ApiResponse<{ tx_hash: string }>>('/api/v1/transactions/send', data)
  },

  /**
   * 预估 Gas 费用
   */
  estimateGas(data: Partial<SendTxParams>) {
    return post<ApiResponse<{ gas_limit: number; gas_price: string }>>(
      '/api/v1/transactions/estimate-gas',
      data
    )
  }
}
```

### 5.4 Contract API 示例

```typescript
// api/modules/contract.api.ts
import type { ApiResponse, PageResult, Contract, DeployContractParams, CallContractParams } from '@/types'
import { get, post } from './client'

export const contractApi = {
  /**
   * 获取合约列表
   */
  getContracts(params: { page?: number; size?: number }) {
    return get<ApiResponse<PageResult<Contract>>>('/api/v1/contracts', { params })
  },

  /**
   * 获取合约详情
   */
  getContract(address: string) {
    return get<ApiResponse<Contract>>(`/api/v1/contracts/${address}`)
  },

  /**
   * 部署合约
   */
  deploy(data: DeployContractParams) {
    return post<ApiResponse<{ contract_address: string; tx_hash: string }>>(
      '/api/v1/contracts/deploy',
      data
    )
  },

  /**
   * 调用合约（只读）
   */
  call(data: CallContractParams) {
    return post<ApiResponse<any>>('/api/v1/contracts/call', data)
  },

  /**
   * 发送合约交易（写入）
   */
  send(data: CallContractParams) {
    return post<ApiResponse<{ tx_hash: string }>>('/api/v1/contracts/send', data)
  },

  /**
   * 获取合约事件日志
   */
  getEvents(address: string, params: { from_block?: number; to_block?: number; event_name?: string }) {
    return get<ApiResponse<any[]>>(`/api/v1/contracts/${address}/events`, { params })
  }
}
```

### 5.5 Node API 示例

```typescript
// api/modules/node.api.ts
import type { ApiResponse, NodeInfo, NetworkStats, BlockInfo } from '@/types'
import { get } from './client'

export const nodeApi = {
  /**
   * 获取节点信息
   */
  getNodeInfo() {
    return get<ApiResponse<NodeInfo>>('/api/v1/node/info')
  },

  /**
   * 获取网络统计
   */
  getNetworkStats() {
    return get<ApiResponse<NetworkStats>>('/api/v1/node/stats')
  },

  /**
   * 获取最新区块
   */
  getLatestBlocks(limit: number = 10) {
    return get<ApiResponse<BlockInfo[]>>('/api/v1/node/blocks/latest', {
      params: { limit }
    })
  },

  /**
   * 获取节点健康状态
   */
  getHealth() {
    return get<ApiResponse<{ status: 'healthy' | 'unhealthy'; checks: any }>>('/api/v1/node/health')
  }
}
```

## 6. API 类型生成（OpenAPI 3.0）

### 6.1 使用 OpenAPI Generator

如果后端提供 OpenAPI 3.0 规范文件（`openapi.yaml`），可使用工具自动生成 TypeScript 类型：

```bash
# 安装 openapi-typescript
npm install -D openapi-typescript

# 生成类型文件
npx openapi-typescript http://localhost:8080/api/v1/openapi.yaml --output src/types/api.d.ts
```

### 6.2 生成配置文件

```jsonc
// openapi-generator.config.json
{
  "schema": "http://localhost:8080/api/v1/openapi.yaml",
  "output": "./src/types/api.d.ts",
  "options": {
    "exportTypes": true,       // 导出类型
    "exportServices": true,    // 导出服务函数（可选）
    "throwOnError": true       // 业务错误抛出异常
  }
}
```

### 6.3 手动类型示例

如果无法自动生成，手动定义类型：

```typescript
// types/business.ts
export interface User {
  id: number
  name: string
  email: string
  wallet_address: string  // 以太坊地址格式 0x...
  role: UserRole
  created_at: string      // ISO 8601
  updated_at: string
}

export interface Transaction {
  hash: string           // 交易哈希
  block_number: number
  from: string           // 发送地址
  to: string             // 接收地址（合约创建时为 null）
  value: string          // 金额（字符串，避免精度丢失，单位 wei）
  gas_used: number
  gas_price: string      // gas 价格（wei）
  status: TxStatus
  input: string          // 合约调用数据（hex）
  created_at: string
}

export interface Contract {
  address: string
  name: string
  abi: ContractABI[]     // ABI 数组
  bytecode: string       // 部署字节码（hex）
  source_code?: string   // 源代码（可选）
  created_at: string
}

export interface ContractABI {
  type: 'function' | 'event' | 'constructor'
  name: string
  inputs: Array<{ name: string; type: string }>
  outputs?: Array<{ name: string; type: string }>
  stateMutability?: 'view' | 'pure' | 'payable' | 'nonpayable'
  anonymous?: boolean    // 事件是否匿名
}

export interface Block {
  number: number
  hash: string
  parent_hash: string
  timestamp: number      // Unix 时间戳
  transactions: string[] // 交易哈希列表
  gas_used: number
  gas_limit: number
  miner: string          // 矿工地址
}

export type TxStatus = 'pending' | 'confirming' | 'success' | 'failed'
export type UserRole = 'admin' | 'user' | 'guest'
```

## 7. 错误码约定

| Code | 含义 | 处理建议 |
|------|------|---------|
| 0 | 成功 | - |
| 400 | 请求参数错误 | 提示用户检查输入 |
| 401 | 未授权 | 跳转登录页 |
| 403 | 权限不足 | 提示权限不足 |
| 404 | 资源不存在 | 提示资源不存在 |
| 409 | 资源冲突 | 如钱包地址已绑定 |
| 422 | 业务逻辑错误 | 显示后端 message |
| 429 | 请求频率过高 | 提示稍后再试，前端限流 |
| 500 | 服务器错误 | 提示稍后再试 |
| 503 | 服务维护中 | 提示维护时间 |

```typescript
// enums/ErrorCode.ts
export enum BusinessErrorCode {
  // Auth
  WALLET_NOT_CONNECTED = 1001,
  INVALID_SIGNATURE = 1002,

  // Transaction
  INSUFFICIENT_BALANCE = 2001,
  GAS_PRICE_TOO_LOW = 2002,
  TX_FAILED = 2003,

  // Contract
  CONTRACT_NOT_FOUND = 3001,
  CALL_FAILED = 3002,
  INVALID_ABI = 3003
}
```

## 8. 请求日志（可选）

```typescript
// utils/logger.ts
interface RequestLog {
  requestId: string
  method: string
  url: string
  params?: any
  body?: any
  status?: number
  response?: any
  duration?: number
  timestamp: string
  userId?: string
}

// 发送到日志服务（可选）
async function sendLog(log: RequestLog) {
  if (!import.meta.env.DEV && import.meta.env.VITE_LOG_ENDPOINT) {
    await fetch(import.meta.env.VITE_LOG_ENDPOINT, {
      method: 'POST',
      body: JSON.stringify(log)
    })
  }
}
```

## 9. 测试建议

```typescript
// api/modules/__tests__/account.api.spec.ts
import { accountApi } from '../account.api'
import { mockClient } from './mock-client'

vi.mock('./client', () => ({ request: mockClient }))

describe('accountApi', () => {
  it('login success', async () => {
    mockClient.post.mockResolvedValue({
      code: 0,
      data: { id: 1, name: 'Test', wallet_address: '0x123' }
    })

    const result = await accountApi.login({ signature: '0xabc' })
    expect(result).toEqual({ id: 1, name: 'Test', wallet_address: '0x123' })
  })

  it('login failed with 401', async () => {
    mockClient.post.mockResolvedValue({
      code: 401,
      message: 'Invalid signature'
    })

    await expect(accountApi.login({ signature: '0xabc' })).rejects.toMatchObject({
      code: 401,
      message: 'Invalid signature'
    })
  })
})
```

## 10. 生产环境注意事项

1. **请求超时**：生产环境适当延长（如 60s）
2. **重试策略**：仅对 5xx 错误重试，避免幂等问题
3. **请求取消**：页面切换时取消不必要的请求
4. **内存泄漏**：清理未完成的请求（onUnmounted）
5. **错误上报**：业务错误上报到日志平台（Sentry）
6. **Token刷新**：401 自动刷新 token，避免用户登出
