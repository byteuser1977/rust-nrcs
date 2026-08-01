/**
 * useAssetBookmarks composable —— 资产收藏（书签）与分组管理。
 *
 * 对标 `nrs.assetexchange.js` 中的资产收藏相关函数：
 *   - `NRS.cacheAsset`（:131）：内存缓存
 *   - `NRS.saveAssetBookmarks`（:247）：写入 IndexedDB `assets` 表
 *   - `NRS.forms.addAssetBookmark`（:153）：按 ID/RS 添加收藏
 *   - 移除收藏（:1267 `remove_from_bookmarks`）：调 `storageDelete`
 *   - 加入/移出分组（:1218 `add_to_group` / :1254 `remove_from_group`）：调 `storageUpdate`
 *   - 重命名分组（:1186 `assetExchangeChangeGroupName`）：调 `storageUpdate`
 *
 * 数据存储：使用 `utils/nrcs-storage.ts` 的 IndexedDB 层，表名 `assets`，主键 `asset`。
 * 账户级隔离由 storage 层根据 `setStorageAccount` 自动处理。
 */
import { ref, computed } from 'vue'
import { nrcsApi, type NrcsAsset } from '@/api/modules'
import {
  storageSelect,
  storageInsert,
  storageUpdate,
  storageDelete,
  type StorageQuery,
} from '@/utils/nrcs-storage'

/** 资产收藏记录（对标 NRS.saveAssetBookmarks 中的 newAsset 结构） */
export interface AssetBookmark {
  /** 资产 ID（数字字符串） */
  asset: string
  /** 资产名称 */
  name: string
  /** 资产描述 */
  description: string
  /** 发行方账户 ID */
  account: string
  /** 发行方 RS 地址 */
  accountRS: string
  /** 资产总量 QNT */
  quantityQNT: string
  /** 小数位数 */
  decimals: number
  /** 分组名称（空字符串表示未分组） */
  groupName: string
}

/** IndexedDB 表名（与 nrcs-storage.ts DEFAULT_STORES 一致） */
const BOOKMARKS_TABLE = 'assets'

/**
 * 资产收藏与分组管理 composable。
 *
 * 提供：
 *   - 加载/保存/删除收藏
 *   - 按 ID 或发行方 RS 添加收藏
 *   - 分组管理（加入/移出/重命名/列举）
 *   - 内存缓存（cacheAsset）+ 持久化（IndexedDB）
 *   - 搜索过滤
 */
