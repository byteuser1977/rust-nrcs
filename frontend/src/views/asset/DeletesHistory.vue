<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Delete /></el-icon> {{ t('asset.deletesHistory') }}</h2>
      <div class="header-actions">
        <el-button size="small" @click="refreshData"><el-icon><Refresh /></el-icon> {{ t('common.refresh') }}</el-button>
      </div>
    </div>
    <el-card shadow="hover" v-loading="loading">
      <el-table :data="items" stripe style="width:100%" :empty-text="t('common.noData')">
        <el-table-column prop="asset" :label="t('asset.asset')" width="180" show-overflow-tooltip />
        <el-table-column :label="t('asset.quantity')" width="120" align="right">
          <template #default="{ row }">{{ formatQNT(row.quantityQNT, row.decimals) }}</template>
        </el-table-column>
        <el-table-column prop="senderRS" :label="t('common.sender')" width="200" show-overflow-tooltip />
        <el-table-column prop="recipientRS" :label="t('common.recipient')" width="200" show-overflow-tooltip />
        <el-table-column :label="t('common.date')" width="160">
          <template #default="{ row }">{{ formatDate(row.timestamp) }}</template>
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
import { ElMessage } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'

const { t } = useI18n()
const loading = ref(false)
const items = ref<any[]>([])
const total = ref(0)
const page = ref(1)

function formatQNT(qnt: string, decimals: number) { return qnt ? (Number(qnt) / Math.pow(10, decimals || 0)).toLocaleString() : '0' }
function formatDate(ts?: number) { if (!ts) return ''; return new Date(new Date(Date.UTC(2013, 10, 24, 12, 0, 0)).getTime() + ts * 1000).toLocaleString() }

onMounted(() => refreshData())

async function refreshData() {
  loading.value = true
  try {
    const accountId = localStorage.getItem('nrcs_account_id') || ''
    const result = await nrcsApi.getAssetTransfers(undefined, accountId, (page.value - 1) * 20, page.value * 20 - 1)
    items.value = (result as any).transfers || []
    total.value = items.value.length
  } catch (e: any) { ElMessage.error(e?.message || t('common.loadError')) }
  finally { loading.value = false }
}
</script>
<style scoped lang="scss">
@use '@/assets/styles/variables' as *;
.page-container { .page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; .page-title { font-size: 18px; font-weight: 600; display: flex; align-items: center; gap: 8px; margin: 0; } .header-actions { display: flex; gap: 8px; } } .pagination-container { display: flex; justify-content: center; margin-top: 16px; } }
</style>
