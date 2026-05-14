import { defineStore } from 'pinia'
import { ref, computed, watch } from 'vue'

export interface ThemeConfig {
  name: 'light' | 'dark'
  primaryColor: string
  sidebarCollapsed: boolean
  navbarPosition: 'top' | 'side'
  layout: 'classic' | 'compact'
}

export const useAppStore = defineStore('app', () => {
  // --- State ---
  const theme = ref<ThemeConfig>({
    name: 'dark',
    primaryColor: '#ff5c5c',
    sidebarCollapsed: false,
    navbarPosition: 'top',
    layout: 'classic'
  })
  const sidebarCollapsed = ref(false)
  const navbarHeight = ref(60)
  const device = ref<'desktop' | 'mobile'>('desktop')
  const locale = ref('zh-CN')
  const loading = ref(false)
  const isConnected = ref(false)
  const blockHeight = ref(0)
  const forging = ref(false)

  // --- Getters ---
  const currentTheme = computed(() => theme.value.name)
  const isDark = computed(() => theme.value.name === 'dark')
  const isSidebarCollapsed = computed(() => sidebarCollapsed.value)
  const currentDevice = computed(() => device.value)
  const currentLocale = computed(() => locale.value)

  // --- Actions ---

  function initTheme(): void {
    const savedTheme = localStorage.getItem('app-theme')
    const savedSidebar = localStorage.getItem('sidebar-collapsed')
    const savedLocale = localStorage.getItem('app-locale')

    if (savedTheme) {
      try {
        const cfg = JSON.parse(savedTheme)
        theme.value = { ...theme.value, ...cfg }
      } catch {
        // ignore parse errors
      }
    }
    if (savedSidebar !== null) {
      sidebarCollapsed.value = savedSidebar === 'true'
    }
    if (savedLocale) {
      locale.value = savedLocale
    }

    applyTheme(theme.value.name)
  }

  async function toggleTheme(name?: 'light' | 'dark'): Promise<void> {
    const newTheme = name || (theme.value.name === 'light' ? 'dark' : 'light')
    theme.value.name = newTheme
    applyTheme(newTheme)
    persistTheme()
  }

  function applyTheme(themeName: 'light' | 'dark'): void {
    if (themeName === 'dark') {
      document.documentElement.classList.add('dark')
      document.documentElement.setAttribute('data-theme', 'dark')
    } else {
      document.documentElement.classList.remove('dark')
      document.documentElement.setAttribute('data-theme', 'light')
    }
  }

  function toggleSidebar(): void {
    sidebarCollapsed.value = !sidebarCollapsed.value
    localStorage.setItem('sidebar-collapsed', String(sidebarCollapsed.value))
  }

  function setSidebarCollapsed(collapsed: boolean): void {
    sidebarCollapsed.value = collapsed
    localStorage.setItem('sidebar-collapsed', String(collapsed))
  }

  function setDevice(d: 'desktop' | 'mobile'): void {
    device.value = d
    if (d === 'mobile') {
      setSidebarCollapsed(true)
    }
  }

  function setLocale(loc: string): void {
    locale.value = loc
    localStorage.setItem('app-locale', loc)
  }

  function setLoading(l: boolean): void {
    loading.value = l
  }

  function setConnected(c: boolean): void {
    isConnected.value = c
  }

  function setBlockHeight(h: number): void {
    blockHeight.value = h
  }

  function setForging(f: boolean): void {
    forging.value = f
  }

  function persistTheme(): void {
    localStorage.setItem('app-theme', JSON.stringify(theme.value))
  }

  function reset(): void {
    theme.value = {
      name: 'dark',
      primaryColor: '#ff5c5c',
      sidebarCollapsed: false,
      navbarPosition: 'top',
      layout: 'classic'
    }
    sidebarCollapsed.value = false
    device.value = 'desktop'
    locale.value = 'zh-CN'
    loading.value = false
    isConnected.value = false
    blockHeight.value = 0
    forging.value = false

    localStorage.removeItem('app-theme')
    localStorage.removeItem('sidebar-collapsed')
    localStorage.removeItem('app-locale')
  }

  // Auto-init
  initTheme()

  return {
    // state
    theme,
    sidebarCollapsed,
    navbarHeight,
    device,
    locale,
    loading,
    isConnected,
    blockHeight,
    forging,
    // getters
    currentTheme,
    isDark,
    isSidebarCollapsed,
    currentDevice,
    currentLocale,
    // actions
    initTheme,
    toggleTheme,
    applyTheme,
    toggleSidebar,
    setSidebarCollapsed,
    setDevice,
    setLocale,
    setLoading,
    setConnected,
    setBlockHeight,
    setForging,
    persistTheme,
    reset
  }
})

export default useAppStore
