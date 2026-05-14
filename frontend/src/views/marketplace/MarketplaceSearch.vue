<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><ShoppingCart /></el-icon> {{ t('marketplace.title') }}</h2>
      <div class="header-actions">
        <el-button type="primary" size="small" @click="showListing = true"><el-icon><Plus /></el-icon> {{ t('marketplace.listProduct') }}</el-button>
        <el-button size="small" @click="refreshData"><el-icon><Refresh /></el-icon> {{ t('common.refresh') }}</el-button>
      </div>
    </div>
    <el-card shadow="hover" class="mb-4">
      <el-form :inline="true">
        <el-form-item :label="t('marketplace.seller')">
          <el-input v-model="filters.seller" :placeholder="t('marketplace.sellerPlaceholder')" clearable style="width:220px" @change="refreshData" />
        </el-form-item>
        <el-form-item :label="t('marketplace.tag')">
          <el-input v-model="filters.tag" :placeholder="t('marketplace.tagPlaceholder')" clearable style="width:160px" @change="refreshData" />
        </el-form-item>
        <el-form-item>
          <el-checkbox v-model="filters.inStockOnly" @change="refreshData">{{ t('marketplace.inStockOnly') }}</el-checkbox>
        </el-form-item>
      </el-form>
    </el-card>
    <el-card shadow="hover" v-loading="loading">
      <el-table :data="items" stripe style="width:100%" :empty-text="t('common.noData')">
        <el-table-column prop="name" :label="t('marketplace.product')" min-width="160" />
        <el-table-column prop="description" :label="t('common.description')" min-width="200" show-overflow-tooltip />
        <el-table-column prop="sellerRS" :label="t('marketplace.seller')" width="200" show-overflow-tooltip />
        <el-table-column :label="t('marketplace.price')" width="120" align="right">
          <template #default="{ row }">{{ formatNQT(row.priceNQT) }} NRC</template>
        </el-table-column>
        <el-table-column prop="quantity" :label="t('marketplace.quantity')" width="80" align="center" />
        <el-table-column :label="t('common.actions')" width="100" fixed="right">
          <template #default="{ row }">
            <el-button size="small" text type="primary" @click="openPurchase(row)">{{ t('marketplace.buy') }}</el-button>
          </template>
        </el-table-column>
      </el-table>
      <div class="pagination-container" v-if="total > 20">
        <el-pagination v-model:current-page="page" :page-size="20" :total="total" layout="prev,pager,next" @current-change="refreshData" />
      </div>
    </el-card>
    <DGSListingModal v-model:visible="showListing" @success="refreshData" />
    <PurchaseProductModal v-model:visible="showPurchase" :product="selectedProduct" @success="refreshData" />
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'
import DGSListingModal from '@/components/modals/DGSListingModal.vue'
import PurchaseProductModal from '@/components/modals/PurchaseProductModal.vue'

const { t } = useI18n()
const loading = ref(false)
const items = ref<any[]>([])
const total = ref(0)
const page = ref(1)
const showListing = ref(false)
const showPurchase = ref(false)
const selectedProduct = ref<any>(null)
const filters = reactive({ seller: '', tag: '', inStockOnly: true })

function formatNQT(nqt: string) { return (Number(nqt || '0') / 1e8).toFixed(2) }

onMounted(() => refreshData())

async function refreshData() {
  loading.value = true
  try {
    const params: any = { firstIndex: (page.value - 1) * 20, lastIndex: page.value * 20 - 1, inStockOnly: filters.inStockOnly }
    if (filters.seller) params.seller = filters.seller
    if (filters.tag) params.tag = filters.tag
    const result = await nrcsApi.searchDGSGoods(params.query, params.tag, params.seller, params.firstIndex, params.lastIndex)
    items.value = result.goods || []
    total.value = items.value.length
  } catch (e: any) { ElMessage.error(e?.message || t('common.loadError')) }
  finally { loading.value = false }
}

function openPurchase(row: any) { selectedProduct.value = row; showPurchase.value = true }
</script>
<style scoped lang="scss">
@use '@/assets/styles/variables' as *;
.page-container { .page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; .page-title { font-size: 18px; font-weight: 600; display: flex; align-items: center; gap: 8px; margin: 0; } .header-actions { display: flex; gap: 8px; } } .pagination-container { display: flex; justify-content: center; margin-top: 16px; } }
.mb-4 { margin-bottom: $space-md; }
</style>
