<template>
  <div class="node-status-page">
    <el-row :gutter="20" style="margin-bottom: 20px;">
      <!-- 节点健康状态 -->
      <el-col :xs="24" :sm="12" :lg="6">
        <el-card shadow="hover" class="status-card">
          <div class="status-indicator">
            <div
              class="dot"
              :class="{ healthy: nodeInfo?.sync_status === 'synced', warning: nodeInfo?.sync_status === 'syncing', danger: nodeInfo?.sync_status === 'caught_up' }"
            ></div>
            <div class="status-text">
              <span class="label">同步状态</span>
              <span class="value">{{ syncStatusText }}</span>
            </div>
          </div>
        </el-card>
      </el-col>

      <el-col :xs="24" :sm="12" :lg="6">
        <el-card shadow="hover" class="metric-card">
          <div class="metric">
            <span class="metric-label">当前区块</span>
            <span class="metric-value">{{ nodeInfo?.block_number.toLocaleString() || '0' }}</span>
          </div>
        </el-card>
      </el-col>

      <el-col :xs="24" :sm="12" :lg="6">
        <el-card shadow="hover" class="metric-card">
          <div class="metric">
            <span class="metric-label">连接节点</span>
            <span class="metric-value">{{ nodeInfo?.peer_count || '0' }}</span>
          </div>
        </el-card>
      </el-col>

      <el-col :xs="24" :sm="12" :lg="6">
        <el-card shadow="hover" class="metric-card">
          <div class="metric">
            <span class="metric-label">运行时间</span>
            <span class="metric-value">{{ formattedUptime }}</span>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <el-row :gutter="20">
      <!-- TPS 图表 -->
      <el-col :xs="24" :lg="12">
        <el-card>
          <template #header>
            <span>交易性能 (TPS)</span>
            <el-button
              size="small"
              text
              type="primary"
              @click="refreshData"
              :loading="refreshing"
            >
              刷新
            </el-button>
          </template>
          <div ref="tpsChartRef" class="chart-container"></div>
        </el-card>
      </el-col>

      <!-- 网络统计 -->
      <el-col :xs="24" :lg="12">
        <el-card>
          <template #header>
            <span>网络统计</span>
          </template>
          <div class="network-stats">
            <div class="stat-item">
              <div class="stat-label">总交易数</div>
              <div class="stat-value">{{ networkStats?.total_transactions.toLocaleString() || '0' }}</div>
            </div>
            <div class="stat-item">
              <div class="stat-label">总区块数</div>
              <div class="stat-value">{{ networkStats?.total_blocks.toLocaleString() || '0' }}</div>
            </div>
            <div class="stat-item">
              <div class="stat-label">总账户数</div>
              <div class="stat-value">{{ networkStats?.total_accounts.toLocaleString() || '0' }}</div>
            </div>
            <div class="stat-item">
              <div class="stat-label">平均出块时间</div>
              <div class="stat-value">{{ networkStats?.block_time }}</div>
            </div>
            <div class="stat-item">
              <div class="stat-label">平均Gas价格</div>
              <div class="stat-value">{{ formatGasPrice(networkStats?.gas_price_avg) }}</div>
            </div>
            <div class="stat-item">
              <div class="stat-label">待处理交易</div>
              <div class="stat-value">{{ networkStats?.pending_txs }}</div>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 实时监控选项 -->
    <el-row :gutter="20" style="margin-top: 20px;">
      <el-col :span="24">
        <el-card>
          <template #header>
            <span>监控设置</span>
          </template>
          <div class="monitor-controls">
            <el-button
              :type="isPolling ? 'danger' : 'primary'"
              @click="togglePolling"
            >
              <el-icon><VideoCamera v-if="isPolling" /><VideoCameraFilled v-else /></el-icon>
              {{ isPolling ? '停止刷新' : '开始刷新' }}
            </el-button>
            <span class="polling-info" v-if="isPolling">
              每 {{ POLL_INTERVAL / 1000 }} 秒自动刷新
            </span>
          </div>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useNodeStore } from '@/stores/modules/node.store'
import { ElMessage } from 'element-plus'
import { VideoCamera, VideoCameraFilled } from '@element-plus/icons-vue'
import * as echarts from 'echarts'

const nodeStore = useNodeStore()

const refreshing = ref(false)
const tpsChartRef = ref<HTMLElement>()
let tpsChart: echarts.ECharts | null = null
const POLL_INTERVAL = 10000 // 10秒

const nodeInfo = computed(() => nodeStore.nodeInfo)
const networkStats = computed(() => nodeStore.networkStats)
const isPolling = computed(() => nodeStore.isPolling)
const formattedUptime = computed(() => nodeStore.formattedUptime)

