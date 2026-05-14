<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Sell /></el-icon> {{ t('asset.exchange') }}</h2>
      <div class="header-actions">
        <el-button size="small" @click="refreshData"><el-icon><Refresh /></el-icon> {{ t('common.refresh') }}</el-button>
      </div>
    </div>
    <el-row :gutter="16">
      <el-col :span="12">
        <el-card shadow="hover" class="mb-4">
          <template #header><h3 class="card-title">{{ t('asset.buyOrders') }}</h3></template>
          <el-table :data="buyOrders" stripe size="small" style="width:100%" :empty-text="t('common.noData')">
            <el-table-column :label="t('asset.asset')" min-width="100">
              <template #default="{ row }">{{ row.assetName || row.asset }}</template>
            </el-table-column>
            <el-table-column :label="t('asset.price')" width="110" align="right">
              <template #default="{ row }">{{ formatNQT(row.priceNQT) }} NRC</template>
            </el-table-column>
            <el-table-column :label="t('asset.quantity')" width="80" align="center">
              <template #default="{ row }">{{ formatQNT(row.quantityQNT, 0) }}</template>
            </el-table-column>
          </el-table>
        </el-card>
      </el-col>
      <el-col :span="12">
        <el-card shadow="hover" class="mb-4">
          <template #header><h3 class="card-title">{{ t('asset.sellOrders') }}</h3></template>
          <el-table :data="sellOrders" stripe size="small" style="width:100%" :empty-text="t('common.noData')">
            <el-table-column :label="t('asset.asset')" min-width="100">
              <template #default="{ row }">{{ row.assetName || row.asset }}</template>
            </el-table-column>
            <el-table-column :label="t('asset.price')" width="110" align="right">
              <template #default="{ row }">{{ formatNQT(row.priceNQT) }} NRC</template>
            </el-table-column>
            <el-table-column :label="t('asset.quantity')" width="80" align="center">
              <template #default="{ row }">{{ formatQNT(row.quantityQNT, 0) }}</template>
            </el-table-column>
          </el-table>
        </el-card>
      </el-col>
    </el-row>
    <AssetOrderModal />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'
import AssetOrderModal from '@/components/modals/AssetOrderModal.vue'

const { t } = useI18n()
const loading = ref(false)
const buyOrders = ref<any[]>([])
const sellOrders = ref<any[]>([])
const showOrder = ref(false)
const selectedAsset = ref<any>(null)

function formatNQT(nqt: string) { return (Number(nqt || '0') / 1e8).toFixed(2) }
function formatQNT(qnt: string, _d: number) { return qnt || '0' }

onMounted(() => refreshData())

async function refreshData() {
  loading.value = true
  try {
    const accountId = localStorage.getItem('nrcs_account_id') || ''
    const [buy, sell] = await Promise.all([
      nrcsApi.getAccountCurrentBidOrders(accountId).catch(() => ({ bidOrders: [] })),
      nrcsApi.getAccountCurrentAskOrders(accountId).catch(() => ({ askOrders: [] }))
    ])
    buyOrders.value = (buy as any).bidOrders || []
    sellOrders.value = (sell as any).askOrders || []
  } catch (e: any) { ElMessage.error(e?.message || t('common.loadError')) }
  finally { loading.value = false }
}
</script>
<style scoped lang="scss">
@use '@/assets/styles/variables' as *;
.page-container { .page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; .page-title { font-size: 18px; font-weight: 600; display: flex; align-items: center; gap: 8px; margin: 0; } .header-actions { display: flex; gap: 8px; } } }
.card-title { margin: 0; font-size: 15px; color: $text-primary; }
.mb-4 { margin-bottom: $space-md; }
</style>
