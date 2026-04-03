import type { ApiResponse, User, LoginCredentials, RegisterCredentials } from '@/types'
import { post, get } from '../client'

/**
 * 账户相关 API 模块
 */
export const accountApi = {
  /**
   * 用户登录
   */
  login(credentials: LoginCredentials) {
    return post<ApiResponse<User>>('/auth/login', credentials)
  },

  /**
   * 用户注册
   */
  register(data: RegisterCredentials) {
    return post<ApiResponse<User>>('/auth/register', data)
  },

  /**
   * 获取当前用户信息
   */
  getUserInfo() {
    return get<ApiResponse<User>>('/auth/userinfo')
  },

  /**
   * 登出
   */
  logout() {
    return post<ApiResponse<null>>('/auth/logout')
  },

  /**
   * 刷新访问令牌
   */
  refreshToken(refreshToken: string) {
    return post<ApiResponse<{ access_token: string; refresh_token?: string }>>(
      '/auth/refresh',
      { refresh_token: refreshToken }
    )
  },

  /**
   * 更新用户资料
   */
  updateProfile(data: Partial<User>) {
    return put<ApiResponse<User>>('/auth/profile', data)
  },

  /**
   * 修改密码
   */
  changePassword(data: { old_password: string; new_password: string }) {
    return post<ApiResponse<null>>('/auth/change-password', data)
  }
}
