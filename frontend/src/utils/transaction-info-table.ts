/******************************************************************************
 * NRCS 信息表数据结构 —— 对标 nrs.util.js NRS.createInfoTable(data, orderAlphabetically)。
 *
 * 参考实现通过 jQuery 拼接 `<table><tr><td>label</td><td>value</td></tr></table>`。
 * Vue3 环境下改为返回结构化行数组，由组件层用 el-descriptions 或自定义表格渲染。
 *
 * 字段名约定（对标参考中的 `_formatted_html` 后缀）：
 *   - 参考中 `data.xxx_formatted_html = "<a>...</a>"` 表示该字段值为 HTML
 *   - 这里改用 InfoRow.html 字段表示，避免在数据层拼 HTML
 *   - 链接类字段使用 LinkRef，由组件渲染为可点击元素
 ******************************************************************************/
import type { InfoValue, LinkRef } from './transaction-links'

// ============================================================================
// 类型定义
// ============================================================================

/** 信息表行（对标参考中 createInfoTable 的每一行） */
export interface InfoRow {
  /** 字段标签（已 i18n 化的展示文本） */
  label: string
  /** 字段值（字符串 / 链接 / HTML / 数量二元组） */
  value: InfoValue
  /** 是否跨整行（span=2，对标参考中某些字段的 colspan） */
  span?: boolean
}

// ============================================================================
// createInfoTable —— 对标 NRS.createInfoTable
// ============================================================================

/**
 * 将数据对象转换为信息表行数组（对标 NRS.createInfoTable(data, orderAlphabetically)）。
 *
 * 参考实现：遍历 data 对象的 key-value，按 key 排序（可选），生成 `<tr><td>key</td><td>value</td></tr>`。
 * key 中的下划线会被替换为空格并首字母大写。
 *
 * Vue3 实现：
 *   - 保留 key 的原始顺序（orderAlphabetically=false 默认）
 *   - 自动识别 value 类型：LinkRef / LinkRef[] / {html} / {quantity,decimals} / {amount} / 字符串/数字
 *   - key 转换为展示标签：下划线→空格，首字母大写
 *   - `_formatted_html` 后缀的字段自动识别为 HTML 值
 *
 * @param data 数据对象
 * @param orderAlphabetically 是否按字母排序
 * @returns 信息表行数组
 */
export function createInfoTable(
  data: Record<string, any>,
  orderAlphabetically = false,
): InfoRow[] {
  const keys = Object.keys(data)
  if (orderAlphabetically) {
    keys.sort((a, b) => a.localeCompare(b))
  }

  const rows: InfoRow[] = []
  for (const key of keys) {
    const value = data[key]
    if (value === undefined || value === null) continue

    // 跳过 version.* 字段（参考实现中 NRS.mergeMaps 会保留但 createInfoTable 不展示）
    if (key.startsWith('version.')) continue

    const label = keyToLabel(key)
    const processedValue = processValue(key, value)
    rows.push({ label, value: processedValue })
  }
  return rows
}

// ============================================================================
// 内部辅助函数
// ============================================================================

/**
 * 将字段 key 转换为展示标签（对标参考中 `$.t(key)` 的回退行为）。
 *
 * 规则：
 *   - 去掉 `_formatted_html` 后缀
 *   - 下划线替换为空格
 *   - 首字母大写
 *
 * @param key 字段名
 * @returns 展示标签
 */
function keyToLabel(key: string): string {
  let k = key
  // 去掉 _formatted_html 后缀
  if (k.endsWith('_formatted_html')) {
    k = k.substring(0, k.length - '_formatted_html'.length)
  }
  // 下划线替换为空格
  k = k.replace(/_/g, ' ')
  // 首字母大写
  if (k.length > 0) {
    k = k.charAt(0).toUpperCase() + k.slice(1)
  }
  return k
}

/**
 * 处理字段值，识别不同类型并转换为 InfoValue。
 *
 * @param key 字段名（用于识别 _formatted_html 后缀）
 * @param value 原始值
 * @returns InfoValue
 */
function processValue(key: string, value: any): InfoValue {
  // _formatted_html 后缀 → HTML 值
  if (key.endsWith('_formatted_html') && typeof value === 'string') {
    return { html: value }
  }

  // LinkRef 对象
  if (isLinkRef(value)) {
    return value as LinkRef
  }

  // LinkRef 数组
  if (Array.isArray(value) && value.length > 0 && value.every(isLinkRef)) {
    return value as LinkRef[]
  }

  // 数量二元组 [quantityQNT, decimals]（参考实现中 `data["quantity"] = [qnt, decimals]`）
  if (
    Array.isArray(value) &&
    value.length === 2 &&
    typeof value[0] === 'string' &&
    typeof value[1] === 'number'
  ) {
    return { quantity: value[0], decimals: value[1] }
  }

  // 字符串 / 数字
  if (typeof value === 'string' || typeof value === 'number') {
    return String(value)
  }

  // 布尔值转字符串
  if (typeof value === 'boolean') {
    return value ? 'true' : 'false'
  }

  // 其他类型转 JSON 字符串（兜底）
  return JSON.stringify(value)
}

/**
 * 类型守卫：判断对象是否为 LinkRef。
 */
function isLinkRef(obj: any): boolean {
  return (
    obj !== null &&
    typeof obj === 'object' &&
    !Array.isArray(obj) &&
    typeof obj.type === 'string' &&
    typeof obj.value === 'string'
  )
}

// ============================================================================
// mergeMaps —— 对标 NRS.mergeMaps(source, target, excludedKeys)
// ============================================================================

/**
 * 合并两个对象，可选择排除某些 key（对标 NRS.mergeMaps）。
 *
 * 用于 Shuffling 类型交易中将 attachment 字段合并进 data，
 * 同时排除 `version.*` 等元数据字段。
 *
 * @param source 源对象（如 transaction.attachment）
 * @param target 目标对象（如 data）
 * @param excludeKeys 排除的 key 集合
 * @returns 合并后的新对象（不修改入参）
 */
export function mergeMaps(
  source: Record<string, any>,
  target: Record<string, any>,
  excludeKeys: Record<string, boolean> = {},
): Record<string, any> {
  const result: Record<string, any> = { ...target }
  for (const key of Object.keys(source)) {
    if (excludeKeys[key]) continue
    result[key] = source[key]
  }
  return result
}
