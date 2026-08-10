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
      <el-table :data="items" stripe size="small" style="width:100%" :empty-text="t('common.noData')">
        <el-table-column prop="name" :label="t('datacloud.name')" min-width="140" />
        <el-table-column prop="description" :label="t('common.description')" min-width="180" show-overflow-tooltip />
        <el-table-column prop="channel" :label="t('datacloud.channel')" width="120" />
        <el-table-column :label="t('datacloud.tags')" width="140">
          <template #default="{ row }">
            <el-tag v-for="tag in (row.tags || '').split(',')" :key="tag" size="small" class="tag-item" v-show="tag">{{ tag }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.uploader')" min-width="160" show-overflow-tooltip>
          <template #default="{ row }">{{ row.accountRS }}</template>
        </el-table-column>
        <el-table-column :label="t('common.date')" width="150">
          <template #default="{ row }">{{ formatDate(row.transactionTimestamp || row.timestamp) }}</template>
        </el-table-column>
        <el-table-column :label="t('common.actions')" width="120" fixed="right">
          <template #default="{ row }">
            <el-button size="small" text type="primary" @click="downloadData(row)">
              <el-icon><Download /></el-icon> {{ t('common.download') || 'Download' }}
            </el-button>
          </template>
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
import { Download, Plus, Refresh, Upload } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { useAccountStore } from '@/stores/modules/account.store'
import { formatTimestamp } from '@/utils/format'
import UploadTaggedDataModal from '@/components/modals/UploadTaggedDataModal.vue'

const { t } = useI18n()
const accountStore = useAccountStore()

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
    const account = accountStore.accountRS || localStorage.getItem('nrcs_account_rs') || ''
    const result = await nrcsApi.getAccountTaggedData(account, (page.value - 1) * 20, page.value * 20 - 1)
    items.value = (result as any).data || []
    total.value = items.value.length
  } catch (e: any) { ElMessage.error(e?.message || t('common.loadError')) }
  finally { loading.value = false }
}

async function downloadData(row: any) {
  if (row.transaction) {
    try {
      window.open('/nrcs?requestType=downloadTaggedData&transaction=' + row.transaction, '_blank')
    } catch {
      ElMessage.warning(t('common.loadError'))
    }
  }
}
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;
.page-container {
  .page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;
    .page-title { font-size: 18px; font-weight: 600; display: flex; align-items: center; gap: 8px; margin: 0; }
    .header-actions { display: flex; gap: 8px; }
  }
  .pagination-container { display: flex; justify-content: center; margin-top: 16px; }
  .tag-item { margin-right: 4px; margin-bottom: 2px; }
}
</style>
