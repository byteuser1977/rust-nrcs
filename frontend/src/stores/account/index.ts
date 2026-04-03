import { defineStore } from 'pinia'
import type { UserInfo, UserRole } from '@/types'

/**
 * 账户状态管理 Store
 */

interface AccountState {
  userInfo: UserInfo | null
  token: string | null
  refreshToken: string | null
  isLoggedIn: boolean
  permissions: string[]
  roles: UserRole[]
  loading: boolean
}

export const useAccountStore = defineStore('account', {
  state: (): AccountState => ({
    userInfo: null,
    token: localStorage.getItem('access_token'),
    refreshToken: localStorage.getItem('refresh_token'),
    isLoggedIn: false,
    permissions: [],
    roles: [],
    loading: false
  }),

  getters: {
    /**
     * 用户名
     */
    username: (state) => state.userInfo?.username ?? '',

    /**
     * 是否为管理员
     */
    isAdmin: (state) => state.roles.includes('admin'),

    /**
     * 是否有指定权限
     */
    hasPermission: (state) => {
      return (permissionCode: string) => {
        return state.permissions.includes(permissionCode)
      }
    },

    /**
     * 是否有指定角色
     */
    hasRole: (state) => {
      return (role: UserRole) => {
        return state.roles.includes(role)
      }
    },

    /**
     * 用户ID
     */
    userId: (state) => state.userInfo?.id ?? ''
  },

  actions: {
    /**
     * 设置用户信息
     */
    setUserInfo(userInfo: UserInfo) {
      this.userInfo = userInfo
      this.roles = userInfo.roles
      this.permissions = userInfo.permissions
      this.isLoggedIn = true
    },

    /**
     * 设置认证Token
     */
    setToken(token: string, refreshToken: string) {
      this.token = token
      this.refreshToken = refreshToken
      localStorage.setItem('access_token', token)
      localStorage.setItem('refresh_token', refreshToken)
    },

    /**
     * 登出
     */
    logout() {
      this.userInfo = null
      this.token = null
      this.refreshToken = null
      this.isLoggedIn = false
      this.permissions = []
      this.roles = []

      localStorage.removeItem('access_token')
      localStorage.removeItem('refresh_token')

      // 跳转到登录页
      window.location.href = '/login'
    },

    /**
     * 刷新Token
     */
    async refreshToken(): Promise<boolean> {
      if (!this.refreshToken) {
        return false
      }

      try {
        // TODO: 调用API
        // const response = await accountApi.refreshAccessToken(this.refreshToken)
        // this.setToken(response.data.accessToken, response.data.refreshToken)
        // return true
        return false
      } catch (error) {
        console.error('Failed to refresh token:', error)
        this.logout()
        return false
      }
    }
  },

  persist: {
    key: 'account-store',
    storage: localStorage,
    include: ['token', 'refreshToken']  // 用户信息不持久化
  }
})

export default useAccountStore
