/**
 * @deprecated Ethereum 风格节点 API（/node/info、/node/blocks、gas-stats、txpool 等）。
 *
 * NRCS 节点状态通过 `nrcsApi.getBlockchainStatus` / `nrcsApi.getState` / `nrcsApi.getPeers` /
 * `nrcsApi.getBlocks` / `nrcsApi.getBlock` 获取，字段为 height / baseTarget / cumulativeDifficulty 等。
 *
 * 保留该文件仅为兼容尚未重写的以太坊占位视图。新代码禁止使用。
 *
 * @see @/api/modules/nrcs.api.ts nrcsApi（推荐）
 * @see /Volumes/DATA/data/develop/git/nrcs/nrcs-main/html/www/ui/js/nrs.blocks.js / nrs.peers.js 参考实现
 */
import type { ApiResponse, NodeInfo, NetworkStats, BlockInfo } from '@/types'
import { get } from '../client'

/**
 * 节点监控相关 API 模块
 */
export const nodeApi = {
  /**
   * 获取节点信息
   */
  getNodeInfo() {
    return get<ApiResponse<NodeInfo>>('/node/info')
  },

  /**
   * 获取网络统计
   */
  getNetworkStats() {
    return get<ApiResponse<NetworkStats>>('/node/stats')
  },

  /**
   * 获取最新区块列表
   */
  getLatestBlocks(limit: number = 20) {
    return get<ApiResponse<BlockInfo[]>>('/node/blocks/latest', { params: { limit } })
  },

  /**
   * 按区块号查询区块
   */
  getBlockByNumber(number: number) {
    return get<ApiResponse<BlockInfo>>(`/node/blocks/number/${number}`)
  },

  /**
   * 按哈希查询区块
   */
  getBlockByHash(hash: string) {
    return get<ApiResponse<BlockInfo>>(`/node/blocks/hash/${hash}`)
  },

  /**
   * 获取节点健康状态
   */
  getHealth() {
    return get<ApiResponse<{ status: 'healthy' | 'unhealthy'; checks: Record<string, any> }>>('/node/health')
  },

  /**
   * 获取节点 peers 列表
   */
  getPeers() {
    return get<ApiResponse<Array<{ node_id: string; address: string; client_version: string }>>('/node/peers')
  },

  /**
   * 获取链信息
   */
  getChainInfo() {
    return get<ApiResponse<{
      chain_id: number
      chain_name: string
      consensus_algorithm: string
      block_time: number
      latest_block: number
    }>>('/node/chain')
  },

  /**
   * 获取交易池状态（pending transactions）
   */
  getTxPoolStats() {
    return get<ApiResponse<{
      pending: number
      queued: number
    }>>('/node/txpool')
  },

  /**
   * 获取 Gas 价格统计
   */
  getGasStats() {
    return get<ApiResponse<{
      base_fee: string
      avg_fee: string
      fast_fee: string
      instant_fee: string
    }>>('/node/gas-stats')
  }
}
