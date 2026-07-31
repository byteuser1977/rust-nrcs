/******************************************************************************
 * NRCS 通知系统单元测试（阶段 1.3）
 *
 * 验证 useNotifications composable 的核心功能（对标 nrs.notifications.js + growl.js）：
 *   - Toast 通知：notify/notifySuccess/notifyError/notifyWarning/notifyInfo
 *   - NRCS 专用通知：notifyAssetReceived/notifyAssetSold/notifyForkWarning 等
 *   - 交易类型计数：updateNotifications/resetNotificationState/initNotificationCounts
 *   - 特殊计数：setUnconfirmedNotifications/setPhasingNotifications/setShufflingNotifications
 *
 * Mock 依赖：
 *   - @/stores/modules/ui.store：addNotification/removeNotification/clearAllNotifications
 *   - @/utils/nrcs-storage：storageSelect/storageInsert/storageUpdate
 *   - @/api/modules：nrcsApi.getTime/getBlockchainTransactions/getAccountPhasedTransactionCount/getAllShufflings
 ******************************************************************************/
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'

// ============================================================================
// 共享 Mock 实例（确保 useUiStore() 每次返回同一对象）
// ============================================================================

const mockAddNotification = vi.fn((n: { type: string; title: string; message: string; duration?: number }) => {
  return Date.now() + Math.random()
})
const mockRemoveNotification = vi.fn()
const mockClearAllNotifications = vi.fn()

vi.mock('@/stores/modules/ui.store', () => ({
  useUiStore: () => ({
    addNotification: mockAddNotification,
    removeNotification: mockRemoveNotification,
    clearAllNotifications: mockClearAllNotifications,
    notifications: [],
    notificationCount: 0,
  }),
}))

vi.mock('@/utils/nrcs-storage', () => ({
  storageSelect: vi.fn(),
  storageInsert: vi.fn(),
  storageUpdate: vi.fn(),
}))

vi.mock('@/api/modules', () => ({
  nrcsApi: {
    getTime: vi.fn(),
    getBlockchainTransactions: vi.fn(),
    getAccountPhasedTransactionCount: vi.fn(),
    getAllShufflings: vi.fn(),
  },
}))

import { useNotifications } from '@/composables/useNotifications'
import { storageSelect, storageInsert, storageUpdate } from '@/utils/nrcs-storage'
import { nrcsApi } from '@/api/modules'

// ============================================================================
// 测试用例
// ============================================================================

describe('useNotifications: Toast 通知', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('notify 默认为 info 类型', () => {
    const { notify } = useNotifications()
    notify('测试消息')

    expect(mockAddNotification).toHaveBeenCalledWith({
      type: 'info',
      title: '',
      message: '测试消息',
      duration: 5000,
    })
  })

  it('notify 支持自定义 type/title/duration', () => {
    const { notify } = useNotifications()
    notify('错误消息', { type: 'error', title: '错误', duration: 0 })

    expect(mockAddNotification).toHaveBeenCalledWith({
      type: 'error',
      title: '错误',
      message: '错误消息',
      duration: 0,
    })
  })

  it('notifySuccess 调用 addNotification with type=success', () => {
    const { notifySuccess } = useNotifications()
    notifySuccess('成功')

    expect(mockAddNotification).toHaveBeenCalledWith(
      expect.objectContaining({ type: 'success', message: '成功' }),
    )
  })

  it('notifyError 调用 addNotification with type=error', () => {
    const { notifyError } = useNotifications()
    notifyError('失败')

    expect(mockAddNotification).toHaveBeenCalledWith(
      expect.objectContaining({ type: 'error', message: '失败' }),
    )
  })

  it('notifyWarning 调用 addNotification with type=warning', () => {
    const { notifyWarning } = useNotifications()
    notifyWarning('警告')

    expect(mockAddNotification).toHaveBeenCalledWith(
      expect.objectContaining({ type: 'warning', message: '警告' }),
    )
  })

  it('notifyInfo 调用 addNotification with type=info', () => {
    const { notifyInfo } = useNotifications()
    notifyInfo('信息')

    expect(mockAddNotification).toHaveBeenCalledWith(
      expect.objectContaining({ type: 'info', message: '信息' }),
    )
  })

  it('dismissNotification 调用 removeNotification', () => {
    const { dismissNotification } = useNotifications()
    dismissNotification(123)

    expect(mockRemoveNotification).toHaveBeenCalledWith(123)
  })

  it('clearAllToasts 调用 clearAllNotifications', () => {
    const { clearAllToasts } = useNotifications()
    clearAllToasts()

    expect(mockClearAllNotifications).toHaveBeenCalled()
  })
})

