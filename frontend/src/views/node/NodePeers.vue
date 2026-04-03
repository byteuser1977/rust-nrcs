<template>
  <div class="node-peers-page">
    <el-card>
      <template #header>
        <div class="card-header">
          <div class="header-left">
            <el-icon><Connection /></el-icon>
            <span>P2P 节点</span>
          </div>
          <div class="header-right">
            <el-button type="primary" @click="fetchPeers">
              <el-icon><Refresh /></el-icon>
              刷新
            </el-button>
          </div>
        </div>
      </template>

      <!-- 统计信息 -->
      <el-row :gutter="20" style="margin-bottom: 20px;">
        <el-col :xs="24" :sm="8" :md="4">
          <div class="stat-box">
            <div class="stat-value">{{ peers.length }}</div>
            <div class="stat-label">连接节点</div>
          </div>
        </el-col>
        <el-col :xs="24" :sm="8" :md="4">
          <div class="stat-box">
            <div class="stat-value">{{ activePeersCount }}</div>
            <div class="stat-label">活跃节点</div>
          </div>
        </el-col>
        <el-col :xs="24" :sm="8" :md="4">
          <div class="stat-box">
            <div class="stat-value">{{ averageLatency.toFixed(0) }}</div>
            <div class="stat-label">平均延迟(ms)</div>
          </div>
        </el-col>
      </el-row>

      <!-- 节点表格 -->
      <el-table :data="peers" style="width: 100%" v-loading="loading">
        <el-table-column prop="node_id" label="节点ID" min-width="120">
          <template #default="{ row }">
            <el-tooltip :content="row.node_id" placement="top">
              <span class="node-id">{{ formatNodeId(row.node_id) }}</span>
            </el-tooltip>
          </template>
        </el-table-column>
        <el-table-column prop="address" label="地址" min-width="160">
          <template #default="{ row }">
            <span class="address-text">{{ row.address }}</span>
          </template>
        </el-table-column>
        <el-table-column prop="country" label="国家/地区" width="120">
          <template #default="{ row }">
            <el-tag size="small">{{ row.country || '未知' }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="latency" label="延迟(ms)" width="120">
          <template #default="{ row }">
            <span :class="getLatencyClass(row.latency)">
              {{ row.latency || '-' }}
            </span>
          </template>
        </el-table-column>
        <el-table-column prop="sync_progress" label="同步进度" width="120">
          <template #default="{ row }">
            <el-progress
              :percentage="row.sync_progress || 100"
              :stroke-width="6"
              :color="getProgressColor(row.sync_progress)"
            />
          </template>
        </el-table-column>
        <el-table-column prop="connected_at" label="连接时间" width="180">
          <template #default="{ row }">
            {{ formatTime(row.connected_at) }}
          </template>
        </el-table-column>
        <el-table-column label="状态" width="100" fixed="right">
          <template #default="{ row }">
            <el-tag
              :type="row.active ? 'success' : 'info'"
              size="small"
              effect="plain"
            >
              {{ row.active ? '活跃' : '离线' }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="120" fixed="right">
          <template #default="{ row }">
            <el-button
              size="small"
              type="danger"
              link
              @click="disconnectPeer(row.node_id)"
              v-if="row.active"
            >
              断开
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Connection, Refresh } from '@element-plus/icons-vue'
import { nodeApi } from '@/api/modules'
import type { PeerInfo } from '@/types/business'
import { formatTime } from '@/utils/format'

const peers = ref<PeerInfo[]>([])
const loading = ref(false)

const activePeersCount = computed(() => peers.value.filter(p => p.active).length)
const averageLatency = computed(() => {
  const activePeers = peers.value.filter(p => p.active && p.latency)
  if (activePeers.length === 0) return 0
  const sum = activePeers.reduce((acc, p) => acc + (p.latency || 0), 0)
  return sum / activePeers.length
})

const fetchPeers = async () => {
  try {
    loading.value = true
    const response = await nodeApi.getPeers()
    peers.value = response.data
  } catch (error: any) {
    console.error('Failed to fetch peers:', error)
    ElMessage.error('获取节点列表失败')
  } finally {
    loading.value = false
  }
}

const disconnectPeer = async (nodeId: string) => {
  try {
    await ElMessageBox.confirm(
      '确定要断开与该节点的连接吗？',
      '警告',
      {
        confirmButtonText: '断开',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )

    await nodeApi.removePeer(nodeId)
    ElMessage.success('已断开连接')
    fetchPeers()
  } catch (error) {
    // 用户取消或操作失败
  }
}

const formatNodeId = (nodeId: string): string => {
  if (nodeId.length <= 16) return nodeId
  return `${nodeId.slice(0, 8)}...${nodeId.slice(-8)}`
}

const formatTime = (dateStr: string): string => {
  return formatTime(dateStr, 'YYYY-MM-DD HH:mm:ss')
}

const getLatencyClass = (latency?: number): string => {
  if (!latency) return 'text-muted'
  if (latency < 100) return 'text-success'
  if (latency < 300) return 'text-warning'
  return 'text-danger'
}

const getProgressColor = (progress?: number): string => {
  if (!progress) return '#67c23a'
  if (progress < 30) return '#f56c6c'
  if (progress < 80) return '#e6a23c'
  return '#67c23a'
}

onMounted(() => {
  fetchPeers()
})
</script>

<style scoped lang="scss">
.node-peers-page {
  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;

    .header-left {
      display: flex;
      align-items: center;
      gap: 8px;
      font-size: 16px;
      font-weight: 600;
    }
  }

  .stat-box {
    text-align: center;
    padding: 20px;
    background: #f5f7fa;
    border-radius: 8px;

    .stat-value {
      font-size: 28px;
      font-weight: bold;
      color: #409eff;
      font-family: 'Roboto Mono', monospace;
    }

    .stat-label {
      font-size: 13px;
      color: #909399;
      margin-top: 4px;
    }
  }

  .node-id {
    font-family: 'Roboto Mono', monospace;
    color: #409eff;
  }

  .address-text {
    font-family: 'Roboto Mono', monospace;
    color: #606266;
  }

  .text-muted {
    color: #909399;
  }

  .text-success {
    color: #67c23a;
  }

  .text-warning {
    color: #e6a23c;
  }

  .text-danger {
    color: #f56c6c;
  }
}
</style>
