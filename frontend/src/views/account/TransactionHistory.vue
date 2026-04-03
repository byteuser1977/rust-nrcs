<template>
  <div class="transaction-history">
    <BaseTable
      :data="transactions"
      :columns="columns"
      :loading="loading"
      :pagination="paginationConfig"
      @pagination-change="handlePageChange"
      @refresh="fetchData"
    >
      <!-- 交易哈希 -->
      <template #column-hash="{ row }">
        <el-tooltip :content="row.hash" placement="top">
          <span class="hash-text">{{ formatHash(row.hash) }}</span>
        </el-tooltip>
      </template>

      <!-- 地址格式化 -->
      <template #column-from="{ row }">
        <el-tooltip :content="row.from" placement="top">
          <span class="address-text">{{ formatAddress(row.from) }}</span>
        </el-tooltip>
      </template>

      <template #column-to="{ row }">
        <el-tooltip v-if="row.to" :content="row.to" placement="top">
          <span class="address-text">{{ formatAddress(row.to) }}</span>
        </el-tooltip>
        <span v-else class="text-muted">(合约创建)</span>
      </template>

      <!-- 金额 -->
      <template #column-value="{ row }">
        <span class="amount-text">{{ formatAmount(row.value) }}</span>
      </template>

      <!-- 状态 -->
      <template #column-status="{ row }">
        <StatusBadge :value="row.status" />
      </template>

      <!-- 时间 -->
      <template #column-created_at="{ row }">
        {{ formatDateTime(row.created_at) }}
      </template>

      <!-- 操作 -->
      <template #actions="{ row }">
        <el-button
          size="small"
          type="primary"
          link
          @click.stop="viewTransactionDetail(row.hash)"
        >
          详情
        </el-button>
      </template>
    </BaseTable>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import BaseTable from '@/components/base/BaseTable.vue'
import StatusBadge from '@/components/base/StatusBadge.vue'
import { transactionApi } from '@/api/modules'
import type { Transaction } from '@/types/business'
import { formatAddress, formatWeiToEth, formatTime } from '@/utils/format'

const props = defineProps<{
  address: string
}>()

const router = useRouter()

const transactions = ref<Transaction[]>([])
const loading = ref(false)
const currentPage = ref(1)
const total = ref(0)
const pageSize = ref(10)

const paginationConfig = computed(() => ({
  currentPage: currentPage.value,
  pageSize: pageSize.value,
  total: total.value,
  pageSizes: [10, 20, 50],
  layout: 'total, sizes, prev, pager, next, jumper'
}))

const columns = [
  { prop: 'hash', label: '交易哈希', minWidth: 180 },
  { prop: 'from', label: '发送方', minWidth: 180 },
  { prop: 'to', label: '接收方', minWidth: 180 },
  { prop: 'value', label: '金额', width: 120 },
  { prop: 'status', label: '状态', width: 100 },
  { prop: 'created_at', label: '时间', width: 180 }
]

const fetchData = async (page: number) => {
  try {
    loading.value = true
    currentPage.value = page

    const params = {
      address: props.address,
      page,
      size: pageSize.value
    }

    const response = await transactionApi.getTransactions(params)
    transactions.value = response.data.list || []
    total.value = response.data.total || 0
  } catch (error: any) {
    console.error('Failed to fetch transactions:', error)
  } finally {
    loading.value = false
  }
}

const handlePageChange = (page: number, size: number) => {
  pageSize.value = size
  fetchData(page)
}

const viewTransactionDetail = (hash: string) => {
  router.push(`/transaction/detail/${hash}`)
}

const formatHash = (hash: string): string => {
  if (hash.length <= 16) return hash
  return `${hash.slice(0, 8)}...${hash.slice(-6)}`
}

const formatAddress = (addr: string, length: number = 6): string => {
  return formatAddress(addr, length)
}

const formatAmount = (value: string): string => {
  const eth = formatWeiToEth(value)
  return parseFloat(eth).toLocaleString('en-US', {
    minimumFractionDigits: 4,
    maximumFractionDigits: 6
  }) + ' NRCS'
}

const formatDateTime = (dateStr: string): string => {
  return formatTime(dateStr, 'YYYY-MM-DD HH:mm:ss')
}

onMounted(() => {
  fetchData(1)
})
</script>

<style scoped lang="scss">
.transaction-history {
  .hash-text {
    font-family: 'Roboto Mono', monospace;
    color: #409eff;
    cursor: pointer;
  }

  .address-text {
    font-family: 'Roboto Mono', monospace;
    color: #606266;
    cursor: pointer;

    &:hover {
      color: #409eff;
    }
  }

  .amount-text {
    font-family: 'Roboto Mono', monospace;
    font-weight: 500;
    color: #303133;
  }

  .text-muted {
    color: #909399;
    font-size: 13px;
  }
}
</style>
