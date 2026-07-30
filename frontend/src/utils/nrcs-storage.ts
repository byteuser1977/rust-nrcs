/**
 * NRCS 客户端存储层。
 *
 * 端口自 `/Volumes/DATA/data/develop/git/nrcs/nrcs-main/html/www/ui/js/nrs.localstorage.js`。
 *
 * 设计：
 * - IndexedDB 优先（每个账户一个数据库 `nrcs-{accountId}`，table 为 object store），实现账户级数据隔离
 * - IndexedDB 不可用时回退到 localStorage，key 形如 `{table}.{account}` 实现账户隔离
 * - 用 async/Promise 替代原 callback 风格，保留原 select/insert/update/delete 语义
 *
 * 该层供联系人、资产余额缓存、未确认交易缓存、资产收藏等需要按账户隔离的本地数据使用。
 * 通用 localStorage 字符串/JSON 读写（getStrItem/setJSONItem 等）不经账户隔离，供全局设置使用。
 */

/** 查询条件：数组，每个元素为一组字段匹配（OR 语义，匹配任意一组即选中） */
export type StorageQuery = Array<Record<string, unknown>>

/** 当前账户 ID（用于账户作用域隔离） */
let currentAccount = ''

/** IndexedDB 数据库版本 */
const DB_VERSION = 1

/** 默认创建的 object store 列表（按 NRCS 客户端常用表） */
const DEFAULT_STORES = [
  'contacts',          // 联系人
  'assets',            // 资产余额缓存
  'asset_bookmarks',   // 资产收藏
  'asset_groups',      // 资产分组
  'unconfirmed_transactions', // 未确认交易缓存
  'followed_polls',    // 关注的投票
  'data_cache',        // 通用数据缓存
]

/**
 * 检测当前环境是否支持 IndexedDB。
 * @returns 是否支持 IndexedDB
 */
export function isIndexedDBSupported(): boolean {
  return typeof indexedDB !== 'undefined'
}

/**
 * 设置当前账户上下文（用于账户作用域隔离）。
 * 切换账户时调用，后续 storageSelect/Insert/Update/Delete 将作用于该账户的数据库。
 * @param account 账户 ID（数字形式）
 */
export function setStorageAccount(account: string): void {
  currentAccount = account || ''
}

/**
 * 获取当前账户上下文。
 * @returns 当前账户 ID
 */
export function getStorageAccount(): string {
  return currentAccount
}

/**
 * 构造账户作用域的 localStorage key。
 * 账户为空时返回原 key（全局作用域）。
 * @param key 原 key
 * @returns 账户作用域 key
 */
function getAccountKey(key: string): string {
  if (!currentAccount) return key
  return `${key}.${currentAccount}`
}

/**
 * 构造账户作用域的 IndexedDB 数据库名称。
 * @param account 账户 ID
 * @returns 数据库名称
 */
function getDbName(account: string): string {
  return `nrcs-${account}`
}

/**
 * 打开（或创建）指定账户的 IndexedDB 数据库。
 * 首次打开时创建 DEFAULT_STORES 中列出的 object store。
 * @param account 账户 ID
 * @returns IDBDatabase 实例
 */
function openAccountDB(account: string): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    if (!isIndexedDBSupported()) {
      reject(new Error('IndexedDB not supported'))
      return
    }
    const request = indexedDB.open(getDbName(account), DB_VERSION)
    request.onupgradeneeded = () => {
      const db = request.result
      for (const storeName of DEFAULT_STORES) {
        if (!db.objectStoreNames.contains(storeName)) {
          db.createObjectStore(storeName, { autoIncrement: true })
        }
      }
    }
    request.onsuccess = () => resolve(request.result)
    request.onerror = () => reject(request.error)
  })
}

/**
 * 初始化用户数据库（登录或切换账户时调用）。
 * 创建账户的 IndexedDB 数据库与默认 object store。
 * @param account 账户 ID
 */
