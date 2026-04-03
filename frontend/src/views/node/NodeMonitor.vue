<template>
  <div class="node-monitor-page">
    <el-row :gutter="20">
      <el-col :xs="24" :sm="12" :lg="6">
        <el-card>
          <div class="metric-item">
            <div class="metric-label">CPU 使用率</div>
            <el-progress type="circle" :percentage="metrics.cpuUsage" :width="80" />
          </div>
        </el-card>
      </el-col>

      <el-col :xs="24" :sm="12" :lg="6">
        <el-card>
          <div class="metric-item">
            <div class="metric-label">内存使用率</div>
            <el-progress type="circle" :percentage="memoryPercent" :width="80" />
            <div class="metric-detail">{{ formatBytes(metrics.memoryUsage) }} / {{ formatBytes(metrics.memoryTotal) }}</div>
          </div>
        </el-card>
      </el-col>

      <el-col :xs="24" :sm="12" :lg="6">
        <el-card>
          <div class="metric-item">
            <div class="metric-label">磁盘使用率</div>
            <el-progress type="circle" :percentage="diskPercent" :width="80" />
            <div class="metric-detail">{{ formatBytes(metrics.diskUsage) }} / {{ formatBytes(metrics.diskTotal) }}</div>
          </div>
        </el-card>
      </el-col>

      <el-col :xs="24" :sm="12" :lg="6">
        <el-card>
          <div class="metric-item">
            <div class="metric-label">网络流入</div>
            <div class="metric-value">{{ formatBytes(metrics.networkIn) }}</div>
            <div class="metric-detail">流出: {{ formatBytes(metrics.networkOut) }}</div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <el-card style="margin-top: 20px;">
      <template #header>
        <span>节点运行指标</span>
      </template>
      <el-descriptions :column="3" border>
        <el-descriptions-item label="TPS">
          {{ metrics.tps }}
        </el-descriptions-item>
        <el-descriptions-item label="Pending交易数">
          {{ metrics.pendingTxCount }}
        </el-descriptions-item>
        <el-descriptions-item label="运行时间">
          {{ formatUptime(metrics.uptime) }}
        </el-descriptions-item>
      </el-descriptions>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

const metrics = ref({
  cpuUsage: 0,
  memoryUsage: 0,
  memoryTotal: 0,
  diskUsage: 0,
  diskTotal: 0,
  networkIn: 0,
  networkOut: 0,
  pendingTxCount: 0,
  tps: 0,
  uptime: 0
})

const memoryPercent = computed(() => {
  if (metrics.value.memoryTotal === 0) return 0
  return Math.round((metrics.value.memoryUsage / metrics.value.memoryTotal) * 100)
})

const diskPercent = computed(() => {
  if (metrics.value.diskTotal === 0) return 0
  return Math.round((metrics.value.diskUsage / metrics.value.diskTotal) * 100)
})

const formatBytes = (bytes: number): string => {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

const formatUptime = (seconds: number): string => {
  const days = Math.floor(seconds / 86400)
  const hours = Math.floor((seconds % 86400) / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  return `${days}天 ${hours}小时 ${minutes}分钟`
}
</script>

<style lang="scss" scoped>
.metric-item {
  text-align: center;
  padding: 16px;

  .metric-label {
    color: #606266;
    margin-bottom: 12px;
  }

  .metric-value {
    font-size: 20px;
    font-weight: bold;
    color: #303133;
  }

  .metric-detail {
    margin-top: 8px;
    font-size: 12px;
    color: #909399;
  }
}
</style>
