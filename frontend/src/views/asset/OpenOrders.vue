<template>
  <div class="open-orders-page">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Document /></el-icon> {{ t('asset.openOrders') }}</h2>
      <div class="header-actions">
        <el-button size="small" @click="refreshData"><el-icon><Refresh /></el-icon> {{ t('common.refresh') }}</el-button>
      </div>
    </div>

    <el-card shadow="hover">
      <el-tabs v-model="activeTab" @tab-change="onTabChange">
        <!-- Bid Orders -->
        <el-tab-pane :label="t('asset.buyOrders')" name="bid">
          <el-table
            :data="paginatedBids"
            stripe
            style="width: 100%"
            v-loading="loadingBids"
            :empty-text="t('common.noData')"
          >
            <el-table-column :label="t('asset.asset')" min-width="140">
              <template #default="{ row }">
                {{ row.assetName || truncateHash(row.asset, 6) }}
              </template>
            </el-table-column>
            <el-table-column :label="t('asset.quantity')" width="140" align="right">
              <template #default="{ row }">
                {{ formatQNT(row.quantityQNT, row.decimals) }}
              </template>
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
            <el-table-column :label="t('common.date')" width="160">
              <template #default="{ row }">{{ formatTimestamp(row.timestamp) }}</template>
            </el-table-column>
            <el-table-column :label="t('common.actions')" width="100" fixed="right">
              <template #default="{ row }">
                <el-button size="small" text type="danger" @click="cancelOrder(row, 'bid')">
                  {{ t('common.cancel') }}
                </el-button>
              </template>
            </el-table-column>
          </el-table>
          <div class="pagination-container">
            <el-pagination
              v-model:current-page="bidPage"
              :page-size="pageSize"
              :total="bidOrders.length"
              layout="prev, pager, next"
              background
              small
            />
          </div>
        </el-tab-pane>

        <!-- Ask Orders -->
        <el-tab-pane :label="t('asset.sellOrders')" name="ask">
          <el-table
            :data="paginatedAsks"
            stripe
            style="width: 100%"
            v-loading="loadingAsks"
            :empty-text="t('common.noData')"
          >
            <el-table-column :label="t('asset.asset')" min-width="140">
              <template #default="{ row }">
                {{ row.assetName || truncateHash(row.asset, 6) }}
              </template>
            </el-table-column>
            <el-table-column :label="t('asset.quantity')" width="140" align="right">
              <template #default="{ row }">
                {{ formatQNT(row.quantityQNT, row.decimals) }}
              </template>
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
            <el-table-column :label="t('common.date')" width="160">
              <template #default="{ row }">{{ formatTimestamp(row.timestamp) }}</template>
            </el-table-column>
            <el-table-column :label="t('common.actions')" width="100" fixed="right">
              <template #default="{ row }">
                <el-button size="small" text type="danger" @click="cancelOrder(row, 'ask')">
                  {{ t('common.cancel') }}
                </el-button>
              </template>
            </el-table-column>
          </el-table>
          <div class="pagination-container">
            <el-pagination
              v-model:current-page="askPage"
              :page-size="pageSize"
              :total="askOrders.length"
              layout="prev, pager, next"
              background
              small
            />
          </div>
        </el-tab-pane>
      </el-tabs>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Document, Refresh } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { useAccountStore } from '@/stores/modules/account.store'
import {
  qntToQntf,
  formatTimestamp,
  formatOrderPricePerWholeQNT,
  calculateOrderTotalNQT,
  truncateHash,
} from '@/utils/format'
import { usePolling } from '@/composables/usePolling'

const { t } = useI18n()
const accountStore = useAccountStore()

const accountRS = computed(() => accountStore.accountRS)
const secretPhrase = computed(() => accountStore.secretPhrase)

// --- State ---
const activeTab = ref('bid')
const loadingBids = ref(false)
const loadingAsks = ref(false)
const bidOrders = ref<any[]>([])
const askOrders = ref<any[]>([])
const bidPage = ref(1)
const askPage = ref(1)
const pageSize = 20

// --- Computed ---
const paginatedBids = computed(() => {
  const start = (bidPage.value - 1) * pageSize
  return bidOrders.value.slice(start, start + pageSize)
})

const paginatedAsks = computed(() => {
  const start = (askPage.value - 1) * pageSize
  return askOrders.value.slice(start, start + pageSize)
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

async function loadBidOrders() {
  if (!accountRS.value) return
  loadingBids.value = true
  try {
    const result = await nrcsApi.getAccountCurrentBidOrders(accountRS.value)
    const orders = result?.bidOrders || []
    // Load asset names
    for (const order of orders) {
      try {
        const asset = await nrcsApi.getAsset(order.asset)
        order.assetName = asset?.name
        order.decimals = asset?.decimals ?? 0
      } catch {
        order.decimals = 0
      }
    }
    bidOrders.value = orders
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  } finally {
    loadingBids.value = false
  }
}

async function loadAskOrders() {
  if (!accountRS.value) return
  loadingAsks.value = true
  try {
    const result = await nrcsApi.getAccountCurrentAskOrders(accountRS.value)
    const orders = result?.askOrders || []
    // Load asset names
    for (const order of orders) {
      try {
        const asset = await nrcsApi.getAsset(order.asset)
        order.assetName = asset?.name
        order.decimals = asset?.decimals ?? 0
      } catch {
        order.decimals = 0
      }
    }
    askOrders.value = orders
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  } finally {
    loadingAsks.value = false
  }
}

async function cancelOrder(row: any, type: 'bid' | 'ask') {
  if (!secretPhrase.value) {
    ElMessage.warning(t('asset.enterSecretPhrase'))
    return
  }

  try {
    await ElMessageBox.confirm(
      t('asset.cancelConfirm'),
      t('common.warning'),
      { type: 'warning' },
    )
  } catch {
    return
  }

  try {
    const params = {
      secretPhrase: secretPhrase.value,
      order: row.order || row.asset,
      feeNQT: '100000000',
      deadline: 1440,
    }

    if (type === 'bid') {
      await nrcsApi.cancelBidOrder(params)
    } else {
      await nrcsApi.cancelAskOrder(params)
    }
    ElMessage.success(t('asset.cancelSuccess'))
    if (type === 'bid') {
      bidPage.value = 1
      await loadBidOrders()
    } else {
      askPage.value = 1
      await loadAskOrders()
    }
  } catch (e: any) {
    ElMessage.error(e?.message || t('asset.cancelError'))
  }
}

async function loadAllOrders() {
  await Promise.all([loadBidOrders(), loadAskOrders()])
}

function onTabChange(tabName: string) {
  if (tabName === 'bid') {
    bidPage.value = 1
  } else {
    askPage.value = 1
  }
}

async function refreshData() {
  bidPage.value = 1
  askPage.value = 1
  await loadAllOrders()
}

// --- Lifecycle ---
onMounted(() => {
  loadAllOrders()
})

usePolling(refreshData, 30000, false)
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;

.open-orders-page {
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