export async function initUserDB(account: string): Promise<void> {
  if (!account) return
  setStorageAccount(account)
  if (!isIndexedDBSupported()) return
  await openAccountDB(account)
}

/**
 * 删除用户数据库（登出或清除用户数据时调用）。
 * @param account 账户 ID
 */
export function dropUserDB(account: string): Promise<void> {
  return new Promise((resolve, reject) => {
    if (!isIndexedDBSupported()) {
      resolve()
      return
    }
    const request = indexedDB.deleteDatabase(getDbName(account))
    request.onsuccess = () => resolve()
    request.onerror = () => reject(request.error)
    request.onblocked = () => resolve()
  })
}

/**
 * 判断 item 是否匹配查询条件（OR 语义：匹配任一条件组即选中）。
 * @param item 待检测对象
 * @param query 查询条件数组
 * @returns 是否匹配
 */
function matchesQuery(item: Record<string, unknown>, query?: StorageQuery): boolean {
  if (!query || query.length === 0) return true
  return query.some((condition) =>
    Object.keys(condition).every((key) => item[key] === condition[key])
  )
}

/**
 * 从表中查询数据。
 * IndexedDB 不可用时回退到 localStorage（账户作用域 JSON）。
 * @param table 表名
 * @param query 查询条件（可选，空表示全部）
 * @returns 匹配的记录数组
 */
export async function storageSelect<T = Record<string, unknown>>(
  table: string,
  query?: StorageQuery
): Promise<T[]> {
  if (isIndexedDBSupported() && currentAccount) {
    const db = await openAccountDB(currentAccount)
    return new Promise<T[]>((resolve, reject) => {
      if (!db.objectStoreNames.contains(table)) {
        resolve([])
        return
      }
      const tx = db.transaction(table, 'readonly')
      const store = tx.objectStore(table)
      const request = store.getAll()
      request.onsuccess = () => {
        const items = (request.result as T[]) || []
        const filtered = query && query.length > 0 ? items.filter((it) => matchesQuery(it as Record<string, unknown>, query)) : items
        resolve(filtered)
      }
      request.onerror = () => reject(request.error)
    })
  }

  // localStorage 回退
  const items = getAccountJSONItem<T[]>(table) || []
  const filtered = query && query.length > 0 ? items.filter((it) => matchesQuery(it as Record<string, unknown>, query)) : items
  return filtered
}

/**
 * 向表中插入数据。
 * 主键字段用于去重（已存在相同主键值则报错）。
 * @param table 表名
 * @param key 主键字段名（用于去重判断）
 * @param data 待插入数据（单条或多条）
 * @param isAutoIncrement 是否自动递增 id
 * @returns 插入后的全表数据
 */