describe('useNotifications: NRCS 专用通知', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('notifyAssetReceived count=1 → 单数消息（对标 nrs.js:1602）', () => {
    const { notifyAssetReceived } = useNotifications()
    notifyAssetReceived('TESTAsset', '1')

    expect(mockAddNotification).toHaveBeenCalledWith(
      expect.objectContaining({
        type: 'success',
        message: expect.stringContaining('TESTAsset'),
      }),
    )
    const call = mockAddNotification.mock.calls[0][0]
    expect(call.message).toContain('1')
  })

  it('notifyAssetReceived count>1 → 复数消息（对标 nrs.js:1608）', () => {
    const { notifyAssetReceived } = useNotifications()
    notifyAssetReceived('TESTAsset', '5')

    expect(mockAddNotification).toHaveBeenCalledWith(
      expect.objectContaining({
        type: 'success',
        message: expect.stringContaining('5'),
      }),
    )
    const call = mockAddNotification.mock.calls[0][0]
    expect(call.message).toContain('TESTAsset')
  })

  it('notifyAssetSold count=1 → 单数消息（对标 nrs.js:1622）', () => {
    const { notifyAssetSold } = useNotifications()
    notifyAssetSold('TESTAsset', '1')

    expect(mockAddNotification).toHaveBeenCalledWith(
      expect.objectContaining({ type: 'success' }),
    )
  })

  it('notifyMultipleAssetDifferences → success 通知（对标 nrs.js:1641）', () => {
    const { notifyMultipleAssetDifferences } = useNotifications()
    notifyMultipleAssetDifferences()

    expect(mockAddNotification).toHaveBeenCalledWith(
      expect.objectContaining({ type: 'success' }),
    )
  })

  it('notifyForkWarning → error 通知 + duration=0 不自动关闭（对标 nrs.js:1713）', () => {
    const { notifyForkWarning } = useNotifications()
    notifyForkWarning()

    expect(mockAddNotification).toHaveBeenCalledWith(
      expect.objectContaining({ type: 'error', duration: 0 }),
    )
  })

  it('notifyForkWarningBaseTarget → error 通知（对标 nrs.js:1719）', () => {
    const { notifyForkWarningBaseTarget } = useNotifications()
    notifyForkWarningBaseTarget()

    expect(mockAddNotification).toHaveBeenCalledWith(
      expect.objectContaining({ type: 'error', duration: 0 }),
    )
  })

  it('notifyConnectionError 包含 URL 和描述（对标 nrs.js:493）', () => {
    const { notifyConnectionError } = useNotifications()
    notifyConnectionError('http://localhost:7876', 'Connection refused')

    expect(mockAddNotification).toHaveBeenCalledWith(
      expect.objectContaining({
        type: 'error',
        message: expect.stringContaining('http://localhost:7876'),
      }),
    )
    const call = mockAddNotification.mock.calls[0][0]
    expect(call.message).toContain('Connection refused')
  })

  it('notifyClipboardCopy → info 通知（对标 nrs.js:377）', () => {
    const { notifyClipboardCopy } = useNotifications()
    notifyClipboardCopy()

    expect(mockAddNotification).toHaveBeenCalledWith(
      expect.objectContaining({ type: 'info' }),
    )
  })
})

describe('useNotifications: 交易类型计数', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('updateNotifications account 为空时跳过', async () => {
    const { updateNotifications } = useNotifications()
    await updateNotifications('')

    expect(nrcsApi.getTime).not.toHaveBeenCalled()
  })

  it('updateNotifications 成功加载时间戳并初始化计数（对标 nrs.notifications.js:196-215）', async () => {
    vi.mocked(nrcsApi.getTime).mockResolvedValue({ time: 100000 })
    vi.mocked(storageSelect).mockResolvedValue([
      { id: 'notification_timestamps', contents: '{"ts_0_0":50000}' },
    ])
    vi.mocked(nrcsApi.getBlockchainTransactions).mockResolvedValue({ transactions: [] })
    vi.mocked(storageUpdate).mockResolvedValue([] as any)
    vi.mocked(storageInsert).mockResolvedValue([] as any)

    const { updateNotifications } = useNotifications()
    await updateNotifications('1234567890')

    expect(nrcsApi.getTime).toHaveBeenCalled()
    expect(storageSelect).toHaveBeenCalledWith('data', [{ id: 'notification_timestamps' }])
    expect(nrcsApi.getBlockchainTransactions).toHaveBeenCalled()
  })

  it('updateNotifications IndexedDB 无记录时用空时间戳初始化', async () => {
    vi.mocked(nrcsApi.getTime).mockResolvedValue({ time: 100000 })
    vi.mocked(storageSelect).mockResolvedValue([])
    vi.mocked(nrcsApi.getBlockchainTransactions).mockResolvedValue({ transactions: [] })
    vi.mocked(storageInsert).mockResolvedValue([] as any)

    const { updateNotifications } = useNotifications()
    await updateNotifications('1234567890')

    // 首次存储时调用 storageInsert
    expect(storageInsert).toHaveBeenCalledWith(
      'data',
      'id',
      expect.objectContaining({ id: 'notification_timestamps' }),
    )
  })

  it('updateNotifications getTime 失败时静默处理', async () => {
    vi.mocked(nrcsApi.getTime).mockRejectedValue(new Error('network'))

    const { updateNotifications } = useNotifications()
    await updateNotifications('1234567890')

    // 不抛出异常，仅记录警告
    expect(storageSelect).not.toHaveBeenCalled()
  })

  it('resetNotificationState 清零计数并更新时间戳（对标 nrs.notifications.js:124-145）', async () => {
    // 先通过 updateNotifications 初始化（使 saveNotificationTimestamps 走 storageUpdate 路径）
    vi.mocked(nrcsApi.getTime).mockResolvedValue({ time: 100000 })
    vi.mocked(storageSelect).mockResolvedValue([
      { id: 'notification_timestamps', contents: '{}' },
    ])
    vi.mocked(nrcsApi.getBlockchainTransactions).mockResolvedValue({ transactions: [] })
    vi.mocked(storageUpdate).mockResolvedValue([] as any)
    vi.mocked(storageInsert).mockResolvedValue([] as any)

    const notif = useNotifications()
    await notif.updateNotifications('1234567890')

    // 现在 reset
    vi.mocked(nrcsApi.getTime).mockResolvedValue({ time: 200000 })
    await notif.resetNotificationState()

    expect(nrcsApi.getTime).toHaveBeenCalled()
    // saveNotificationTimestamps 会先 storageSelect，已有记录 → storageUpdate
    expect(storageUpdate).toHaveBeenCalled()
  })

  it('resetNotificationState 指定 page 时只重置该 page 的子类型', async () => {
    vi.mocked(nrcsApi.getTime).mockResolvedValue({ time: 200000 })
    vi.mocked(storageSelect).mockResolvedValue([
      { id: 'notification_timestamps', contents: '{}' },
    ])
    vi.mocked(storageUpdate).mockResolvedValue([] as any)
    vi.mocked(storageInsert).mockResolvedValue([] as any)

    const { resetNotificationState } = useNotifications()
    await resetNotificationState('transactions')

    expect(nrcsApi.getTime).toHaveBeenCalled()
  })
})

