/******************************************************************************
 * 全局设置 Store —— 对标 nrs.settings.js
 *
 * 参考：
 *   - nrs.settings.js NRS.defaultSettings（22+ 配置项）
 *   - nrs.settings.js NRS.getSettings（IndexedDB 加载）
 *   - nrs.settings.js NRS.updateSettings（IndexedDB 保存）
 *   - nrs.settings.js NRS.applySettings（运行时应用）
 *
 * 安全模型：设置存储在 localStorage，不涉及敏感数据。
 * 所有设置项都有默认值，首次加载时自动初始化。
 ******************************************************************************/
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

/** 设置项类型定义 */
export interface AppSettings {
  /** 按回车键提交表单（0=禁用, 1=启用） */
  submit_on_enter: string
  /** 显示锻造动画（0=禁用, 1=启用） */
  animate_forging: string
  /** 新闻显示（-1=首次, 0=隐藏, 1=显示） */
  news: string
  /** 市场显示（-1=首次, 0=隐藏, 1=显示） */
  marketplace: string
  /** 交易所显示（-1=首次, 0=隐藏, 1=显示） */
  exchange: string
  /** 控制台日志（0=禁用, 1=启用） */
  console_log: string
  /** 费用警告阈值（NQT） */
  fee_warning: string
  /** 金额警告阈值（NQT） */
  amount_warning: string
  /** 资产转移警告阈值（QNT） */
  asset_transfer_warning: string
  /** 积分转移警告阈值（QNT） */
  currency_transfer_warning: string
  /** 24小时格式（0=12小时, 1=24小时） */
  '24_hour_format': string
  /** 语言 */
  language: string
  /** 区域格式 */
  regional_format: string
  /** 启用插件（0=禁用, 1=启用） */
  enable_plugins: string
  /** 每页显示条数 */
  items_page: string
  /** 管理员密码 */
  admin_password: string
  /** 最大小数位数 */
  max_nxt_decimals: string
  /** 虚假实体警告（0=禁用, 1=启用） */
  fake_entity_warning: string
  /** 记住解密密码短语（0=禁用, 1=启用） */
  remember_decryption_passphrase: string
}

/** 默认设置（对标 nrs.settings.js NRS.defaultSettings） */
export const defaultSettings: AppSettings = {
  submit_on_enter: '0',
  animate_forging: '1',
  news: '-1',
  marketplace: '-1',
  exchange: '-1',
  console_log: '0',
  fee_warning: '100000000000',
  amount_warning: '10000000000000',
  asset_transfer_warning: '10000',
  currency_transfer_warning: '10000',
  '24_hour_format': '1',
  language: 'zh-CN',
  regional_format: 'default',
  enable_plugins: '0',
  items_page: '15',
  admin_password: '',
  max_nxt_decimals: '2',
  fake_entity_warning: '1',
  remember_decryption_passphrase: '0',
}

/** localStorage 键名 */
const SETTINGS_STORAGE_KEY = 'nrcs-settings'

export const useSettingsStore = defineStore('settings', () => {
  // --- State ---
  const settings = ref<AppSettings>({ ...defaultSettings })

  // --- Getters ---

  /** 每页显示条数（数字） */
  const itemsPerPage = computed(() => parseInt(settings.value.items_page, 10) || 15)

  /** 是否启用回车提交 */
  const submitOnEnter = computed(() => settings.value.submit_on_enter === '1')

  /** 是否显示锻造动画 */
  const animateForging = computed(() => settings.value.animate_forging === '1')

  /** 是否启用控制台日志 */
  const consoleLog = computed(() => settings.value.console_log === '1')

  /** 费用警告阈值（NQT） */
  const feeWarning = computed(() => settings.value.fee_warning)

  /** 金额警告阈值（NQT） */
  const amountWarning = computed(() => settings.value.amount_warning)

  /** 资产转移警告阈值 */
  const assetTransferWarning = computed(() => settings.value.asset_transfer_warning)

  /** 积分转移警告阈值 */
  const currencyTransferWarning = computed(() => settings.value.currency_transfer_warning)

  /** 是否为24小时格式 */
  const is24HourFormat = computed(() => settings.value['24_hour_format'] === '1')

  /** 当前语言 */
  const currentLanguage = computed(() => settings.value.language)

  /** 是否启用插件 */
  const enablePlugins = computed(() => settings.value.enable_plugins === '1')

  /** 是否记住解密密码短语 */
  const rememberDecryptionPassphrase = computed(() => settings.value.remember_decryption_passphrase === '1')

  // --- Actions ---

  /**
   * 初始化设置（对标 NRS.getSettings）。
   *
   * 从 localStorage 加载已保存的设置，与默认设置合并。
   */
  function initSettings(): void {
    try {
      const saved = localStorage.getItem(SETTINGS_STORAGE_KEY)
      if (saved) {
        const parsed = JSON.parse(saved) as Partial<AppSettings>
        settings.value = { ...defaultSettings, ...parsed }
      }
    } catch {
      // 解析失败时使用默认设置
      settings.value = { ...defaultSettings }
    }
  }

  /**
   * 更新单个设置项（对标 NRS.updateSettings）。
   *
   * @param key 设置项键名
   * @param value 设置项值
   */
  function updateSetting<K extends keyof AppSettings>(key: K, value: AppSettings[K]): void {
    settings.value[key] = value
    persistSettings()
  }

  /**
   * 批量更新设置。
   *
   * @param partial 部分设置项
   */
  function updateSettings(partial: Partial<AppSettings>): void {
    settings.value = { ...settings.value, ...partial }
    persistSettings()
  }

  /**
   * 重置所有设置为默认值。
   */
  function resetSettings(): void {
    settings.value = { ...defaultSettings }
    persistSettings()
  }

  /**
   * 持久化设置到 localStorage。
   */
  function persistSettings(): void {
    try {
      localStorage.setItem(SETTINGS_STORAGE_KEY, JSON.stringify(settings.value))
    } catch {
      // localStorage 写入失败时静默忽略
    }
  }

  // Auto-init
  initSettings()

  return {
    // state
    settings,
    // getters
    itemsPerPage,
    submitOnEnter,
    animateForging,
    consoleLog,
    feeWarning,
    amountWarning,
    assetTransferWarning,
    currencyTransferWarning,
    is24HourFormat,
    currentLanguage,
    enablePlugins,
    rememberDecryptionPassphrase,
    // actions
    initSettings,
    updateSetting,
    updateSettings,
    resetSettings,
    persistSettings,
  }
})

export default useSettingsStore
