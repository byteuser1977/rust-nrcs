<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useAccountStore } from '@/stores/modules/account.store'
import { useTransactionStore } from '@/stores/modules/transaction.store'
import { useContractStore } from '@/stores/modules/contract.store'
import { useNodeStore } from '@/stores/modules/node.store'
import { useUiStore } from '@/stores/modules/ui.store'
import { ElCard, ElRow, ElCol, ElStatistic, ElProgress, ElTable, ElTableColumn } from 'element-plus'
import { formatAddress, formatWeiToEth, formatTime } from '@/utils/format'

const router = useRouter()
const { t } = useI18n()
const accountStore = useAccountStore()
const transactionStore = useTransactionStore()
const contractStore = useContractStore()
const nodeStore = useNodeStore()
const uiStore = useUiStore()

// 初始化数据
onMounted(async () => {
  // 并行加载初始数据
  await Promise.all([
    loadAccountData(),
    loadTransactionData(),
    loadNodeData()
  ])

  // 开始节点数据轮询
  nodeStore.startPolling()
})

onUnmounted(() => {
  // 页面卸载时停止轮询
  nodeStore.stopPolling()
})

async function loadAccountData() {
  if (!accountStore.isLoggedIn) {
    await accountStore.initFromStorage()
    if (accountStore.isLoggedIn) {
      try {
        await accountStore.fetchUserInfo()
      } catch (error) {
        console.log('Not logged in or token expired')
      }
    }
    return
  }

  try {
    await accountStore.fetchUserInfo()
  } catch (error) {
    console.error('Failed to fetch account info', error)
  }
}

async function loadTransactionData() {
  try {
    await transactionStore.fetchTransactions(1, 5) // 仅加载最近5条
  } catch (error) {
    console.error('Failed to fetch transactions', error)
  }
}

async function loadNodeData() {
  try {
    await nodeStore.refreshAll()
  } catch (error) {
    console.error('Failed to fetch node data', error)
  }
}

// 导航方法
function navigateTo(path: string) {
  router.push(path)
}

// 格式化交易状态标签
function getTxStatusType(status: string): 'success' | 'warning' | 'danger' | 'info' {
  switch (status) {
    case 'success': return 'success'
    case 'failed': return 'danger'
    case 'confirming': return 'warning'
    default: return 'info'
  }
}

function getTxStatusText(status: string): string {
  return t(`transaction.status.${status}`) || status
}
</script>

