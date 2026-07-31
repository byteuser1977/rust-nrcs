<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title">
        <el-icon><Cloudy /></el-icon>
        {{ t('datacloud.title') }}
      </h2>
      <div class="header-actions">
        <el-button type="primary" size="small" @click="showUpload = true">
          <el-icon><Upload /></el-icon>
          {{ t('datacloud.uploadData') }}
        </el-button>
        <el-button size="small" @click="searchData">
          <el-icon><Refresh /></el-icon>
          {{ t('common.refresh') }}
        </el-button>
      </div>
    </div>

    <el-card shadow="hover" class="search-card">
      <el-tabs v-model="searchMode" @tab-change="onSearchModeChange">
        <el-tab-pane :label="t('datacloud.byAccount')" name="account">
          <el-form :inline="true" class="search-form">
            <el-form-item :label="t('datacloud.account')">
              <el-input
                v-model="searchAccount"
                :placeholder="t('datacloud.accountPlaceholder')"
                clearable
                style="width: 280px"
                @keyup.enter="searchData"
              />
            </el-form-item>
            <el-form-item>
              <el-button type="primary" @click="searchData">
                <el-icon><Search /></el-icon>
                {{ t('common.search') }}
              </el-button>
            </el-form-item>
          </el-form>
        </el-tab-pane>
        <el-tab-pane :label="t('datacloud.byFulltext')" name="fulltext">
          <el-form :inline="true" class="search-form">
            <el-form-item :label="t('datacloud.query')">
              <el-input
                v-model="searchQuery"
                :placeholder="t('datacloud.queryPlaceholder')"
                clearable
                style="width: 280px"
                @keyup.enter="searchData"
              />
            </el-form-item>
            <el-form-item :label="t('datacloud.channel')">
              <el-input
                v-model="searchChannel"
                :placeholder="t('datacloud.channelPlaceholder')"
                clearable
                style="width: 160px"
                @keyup.enter="searchData"
              />
            </el-form-item>
            <el-form-item>
              <el-button type="primary" @click="searchData">
                <el-icon><Search /></el-icon>
                {{ t('common.search') }}
              </el-button>
            </el-form-item>
          </el-form>
        </el-tab-pane>
        <el-tab-pane :label="t('datacloud.byTag')" name="tag">
          <div class="tag-cloud" v-if="popularTags.length">
            <el-tag
              v-for="tag in popularTags"
              :key="tag"
              :type="activeTag === tag ? 'primary' : 'info'"
              size="default"
              effect="plain"
              class="tag-item"
              @click="selectTag(tag)"
            >
              {{ tag }}
            </el-tag>
          </div>
          <div class="empty-tags" v-else>
            <span class="text-muted">{{ t('datacloud.noTags') }}</span>
          </div>
        </el-tab-pane>
      </el-tabs>
    </el-card>

    <el-card shadow="hover" v-loading="loading" class="result-card">
      <el-table
        :data="items"
        stripe
        style="width: 100%"
        :empty-text="t('common.noData')"
        row-key="transaction"
      >
        <el-table-column prop="name" :label="t('datacloud.name')" min-width="180">
          <template #default="{ row }">
            <span class="text-accent">{{ row.name || '-' }}</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.account')" width="200" show-overflow-tooltip>
          <template #default="{ row }">
            <span class="text-mono text-sm">{{ row.accountRS || row.account || '-' }}</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('datacloud.mimeType')" width="120">
          <template #default="{ row }">
            <el-tag size="small" type="info" effect="plain">{{ row.type || '-' }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column :label="t('datacloud.channel')" width="100">
          <template #default="{ row }">
            <span class="text-sm">{{ row.channel || '-' }}</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('datacloud.filename')" width="160" show-overflow-tooltip>
          <template #default="{ row }">
            <span class="text-sm">{{ row.filename || '-' }}</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('datacloud.data')" width="100" align="center">
          <template #default="{ row }">
            <el-button
              v-if="row.transaction"
              size="small"
              text
              type="primary"
              @click="downloadData(row)"
            >
              <el-icon><Download /></el-icon>
              {{ t('datacloud.download') }}
            </el-button>
            <span v-else class="text-muted text-sm">-</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.date')" width="170">
          <template #default="{ row }">
            <span class="text-muted text-sm">{{ formatDate(row.transactionTimestamp || row.timestamp) }}</span>
          </template>
        </el-table-column>
        <el-table-column type="expand" width="50">
          <template #default="{ row }">
            <div class="expand-detail">
              <el-descriptions :column="2" border size="small">
                <el-descriptions-item :label="t('datacloud.name')">{{ row.name }}</el-descriptions-item>
                <el-descriptions-item :label="t('datacloud.description')">{{ row.description || '-' }}</el-descriptions-item>
                <el-descriptions-item :label="t('datacloud.channel')">{{ row.channel || '-' }}</el-descriptions-item>
                <el-descriptions-item :label="t('datacloud.tags')">{{ row.tags || '-' }}</el-descriptions-item>
                <el-descriptions-item :label="t('datacloud.mimeType')">{{ row.type || '-' }}</el-descriptions-item>
                <el-descriptions-item :label="t('datacloud.filename')">{{ row.filename || '-' }}</el-descriptions-item>
                <el-descriptions-item :label="t('common.transaction')" :span="2">
                  <span class="text-mono text-xs">{{ row.transaction || '-' }}</span>
                </el-descriptions-item>
              </el-descriptions>
            </div>
          </template>
        </el-table-column>
      </el-table>

      <div class="pagination-container" v-if="pagination.total.value > pagination.pageSize.value">
        <el-pagination
          v-model:current-page="pagination.currentPage.value"
          :page-size="pagination.pageSize.value"
          :total="pagination.total.value"
          :disabled="pagination.isLoading.value"
          layout="total, prev, pager, next"
          @current-change="handlePageChange"
        />
      </div>
    </el-card>

    <UploadTaggedDataModal v-model:visible="showUpload" @success="searchData" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { Cloudy, Upload, Refresh, Search, Download } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { usePagination } from '@/composables/usePagination'
import { formatTimestamp } from '@/utils/format'
import UploadTaggedDataModal from '@/components/modals/UploadTaggedDataModal.vue'

const { t } = useI18n()

const loading = ref(false)
const items = ref<any[]>([])
const pagination = usePagination(20)
const popularTags = ref<string[]>([])
const activeTag = ref('')

const searchMode = ref('fulltext')
const searchAccount = ref('')
const searchQuery = ref('')
const searchChannel = ref('')
const showUpload = ref(false)

onMounted(() => {
  loadTags()
  searchData()
})

function onSearchModeChange() {
  pagination.reset()
  if (searchMode.value === 'tag') {
    loadTags()
  } else {
    searchData()
  }
}

function handlePageChange(page: number) {
  pagination.goToPage(page)
  searchData()
}

async function loadTags() {
  try {
    const result = await nrcsApi.getDataTags()
    popularTags.value = (result as any).tags || []
  } catch {
    popularTags.value = []
  }
}

function selectTag(tag: string) {
  activeTag.value = tag
  searchByTag(tag)
}

async function searchByTag(tag: string) {
  loading.value = true
  try {
    const result = await nrcsApi.getAllTaggedData(
      pagination.firstIndex.value,
      pagination.lastIndex.value
    )
    let list = (result as any).data || []
    if (tag) {
      list = list.filter((d: any) => {
        const tags = (d.tags || '').toLowerCase()
        return tags.includes(tag.toLowerCase())
      })
    }
    items.value = list
    pagination.setTotalFromList(list.length)
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  } finally {
    loading.value = false
  }
}

async function searchData() {
  loading.value = true
  try {
    const firstIndex = pagination.firstIndex.value
    const lastIndex = pagination.lastIndex.value

    let result: any

    if (searchMode.value === 'account' && searchAccount.value) {
      result = await nrcsApi.getAccountTaggedData(searchAccount.value, firstIndex, lastIndex)
    } else if (searchMode.value === 'fulltext' && (searchQuery.value || searchChannel.value)) {
      let query = searchQuery.value || ''
      if (searchChannel.value) {
        query = query ? `${query} channel:${searchChannel.value}` : `channel:${searchChannel.value}`
      }
      result = await nrcsApi.searchTaggedData(query, firstIndex, lastIndex)
    } else {
      result = await nrcsApi.getAllTaggedData(firstIndex, lastIndex)
    }

    const list = (result as any).data || []
    items.value = list
    pagination.setTotalFromList(list.length)
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  } finally {
    loading.value = false
  }
}

async function downloadData(row: any) {
  if (!row.transaction) return
  try {
    const result = await nrcsApi.downloadTaggedData(row.transaction)
    const data = (result as any).data
    if (data) {
      const byteString = atob(data)
      const mimeType = row.type || 'application/octet-stream'
      const blob = new Blob(
        [Uint8Array.from(byteString, (c: string) => c.charCodeAt(0))],
        { type: mimeType }
      )
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = row.filename || row.name || 'download'
      a.click()
      URL.revokeObjectURL(url)
      ElMessage.success(t('common.operationSuccess'))
    } else {
      ElMessage.warning(t('datacloud.noDataForDownload'))
    }
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  }
}

function formatDate(ts?: number): string { return ts ? formatTimestamp(ts) : '' }
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;

.page-container {
  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: $space-lg;
    .page-title {
      font-size: $font-size-lg;
      font-weight: 600;
      display: flex;
      align-items: center;
      gap: $space-sm;
      margin: 0;
      color: $text-primary;
    }
    .header-actions {
      display: flex;
      gap: $space-sm;
    }
  }
}

