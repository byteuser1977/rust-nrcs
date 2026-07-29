<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title">
        <el-icon><User /></el-icon>
        {{ t('shuffling.myShufflings') }}
      </h2>
      <div class="header-actions">
        <el-button type="primary" size="small" @click="showCreate = true">
          <el-icon><Plus /></el-icon>
          {{ t('shuffling.createShuffling') }}
        </el-button>
        <el-button size="small" @click="refreshData">
          <el-icon><Refresh /></el-icon>
          {{ t('common.refresh') }}
        </el-button>
      </div>
    </div>

    <el-card shadow="hover" v-loading="loading" class="shuffling-card">
      <el-table
        :data="items"
        stripe
        style="width: 100%"
        :empty-text="t('common.noData')"
        row-key="shuffling"
      >
        <el-table-column :label="t('shuffling.id')" width="180" show-overflow-tooltip>
          <template #default="{ row }">
            <span class="text-mono text-sm">{{ truncateFullHash(row.shuffling) }}</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('shuffling.stage')" width="110" align="center">
          <template #default="{ row }">
            <el-tag size="small" :type="stageTagType(row.stage)" effect="plain">
              {{ getStageName(row.stage) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column :label="t('shuffling.holdingType')" width="100">
          <template #default="{ row }">
            <span class="text-sm">{{ holdingTypeName(row.holdingType) }}</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('shuffling.amount')" width="130" align="right">
          <template #default="{ row }">
            <span class="text-mono">{{ formatNrcAmount(row.amount) }}</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('shuffling.participants')" width="120" align="center">
          <template #default="{ row }">
            <span class="text-sm">{{ row.registrantCount || 0 }} / {{ row.participantCount || 0 }}</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('shuffling.shufflerStatus')" width="140" align="center">
          <template #default="{ row }">
            <el-tag v-if="row.isShufflerRunning" size="small" type="success" effect="plain">
              {{ t('common.running') }}
            </el-tag>
            <el-tag v-else size="small" type="info" effect="plain">
              {{ t('common.stopped') }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column :label="t('shuffling.recipient')" width="200" show-overflow-tooltip>
          <template #default="{ row }">
            <span class="text-mono text-sm">{{ row.recipientRS || row.recipient || '-' }}</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.state')" width="100" align="center">
          <template #default="{ row }">
            <el-tag size="small" :type="stateTagType(row)" effect="plain">
              {{ getStateText(row) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.actions')" width="160" fixed="right">
          <template #default="{ row }">
            <el-button
              v-if="!row.isShufflerRunning && row.stage !== 2 && row.stage !== 3"
              size="small"
              type="primary"
              @click="startShuffler(row)"
            >
              {{ t('shuffling.startShuffler') }}
            </el-button>
            <el-button
              v-if="row.isShufflerRunning"
              size="small"
              type="warning"
              @click="stopShuffler(row)"
            >
              {{ t('shuffling.stopShuffler') }}
            </el-button>
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

    <ShufflingCreateModal v-model:visible="showCreate" @success="refreshData" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { User, Plus, Refresh } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { usePagination } from '@/composables/usePagination'
import { formatAmount, truncateHash } from '@/utils/format'
import { useAccountStore } from '@/stores/modules/account.store'
import ShufflingCreateModal from '@/components/modals/ShufflingCreateModal.vue'

const { t } = useI18n()
const accountStore = useAccountStore()

const loading = ref(false)
const items = ref<any[]>([])
const pagination = usePagination(20)
const showCreate = ref(false)

const accountRS = ref(accountStore.accountRS || localStorage.getItem('nrcs_account_rs') || '')

onMounted(() => {
  refreshData()
})

function handlePageChange(page: number) {
  pagination.goToPage(page)
  refreshData()
}

async function refreshData() {
  loading.value = true
  try {
    const acct = accountRS.value
    if (!acct) {
      ElMessage.warning(t('common.noAccount'))
      return
    }
    const result = await nrcsApi.getAccountShufflings(
      acct,
      pagination.firstIndex.value,
      pagination.lastIndex.value
    )
    const list = (result as any).shufflings || []
    items.value = list
    pagination.setTotalFromList(list.length)
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  } finally {
    loading.value = false
  }
}

function getStageName(stage: number): string {
  const stages: Record<number, string> = {
    0: t('shuffling.processing'),
    1: t('shuffling.verification'),
    2: t('shuffling.done'),
    3: t('shuffling.cancelled')
  }
  return stages[stage] || t('common.unknown')
}

function stageTagType(stage: number): string {
  if (stage === 0 || stage === 1) return 'warning'
  if (stage === 2) return 'success'
  if (stage === 3) return 'danger'
  return 'info'
}

function holdingTypeName(ht: number): string {
  if (ht === 0) return 'NRC'
  if (ht === 1) return t('shuffling.asset')
  if (ht === 2) return t('shuffling.currency')
  return '-'
}

function stateTagType(row: any): string {
  if (row.stage === 2) return 'success'
  if (row.stage === 3) return 'danger'
  if (row.isShufflerRunning) return 'warning'
  return 'info'
}

function getStateText(row: any): string {
  if (row.stage === 2) return t('common.completed')
  if (row.stage === 3) return t('common.cancelled')
  if (row.isShufflerRunning) return t('common.active')
  return t('common.idle')
}

function truncateFullHash(hash?: string): string {
  return truncateHash(hash || '', 8)
}

function formatNrcAmount(nqt?: string): string {
  if (!nqt || nqt === '0') return '0'
  return formatAmount(nqt)
}

async function startShuffler(row: any) {
  const secretPhrase = accountStore.secretPhrase || localStorage.getItem('nrcs-secretPhrase') || ''
  if (!secretPhrase) {
    ElMessage.warning(t('dashboard.enterSecretPhrase'))
    return
  }
  try {
    await nrcsApi.startShuffler({
      secretPhrase,
      shufflingFullHash: row.shuffling,
      feeNQT: '100000000',
      deadline: 1440
    })
    ElMessage.success(t('common.operationSuccess'))
    refreshData()
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  }
}

async function stopShuffler(row: any) {
  const secretPhrase = accountStore.secretPhrase || localStorage.getItem('nrcs-secretPhrase') || ''
  if (!secretPhrase) {
    ElMessage.warning(t('dashboard.enterSecretPhrase'))
    return
  }
  try {
    await nrcsApi.stopShuffler({
      secretPhrase,
      shufflingFullHash: row.shuffling,
      feeNQT: '100000000',
      deadline: 1440
    })
    ElMessage.success(t('common.operationSuccess'))
    refreshData()
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  }
}
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

.shuffling-card {
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

.text-mono {
  font-family: $font-mono;
}

.text-sm {
  font-size: $font-size-sm;
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
