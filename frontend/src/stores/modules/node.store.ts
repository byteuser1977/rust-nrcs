import { defineStore } from 'pinia'
import { ref, computed, onUnmounted } from 'vue'
import type { NodeInfo, NetworkStats, BlockInfo } from '@/types/business'
import { nodeApi } from '@/api/modules'
import type { ApiResponse } from '@/types'
import { ElMessage } from 'element-plus'
import { formatTime } from '@/utils/format'

// 轮询间隔（毫秒）
const POLL_INTERVAL = 30000  // 30秒

export const useNodeStore = defineStore('node', () => {
  // 节点信息
  const nodeInfo = ref<NodeInfo | null>(null)
  const networkStats = ref<NetworkStats | null>(null)
  const latestBlocks = ref<BlockInfo[]>([])

  // 健康状态
  const isHealthy = ref(false)
  const healthCheckTime = ref<string>('')

  // 轮询控制
  const isPolling = ref(false)
  const pollTimer = ref<number | null>(null)

  // 计算属性
  const currentBlockNumber = computed(() => nodeInfo.value?.block_number || 0)
  const peerCount = computed(() => nodeInfo.value?.peer_count || 0)
  const nodeVersion = computed(() => nodeInfo.value?.version || 'Unknown')
  const uptime = computed(() => nodeInfo.value?.uptime || 0)

  const formattedUptime = computed(() => {
    const days = Math.floor(uptime.value / 86400)
    const hours = Math.floor((uptime.value % 86400) / 3600)
    const minutes = Math.floor((uptime.value % 3600) / 60)
    return `${days}d ${hours}h ${minutes}m`
  })

  const tps = computed(() => networkStats.value?.tps || 0)
  const totalTransactions = computed(() => networkStats.value?.total_transactions || 0)
  const networkDifficulty = computed(() => networkStats.value?.difficulty || '0')

  /**
   * 获取节点信息
   */
  async function fetchNodeInfo(): Promise<void> {
    try {
      const response: ApiResponse<NodeInfo> = await nodeApi.getNodeInfo()
      nodeInfo.value = response.data
    } catch (error: any) {
      console.error('Failed to fetch node info', error)
      // 不显示错误消息，因为轮询期间可能失败
    }
  }

  /**
   * 获取网络统计
   */
  async function fetchNetworkStats(): Promise<void> {
    try {
      const response: ApiResponse<NetworkStats> = await nodeApi.getNetworkStats()
      networkStats.value = response.data
    } catch (error: any) {
      console.error('Failed to fetch network stats', error)
    }
  }

  /**
   * 获取最新区块
   */
  async function fetchLatestBlocks(limit: number = 20): Promise<void> {
    try {
      const response: ApiResponse<BlockInfo[]> = await nodeApi.getLatestBlocks(limit)
      latestBlocks.value = response.data
    } catch (error: any) {
      console.error('Failed to fetch latest blocks', error)
      ElMessage.error('获取区块数据失败')
    }
  }

  /**
   * 获取节点健康状态
   */
  async function checkHealth(): Promise<boolean> {
    try {
      const response: ApiResponse<{ status: 'healthy' | 'unhealthy'; checks: any }> =
        await nodeApi.getHealth()
      isHealthy.value = response.data.status === 'healthy'
      healthCheckTime.value = new Date().toISOString()
      return isHealthy.value
    } catch (error: any) {
      isHealthy.value = false
      console.error('Health check failed', error)
      return false
    }
  }

  /**
   * 刷新所有数据（单次）
   */
  async function refreshAll(): Promise<void> {
    await Promise.all([
      fetchNodeInfo(),
      fetchNetworkStats(),
      fetchLatestBlocks(),
      checkHealth()
    ])
  }

  /**
   * 开始轮询数据
   */
  function startPolling() {
    if (isPolling.value) return

    isPolling.value = true

    // 立即执行一次
    refreshAll().catch(console.error)

    // 设置定时器
    pollTimer.value = window.setInterval(() => {
      if (!isPolling.value) return
      refreshAll().catch(console.error)
    }, POLL_INTERVAL)
  }

  /**
   * 停止轮询数据
   */
  function stopPolling() {
    if (pollTimer.value) {
      clearInterval(pollTimer.value)
      pollTimer.value = null
    }
    isPolling.value = false
  }

  /**
   * 格式化区块时间
   */
  function formatBlockTime(timestamp: number): string {
    return formatTime(new Date(timestamp * 1000).toISOString())
  }

  /**
   * 计算区块生成时间间隔（秒）
   */
  function calculateBlockInterval(): number {
    if (latestBlocks.value.length < 2) return 0

    const latest = latestBlocks.value[0]
    const previous = latestBlocks.value[1]

    if (!latest.timestamp || !previous.timestamp) return 0

    return latest.timestamp - previous.timestamp
  }

  /**
   * 获取健康状态颜色
   */
  function getHealthColor(): string {
    return isHealthy.value ? '#67c23a' : '#f56c6c'
  }

  /**
   * 获取健康状态文本
   */
  function getHealthText(): string {
    return isHealthy.value ? '健康' : '异常'
  }

  /**
   * 清除所有缓存
   */
  function clearCache() {
    nodeInfo.value = null
    networkStats.value = null
    latestBlocks.value = []
    isHealthy.value = false
    healthCheckTime.value = ''
    stopPolling()
  }

  // 组件卸载时停止轮询
  onUnmounted(() => {
    stopPolling()
  })

  return {
    // state
    nodeInfo,
    networkStats,
    latestBlocks,
    isHealthy,
    healthCheckTime,
    isPolling,
    // getters
    currentBlockNumber,
    peerCount,
    nodeVersion,
    formattedUptime,
    tps,
    totalTransactions,
    networkDifficulty,
    // actions
    fetchNodeInfo,
    fetchNetworkStats,
    fetchLatestBlocks,
    checkHealth,
    refreshAll,
    startPolling,
    stopPolling,
    formatBlockTime,
    calculateBlockInterval,
    getHealthColor,
    getHealthText,
    clearCache
  }
})
