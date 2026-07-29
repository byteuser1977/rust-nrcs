<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title">
        <el-icon><Connection /></el-icon>
        {{ t('shuffling.activeShufflings') }}
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
            <span class="text-sm">{{ row.holdingType === 0 ? 'NRC' : (row.holdingType === 1 ? t('shuffling.asset') : t('shuffling.currency')) }}</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('shuffling.amount')" width="130" align="right">
          <template #default="{ row }">
            <span class="text-mono">{{ formatNrcAmount(row.amount) }}</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('shuffling.blocksRemaining')" width="130" align="center">
          <template #default="{ row }">
            <el-tag v-if="row.blocksRemaining !== undefined" size="small" :type="row.blocksRemaining > 0 ? 'warning' : 'danger'" effect="plain">
              {{ row.blocksRemaining }}
            </el-tag>
            <span v-else class="text-muted text-sm">-</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('shuffling.participants')" width="130" align="center">
          <template #default="{ row }">
            <span class="text-sm">{{ row.registrantCount || 0 }} / {{ row.participantCount || 0 }}</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('shuffling.issuer')" width="200" show-overflow-tooltip>
          <template #default="{ row }">
            <span class="text-mono text-sm">{{ row.issuerRS || row.accountRS || row.account || '-' }}</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.status')" width="100" align="center">
          <template #default="{ row }">
            <el-tag v-if="isActive(row)" size="small" type="success" effect="plain">
              {{ t('common.active') }}
            </el-tag>
            <el-tag v-else size="small" type="info" effect="plain">
              {{ t('common.inactive') }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.actions')" width="140" fixed="right">
          <template #default="{ row }">
            <el-button size="small" text type="primary" @click="openRegister(row)">
              {{ t('shuffling.register') }}
            </el-button>
            <el-button size="small" text type="success" @click="openProcess(row)">
              {{ t('shuffling.process') }}
            </el-button>
          </template>
        </el-table-column>
      </el-table>

      <div class="pagination-container" v-if="total > 20">
        <el-pagination
          v-model:current-page="page"
          :page-size="20"
          :total="total"
          layout="total, prev, pager, next"
          @current-change="handlePageChange"
        />
      </div>
    </el-card>

    <ShufflingCreateModal v-model:visible="showCreate" @success="refreshData" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { Connection, Plus, Refresh } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { formatAmount, truncateHash } from '@/utils/format'
import { useAccountStore } from '@/stores/modules/account.store'
import ShufflingCreateModal from '@/components/modals/ShufflingCreateModal.vue'

const { t } = useI18n()
const accountStore = useAccountStore()

const loading = ref(false)
const items = ref<any[]>([])
const total = ref(0)
const page = ref(1)
const showCreate = ref(false)

let pollTimer: number | null = null

onMounted(() => {
  refreshData()
  pollTimer = window.setInterval(refreshData, 15000)
})

onUnmounted(() => {
  if (pollTimer) {
    clearInterval(pollTimer)
    pollTimer = null
  }
})

function handlePageChange(p: number) {
  page.value = p
  refreshData()
}

async function refreshData() {
  loading.value = true
  try {
    const firstIndex = (page.value - 1) * 20
    const lastIndex = page.value * 20 - 1
    const result = await nrcsApi.getAllShufflings(firstIndex, lastIndex)
    const list = (result as any).shufflings || []
    items.value = list.filter((s: any) => isActive(s))
    total.value = list.length
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  } finally {
    loading.value = false
  }
}

function isActive(shuffling: any): boolean {
  const stage = shuffling.stage || 0
  return stage >= 0 && stage <= 3
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

function truncateFullHash(hash?: string): string {
  return truncateHash(hash || '', 8)
}

function formatNrcAmount(nqt?: string): string {
  if (!nqt || nqt === '0') return '0'
  return formatAmount(nqt)
}

async function openRegister(row: any) {
  const secretPhrase = accountStore.secretPhrase || localStorage.getItem('nrcs-secretPhrase') || ''
  if (!secretPhrase) {
    ElMessage.warning(t('dashboard.enterSecretPhrase'))
    return
  }
  try {
    await nrcsApi.shufflingRegister({
      secretPhrase,
      shuffling: row.shuffling,
      feeNQT: '100000000',
      deadline: 1440
    })
    ElMessage.success(t('common.operationSuccess'))
    refreshData()
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  }
}

async function openProcess(row: any) {
  const secretPhrase = accountStore.secretPhrase || localStorage.getItem('nrcs-secretPhrase') || ''
  if (!secretPhrase) {
    ElMessage.warning(t('dashboard.enterSecretPhrase'))
    return
  }
  try {
    await nrcsApi.shufflingProcess({
      secretPhrase,
      shuffling: row.shuffling,
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
