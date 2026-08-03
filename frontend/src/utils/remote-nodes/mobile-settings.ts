/**
 * 移动端设置读写工具。
 *
 * 端口自参考实现 `nrs.mobile.js` 中对 `NRS.mobileSettings` 的 localStorage 持久化逻辑。
 * 本模块集中管理 `mobile_settings` 键的读写，供 RemoteNodesManager 与其他模块复用，
 * 避免各处重复解析 localStorage。
 *
 * 适配说明（Vue3 与参考的差异）：
 * - 参考使用全局 `NRS.mobileSettings` 对象，本模块改为函数式读取，每次调用返回最新快照。
 * - 默认值对标 `nrs.mobile.js:5-15` 的 `NRS.mobileSettings` 初始定义。
 */

/** 移动端设置结构（对标 NRS.mobileSettings） */
export interface MobileSettings {
  /** 是否显示"记住我"勾选框 */
  is_check_remember_me: boolean
  /** 是否存储已记住的密码 */
  is_store_remembered_passphrase: boolean
  /** 是否启用移动应用模拟（Web 端模拟移动钱包行为） */
  is_simulate_app: boolean
  /** 是否测试网 */
  is_testnet: boolean
  /** 远程节点地址（手动配置时使用，为空则从 bootstrap 列表随机选取） */
  remote_node_address: string
  /** 远程节点端口 */
  remote_node_port: number
  /** 远程节点是否使用 SSL */
  is_remote_node_ssl: boolean
  /** 验证者数量（响应交叉验证的节点数，0-3） */
  validators_count: number
  /** 引导节点数量（bootstrap 阶段需连接成功的节点数，0-5） */
  bootstrap_nodes_count: number
}

/** localStorage 键名（对标 nrs.mobile.js 的 'mobile_settings'） */
const STORAGE_KEY = 'mobile_settings'

/** 默认值（对标 nrs.mobile.js:5-15） */
export const DEFAULT_MOBILE_SETTINGS: MobileSettings = {
  is_check_remember_me: false,
  is_store_remembered_passphrase: false,
  is_simulate_app: false,
  is_testnet: false,
  remote_node_address: '',
  remote_node_port: 0,
  is_remote_node_ssl: false,
  validators_count: 0,
  bootstrap_nodes_count: 0,
}

/**
 * 从 localStorage 读取移动端设置。
 *
 * 对标 `nrs.mobile.js:22` 中 `show.bs.modal` 回调对 `NRS.mobileSettings` 的加载。
 * 解析失败或缺失字段时回退到默认值。
 *
 * @returns 合并默认值后的移动端设置快照
 */
export function getMobileSettings(): MobileSettings {
  if (typeof localStorage === 'undefined') {
    return { ...DEFAULT_MOBILE_SETTINGS }
  }
  const stored = localStorage.getItem(STORAGE_KEY)
  if (!stored) {
    return { ...DEFAULT_MOBILE_SETTINGS }
  }
  try {
    const parsed = JSON.parse(stored)
    return {
      ...DEFAULT_MOBILE_SETTINGS,
      is_check_remember_me: !!parsed.is_check_remember_me,
      is_store_remembered_passphrase: !!parsed.is_store_remembered_passphrase,
      is_simulate_app: !!parsed.is_simulate_app,
      is_testnet: !!parsed.is_testnet,
      remote_node_address: parsed.remote_node_address || '',
      remote_node_port: Number(parsed.remote_node_port) || 0,
      is_remote_node_ssl: !!parsed.is_remote_node_ssl,
      validators_count: Number(parsed.validators_count) || 0,
      bootstrap_nodes_count: Number(parsed.bootstrap_nodes_count) || 0,
    }
  } catch {
    return { ...DEFAULT_MOBILE_SETTINGS }
  }
}

/**
 * 持久化移动端设置到 localStorage。
 *
 * 对标 `nrs.mobile.js:99` 的 `NRS.setJSONItem('mobile_settings', settings)`。
 *
 * @param settings - 完整的移动端设置
 */
export function setMobileSettings(settings: MobileSettings): void {
  if (typeof localStorage === 'undefined') return
  localStorage.setItem(STORAGE_KEY, JSON.stringify(settings))
}
