<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><User /></el-icon> {{ t('menu.myShufflings') }}</h2>
      <el-button type="primary" size="small" @click="refreshData">
        <el-icon><Refresh /></el-icon> {{ t('common.refresh') }}
      </el-button>
    </div>
    <el-card shadow="hover" v-loading="isLoading">
      <el-table :data="items" style="width: 100%" :empty-text="t('common.noData')">
        <el-table-column type="index" width="60" label="#" />
        <el-table-column prop="shuffling" label="ID" min-width="120">
          <template #default="{ row }">
            <span class="hash-truncate">{{ row.shuffling }}</span>
          </template>
        </el-table-column>
        <el-table-column prop="amount" label="Amount" width="120" />
        <el-table-column prop="participantCount" label="Participants" width="120" />
        <el-table-column prop="stage" label="Stage" width="100">
          <template #default="{ row }">
            <el-tag size="small">{{ row.stage }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column label="Date" width="180">
          <template #default="{ row }">
            {{ formatDate(row.timestamp) }}
          </template>
        </el-table-column>
      </el-table>
      <div class="pagination-container" v-if="total > pageSize">
        <el-pagination
          v-model:current-page="currentPage"
          :page-size="pageSize"
          :total="total"
          layout="prev, pager, next"
          @current-change="handlePageChange"
        />
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { nrcsApi } from '@/api/modules/nrcs.api'

const { t } = useI18n()
const isLoading = ref(false)
const items = ref<any[]>([])
const total = ref(0)
const currentPage = ref(1)
const pageSize = ref(20)

onMounted(() => { refreshData() })

async function refreshData() {
  isLoading.value = true
  try {
    const account = localStorage.getItem('nrcs_account_rs')
    if (account) {
      const result = await nrcsApi.getAccountShufflings(account, (currentPage.value - 1) * 20, currentPage.value * 20 - 1)
      items.value = result.shufflings || []
      total.value = items.value.length
    }
  } catch (error) {
    console.error('Failed to load shufflings:', error)
  } finally {
    isLoading.value = false
  }
}

function formatDate(timestamp?: number): string {
  if (!timestamp) return ''
  const epochStart = new Date(Date.UTC(2013, 10, 24, 12, 0, 0))
  return new Date(epochStart.getTime() + timestamp * 1000).toLocaleString()
}

function handlePageChange(page: number) {
  currentPage.value = page
  refreshData()
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
  .pagination-container {
    display: flex;
    justify-content: center;
    margin-top: 16px;
  }
}
.hash-truncate {
  font-family: 'Roboto Mono', monospace;
  font-size: 0.85em;
}
</style>
