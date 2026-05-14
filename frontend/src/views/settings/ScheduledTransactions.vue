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
        <el-table-column label="Transaction ID" min-width="200">
          <template #default="{ row }">
            <el-tooltip :content="row.transaction" placement="top">
              <span class="mono-text">{{ truncate(row.transaction, 16) }}</span>
            </el-tooltip>
          </template>
        </el-table-column>
        <el-table-column label="Type" width="100">
          <template #default="{ row }">
            {{ row.type }}.{{ row.subtype }}
          </template>
        </el-table-column>
        <el-table-column label="Sender" min-width="180">
          <template #default="{ row }">
            <span class="mono-text">{{ truncate(row.senderRS || row.sender, 14) }}</span>
          </template>
        </el-table-column>
        <el-table-column label="Recipient" min-width="180">
          <template #default="{ row }">
            <span class="mono-text">{{ truncate(row.recipientRS || row.recipient, 14) }}</span>
          </template>
        </el-table-column>
        <el-table-column label="Amount (NRC)" width="120" align="right">
          <template #default="{ row }">
            {{ formatNrcsAmount(row.amountNQT) }}
          </template>
        </el-table-column>
        <el-table-column label="Fee (NRC)" width="110" align="right">
          <template #default="{ row }">
            {{ formatNrcsAmount(row.feeNQT) }}
          </template>
        </el-table-column>
        <el-table-column label="Height" width="90" align="right">
          <template #default="{ row }">
            {{ row.height || '-' }}
          </template>
        </el-table-column>
        <el-table-column label="Timestamp" width="170">
          <template #default="{ row }">
            {{ formatNrcsTime(row.timestamp) }}
          </template>
        </el-table-column>
        <el-table-column label="Actions" width="100" fixed="right">
          <template #default="{ row }">
            <el-popconfirm title="Delete this scheduled transaction?" @confirm="deleteTx(row)">
              <template #reference>
                <el-button size="small" type="danger" link>Delete</el-button>
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
import type { NrcsTransaction } from '@/api/modules/nrcs.api'

const { t } = useI18n()

const isLoading = ref(false)
const scheduledTxs = ref<NrcsTransaction[]>([])
const totalTxs = ref(0)
const currentPage = ref(1)
const pageSize = ref(20)

onMounted(() => { refreshData() })

async function refreshData() {
  isLoading.value = true
  try {
    const account = localStorage.getItem('nrcs_account_rs') || ''
    const firstIndex = (currentPage.value - 1) * pageSize.value
    const lastIndex = firstIndex + pageSize.value - 1
    const result = await nrcsApi.getScheduledTransactions(account, firstIndex, lastIndex)
    scheduledTxs.value = result.transactions || []
    totalTxs.value = scheduledTxs.value.length >= pageSize.value ? (currentPage.value + 1) * pageSize.value : scheduledTxs.value.length
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
    ElMessage.success('Deleted successfully')
    await refreshData()
  } catch (e: any) {
    ElMessage.error(e?.description || 'Failed to delete scheduled transaction')
  }
}

function handlePageChange(page: number) {
  currentPage.value = page
  refreshData()
}

function formatNrcsTime(timestamp?: number): string {
  if (!timestamp) return ''
  const epoch = new Date(Date.UTC(2013, 10, 24, 12, 0, 0))
  return new Date(epoch.getTime() + timestamp * 1000).toLocaleString()
}

function formatNrcsAmount(nqt?: string): string {
  if (!nqt) return '0.00'
  return (Number(BigInt(nqt)) / 100000000).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })
}

function truncate(text: string | undefined, len: number): string {
  if (!text) return '-'
  if (text.length <= len + 4) return text
  return text.slice(0, len) + '...' + text.slice(-4)
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
