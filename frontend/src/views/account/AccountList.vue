<template>
  <div class="account-list-page">
    <el-card>
      <template #header>
        <div class="card-header">
          <div class="header-left">
            <el-icon><Wallet /></el-icon>
            <span>账户列表</span>
          </div>
          <div class="header-right">
            <el-button type="primary" @click="goToSendTransaction">
              <el-icon><Position /></el-icon>
              发送交易
            </el-button>
          </div>
        </div>
      </template>

      <!-- 搜索和筛选 -->
      <div class="filter-bar">
        <el-form :model="filters" inline>
          <el-form-item label="地址">
            <el-input
              v-model="filters.address"
              placeholder="输入地址前缀"
              clearable
              style="width: 240px"
              @input="handleSearch"
            />
          </el-form-item>

          <el-form-item label="余额范围">
            <el-input
              v-model="filters.minBalance"
              placeholder="最小余额"
              type="number"
              style="width: 140px"
            />
            <span class="separator">-</span>
            <el-input
              v-model="filters.maxBalance"
              placeholder="最大余额"
              type="number"
              style="width: 140px"
            />
          </el-form-item>

          <el-form-item>
            <el-button type="primary" @click="fetchAccounts(1)">
              <el-icon><Search /></el-icon>
              查询
            </el-button>
            <el-button @click="resetFilters">
              <el-icon><Refresh /></el-icon>
              重置
            </el-button>
          </el-form-item>
        </el-form>
      </div>

      <!-- 账户表格 -->
      <BaseTable
        :data="accounts"
        :columns="columns"
        :loading="loading"
        :pagination="pagination"
        :actions="actions"
        @refresh="fetchAccounts(currentPage)"
        @pagination-change="handlePageChange"
        row-key="address"
      >
        <!-- 余额列自定义格式化 -->
        <template #column-balance="{ row }">
          <span class="balance-value">{{ formatBalance(row.balance) }}</span>
        </template>

        <!-- 地址列截断显示 -->
        <template #column-address="{ row }">
          <el-tooltip :content="row.address" placement="top">
            <span class="address-text">{{ formatAddress(row.address) }}</span>
          </el-tooltip>
        </template>

        <!-- 状态列徽章 -->
        <template #column-status="{ row }">
          <StatusBadge :value="row.status" />
        </template>

        <!-- 时间格式化 -->
        <template #column-created_at="{ row }">
          {{ formatDateTime(row.created_at) }}
        </template>

        <!-- 操作列插槽（如果 BaseTable 不支持 actions） -->
        <template #actions="{ row }">
          <el-button
            size="small"
            type="primary"
            link
            @click.stop="viewDetail(row.address)"
          >
            详情
          </el-button>
          <el-button
            size="small"
            type="success"
            link
            @click.stop="sendToAddress(row.address)"
          >
            转账
          </el-button>
        </template>
      </BaseTable>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { Wallet, Position, Search, Refresh } from '@element-plus/icons-vue'
import BaseTable from '@/components/base/BaseTable.vue'
import StatusBadge from '@/components/base/StatusBadge.vue'
import { useAccountStore } from '@/stores/modules/account.store'
import { accountApi } from '@/api/modules'
import type { Account } from '@/types/business'
import { formatAddress, formatWeiToEth, formatTime } from '@/utils/format'

const router = useRouter()
const accountStore = useAccountStore()

// 状态
const accounts = ref<Account[]>([])
const loading = ref(false)
const currentPage = ref(1)
const total = ref(0)

// 筛选条件
const filters = reactive({
  address: '',
  minBalance: '',
  maxBalance: ''
})

// 分页配置
const pagination = computed(() => ({
  currentPage: currentPage.value,
  pageSize: 20,
  total: total.value,
  pageSizes: [10, 20, 50, 100],
  layout: 'total, sizes, prev, pager, next, jumper'
}))

// 表格列定义
const columns = [
  { prop: 'address', label: '地址', minWidth: 200 },
  { prop: 'balance', label: '余额', width: 180, type: 'amount' as const },
  { prop: 'nonce', label: 'Nonce', width: 100 },
  { prop: 'status', label: '状态', width: 100, type: 'status' as const },
  { prop: 'created_at', label: '创建时间', width: 180 }
]

// 操作按钮
const actions = computed(() => [
  {
    prop: 'view',
    label: '详情',
    type: 'primary' as const,
    handler: (row: Account) => viewDetail(row.address)
  },
  {
    prop: 'send',
    label: '转账',
    type: 'success' as const,
    handler: (row: Account) => sendToAddress(row.address)
  }
])

// 获取账户列表
const fetchAccounts = async (page: number) => {
  try {
    loading.value = true
    currentPage.value = page

    const params: any = {
      page,
      size: 20
    }

    if (filters.address) {
      params.address = filters.address
    }
    if (filters.minBalance) {
      params.min_balance = filters.minBalance
    }
    if (filters.maxBalance) {
      params.max_balance = filters.maxBalance
    }

    const response = await accountApi.getAccounts(params)
    accounts.value = response.data.items || response.data.list || []
    total.value = response.data.total || 0
  } catch (error: any) {
    console.error('Failed to fetch accounts:', error)
    ElMessage.error('获取账户列表失败')
  } finally {
    loading.value = false
  }
}

// 搜索处理
const handleSearch = () => {
  fetchAccounts(1)
}

// 重置筛选器
const resetFilters = () => {
  filters.address = ''
  filters.minBalance = ''
  filters.maxBalance = ''
  fetchAccounts(1)
}

// 页面变更
const handlePageChange = (page: number, size: number) => {
  fetchAccounts(page)
}

// 查看详情
const viewDetail = (address: string) => {
  router.push(`/account/detail/${address}`)
}

// 发送到指定地址
const sendToAddress = (to: string) => {
  router.push({
    path: '/transaction/send',
    query: { to }
  })
}

// 格式化辅助方法
const formatBalance = (balance: string | number): string => {
  const wei = typeof balance === 'string' ? balance : String(balance)
  const eth = formatWeiToEth(wei)
  return `${eth} ETH`
}

const formatAddress = (addr: string, length: number = 8): string => {
  return formatAddress(addr, length)
}

const formatDateTime = (dateStr: string): string => {
  return formatTime(dateStr)
}

onMounted(() => {
  fetchAccounts(1)
})
</script>

<style lang="scss" scoped>
.account-list-page {
  padding: 16px 0;

  .filter-bar {
    margin-bottom: 16px;
    padding: 16px;
    background: #fff;
    border-radius: 4px;

    .el-form-item {
      margin-bottom: 12px;
    }

    .separator {
      margin: 0 8px;
      color: #909399;
    }
  }

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

  .balance-value {
    font-family: 'Roboto Mono', monospace;
    font-weight: 500;
    color: #303133;
  }

  .address-text {
    font-family: 'Roboto Mono', monospace;
    cursor: pointer;
    color: #409eff;

    &:hover {
      text-decoration: underline;
    }
  }
}
</style>
