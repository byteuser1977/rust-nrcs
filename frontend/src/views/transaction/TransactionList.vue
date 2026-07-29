<template>
  <div class="transaction-list-page">
    <el-card shadow="never">
      <template #header>
        <div class="card-header">
          <div class="header-left">
            <span>{{ t('transaction.history') }}</span>
          </div>
          <div class="header-right">
            <el-input
              v-model="searchQuery"
              :placeholder="t('transaction.searchByAccountOrBlock')"
              style="width: 320px;"
              size="small"
              clearable
              @keyup.enter="handleSearch"
            >
              <template #append>
                <el-button @click="handleSearch">
                  <el-icon><Search /></el-icon>
                </el-button>
              </template>
            </el-input>
            <el-button
              size="small"
              text
              type="primary"
              style="margin-left: 8px;"
              @click="fetchTransactions"
            >
              <el-icon><Refresh /></el-icon>
              {{ t('common.refresh') }}
            </el-button>
          </div>
        </div>
      </template>

      <el-table
        :data="transactions"
        style="width: 100%"
        v-loading="loading"
        @row-click="goToDetail"
        highlight-current-row
      >
        <el-table-column :label="t('transaction.id')" width="160">
          <template #default="{ row }">
            <el-tooltip :content="row.transaction || row.signatureHash" placement="top">
              <span class="mono-text link-text">
                {{ truncateHash(row.transaction || row.signatureHash, 8) }}
              </span>
            </el-tooltip>
          </template>
        </el-table-column>

        <el-table-column :label="t('transaction.date')" width="170">
          <template #default="{ row }">
            {{ formatBlockTime(row.timestamp) }}
          </template>
        </el-table-column>

        <el-table-column :label="t('transaction.type')" width="130">
          <template #default="{ row }">
            <el-tag size="small" type="info">
              {{ getSubTypeName(row.type, row.subtype) }}
            </el-tag>
          </template>
        </el-table-column>

        <el-table-column :label="t('transaction.amount')" width="150" align="right">
          <template #default="{ row }">
            <span :class="parseFloat(row.amountNQT || '0') > 0 ? 'amount-positive' : 'amount-zero'">
              {{ formatNrc(row.amountNQT) }} NRC
            </span>
          </template>
        </el-table-column>

        <el-table-column :label="t('transaction.fee')" width="120" align="right">
          <template #default="{ row }">
            {{ formatNrc(row.feeNQT) }}
          </template>
        </el-table-column>

        <el-table-column :label="t('transaction.sender')" width="180">
          <template #default="{ row }">
            <el-tooltip :content="row.senderRS || row.sender" placement="top">
              <span class="mono-text">{{ truncateHash(row.senderRS || row.sender, 8) }}</span>
            </el-tooltip>
          </template>
        </el-table-column>

        <el-table-column :label="t('transaction.recipient')" width="180">
          <template #default="{ row }">
            <el-tooltip v-if="row.recipientRS || row.recipient" :content="row.recipientRS || row.recipient" placement="top">
              <span class="mono-text">{{ truncateHash(row.recipientRS || row.recipient, 8) }}</span>
            </el-tooltip>
            <span v-else class="text-muted">-</span>
          </template>
        </el-table-column>

        <el-table-column :label="t('transaction.height')" width="100" align="center">
          <template #default="{ row }">
            <span class="mono-text">{{ row.height?.toLocaleString() || '-' }}</span>
          </template>
        </el-table-column>

        <el-table-column :label="t('transaction.confirmations')" width="120" align="center">
          <template #default="{ row }">
            <el-tag :type="(row.confirmations || 0) > 0 ? 'success' : 'warning'" size="small">
              {{ row.confirmations || 0 }}
            </el-tag>
          </template>
        </el-table-column>
      </el-table>

      <div v-if="transactions.length === 0 && !loading" class="empty-state">
        <el-empty :description="t('common.noData')" />
      </div>

      <div class="pagination-wrapper" v-if="pagination.total.value > 0">
        <el-pagination
          v-model:current-page="pagination.currentPage.value"
          v-model:page-size="pagination.pageSize.value"
          :page-sizes="[15, 25, 50, 100]"
          :total="pagination.total.value"
          layout="total, sizes, prev, pager, next, jumper"
          @current-change="onPageChange"
          @size-change="onSizeChange"
        />
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { Search, Refresh } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import type { NrcsTransaction } from '@/api/modules/nrcs.api'
import { usePagination } from '@/composables/usePagination'
import { truncateHash, formatNrc, formatTimestamp } from '@/utils/format'
import { getSubTypeName } from '@/constants/transaction-types'

const { t } = useI18n()
const router = useRouter()

const transactions = ref<NrcsTransaction[]>([])
const loading = ref(false)
const searchQuery = ref('')
const filterAccount = ref<string | undefined>(undefined)

const pagination = usePagination(15)

async function fetchTransactions() {
  try {
    loading.value = true
    const firstIdx = pagination.firstIndex.value
    const lastIdx = pagination.lastIndex.value
    const res = await nrcsApi.getBlockchainTransactions(
      filterAccount.value as any,
      firstIdx,
      lastIdx
    )
    transactions.value = res.transactions || []
    if (res.transactions && res.transactions.length < pagination.pageSize.value) {
      pagination.setTotal(firstIdx + (res.transactions?.length || 0))
    } else {
      pagination.setTotal(pagination.total.value + pagination.pageSize.value)
    }
  } catch (err) {
    console.error('[TransactionList] Failed to fetch transactions:', err)
  } finally {
    loading.value = false
  }
}

function handleSearch() {
  const q = searchQuery.value.trim()
  if (!q) {
    filterAccount.value = undefined
    pagination.currentPage.value = 1
    fetchTransactions()
    return
  }
  filterAccount.value = q
  pagination.currentPage.value = 1
  fetchTransactions()
}

function onPageChange() {
  fetchTransactions()
}

function onSizeChange() {
  pagination.currentPage.value = 1
  fetchTransactions()
}

function goToDetail(row: NrcsTransaction) {
  const txId = row.transaction || row.signatureHash
  if (txId) {
    router.push(`/transaction/detail/${txId}`)
  }
}

function formatBlockTime(ts?: number): string {
  if (!ts) return '-'
  return formatTimestamp(ts)
}

fetchTransactions()
</script>

<style scoped lang="scss">
.transaction-list-page {
  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;

    .header-left {
      font-size: 16px;
      font-weight: 600;
    }

    .header-right {
      display: flex;
      align-items: center;
    }
  }

  .mono-text {
    font-family: 'Roboto Mono', monospace;
    font-size: 13px;

    &.link-text {
      color: #409eff;
      cursor: pointer;
    }
  }

  .text-muted {
    color: #c0c4cc;
  }

  .amount-positive {
    color: #67c23a;
    font-family: 'Roboto Mono', monospace;
    font-size: 13px;
  }

  .amount-zero {
    color: #909399;
    font-family: 'Roboto Mono', monospace;
    font-size: 13px;
  }

  .empty-state {
    padding: 40px 0;
  }

  .pagination-wrapper {
    margin-top: 24px;
    display: flex;
    justify-content: flex-end;
  }
}
</style>
