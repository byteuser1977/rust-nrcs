import apiClient from '../client'
import type {
  ApiResponse,
  UserInfo,
  LoginCredentials,
  LoginResponse,
  RegisterRequest,
  UpdateProfileRequest,
  Permission,
  Balance
} from '@/types'

/**
 * 账户管理 API 模块
 */

export const accountApi = {
  /**
   * 用户登录
   * @param credentials 登录凭证
   * @returns 登录结果（包含Token和用户信息）
   */
  async login(
    credentials: LoginCredentials
  ): Promise<ApiResponse<LoginResponse>> {
    return apiClient.post('/auth/login', credentials)
  },

  /**
   * 用户注册
   * @param data 注册信息
   * @returns 创建的用户信息
   */
  async register(data: RegisterRequest): Promise<ApiResponse<UserInfo>> {
    return apiClient.post('/auth/register', data)
  },

  /**
   * 获取当前用户信息
   * @returns 用户信息
   */
  async getUserInfo(): Promise<ApiResponse<UserInfo>> {
    return apiClient.get('/account/info')
  },

  /**
   * 更新用户信息
   * @param data 更新的用户信息
   * @returns 更新后的用户信息
   */
  async updateProfile(
    data: UpdateProfileRequest
  ): Promise<ApiResponse<UserInfo>> {
    return apiClient.put('/account/profile', data)
  },

  /**
   * 修改密码
   * @param data 包含旧密码和新密码
   * @returns 操作结果
   */
  async changePassword(
    data: { oldPassword: string; newPassword: string }
  ): Promise<ApiResponse<null>> {
    return apiClient.post('/account/password', data)
  },

  /**
   * 获取用户权限列表
   * @returns 权限列表
   */
  async getPermissions(): Promise<ApiResponse<Permission[]>> {
    return apiClient.get('/account/permissions')
  },

  /**
   * 刷新访问令牌
   * @param refreshToken 刷新令牌
   * @returns 新的令牌对
   */
  async refreshAccessToken(
    refreshToken: string
  ): Promise<ApiResponse<LoginResponse>> {
    return apiClient.post('/auth/refresh', { refresh_token: refreshToken })
  },

  /**
   * 用户登出
   * @returns 登出结果
   */
  async logout(): Promise<ApiResponse<null>> {
    return apiClient.post('/auth/logout')
  },

  /**
   * 获取用户会话列表
   * @returns 当前用户的所有活跃会话
   */
  async getSessions(): Promise<ApiResponse<any[]>> {
    return apiClient.get('/account/sessions')
  },

  /**
   * 终止指定会话
   * @param sessionId 会话ID
   * @returns 操作结果
   */
  async terminateSession(sessionId: string): Promise<ApiResponse<null>> {
    return apiClient.delete(`/account/sessions/${sessionId}`)
  },

  /**
   * 获取账户余额
   * @param address 账户地址
   * @param tokenAddress 代币合约地址（可选，不传则查询原生代币）
   * @returns 余额信息
   */
  async getBalance(
    address: string,
    tokenAddress?: string
  ): Promise<ApiResponse<Balance>> {
    const params = tokenAddress ? { token: tokenAddress } : {}
    return apiClient.get(`/accounts/${address}/balance`, { params })
  },

  /**
   * 获取账户交易列表
   * @param address 账户地址
   * @param params 分页和筛选参数
   * @returns 分页交易列表
   */
  async getAccountTransactions(
    address: string,
    params: { page?: number; size?: number; status?: string } = {}
  ): Promise<ApiResponse<PageResult<any>>> {
    return apiClient.get(`/accounts/${address}/transactions`, { params })
  }
}

export default accountApi
