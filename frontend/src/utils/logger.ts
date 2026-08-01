/**
 * NRCS 调试日志工具。
 *
 * 端口自参考实现 `nrs.console.js`（175 行）：
 *   - `logConsole`：带时间戳的日志输出
 *   - `logException`：异常日志（含堆栈）
 *   - `addToConsole`：API 请求/响应日志（敏感字段脱敏）
 *   - `queryStringToObject`：查询字符串解析（secretPhrase 脱敏）
 *
 * 与参考的差异：
 *   - 参考实现使用 `window.open` 打开独立窗口；Vue3 版改为内存环形缓冲区 + DebugConsole 组件
 *   - 敏感字段脱敏列表扩展：secretPhrase/adminPassword/prvKey/privateKey
 *   - 日志级别过滤（isLogConsole/setLogConsoleLevel）
 *
 * 用法：
 *   import { logger } from '@/utils/logger'
 *   logger.addToConsole('/nrs?requestType=getBlock', 'GET', null, response, false)
 */

/** 日志条目类型 */
export interface ConsoleEntry {
  /** 唯一 ID */
  id: number
  /** 请求 URL（已去除 random 参数） */
  url: string
  /** 请求方法（GET/POST） */
  method: string
  /** 请求时间戳 */
  timestamp: Date
  /** 请求数据（已脱敏） */
  data?: Record<string, any> | null
  /** 响应数据（已脱敏） */
  response?: any
  /** 是否为错误响应 */
  isError: boolean
}

/** 需要脱敏的敏感字段名（小写匹配） */
const SENSITIVE_FIELDS = new Set([
  'secretphrase',
  'secret',
  'adminpassword',
  'prvkey',
  'privatekey',
  'password',
])

/** 最大缓冲区条目数（超出后丢弃最早的） */
const MAX_ENTRIES = 500

/**
 * NRCS 调试日志记录器（单例）。
 *
 * 端口自 nrs.console.js 的 NRS.addToConsole + NRS.logConsole。
 * 维护一个环形缓冲区，供 DebugConsole 组件实时展示。
 */
class NrcsLogger {
  /** 日志条目缓冲区 */
  private entries: ConsoleEntry[] = []

  /** 下一个条目 ID */
  private nextId = 1

  /** 日志级别（1=基本/5=详细/10=全部） */
  private level = 1

  /** 监听器列表（供组件订阅变化） */
  private listeners: Set<() => void> = new Set()

  /**
   * 获取当前所有日志条目。
   */
  getEntries(): ConsoleEntry[] {
    return this.entries
  }

  /**
   * 获取当前日志级别。
   */
  getLevel(): number {
    return this.level
  }

  /**
   * 设置日志级别。
   *
   * @param logLevel - 1=基本 / 5=详细 / 10=全部
   */
  setLevel(logLevel: number): void {
    this.level = logLevel
  }

  /**
   * 判断指定级别的日志是否应该输出。
   *
   * @param msgLevel - 消息级别
   * @returns 是否应该输出
   */
  isLogConsole(msgLevel: number): boolean {
    return msgLevel <= this.level
  }

  /**
   * 通用日志输出（对标 NRS.logConsole）。
   * 同时写入浏览器 console 和内部缓冲区。
   *
   * @param msg - 日志消息
   */
  logConsole(msg: string): void {
    const prefix = `${new Date().toISOString()} `
    const line = prefix + msg
    // eslint-disable-next-line no-console
    console.log(line)
  }

  /**
   * 异常日志输出（对标 NRS.logException）。
   *
   * @param e - 异常对象
   */
  logException(e: Error | unknown): void {
    if (e instanceof Error) {
      this.logConsole(e.message)
      if (e.stack) {
        this.logConsole(e.stack)
      }
    } else {
      this.logConsole(String(e))
    }
  }

