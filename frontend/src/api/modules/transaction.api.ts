import type { ApiResponse, Transaction, PageResult, SendTxParams, GasEstimate } from '@/types'
import { get, post } from '../client'

/**
 * 交易相关 API 模块
 */
export const transactionApi = {
  /**
   * 查询交易列表（分页）
   */
  getTransactions(params: {
    page?: number
    size?: number
    status?: string
    address?: string
    start_time?: string
    end_time?: string
  }) {
    return get<ApiResponse<PageResult<Transaction>>>('/transactions', { params })
  },

  /**
   * 获取交易详情
   */
  getTransaction(hash: string) {
    return get<ApiResponse<Transaction>>(`/transactions/${hash}`)
  },

  /**
   * 发送交易
   */
  sendTransaction(data: SendTxParams) {
    return post<ApiResponse<{ tx_hash: string }>>('/transactions/send', data)
  },

  /**
   * 预估 Gas 费用
   */
  estimateGas(data: Partial<SendTxParams>) {
    return post<ApiResponse<GasEstimate>>('/transactions/estimate-gas', data)
  },

  /**
   * 获取钱包余额
   */
  getBalance(address: string) {
    return get<ApiResponse<{ balance: string; symbol: string }>>('/transactions/balance', {
      params: { address }
    })
  },

  /**
   * 获取 Gas 价格
   */
  getGasPrice() {
    return get<ApiResponse<{ slow: string; standard: string; fast: string; instant: string }>>('/transactions/gas-price')
  },

  /**
   * 构建并发送 Native Token 转账
   */
  transfer(data: { from: string; to: string; amount: string; gasPrice?: string; gasLimit?: number }) {
    return post<ApiResponse<{ tx_hash: string }>>('/transactions/transfer', data)
  },

  /**
   * 获取待确认交易
   */
  getPendingTransactions(address: string) {
    return get<ApiResponse<Transaction[]>>('/transactions/pending', { params: { address } })
  },

  /**
   * 取消交易（替换为更高 gas 的新交易）
   */
  cancelTransaction(txHash: string, gasPrice: string) {
    return post<ApiResponse<{ tx_hash: string }>>('/transactions/cancel', {
      original_tx_hash: txHash,
      gas_price: gasPrice
    })
  }
}