<template>
  <div class="dashboard">
    <h1 class="page-title">{{ t('dashboard.title') }}</h1>

    <!-- 统计卡片 -->
    <el-row :gutter="20" class="stats-row">
      <el-col :xs="24" :sm="12" :lg="6">
        <el-card shadow="hover" class="stat-card">
          <el-statistic :title="t('dashboard.totalBalance')" :value="accountStore.userInfo?.wallet_address ? 'Loading...' : 'N/A'">
            <template #suffix>
              <span class="stat-suffix">ETH</span>
            </template>
          </el-statistic>
        </el-card>
      </el-col>

      <el-col :xs="24" :sm="12" :lg="6">
        <el-card shadow="hover" class="stat-card">
          <el-statistic :title="t('dashboard.transactions')" :value="transactionStore.transactions.length">
            <template #suffix>
              <el-button link type="primary" size="small" @click="navigateTo('/transaction/history')">
                {{ t('dashboard.viewAll') }}
              </el-button>
            </template>
          </el-statistic>
        </el-card>
      </el-col>

      <el-col :xs="24" :sm="12" :lg="6">
        <el-card shadow="hover" class="stat-card">
          <el-statistic :title="t('dashboard.contracts')" :value="contractStore.contractCount">
            <template #suffix>
              <el-button link type="primary" size="small" @click="navigateTo('/contract/list')">
                {{ t('dashboard.viewAll') }}
              </el-button>
            </template>
          </el-statistic>
        </el-card>
      </el-col>

      <el-col :xs="24" :sm="12" :lg="6">
        <el-card shadow="hover" class="stat-card">
          <el-statistic :title="t('dashboard.tps')" :value="nodeStore.tps.toFixed(2)">
            <template #suffix>
              <span class="stat-suffix">/s</span>
            </template>
          </el-statistic>
        </el-card>
      </el-col>
    </el-row>

    <!-- 详细信息区域 -->
    <el-row :gutter="20" class="content-row">
      <!-- 节点状态 -->
      <el-col :xs="24" :lg="12">
        <el-card shadow="hover" class="info-card">
          <template #header>
            <div class="card-header">
              <span>{{ t('dashboard.networkStats') }}</span>
              <el-button link type="primary" size="small" @click="nodeStore.refreshAll">
                {{ t('common.refresh') }}
              </el-button>
            </div>
          </template>

          <el-descriptions :column="2" border>
            <el-descriptions-item :label="t('dashboard.blockHeight')">
              {{ nodeStore.currentBlockNumber.toLocaleString() }}
            </el-descriptions-item>
            <el-descriptions-item :label="t('dashboard.peers')">
              {{ nodeStore.peerCount }}
            </el-descriptions-item>
            <el-descriptions-item :label="t('dashboard.tps')">
              {{ nodeStore.tps.toFixed(2) }}
            </el-descriptions-item>
            <el-descriptions-item :label="t('dashboard.difficulty')">
              {{ nodeStore.networkDifficulty }}
            </el-descriptions-item>
            <el-descriptions-item :label="t('dashboard.health')" :span="2">
              <el-tag :color="nodeStore.getHealthColor()" effect="dark">
                {{ nodeStore.getHealthText() }}
              </el-tag>
            </el-descriptions-item>
            <el-descriptions-item :label="t('dashboard.uptime')" :span="2">
              {{ nodeStore.formattedUptime }}
            </el-descriptions-item>
          </el-descriptions>
        </el-card>
      </el-col>

      <!-- 最近交易 -->
      <el-col :xs="24" :lg="12">
        <el-card shadow="hover" class="info-card">
          <template #header>
            <div class="card-header">
              <span>{{ t('dashboard.recentTxs') }}</span>
              <el-button link type="primary" size="small" @click="navigateTo('/transaction/history')">
                {{ t('dashboard.viewAll') }}
              </el-button>
            </div>
          </template>

          <el-table :data="transactionStore.transactions" style="width: 100%" max-height="300" v-loading="transactionStore.isLoading">
            <el-table-column prop="hash" :label="t('transaction.hash')" width="120">
              <template #default="{ row }">
                {{ formatAddress(row.hash) }}
              </template>
            </el-table-column>
            <el-table-column prop="from" :label="t('transaction.from')" width="120">
              <template #default="{ row }">
                {{ formatAddress(row.from) }}
              </template>
            </el-table-column>
            <el-table-column prop="to" :label="t('transaction.to')" width="120">
              <template #default="{ row }">
                {{ row.to ? formatAddress(row.to) : '-' }}
              </template>
            </el-table-column>
            <el-table-column prop="value" :label="t('transaction.value')" width="100">
              <template #default="{ row }">
                {{ formatWeiToEth(row.value) }}
              </template>
            </el-table-column>
            <el-table-column prop="status" :label="t('transaction.status')" width="80">
              <template #default="{ row }">
                <el-tag :type="getTxStatusType(row.status)" size="small">
                  {{ getTxStatusText(row.status) }}
                </el-tag>
              </template>
            </el-table-column>
          </el-table>

          <div v-if="transactionStore.transactions.length === 0" class="empty-state">
            {{ t('common.noData') }}
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 快速操作 -->
    <el-row :gutter="20" class="action-row">
      <el-col :span="6">
        <el-card shadow="hover" class="action-card" @click="navigateTo('/transaction/send')">
          <div class="action-icon">
            <el-icon :size="32"><Position /></el-icon>
          </div>
          <div class="action-title">{{ t('transaction.send') }}</div>
        </el-card>
      </el-col>

      <el-col :span="6">
        <el-card shadow="hover" class="action-card" @click="navigateTo('/contract/deploy')">
          <div class="action-icon">
            <el-icon :size="32"><Files /></el-icon>
          </div>
          <div class="action-title">{{ t('contract.deploy') }}</div>
        </el-card>
      </el-col>

      <el-col :span="6">
        <el-card shadow="hover" class="action-card" @click="navigateTo('/contract/list')">
          <div class="action-icon">
            <el-icon :size="32"><Document /></el-icon>
          </div>
          <div class="action-title">{{ t('contract.list') }}</div>
        </el-card>
      </el-col>

      <el-col :span="6">
        <el-card shadow="hover" class="action-card" @click="navigateTo('/node/status')">
          <div class="action-icon">
            <el-icon :size="32"><Monitor /></el-icon>
          </div>
          <div class="action-title">{{ t('node.status') }}</div>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<style lang="scss" scoped>
.dashboard {
  .page-title {
    margin-bottom: 24px;
    font-size: 24px;
    font-weight: 600;
  }

  .stats-row {
    margin-bottom: 24px;
  }

  .stat-card {
    .stat-suffix {
      font-size: 14px;
      opacity: 0.7;
    }
  }

  .content-row {
    margin-bottom: 24px;
  }

  .info-card {
    .card-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
    }
  }

  .empty-state {
    text-align: center;
    padding: 40px 0;
    color: #909399;
  }

  .action-card {
    text-align: center;
    cursor: pointer;
    transition: transform 0.2s, box-shadow 0.2s;

    &:hover {
      transform: translateY(-4px);
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    }

    .action-icon {
      margin: 16px 0;
      color: #409eff;
    }

    .action-title {
      font-size: 14px;
      font-weight: 500;
    }
  }
}
</style>