  /**
   * 添加 API 请求/响应日志条目（对标 NRS.addToConsole）。
   *
   * @param url - 请求 URL
   * @param method - 请求方法（GET/POST）
   * @param data - 请求参数（将被脱敏）
   * @param response - 响应数据（将被脱敏）
   * @param isError - 是否为错误响应
   */
  addToConsole(
    url: string,
    method: string,
    data: Record<string, any> | string | null,
    response: any,
    isError: boolean,
  ): void {
    // 去除 URL 中的 random 参数（对标 nrs.console.js:104）
    const cleanUrl = url.replace(/&random=[.\d]+/, '')

    // 解析并脱敏请求数据
    let sanitizedData: Record<string, any> | null = null
    if (data) {
      if (typeof data === 'string') {
        sanitizedData = this.queryStringToObject(data)
      } else {
        sanitizedData = this.sanitizeObject({ ...data })
      }
    }

    // 脱敏响应数据
    const sanitizedResponse = response ? this.sanitizeObject(
      typeof response === 'string' ? this.tryParseJSON(response) : response
    ) : undefined

    const entry: ConsoleEntry = {
      id: this.nextId++,
      url: cleanUrl,
      method: method.toUpperCase(),
      timestamp: new Date(),
      data: sanitizedData,
      response: sanitizedResponse,
      isError,
    }

    // 添加到缓冲区，超出上限则丢弃最早的
    this.entries.push(entry)
    if (this.entries.length > MAX_ENTRIES) {
      this.entries = this.entries.slice(-MAX_ENTRIES)
    }

    // 通知监听器
    this.notifyListeners()

    // 同时输出到浏览器 console（仅详细级别）
    if (this.isLogConsole(5)) {
      this.logConsole(`${cleanUrl} (${method})`)
      if (sanitizedData) {
        this.logConsole(`  Request: ${JSON.stringify(sanitizedData)}`)
      }
      if (sanitizedResponse) {
        this.logConsole(`  Response: ${JSON.stringify(sanitizedResponse)}`)
      }
    }
  }

  /**
   * 清空所有日志条目。
   */
  clear(): void {
    this.entries = []
    this.notifyListeners()
  }

  /**
   * 订阅日志变化（供 Vue 组件响应式更新）。
   *
   * @param listener - 变化回调
   * @returns 取消订阅函数
   */
  subscribe(listener: () => void): () => void {
    this.listeners.add(listener)
    return () => {
      this.listeners.delete(listener)
    }
  }

  /**
   * 查询字符串解析为对象（对标 NRS.queryStringToObject）。
   * secretPhrase 等敏感字段会被脱敏为 ***。
   *
   * @param qs - 查询字符串（如 "a=1&b=2&secretPhrase=xxx"）
   * @returns 解析后的对象
   */
  queryStringToObject(qs: string): Record<string, any> {
    const parts = qs.split('&')
    const obj: Record<string, any> = {}

    for (const part of parts) {
      const p = part.split('=')
      if (p.length !== 2) continue
      const key = p[0]
      const value = decodeURIComponent(p[1].replace(/\+/g, ' '))
      obj[key] = this.isSensitiveField(key) ? '***' : value
    }

    return obj
  }

  /**
   * 递归脱敏对象中的敏感字段。
   */
  private sanitizeObject(obj: any): any {
    if (obj === null || obj === undefined) return obj
    if (typeof obj !== 'object') return obj
    if (Array.isArray(obj)) {
      return obj.map((item) => this.sanitizeObject(item))
    }

    const result: Record<string, any> = {}
    for (const key of Object.keys(obj)) {
      if (this.isSensitiveField(key)) {
        result[key] = '***'
      } else {
        result[key] = this.sanitizeObject(obj[key])
      }
    }
    return result
  }

  /**
   * 判断字段名是否为敏感字段（不区分大小写）。
   */
  private isSensitiveField(fieldName: string): boolean {
    return SENSITIVE_FIELDS.has(fieldName.toLowerCase())
  }

  /**
   * 尝试解析 JSON 字符串，失败则返回原始字符串。
   */
  private tryParseJSON(str: string): any {
    try {
      return JSON.parse(str)
    } catch {
      return str
    }
  }

  /**
   * 通知所有监听器日志已更新。
   */
  private notifyListeners(): void {
    for (const listener of this.listeners) {
      listener()
    }
  }
}

/** 全局 logger 单例 */
export const logger = new NrcsLogger()
