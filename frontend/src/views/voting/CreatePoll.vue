<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Plus /></el-icon> {{ t('voting.createPoll') }}</h2>
      <div class="header-actions">
        <el-button type="primary" size="small" @click="showCreate = true"><el-icon><Plus /></el-icon> {{ t('voting.createPoll') }}</el-button>
        <el-button size="small" @click="refreshData"><el-icon><Refresh /></el-icon> {{ t('common.refresh') }}</el-button>
      </div>
    </div>
    <el-card shadow="hover" v-loading="loading">
      <el-table :data="items" stripe style="width:100%" :empty-text="t('common.noData')">
        <el-table-column prop="name" :label="t('voting.name')" min-width="180" />
        <el-table-column prop="description" :label="t('voting.description')" min-width="220" show-overflow-tooltip />
        <el-table-column :label="t('voting.finishHeight')" width="120" align="right">
          <template #default="{ row }">{{ row.finishHeight }}</template>
        </el-table-column>
        <el-table-column :label="t('common.date')" width="160">
          <template #default="{ row }">{{ formatDate(row.timestamp) }}</template>
        </el-table-column>
      </el-table>
      <div class="pagination-container" v-if="total > 20">
        <el-pagination v-model:current-page="page" :page-size="20" :total="total" layout="prev,pager,next" @current-change="refreshData" />
      </div>
    </el-card>
    <CreatePollModal v-model:visible="showCreate" @success="refreshData" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'
import CreatePollModal from '@/components/modals/CreatePollModal.vue'

const { t } = useI18n()
const loading = ref(false)
const items = ref<any[]>([])
const total = ref(0)
const page = ref(1)
const showCreate = ref(false)

function formatDate(ts?: number) { if (!ts) return ''; return new Date(new Date(Date.UTC(2013, 10, 24, 12, 0, 0)).getTime() + ts * 1000).toLocaleString() }

onMounted(() => refreshData())

async function refreshData() {
  loading.value = true
  try {
    const result = await nrcsApi.getPolls((page.value - 1) * 20, page.value * 20 - 1)
    const accountId = localStorage.getItem('nrcs_account_id') || ''
    items.value = ((result as any).polls || []).filter((p: any) => p.account === accountId || p.accountRS && p.accountRS.endsWith(accountId))
    total.value = items.value.length
  } catch (e: any) { ElMessage.error(e?.message || t('common.loadError')) }
  finally { loading.value = false }
}
</script>
<style scoped lang="scss">
@use '@/assets/styles/variables' as *;
.page-container { .page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; .page-title { font-size: 18px; font-weight: 600; display: flex; align-items: center; gap: 8px; margin: 0; } .header-actions { display: flex; gap: 8px; } } .pagination-container { display: flex; justify-content: center; margin-top: 16px; } }
</style>
