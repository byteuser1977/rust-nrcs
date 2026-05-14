<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Document /></el-icon> {{ t('asset.openOrders') }}</h2>
      <div class="header-actions">
        <el-button size="small" @click="refreshData"><el-icon><Refresh /></el-icon> {{ t('common.refresh') }}</el-button>
      </div>
    </div>
    <el-card shadow="hover" v-loading="loading">
      <el-table :data="items" stripe style="width:100%" :empty-text="t('common.noData')">
        <el-table-column :label="t('asset.type')" width="80">
          <template #default="{ row }">{{ row.type === 'ask' ? t('asset.sell') : t('asset.buy') }}</template>
        </el-table-column>
        <el-table-column :label="t('asset.asset')" min-width="140">
          <template #default="{ row }">{{ row.assetName || row.asset }}</template>
        </el-table-column>
        <el-table-column :label="t('asset.price')" width="120" align="right">
          <template #default="{ row }">{{ formatNQT(row.priceNQT) }} NRC</template>
        </el-table-column>
        <el-table-column :label="t('asset.quantity')" width="80" align="center">
          <template #default="{ row }">{{ row.quantityQNT }}</template>
        </el-table-column>
        <el-table-column :label="t('common.date')" width="160">
          <template #default="{ row }">{{ formatDate(row.timestamp) }}</template>
        </el-table-column>
        <el-table-column :label="t('common.actions')" width="100" fixed="right">
          <template #default="{ row }">
            <el-button size="small" text type="danger" @click="cancelOrder(row)">{{ t('common.cancel') }}</el-button>
          </template>
        </el-table-column>
      </el-table>
      <div class="pagination-container" v-if="total > 20">
        <el-pagination v-model:current-page="page" :page-size="20" :total="total" layout="prev,pager,next" @current-change="refreshData" />
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, ElMessageBox } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'

const { t } = useI18n()
const loading = ref(false)
const items = ref<any[]>([])
const total = ref(0)
const page = ref(1)

function formatNQT(nqt: string) { return (Number(nqt || '0') / 1e8).toFixed(2) }
function formatDate(ts?: number) {
  if (!ts) return ''
  return new Date(new Date(Date.UTC(2013, 10, 24, 12, 0, 0)).getTime() + ts * 1000).toLocaleString()
}

onMounted(() => refreshData())

async function refreshData() {
  loading.value = true
  try {
    const accountId = localStorage.getItem('nrcs_account_id') || ''
    const [buy, sell] = await Promise.all([
      nrcsApi.getAccountCurrentBidOrders(accountId).catch(() => ({ bidOrders: [] })),
      nrcsApi.getAccountCurrentAskOrders(accountId).catch(() => ({ askOrders: [] }))
    ])
    items.value = [
      ...((buy as any).bidOrders || []).map((o: any) => ({ ...o, type: 'bid' })),
      ...((sell as any).askOrders || []).map((o: any) => ({ ...o, type: 'ask' }))
    ]
    total.value = items.value.length
  } catch (e: any) { ElMessage.error(e?.message || t('common.loadError')) }
  finally { loading.value = false }
}

async function cancelOrder(row: any) {
  ElMessageBox.confirm(t('asset.cancelConfirm'), t('common.warning'), { type: 'warning' }).then(async () => {
    try {
      const phrase = localStorage.getItem('nrcs_passphrase') || ''
      if (row.type === 'bid') await nrcsApi.cancelBidOrder({ secretPhrase: phrase, order: row.order, feeNQT: '100000000', deadline: 1440 })
      else await nrcsApi.cancelAskOrder({ secretPhrase: phrase, order: row.order, feeNQT: '100000000', deadline: 1440 })
      ElMessage.success(t('asset.cancelSuccess'))
      refreshData()
    } catch (e: any) { ElMessage.error(e?.message || t('asset.cancelError')) }
  }).catch(() => {})
}
</script>
<style scoped lang="scss">
@use '@/assets/styles/variables' as *;
.page-container { .page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; .page-title { font-size: 18px; font-weight: 600; display: flex; align-items: center; gap: 8px; margin: 0; } .header-actions { display: flex; gap: 8px; } } .pagination-container { display: flex; justify-content: center; margin-top: 16px; } }
</style>
