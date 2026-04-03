import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { UserRole } from '@/types/enums'

export const useAppStore = defineStore('app', () => {
  // 应用信息
  const version = ref('1.0.0')
  const buildTime = ref('')

  // 全局语言设置
  const language = ref<'zh-CN' | 'en-US'>('zh-CN')
  const theme = ref<'light' | 'dark'>('dark')

  // 首次加载状态（用于显示启动屏）
  const isInitialized = ref(false)

  // 计算属性
  const isDarkMode = computed(() => theme.value === 'dark')

  // 方法
  function setVersion(ver: string) {
    version.value = ver
  }

  function setBuildTime(time: string) {
    buildTime.value = time
  }

  function setLanguage(lang: 'zh-CN' | 'en-US') {
    language.value = lang
  }

  function toggleTheme() {
    theme.value = theme.value === 'light' ? 'dark' : 'light'
  }

  function setTheme(newTheme: 'light' | 'dark') {
    theme.value = newTheme
  }

  function markInitialized() {
    isInitialized.value = true
  }

  function reset() {
    version.value = '1.0.0'
    buildTime.value = ''
    language.value = 'zh-CN'
    theme.value = 'dark'
    isInitialized.value = false
  }

  return {
    // state
    version,
    buildTime,
    language,
    theme,
    isInitialized,
    // getters
    isDarkMode,
    // actions
    setVersion,
    setBuildTime,
    setLanguage,
    toggleTheme,
    setTheme,
    markInitialized,
    reset
  }
})
