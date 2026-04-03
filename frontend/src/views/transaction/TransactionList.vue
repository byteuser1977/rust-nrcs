<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useTransactionStore } from '@/stores/modules/transaction.store'
import { ElCard, ElTable, ElTableColumn, ElTag, ElPagination, ElEmpty } from 'element-plus'
import { formatAddress, formatWeiToEth, formatTime } from '@/utils/format'

const { t } = useI18n()
const transactionStore = useTransactionStore()

const currentPage = ref(1)
const pageSize = ref(10)

onMounted(() => {
  loadTransactions()
})

async function loadTransactions() {
  await transactionStore.fetchTransactions(currentPage.value, pageSize.value)
}

function handlePageChange(page: number) {
  currentPage.value = page
  loadTransactions()
}

function getTxStatusType(status: string) {
  switch (status) {
    case 'success': return 'success'
    case 'failed': return 'danger'
    case 'confirming': return 'warning'
    default: return 'info'
  }
}

function getTxStatusText(status: string) {
  return t(`transaction.status.${status}`) || status
}
</script>

<template>
  <div class="transaction-list">
    <h1 class="page-title">{{ t('transaction.history') }}</h1>

    <el-card shadow="never" v-loading="transactionStore.isLoading">
      <el-table :data="transactionStore.transactions" style="width: 100%">
        <el-table-column prop="hash" :label="t('transaction.hash')" width="180">
          <template #default="{ row }">
            <el-link type="primary" :underline="false" @click="/* 查看详情 */">
              {{ formatAddress(row.hash) }}
            </el-link>
          </template>
        </el-table-column>

        <el-table-column prop="from" :label="t('transaction.from')" width="180">
          <template #default="{ row }">
            {{ formatAddress(row.from) }}
          </template>
        </el-table-column>

        <el-table-column prop="to" :label="t('transaction.to')" width="180">
          <template #default="{ row }">
            {{ row.to ? formatAddress(row.to) : '-' }}
          </template>
        </el-table-column>

        <el-table-column prop="value" :label="t('transaction.value')" width="120">
          <template #default="{ row }">
            {{ formatWeiToEth(row.value) }} ETH
          </template>
        </el-table-column>

        <el-table-column prop="gas_price" :label="t('transaction.gasPrice')" width="120">
          <template #default="{ row }">
            {{ formatWeiToEth(row.gas_price) }}
          </template>
        </el-table-column>

        <el-table-column prop="status" :label="t('transaction.status')" width="100">
          <template #default="{ row }">
            <el-tag :type="getTxStatusType(row.status)" size="small">
              {{ getTxStatusText(row.status) }}
            </el-tag>
          </template>
        </el-table-column>

        <el-table-column prop="created_at" :label="t('transaction.time')" width="180">
          <template #default="{ row }">
            {{ formatTime(row.created_at, { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' }) }}
          </template>
        </el-table-column>

        <el-table-column :label="t('common.operation')" width="100" fixed="right">
          <template #default="{ row }">
            <el-button link type="primary" size="small" @click="/* 查看详情 */">
              {{ t('common.view') || '查看' }}
            </el-button>
          </template>
        </el-table-column>
      </el-table>

      <div v-if="transactionStore.transactions.length > 0" class="pagination-wrapper">
        <el-pagination
          v-model:current-page="currentPage"
          v-model:page-size="pageSize"
          :page-sizes="[10, 20, 50, 100]"
          :total="transactionStore.total"
          layout="total, sizes, prev, pager, next, jumper"
          @current-change="handlePageChange"
          @size-change="loadTransactions"
        />
      </div>

      <el-empty v-if="!transactionStore.isLoading && transactionStore.transactions.length === 0" :description="t('common.noData')" />
    </el-card>
  </div>
</template>

<style lang="scss" scoped>
.transaction-list {
  .page-title {
    margin-bottom: 24px;
    font-size: 24px;
    font-weight: 600;
  }

  .pagination-wrapper {
    margin-top: 24px;
    display: flex;
    justify-content: flex-end;
  }
}
</style>
