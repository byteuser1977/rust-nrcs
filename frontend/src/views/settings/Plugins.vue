<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Opportunity /></el-icon> {{ t('settings.plugins') }}</h2>
      <el-button type="primary" size="small" @click="refreshData">
        <el-icon><Refresh /></el-icon> {{ t('common.refresh') }}
      </el-button>
    </div>

    <el-card shadow="hover" v-loading="isLoading">
      <el-table :data="plugins" style="width: 100%" :empty-text="t('common.noData')">
        <el-table-column label="Name" min-width="180">
          <template #default="{ row }">
            <span class="plugin-name">{{ row.name || row.plugin }}</span>
          </template>
        </el-table-column>
        <el-table-column label="Version" width="100">
          <template #default="{ row }">
            <el-tag size="small">{{ row.version || '-' }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column label="Website" min-width="180">
          <template #default="{ row }">
            <a v-if="row.website" :href="row.website" target="_blank" class="link">{{ row.website }}</a>
            <span v-else>-</span>
          </template>
        </el-table-column>
        <el-table-column label="Validation" width="110">
          <template #default="{ row }">
            <el-tag :type="row.validation === 'VALID' ? 'success' : 'danger'" size="small">
              {{ row.validation || 'Unknown' }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="Compatibility" width="130">
          <template #default="{ row }">
            <el-tag :type="row.compatibility === 'COMPATIBLE' ? 'success' : 'warning'" size="small">
              {{ row.compatibility || 'Unknown' }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="Status" width="120">
          <template #default="{ row }">
            <el-tag :type="row.launched ? 'success' : 'info'" size="small">
              {{ row.launched ? 'Launched' : 'Not launched' }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="Description" min-width="200">
          <template #default="{ row }">
            <span class="text-muted">{{ row.description || '-' }}</span>
          </template>
        </el-table-column>
      </el-table>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { nrcsApi } from '@/api/modules/nrcs.api'

const { t } = useI18n()
const isLoading = ref(false)
const plugins = ref<any[]>([])

onMounted(() => { refreshData() })

async function refreshData() {
  isLoading.value = true
  try {
    const result = await nrcsApi.getPlugins()
    plugins.value = result.plugins || []
  } catch (error) {
    console.error('Failed to load plugins:', error)
  } finally {
    isLoading.value = false
  }
}
</script>

<style scoped lang="scss">
.page-container {
  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
    .page-title {
      font-size: 18px;
      font-weight: 600;
      display: flex;
      align-items: center;
      gap: 8px;
      margin: 0;
    }
  }
}
.plugin-name {
  font-weight: 500;
}
.link {
  color: #409eff;
  text-decoration: none;
  &:hover { text-decoration: underline; }
}
.text-muted {
  color: #909399;
  font-size: 13px;
}
</style>
