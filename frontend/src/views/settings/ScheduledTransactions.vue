<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><AlarmClock /></el-icon> {{ t('settings.scheduledTransactions') }}</h2>
      <el-button type="primary" size="small" @click="refreshData">
        <el-icon><Refresh /></el-icon> {{ t('common.refresh') }}
      </el-button>
    </div>

    <el-card shadow="hover" v-loading="isLoading">
      <el-table :data="scheduledTxs" style="width: 100%" :empty-text="t('common.noData')">
        <el-table-column :label="t('common.transaction')" min-width="200">
          <template #default="{ row }">
            <el-tooltip :content="row.transaction" placement="top">
              <span class="mono-text">{{ truncateHash(row.transaction, 10) }}</span>
            </el-tooltip>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.date')" width="180">
          <template #default="{ row }">
            {{ formatTimestamp(row.timestamp) }}
          </template>
        </el-table-column>
        <el-table-column :label="t('common.type')" width="100">
          <template #default="{ row }">
            <el-tag size="small" type="info">{{ row.type }}.{{ row.subtype }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.amount') + ' (NRC)'" width="130" align="right">
          <template #default="{ row }">
            {{ formatNrc(row.amountNQT) }}
          </template>
        </el-table-column>
        <el-table-column :label="t('common.fee') + ' (NRC)'" width="110" align="right">
          <template #default="{ row }">
            {{ formatNrc(row.feeNQT) }}
          </template>
        </el-table-column>
        <el-table-column :label="t('common.recipient')" min-width="180">
          <template #default="{ row }">
            <span class="mono-text">{{ truncateHash(row.recipientRS || row.recipient, 8) }}</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.actions')" width="100" fixed="right">
          <template #default="{ row }">
            <el-popconfirm
              :title="t('settings.deleteScheduledConfirm')"
              @confirm="deleteTx(row)"
            >
              <template #reference>
                <el-button size="small" type="danger" link>{{ t('common.delete') }}</el-button>
              </template>
            </el-popconfirm>
          </template>
        </el-table-column>
      </el-table>

      <div class="pagination-container" v-if="totalTxs > pageSize">
        <el-pagination
          v-model:current-page="currentPage"
          :page-size="pageSize"
          :total="totalTxs"
          layout="prev, pager, next"
          @current-change="handlePageChange"
        />
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { useAccountStore } from '@/stores/modules/account.store'
import { usePagination } from '@/composables/usePagination'
import { formatTimestamp, formatNrc, truncateHash } from '@/utils/format'
import type { NrcsTransaction } from '@/api/modules/nrcs.api'

const { t } = useI18n()
const accountStore = useAccountStore()

const isLoading = ref(false)
const scheduledTxs = ref<NrcsTransaction[]>([])

const {
  currentPage,
  pageSize,
  total: totalTxs,
  firstIndex,
  lastIndex,
  goToPage,
  setTotal,
} = usePagination(15)

onMounted(() => {
  refreshData()
})

async function refreshData() {
  if (!accountStore.accountRS) return
  isLoading.value = true
  try {
    const result = await nrcsApi.getScheduledTransactions(accountStore.accountRS, firstIndex.value, lastIndex.value)
    scheduledTxs.value = result.transactions || []
    if (result.transactions && result.transactions.length >= pageSize.value) {
      setTotal((currentPage.value + 1) * pageSize.value)
    } else {
      setTotal(firstIndex.value + (result.transactions || []).length)
    }
  } catch (error) {
    console.error('Failed to load scheduled transactions:', error)
  } finally {
    isLoading.value = false
  }
}

async function deleteTx(tx: NrcsTransaction) {
  try {
    if (tx.transaction) {
      await nrcsApi.deleteScheduledTransaction(tx.transaction)
    }
    ElMessage.success(t('common.operationSuccess'))
    await refreshData()
  } catch (e: any) {
    ElMessage.error(e?.description || t('common.operationFailed'))
  }
}

function handlePageChange(page: number) {
  goToPage(page)
  refreshData()
}
</script>

<style scoped lang="scss">
.page-container {
  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
    .page-title {
      font-size: 18px;
      font-weight: 600;
      display: flex;
      align-items: center;
      gap: 8px;
      margin: 0;
    }
  }
  .pagination-container {
    display: flex;
    justify-content: center;
    margin-top: 16px;
  }
}
.mono-text {
  font-family: 'Roboto Mono', monospace;
  font-size: 13px;
  color: #606266;
}
</style>
