/******************************************************************************
 * 资产收藏 Composable 单元测试
 *
 * 验证 useAssetBookmarks.ts 的核心功能：
 *   - loadBookmarks：从 IndexedDB 加载收藏
 *   - saveBookmarks：保存收藏（含去重）
 *   - addBookmarkById / addBookmarksByIssuer / addBookmark：按 ID/发行方/智能判断添加
 *   - removeBookmark：移除收藏
 *   - updateAssetGroup / renameGroup / deleteGroup：分组管理
 *   - searchBookmarks / bookmarksByGroup / ungroupedBookmarks：查询
 *   - cacheAsset：内存缓存去重
 *   - groupNames：计算属性
 *
 * Mock 依赖：
 *   - @/api/modules：nrcsApi.getAsset/getAssetsByIssuer
 *   - @/utils/nrcs-storage：storageSelect/storageInsert/storageUpdate/storageDelete
 *
 * 对标参考：nrs.assetexchange.js 的 saveAssetBookmarks/forms.addAssetBookmark 等
 ******************************************************************************/
import { describe, it, expect, vi, beforeEach } from 'vitest'

// Mock nrcsApi
vi.mock('@/api/modules', () => ({
  nrcsApi: {
    getAsset: vi.fn(),
    getAssetsByIssuer: vi.fn(),
  },
}))

// Mock nrcs-storage
vi.mock('@/utils/nrcs-storage', () => ({
  storageSelect: vi.fn(),
  storageInsert: vi.fn(),
  storageUpdate: vi.fn(),
  storageDelete: vi.fn(),
}))

import { useAssetBookmarks, type AssetBookmark } from '@/composables/useAssetBookmarks'
import { nrcsApi } from '@/api/modules'
import {
  storageSelect,
  storageInsert,
  storageUpdate,
  storageDelete,
} from '@/utils/nrcs-storage'

/** 构造测试用 AssetBookmark */
function makeBookmark(overrides: Partial<AssetBookmark> = {}): AssetBookmark {
  return {
    asset: '123456789',
    name: 'TestAsset',
    description: 'test description',
    account: '999',
    accountRS: 'NRCS-TEST',
    quantityQNT: '1000000',
    decimals: 2,
    groupName: '',
    ...overrides,
  }
}

/** 构造测试用 NrcsAsset（API 响应） */
function makeAssetResponse(overrides: Record<string, any> = {}) {
  return {
    asset: '123456789',
    name: 'TestAsset',
    description: 'test description',
    quantityQNT: '1000000',
    decimals: 2,
    issuer: '999',
    issuerRS: 'NRCS-TEST',
    ...overrides,
  }
}

