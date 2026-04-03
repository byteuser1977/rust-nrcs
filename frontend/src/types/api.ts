import type { AxiosRequestConfig } from 'axios'

/**
 * API 统一响应格式
 * 符合 OpenAPI 3.0 规范
 */
export interface ApiResponse<T = unknown> {
  code: number
  message: string
  data: T
  request_id?: string
}

/**
 * 分页响应
 */
export interface PageResult<T> {
  total: number
  list: T[]
  page: number
  size: number
}

/**
 * 分页请求参数
 */
export interface PageParams {
  page: number
  size: number
}

/**
 * API 错误信息
 */
export interface ApiError {
  code: number
  message: string
  details?: any
}

/**
 * 增强的请求配置
 */
export interface RequestConfig extends AxiosRequestConfig {
  showLoading?: boolean
  silent?: boolean
  retryCount?: number
}

/**
 * 排序选项
 */
export interface SortOption {
  field: string
  desc: boolean
}

/**
 * 过滤条件
 */
export interface FilterCondition {
  field: string
  operator: 'eq' | 'ne' | 'gt' | 'gte' | 'lt' | 'lte' | 'contains' | 'startsWith' | 'endsWith'
  value: any
}
