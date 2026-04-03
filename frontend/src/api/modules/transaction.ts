import apiClient from '../client'
import type {
  ApiResponse,
  Transaction,
  SendTransactionRequest,
  TransactionFilter,
  PaginatedResponse,
  FeeEstimate,
  TransactionReceipt
} from '@/types'

/**
 * 交易管理 API 模块
 */

export const transactionApi = {
  /**
   * 获取交易列表
   * @param params 筛选条件和分页参数
   * @returns 分页交易列表
   */
  async getTransactions(
    params: TransactionFilter & { page?: number; size?: number }
  ): Promise<ApiResponse<PaginatedResponse<Transaction>>> {
    return apiClient.get('/transactions', { params })
  },

  /**
   * 根据交易哈希查询交易
   * @param hash 交易哈希
   * @returns 交易详情
   */
  async getTransactionByHash(
    hash: string
  ): Promise<ApiResponse<Transaction>> {
    return apiClient.get(`/transactions/hash/${hash}`)
  },

  /**
   * 根据区块号获取交易列表
   * @param blockNumber 区块号
   * @returns 该区块中的所有交易
   */
  async getTransactionsByBlock(
    blockNumber: number
  ): Promise<ApiResponse<Transaction[]>> {
    return apiClient.get(`/transactions/block/${blockNumber}`)
  },

  /**
   * 发送交易（需要钱包签名）
   * @param data 交易数据
   * @returns 交易哈希
   */
  async sendTransaction(
    data: SendTransactionRequest
  ): Promise<ApiResponse<{ txHash: string }>> {
    return apiClient.post('/transactions/send', data)
  },

  /**
   * 批量发送交易
   * @param transactions 交易数据数组
   * @returns 所有交易的哈希列表
   */
  async batchSendTransactions(
    transactions: SendTransactionRequest[]
  ): Promise<ApiResponse<{ txHashes: string[] }>> {
    return apiClient.post('/transactions/batch-send', { transactions })
  },

  /**
   * 预估交易费用
   * @param data 交易参数
   * @returns 费用估算结果
   */
  async estimateFee(
    data: Partial<SendTransactionRequest>
  ): Promise<ApiResponse<FeeEstimate>> {
    return apiClient.post('/transactions/estimate-fee', data)
  },

  /**
   * 查询交易收据
   * @param txHash 交易哈希
   * @returns 交易收据
   */
  async getTransactionReceipt(
    txHash: string
  ): Promise<ApiResponse<TransactionReceipt>> {
    return apiClient.get(`/transactions/${txHash}/receipt`)
  },

  /**
   * 取消挂起的交易（替换为高GasPrice的相同交易）
   * @param txHash 待取消的交易哈希
   * @param newGasPrice 新的GasPrice
   * @returns 新交易哈希
   */
  async cancelTransaction(
    txHash: string,
    newGasPrice: number
  ): Promise<ApiResponse<{ txHash: string }>> {
    return apiClient.post(`/transactions/${txHash}/cancel`, { gasPrice: newGasPrice })
  },

  /**
   * 获取交易状态（确认数）
   * @param txHash 交易哈希
   * @returns 交易状态和确认数
   */
  async getTransactionStatus(
    txHash: string
  ): Promise<ApiResponse<{ status: string; confirmations: number }>> {
    return apiClient.get(`/transactions/${txHash}/status`)
  },

  /**
   * 查询账户的交易计数（nonce）
   * @param address 账户地址
   * @returns 当前nonce值
   */
  async getAccountNonce(address: string): Promise<ApiResponse<{ nonce: number }>> {
    return apiClient.get(`/accounts/${address}/nonce`)
  },

  /**
   * 获取账户余额
   * @param address 账户地址
   * @param tokenAddress 代币合约地址（可选，不传则查原生代币）
   * @returns 余额
   */
  async getAccountBalance(
    address: string,
    tokenAddress?: string
  ): Promise<ApiResponse<{ balance: string; symbol: string; decimals: number }>> {
    const params = tokenAddress ? { token: tokenAddress } : {}
    return apiClient.get(`/accounts/${address}/balance`, { params })
  }
}

export default transactionApi
