<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Present /></el-icon> {{ t('asset.myAssets') }}</h2>
      <div class="header-actions">
        <el-button type="primary" size="small" @click="showIssue = true"><el-icon><Plus /></el-icon> {{ t('asset.issueAsset') }}</el-button>
        <el-button size="small" @click="refreshData"><el-icon><Refresh /></el-icon> {{ t('common.refresh') }}</el-button>
      </div>
    </div>
    <el-card shadow="hover" v-loading="loading">
      <el-table :data="items" stripe style="width:100%" :empty-text="t('common.noData')">
        <el-table-column prop="asset" :label="t('common.id')" width="180" show-overflow-tooltip />
        <el-table-column prop="name" :label="t('common.name')" min-width="140" />
        <el-table-column :label="t('asset.quantity')" min-width="140">
          <template #default="{ row }">{{ formatQNT(row.quantityQNT, row.decimals) }}</template>
        </el-table-column>
        <el-table-column prop="decimals" :label="t('asset.decimals')" width="80" align="center" />
        <el-table-column :label="t('common.actions')" width="160" fixed="right">
          <template #default="{ row }">
            <el-button size="small" text type="primary" @click="openTransfer(row)">{{ t('asset.transfer') }}</el-button>
          </template>
        </el-table-column>
      </el-table>
      <div class="pagination-container" v-if="total > 20">
        <el-pagination v-model:current-page="page" :page-size="20" :total="total" layout="prev,pager,next" @current-change="refreshData" />
      </div>
    </el-card>
    <IssueAssetModal v-model:visible="showIssue" @success="refreshData" />
    <TransferAssetModal v-model:visible="showTransfer" @success="refreshData" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'
import IssueAssetModal from '@/components/modals/IssueAssetModal.vue'
import TransferAssetModal from '@/components/modals/TransferAssetModal.vue'

const { t } = useI18n()
const loading = ref(false)
const items = ref<any[]>([])
const total = ref(0)
const page = ref(1)
const showIssue = ref(false)
const showTransfer = ref(false)
const selectedAsset = ref<any>(null)

function formatQNT(qnt: string, decimals: number) {
  if (!qnt) return '0'
  return (Number(qnt) / Math.pow(10, decimals || 0)).toLocaleString()
}

onMounted(() => refreshData())

async function refreshData() {
  loading.value = true
  try {
    const accountId = localStorage.getItem('nrcs_account_id') || ''
    const accountRS = localStorage.getItem('nrcs_account_rs') || ''
    const result = await nrcsApi.getAccountAssets(accountId)
    items.value = result.accountAssets || []
    total.value = items.value.length
  } catch (e: any) { ElMessage.error(e?.message || t('common.loadError')) }
  finally { loading.value = false }
}

function openTransfer(row: any) { selectedAsset.value = row; showTransfer.value = true }
</script>
<style scoped lang="scss">
@use '@/assets/styles/variables' as *;
.page-container { .page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; .page-title { font-size: 18px; font-weight: 600; display: flex; align-items: center; gap: 8px; margin: 0; } .header-actions { display: flex; gap: 8px; } } .pagination-container { display: flex; justify-content: center; margin-top: 16px; } }
</style>