export function useAssetBookmarks() {
  /** 内存缓存的收藏列表（对标 NRS.assets 全局变量） */
  const bookmarks = ref<AssetBookmark[]>([])

  /** 已缓存的资产 ID 集合（对标 NRS.assetIds，去重用） */
  const cachedAssetIds = ref<Set<string>>(new Set())

  /** 是否正在加载 */
  const isLoading = ref(false)

  /** 错误信息 */
  const error = ref<string>('')

  /** 所有分组名称（按字母序排序，不含空分组） */
  const groupNames = computed(() => {
    const names = new Set<string>()
    for (const b of bookmarks.value) {
      if (b.groupName) names.add(b.groupName)
    }
    return Array.from(names).sort((a, b) =>
      a.toLowerCase().localeCompare(b.toLowerCase()),
    )
  })

  /**
   * 将资产加入内存缓存（去重）。
   * 对标 NRS.cacheAsset（:131-151）。
   *
   * @param asset 资产信息
   */
  function cacheAsset(asset: AssetBookmark): void {
    const assetId = String(asset.asset)
    if (cachedAssetIds.value.has(assetId)) return
    cachedAssetIds.value.add(assetId)
    // 统一字段类型与默认值（防御性规范化，避免 API/存储层返回类型不一致）
    const normalized: AssetBookmark = {
      ...asset,
      asset: assetId,
      name: String(asset.name),
      description: String(asset.description || ''),
      account: String(asset.account),
      accountRS: String(asset.accountRS),
      quantityQNT: String(asset.quantityQNT),
      decimals: parseInt(String(asset.decimals), 10) || 0,
      groupName: String(asset.groupName || ''),
    }
    bookmarks.value.push(normalized)
  }

  /**
   * 从 NrcsAsset（API 响应）构造 AssetBookmark。
   */
  function fromApiResponse(asset: NrcsAsset, groupName = ''): AssetBookmark {
    return {
      asset: String(asset.asset),
      name: String(asset.name),
      description: String(asset.description || ''),
      account: String(asset.issuer),
      accountRS: String(asset.issuerRS),
      quantityQNT: String(asset.quantityQNT),
      decimals: parseInt(String(asset.decimals), 10) || 0,
      groupName,
    }
  }

  /**
   * 从 IndexedDB 加载全部收藏到内存。
   * 对标 NRS.pages.asset_exchange（:79-129）中 storageSelect('assets', null, ...) 分支。
   */
  async function loadBookmarks(): Promise<AssetBookmark[]> {
    isLoading.value = true
    error.value = ''
    try {
      const items = await storageSelect<AssetBookmark>(BOOKMARKS_TABLE)
      // 重置内存缓存
      bookmarks.value = []
      cachedAssetIds.value = new Set()
      for (const item of items) {
        cacheAsset(item)
      }
      return bookmarks.value.slice()
    } catch (e: any) {
      error.value = e?.message || 'Failed to load asset bookmarks'
      return []
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 保存资产到收藏（批量，去重）。
   * 对标 NRS.saveAssetBookmarks（:247-301）。
   *
   * @param newAssets 待添加的资产列表
   * @returns 实际新增的资产列表（已去重）
   */
  async function saveBookmarks(
    newAssets: AssetBookmark[],
  ): Promise<AssetBookmark[]> {
    // 去重：过滤掉已存在的 asset ID
    const filtered = newAssets.filter(
      (a) => !cachedAssetIds.value.has(a.asset),
    )
    if (filtered.length === 0) return []

    try {
      await storageInsert<AssetBookmark>(BOOKMARKS_TABLE, 'asset', filtered)
      // 同步内存缓存
      for (const a of filtered) {
        cacheAsset(a)
      }
      return filtered
    } catch (e: any) {
      // 主键冲突（已存在）视为非致命：仅更新内存缓存
      if (String(e?.message || '').includes('already exists')) {
        for (const a of filtered) {
          if (!cachedAssetIds.value.has(a.asset)) cacheAsset(a)
        }
        return []
      }
      throw e
    }
  }

  /**
   * 按资产 ID 添加单个收藏。
   * 对标 NRS.forms.addAssetBookmark（:153-209）中 data.id 为纯数字的分支。
   *
   * @param assetId 资产 ID
   * @returns 新增的资产（若已存在则返回 undefined）
   */
  async function addBookmarkById(
    assetId: string,
  ): Promise<AssetBookmark | undefined> {
    const trimmed = assetId.trim()
    if (!trimmed) throw new Error('Asset ID is required')

    const result = await nrcsApi.getAsset(trimmed)
    if ((result as any).errorCode) {
      throw new Error((result as any).errorDescription || 'Asset not found')
    }
    const bookmark = fromApiResponse(result as NrcsAsset)
    const added = await saveBookmarks([bookmark])
    return added[0]
  }

  /**
   * 按发行方 RS 地址添加收藏（批量添加该发行方的所有资产）。
   * 对标 NRS.forms.addAssetBookmark（:168-180）中 /^NRCS-/i 分支。
   *
   * @param accountRS 发行方 RS 地址
   * @returns 新增的资产列表
   */
  async function addBookmarksByIssuer(
    accountRS: string,
  ): Promise<AssetBookmark[]> {
    const trimmed = accountRS.trim()
    if (!trimmed) throw new Error('Account RS is required')

    const result = (await nrcsApi.getAssetsByIssuer(trimmed)) as any
    if (result.errorCode) {
      throw new Error(result.errorDescription || 'Account not found')
    }
    const assets: NrcsAsset[] = result.assets?.[0] || []
    if (assets.length === 0) {
      throw new Error('Account has no assets')
    }
    const bookmarksToAdd = assets.map((a) => fromApiResponse(a))
    return saveBookmarks(bookmarksToAdd)
  }

  /**
   * 智能添加收藏：自动判断输入是资产 ID 还是发行方 RS。
   * 对标 NRS.forms.addAssetBookmark（:153-209）的完整流程。
   *
   * @param input 资产 ID 或发行方 RS 地址
   * @returns 新增的资产列表
   */
  async function addBookmark(
    input: string,
  ): Promise<AssetBookmark[]> {
    const trimmed = input.trim()
    if (!trimmed) throw new Error('Input is required')

    // RS 地址 → 按发行方批量添加
    if (/^NRCS-/i.test(trimmed)) {
      return addBookmarksByIssuer(trimmed)
    }
    // 纯数字 → 按资产 ID 添加
    if (/^\d+$/.test(trimmed)) {
      const added = await addBookmarkById(trimmed)
      return added ? [added] : []
    }
    throw new Error('Invalid asset ID or account RS')
  }

  /**
   * 从收藏中移除资产。
   * 对标 NRS remove_from_bookmarks（:1267-1295）。
   *
   * 注意：参考实现对"账户仍持有"的资产禁止移除。此处提供 `force` 选项，
   * 由调用方决定是否强制移除。
   *
   * @param assetId 资产 ID
   * @param force 是否强制移除（默认 false）
   */
  async function removeBookmark(assetId: string, force = false): Promise<void> {
    if (!force) {
      // 检查是否仍持有该资产（调用方应在调用前检查）
      const item = bookmarks.value.find((b) => b.asset === assetId)
      if (!item) return
    }

    const query: StorageQuery = [{ asset: assetId }]
    await storageDelete<AssetBookmark>(BOOKMARKS_TABLE, query)

    // 同步内存缓存
    bookmarks.value = bookmarks.value.filter((b) => b.asset !== assetId)
    cachedAssetIds.value.delete(assetId)
  }

  /**
   * 更新资产的分组。
   * 对标 NRS add_to_group（:1218-1253）+ remove_from_group（:1254-1266）。
   *
   * @param assetId 资产 ID
   * @param groupName 分组名称（空字符串表示移出分组）
   */
  async function updateAssetGroup(
    assetId: string,
    groupName: string,
  ): Promise<void> {
    const query: StorageQuery = [{ asset: assetId }]
    await storageUpdate<AssetBookmark>(
      BOOKMARKS_TABLE,
      { groupName },
      query,
    )
    // 同步内存缓存
    const item = bookmarks.value.find((b) => b.asset === assetId)
    if (item) {
      item.groupName = groupName
    }
  }

  /**
   * 重命名分组（更新所有匹配旧名称的资产）。
   * 对标 NRS.forms.assetExchangeChangeGroupName（:1186-1211）。
   *
   * @param oldGroupName 旧分组名称
   * @param newGroupName 新分组名称
   */
  async function renameGroup(
    oldGroupName: string,
    newGroupName: string,
  ): Promise<void> {
    // 校验新名称（对标 :1189 `/^[a-z0-9 ]+$/i`）
    if (!newGroupName.match(/^[a-z0-9 ]+$/i)) {
      throw new Error('Group name can only contain letters, numbers and spaces')
    }

    const query: StorageQuery = [{ groupName: oldGroupName }]
    await storageUpdate<AssetBookmark>(
      BOOKMARKS_TABLE,
      { groupName: newGroupName },
      query,
    )
    // 同步内存缓存
    for (const b of bookmarks.value) {
      if (b.groupName === oldGroupName) {
        b.groupName = newGroupName
      }
    }
  }

  /**
   * 删除分组（将所有匹配资产的 groupName 置空）。
   *
   * 与 `renameGroup` 不同，本操作语义为"移出分组"而非"重命名"，
   * 因此跳过 `renameGroup` 的新名称合法性校验（允许置空）。
   *
   * @param groupName 待删除的分组名称
   */
  async function deleteGroup(groupName: string): Promise<void> {
    const query: StorageQuery = [{ groupName }]
    await storageUpdate<AssetBookmark>(
      BOOKMARKS_TABLE,
      { groupName: '' },
      query,
    )
    // 同步内存缓存
    for (const b of bookmarks.value) {
      if (b.groupName === groupName) {
        b.groupName = ''
      }
    }
  }

  /**
   * 按关键词搜索收藏。
   * 对标 NRS assetExchangeSearch（:872-904）。
   *
   * @param query 搜索关键词（匹配名称/ID/发行方 RS）
   * @returns 匹配的收藏列表
   */
  function searchBookmarks(query: string): AssetBookmark[] {
    const q = query.trim().toLowerCase()
    if (!q) return bookmarks.value.slice()
    return bookmarks.value.filter(
      (b) =>
        b.name.toLowerCase().includes(q) ||
        b.asset.toLowerCase().includes(q) ||
        b.accountRS.toLowerCase().includes(q) ||
        b.account.toLowerCase().includes(q),
    )
  }

  /**
   * 按分组列举收藏。
   *
   * @param groupName 分组名称（空字符串表示未分组）
   * @returns 匹配的收藏列表
   */
  function bookmarksByGroup(groupName: string): AssetBookmark[] {
    return bookmarks.value.filter((b) => b.groupName === groupName)
  }

  /** 列举未分组的收藏 */
  function ungroupedBookmarks(): AssetBookmark[] {
    return bookmarks.value.filter((b) => !b.groupName)
  }

  /**
   * 重置内存状态（对标 NRS.resetAssetExchangeState）。
   * 不会清空 IndexedDB，仅清空内存缓存。
   */
  function resetState(): void {
    bookmarks.value = []
    cachedAssetIds.value = new Set()
    error.value = ''
    isLoading.value = false
  }

  return {
    // state
    bookmarks,
    isLoading,
    error,
    groupNames,
    // cache management
    cacheAsset,
    fromApiResponse,
    resetState,
    // persistence
    loadBookmarks,
    saveBookmarks,
    addBookmark,
    addBookmarkById,
    addBookmarksByIssuer,
    removeBookmark,
    // group management
    updateAssetGroup,
    renameGroup,
    deleteGroup,
    // queries
    searchBookmarks,
    bookmarksByGroup,
    ungroupedBookmarks,
  }
}
