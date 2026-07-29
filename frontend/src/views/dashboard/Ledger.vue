<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title">
        <el-icon><Notebook /></el-icon>
        {{ t('dashboard.ledger') }}
      </h2>
      <div class="header-actions">
        <el-button size="small" @click="refreshData">
          <el-icon><Refresh /></el-icon>
          {{ t('common.refresh') }}
        </el-button>
      </div>
    </div>

    <el-alert
      type="info"
      :title="t('dashboard.ledgerInfo')"
      :closable="false"
      show-icon
      class="ledger-info-banner"
    />

    <el-card shadow="hover" v-loading="isLoading" class="ledger-card">
      <el-table
        :data="entries"
        stripe
        style="width: 100%"
        :empty-text="t('common.noData')"
        row-key="ledgerId"
      >
        <el-table-column :label="t('common.date')" width="170">
          <template #default="{ row }">
            <span class="text-muted text-sm">{{ formatTimestamp(row.timestamp) }}</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('dashboard.event')" width="140">
          <template #default="{ row }">
            <el-tag size="small" type="info" effect="plain">{{ row.eventType || '-' }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column :label="t('dashboard.change')" width="150" align="right">
          <template #default="{ row }">
            <span
              class="change-value text-mono"
              :class="getChangeClass(row.change)"
            >
              {{ formatChangeAmount(row.change) }}
            </span>
          </template>
        </el-table-column>
        <el-table-column :label="t('dashboard.balance')" width="150" align="right">
          <template #default="{ row }">
            <span class="text-mono">{{ formatAmount(row.balance) }}</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('dashboard.holdingType')" width="120">
          <template #default="{ row }">
            <el-tag size="small" effect="plain" :type="holdingTypeTag(row.holdingType)">
              {{ row.holdingType || 'NRC' }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="ledgerId" :label="t('common.id')" min-width="200" show-overflow-tooltip>
          <template #default="{ row }">
            <span class="text-mono text-sm">{{ row.ledgerId || '-' }}</span>
          </template>
        </el-table-column>
      </el-table>

      <div class="pagination-container">
        <el-pagination
          v-model:current-page="pagination.currentPage.value"
          :page-size="pagination.pageSize.value"
          :total="pagination.total.value"
          :disabled="pagination.isLoading.value"
          layout="total, prev, pager, next"
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
import { Notebook, Refresh } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { usePagination } from '@/composables/usePagination'
import { formatAmount, formatTimestamp } from '@/utils/format'
import { useAccountStore } from '@/stores/modules/account.store'

const { t } = useI18n()
const accountStore = useAccountStore()

const entries = ref<any[]>([])
const pagination = usePagination(20)

const accountRS = ref(
  accountStore.accountRS || localStorage.getItem('nrcs_account_rs') || ''
)

onMounted(() => {
  refreshData()
})

function formatChangeAmount(change: string | null | undefined): string {
  if (!change || change === '0') return '0'
  const formatted = formatAmount(change)
  const num = Number(BigInt(change)) / 100000000
  return (num >= 0 ? '+' : '') + formatted
}

function getChangeClass(change: string | null | undefined): string {
  if (!change || change === '0') return ''
  try {
    const num = Number(BigInt(change))
    if (num > 0) return 'change-positive'
    if (num < 0) return 'change-negative'
    return ''
  } catch {
    return ''
  }
}

function holdingTypeTag(ht: string | undefined): string {
  if (!ht || ht === 'NRCS' || ht === 'NRC') return 'primary'
  if (ht === 'ASSET') return 'success'
  if (ht === 'CURRENCY') return 'warning'
  return 'info'
}

function handlePageChange(page: number) {
  pagination.goToPage(page)
  refreshData()
}

async function refreshData() {
  pagination.isLoading.value = true
  try {
    const acct = accountRS.value
    if (!acct) {
      ElMessage.warning(t('common.noAccount'))
      return
    }
    const result = await nrcsApi.getAccountLedger(
      acct,
      pagination.firstIndex.value,
      pagination.lastIndex.value
    )
    const list = (result as any).entries || []
    entries.value = list
    pagination.setTotalFromList(list.length)
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  } finally {
    pagination.isLoading.value = false
  }
}
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;

.page-container {
  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: $space-lg;
    .page-title {
      font-size: $font-size-lg;
      font-weight: 600;
      display: flex;
      align-items: center;
      gap: $space-sm;
      margin: 0;
      color: $text-primary;
    }
    .header-actions {
      display: flex;
      gap: $space-sm;
    }
  }
}

.ledger-info-banner {
  margin-bottom: $space-lg;
  border-radius: $radius-md;
  background: $info-subtle !important;
  border: 1px solid rgba($info, 0.2) !important;

  :deep(.el-alert__title) {
    font-size: $font-size-sm;
    color: $info;
  }
}

.ledger-card {
  border: 1px solid $border-subtle !important;
  border-radius: $radius-lg !important;
  background: linear-gradient(180deg, rgba($surface-800, 0.8), rgba($surface-900, 0.9)) !important;

  :deep(.el-table) {
    background: transparent !important;
    --el-table-bg-color: transparent;
    --el-table-tr-bg-color: transparent;
    --el-table-header-bg-color: rgba(255, 255, 255, 0.02);
    --el-table-row-hover-bg-color: rgba($primary, 0.06);
    --el-table-border-color: $border-subtle;
    --el-table-text-color: $text-primary;
    --el-table-header-text-color: $text-muted;

    th.el-table__cell {
      background: rgba(255, 255, 255, 0.025) !important;
      font-weight: 600;
      font-size: $font-size-xs;
      text-transform: uppercase;
      border-bottom: 1px solid $border-subtle;
    }

    td.el-table__cell {
      border-bottom: 1px solid rgba($border-default, 0.5);
    }
  }
}

.change-positive {
  color: $success;
  font-weight: 600;
}

.change-negative {
  color: $danger;
  font-weight: 600;
}

.pagination-container {
  display: flex;
  justify-content: center;
  margin-top: $space-lg;
}

.text-muted {
  color: $text-muted;
}

.text-sm {
  font-size: $font-size-sm;
}

.text-mono {
  font-family: $font-mono;
  font-size: $font-size-sm;
}
</style>
