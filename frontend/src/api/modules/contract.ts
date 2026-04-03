import apiClient from '../client'
import type {
  ApiResponse,
  Contract,
  ContractDeployRequest,
  ContractCallRequest,
  ContractEvent,
  ContractABI
} from '@/types'

/**
 * 合约管理 API 模块
 */

export const contractApi = {
  /**
   * 获取合约列表
   * @param params 查询参数
   * @returns 分页合约列表
   */
  async getContracts(
    params?: { page?: number; size?: number; address?: string; name?: string }
  ): Promise<ApiResponse<any>> {
    return apiClient.get('/contracts', { params })
  },

  /**
   * 根据地址查询合约
   * @param address 合约地址
   * @returns 合约详情
   */
  async getContractByAddress(
    address: string
  ): Promise<ApiResponse<Contract>> {
    return apiClient.get(`/contracts/address/${address}`)
  },

  /**
   * 部署合约（需要钱包签名）
   * @param data 部署参数
   * @returns 合约地址和交易哈希
   */
  async deployContract(
    data: ContractDeployRequest
  ): Promise<ApiResponse<{ contractAddress: string; txHash: string }>> {
    return apiClient.post('/contracts/deploy', data)
  },

  /**
   * 验证合约代码
   * @param data 验证参数（地址、源代码、编译器版本）
   * @returns 验证结果
   */
  async verifyContract(
    data: { address: string; sourceCode: string; compilerVersion: string; optimizer?: boolean }
  ): Promise<ApiResponse<{ verified: boolean; message?: string }>> {
    return apiClient.post('/contracts/verify', data)
  },

  /**
   * 调用合约方法（只读）
   * @param data 调用参数
   * @returns 调用结果
   */
  async callContract(
    data: ContractCallRequest
  ): Promise<ApiResponse<{ result: string; decoded?: any }>> {
    return apiClient.post('/contracts/call', data)
  },

  /**
   * 发送合约交易（写操作，需要签名）
   * @param data 交易参数
   * @returns 交易哈希
   */
  async sendContractTransaction(
    data: ContractCallRequest & { value?: string; gasLimit?: number }
  ): Promise<ApiResponse<{ txHash: string }>> {
    return apiClient.post('/contracts/transact', data)
  },

  /**
   * 获取合约事件
   * @param params 筛选条件
   * @returns 事件列表
   */
  async getContractEvents(
    params: {
      contractAddress: string
      eventName?: string
      fromBlock?: number
      toBlock?: number
      page?: number
      size?: number
    }
  ): Promise<ApiResponse<any>> {
    return apiClient.get('/contracts/events', { params })
  },

  /**
   * 获取合约ABI
   * @param address 合约地址
   * @returns ABI定义
   */
  async getContractABI(
    address: string
  ): Promise<ApiResponse<ContractABI>> {
    return apiClient.get(`/contracts/${address}/abi`)
  },

  /**
   * 读取合约存储数据
   * @param address 合约地址
   * @param slot 存储插槽
   * @returns 存储值
   */
  async getStorageAt(
    address: string,
    slot: number | string
  ): Promise<ApiResponse<{ value: string }>> {
    return apiClient.get(`/contracts/${address}/storage`, { params: { slot } })
  },

  /**
   * 估算合约调用Gas消耗
   * @param data 调用参数
   * @returns Gas估算
   */
  async estimateContractGas(
    data: ContractCallRequest & { value?: string }
  ): Promise<ApiResponse<{ gasEstimate: number; recommendedGasLimit: number }>> {
    return apiClient.post('/contracts/estimate-gas', data)
  },

  /**
   * 获取合约的Token信息（如果是代币合约）
   * @param address 合约地址
   * @returns 代币信息
   */
  async getTokenInfo(
    address: string
  ): Promise<ApiResponse<any>> {
    return apiClient.get(`/contracts/${address}/token`)
  },

  /**
   * 批量查询合约状态
   * @param addresses 合约地址列表
   * @returns 合约状态映射
   */
  async batchGetContractStatus(
    addresses: string[]
  ): Promise<ApiResponse<Record<string, Contract>>> {
    return apiClient.post('/contracts/batch-status', { addresses })
  }
}

export default contractApi
