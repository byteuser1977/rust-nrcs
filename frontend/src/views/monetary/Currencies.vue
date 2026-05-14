<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Coin /></el-icon> {{ t('monetary.title') }}</h2>
      <div class="header-actions">
        <el-button type="primary" size="small" @click="showIssue = true"><el-icon><Plus /></el-icon> {{ t('monetary.issueCurrency') }}</el-button>
        <el-button size="small" @click="refreshData"><el-icon><Refresh /></el-icon> {{ t('common.refresh') }}</el-button>
      </div>
    </div>
    <el-card shadow="hover" v-loading="loading">
      <el-table :data="items" stripe style="width:100%" :empty-text="t('common.noData')">
        <el-table-column prop="code" :label="t('monetary.code')" width="80" />
        <el-table-column prop="name" :label="t('common.name')" min-width="140" />
        <el-table-column prop="description" :label="t('common.description')" min-width="200" show-overflow-tooltip />
        <el-table-column :label="t('monetary.currentSupply')" width="140" align="right">
          <template #default="{ row }">{{ formatQNT(row.currentSupply, row.decimals) }}</template>
        </el-table-column>
        <el-table-column prop="decimals" :label="t('monetary.decimals')" width="80" align="center" />
        <el-table-column :label="t('monetary.type')" width="160">
          <template #default="{ row }">
            <el-tag v-if="row.type & 1" size="small" class="mr-1">EXCH</el-tag>
            <el-tag v-if="row.type & 2" size="small" type="warning" class="mr-1">CTRL</el-tag>
            <el-tag v-if="row.type & 16" size="small" type="success" class="mr-1">MINT</el-tag>
          </template>
        </el-table-column>
      </el-table>
      <div class="pagination-container" v-if="total > 20">
        <el-pagination v-model:current-page="page" :page-size="20" :total="total" layout="prev,pager,next" @current-change="refreshData" />
      </div>
    </el-card>
    <IssueCurrencyModal v-model:visible="showIssue" @success="refreshData" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'
import IssueCurrencyModal from '@/components/modals/IssueCurrencyModal.vue'

const { t } = useI18n()
const loading = ref(false)
const items = ref<any[]>([])
const total = ref(0)
const page = ref(1)
const showIssue = ref(false)

function formatQNT(qnt: string, decimals: number) {
  if (!qnt) return '0'
  return (Number(qnt) / Math.pow(10, decimals || 0)).toLocaleString()
}

onMounted(() => refreshData())

async function refreshData() {
  loading.value = true
  try {
    const result = await nrcsApi.getAllCurrencies((page.value - 1) * 20, page.value * 20 - 1)
    items.value = result.currencies || []
    total.value = items.value.length
  } catch (e: any) { ElMessage.error(e?.message || t('common.loadError')) }
  finally { loading.value = false }
}
</script>
<style scoped lang="scss">
@use '@/assets/styles/variables' as *;
.page-container { .page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; .page-title { font-size: 18px; font-weight: 600; display: flex; align-items: center; gap: 8px; margin: 0; } .header-actions { display: flex; gap: 8px; } } .pagination-container { display: flex; justify-content: center; margin-top: 16px; } }
.mr-1 { margin-right: 4px; }
</style>