const syncStatusText = computed(() => {
  if (!nodeInfo.value) return '未知'
  switch (nodeInfo.value.sync_status) {
    case 'synced': return '已同步'
    case 'syncing': return '同步中'
    case 'caught_up': return '已追上'
    default: return nodeInfo.value.sync_status
  }
})

// 刷新数据
const refreshData = async () => {
  try {
    refreshing.value = true
    await nodeStore.refreshAll()
  } catch (error) {
    ElMessage.error('刷新数据失败')
  } finally {
    refreshing.value = false
  }
}

// 切换轮询
const togglePolling = () => {
  if (isPolling.value) {
    nodeStore.stopPolling()
  } else {
    nodeStore.startPolling()
  }
}

// 格式化Gas价格
const formatGasPrice = (value?: string): string => {
  if (!value) return '-'
  const wei = BigInt(value)
  const gwei = wei / BigInt(1e9)
  return `${gwei.toLocaleString()} Gwei`
}

// 初始化TPS图表
const initTpsChart = () => {
  if (!tpsChartRef.value) return

  tpsChart = echarts.init(tpsChartRef.value)
  const option = {
    title: {
      text: 'TPS 实时趋势',
      left: 'center'
    },
    tooltip: {
      trigger: 'axis'
    },
    xAxis: {
      type: 'category',
      data: [] as string[]
    },
    yAxis: {
      type: 'value',
      name: 'TPS'
    },
    series: [{
      data: [] as number[],
      type: 'line',
      smooth: true,
      areaStyle: {
        color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
          { offset: 0, color: 'rgba(64, 158, 255, 0.3)' },
          { offset: 1, color: 'rgba(64, 158, 255, 0.05)' }
        ])
      }
    }]
  }
  tpsChart.setOption(option)
}

// 更新TPS图表
const updateTpsChart = () => {
  if (!tpsChart || !networkStats.value) return

  const now = new Date().toLocaleTimeString()
  const tps = networkStats.value.tps

  const option = tpsChart.getOption()
  const xData = option.xAxis[0].data as string[]
  const seriesData = option.series[0].data as number[]

  xData.push(now)
  seriesData.push(tps)

  // 保留最近20个数据点
  if (xData.length > 20) {
    xData.shift()
    seriesData.shift()
  }

  tpsChart.setOption({
    xAxis: { data: xData },
    series: [{ data: seriesData }]
  })
}

// 监听网络统计变化
watch(
  () => networkStats.value,
  () => {
    updateTpsChart()
  },
  { deep: true }
)

onMounted(async () => {
  await refreshData()
  initTpsChart()
  updateTpsChart()
})

onUnmounted(() => {
  if (tpsChart) {
    tpsChart.dispose()
  }
  nodeStore.stopPolling()
})
</script>

<script lang="ts">
import { watch } from 'vue'
</script>

<style scoped lang="scss">
.node-status-page {
  .status-card {
    .status-indicator {
      display: flex;
      align-items: center;
      gap: 12px;

      .dot {
        width: 12px;
        height: 12px;
        border-radius: 50%;
        background: #909399;

        &.healthy {
          background: #67c23a;
          box-shadow: 0 0 8px rgba(103, 194, 58, 0.6);
        }

        &.warning {
          background: #e6a23c;
          box-shadow: 0 0 8px rgba(230, 162, 60, 0.6);
        }

        &.danger {
          background: #f56c6c;
          box-shadow: 0 0 8px rgba(245, 108, 108, 0.6);
        }
      }

      .status-text {
        display: flex;
        flex-direction: column;

        .label {
          font-size: 12px;
          color: #909399;
        }

        .value {
          font-size: 16px;
          font-weight: 600;
          color: #303133;
        }
      }
    }
  }

  .metric-card {
    .metric {
      text-align: center;
      padding: 12px;

      .metric-label {
        display: block;
        font-size: 13px;
        color: #909399;
        margin-bottom: 8px;
      }

      .metric-value {
        display: block;
        font-size: 24px;
        font-weight: bold;
        color: #303133;
        font-family: 'Roboto Mono', monospace;
      }
    }
  }

  .chart-container {
    height: 300px;
  }

  .network-stats {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 16px;

    .stat-item {
      padding: 12px;
      background: #f5f7fa;
      border-radius: 8px;
      text-align: center;

      .stat-label {
        font-size: 13px;
        color: #909399;
        margin-bottom: 4px;
      }

      .stat-value {
        font-size: 18px;
        font-weight: 600;
        color: #303133;
        font-family: 'Roboto Mono', monospace;
      }
    }
  }

  .monitor-controls {
    display: flex;
    align-items: center;
    gap: 16px;

    .polling-info {
      font-size: 13px;
      color: #909399;
    }
  }
}
</style>
