<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><DataLine /></el-icon> {{ t('voting.activePolls') }}</h2>
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
        <el-table-column :label="t('common.actions')" width="180" fixed="right">
          <template #default="{ row }">
            <el-button size="small" text type="primary" @click="openVote(row)">{{ t('voting.vote') }}</el-button>
            <el-button size="small" text type="success" @click="openResults(row)">{{ t('voting.results') }}</el-button>
          </template>
        </el-table-column>
      </el-table>
      <div class="pagination-container" v-if="total > 20">
        <el-pagination v-model:current-page="page" :page-size="20" :total="total" layout="prev,pager,next" @current-change="refreshData" />
      </div>
    </el-card>
    <CreatePollModal v-model:visible="showCreate" @success="refreshData" />
    <CastVoteModal v-model:visible="showVote" :poll="selectedPoll" @success="refreshData" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'
import CreatePollModal from '@/components/modals/CreatePollModal.vue'
import CastVoteModal from '@/components/modals/CastVoteModal.vue'

const { t } = useI18n()
const loading = ref(false)
const items = ref<any[]>([])
const total = ref(0)
const page = ref(1)
const showCreate = ref(false)
const showVote = ref(false)
const selectedPoll = ref<any>(null)
const currentHeight = ref(0)

onMounted(async () => {
  try { const s = await nrcsApi.getBlockchainStatus(); currentHeight.value = (s as any).numberOfBlocks || 0 } catch {}
  refreshData()
})

async function refreshData() {
  loading.value = true
  try {
    const result = await nrcsApi.getPolls((page.value - 1) * 20, page.value * 20 - 1)
    items.value = (result.polls || []).filter((p: any) => p.finishHeight > currentHeight.value)
    total.value = items.value.length
  } catch (e: any) { ElMessage.error(e?.message || t('common.loadError')) }
  finally { loading.value = false }
}

function openVote(row: any) { selectedPoll.value = row; showVote.value = true }
function openResults(row: any) { /* navigate to results view */ ElMessage.info(`Poll: ${row.name}`) }
</script>
<style scoped lang="scss">
@use '@/assets/styles/variables' as *;
.page-container { .page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; .page-title { font-size: 18px; font-weight: 600; display: flex; align-items: center; gap: 8px; margin: 0; } .header-actions { display: flex; gap: 8px; } } .pagination-container { display: flex; justify-content: center; margin-top: 16px; } }
</style>
