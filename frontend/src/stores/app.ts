import { defineStore } from 'pinia'
import type { ThemeConfig } from '@/types'

/**
 * 应用全局状态 Store
 * 管理主题、语言、布局等UI状态
 */

interface AppState {
  // 主题配置
  theme: ThemeConfig
  // 侧边栏状态
  sidebarCollapsed: boolean
  // 导航栏高度
  navbarHeight: number
  // 设备类型
  device: 'desktop' | 'mobile'
  // 语言
  locale: string
  // 加载状态
  loading: boolean
}

export const useAppStore = defineStore('app', {
  state: (): AppState => ({
    theme: {
      name: 'light',
      primaryColor: '#409eff',
      sidebarCollapsed: false,
      navbarPosition: 'top',
      layout: 'classic'
    },
    sidebarCollapsed: false,
    navbarHeight: 60,
    device: 'desktop',
    locale: 'zh-CN',
    loading: false
  }),

  getters: {
    /**
     * 当前主题名称
     */
    currentTheme: (state) => state.theme.name,

    /**
     * 是否暗色主题
     */
    isDark: (state) => state.theme.name === 'dark',

    /**
     * 侧边栏是否折叠
     */
    isSidebarCollapsed: (state) => state.sidebarCollapsed,

    /**
     * 当前设备
     */
    currentDevice: (state) => state.device,

    /**
     * 当前语言
     */
    currentLocale: (state) => state.locale
  },

  actions: {
    /**
     * 初始化主题
     */
    initTheme() {
      // 从本地存储恢复主题设置
      const savedTheme = localStorage.getItem('app-theme')
      const savedSidebar = localStorage.getItem('sidebar-collapsed')

      if (savedTheme) {
        try {
          const themeConfig = JSON.parse(savedTheme)
          this.theme = { ...this.theme, ...themeConfig }
        } catch (e) {
          console.error('Failed to parse saved theme:', e)
        }
      }

      if (savedSidebar) {
        this.sidebarCollapsed = savedSidebar === 'true'
      }

      // 应用主题到DOM
      this.applyTheme(this.theme.name)
    },

    /**
     * 切换主题
     */
    async toggleTheme(name?: 'light' | 'dark') {
      const newTheme = name || (this.theme.name === 'light' ? 'dark' : 'light')
      this.theme.name = newTheme
      this.applyTheme(newTheme)
      this.persistTheme()
    },

    /**
     * 应用主题到DOM
     */
    applyTheme(themeName: 'light' | 'dark') {
      if (themeName === 'dark') {
        document.documentElement.classList.add('dark')
        document.documentElement.setAttribute('data-theme', 'dark')
      } else {
        document.documentElement.classList.remove('dark')
        document.documentElement.setAttribute('data-theme', 'light')
      }
    },

    /**
     * 切换侧边栏折叠状态
     */
    toggleSidebar() {
      this.sidebarCollapsed = !this.sidebarCollapsed
      localStorage.setItem('sidebar-collapsed', String(this.sidebarCollapsed))
    },

    /**
     * 设置侧边栏状态
     */
    setSidebarCollapsed(collapsed: boolean) {
      this.sidebarCollapsed = collapsed
      localStorage.setItem('sidebar-collapsed', String(collapsed))
    },

    /**
     * 设置设备类型
     */
    setDevice(device: 'desktop' | 'mobile') {
      this.device = device
      // 移动端自动折叠侧边栏
      if (device === 'mobile') {
        this.setSidebarCollapsed(true)
      }
    },

    /**
     * 设置语言
     */
    setLocale(locale: string) {
      this.locale = locale
      localStorage.setItem('app-locale', locale)
    },

    /**
     * 设置加载状态
     */
    setLoading(loading: boolean) {
      this.loading = loading
    },

    /**
     * 持久化主题设置
     */
    persistTheme() {
      localStorage.setItem('app-theme', JSON.stringify(this.theme))
    },

    /**
     * 重置应用状态
     */
    reset() {
      this.$reset()
      localStorage.removeItem('app-theme')
      localStorage.removeItem('sidebar-collapsed')
    }
  },

  // 持久化配置
  persist: {
    key: 'app-store',
    storage: localStorage,
    include: ['theme', 'sidebarCollapsed', 'locale']
  }
})

export default useAppStore