describe('useNotifications: 特殊计数', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('setUnconfirmedNotifications count <= itemsPerPage 时直接设置（对标 nrs.notifications.js:217-226）', () => {
    const { setUnconfirmedNotifications, unconfirmedCount } = useNotifications()
    setUnconfirmedNotifications(5, 15)

    expect(unconfirmedCount.value).toBe(5)
  })

  it('setUnconfirmedNotifications count > itemsPerPage 时截断（对标 nrs.notifications.js:219-223）', () => {
    const { setUnconfirmedNotifications, unconfirmedCount } = useNotifications()
    setUnconfirmedNotifications(20, 15)

    // UI 层应显示 "15+"
    expect(unconfirmedCount.value).toBe(15)
  })

  it('setPhasingNotifications 成功时更新 phasingCount（对标 nrs.notifications.js:228-239）', async () => {
    vi.mocked(nrcsApi.getAccountPhasedTransactionCount).mockResolvedValue({
      numberOfPhasedTransactions: 3,
    })

    const { setPhasingNotifications, phasingCount } = useNotifications()
    await setPhasingNotifications('1234567890')

    expect(nrcsApi.getAccountPhasedTransactionCount).toHaveBeenCalledWith('1234567890')
    expect(phasingCount.value).toBe(3)
  })

  it('setPhasingNotifications 失败时保持原值不变（对标 NRS 行为：失败不更新计数）', async () => {
    vi.mocked(nrcsApi.getAccountPhasedTransactionCount).mockRejectedValue(new Error('network'))

    const { setPhasingNotifications, phasingCount } = useNotifications()
    // 初始值为 0（模块级 state 可能为前一个测试的值，但失败时不更新）
    const before = phasingCount.value
    await setPhasingNotifications('1234567890')

    expect(phasingCount.value).toBe(before)
  })

  it('setShufflingNotifications 成功时更新 shufflingCount（对标 nrs.notifications.js:241-252）', async () => {
    vi.mocked(nrcsApi.getAllShufflings).mockResolvedValue({
      shufflings: [{}, {}, {}],
    } as any)

    const { setShufflingNotifications, shufflingCount } = useNotifications()
    await setShufflingNotifications()

    expect(nrcsApi.getAllShufflings).toHaveBeenCalled()
    expect(shufflingCount.value).toBe(3)
  })

  it('setShufflingNotifications 无 shufflings 时设为 0', async () => {
    vi.mocked(nrcsApi.getAllShufflings).mockResolvedValue({} as any)

    const { setShufflingNotifications, shufflingCount } = useNotifications()
    await setShufflingNotifications()

    expect(shufflingCount.value).toBe(0)
  })
})

describe('useNotifications: 计算属性', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('totalNotificationCount 初始为 0', () => {
    const { totalNotificationCount } = useNotifications()
    expect(totalNotificationCount.value).toBe(0)
  })

  it('subTypeNotificationCount 初始为 0', () => {
    const { subTypeNotificationCount } = useNotifications()
    expect(subTypeNotificationCount.value).toBe(0)
  })

  it('notifySubTypes 初始为空数组', () => {
    const { notifySubTypes } = useNotifications()
    expect(notifySubTypes.value).toEqual([])
  })
})
