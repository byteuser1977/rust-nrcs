<template>
  <div class="dashboard-page">
    <el-row :gutter="20">
      <!-- 统计数据卡片 -->
      <el-col :xs="24" :sm="12" :lg="6">
        <div class="stat-card">
          <div class="stat-icon" style="background-color: #ecf5ff;">
            <el-icon :size="32" color="#409eff"><Wallet /></el-icon>
          </div>
          <div class="stat-info">
            <div class="stat-value">{{ stats.balance || '0' }}</div>
            <div class="stat-label">总余额 (NRCS)</div>
          </div>
        </div>
      </el-col>

      <el-col :xs="24" :sm="12" :lg="6">
        <div class="stat-card">
          <div class="stat-icon" style="background-color: #f0f9eb;">
            <el-icon :size="32" color="#67c23a"><Document /></el-icon>
          </div>
          <div class="stat-info">
            <div class="stat-value">{{ stats.transactions || '0' }}</div>
            <div class="stat-label">交易总数</div>
          </div>
        </div>
      </el-col>

      <el-col :xs="24" :sm="12" :lg="6">
        <div class="stat-card">
          <div class="stat-icon" style="background-color: #fdf6ec;">
            <el-icon :size="32" color="#e6a23c"><Box /></el-icon>
          </div>
          <div class="stat-info">
            <div class="stat-value">{{ stats.contracts || '0' }}</div>
            <div class="stat-label">合约数量</div>
          </div>
        </div>
      </el-col>

      <el-col :xs="24" :sm="12" :lg="6">
        <div class="stat-card">
          <div class="stat-icon" style="background-color: #fef0f0;">
            <el-icon :size="32" color="#f56c6c"><Monitor /></el-icon>
          </div>
          <div class="stat-info">
            <div class="stat-value">{{ stats.nodes || '0' }}</div>
            <div class="stat-label">节点数量</div>
          </div>
        </div>
      </el-col>
    </el-row>

    <!-- 主要内容区域 -->
    <el-row :gutter="20" style="margin-top: 20px;">
      <!-- 最近交易 -->
      <el-col :xs="24" :lg="12">
        <el-card>
          <template #header>
            <div class="card-header">
              <span>最近交易</span>
              <el-button type="primary" link @click="$router.push('/transaction/list')">
                查看更多
              </el-button>
            </div>
          </template>

          <el-table :data="recentTransactions" style="width: 100%" v-loading="loading">
            <el-table-column prop="hash" label="交易哈希" width="200">
              <template #default="{ row }">
                <el-text class="hash-truncate" :title="row.hash">{{ row.hash }}</el-text>
              </template>
            </el-table-column>
            <el-table-column prop="from" label="发送方" width="180">
              <template #default="{ row }">
                <el-text class="address-truncate" :title="row.from">{{ row.from }}</el-text>
              </template>
            </el-table-column>
            <el-table-column prop="to" label="接收方" width="180">
              <template #default="{ row }">
                <el-text class="address-truncate" :title="row.to">{{ row.to }}</el-text>
              </template>
            </el-table-column>
            <el-table-column prop="value" label="金额" width="120">
              <template #default="{ row }">
                {{ formatWei(row.value) }} NRCS
              </template>
            </el-table-column>
            <el-table-column prop="status" label="状态" width="80">
              <template #default="{ row }">
                <el-tag :type="getStatusType(row.status)" size="small">
                  {{ row.status }}
                </el-tag>
              </template>
            </el-table-column>
          </el-table>
        </el-card>
      </el-col>

      <!-- 节点状态 -->
      <el-col :xs="24" :lg="12">
        <el-card>
          <template #header>
            <div class="card-header">
              <span>节点状态</span>
              <el-button type="primary" link @click="$router.push('/node/status')">
                查看详情
              </el-button>
            </div>
          </template>

          <div v-loading="loading" class="node-status">
            <div class="status-item">
              <span class="status-label">同步状态:</span>
              <el-tag :type="nodeStatus.syncing ? 'warning' : 'success'" size="small">
                {{ nodeStatus.syncing ? '同步中' : '已同步' }}
              </el-tag>
            </div>

            <div class="status-item">
              <span class="status-label">当前区块:</span>
              <span class="status-value">{{ nodeStatus.currentBlock.toLocaleString() }}</span>
            </div>

            <div class="status-item">
              <span class="status-label">最高区块:</span>
              <span class="status-value">{{ nodeStatus.highestBlock.toLocaleString() }}</span>
            </div>

            <div class="status-item">
              <span class="status-label">连接节点:</span>
              <span class="status-value">{{ nodeStatus.peers }} 个</span>
            </div>

            <el-progress
              v-if="nodeStatus.syncing"
              :percentage="syncProgress"
              :stroke-width="8"
              style="margin-top: 16px;"
            />
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 快速操作 -->
    <el-row :gutter="20" style="margin-top: 20px;">
      <el-col :span="24">
        <el-card>
          <template #header>
            <span>快速操作</span>
          </template>

          <div class="quick-actions">
            <el-button type="primary" @click="$router.push('/transaction/send')">
              <el-icon><Position /></el-icon>
              发送交易
            </el-button>

            <el-button type="success" @click="$router.push('/contract/deploy')">
              <el-icon><Plus /></el-icon>
              部署合约
            </el-button>

            <el-button type="info" @click="$router.push('/contract/list')">
              <el-icon><Files /></el-icon>
              查看合约
            </el-button>

            <el-button @click="$router.push('/node/status')">
              <el-icon><Monitor /></el-icon>
              节点监控
            </el-button>
          </div>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useAccountStore } from '@/stores/account'
