<template>
  <div class="node-monitor-page">
    <!-- Real-time Stats Cards -->
    <el-row :gutter="20" style="margin-bottom: 20px;">
      <el-col :xs="24" :sm="12" :lg="6">
        <el-card shadow="hover" class="metric-card">
          <div class="metric">
            <span class="metric-label">{{ t('node.tps') }}</span>
            <span class="metric-value">{{ tps }}</span>
          </div>
        </el-card>
      </el-col>

      <el-col :xs="24" :sm="12" :lg="6">
        <el-card shadow="hover" class="metric-card">
          <div class="metric">
            <span class="metric-label">{{ t('node.pendingTxCount') }}</span>
            <span class="metric-value">{{ state?.numberOfUnconfirmedTransactions?.toLocaleString() || '0' }}</span>
          </div>
        </el-card>
      </el-col>

      <el-col :xs="24" :sm="12" :lg="6">
        <el-card shadow="hover" class="metric-card">
          <div class="metric">
            <span class="metric-label">{{ t('node.totalPeers') }}</span>
            <span class="metric-value">{{ state?.numberOfPeers?.toLocaleString() || '0' }}</span>
          </div>
        </el-card>
      </el-col>

      <el-col :xs="24" :sm="12" :lg="6">
        <el-card shadow="hover" class="metric-card">
          <div class="metric">
            <span class="metric-label">{{ t('node.memoryUsage') }}</span>
            <span class="metric-value">{{ memoryUsage }}</span>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- ECharts Charts -->
    <el-row :gutter="20" style="margin-bottom: 20px;">
      <el-col :xs="24" :lg="12">
        <el-card shadow="never">
          <template #header>
            <span>{{ t('node.blocksPerHour') }}</span>
          </template>
          <div ref="blocksChartRef" class="chart-container"></div>
        </el-card>
      </el-col>

      <el-col :xs="24" :lg="12">
        <el-card shadow="never">
          <template #header>
            <span>{{ t('node.transactionsPerHour') }}</span>
          </template>
          <div ref="txChartRef" class="chart-container"></div>
        </el-card>
      </el-col>
    </el-row>

    <!-- Cumulative Difficulty Chart -->
    <el-row :gutter="20">
      <el-col :span="24">
        <el-card shadow="never">
          <template #header>
            <span>{{ t('node.difficultyTrend') }}</span>
          </template>
          <div ref="difficultyChartRef" class="chart-container-large"></div>
        </el-card>
      </el-col>
    </el-row>

    <!-- Refresh Info -->
    <div class="polling-info" v-if="lastPollTime">
      {{ t('node.autoRefresh', { seconds: POLL_INTERVAL / 1000 }) }}
      &mdash; {{ t('node.lastRefresh') }}: {{ formatTime(lastPollTime) }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import * as echarts from 'echarts'
import { nrcsApi } from '@/api/modules/nrcs.api'
import type { NrcsBlockchainStatus } from '@/api/modules/nrcs.api'
import { usePolling } from '@/composables/usePolling'
import { formatTimestamp, formatVolume } from '@/utils/format'

const { t } = useI18n()

const POLL_INTERVAL = 5000
const MAX_DATA_POINTS = 30

const state = ref<any>(null)
const status = ref<NrcsBlockchainStatus | null>(null)
const tps = ref('0')

const blocksChartRef = ref<HTMLElement>()
const txChartRef = ref<HTMLElement>()
const difficultyChartRef = ref<HTMLElement>()

let blocksChart: echarts.ECharts | null = null
let txChart: echarts.ECharts | null = null
let difficultyChart: echarts.ECharts | null = null

const blocksData = ref<{ time: string; value: number }[]>([])
const txData = ref<{ time: string; value: number }[]>([])
const difficultyData = ref<{ time: string; value: string }[]>([])

let prevBlockCount = 0
let prevTxCount = 0

async function fetchData() {
  try {
    const [stateData, blockchainStatus] = await Promise.all([
      nrcsApi.getState(true),
      nrcsApi.getBlockchainStatus()
    ])
    state.value = stateData
    status.value = blockchainStatus

    // Calculate TPS (rough estimate from state)
    tps.value = stateData?.tps ?? '0'

    // Record data points
    const now = new Date().toLocaleTimeString()

    const currentBlocks = stateData?.numberOfBlocks || 0
    if (prevBlockCount > 0 && currentBlocks > prevBlockCount) {
      blocksData.value.push({ time: now, value: currentBlocks - prevBlockCount })
    } else {
      blocksData.value.push({ time: now, value: 0 })
    }
    prevBlockCount = currentBlocks

    const currentTxs = stateData?.numberOfTransactions || 0
    if (prevTxCount > 0 && currentTxs > prevTxCount) {
      txData.value.push({ time: now, value: currentTxs - prevTxCount })
    } else {
      txData.value.push({ time: now, value: 0 })
    }
    prevTxCount = currentTxs

    if (blockchainStatus?.cumulativeDifficulty) {
      difficultyData.value.push({ time: now, value: blockchainStatus.cumulativeDifficulty })
    }

    // Trim
    while (blocksData.value.length > MAX_DATA_POINTS) blocksData.value.shift()
    while (txData.value.length > MAX_DATA_POINTS) txData.value.shift()
    while (difficultyData.value.length > MAX_DATA_POINTS) difficultyData.value.shift()

    updateCharts()
  } catch (err) {
    console.error('[NodeMonitor] Failed to fetch data:', err)
  }
}

const memoryUsage = computed(() => {
  if (!state.value) return '-'
  // Estimate memory usage from state data or format available metrics
  const peers = state.value.numberOfPeers || 0
  const blocks = state.value.numberOfBlocks || 0
  const txs = state.value.numberOfTransactions || 0
  const estimatedMB = Math.round((peers * 0.5 + blocks * 0.01 + txs * 0.001))
  return formatVolume(estimatedMB * 1024 * 1024)
})

function createChartOptions(title: string, data: any[], seriesName: string, yAxisName: string, isDifficulty = false) {
  return {
    tooltip: { trigger: 'axis' as const },
    grid: { left: 50, right: 20, top: 20, bottom: 30 },
    xAxis: {
      type: 'category' as const,
      data: data.map(d => d.time),
      axisLabel: { fontSize: 11, rotate: 45 }
    },
    yAxis: {
      type: 'value' as const,
      name: yAxisName,
      axisLabel: isDifficulty
        ? { formatter: (v: number) => (v / 1e9).toFixed(1) + 'G' }
        : { fontSize: 11 }
    },
    series: [{
      name: seriesName,
      data: data.map(d => isDifficulty ? parseFloat(d.value) / 1e9 : d.value),
      type: 'line' as const,
      smooth: true,
      areaStyle: {
        color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
          { offset: 0, color: 'rgba(64, 158, 255, 0.3)' },
          { offset: 1, color: 'rgba(64, 158, 255, 0.05)' }
        ])
      },
      lineStyle: { color: '#409eff', width: 2 },
      itemStyle: { color: '#409eff' }
    }]
  }
}

