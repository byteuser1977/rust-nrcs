<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Gift /></el-icon> {{ t('marketplace.purchased') }}</h2>
      <div class="header-actions">
        <el-button size="small" @click="refreshData">
          <el-icon><Refresh /></el-icon> {{ t('common.refresh') }}
        </el-button>
      </div>
    </div>

    <!-- Filter tabs -->
    <el-card shadow="hover" class="filter-card">
      <el-radio-group v-model="statusFilter" @change="onFilterChange" size="small">
        <el-radio-button value="all">{{ t('marketplace.allPurchases', 'All') }}</el-radio-button>
        <el-radio-button value="pending">{{ t('marketplace.pending', 'Pending') }}</el-radio-button>
        <el-radio-button value="delivered">{{ t('marketplace.delivered', 'Delivered') }}</el-radio-button>
        <el-radio-button value="refunded">{{ t('marketplace.refunded', 'Refunded') }}</el-radio-button>
      </el-radio-group>
    </el-card>

    <el-card shadow="hover" v-loading="loading" style="margin-top: 16px">
      <el-table :data="filteredItems" stripe style="width: 100%" :empty-text="t('common.noData')">
        <el-table-column prop="name" :label="t('marketplace.product')" min-width="180" />
        <el-table-column :label="t('marketplace.price')" width="140" align="right">
          <template #default="{ row }">
            <span class="price-cell">{{ formatNQT(row.priceNQT) }} <span class="currency-label">NRC</span></span>
          </template>
        </el-table-column>
        <el-table-column prop="quantity" :label="t('marketplace.quantity')" width="80" align="center" />
        <el-table-column :label="t('marketplace.seller')" width="200" show-overflow-tooltip>
          <template #default="{ row }">{{ row.sellerRS || truncateHash(row.seller) }}</template>
        </el-table-column>
        <el-table-column :label="t('common.date')" width="160">
          <template #default="{ row }">{{ formatTimestamp(row.timestamp) }}</template>
        </el-table-column>
        <el-table-column :label="t('common.status')" width="110" align="center">
          <template #default="{ row }">
            <el-tag :type="statusTagType(row)" size="small">
              {{ statusLabel(row) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.actions')" width="260" fixed="right">
          <template #default="{ row }">
            <el-button
              v-if="row.pending"
              size="small"
              text
              type="info"
              @click="viewDelivery(row)"
            >
              {{ t('marketplace.viewDelivery', 'View Delivery') }}
            </el-button>
            <el-button
              size="small"
              text
              type="primary"
              @click="openFeedback(row)"
            >
              {{ t('marketplace.feedback') }}
            </el-button>
            <el-button
              v-if="!row._refunded && row.pending"
              size="small"
              text
              type="warning"
              @click="openRefund(row)"
            >
              {{ t('marketplace.refund') }}
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

    <DGSDeliveryModal v-model:visible="showDelivery" :purchase="selectedItem" @success="refreshData" />
    <DGSFeedbackModal v-model:visible="showFeedback" :purchase="selectedItem" @success="refreshData" />
    <DGSRefundModal v-model:visible="showRefund" :purchase="selectedItem" @success="refreshData" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { formatTimestamp, truncateHash } from '@/utils/format'
import { useAccountStore } from '@/stores/modules/account.store'
import DGSDeliveryModal from '@/components/modals/DGSDeliveryModal.vue'
import DGSFeedbackModal from '@/components/modals/DGSFeedbackModal.vue'
import DGSRefundModal from '@/components/modals/DGSRefundModal.vue'

const { t } = useI18n()
const accountStore = useAccountStore()
const loading = ref(false)
const items = ref<any[]>([])
const total = ref(0)
const page = ref(1)
const statusFilter = ref('all')
const showDelivery = ref(false)
const showFeedback = ref(false)
const showRefund = ref(false)
const selectedItem = ref<any>(null)

function formatNQT(nqt: string) {
  if (!nqt) return '0.00'
  return (Number(nqt) / 1e8).toFixed(2)
}

const filteredItems = computed(() => {
  if (statusFilter.value === 'all') return items.value
  if (statusFilter.value === 'pending') return items.value.filter((i) => i.pending && !i._refunded)
  if (statusFilter.value === 'delivered') return items.value.filter((i) => !i.pending && !i._refunded)
  if (statusFilter.value === 'refunded') return items.value.filter((i) => i._refunded)
  return items.value
})

function statusTagType(row: any): 'success' | 'warning' | 'danger' {
  if (row._refunded) return 'danger'
  if (row.pending) return 'warning'
  return 'success'
}

function statusLabel(row: any): string {
  if (row._refunded) return t('marketplace.refunded', 'Refunded')
  if (row.pending) return t('marketplace.pending', 'Pending')
  return t('marketplace.delivered', 'Delivered')
}

onMounted(() => refreshData())

function onFilterChange() {
  page.value = 1
}

async function refreshData() {
  loading.value = true
  try {
    const accountRS = accountStore.accountRS
    if (!accountRS) {
      items.value = []
      total.value = 0
      return
    }

    // Fetch both completed and pending purchases
    const [completedRes, pendingRes] = await Promise.all([
      nrcsApi.getDGSPurchases(accountRS, undefined, (page.value - 1) * 20, page.value * 20 - 1, true).catch(() => null),
      nrcsApi.getDGSPurchases(accountRS, undefined, (page.value - 1) * 20, page.value * 20 - 1, false).catch(() => null),
    ])

    const completed = (completedRes as any)?.purchases || []
    const pending = (pendingRes as any)?.purchases || []

    // Merge and deduplicate by purchase ID
    const seen = new Set<string>()
    const all: any[] = []

    for (const p of [...pending, ...completed]) {
      if (!seen.has(p.purchase)) {
        seen.add(p.purchase)
        all.push({
          ...p,
          _refunded: !!(p as any).refundNQT || !!(p as any).refund_nqt,
          pending: !!(p as any).pending,
        })
      }
    }

    items.value = all
    total.value = items.value.length
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  } finally {
    loading.value = false
  }
}

function viewDelivery(row: any) {
  selectedItem.value = row
  showDelivery.value = true
}

function openFeedback(row: any) {
  selectedItem.value = row
  showFeedback.value = true
}

function openRefund(row: any) {
  selectedItem.value = row
  showRefund.value = true
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
  .pagination-container {
    display: flex;
    justify-content: center;
    margin-top: $space-md;
  }
}

.filter-card {
  margin-bottom: 0;
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
</style>