export async function storageInsert<T = Record<string, unknown>>(
  table: string,
  key: string,
  data: T | T[],
  isAutoIncrement = false
): Promise<T[]> {
  const dataArray = Array.isArray(data) ? data : [data]

  if (isIndexedDBSupported() && currentAccount) {
    const db = await openAccountDB(currentAccount)
    // 确保表存在
    if (!db.objectStoreNames.contains(table)) {
      await createObjectStore(db, table)
    }
    // 去重检查
    const existing = await storageSelect<T>(table)
    for (const item of dataArray) {
      const itemKey = (item as Record<string, unknown>)[key]
      if (itemKey !== undefined && existing.some((it) => (it as Record<string, unknown>)[key] === itemKey)) {
        throw new Error(`Key already exists: ${String(itemKey)}`)
      }
    }
    return new Promise<T[]>((resolve, reject) => {
      const tx = db.transaction(table, 'readwrite')
      const store = tx.objectStore(table)
      let lastId = 0
      if (isAutoIncrement) {
        const allReq = store.getAll()
        allReq.onsuccess = () => {
          const all = (allReq.result as Array<Record<string, unknown>>) || []
          // eslint-disable-next-line @typescript-eslint/no-explicit-any
          lastId = all.length > 0 ? Math.max(...all.map((it: any) => Number(it.id) || 0)) : 0
          for (const item of dataArray) {
            const record = { ...(item as Record<string, unknown>) }
            if (record.id !== undefined) {
              reject(new Error('Cannot use auto increment id since data already contains id value'))
              return
            }
            record.id = ++lastId
            store.add(record)
          }
        }
      } else {
        for (const item of dataArray) {
          store.add(item)
        }
      }
      tx.oncomplete = () => resolve(storageSelect<T>(table).then((r) => r) as unknown as T[])
      tx.onerror = () => reject(tx.error)
    })
  }

  // localStorage 回退
  const items = getAccountJSONItem<T[]>(table) || []
  let nextId = items.length > 0 ? Math.max(...items.map((it) => Number((it as Record<string, unknown>).id) || 0)) : 0
  for (const item of dataArray) {
    const itemRecord = item as Record<string, unknown>
    const itemKey = itemRecord[key]
    if (itemKey !== undefined && items.some((it) => (it as Record<string, unknown>)[key] === itemKey)) {
      throw new Error(`Key already exists: ${String(itemKey)}`)
    }
    if (isAutoIncrement) {
      if (itemRecord.id !== undefined) {
        throw new Error('Cannot use auto increment id since data already contains id value')
      }
      itemRecord.id = ++nextId
    }
    items.push(item)
  }
  setAccountJSONItem(table, items)
  return items
}

/**
 * 创建 object store（若不存在）。
 * @param db 数据库实例
 * @param storeName store 名称
 */
function createObjectStore(db: IDBDatabase, storeName: string): Promise<void> {
  return new Promise((resolve, reject) => {
    db.close()
    const request = indexedDB.open(getDbName(currentAccount), DB_VERSION + 1)
    request.onupgradeneeded = () => {
      const newDb = request.result
      if (!newDb.objectStoreNames.contains(storeName)) {
        newDb.createObjectStore(storeName, { autoIncrement: true })
      }
    }
    request.onsuccess = () => {
      request.result.close()
      resolve()
    }
    request.onerror = () => reject(request.error)
  })
}

/**
 * 更新表中匹配查询条件的记录。
 * @param table 表名
 * @param data 待更新字段
 * @param query 匹配条件
 * @returns 更新后的全表数据
 */
export async function storageUpdate<T = Record<string, unknown>>(
  table: string,
  data: Partial<T>,
  query: StorageQuery
): Promise<T[]> {
  if (!query || query.length === 0) {
    throw new Error('No update query')
  }

  if (isIndexedDBSupported() && currentAccount) {
    const db = await openAccountDB(currentAccount)
    if (!db.objectStoreNames.contains(table)) {
      throw new Error('No items to update')
    }
    return new Promise<T[]>((resolve, reject) => {
      const tx = db.transaction(table, 'readwrite')
      const store = tx.objectStore(table)
      const cursorReq = store.openCursor()
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      cursorReq.onsuccess = () => {
        const cursor = cursorReq.result
        if (cursor) {
          const value = cursor.value as Record<string, unknown>
          if (matchesQuery(value, query)) {
            Object.keys(data as Record<string, unknown>).forEach((dataKey) => {
              value[dataKey] = (data as Record<string, unknown>)[dataKey]
            })
            cursor.update(value)
          }
          cursor.continue()
        }
      }
      tx.oncomplete = () => resolve(storageSelect<T>(table).then((r) => r) as unknown as T[])
      tx.onerror = () => reject(tx.error)
    })
  }

  // localStorage 回退
  const items = getAccountJSONItem<Record<string, unknown>[]>(table) || []
  if (items.length === 0) {
    throw new Error('No items to update')
  }
  for (const item of items) {
    if (matchesQuery(item, query)) {
      Object.keys(data as Record<string, unknown>).forEach((dataKey) => {
        item[dataKey] = (data as Record<string, unknown>)[dataKey]
      })
    }
  }
  setAccountJSONItem(table, items)
  return items as T[]
}

