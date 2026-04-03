/**
 * 格式化工具函数
 */

/**
 * 格式化以太坊地址（显示前6位和后4位）
 */
export function formatAddress(address: string, chars: number = 6): string {
  if (!address) return ''
  if (address.length <= chars * 2 + 4) return address

  const start = address.slice(0, chars)
  const end = address.slice(-chars)
  return `${start}...${end}`
}

/**
 * 格式化 Wei 为 ETH
 */
export function formatWeiToEth(wei: string | number, decimals: number = 4): string {
  if (!wei) return '0'

  const weiNum = typeof wei === 'string' ? BigInt(wei) : BigInt(wei)
  const eth = Number(weiNum) / 1e18

  return eth.toFixed(decimals)
}

/**
 * 格式化 GWei 为 ETH
 */
export function formatGweiToEth(gwei: string | number, decimals: number = 4): string {
  if (!gwei) return '0'

  const gweiNum = typeof gwei === 'string' ? BigInt(gwei) : BigInt(gwei)
  const eth = Number(gweiNum) / 1e9

  return eth.toFixed(decimals)
}

/**
 * 格式化时间（ISO 8601 转相对时间或指定格式）
 */
export function formatTime(
  isoString: string | Date | number,
  options: Intl.DateTimeFormatOptions = {}
): string {
  if (!isoString) return ''

  const date = typeof isoString === 'number'
    ? new Date(isoString * 1000)
    : new Date(isoString)

  if (isNaN(date.getTime())) return ''

  const defaultOptions: Intl.DateTimeFormatOptions = {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false,
    ...options
  }

  return new Intl.DateTimeFormat('zh-CN', defaultOptions).format(date)
}

/**
 * 格式化相对时间（几天前、几小时前等）
 */
export function formatRelativeTime(
  isoString: string | Date | number,
  locale: string = 'zh-CN'
): string {
  if (!isoString) return ''

  const date = typeof isoString === 'number'
    ? new Date(isoString * 1000)
    : new Date(isoString)

  if (isNaN(date.getTime())) return ''

  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffSec = Math.floor(diffMs / 1000)
  const diffMin = Math.floor(diffSec / 60)
  const diffHour = Math.floor(diffMin / 60)
  const diffDay = Math.floor(diffHour / 24)
  const diffMonth = Math.floor(diffDay / 30)
  const diffYear = Math.floor(diffDay / 365)

  const rtf = new Intl.RelativeTimeFormat(locale, { numeric: 'auto' })

  if (diffSec < 60) {
    return rtf.format(-diffSec, 'second')
  } else if (diffMin < 60) {
    return rtf.format(-diffMin, 'minute')
  } else if (diffHour < 24) {
    return rtf.format(-diffHour, 'hour')
  } else if (diffDay < 30) {
    return rtf.format(-diffDay, 'day')
  } else if (diffMonth < 12) {
    return rtf.format(-diffMonth, 'month')
  } else {
    return rtf.format(-diffYear, 'year')
  }
}

/**
 * 格式化文件大小
 */
export function formatFileSize(bytes: number, decimals: number = 2): string {
  if (bytes === 0) return '0 B'

  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))

  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(decimals))} ${sizes[i]}`
}

/**
 * 格式化数字（添加千分位）
 */
export function formatNumber(num: number | string, decimals: number = 0): string {
  const n = typeof num === 'string' ? parseFloat(num) : num
  return n.toLocaleString('en-US', {
    minimumFractionDigits: decimals,
    maximumFractionDigits: decimals
  })
}

/**
 * 格式化哈希值（显示前8位和后8位）
 */
export function formatHash(hash: string): string {
  if (!hash) return ''
  if (hash.length <= 20) return hash

  return `${hash.slice(0, 8)}...${hash.slice(-8)}`
}

/**
 * 格式化 Gas 价格（GWei）
 */
export function formatGasPrice(wei: string | number): string {
  if (!wei) return '0'

  const weiNum = typeof wei === 'string' ? BigInt(wei) : BigInt(wei)
  const gwei = Number(weiNum) / 1e9

  if (gwei < 1) {
    return `${gwei.toFixed(2)} Gwei`
  } else if (gwei < 1000) {
    return `${gwei.toFixed(1)} Gwei`
  } else {
    // 超过 1000 Gwei 显示为 ETH
    return `${(gwei / 1000).toFixed(4)} ETH`
  }
}

/**
 * 格式化区块高度
 */
export function formatBlockNumber(number: number): string {
  return `#${formatNumber(number)}`
}

/**
 * 格式化百分比
 */
export function formatPercent(value: number, decimals: number = 2): string {
  return `${value.toFixed(decimals)}%`
}

/**
 * 解析 ETH 为 Wei
 */
export function parseEthToWei(eth: string | number): bigint {
  const ethNum = typeof eth === 'string' ? parseFloat(eth) : eth
  return BigInt(Math.round(ethNum * 1e18))
}

/**
 * 克隆对象（深拷贝）
 */
export function deepClone<T>(obj: T): T {
  return JSON.parse(JSON.stringify(obj))
}
