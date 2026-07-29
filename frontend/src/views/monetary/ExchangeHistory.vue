<template>
  <div class="exchange-history-page">
    <div class="page-header">
      <h2 class="page-title"><el-icon><TrendCharts /></el-icon> {{ t('monetary.exchangeHistory') }}</h2>
      <div class="header-actions">
        <el-radio-group v-model="viewMode" size="small" @change="onViewModeChange">
          <el-radio-button value="my">{{ t('dashboard.owned') }}</el-radio-button>
          <el-radio-button value="all">{{ t('dashboard.total') }}</el-radio-button>
        </el-radio-group>
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
        <el-table-column :label="t('common.date')" width="160">
          <template #default="{ row }">{{ formatTimestamp(row.timestamp) }}</template>
        </el-table-column>
        <el-table-column :label="t('monetary.exchangeType')" width="100">
          <template #default="{ row }">
            <el-tag :type="row.isBuy ? 'success' : 'danger'" size="small">
              {{ row.isBuy ? t('monetary.buy') : t('monetary.sell') }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="code" :label="t('monetary.code')" width="80" />
        <el-table-column :label="t('monetary.counterparty')" min-width="160">
          <template #default="{ row }">
            {{ row.isBuy ? row.sellerRS : row.buyerRS }}
          </template>
        </el-table-column>
        <el-table-column :label="t('common.buyer')" min-width="160">
          <template #default="{ row }">{{ row.buyerRS || '-' }}</template>
        </el-table-column>
        <el-table-column :label="t('common.seller')" min-width="160">
          <template #default="{ row }">{{ row.sellerRS || '-' }}</template>
        </el-table-column>
        <el-table-column :label="t('monetary.units')" width="120" align="right">
          <template #default="{ row }">{{ formatQNT(row.units, row.decimals || 0) }}</template>
        </el-table-column>
        <el-table-column :label="t('monetary.rate')" width="120" align="right">
          <template #default="{ row }">{{ formatNrc(row.rateNQT) }} NRC</template>
        </el-table-column>
        <el-table-column :label="t('common.amount')" width="120" align="right">
          <template #default="{ row }">{{ formatExchangeAmount(row.units, row.rateNQT) }} NRC</template>
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
import { TrendCharts, Refresh } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { useAccountStore } from '@/stores/modules/account.store'
import { usePagination } from '@/composables/usePagination'
import { formatTimestamp, nqtToNxt, formatAmount } from '@/utils/format'

const { t } = useI18n()
const accountStore = useAccountStore()

const accountRS = computed(() => accountStore.accountRS)

// --- State ---
const items = ref<any[]>([])
const viewMode = ref<'my' | 'all'>('my')

const pagination = usePagination(15)

// --- Methods ---
function formatNrc(nqt: string): string {
  if (!nqt) return '0'
  try {
    return nqtToNxt(nqt)
  } catch {
    return nqt
  }
}

function formatQNT(qnt: string, decimals: number): string {
  if (!qnt) return '0'
  try {
    return formatAmount(Number(qnt) / Math.pow(10, decimals), false)
  } catch {
    return qnt
  }
}

function formatExchangeAmount(units: string, rateNQT: string): string {
  if (!units || !rateNQT) return '0'
  try {
    const u = Number(units) || 0
    const r = Number(rateNQT) || 0
    return nqtToNxt(String(Math.round(u * r)))
  } catch {
    return '0'
  }
}

async function loadExchanges() {
  pagination.isLoading.value = true
  try {
    const account = viewMode.value === 'my' && accountRS.value ? accountRS.value : undefined
    const result = await nrcsApi.getExchanges(
      undefined,
      account,
      pagination.firstIndex.value,
      pagination.lastIndex.value,
    )
    const exchanges = (result as any).exchanges || []
    items.value = exchanges
    pagination.setTotalFromList(exchanges.length)
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  } finally {
    pagination.isLoading.value = false
  }
}

function onViewModeChange() {
  pagination.reset()
  loadExchanges()
}

function onPageChange() {
  loadExchanges()
}

async function refreshData() {
  await loadExchanges()
}

// --- Lifecycle ---
onMounted(() => {
  loadExchanges()
})

// Reload when page changes (reactive watch on computed values)
watch([() => pagination.currentPage.value, () => pagination.pageSize.value], () => {
  loadExchanges()
})
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;

.exchange-history-page {
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