/**
 * 删除表中匹配查询条件的记录。
 * @param table 表名
 * @param query 匹配条件
 * @returns 删除后的全表数据
 */
export async function storageDelete<T = Record<string, unknown>>(
  table: string,
  query: StorageQuery
): Promise<T[]> {
  if (isIndexedDBSupported() && currentAccount) {
    const db = await openAccountDB(currentAccount)
    if (!db.objectStoreNames.contains(table)) {
      return []
    }
    return new Promise<T[]>((resolve, reject) => {
      const tx = db.transaction(table, 'readwrite')
      const store = tx.objectStore(table)
      const cursorReq = store.openCursor()
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      cursorReq.onsuccess = () => {
        const cursor = cursorReq.result
        if (cursor) {
          const value = cursor.value as Record<string, unknown>
          if (matchesQuery(value, query)) {
            cursor.delete()
          }
          cursor.continue()
        }
      }
      tx.oncomplete = () => resolve(storageSelect<T>(table).then((r) => r) as unknown as T[])
      tx.onerror = () => reject(tx.error)
    })
  }

  // localStorage 回退
  const items = getAccountJSONItem<Record<string, unknown>[]>(table) || []
  const remaining = items.filter((it) => !matchesQuery(it, query))
  setAccountJSONItem(table, remaining)
  return remaining as T[]
}

/**
 * 删除整张表（账户作用域）。
 * @param table 表名
 */
export function localStorageDrop(table: string): void {
  if (isIndexedDBSupported() && currentAccount) {
    // IndexedDB 模式：清空 object store（异步，fire-and-forget）
    openAccountDB(currentAccount)
      .then((db) => {
        if (!db.objectStoreNames.contains(table)) return
        const tx = db.transaction(table, 'readwrite')
        tx.objectStore(table).clear()
      })
      .catch(() => {
        // 静默失败
      })
  }
  removeAccountItem(table)
}

// ===================== 通用 localStorage（不经账户隔离） =====================

/**
 * 读取字符串 localStorage 项。
 * @param key 键名
 * @returns 值（字符串或 null）
 */
export function getStrItem(key: string): string | null {
  return localStorage.getItem(key)
}

/**
 * 写入字符串 localStorage 项。
 * @param key 键名
 * @param data 值
 */
export function setStrItem(key: string, data: string): void {
  localStorage.setItem(key, data)
}

/**
 * 读取 JSON localStorage 项。
 * @param key 键名
 * @returns 反序列化后的值（或 null）
 */
export function getJSONItem<T = unknown>(key: string): T | null {
  const raw = localStorage.getItem(key)
  if (raw === null) return null
  try {
    return JSON.parse(raw) as T
  } catch {
    return null
  }
}

/**
 * 写入 JSON localStorage 项。
 * @param key 键名
 * @param data 值
 */
export function setJSONItem(key: string, data: unknown): void {
  localStorage.setItem(key, JSON.stringify(data))
}

/**
 * 删除 localStorage 项。
 * @param key 键名
 */
export function removeItem(key: string): void {
  localStorage.removeItem(key)
}

// ===================== 账户作用域 localStorage =====================

/**
 * 读取账户作用域 JSON 项。
 * key 自动加上当前账户前缀（`{key}.{account}`）。
 * @param key 原 key
 * @returns 反序列化后的值（或 null）
 */
export function getAccountJSONItem<T = unknown>(key: string): T | null {
  return getJSONItem<T>(getAccountKey(key))
}

/**
 * 写入账户作用域 JSON 项。
 * @param key 原 key
 * @param data 值
 */
export function setAccountJSONItem(key: string, data: unknown): void {
  setJSONItem(getAccountKey(key), data)
}

/**
 * 删除账户作用域项。
 * @param key 原 key
 */
export function removeAccountItem(key: string): void {
  removeItem(getAccountKey(key))
}
