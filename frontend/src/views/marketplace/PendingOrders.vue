<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Clock /></el-icon> {{ t('marketplace.pendingOrders') }}</h2>
      <div class="header-actions">
        <el-button size="small" @click="refreshData"><el-icon><Refresh /></el-icon> {{ t('common.refresh') }}</el-button>
      </div>
    </div>
    <el-card shadow="hover" v-loading="loading">
      <el-table :data="items" stripe style="width:100%" :empty-text="t('common.noData')">
        <el-table-column prop="name" :label="t('marketplace.product')" min-width="160" />
        <el-table-column prop="buyerRS" :label="t('marketplace.buyer')" width="200" show-overflow-tooltip />
        <el-table-column prop="quantity" :label="t('marketplace.quantity')" width="80" align="center" />
        <el-table-column :label="t('marketplace.price')" width="120" align="right">
          <template #default="{ row }">{{ formatNQT(row.priceNQT) }} NRC</template>
        </el-table-column>
        <el-table-column :label="t('common.date')" width="160">
          <template #default="{ row }">{{ formatDate(row.timestamp) }}</template>
        </el-table-column>
        <el-table-column :label="t('common.actions')" width="200" fixed="right">
          <template #default="{ row }">
            <el-button size="small" text type="primary" @click="openDelivery(row)">{{ t('marketplace.deliver') }}</el-button>
            <el-button size="small" text type="warning" @click="openRefund(row)">{{ t('marketplace.refund') }}</el-button>
          </template>
        </el-table-column>
      </el-table>
      <div class="pagination-container" v-if="total > 20">
        <el-pagination v-model:current-page="page" :page-size="20" :total="total" layout="prev,pager,next" @current-change="refreshData" />
      </div>
    </el-card>
    <DGSDeliveryModal v-model:visible="showDelivery" :purchase="selectedOrder" @success="refreshData" />
    <DGSRefundModal v-model:visible="showRefund" :purchase="selectedOrder" @success="refreshData" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'
import DGSDeliveryModal from '@/components/modals/DGSDeliveryModal.vue'
import DGSRefundModal from '@/components/modals/DGSRefundModal.vue'

const { t } = useI18n()
const loading = ref(false)
const items = ref<any[]>([])
const total = ref(0)
const page = ref(1)
const showDelivery = ref(false)
const showRefund = ref(false)
const selectedOrder = ref<any>(null)

function formatNQT(nqt: string) { return (Number(nqt || '0') / 1e8).toFixed(2) }
function formatDate(ts?: number) {
  if (!ts) return ''
  const epoch = new Date(Date.UTC(2013, 10, 24, 12, 0, 0))
  return new Date(epoch.getTime() + ts * 1000).toLocaleString()
}

onMounted(() => refreshData())

async function refreshData() {
  loading.value = true
  try {
    const accountId = localStorage.getItem('nrcs_account_id') || ''
    const result = await nrcsApi.getDGSPendingPurchases(accountId, (page.value - 1) * 20, page.value * 20 - 1)
    items.value = result.purchases || []
    total.value = items.value.length
  } catch (e: any) { ElMessage.error(e?.message || t('common.loadError')) }
  finally { loading.value = false }
}

function openDelivery(row: any) { selectedOrder.value = row; showDelivery.value = true }
function openRefund(row: any) { selectedOrder.value = row; showRefund.value = true }
</script>
<style scoped lang="scss">
@use '@/assets/styles/variables' as *;
.page-container { .page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; .page-title { font-size: 18px; font-weight: 600; display: flex; align-items: center; gap: 8px; margin: 0; } .header-actions { display: flex; gap: 8px; } } .pagination-container { display: flex; justify-content: center; margin-top: 16px; } }
</style>
