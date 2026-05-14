<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><EditPen /></el-icon> {{ t('voting.myPolls') }}</h2>
      <div class="header-actions">
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
        <el-table-column :label="t('voting.votingModel')" width="110" align="center">
          <template #default="{ row }">{{ ['', 'Account', 'Balance', 'Asset', 'Currency'][row.votingModel] || row.votingModel }}</template>
        </el-table-column>
        <el-table-column :label="t('common.actions')" width="180" fixed="right">
          <template #default="{ row }">
            <el-button size="small" text type="primary" @click="openResults(row)">{{ t('voting.results') }}</el-button>
            <el-button size="small" text type="success" @click="openVotes(row)">{{ t('voting.votes') }}</el-button>
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
import { ElMessage } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'

const { t } = useI18n()
const loading = ref(false)
const items = ref<any[]>([])
const total = ref(0)
const page = ref(1)

onMounted(() => refreshData())

async function refreshData() {
  loading.value = true
  try {
    const accountId = localStorage.getItem('nrcs_account_id') || ''
    const result = await nrcsApi.getBlockchainTransactions(accountId, (page.value - 1) * 20, page.value * 20 - 1, 7, 0)
    items.value = ((result as any).transactions || []).map((tx: any) => ({ ...tx, ...tx.attachment }))
    total.value = items.value.length
  } catch (e: any) { ElMessage.error(e?.message || t('common.loadError')) }
  finally { loading.value = false }
}

function openResults(row: any) { ElMessage.info(`Poll: ${row.name}`) }
function openVotes(row: any) { ElMessage.info(`Votes for: ${row.name}`) }
</script>
<style scoped lang="scss">
@use '@/assets/styles/variables' as *;
.page-container { .page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; .page-title { font-size: 18px; font-weight: 600; display: flex; align-items: center; gap: 8px; margin: 0; } .header-actions { display: flex; gap: 8px; } } .pagination-container { display: flex; justify-content: center; margin-top: 16px; } }
</style>
