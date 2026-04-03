import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { User, LoginCredentials, RegisterCredentials } from '@/types/business'
import { accountApi } from '@/api/modules'
import type { ApiResponse } from '@/types'
import { useUiStore } from '@/stores/modules/ui.store'

export const useAccountStore = defineStore('account', () => {
  // 用户信息
  const userInfo = ref<User | null>(null)
  const walletAddress = ref('')
  const accessToken = ref<string>('')
  const refreshToken = ref<string>('')

  // 钱包连接状态
  const isWalletConnected = computed(() => !!walletAddress.value)

  // 计算属性
  const isLoggedIn = computed(() => !!accessToken.value)
  const userRole = computed(() => userInfo.value?.role || 'guest')
  const isAdmin = computed(() => userRole.value === 'admin')
  const displayName = computed(() => userInfo.value?.name || 'Unknown')

  // 方法
  /**
   * 用户登录
   */
  async function login(credentials: LoginCredentials): Promise<void> {
    try {
      const response: ApiResponse<User> = await accountApi.login(credentials)
      const { user, access_token, refresh_token } = response

      userInfo.value = user
      accessToken.value = access_token
      refreshToken.value = refresh_token

      // 持久化 token
      localStorage.setItem('access_token', access_token)
      localStorage.setItem('refresh_token', refresh_token)
    } catch (error: any) {
      throw error
    }
  }

  /**
   * 用户注册
   */
  async function register(data: RegisterCredentials): Promise<void> {
    try {
      const response: ApiResponse<User> = await accountApi.register(data)
      const { user, access_token, refresh_token } = response

      userInfo.value = user
      accessToken.value = access_token
      refreshToken.value = refresh_token

      localStorage.setItem('access_token', access_token)
      localStorage.setItem('refresh_token', refresh_token)
    } catch (error: any) {
      throw error
    }
  }

  /**
   * 获取当前用户信息
   */
  async function fetchUserInfo(): Promise<void> {
    try {
      const response: ApiResponse<User> = await accountApi.getUserInfo()
      userInfo.value = response

      // 如果之前未保存，保存钱包地址
      if (!walletAddress.value && response.wallet_address) {
        walletAddress.value = response.wallet_address
      }
    } catch (error: any) {
      throw error
    }
  }

  /**
   * 登出
   */
  async function logout(): Promise<void> {
    try {
      await accountApi.logout()
    } catch (error: any) {
      console.warn('Logout request failed, clearing local state anyway', error)
    } finally {
      // 清除本地状态
      userInfo.value = null
      walletAddress.value = ''
      accessToken.value = ''
      refreshToken.value = ''

      // 清除持久化数据
      localStorage.removeItem('access_token')
      localStorage.removeItem('refresh_token')
    }
  }

  /**
   * 刷新 token
   */
  async function refreshAccessToken(): Promise<boolean> {
    const refresh = refreshToken.value || localStorage.getItem('refresh_token')
    if (!refresh) {
      return false
    }

    try {
      const response: ApiResponse<{ access_token: string; refresh_token?: string }> =
        await accountApi.refreshToken(refresh)

      accessToken.value = response.data.access_token
      localStorage.setItem('access_token', response.data.access_token)

      if (response.data.refresh_token) {
        refreshToken.value = response.data.refresh_token
        localStorage.setItem('refresh_token', response.data.refresh_token)
      }

      return true
    } catch (error: any) {
      console.error('Refresh token failed', error)
      await logout()
      return false
    }
  }

  /**
   * 设置钱包地址（连接钱包后调用）
   */
  function setWalletAddress(address: string) {
    walletAddress.value = address
    // 可选：将钱包地址保存到 localStorage
    localStorage.setItem('wallet_address', address)
  }

  /**
   * 初始化（从本地存储恢复登录状态）
   */
  function initFromStorage() {
    const token = localStorage.getItem('access_token')
    const refresh = localStorage.getItem('refresh_token')
    const savedWallet = localStorage.getItem('wallet_address') as string | null

    if (token) {
      accessToken.value = token
    }
    if (refresh) {
      refreshToken.value = refresh
    }
    if (savedWallet) {
      walletAddress.value = savedWallet
    }
  }

  // 从存储初始化
  initFromStorage()

  return {
    // state
    userInfo,
    walletAddress,
    accessToken,
    refreshToken,
    // getters
    isLoggedIn,
    isWalletConnected,
    userRole,
    isAdmin,
    displayName,
    // actions
    login,
    register,
    fetchUserInfo,
    logout,
    refreshAccessToken,
    setWalletAddress,
    initFromStorage
  }
})
