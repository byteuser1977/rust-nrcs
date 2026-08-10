<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Sell /></el-icon> {{ t('marketplace.listProduct') }}</h2>
      <div class="header-actions">
        <el-button type="primary" size="small" @click="showList = true"><el-icon><Plus /></el-icon> {{ t('marketplace.listProduct') }}</el-button>
        <el-button size="small" @click="refreshData"><el-icon><Refresh /></el-icon> {{ t('common.refresh') }}</el-button>
      </div>
    </div>
    <el-card shadow="hover" v-loading="loading">
      <el-table :data="items" stripe size="small" style="width:100%" :empty-text="t('common.noData')">
        <el-table-column prop="name" :label="t('common.name')" min-width="140" />
        <el-table-column prop="description" :label="t('common.description')" min-width="180" show-overflow-tooltip />
        <el-table-column :label="t('marketplace.price')" width="110" align="right">
          <template #default="{ row }">{{ formatNrcAmount(row.priceNQT) }}</template>
        </el-table-column>
        <el-table-column prop="quantity" :label="t('marketplace.quantity')" width="80" align="center" />
        <el-table-column :label="t('marketplace.tags')" width="140">
          <template #default="{ row }">
            <el-tag v-for="tag in (row.tags || '').split(',')" :key="tag" size="small" class="tag-item" v-show="tag">{{ tag }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.date')" width="150">
          <template #default="{ row }">{{ formatDate(row.timestamp) }}</template>
        </el-table-column>
        <el-table-column :label="t('common.actions')" width="180" fixed="right">
          <template #default="{ row }">
            <el-button size="small" text type="primary" @click="changePrice(row)">{{ t('marketplace.changePrice') }}</el-button>
            <el-button size="small" text type="warning" @click="changeQty(row)">{{ t('marketplace.changeQuantity') }}</el-button>
            <el-button size="small" text type="danger" @click="delistProduct(row)">{{ t('marketplace.delete') }}</el-button>
          </template>
        </el-table-column>
      </el-table>
      <div class="pagination-container" v-if="total > 20">
        <el-pagination v-model:current-page="page" :page-size="20" :total="total" layout="prev,pager,next" @current-change="refreshData" />
      </div>
    </el-card>

    <DGSListingModal v-model:visible="showList" @success="refreshData" />
    <DGSPriceChangeModal v-model:visible="showPriceChange" :goods="selectedProduct" @success="refreshData" />
    <DGSQuantityChangeModal v-model:visible="showQtyChange" :goods="selectedProduct" @success="refreshData" />
    <DGSDelistingModal v-model:visible="showDelist" :product="selectedProduct" @success="refreshData" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { Plus, Refresh, Sell } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { useAccountStore } from '@/stores/modules/account.store'
import { formatNrc, formatTimestamp } from '@/utils/format'
import DGSListingModal from '@/components/modals/DGSListingModal.vue'
import DGSPriceChangeModal from '@/components/modals/DGSPriceChangeModal.vue'
import DGSQuantityChangeModal from '@/components/modals/DGSQuantityChangeModal.vue'
import DGSDelistingModal from '@/components/modals/DGSDelistingModal.vue'

const { t } = useI18n()
const accountStore = useAccountStore()

const loading = ref(false)
const items = ref<any[]>([])
const total = ref(0)
const page = ref(1)
const showList = ref(false)
const showPriceChange = ref(false)
const showQtyChange = ref(false)
const showDelist = ref(false)
const selectedProduct = ref<any>(null)

function formatNrcAmount(nqt: string) { return formatNrc(nqt) + ' NRC' }
function formatDate(ts?: number) { return ts ? formatTimestamp(ts) : '' }

onMounted(() => refreshData())

async function refreshData() {
  loading.value = true
  try {
    const seller = accountStore.accountRS || localStorage.getItem('nrcs_account_rs') || ''
    const result = await nrcsApi.getDGSGoods(seller, (page.value - 1) * 20, page.value * 20 - 1)
    items.value = (result as any).goods || []
    total.value = items.value.length
  } catch (e: any) { ElMessage.error(e?.message || t('common.loadError')) }
  finally { loading.value = false }
}

function changePrice(row: any) { selectedProduct.value = row; showPriceChange.value = true }
function changeQty(row: any) { selectedProduct.value = row; showQtyChange.value = true }
function delistProduct(row: any) { selectedProduct.value = row; showDelist.value = true }
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;
.page-container {
  .page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;
    .page-title { font-size: 18px; font-weight: 600; display: flex; align-items: center; gap: 8px; margin: 0; }
    .header-actions { display: flex; gap: 8px; }
  }
  .pagination-container { display: flex; justify-content: center; margin-top: 16px; }
  .tag-item { margin-right: 4px; margin-bottom: 2px; }
}
</style>
