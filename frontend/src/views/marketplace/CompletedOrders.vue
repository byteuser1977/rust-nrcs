<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><CircleCheck /></el-icon> {{ t('marketplace.completedOrders') }}</h2>
      <div class="header-actions">
        <el-button size="small" @click="refreshData"><el-icon><Refresh /></el-icon> {{ t('common.refresh') }}</el-button>
      </div>
    </div>
    <el-card shadow="hover" v-loading="loading">
      <el-table :data="items" stripe size="small" style="width:100%" :empty-text="t('common.noData')">
        <el-table-column prop="name" :label="t('marketplace.product')" min-width="140" />
        <el-table-column :label="t('marketplace.price')" width="110" align="right">
          <template #default="{ row }">{{ formatNrcAmount(row.priceNQT) }}</template>
        </el-table-column>
        <el-table-column prop="quantity" :label="t('marketplace.quantity')" width="80" align="center" />
        <el-table-column :label="t('marketplace.buyer')" min-width="160" show-overflow-tooltip>
          <template #default="{ row }">{{ row.buyerRS }}</template>
        </el-table-column>
        <el-table-column :label="t('marketplace.seller')" min-width="160" show-overflow-tooltip>
          <template #default="{ row }">{{ row.sellerRS }}</template>
        </el-table-column>
        <el-table-column :label="t('common.date')" width="160">
          <template #default="{ row }">{{ formatDate(row.timestamp) }}</template>
        </el-table-column>
        <el-table-column :label="t('common.actions')" width="120" fixed="right">
          <template #default="{ row }">
            <el-button size="small" text type="primary" @click="viewDelivery(row)">
              {{ t('marketplace.deliver') }}
            </el-button>
          </template>
        </el-table-column>
      </el-table>
      <div class="pagination-container" v-if="total > 20">
        <el-pagination v-model:current-page="page" :page-size="20" :total="total" layout="prev,pager,next" @current-change="refreshData" />
      </div>
    </el-card>
    <DGSDeliveryModal v-model:visible="showDelivery" :purchase="selectedPurchase" @success="refreshData" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { CircleCheck, Refresh } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { useAccountStore } from '@/stores/modules/account.store'
import { formatNrc, formatTimestamp } from '@/utils/format'
import DGSDeliveryModal from '@/components/modals/DGSDeliveryModal.vue'

const { t } = useI18n()
const accountStore = useAccountStore()

const loading = ref(false)
const items = ref<any[]>([])
const total = ref(0)
const page = ref(1)
const showDelivery = ref(false)
const selectedPurchase = ref<any>(null)

function formatNrcAmount(nqt: string) { return formatNrc(nqt) + ' NRC' }
function formatDate(ts?: number) { return ts ? formatTimestamp(ts) : '' }

onMounted(() => refreshData())

async function refreshData() {
  loading.value = true
  try {
    const seller = accountStore.accountRS || localStorage.getItem('nrcs_account_rs') || ''
    const result = await nrcsApi.getDGSPurchases(undefined, seller, (page.value - 1) * 20, page.value * 20 - 1, true)
    items.value = (result as any).purchases || []
    total.value = items.value.length
  } catch (e: any) { ElMessage.error(e?.message || t('common.loadError')) }
  finally { loading.value = false }
}

function viewDelivery(row: any) { selectedPurchase.value = row; showDelivery.value = true }
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;
.page-container {
  .page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;
    .page-title { font-size: 18px; font-weight: 600; display: flex; align-items: center; gap: 8px; margin: 0; }
    .header-actions { display: flex; gap: 8px; }
  }
  .pagination-container { display: flex; justify-content: center; margin-top: 16px; }
}
</style>
