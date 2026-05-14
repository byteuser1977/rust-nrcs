<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Goods /></el-icon> {{ t('marketplace.myProducts') }}</h2>
      <div class="header-actions">
        <el-button type="primary" size="small" @click="showListing = true"><el-icon><Plus /></el-icon> {{ t('marketplace.listProduct') }}</el-button>
        <el-button size="small" @click="refreshData"><el-icon><Refresh /></el-icon> {{ t('common.refresh') }}</el-button>
      </div>
    </div>
    <el-card shadow="hover" v-loading="loading">
      <el-table :data="items" stripe style="width:100%" :empty-text="t('common.noData')">
        <el-table-column prop="name" :label="t('marketplace.product')" min-width="160" />
        <el-table-column prop="description" :label="t('common.description')" min-width="180" show-overflow-tooltip />
        <el-table-column :label="t('marketplace.price')" width="120" align="right">
          <template #default="{ row }">{{ formatNQT(row.priceNQT) }} NRC</template>
        </el-table-column>
        <el-table-column prop="quantity" :label="t('marketplace.quantity')" width="80" align="center" />
        <el-table-column :label="t('marketplace.tags')" width="140">
          <template #default="{ row }">{{ row.tags }}</template>
        </el-table-column>
        <el-table-column :label="t('common.actions')" width="200" fixed="right">
          <template #default="{ row }">
            <el-button size="small" text type="primary" @click="openPriceChange(row)">{{ t('marketplace.changePrice') }}</el-button>
            <el-button size="small" text type="warning" @click="openQuantityChange(row)">{{ t('marketplace.changeQuantity') }}</el-button>
            <el-button size="small" text type="danger" @click="openDelist(row)">{{ t('marketplace.delete') }}</el-button>
          </template>
        </el-table-column>
      </el-table>
      <div class="pagination-container" v-if="total > 20">
        <el-pagination v-model:current-page="page" :page-size="20" :total="total" layout="prev,pager,next" @current-change="refreshData" />
      </div>
    </el-card>
    <DGSListingModal v-model:visible="showListing" @success="refreshData" />
    <DGSPriceChangeModal v-model:visible="showPriceChange" :goods="selectedGoods" @success="refreshData" />
    <DGSQuantityChangeModal v-model:visible="showQuantityChange" :goods="selectedGoods" @success="refreshData" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, ElMessageBox } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'
import DGSListingModal from '@/components/modals/DGSListingModal.vue'
import DGSPriceChangeModal from '@/components/modals/DGSPriceChangeModal.vue'
import DGSQuantityChangeModal from '@/components/modals/DGSQuantityChangeModal.vue'

const { t } = useI18n()
const loading = ref(false)
const items = ref<any[]>([])
const total = ref(0)
const page = ref(1)
const showListing = ref(false)
const showPriceChange = ref(false)
const showQuantityChange = ref(false)
const selectedGoods = ref<any>(null)

function formatNQT(nqt: string) { return (Number(nqt || '0') / 1e8).toFixed(2) }

onMounted(() => refreshData())

async function refreshData() {
  loading.value = true
  try {
    const accountId = localStorage.getItem('nrcs_account_id') || ''
    const result = await nrcsApi.getDGSGoods({ seller: accountId, firstIndex: (page.value - 1) * 20, lastIndex: page.value * 20 - 1, inStockOnly: false } as any)
    items.value = result.goods || []
    total.value = items.value.length
  } catch (e: any) { ElMessage.error(e?.message || t('common.loadError')) }
  finally { loading.value = false }
}

function openDelist(row: any) {
  ElMessageBox.confirm(t('marketplace.deleteConfirm'), t('common.warning'), { type: 'warning' }).then(async () => {
    try {
      const phrase = localStorage.getItem('nrcs_passphrase') || ''
      await nrcsApi.dgsDelisting({ secretPhrase: phrase, goods: row.goods, feeNQT: '100000000', deadline: 1440 })
      ElMessage.success(t('marketplace.deleteSuccess'))
      refreshData()
    } catch (e: any) { ElMessage.error(e?.message || t('marketplace.deleteError')) }
  }).catch(() => {})
}

function openPriceChange(row: any) { selectedGoods.value = row; showPriceChange.value = true }
function openQuantityChange(row: any) { selectedGoods.value = row; showQuantityChange.value = true }
</script>
<style scoped lang="scss">
@use '@/assets/styles/variables' as *;
.page-container { .page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; .page-title { font-size: 18px; font-weight: 600; display: flex; align-items: center; gap: 8px; margin: 0; } .header-actions { display: flex; gap: 8px; } } .pagination-container { display: flex; justify-content: center; margin-top: 16px; } }
</style>
