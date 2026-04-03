import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export const useUiStore = defineStore('ui', () => {
  // 侧边栏状态
  const sidebarCollapsed = ref(false)
  const sidebarWidth = ref(240)

  // 全局加载状态
  const globalLoading = ref(false)

  // 通知中心
  const notifications = ref<Array<{
    id: number
    type: 'success' | 'warning' | 'info' | 'error'
    title: string
    message: string
    duration?: number
  }>>([])
  const notificationCount = computed(() => notifications.value.length)

  // 主题设置
  const themeMode = ref<'dark' | 'light'>('dark')
  const primaryColor = ref('#409eff')

  // 窗口尺寸
  const windowWidth = ref(window.innerWidth)
  const windowHeight = ref(window.innerHeight)

  // 计算属性
  const isMobile = computed(() => windowWidth.value < 768)
  const isTablet = computed(() => windowWidth.value >= 768 && windowWidth.value < 1024)
  const isDesktop = computed(() => windowWidth.value >= 1024)
  const isLargeScreen = computed(() => windowWidth.value >= 1440)

  // 方法
  /**
   * 切换侧边栏
   */
  function toggleSidebar() {
    sidebarCollapsed.value = !sidebarCollapsed.value
    sidebarWidth.value = sidebarCollapsed.value ? 64 : 240
  }

  /**
   * 展开侧边栏
   */
  function expandSidebar() {
    sidebarCollapsed.value = false
    sidebarWidth.value = 240
  }

  /**
   * 收起侧边栏
   */
  function collapseSidebar() {
    sidebarCollapsed.value = true
    sidebarWidth.value = 64
  }

  /**
   * 设置全局加载状态
   */
  function setGlobalLoading(loading: boolean) {
    globalLoading.value = loading
  }

  /**
   * 添加通知
   */
  function addNotification(notification: {
    type: 'success' | 'warning' | 'info' | 'error'
    title: string
    message: string
    duration?: number
  }) {
    const id = Date.now()
    notifications.value.push({
      id,
      ...notification
    })

    // 自动移除（如果设置了 duration）
    if (notification.duration !== 0) {
      setTimeout(() => {
        removeNotification(id)
      }, notification.duration || 5000)
    }

    return id
  }

  /**
   * 移除通知
   */
  function removeNotification(id: number) {
    const index = notifications.value.findIndex(n => n.id === id)
    if (index !== -1) {
      notifications.value.splice(index, 1)
    }
  }

  /**
   * 清除所有通知
   */
  function clearAllNotifications() {
    notifications.value = []
  }

  /**
   * 设置主题模式
   */
  function setThemeMode(mode: 'dark' | 'light') {
    themeMode.value = mode
    // 可以在这里触发全局主题更新
    document.documentElement.setAttribute('data-theme', mode)
  }

  /**
   * 切换主题
   */
  function toggleTheme() {
    setThemeMode(themeMode.value === 'dark' ? 'light' : 'dark')
  }

  /**
   * 设置主题色
   */
  function setPrimaryColor(color: string) {
    primaryColor.value = color
    // 可以在这里更新 CSS 变量
    document.documentElement.style.setProperty('--el-color-primary', color)
  }

  /**
   * 更新窗口尺寸
   */
  function updateWindowSize() {
    windowWidth.value = window.innerWidth
    windowHeight.value = window.innerHeight

    // 移动端自动收起侧边栏
    if (windowWidth.value < 768 && !sidebarCollapsed.value) {
      collapseSidebar()
    } else if (windowWidth.value >= 768 && sidebarCollapsed.value) {
      expandSidebar()
    }
  }

  /**
   * 重置所有设置
   */
  function reset() {
    sidebarCollapsed.value = false
    sidebarWidth.value = 240
    themeMode.value = 'dark'
    primaryColor.value = '#409eff'
    clearAllNotifications()
  }

  // 监听窗口大小变化
  if (typeof window !== 'undefined') {
    window.addEventListener('resize', updateWindowSize)
  }

  return {
    // state
    sidebarCollapsed,
    sidebarWidth,
    globalLoading,
    notifications,
    notificationCount,
    themeMode,
    primaryColor,
    windowWidth,
    windowHeight,
    // getters
    isMobile,
    isTablet,
    isDesktop,
    isLargeScreen,
    // actions
    toggleSidebar,
    expandSidebar,
    collapseSidebar,
    setGlobalLoading,
    addNotification,
    removeNotification,
    clearAllNotifications,
    setThemeMode,
    toggleTheme,
    setPrimaryColor,
    updateWindowSize,
    reset
  }
})