describe('useAssetBookmarks', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  // ─────────────────────────────────────────────────────────────────────────
  // loadBookmarks
  // ─────────────────────────────────────────────────────────────────────────
  describe('loadBookmarks', () => {
    it('应从 IndexedDB 加载收藏到内存缓存', async () => {
      const items = [makeBookmark({ asset: '1', name: 'Asset1' }), makeBookmark({ asset: '2', name: 'Asset2' })]
      vi.mocked(storageSelect).mockResolvedValue(items)

      const { bookmarks, loadBookmarks } = useAssetBookmarks()
      const result = await loadBookmarks()

      expect(storageSelect).toHaveBeenCalledWith('assets')
      expect(result).toHaveLength(2)
      expect(bookmarks.value).toHaveLength(2)
      expect(bookmarks.value[0].name).toBe('Asset1')
    })

    it('IndexedDB 返回空数组时应清空内存缓存', async () => {
      vi.mocked(storageSelect).mockResolvedValue([])

      const { bookmarks, loadBookmarks } = useAssetBookmarks()
      const result = await loadBookmarks()

      expect(result).toHaveLength(0)
      expect(bookmarks.value).toHaveLength(0)
    })

    it('加载失败时应设置 error 并返回空数组', async () => {
      vi.mocked(storageSelect).mockRejectedValue(new Error('DB error'))

      const { error, loadBookmarks } = useAssetBookmarks()
      const result = await loadBookmarks()

      expect(result).toHaveLength(0)
      expect(error.value).toBe('DB error')
    })
  })

  // ─────────────────────────────────────────────────────────────────────────
  // saveBookmarks（去重）
  // ─────────────────────────────────────────────────────────────────────────
  describe('saveBookmarks', () => {
    it('应保存新资产并同步内存缓存', async () => {
      vi.mocked(storageInsert).mockResolvedValue([])
      const { bookmarks, saveBookmarks } = useAssetBookmarks()
      const newAsset = makeBookmark({ asset: '100', name: 'NewAsset' })

      await saveBookmarks([newAsset])

      expect(storageInsert).toHaveBeenCalledWith('assets', 'asset', [newAsset])
      expect(bookmarks.value).toHaveLength(1)
      expect(bookmarks.value[0].asset).toBe('100')
    })

    it('应过滤已缓存的重复资产', async () => {
      vi.mocked(storageInsert).mockResolvedValue([])
      const { bookmarks, saveBookmarks, cacheAsset } = useAssetBookmarks()
      const existing = makeBookmark({ asset: '1', name: 'Existing' })
      cacheAsset(existing)

      const newAsset = makeBookmark({ asset: '2', name: 'New' })
      await saveBookmarks([existing, newAsset])

      // 只应插入新资产
      expect(storageInsert).toHaveBeenCalledWith('assets', 'asset', [newAsset])
      expect(bookmarks.value).toHaveLength(2)
    })

    it('全部重复时应返回空数组且不调 storageInsert', async () => {
      const { saveBookmarks, cacheAsset } = useAssetBookmarks()
      const existing = makeBookmark({ asset: '1' })
      cacheAsset(existing)

      const result = await saveBookmarks([existing])

      expect(result).toHaveLength(0)
      expect(storageInsert).not.toHaveBeenCalled()
    })
  })

  // ─────────────────────────────────────────────────────────────────────────
  // addBookmarkById
  // ─────────────────────────────────────────────────────────────────────────
  describe('addBookmarkById', () => {
    it('应通过资产 ID 查询并加入收藏', async () => {
      vi.mocked(nrcsApi.getAsset).mockResolvedValue(makeAssetResponse() as any)
      vi.mocked(storageInsert).mockResolvedValue([])

      const { addBookmarkById } = useAssetBookmarks()
      const result = await addBookmarkById('123456789')

      expect(nrcsApi.getAsset).toHaveBeenCalledWith('123456789')
      expect(result).toBeDefined()
      expect(result?.asset).toBe('123456789')
    })

    it('API 返回 errorCode 时应抛出异常', async () => {
      vi.mocked(nrcsApi.getAsset).mockResolvedValue({
        errorCode: 5,
        errorDescription: 'Unknown asset',
      } as any)

      const { addBookmarkById } = useAssetBookmarks()
      await expect(addBookmarkById('999')).rejects.toThrow('Unknown asset')
    })

    it('空 ID 时应抛出异常', async () => {
      const { addBookmarkById } = useAssetBookmarks()
      await expect(addBookmarkById('')).rejects.toThrow('Asset ID is required')
    })
  })

  // ─────────────────────────────────────────────────────────────────────────
  // addBookmarksByIssuer
  // ─────────────────────────────────────────────────────────────────────────
  describe('addBookmarksByIssuer', () => {
    it('应通过发行方 RS 批量查询并加入收藏', async () => {
      const assets = [makeAssetResponse({ asset: '1' }), makeAssetResponse({ asset: '2' })]
      vi.mocked(nrcsApi.getAssetsByIssuer).mockResolvedValue({ assets: [assets] } as any)
      vi.mocked(storageInsert).mockResolvedValue([])

      const { addBookmarksByIssuer } = useAssetBookmarks()
      const result = await addBookmarksByIssuer('NRCS-TEST')

      expect(nrcsApi.getAssetsByIssuer).toHaveBeenCalledWith('NRCS-TEST')
      expect(result).toHaveLength(2)
      expect(result[0].asset).toBe('1')
    })

    it('发行方无资产时应抛出异常', async () => {
      vi.mocked(nrcsApi.getAssetsByIssuer).mockResolvedValue({ assets: [[]] } as any)

      const { addBookmarksByIssuer } = useAssetBookmarks()
      await expect(addBookmarksByIssuer('NRCS-EMPTY')).rejects.toThrow('Account has no assets')
    })

    it('API 返回 errorCode 时应抛出异常', async () => {
      vi.mocked(nrcsApi.getAssetsByIssuer).mockResolvedValue({
        errorCode: 5,
        errorDescription: 'Unknown account',
      } as any)

      const { addBookmarksByIssuer } = useAssetBookmarks()
      await expect(addBookmarksByIssuer('NRCS-UNKNOWN')).rejects.toThrow('Unknown account')
    })
  })

  // ─────────────────────────────────────────────────────────────────────────
  // addBookmark（智能判断）
  // ─────────────────────────────────────────────────────────────────────────
  describe('addBookmark', () => {
    it('纯数字输入应按资产 ID 添加', async () => {
      vi.mocked(nrcsApi.getAsset).mockResolvedValue(makeAssetResponse() as any)
      vi.mocked(storageInsert).mockResolvedValue([])

      const { addBookmark } = useAssetBookmarks()
      const result = await addBookmark('123456789')

      expect(nrcsApi.getAsset).toHaveBeenCalledWith('123456789')
      expect(result).toHaveLength(1)
    })

    it('NRCS- 开头应按发行方批量添加', async () => {
      const assets = [makeAssetResponse({ asset: '1' })]
      vi.mocked(nrcsApi.getAssetsByIssuer).mockResolvedValue({ assets: [assets] } as any)
      vi.mocked(storageInsert).mockResolvedValue([])

      const { addBookmark } = useAssetBookmarks()
      const result = await addBookmark('NRCS-TEST')

      expect(nrcsApi.getAssetsByIssuer).toHaveBeenCalledWith('NRCS-TEST')
      expect(result).toHaveLength(1)
    })

    it('空输入应抛出异常', async () => {
      const { addBookmark } = useAssetBookmarks()
      await expect(addBookmark('')).rejects.toThrow('Input is required')
    })

    it('无效格式应抛出异常', async () => {
      const { addBookmark } = useAssetBookmarks()
      await expect(addBookmark('invalid!')).rejects.toThrow('Invalid asset ID or account RS')
    })
  })

  // ─────────────────────────────────────────────────────────────────────────
  // removeBookmark
  // ─────────────────────────────────────────────────────────────────────────
  describe('removeBookmark', () => {
    it('应从 IndexedDB 删除并同步内存缓存', async () => {
      vi.mocked(storageDelete).mockResolvedValue([])
      const { bookmarks, cacheAsset, removeBookmark } = useAssetBookmarks()
      cacheAsset(makeBookmark({ asset: '1' }))
      expect(bookmarks.value).toHaveLength(1)

      await removeBookmark('1', true)

      expect(storageDelete).toHaveBeenCalledWith('assets', [{ asset: '1' }])
      expect(bookmarks.value).toHaveLength(0)
    })

    it('不存在的资产应静默返回', async () => {
      vi.mocked(storageDelete).mockResolvedValue([])
      const { removeBookmark } = useAssetBookmarks()

      await removeBookmark('999', true)
      // 不抛异常即通过
      expect(storageDelete).toHaveBeenCalledWith('assets', [{ asset: '999' }])
    })
  })

  // ─────────────────────────────────────────────────────────────────────────
  // updateAssetGroup
  // ─────────────────────────────────────────────────────────────────────────
  describe('updateAssetGroup', () => {
    it('应更新资产分组并同步内存缓存', async () => {
      vi.mocked(storageUpdate).mockResolvedValue([])
      const { bookmarks, cacheAsset, updateAssetGroup } = useAssetBookmarks()
      const asset = makeBookmark({ asset: '1', groupName: '' })
      cacheAsset(asset)

      await updateAssetGroup('1', 'MyGroup')

      expect(storageUpdate).toHaveBeenCalledWith('assets', { groupName: 'MyGroup' }, [{ asset: '1' }])
      expect(bookmarks.value[0].groupName).toBe('MyGroup')
    })

    it('传入空字符串应移出分组', async () => {
      vi.mocked(storageUpdate).mockResolvedValue([])
      const { bookmarks, cacheAsset, updateAssetGroup } = useAssetBookmarks()
      const asset = makeBookmark({ asset: '1', groupName: 'OldGroup' })
      cacheAsset(asset)

      await updateAssetGroup('1', '')

      expect(bookmarks.value[0].groupName).toBe('')
    })
  })

  // ─────────────────────────────────────────────────────────────────────────
  // renameGroup
  // ─────────────────────────────────────────────────────────────────────────
  describe('renameGroup', () => {
    it('应重命名分组并同步内存缓存', async () => {
      vi.mocked(storageUpdate).mockResolvedValue([])
      const { bookmarks, cacheAsset, renameGroup } = useAssetBookmarks()
      cacheAsset(makeBookmark({ asset: '1', groupName: 'Old' }))
      cacheAsset(makeBookmark({ asset: '2', groupName: 'Old' }))
      cacheAsset(makeBookmark({ asset: '3', groupName: 'Other' }))

      await renameGroup('Old', 'New')

      expect(storageUpdate).toHaveBeenCalledWith('assets', { groupName: 'New' }, [{ groupName: 'Old' }])
      expect(bookmarks.value[0].groupName).toBe('New')
      expect(bookmarks.value[1].groupName).toBe('New')
      expect(bookmarks.value[2].groupName).toBe('Other')
    })

    it('新名称含非法字符应抛出异常', async () => {
      const { renameGroup } = useAssetBookmarks()
      await expect(renameGroup('Old', 'New@Group')).rejects.toThrow(
        'Group name can only contain letters, numbers and spaces',
      )
    })
  })

  // ─────────────────────────────────────────────────────────────────────────
  // deleteGroup
  // ─────────────────────────────────────────────────────────────────────────
  describe('deleteGroup', () => {
    it('应将分组内所有资产的 groupName 置空', async () => {
      vi.mocked(storageUpdate).mockResolvedValue([])
      const { bookmarks, cacheAsset, deleteGroup } = useAssetBookmarks()
      cacheAsset(makeBookmark({ asset: '1', groupName: 'ToRemove' }))
      cacheAsset(makeBookmark({ asset: '2', groupName: 'Keep' }))

      await deleteGroup('ToRemove')

      expect(storageUpdate).toHaveBeenCalledWith('assets', { groupName: '' }, [{ groupName: 'ToRemove' }])
      expect(bookmarks.value[0].groupName).toBe('')
      expect(bookmarks.value[1].groupName).toBe('Keep')
    })
  })

  // ─────────────────────────────────────────────────────────────────────────
  // searchBookmarks
  // ─────────────────────────────────────────────────────────────────────────
  describe('searchBookmarks', () => {
    it('应按名称匹配搜索', async () => {
      const { cacheAsset, searchBookmarks } = useAssetBookmarks()
      cacheAsset(makeBookmark({ asset: '1', name: 'Bitcoin' }))
      cacheAsset(makeBookmark({ asset: '2', name: 'Ethereum' }))
      cacheAsset(makeBookmark({ asset: '3', name: 'Bitcoin2' }))

      const result = searchBookmarks('bitcoin')
      expect(result).toHaveLength(2)
    })

    it('应按 asset ID 匹配搜索', async () => {
      const { cacheAsset, searchBookmarks } = useAssetBookmarks()
      cacheAsset(makeBookmark({ asset: '123', name: 'A' }))
      cacheAsset(makeBookmark({ asset: '456', name: 'B' }))

      const result = searchBookmarks('123')
      expect(result).toHaveLength(1)
      expect(result[0].asset).toBe('123')
    })

    it('应按 accountRS 匹配搜索', async () => {
      const { cacheAsset, searchBookmarks } = useAssetBookmarks()
      cacheAsset(makeBookmark({ asset: '1', accountRS: 'NRCS-AAA' }))
      cacheAsset(makeBookmark({ asset: '2', accountRS: 'NRCS-BBB' }))

      const result = searchBookmarks('nrcs-aaa')
      expect(result).toHaveLength(1)
      expect(result[0].asset).toBe('1')
    })

    it('空查询应返回全部收藏', async () => {
      const { cacheAsset, searchBookmarks } = useAssetBookmarks()
      cacheAsset(makeBookmark({ asset: '1' }))
      cacheAsset(makeBookmark({ asset: '2' }))

      const result = searchBookmarks('')
      expect(result).toHaveLength(2)
    })
  })

  // ─────────────────────────────────────────────────────────────────────────
  // bookmarksByGroup / ungroupedBookmarks
  // ─────────────────────────────────────────────────────────────────────────
  describe('bookmarksByGroup', () => {
    it('应按分组名过滤', async () => {
      const { cacheAsset, bookmarksByGroup } = useAssetBookmarks()
      cacheAsset(makeBookmark({ asset: '1', groupName: 'GroupA' }))
      cacheAsset(makeBookmark({ asset: '2', groupName: 'GroupB' }))
      cacheAsset(makeBookmark({ asset: '3', groupName: 'GroupA' }))

      const result = bookmarksByGroup('GroupA')
      expect(result).toHaveLength(2)
    })
  })

  describe('ungroupedBookmarks', () => {
    it('应返回未分组的收藏', async () => {
      const { cacheAsset, ungroupedBookmarks } = useAssetBookmarks()
      cacheAsset(makeBookmark({ asset: '1', groupName: '' }))
      cacheAsset(makeBookmark({ asset: '2', groupName: 'GroupA' }))
      cacheAsset(makeBookmark({ asset: '3', groupName: '' }))

      const result = ungroupedBookmarks()
      expect(result).toHaveLength(2)
      expect(result.every((b) => b.groupName === '')).toBe(true)
    })
  })

  // ─────────────────────────────────────────────────────────────────────────
  // cacheAsset（去重）
  // ─────────────────────────────────────────────────────────────────────────
  describe('cacheAsset', () => {
    it('应跳过已缓存的重复资产', async () => {
      const { bookmarks, cacheAsset } = useAssetBookmarks()
      const asset = makeBookmark({ asset: '1' })

      cacheAsset(asset)
      cacheAsset(asset) // 重复

      expect(bookmarks.value).toHaveLength(1)
    })

    it('应规范化字段类型与默认值', async () => {
      const { bookmarks, cacheAsset } = useAssetBookmarks()
      cacheAsset({
        asset: 123 as any,
        name: 'Test',
        description: undefined as any,
        account: 999 as any,
        accountRS: 'NRCS-X',
        quantityQNT: 1000 as any,
        decimals: '2' as any,
        groupName: undefined as any,
      })

      const cached = bookmarks.value[0]
      expect(typeof cached.asset).toBe('string')
      expect(typeof cached.decimals).toBe('number')
      expect(cached.description).toBe('')
      expect(cached.groupName).toBe('')
    })
  })

  // ─────────────────────────────────────────────────────────────────────────
  // groupNames（计算属性）
  // ─────────────────────────────────────────────────────────────────────────
  describe('groupNames', () => {
    it('应返回按字母序排序的分组名列表（不含空分组）', async () => {
      const { cacheAsset, groupNames } = useAssetBookmarks()
      cacheAsset(makeBookmark({ asset: '1', groupName: 'zebra' }))
      cacheAsset(makeBookmark({ asset: '2', groupName: 'apple' }))
      cacheAsset(makeBookmark({ asset: '3', groupName: '' }))
      cacheAsset(makeBookmark({ asset: '4', groupName: 'apple' })) // 重复分组

      expect(groupNames.value).toEqual(['apple', 'zebra'])
    })

    it('无分组时应返回空数组', async () => {
      const { cacheAsset, groupNames } = useAssetBookmarks()
      cacheAsset(makeBookmark({ asset: '1', groupName: '' }))

      expect(groupNames.value).toEqual([])
    })
  })

  // ─────────────────────────────────────────────────────────────────────────
  // resetState
  // ─────────────────────────────────────────────────────────────────────────
  describe('resetState', () => {
    it('应清空内存缓存但不清空 IndexedDB', async () => {
      const { bookmarks, cacheAsset, resetState } = useAssetBookmarks()
      cacheAsset(makeBookmark({ asset: '1' }))
      expect(bookmarks.value).toHaveLength(1)

      resetState()

      expect(bookmarks.value).toHaveLength(0)
      // 不应调用 storageDelete
      expect(storageDelete).not.toHaveBeenCalled()
    })
  })

  // ─────────────────────────────────────────────────────────────────────────
  // fromApiResponse
  // ─────────────────────────────────────────────────────────────────────────
  describe('fromApiResponse', () => {
    it('应将 NrcsAsset API 响应转换为 AssetBookmark', async () => {
      const { fromApiResponse } = useAssetBookmarks()
      const apiResponse = makeAssetResponse({ asset: '999', name: 'MyAsset', decimals: 4 })
      const bookmark = fromApiResponse(apiResponse, 'MyGroup')

      expect(bookmark.asset).toBe('999')
      expect(bookmark.name).toBe('MyAsset')
      expect(bookmark.decimals).toBe(4)
      expect(bookmark.groupName).toBe('MyGroup')
      expect(bookmark.account).toBe('999') // issuer → account
      expect(bookmark.accountRS).toBe('NRCS-TEST') // issuerRS → accountRS
    })
  })
})