// import { transactionApi, nodeApi } from '@/api/modules'

const accountStore = useAccountStore()

const loading = ref(false)

// 统计数据
const stats = ref({
  balance: '0',
  transactions: '0',
  contracts: '0',
  nodes: '0'
})

// 最近交易
const recentTransactions = ref<any[]>([])

// 节点状态
const nodeStatus = ref({
  syncing: false,
  currentBlock: 0,
  highestBlock: 0,
  peers: 0
})

const syncProgress = ref(0)

// 格式化 wei 为 NRCS
const formatWei = (wei: string) => {
  const value = BigInt(wei) / BigInt(10**18)
  return value.toLocaleString()
}

// 获取状态类型
const getStatusType = (status: string) => {
  switch (status) {
    case 'success':
      return 'success'
    case 'failed':
      return 'danger'
    case 'pending':
      return 'warning'
    default:
      return 'info'
  }
}

onMounted(async () => {
  loading.value = true
  try {
    // TODO: 加载数据
    // const [txRes, nodeRes] = await Promise.all([
    //   transactionApi.getTransactions({ page: 1, size: 5 }),
    //   nodeApi.getStatus()
    // ])
    // recentTransactions.value = txRes.data.items
    // nodeStatus.value = nodeRes.data
  } catch (error) {
    console.error('Failed to load dashboard data:', error)
  } finally {
    loading.value = false
  }
})
</script>

<style lang="scss" scoped>
.dashboard-page {
  padding: 16px;
}

.stat-card {
  display: flex;
  align-items: center;
  padding: 20px;
  background: #fff;
  border-radius: 8px;
  box-shadow: 0 2px 12px 0 rgba(0, 0, 0, 0.1);
  margin-bottom: 16px;

  .stat-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 64px;
    height: 64px;
    border-radius: 12px;
    margin-right: 16px;
  }

  .stat-info {
    flex: 1;

    .stat-value {
      font-size: 24px;
      font-weight: bold;
      color: #303133;
      line-height: 1.2;
    }

    .stat-label {
      font-size: 14px;
      color: #909399;
      margin-top: 4px;
    }
  }
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.hash-truncate,
.address-truncate {
  font-family: 'Roboto Mono', monospace;
  font-size: 0.85em;
}

.node-status {
  .status-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 0;
    border-bottom: 1px solid #f0f0f0;

    &:last-child {
      border-bottom: none;
    }

    .status-label {
      color: #606266;
      font-weight: 500;
    }

    .status-value {
      color: #303133;
      font-weight: bold;
    }
  }
}

.quick-actions {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}
</style>
