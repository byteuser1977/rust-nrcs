<template>
  <div class="asset-exchange-page">
    <div class="page-header">
      <h2 class="page-title"><el-icon><TrendCharts /></el-icon> {{ t('asset.exchange') }}</h2>
      <div class="header-actions">
        <el-button type="primary" size="small" @click="showIssueAsset = true">
          <el-icon><Plus /></el-icon> {{ t('asset.issueAsset') }}
        </el-button>
        <el-button size="small" @click="refreshData"><el-icon><Refresh /></el-icon> {{ t('common.refresh') }}</el-button>
      </div>
    </div>

    <!-- Asset search + bookmark sidebar -->
    <el-row :gutter="16">
      <el-col :span="6">
        <el-card shadow="hover" class="sidebar-card">
          <template #header>
            <div class="sidebar-header">
              <span>{{ t('asset.assets') }}</span>
              <el-input v-model="searchQuery" :placeholder="t('alias.searchPlaceholder')" size="small" clearable class="search-input">
                <template #prefix><el-icon><Search /></el-icon></template>
              </el-input>
            </div>
          </template>
          <div class="asset-list" v-loading="loadingAssets">
            <div
              v-for="asset in filteredAssets"
              :key="asset.asset"
              class="asset-item"
              :class="{ active: selectedAsset?.asset === asset.asset }"
              @click="selectAsset(asset)"
            >
              <div class="asset-name">{{ asset.name }}</div>
              <div class="asset-id">{{ truncateHash(asset.asset, 6) }}</div>
            </div>
            <el-empty v-if="filteredAssets.length === 0 && !loadingAssets" :description="t('common.noData')" :image-size="48" />
          </div>
        </el-card>
      </el-col>

      <!-- Main content -->
      <el-col :span="18">
        <!-- Asset detail -->
        <el-card v-if="selectedAsset" shadow="hover" class="detail-card">
          <template #header>
            <div class="detail-header">
              <h3>{{ selectedAsset.name }}</h3>
              <el-tag size="small" type="info">{{ selectedAsset.decimals }} {{ t('asset.decimals') }}</el-tag>
            </div>
          </template>
          <el-descriptions :column="3" border size="small">
            <el-descriptions-item :label="t('common.id')">{{ truncateHash(selectedAsset.asset, 8) }}</el-descriptions-item>
            <el-descriptions-item :label="t('asset.quantity')">{{ formatQNT(selectedAsset.quantityQNT, selectedAsset.decimals) }}</el-descriptions-item>
            <el-descriptions-item :label="t('common.issuer')">{{ selectedAsset.issuerRS }}</el-descriptions-item>
            <el-descriptions-item v-if="yourBalance" :label="t('asset.myAssets')">
              {{ formatQNT(yourBalance.quantityQNT, selectedAsset.decimals) }}
            </el-descriptions-item>
          </el-descriptions>
          <div class="action-buttons">
            <el-button type="primary" size="small" @click="openBuyOrder"><el-icon><ShoppingCart /></el-icon> {{ t('asset.buy') }}</el-button>
            <el-button type="success" size="small" @click="openSellOrder"><el-icon><Sell /></el-icon> {{ t('asset.sell') }}</el-button>
            <el-button size="small" @click="openTransfer"><el-icon><Sort /></el-icon> {{ t('asset.transfer') }}</el-button>
          </div>
        </el-card>

        <!-- Order books -->
        <el-row :gutter="16" v-if="selectedAsset">
          <el-col :span="12">
            <el-card shadow="hover">
              <template #header><h3 class="card-title">{{ t('asset.buyOrders') }}</h3></template>
              <el-table :data="bidOrders" stripe size="small" v-loading="loadingOrders" :empty-text="t('common.noData')">
                <el-table-column :label="t('asset.price')" width="120" align="right">
                  <template #default="{ row }">{{ formatOrderPrice(row.priceNQT, selectedAsset.decimals) }} NRC</template>
                </el-table-column>
                <el-table-column :label="t('asset.quantity')" width="100" align="right">
                  <template #default="{ row }">{{ formatQNT(row.quantityQNT, selectedAsset.decimals) }}</template>
                </el-table-column>
                <el-table-column :label="t('asset.total')" width="120" align="right">
                  <template #default="{ row }">{{ calculateOrderTotal(row.quantityQNT, row.priceNQT) }} NRC</template>
                </el-table-column>
              </el-table>
            </el-card>
          </el-col>
          <el-col :span="12">
            <el-card shadow="hover">
              <template #header><h3 class="card-title">{{ t('asset.sellOrders') }}</h3></template>
              <el-table :data="askOrders" stripe size="small" v-loading="loadingOrders" :empty-text="t('common.noData')">
                <el-table-column :label="t('asset.price')" width="120" align="right">
                  <template #default="{ row }">{{ formatOrderPrice(row.priceNQT, selectedAsset.decimals) }} NRC</template>
                </el-table-column>
                <el-table-column :label="t('asset.quantity')" width="100" align="right">
                  <template #default="{ row }">{{ formatQNT(row.quantityQNT, selectedAsset.decimals) }}</template>
                </el-table-column>
                <el-table-column :label="t('asset.total')" width="120" align="right">
                  <template #default="{ row }">{{ calculateOrderTotal(row.quantityQNT, row.priceNQT) }} NRC</template>
                </el-table-column>
              </el-table>
            </el-card>
          </el-col>
        </el-row>

        <!-- Trade history -->
        <el-card v-if="selectedAsset" shadow="hover" class="mt-16">
          <template #header>
            <div class="section-header">
              <h3 class="card-title">{{ t('asset.tradeHistory') }}</h3>
              <el-radio-group v-model="tradeFilter" size="small" @change="loadTrades">
                <el-radio-button value="everyone">{{ t('dashboard.total') }}</el-radio-button>
                <el-radio-button value="you">{{ t('dashboard.owned') }}</el-radio-button>
              </el-radio-group>
            </div>
          </template>
          <el-table :data="trades" stripe size="small" v-loading="loadingTrades" :empty-text="t('common.noData')">
            <el-table-column :label="t('common.date')" width="160">
              <template #default="{ row }">{{ formatTimestamp(row.timestamp) }}</template>
            </el-table-column>
            <el-table-column :label="t('asset.type')" width="80">
              <template #default="{ row }">
                <el-tag size="small" :type="row.buyerRS === accountRS ? 'success' : 'danger'">
                  {{ row.buyerRS === accountRS ? t('asset.buy') : t('asset.sell') }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column :label="t('asset.quantity')" width="100" align="right">
              <template #default="{ row }">{{ formatQNT(row.quantityQNT, selectedAsset.decimals) }}</template>
            </el-table-column>
            <el-table-column :label="t('asset.price')" width="120" align="right">
              <template #default="{ row }">{{ formatOrderPrice(row.priceNQT, selectedAsset.decimals) }} NRC</template>
            </el-table-column>
            <el-table-column :label="t('common.buyer')" min-width="160">
              <template #default="{ row }">{{ row.buyerRS }}</template>
            </el-table-column>
            <el-table-column :label="t('common.seller')" min-width="160">
              <template #default="{ row }">{{ row.sellerRS }}</template>
            </el-table-column>
          </el-table>
        </el-card>

        <!-- No asset selected -->
        <el-empty v-if="!selectedAsset" :description="t('asset.selectAsset')" :image-size="80" />
      </el-col>
    </el-row>

    <!-- Modals -->
    <AssetOrderModal v-model:visible="showOrderModal" :asset="selectedAsset" :order-type="orderType" @success="onOrderSuccess" />
    <TransferAssetModal v-model:visible="showTransferModal" :asset="selectedAsset" @success="refreshData" />
    <IssueAssetModal v-model:visible="showIssueAsset" @success="loadAssets" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { TrendCharts, Plus, Refresh, Search, ShoppingCart, Sell, Sort } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { useAccountStore } from '@/stores/modules/account.store'
import { usePolling } from '@/composables/usePolling'
import { formatTimestamp, formatOrderPricePerWholeQNT, calculateOrderTotalNQT, qntToQntf, qntfToQnt, truncateHash } from '@/utils/format'
import AssetOrderModal from '@/components/modals/AssetOrderModal.vue'
import TransferAssetModal from '@/components/modals/TransferAssetModal.vue'
import IssueAssetModal from '@/components/modals/IssueAssetModal.vue'

const { t } = useI18n()
const accountStore = useAccountStore()

const accountRS = computed(() => accountStore.accountRS)

// --- State ---
const searchQuery = ref('')
const loadingAssets = ref(false)
const loadingOrders = ref(false)
const loadingTrades = ref(false)
const assets = ref<any[]>([])
const selectedAsset = ref<any>(null)
const yourBalance = ref<any>(null)
const bidOrders = ref<any[]>([])
const askOrders = ref<any[]>([])
const trades = ref<any[]>([])
const tradeFilter = ref('everyone')

// Modals
const showOrderModal = ref(false)
const showTransferModal = ref(false)
const showIssueAsset = ref(false)
const orderType = ref<'ask' | 'bid'>('bid')

// --- Computed ---
const filteredAssets = computed(() => {
  if (!searchQuery.value) return assets.value
  const q = searchQuery.value.toLowerCase()
  return assets.value.filter(
    (a) =>
      a.name?.toLowerCase().includes(q) ||
      a.asset?.toLowerCase().includes(q) ||
      a.issuerRS?.toLowerCase().includes(q),
  )
})

// --- Methods ---
function formatQNT(qnt: string, decimals: number): string {
  try { return qntToQntf(qnt, decimals) } catch { return qnt }
}

function formatOrderPrice(price: string, decimals: number): string {
  try { return formatOrderPricePerWholeQNT(price, decimals) } catch { return price }
}

function calculateOrderTotal(quantityQNT: string, priceNQT: string): string {
  try { return calculateOrderTotalNQT(quantityQNT, priceNQT) } catch { return '0' }
}

async function loadAssets() {
  loadingAssets.value = true
  try {
    const result = await nrcsApi.getAllAssets()
    assets.value = result.assets || []
    // Load account asset balances for holdings info
    if (accountRS.value) {
      const accountAssets = await nrcsApi.getAccountAssets(accountRS.value)
      const balances = accountAssets?.assetBalances || accountAssets?.accountAssets || []
      // Merge balance info into assets
      for (const asset of assets.value) {
        const balance = balances.find((b: any) => b.asset === asset.asset)
        if (balance) {
          asset.yourBalanceQNT = balance.quantityQNT
          asset.yourUnconfirmedBalanceQNT = balance.unconfirmedQuantityQNT
        }
      }
    }
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  } finally {
    loadingAssets.value = false
  }
}

async function selectAsset(asset: any) {
  selectedAsset.value = asset
  yourBalance.value = null
  bidOrders.value = []
  askOrders.value = []
  trades.value = []

  if (!accountRS.value) return

  // Load asset detail
  try {
    const detail = await nrcsApi.getAsset(asset.asset)
    if (detail) selectedAsset.value = { ...asset, ...detail }
  } catch {}

  // Load user's balance for this asset
  try {
    const accountAssets = await nrcsApi.getAccountAssets(accountRS.value)
    const balances = accountAssets?.assetBalances || accountAssets?.accountAssets || []
    yourBalance.value = balances.find((b: any) => b.asset === asset.asset)
  } catch {}

  // Load orders
  loadingOrders.value = true
  try {
    const [bids, asks] = await Promise.all([
      nrcsApi.getBidOrders(asset.asset, 0, 19),
      nrcsApi.getAskOrders(asset.asset, 0, 19),
    ])
    bidOrders.value = bids?.bidOrders || []
    askOrders.value = asks?.askOrders || []
  } catch (e) {
    console.error('Failed to load orders:', e)
  } finally {
    loadingOrders.value = false
  }

  // Load trades
  loadTrades()
}

async function loadTrades() {
  if (!selectedAsset.value) return
  loadingTrades.value = true
  try {
    const result = await nrcsApi.getTrades(selectedAsset.value.asset, 0, 19)
    const allTrades = result?.trades || []
    if (tradeFilter.value === 'you' && accountRS.value) {
      trades.value = allTrades.filter(
        (t: any) => t.buyerRS === accountRS.value || t.sellerRS === accountRS.value,
      )
    } else {
      trades.value = allTrades
    }
  } catch (e) {
    console.error('Failed to load trades:', e)
  } finally {
    loadingTrades.value = false
  }
}

function openBuyOrder() {
  orderType.value = 'bid'
  showOrderModal.value = true
}

function openSellOrder() {
  orderType.value = 'ask'
  showOrderModal.value = true
}

function openTransfer() {
  showTransferModal.value = true
}

function onOrderSuccess() {
  loadTrades()
  loadOrdersForAsset()
}

async function loadOrdersForAsset() {
  if (!selectedAsset.value) return
  try {
    const [bids, asks] = await Promise.all([
      nrcsApi.getBidOrders(selectedAsset.value.asset, 0, 19),
      nrcsApi.getAskOrders(selectedAsset.value.asset, 0, 19),
    ])
    bidOrders.value = bids?.bidOrders || []
    askOrders.value = asks?.askOrders || []
  } catch {}
}

async function refreshData() {
  await loadAssets()
  if (selectedAsset.value) {
    await selectAsset(selectedAsset.value)
  }
}

// --- Lifecycle ---
onMounted(() => {
  loadAssets()
})

usePolling(refreshData, 30000, false)
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;

.asset-exchange-page {
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
  .sidebar-card {
    .sidebar-header {
      display: flex;
      flex-direction: column;
      gap: 8px;
      .search-input { width: 100%; }
    }
    .asset-list {
      max-height: 60vh;
      overflow-y: auto;
      .asset-item {
        padding: 8px 12px;
        cursor: pointer;
        border-radius: 4px;
        margin-bottom: 2px;
        transition: background 0.2s;
        &:hover { background: var(--el-fill-color-light); }
        &.active { background: var(--el-color-primary-light-9); border-left: 3px solid var(--el-color-primary); }
        .asset-name { font-weight: 500; font-size: 13px; }
        .asset-id { font-size: 11px; color: $text-muted; }
      }
    }
  }
  .detail-card {
    margin-bottom: 16px;
    .detail-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      h3 { margin: 0; font-size: 16px; }
    }
    .action-buttons {
      margin-top: 12px;
      display: flex;
      gap: 8px;
    }
  }
  .card-title { margin: 0; font-size: 15px; color: $text-primary; }
  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .mt-16 { margin-top: 16px; }
}
</style>
