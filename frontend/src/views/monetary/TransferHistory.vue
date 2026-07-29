<template>
  <div class="transfer-history-page">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Connection /></el-icon> {{ t('monetary.transferHistory') }}</h2>
      <div class="header-actions">
        <el-button size="small" @click="refreshData"><el-icon><Refresh /></el-icon> {{ t('common.refresh') }}</el-button>
      </div>
    </div>

    <el-card shadow="hover" v-loading="pagination.isLoading.value">
      <el-table
        :data="items"
        stripe
        size="small"
        :empty-text="t('common.noData')"
      >
        <el-table-column :label="t('common.transaction')" width="160">
          <template #default="{ row }">
            <span v-if="row.transaction">{{ truncateHash(row.transaction, 6) }}</span>
            <span v-else>-</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.date')" width="160">
          <template #default="{ row }">{{ formatTimestamp(row.timestamp) }}</template>
        </el-table-column>
        <el-table-column :label="t('monetary.code')" width="80">
          <template #default="{ row }">{{ row.code || '-' }}</template>
        </el-table-column>
        <el-table-column :label="t('monetary.units')" width="120" align="right">
          <template #default="{ row }">{{ formatQNT(row.units, row.decimals || 0) }}</template>
        </el-table-column>
        <el-table-column prop="senderRS" :label="t('common.sender')" min-width="160" />
        <el-table-column prop="recipientRS" :label="t('common.recipient')" min-width="160" />
        <el-table-column :label="t('common.height')" width="90" align="right">
          <template #default="{ row }">{{ row.height || '-' }}</template>
        </el-table-column>
      </el-table>

      <div class="pagination-container" v-if="pagination.total.value > 0">
        <el-pagination
          v-model:current-page="pagination.currentPage.value"
          :page-size="pagination.pageSize.value"
          :total="pagination.total.value"
          layout="prev, pager, next"
          @current-change="onPageChange"
        />
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { Connection, Refresh } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { useAccountStore } from '@/stores/modules/account.store'
import { usePagination } from '@/composables/usePagination'
import { formatTimestamp, formatAmount, truncateHash } from '@/utils/format'

const { t } = useI18n()
const accountStore = useAccountStore()

const accountRS = computed(() => accountStore.accountRS)

// --- State ---
const items = ref<any[]>([])

const pagination = usePagination(15)

// --- Methods ---
function formatQNT(qnt: string, decimals: number): string {
  if (!qnt) return '0'
  try {
    return formatAmount(Number(qnt) / Math.pow(10, decimals), false)
  } catch {
    return qnt
  }
}

async function loadTransfers() {
  pagination.isLoading.value = true
  try {
    const result = await nrcsApi.getCurrencyTransfers(
      undefined,
      accountRS.value || undefined,
      pagination.firstIndex.value,
      pagination.lastIndex.value,
    )
    const transfers = (result as any).transfers || []
    items.value = transfers
    pagination.setTotalFromList(transfers.length)
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  } finally {
    pagination.isLoading.value = false
  }
}

function onPageChange() {
  loadTransfers()
}

async function refreshData() {
  await loadTransfers()
}

// --- Lifecycle ---
onMounted(() => {
  loadTransfers()
})

watch([() => pagination.currentPage.value, () => pagination.pageSize.value], () => {
  loadTransfers()
})
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;

.transfer-history-page {
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
    .header-actions {
      display: flex;
      gap: 8px;
    }
  }
  .pagination-container {
    display: flex;
    justify-content: center;
    margin-top: 16px;
  }
}
</style>
