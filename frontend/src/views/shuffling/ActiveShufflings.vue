<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Connection /></el-icon> {{ t('shuffling.activeShufflings') }}</h2>
      <div class="header-actions">
        <el-button type="primary" size="small" @click="showCreate = true"><el-icon><Plus /></el-icon> {{ t('shuffling.createShuffling') }}</el-button>
        <el-button size="small" @click="refreshData"><el-icon><Refresh /></el-icon> {{ t('common.refresh') }}</el-button>
      </div>
    </div>
    <el-card shadow="hover" v-loading="loading">
      <el-table :data="items" stripe style="width:100%" :empty-text="t('common.noData')">
        <el-table-column prop="shuffling" :label="t('common.id')" width="180" show-overflow-tooltip />
        <el-table-column :label="t('shuffling.stage')" width="100">
          <template #default="{ row }">{{ ['-', t('shuffling.processing'), t('shuffling.verification'), t('shuffling.done')][row.stage || 0] }}</template>
        </el-table-column>
        <el-table-column :label="t('shuffling.amount')" width="120" align="right">
          <template #default="{ row }">{{ formatNQT(row.amount) }} NRC</template>
        </el-table-column>
        <el-table-column prop="participantCount" :label="t('shuffling.participants')" width="100" align="center" />
        <el-table-column prop="registrantCount" :label="t('shuffling.registrants')" width="100" align="center" />
        <el-table-column :label="t('common.actions')" width="140" fixed="right">
          <template #default="{ row }">
            <el-button size="small" text type="primary" @click="openRegister(row)">{{ t('shuffling.register') }}</el-button>
            <el-button size="small" text type="success" @click="openProcess(row)">{{ t('shuffling.process') }}</el-button>
          </template>
        </el-table-column>
      </el-table>
      <div class="pagination-container" v-if="total > 20">
        <el-pagination v-model:current-page="page" :page-size="20" :total="total" layout="prev,pager,next" @current-change="refreshData" />
      </div>
    </el-card>
    <ShufflingCreateModal v-model:visible="showCreate" @success="refreshData" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'
import ShufflingCreateModal from '@/components/modals/ShufflingCreateModal.vue'

const { t } = useI18n()
const loading = ref(false)
const items = ref<any[]>([])
const total = ref(0)
const page = ref(1)
const showCreate = ref(false)

function formatNQT(nqt: string) { return (Number(nqt || '0') / 1e8).toFixed(2) }

onMounted(() => refreshData())

async function refreshData() {
  loading.value = true
  try {
    const result = await nrcsApi.getAllShufflings((page.value - 1) * 20, page.value * 20 - 1)
    items.value = result.shufflings || []
    total.value = items.value.length
  } catch (e: any) { ElMessage.error(e?.message || t('common.loadError')) }
  finally { loading.value = false }
}

function openRegister(row: any) { /* register for shuffling */ }
function openProcess(row: any) { /* process shuffling */ }
</script>
<style scoped lang="scss">
@use '@/assets/styles/variables' as *;
.page-container { .page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; .page-title { font-size: 18px; font-weight: 600; display: flex; align-items: center; gap: 8px; margin: 0; } .header-actions { display: flex; gap: 8px; } } .pagination-container { display: flex; justify-content: center; margin-top: 16px; } }
</style>
