/**
 * 通用类型定义
 */

// ============ 基本类型 ============

// 空值类型
export type Nullable<T> = T | null

// 可选属性装饰器
export type Optional<T, K extends keyof T> = Omit<T, K> & Partial<Pick<T, K>>

// 只读属性
export type ReadonlyDeep<T> = T extends object
  ? { readonly [K in keyof T]: ReadonlyDeep<T[K]> }
  : T

// 不可变类型
export type Immutable<T> = ReadonlyDeep<T>

// ============ 结果类型 ============

// 成功/失败结果
export type Result<T, E = Error> =
  | { success: true; data: T }
  | { success: false; error: E }

// 操作结果
export interface OperationResult {
  success: boolean
  message?: string
  data?: any
  errors?: string[]
}

// ============ 响应包装器 ============

// API 响应包装器（已定义在 api.ts 中，这里提供别名）
export type Response<T> = ApiResponse<T>

// ============ 事件类型 ============

// 事件发射器事件
export interface EventCallbacks<T = any> {
  [event: string]: Array<(data: T) => void>
}

// 事件上下文
export interface EventContext<T = any> {
  event: string
  payload: T
  timestamp: number
}

// ============ 日期时间类型 ============

// ISO 日期时间字符串
export type ISODateTime = string

// 时间范围
export interface TimeRange {
  start: ISODateTime
  end: ISODateTime
}

// ============ 配置类型 ============

// 应用配置
export interface AppConfig {
  title: string
  version: string
  environment: 'development' | 'staging' | 'production'
  apiBaseUrl: string
  blockchainRpcUrl: string
  blockchainNetwork: string
  features: {
    walletConnect: boolean
    analytics: boolean
    debug: boolean
  }
}

// 主题配置
export interface ThemeConfig {
  name: 'light' | 'dark' | 'auto'
  primaryColor: string
  sidebarCollapsed: boolean
  navbarPosition: 'top' | 'left'
  layout: 'classic' | 'compact'
}

// ============ UI 状态类型 ============

// 加载状态
export interface LoadingState {
  isLoading: boolean
  error: string | null
  data: any
}

// 分页状态
export interface PaginationState {
  currentPage: number
  pageSize: number
  total: number
  totalPages: number
}

// 表格状态
export interface TableState<T> {
  data: T[]
  loading: boolean
  selectedRows: T[]
  sortBy?: string
  sortOrder?: 'asc' | 'desc'
}

// ============ 区块链特定通用类型 ============

// 哈希（32字节）
export type Hash32 = string & { __brand: 'Hash32' }

// 地址（20字节）
export type Address20 = string & { __brand: 'Address20' }

// 私钥（32字节）
export type PrivateKey32 = string & { __brand: 'PrivateKey32' }

// 验证签名结果
export interface SignatureVerification {
  valid: boolean
  signer?: string
  recoveryId?: number
}

// 签名数据
export interface SignedData {
  messageHash: string
  signature: string
  v: number
  r: string
  s: string
}

// ============ 文件类型 ============

// 文件上传配置
export interface UploadConfig {
  maxSize: number  // 最大文件大小（字节）
  accept: string   // MIME类型
  multiple: boolean
  directory: boolean
}

// 文件信息
export interface FileInfo {
  name: string
  size: number
  type: string
  lastModified: number
  url?: string
  token?: string  // 云存储 token
}