function initCharts() {
  if (blocksChartRef.value) {
    blocksChart = echarts.init(blocksChartRef.value)
    blocksChart.setOption(createChartOptions('Blocks', [], 'blocks', 'Blocks'))
  }
  if (txChartRef.value) {
    txChart = echarts.init(txChartRef.value)
    txChart.setOption(createChartOptions('Transactions', [], 'tx', 'Transactions'))
  }
  if (difficultyChartRef.value) {
    difficultyChart = echarts.init(difficultyChartRef.value)
    difficultyChart.setOption(createChartOptions('Difficulty', [], 'diff', 'Difficulty (G)', true))
  }
}

function updateCharts() {
  if (blocksChart) {
    blocksChart.setOption(createChartOptions('', blocksData.value, 'blocks/h', 'Blocks'))
  }
  if (txChart) {
    txChart.setOption(createChartOptions('', txData.value, 'tx/h', 'Transactions'))
  }
  if (difficultyChart) {
    difficultyChart.setOption(createChartOptions('', difficultyData.value, 'difficulty', 'Difficulty (G)', true))
  }
}

function handleResize() {
  blocksChart?.resize()
  txChart?.resize()
  difficultyChart?.resize()
}

const { isPolling, lastPollTime, refreshNow } = usePolling(fetchData, POLL_INTERVAL, true)

function formatTime(ts: number): string {
  return formatTimestamp(Math.floor(ts / 1000), false, true)
}

onMounted(() => {
  initCharts()
  window.addEventListener('resize', handleResize)
})

onUnmounted(() => {
  blocksChart?.dispose()
  txChart?.dispose()
  difficultyChart?.dispose()
  window.removeEventListener('resize', handleResize)
})
</script>

<style scoped lang="scss">
.node-monitor-page {
  .metric-card {
    .metric {
      text-align: center;
      padding: 8px;

      .metric-label {
        display: block;
        font-size: 13px;
        color: #909399;
        margin-bottom: 6px;
      }

      .metric-value {
        display: block;
        font-size: 22px;
        font-weight: bold;
        color: #303133;
        font-family: 'Roboto Mono', monospace;
      }
    }
  }

  .chart-container {
    height: 300px;
  }

  .chart-container-large {
    height: 280px;
  }

  .polling-info {
    text-align: center;
    margin-top: 12px;
    font-size: 12px;
    color: #c0c4cc;
  }
}
</style>
