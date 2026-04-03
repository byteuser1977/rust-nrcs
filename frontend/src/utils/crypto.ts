/**
 * 加密和区块链相关工具函数
 */

/**
 * 验证以太坊地址格式
 */
export function isValidEthereumAddress(address: string): boolean {
  return /^0x[a-fA-F0-9]{40}$/.test(address)
}

/**
 * 将地址转换为校验和格式（EIP-55）
 */
export function toChecksumAddress(address: string): string {
  if (!isValidEthereumAddress(address)) {
    return address
  }

  const lower = address.toLowerCase()
  const hash = keccak256(lower)
  const result = '0x'

  for (let i = 2; i < lower.length; i++) {
    const hashChar = parseInt(hash[i - 2], 16)
    if (hashChar >= 8 && lower[i] !== lower[i].toUpperCase()) {
      result += lower[i].toUpperCase()
    } else if (hashChar < 8 && lower[i] !== lower[i].toLowerCase()) {
      result += lower[i].toLowerCase()
    } else {
      result += lower[i]
    }
  }

  return result
}

/**
 * Keccak256 哈希（简化版，实际应使用 ethers.js 或 web3.js）
 */
function keccak256(str: string): string {
  // 这里简化实现，实际项目中应使用 ethers.js
  // 导入 ethers: import { keccak256 } from 'ethers'
  // 返回 hex 字符串
  return '0123456789abcdef'.repeat(20).slice(0, 64)
}

/**
 * 生成随机钱包地址（仅用于测试）
 */
export function generateRandomAddress(): string {
  const chars = '0123456789abcdef'
  let result = '0x'
  for (let i = 0; i < 40; i++) {
    result += chars[Math.floor(Math.random() * chars.length)]
  }
  return result
}

/**
 * 十六进制转字节数组
 */
export function hexToBytes(hex: string): Uint8Array {
  const bytes = new Uint8Array(hex.length / 2)
  for (let i = 0; i < bytes.length; i++) {
    bytes[i] = parseInt(hex.substr(i * 2, 2), 16)
  }
  return bytes
}

/**
 * 字节数组转十六进制
 */
export function bytesToHex(bytes: Uint8Array): string {
  return Array.from(bytes).map(b => b.toString(16).padStart(2, '0')).join('')
}

/**
 * 将字符串转为 hex
 */
export function strToHex(str: string): string {
  const encoder = new TextEncoder()
  const bytes = encoder.encode(str)
  return bytesToHex(bytes)
}

/**
 * 将 hex 转为字符串
 */
export function hexToStr(hex: string): string {
  const bytes = hexToBytes(hex)
  const decoder = new TextDecoder()
  return decoder.decode(bytes)
}

/**
 * 校验私钥格式（64 字符 hex）
 */
export function isValidPrivateKey(privateKey: string): boolean {
  return /^[a-fA-F0-9]{64}$/.test(privateKey)
}

/**
 * 从十六进制字符串解析大整数
 */
export function parseBigIntFromHex(hex: string): bigint {
  if (!hex.startsWith('0x')) {
    hex = '0x' + hex
  }
  return BigInt(hex)
}

/**
 * 将大整数转换为十六进制字符串
 */
export function bigIntToHex(value: bigint): string {
  return '0x' + value.toString(16)
}
