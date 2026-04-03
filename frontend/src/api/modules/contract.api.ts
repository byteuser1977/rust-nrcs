import type { ApiResponse, Contract, PageResult, DeployContractParams, CallContractParams, ContractEvent } from '@/types'
import { get, post } from '../client'

/**
 * 合约相关 API 模块
 */
export const contractApi = {
  /**
   * 获取合约列表（分页）
   */
  getContracts(params: { page?: number; size?: number; creator?: string }) {
    return get<ApiResponse<PageResult<Contract>>>('/contracts', { params })
  },

  /**
   * 获取合约详情
   */
  getContract(address: string) {
    return get<ApiResponse<Contract>>(`/contracts/${address}`)
  },

  /**
   * 部署合约
   */
  deploy(data: DeployContractParams) {
    return post<ApiResponse<{ contract_address: string; tx_hash: string }>>('/contracts/deploy', data)
  },

  /**
   * 调用合约（只读）
   */
  call(data: CallContractParams) {
    return post<ApiResponse<any>>('/contracts/call', data)
  },

  /**
   * 发送合约交易（写入）
   */
  sendTx(data: CallContractParams) {
    return post<ApiResponse<{ tx_hash: string }>>('/contracts/transact', data)
  },

  /**
   * 获取合约事件日志
   */
  getEvents(address: string, params: { from_block?: number; to_block?: number; event_name?: string }) {
    return get<ApiResponse<ContractEvent[]>>(`/contracts/${address}/events`, { params })
  },

  /**
   * 验证合约源码
   */
  verify(data: { address: string; source_code: string; compiler_version: string; optimization_used: boolean }) {
    return post<ApiResponse<{ is_verified: boolean }>>('/contracts/verify', data)
  },

  /**
   * 生成合约 ABI 接口代码
   */
  generateInterface(address: string) {
    return get<ApiResponse<{ abi: any[]; code_samples: Record<string, string> }>>(`/contracts/${address}/interface`)
  },

  /**
   * 读取合约 Storage
   */
  getStorage(address: string, slot: number) {
    return get<ApiResponse<{ value: string }>>(`/contracts/${address}/storage/${slot}`)
  },

  /**
   * 获取合约代码
   */
  getCode(address: string) {
    return get<ApiResponse<{ bytecode: string; runtime_bytecode: string }>>(`/contracts/${address}/code`)
  }
}
