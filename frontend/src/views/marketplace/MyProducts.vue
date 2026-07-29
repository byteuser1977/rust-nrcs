<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Goods /></el-icon> {{ t('marketplace.myProducts') }}</h2>
      <div class="header-actions">
        <el-button type="primary" size="small" @click="showListing = true">
          <el-icon><Plus /></el-icon> {{ t('marketplace.listProduct') }}
        </el-button>
        <el-button size="small" @click="refreshData">
          <el-icon><Refresh /></el-icon> {{ t('common.refresh') }}
        </el-button>
      </div>
    </div>

    <!-- Stats bar -->
    <el-row :gutter="16" class="stats-row">
      <el-col :span="6">
        <el-card shadow="hover" class="stat-card">
          <div class="stat-value">{{ items.length }}</div>
          <div class="stat-label">{{ t('marketplace.totalProducts', 'Total Products') }}</div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card shadow="hover" class="stat-card">
          <div class="stat-value">{{ activeCount }}</div>
          <div class="stat-label">{{ t('marketplace.active', 'Active') }}</div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card shadow="hover" class="stat-card">
          <div class="stat-value">{{ totalQuantity }}</div>
          <div class="stat-label">{{ t('marketplace.totalQuantity', 'Total Stock') }}</div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card shadow="hover" class="stat-card">
          <div class="stat-value">{{ pendingOrdersCount }}</div>
          <div class="stat-label">{{ t('marketplace.pendingOrders') }}</div>
        </el-card>
      </el-col>
    </el-row>

    <el-card shadow="hover" v-loading="loading">
      <el-table :data="items" stripe style="width: 100%" :empty-text="t('common.noData')">
        <el-table-column prop="name" :label="t('marketplace.product')" min-width="180" />
        <el-table-column prop="description" :label="t('common.description')" min-width="200" show-overflow-tooltip />
        <el-table-column :label="t('marketplace.price')" width="140" align="right">
          <template #default="{ row }">
            <span class="price-cell">{{ formatNQT(row.priceNQT) }} <span class="currency-label">NRC</span></span>
          </template>
        </el-table-column>
        <el-table-column :label="t('marketplace.quantity')" width="100" align="center">
          <template #default="{ row }">
            <el-tag :type="row.quantity > 0 ? 'success' : 'danger'" size="small">{{ row.quantity }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column :label="t('marketplace.tags')" width="180">
          <template #default="{ row }">
            <div class="tags-cell" v-if="row.tags">
              <el-tag
                v-for="t in parseTags(row.tags)"
                :key="t"
                size="small"
                type="info"
                class="tag-tiny"
              >
                {{ t }}
              </el-tag>
            </div>
            <span v-else>-</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('voting.status', 'Status')" width="100" align="center">
          <template #default="{ row }">
            <el-tag :type="row.delisted ? 'info' : (row.quantity <= 0 ? 'warning' : 'success')" size="small">
              {{ row.delisted ? t('marketplace.delisted', 'Delisted') : (row.quantity <= 0 ? t('marketplace.soldOut', 'Sold Out') : t('marketplace.active', 'Active')) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.actions')" width="240" fixed="right">
          <template #default="{ row }">
            <el-button
              size="small"
              text
              type="primary"
              :disabled="row.delisted"
              @click="openPriceChange(row)"
            >
              {{ t('marketplace.changePrice') }}
            </el-button>
            <el-button
              size="small"
              text
              type="warning"
              :disabled="row.delisted"
              @click="openQuantityChange(row)"
            >
              {{ t('marketplace.changeQuantity') }}
            </el-button>
            <el-button
              size="small"
              text
              type="danger"
              :disabled="row.delisted"
              @click="confirmDelist(row)"
            >
              {{ t('marketplace.delete') }}
            </el-button>
          </template>
        </el-table-column>
      </el-table>
      <div class="pagination-container" v-if="total > 20">
        <el-pagination
          v-model:current-page="page"
          :page-size="20"
          :total="total"
          layout="prev,pager,next"
          @current-change="refreshData"
        />
      </div>
    </el-card>

    <DGSListingModal v-model:visible="showListing" @success="refreshData" />
    <DGSPriceChangeModal v-model:visible="showPriceChange" :goods="selectedGoods" @success="refreshData" />
    <DGSQuantityChangeModal v-model:visible="showQuantityChange" :goods="selectedGoods" @success="refreshData" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, ElMessageBox } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { useAccountStore } from '@/stores/modules/account.store'
import DGSListingModal from '@/components/modals/DGSListingModal.vue'
import DGSPriceChangeModal from '@/components/modals/DGSPriceChangeModal.vue'
import DGSQuantityChangeModal from '@/components/modals/DGSQuantityChangeModal.vue'

const { t } = useI18n()
const accountStore = useAccountStore()
const loading = ref(false)
const items = ref<any[]>([])
const total = ref(0)
const page = ref(1)
const showListing = ref(false)
const showPriceChange = ref(false)
const showQuantityChange = ref(false)
const selectedGoods = ref<any>(null)
const pendingOrdersCount = ref(0)

function formatNQT(nqt: string) {
  if (!nqt) return '0.00'
  return (Number(nqt) / 1e8).toFixed(2)
}

function parseTags(tagsStr: string): string[] {
  if (!tagsStr) return []
  return tagsStr.split(',').map((t) => t.trim()).filter(Boolean)
}

const activeCount = computed(() => items.value.filter((i) => !i.delisted && i.quantity > 0).length)
const totalQuantity = computed(() => items.value.reduce((sum, i) => sum + (i.quantity || 0), 0))

onMounted(() => refreshData())

async function refreshData() {
  loading.value = true
  try {
    const accountRS = accountStore.accountRS
    if (!accountRS) {
      items.value = []
      total.value = 0
      return
    }
    const result = await nrcsApi.getDGSGoods(
      accountRS,
      (page.value - 1) * 20,
      page.value * 20 - 1,
      false
    )
    items.value = (result as any).goods || []
    total.value = items.value.length

    // Load pending orders count
    try {
      const pendingRes = await nrcsApi.getDGSPendingPurchases(accountRS, 0, 0)
      pendingOrdersCount.value = (pendingRes as any).purchases?.length || 0
    } catch {
      pendingOrdersCount.value = 0
    }
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  } finally {
    loading.value = false
  }
}

function confirmDelist(row: any) {
  ElMessageBox.confirm(
    t('marketplace.deleteConfirm'),
    t('common.warning'),
    { type: 'warning', confirmButtonText: t('common.confirm'), cancelButtonText: t('common.cancel') }
  ).then(async () => {
    try {
      const phrase = accountStore.secretPhrase
      if (!phrase) {
        ElMessage.warning(t('common.secretPhraseRequired'))
        return
      }
      const deadline = 1440
      await nrcsApi.dgsDelisting({
        secretPhrase: phrase,
        goods: row.goods,
        feeNQT: '100000000',
        deadline,
      })
      ElMessage.success(t('marketplace.deleteSuccess'))
      await refreshData()
    } catch (e: any) {
      ElMessage.error(e?.message || t('marketplace.deleteError'))
    }
  }).catch(() => {})
}

function openPriceChange(row: any) {
  selectedGoods.value = row
  showPriceChange.value = true
}

function openQuantityChange(row: any) {
  selectedGoods.value = row
  showQuantityChange.value = true
}
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;

.page-container {
  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: $space-md;
    .page-title {
      font-size: 18px;
      font-weight: 600;
      display: flex;
      align-items: center;
      gap: $space-sm;
      margin: 0;
    }
    .header-actions {
      display: flex;
      gap: $space-sm;
    }
  }
  .stats-row {
    margin-bottom: $space-md;
  }
  .stat-card {
    text-align: center;
    .stat-value {
      font-size: 24px;
      font-weight: 700;
      color: $primary;
    }
    .stat-label {
      font-size: $font-size-sm;
      color: $text-muted;
      margin-top: $space-xs;
    }
  }
  .pagination-container {
    display: flex;
    justify-content: center;
    margin-top: $space-md;
  }
}

.price-cell {
  font-weight: 600;
  color: $primary;
  .currency-label {
    font-size: $font-size-xs;
    color: $text-muted;
    font-weight: 400;
  }
}

.tags-cell {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  .tag-tiny {
    margin: 0;
  }
}
</style>
