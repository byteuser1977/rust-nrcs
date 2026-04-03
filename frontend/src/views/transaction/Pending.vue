<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useTransactionStore } from '@/stores/modules/transaction.store'
import { ElCard, ElTable, ElTableColumn, ElTag } from 'element-plus'
import { formatAddress, formatTime } from '@/utils/format'

const { t } = useI18n()
const transactionStore = useTransactionStore()

const pendingTxs = computed(() => transactionStore.pendingTransactions)
</script>

<template>
  <div class="pending-transactions">
    <h1 class="page-title">{{ t('transaction.pending') }}</h1>

    <el-card shadow="never">
      <el-table :data="pendingTxs" style="width: 100%">
        <el-table-column prop="hash" :label="t('transaction.hash')" width="180">
          <template #default="{ row }">
            {{ formatAddress(row.hash) }}
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
            {{ formatTime(row.created_at) }}
          </template>
        </el-table-column>

        <el-table-column prop="status" :label="t('transaction.status')" width="100">
          <template #default="{ row }">
            <el-tag type="info" size="small">
              {{ t('transaction.status.pending') }}
            </el-tag>
          </template>
        </el-table-column>
      </el-table>

      <div v-if="pendingTxs.length === 0" class="empty-state">
        {{ t('common.noData') }}
      </div>
    </el-card>
  </div>
</template>

<style lang="scss" scoped>
.pending-transactions {
  .page-title {
    margin-bottom: 24px;
    font-size: 24px;
    font-weight: 600;
  }

  .empty-state {
    text-align: center;
    padding: 40px;
    color: #909399;
  }
}
</style>