.search-card {
  margin-bottom: $space-lg;
  border: 1px solid $border-subtle !important;
  border-radius: $radius-lg !important;
  background: rgba($surface-800, 0.6) !important;

  :deep(.el-card__body) {
    padding: $space-md $space-lg !important;
  }
}

.search-form {
  padding-top: $space-sm;
}

.tag-cloud {
  display: flex;
  flex-wrap: wrap;
  gap: $space-sm;
  padding: $space-md 0;
}

.tag-item {
  cursor: pointer;
}

.empty-tags {
  padding: $space-md 0;
}

.result-card {
  border: 1px solid $border-subtle !important;
  border-radius: $radius-lg !important;
  background: linear-gradient(180deg, rgba($surface-800, 0.8), rgba($surface-900, 0.9)) !important;

  :deep(.el-table) {
    background: transparent !important;
    --el-table-bg-color: transparent;
    --el-table-tr-bg-color: transparent;
    --el-table-header-bg-color: rgba(255, 255, 255, 0.02);
    --el-table-row-hover-bg-color: rgba($primary, 0.06);
    --el-table-border-color: $border-subtle;
    --el-table-text-color: $text-primary;
    --el-table-header-text-color: $text-muted;

    th.el-table__cell {
      background: rgba(255, 255, 255, 0.025) !important;
      font-weight: 600;
      font-size: $font-size-xs;
      text-transform: uppercase;
      border-bottom: 1px solid $border-subtle;
    }

    td.el-table__cell {
      border-bottom: 1px solid rgba($border-default, 0.5);
    }
  }
}

.expand-detail {
  padding: $space-lg;
  background: rgba($surface-800, 0.4);
}

.text-mono {
  font-family: $font-mono;
}

.text-accent {
  color: $primary;
  font-weight: 500;
}

.text-sm {
  font-size: $font-size-sm;
}

.text-xs {
  font-size: $font-size-xs;
}

.text-muted {
  color: $text-muted;
}

.pagination-container {
  display: flex;
  justify-content: center;
  margin-top: $space-lg;
}
</style>
