import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Transaction, SendTxParams, PageResult } from '@/types/business'
import { transactionApi } from '@/api/modules'
import type { ApiResponse } from '@/types'
import { ElMessage } from 'element-plus'
import { useAccountStore } from './account.store'
import { formatTime, formatAddress, formatWeiToEth } from '@/utils/format'

export const useTransactionStore = defineStore('transaction', () => {
  // 交易记录列表
  const transactions = ref<Transaction[]>([])
  const pendingTransactions = ref<Transaction[]>([])

  // 分页状态
  const total = ref(0)
  const currentPage = ref(1)
  const pageSize = ref(20)
  const isLoading = ref(false)

  // 过滤器
  const statusFilter = ref<string>('')
  const addressFilter = ref<string>('')

  // 计算属性
  const hasNextPage = computed(() => {
    return currentPage.value * pageSize.value < total.value
  })

  const confirmedTransactions = computed(() =>
    transactions.value.filter(tx => tx.status === 'success' || tx.status === 'failed')
  )

  const successfulTransactions = computed(() =>
    transactions.value.filter(tx => tx.status === 'success')
  )

  const failedTransactions = computed(() =>
    transactions.value.filter(tx => tx.status === 'failed')
  )

  /**
   * 获取交易列表（分页）
   */
  async function fetchTransactions(page: number = 1, size: number = 20): Promise<void> {
    try {
      isLoading.value = true
      currentPage.value = page
      pageSize.value = size

      const response: ApiResponse<PageResult<Transaction>> = await transactionApi.getTransactions({
        page,
        size,
        status: statusFilter.value || undefined,
        address: addressFilter.value || undefined
      })

      const { list, total: totalCount } = response.data

      transactions.value = list
      total.value = totalCount
    } catch (error: any) {
      console.error('Failed to fetch transactions', error)
      ElMessage.error('获取交易列表失败')
      throw error
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 加载下一页
   */
  async function loadNextPage(): Promise<void> {
    if (!hasNextPage.value || isLoading.value) return
    await fetchTransactions(currentPage.value + 1, pageSize.value)
  }

  /**
   * 刷新当前页
   */
  async function refresh(): Promise<void> {
    await fetchTransactions(currentPage.value, pageSize.value)
  }

  /**
   * 获取交易详情
   */
  async function getTransaction(hash: string): Promise<Transaction | null> {
    try {
      const response: ApiResponse<Transaction> = await transactionApi.getTransaction(hash)
      return response.data
    } catch (error: any) {
      console.error('Failed to fetch transaction', error)
      ElMessage.error('获取交易详情失败')
      return null
    }
  }

  /**
   * 发送交易
   */
  async function sendTransaction(data: SendTxParams): Promise<string | null> {
    try {
      const response: ApiResponse<{ tx_hash: string }> = await transactionApi.sendTransaction(data)
      const txHash = response.data.tx_hash

      // 添加到待确认列表
      const pendingTx: Transaction = {
        hash: txHash,
        block_number: 0,
        from: data.from,
        to: data.to,
        value: data.value,
        gas_used: 0,
        gas_price: data.gasPrice || '0',
        status: 'pending',
        input: data.data || '0x',
        created_at: new Date().toISOString()
      }
      pendingTransactions.value.unshift(pendingTx)

      ElMessage.success('交易发送成功')
      return txHash
    } catch (error: any) {
      console.error('Failed to send transaction', error)
      ElMessage.error(error.message || '交易发送失败')
      return null
    }
  }

  /**
   * 预估 Gas 费用
   */
  async function estimateGas(data: Partial<SendTxParams>): Promise<{ gas_limit: number; gas_price: string } | null> {
    try {
      const response: ApiResponse<{ gas_limit: number; gas_price: string }> =
        await transactionApi.estimateGas(data)
      return response.data
    } catch (error: any) {
      console.error('Failed to estimate gas', error)
      ElMessage.error('Gas 预估失败')
      return null
    }
  }

  /**
   * 更新交易状态（轮询时使用）
   */
  function updateTransactionStatus(hash: string, status: Transaction['status']) {
    const tx = transactions.value.find(t => t.hash === hash)
    if (tx) {
      tx.status = status
    }

    const pendingTx = pendingTransactions.value.find(t => t.hash === hash)
    if (pendingTx) {
      pendingTx.status = status
      // 如果已确认，从待确认中移除
      if (status === 'success' || status === 'failed') {
        const index = pendingTransactions.value.findIndex(t => t.hash === hash)
        if (index !== -1) {
          pendingTransactions.value.splice(index, 1)
        }
      }
    }
  }

  /**
   * 格式化交易显示
   */
  function getTransactionDisplay(tx: Transaction) {
    return {
      hash: formatAddress(tx.hash),
      value: formatWeiToEth(tx.value),
      from: formatAddress(tx.from),
      to: formatAddress(tx.to),
      status: tx.status,
      time: formatTime(tx.created_at)
    }
  }

  /**
   * 清除所有缓存
   */
  function clearCache() {
    transactions.value = []
    pendingTransactions.value = []
    total.value = 0
    currentPage.value = 1
    pageSize.value = 20
    statusFilter.value = ''
    addressFilter.value = ''
  }

  return {
    // state
    transactions,
    pendingTransactions,
    total,
    currentPage,
    pageSize,
    isLoading,
    statusFilter,
    addressFilter,
    // getters
    hasNextPage,
    confirmedTransactions,
    successfulTransactions,
    failedTransactions,
    // actions
    fetchTransactions,
    loadNextPage,
    refresh,
    getTransaction,
    sendTransaction,
    estimateGas,
    updateTransactionStatus,
    getTransactionDisplay,
    clearCache
  }
})
