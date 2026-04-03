import apiClient from '../client'
import type { ApiResponse, NodeStatus, PeerInfo, BlockInfo, NodeMetrics, ChainInfo } from '@/types'

/**
 * 节点管理 API 模块
 */

export const nodeApi = {
  /**
   * 获取节点状态
   * @returns 节点状态信息
   */
  async getStatus(): Promise<ApiResponse<NodeStatus>> {
    return apiClient.get('/node/status')
  },

  /**
   * 获取节点信息
   * @returns 节点详细信息
   */
  async getInfo(): Promise<ApiResponse<ChainInfo>> {
    return apiClient.get('/node/info')
  },

  /**
   * 获取对等节点列表
   * @returns P2P网络节点列表
   */
  async getPeers(): Promise<ApiResponse<PeerInfo[]>> {
    return apiClient.get('/node/peers')
  },

  /**
   * 添加对等节点
   * @param enodeUrl ENODE URL
   * @returns 操作结果
   */
  async addPeer(enodeUrl: string): Promise<ApiResponse<{ success: boolean }>> {
    return apiClient.post('/node/peers', { enode: enodeUrl })
  },

  /**
   * 移除对等节点
   * @param nodeId 节点ID
   * @returns 操作结果
   */
  async removePeer(nodeId: string): Promise<ApiResponse<{ success: boolean }>> {
    return apiClient.delete(`/node/peers/${nodeId}`)
  },

  /**
   * 获取区块列表
   * @param params 查询参数
   * @returns 分页区块列表
   */
  async getBlocks(
    params: {
      page?: number
      size?: number
      fromNumber?: number
      toNumber?: number
      miner?: string
    } = {}
  ): Promise<ApiResponse<any>> {
    return apiClient.get('/node/blocks', { params })
  },

  /**
   * 根据哈希查询区块
   * @param hash 区块哈希
   * @returns 区块详情
   */
  async getBlockByHash(hash: string): Promise<ApiResponse<BlockInfo>> {
    return apiClient.get(`/node/blocks/hash/${hash}`)
  },

  /**
   * 根据高度查询区块
   * @param number 区块高度
   * @returns 区块详情
   */
  async getBlockByNumber(number: number): Promise<ApiResponse<BlockInfo>> {
    return apiClient.get(`/node/blocks/number/${number}`)
  },

  /**
   * 获取最新区块
   * @returns 最新区块信息
   */
  async getLatestBlock(): Promise<ApiResponse<BlockInfo>> {
    return apiClient.get('/node/blocks/latest')
  },

  /**
   * 获取节点监控指标
   * @returns 性能监控数据
   */
  async getMetrics(): Promise<ApiResponse<NodeMetrics>> {
    return apiClient.get('/node/metrics')
  },

  /**
   * 查询区块链状态
   * @returns 区块链基本信息
   */
  async getChainInfo(): Promise<ApiResponse<ChainInfo>> {
    return apiClient.get('/node/chain')
  },

  /**
   * 获取Gas价格
   * @returns 当前Gas价格信息
   */
  async getGasPrice(): Promise<ApiResponse<{ baseFee: number; priorityFee: number }>> {
    return apiClient.get('/node/gas-price')
  },

  /**
   * 重启节点（管理员操作）
   * @returns 操作结果
   */
  async restart(): Promise<ApiResponse<null>> {
    return apiClient.post('/node/restart')
  },

  /**
   * 停止节点（管理员操作）
   * @returns 操作结果
   */
  async stop(): Promise<ApiResponse<null>> {
    return apiClient.post('/node/stop')
  },

  /**
   * 导出节点日志
   * @param params 筛选参数
   * @returns 日志文件
   */
  async exportLogs(
    params?: { level?: string; module?: string; since?: number }
  ): Promise<ApiResponse<{ url: string }>> {
    return apiClient.get('/node/logs/export', { params })
  },

  /**
   * 获取同步状态
   * @returns 同步进度
   */
  async getSyncStatus(): Promise<ApiResponse<any>> {
    return apiClient.get('/node/sync')
  },

  /**
   * 设置共识参数（管理员操作）
   * @param data 共识参数
   * @returns 操作结果
   */
  async setConsensusParams(
    data: { blockTime?: number; difficulty?: number }
  ): Promise<ApiResponse<null>> {
    return apiClient.put('/node/consensus', data)
  },

  /**
   * 获取节点版本
   * @returns 版本信息
   */
  async getVersion(): Promise<ApiResponse<{ version: string; build: string }>> {
    return apiClient.get('/node/version')
  }
}

export default nodeApi
