<template>
  <div class="trade-history-page">
    <div class="page-header">
      <h2 class="page-title"><el-icon><TrendCharts /></el-icon> {{ t('asset.tradeHistory') }}</h2>
      <div class="header-actions">
        <el-button size="small" @click="refreshData"><el-icon><Refresh /></el-icon> {{ t('common.refresh') }}</el-button>
      </div>
    </div>

    <!-- Optional asset filter -->
    <el-card shadow="hover" class="filter-card">
      <el-row :gutter="16" align="middle">
        <el-col :span="8">
          <el-select
            v-model="selectedAssetFilter"
            :placeholder="t('asset.allAssets')"
            clearable
            filterable
            style="width: 100%"
            @change="onFilterChange"
          >
            <el-option
              v-for="a in knownAssets"
              :key="a.asset"
              :label="a.name || truncateHash(a.asset, 8)"
              :value="a.asset"
            />
          </el-select>
        </el-col>
        <el-col :span="4">
          <el-radio-group v-model="sideFilter" size="small" @change="onFilterChange">
            <el-radio-button value="all">{{ t('dashboard.total') }}</el-radio-button>
            <el-radio-button value="you">{{ t('dashboard.owned') }}</el-radio-button>
          </el-radio-group>
        </el-col>
      </el-row>
    </el-card>

    <el-card shadow="hover" v-loading="loading" class="table-card">
      <el-table
        :data="paginatedItems"
        stripe
        style="width: 100%"
        :empty-text="t('common.noData')"
      >
        <el-table-column :label="t('common.date')" width="160">
          <template #default="{ row }">{{ formatTimestamp(row.timestamp) }}</template>
        </el-table-column>
        <el-table-column :label="t('asset.asset')" width="160">
          <template #default="{ row }">
            {{ row.assetName || truncateHash(row.asset, 8) }}
          </template>
        </el-table-column>
        <el-table-column :label="t('asset.type')" width="80">
          <template #default="{ row }">
            <el-tag
              size="small"
              :type="isMyBuy(row) ? 'success' : 'danger'"
            >
              {{ isMyBuy(row) ? t('asset.buy') : t('asset.sell') }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column :label="t('asset.quantity')" width="140" align="right">
          <template #default="{ row }">{{ formatQNT(row.quantityQNT, row.decimals) }}</template>
        </el-table-column>
        <el-table-column :label="t('asset.price')" width="140" align="right">
          <template #default="{ row }">
            {{ formatOrderPrice(row.priceNQT, row.decimals) }} NRC
          </template>
        </el-table-column>
        <el-table-column :label="t('asset.total')" width="140" align="right">
          <template #default="{ row }">
            {{ calculateOrderTotal(row.quantityQNT, row.priceNQT) }} NRC
          </template>
        </el-table-column>
        <el-table-column :label="t('asset.counterparty')" min-width="180">
          <template #default="{ row }">
            <span :class="{ highlight: isCounterparty(row) }">
              {{ getCounterparty(row) }}
            </span>
          </template>
        </el-table-column>
      </el-table>
      <div class="pagination-container">
        <el-pagination
          v-model:current-page="currentPage"
          :page-size="pageSize"
          :total="trades.length"
          layout="prev, pager, next"
          background
          small
        />
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { TrendCharts, Refresh } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { useAccountStore } from '@/stores/modules/account.store'
import { usePolling } from '@/composables/usePolling'
import {
  qntToQntf,
  formatTimestamp,
  formatOrderPricePerWholeQNT,
  calculateOrderTotalNQT,
  truncateHash,
} from '@/utils/format'

const { t } = useI18n()
const accountStore = useAccountStore()

const accountRS = computed(() => accountStore.accountRS)

// --- State ---
const loading = ref(false)
const trades = ref<any[]>([])
const knownAssets = ref<any[]>([])
const selectedAssetFilter = ref<string | undefined>(undefined)
const sideFilter = ref<'all' | 'you'>('all')
const currentPage = ref(1)
const pageSize = 20

// --- Computed ---
const paginatedItems = computed(() => {
  const start = (currentPage.value - 1) * pageSize
  return trades.value.slice(start, start + pageSize)
})

// --- Methods ---
function formatQNT(qnt: string, decimals: number): string {
  if (!qnt || qnt === '0') return '0'
  try {
    return qntToQntf(qnt, decimals)
  } catch {
    return qnt
  }
}

function formatOrderPrice(price: string, decimals: number): string {
  if (!price || price === '0') return '0'
  try {
    return formatOrderPricePerWholeQNT(price, decimals)
  } catch {
    return price
  }
}

function calculateOrderTotal(quantityQNT: string, priceNQT: string): string {
  if (!quantityQNT || !priceNQT) return '0'
  try {
    return calculateOrderTotalNQT(quantityQNT, priceNQT)
  } catch {
    return '0'
  }
}

function getCounterparty(row: any): string {
  if (!accountRS.value) return row.sellerRS || row.buyerRS || '-'
  return isMyBuy(row) ? row.sellerRS : row.buyerRS
}

function isCounterparty(row: any): boolean {
  return accountRS.value === getCounterparty(row)
}

function isMyBuy(row: any): boolean {
  return row.buyerRS === accountRS.value
}

async function loadTrades() {
  if (!accountRS.value) return
  loading.value = true
  try {
    // Load all trades; apply filters client-side for recent data
    const result = await nrcsApi.getTrades(selectedAssetFilter.value, 0, 199)
    let allTrades = result?.trades || []

    // Filter by user's side
    if (sideFilter.value === 'you') {
      allTrades = allTrades.filter(
        (t: any) => t.buyerRS === accountRS.value || t.sellerRS === accountRS.value,
      )
    }

    // Enrich with asset names
    const assetCache = new Map<string, any>()
    for (const trade of allTrades) {
      const assetId = trade.asset
      if (!assetCache.has(assetId)) {
        try {
          const detail = await nrcsApi.getAsset(assetId)
          assetCache.set(assetId, detail)
        } catch {
          assetCache.set(assetId, null)
        }
      }
      const detail = assetCache.get(assetId)
      if (detail) {
        trade.assetName = detail.name
        trade.decimals = detail.decimals ?? 0
      } else {
        trade.decimals = trade.decimals ?? 0
      }
    }

    trades.value = allTrades
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  } finally {
    loading.value = false
  }
}

async function loadKnownAssets() {
  try {
    const result = await nrcsApi.getAllAssets(0, 199)
    knownAssets.value = result?.assets || []
  } catch {
    knownAssets.value = []
  }
}

function onFilterChange() {
  currentPage.value = 1
  loadTrades()
}

async function refreshData() {
  currentPage.value = 1
  await Promise.all([loadTrades(), loadKnownAssets()])
}

// --- Lifecycle ---
onMounted(() => {
  loadKnownAssets()
  loadTrades()
})

usePolling(refreshData, 30000, false)
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;

.trade-history-page {
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
  .filter-card {
    margin-bottom: 16px;
  }
  .table-card {
    .highlight {
      font-weight: 600;
      color: var(--el-color-primary);
    }
    .pagination-container {
      display: flex;
      justify-content: center;
      margin-top: 16px;
    }
  }
}
</style>
