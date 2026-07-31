<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Upload /></el-icon> {{ t('datacloud.fileUpload') }}</h2>
      <div class="header-actions">
        <el-button type="primary" size="small" @click="showUpload = true"><el-icon><Plus /></el-icon> {{ t('datacloud.uploadData') }}</el-button>
        <el-button size="small" @click="refreshData"><el-icon><Refresh /></el-icon> {{ t('common.refresh') }}</el-button>
      </div>
    </div>
    <el-card shadow="hover" v-loading="loading">
      <el-table :data="items" stripe style="width:100%" :empty-text="t('common.noData')">
        <el-table-column prop="name" :label="t('datacloud.name')" min-width="160" />
        <el-table-column prop="description" :label="t('common.description')" min-width="200" show-overflow-tooltip />
        <el-table-column prop="channel" :label="t('datacloud.channel')" width="120" />
        <el-table-column prop="tags" :label="t('datacloud.tags')" width="140" show-overflow-tooltip />
        <el-table-column prop="accountRS" :label="t('common.uploader')" width="200" show-overflow-tooltip />
        <el-table-column :label="t('common.date')" width="160">
          <template #default="{ row }">{{ formatDate(row.transactionTimestamp || row.timestamp) }}</template>
        </el-table-column>
      </el-table>
      <div class="pagination-container" v-if="total > 20">
        <el-pagination v-model:current-page="page" :page-size="20" :total="total" layout="prev,pager,next" @current-change="refreshData" />
      </div>
    </el-card>
    <UploadTaggedDataModal v-model:visible="showUpload" @success="refreshData" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { formatTimestamp } from '@/utils/format'
import UploadTaggedDataModal from '@/components/modals/UploadTaggedDataModal.vue'

const { t } = useI18n()
const loading = ref(false)
const items = ref<any[]>([])
const total = ref(0)
const page = ref(1)
const showUpload = ref(false)

function formatDate(ts?: number) { return ts ? formatTimestamp(ts) : '' }

onMounted(() => refreshData())

async function refreshData() {
  loading.value = true
  try {
    const result = await nrcsApi.getAllTaggedData((page.value - 1) * 20, page.value * 20 - 1)
    items.value = (result as any).data || []
    total.value = items.value.length
  } catch (e: any) { ElMessage.error(e?.message || t('common.loadError')) }
  finally { loading.value = false }
}
</script>
<style scoped lang="scss">
@use '@/assets/styles/variables' as *;
.page-container { .page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; .page-title { font-size: 18px; font-weight: 600; display: flex; align-items: center; gap: 8px; margin: 0; } .header-actions { display: flex; gap: 8px; } } .pagination-container { display: flex; justify-content: center; margin-top: 16px; } }
</style>
