<template>
  <div class="account-detail-page">
    <el-card v-if="loading" v-loading="true" style="min-height: 400px;">
    </el-card>

    <template v-else-if="account">
      <!-- 账户概览 -->
      <el-row :gutter="20" class="overview-section">
        <el-col :xs="24" :sm="24" :md="12" :lg="8" :xl="6">
          <el-card shadow="hover">
            <div class="info-item">
              <span class="label">钱包地址</span>
              <div class="value address-value">
                <el-tooltip :content="account.address" placement="top">
                  <span>{{ formatAddress(account.address) }}</span>
                </el-tooltip>
                <el-button
                  size="small"
                  text
                  @click="copyAddress(account.address)"
                >
                  <el-icon><CopyDocument /></el-icon>
                </el-button>
              </div>
            </div>
          </el-card>
        </el-col>

        <el-col :xs="24" :sm="24" :md="12" :lg="8" :xl="6">
          <el-card shadow="hover">
            <div class="info-item">
              <span class="label">余额</span>
              <div class="value balance-value">
                <span class="amount">{{ formatBalance(account.balance) }}</span>
                <span class="unit">NRCS</span>
              </div>
            </div>
          </el-card>
        </el-col>

        <el-col :xs="24" :sm="24" :md="12" :lg="8" :xl="6">
          <el-card shadow="hover">
            <div class="info-item">
              <span class="label">Nonce</span>
              <div class="value">
                <span class="number">{{ account.nonce }}</span>
              </div>
            </div>
          </el-card>
        </el-col>

        <el-col :xs="24" :sm="24" :md="12" :lg="8" :xl="6">
          <el-card shadow="hover">
            <div class="info-item">
              <span class="label">状态</span>
              <div class="value">
                <StatusBadge :value="account.status" />
              </div>
            </div>
          </el-card>
        </el-col>
      </el-row>

      <!-- 操作按钮 -->
      <div class="action-section">
        <el-button type="primary" @click="goToSendTransaction">
          <el-icon><Position /></el-icon>
          发送交易
        </el-button>
        <el-button @click="fetchAccountData">
          <el-icon><Refresh /></el-icon>
          刷新
        </el-button>
      </div>

      <!-- tabs 内容 -->
      <el-tabs v-model="activeTab" class="detail-tabs" style="margin-top: 20px;">
        <!-- 交易历史 -->
        <el-tab-pane label="交易历史" name="transactions">
          <TransactionHistory :address="account.address" />
        </el-tab-pane>

        <!-- 资产 -->
        <el-tab-pane label="资产" name="assets">
          <TokenBalances :address="account.address" />
        </el-tab-pane>

        <!-- 详情 -->
        <el-tab-pane label="详细信息" name="info">
          <el-descriptions :column="2" border>
            <el-descriptions-item label="地址">
              {{ account.address }}
            </el-descriptions-item>
            <el-descriptions-item label="创建时间">
              {{ formatDateTime(account.created_at) }}
            </el-descriptions-item>
            <el-descriptions-item label="余额">
              {{ formatBalance(account.balance) }} NRCS
            </el-descriptions-item>
            <el-descriptions-item label="Nonce">
              {{ account.nonce }}
            </el-descriptions-item>
            <el-descriptions-item label="交易总数">
              {{ account.transaction_count || 0 }}
            </el-descriptions-item>
            <el-descriptions-item label="状态">
              <StatusBadge :value="account.status" />
            </el-descriptions-item>
            <el-descriptions-item v-if="account.public_key" label="公钥" :span="2">
              <el-text type="info" style="font-family: monospace;">
                {{ account.public_key }}
              </el-text>
            </el-descriptions-item>
          </el-descriptions>
        </el-tab-pane>
      </el-tabs>
    </template>

    <!-- 未找到账户 -->
    <el-empty v-else description="账户不存在或已被删除" />

    <!-- 返回按钮 -->
    <div class="back-section">
      <el-button @click="goBack">
        <el-icon><ArrowLeft /></el-icon>
        返回列表
      </el-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { ElMessage } from 'element-plus'
import {
  CopyDocument,
  Position,
  Refresh,
  ArrowLeft
} from '@element-plus/icons-vue'
import { useAccountStore } from '@/stores/modules/account.store'
import { accountApi } from '@/api/modules'
import StatusBadge from '@/components/base/StatusBadge.vue'
import TransactionHistory from './TransactionHistory.vue'
import TokenBalances from './TokenBalances.vue'
import type { Account } from '@/types/business'
import { formatAddress, formatWeiToEth, formatTime } from '@/utils/format'

const router = useRouter()
const route = useRoute()
const accountStore = useAccountStore()

const activeTab = ref('transactions')
const loading = ref(false)
const account = ref<Account | null>(null)

// 获取地址参数
const address = computed(() => {
  return route.params.address as string
})

// 获取账户详情
const fetchAccountData = async () => {
  if (!address.value) return

  try {
    loading.value = true
    const response = await accountApi.getAccount(address.value)
    account.value = response.data as Account
  } catch (error: any) {
    console.error('Failed to fetch account:', error)
    ElMessage.error('获取账户信息失败')
    account.value = null
  } finally {
    loading.value = false
  }
}

// 复制地址
const copyAddress = async (addr: string) => {
  try {
    await navigator.clipboard.writeText(addr)
    ElMessage.success('地址已复制')
  } catch (error) {
    ElMessage.error('复制失败')
  }
}

// 跳转到发送交易
const goToSendTransaction = () => {
  router.push({
    path: '/transaction/send',
    query: { to: address.value }
  })
}

// 返回列表
const goBack = () => {
  router.back()
}

// 格式化辅助方法
const formatBalance = (balance: string): string => {
  const eth = formatWeiToEth(balance)
  return parseFloat(eth).toLocaleString('en-US', {
    minimumFractionDigits: 2,
    maximumFractionDigits: 6
  })
}

const formatAddress = (addr: string, length: number = 8): string => {
  return formatAddress(addr, length)
}

const formatDateTime = (dateStr: string): string => {
  return formatTime(dateStr)
}

// 监听地址变化
watch(
  () => address.value,
  () => {
    fetchAccountData()
  },
  { immediate: true }
)
</script>

<style lang="scss" scoped>
.account-detail-page {
  padding-bottom: 40px;
}

.overview-section {
  margin-bottom: 16px;

  .el-card {
    .info-item {
      padding: 12px;

      .label {
        display: block;
        font-size: 12px;
        color: #909399;
        margin-bottom: 8px;
      }

      .value {
        display: flex;
        align-items: center;
        gap: 8px;

        .address-value {
          font-family: 'Roboto Mono', monospace;
          color: #409eff;
        }

        .balance-value {
          align-items: baseline;

          .amount {
            font-size: 24px;
            font-weight: bold;
            color: #303133;
          }

          .unit {
            font-size: 14px;
            color: #909399;
            margin-left: 4px;
          }
        }

        .number {
          font-size: 18px;
          font-weight: 600;
          color: #303133;
        }
      }
    }
  }
}

.action-section {
  margin-bottom: 16px;
  display: flex;
  gap: 12px;
}

.detail-tabs {
  :deep(.el-tabs__content) {
    padding: 16px 0;
  }
}

.back-section {
  margin-top: 24px;
  padding-top: 16px;
  border-top: 1px solid #e4e7ed;
}
</style>
