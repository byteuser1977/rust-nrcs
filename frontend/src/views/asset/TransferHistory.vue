<template>
  <div class="transfer-history-page">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Connection /></el-icon> {{ t('asset.transferHistory') }}</h2>
      <div class="header-actions">
        <el-button size="small" @click="refreshData"><el-icon><Refresh /></el-icon> {{ t('common.refresh') }}</el-button>
      </div>
    </div>

    <el-card shadow="hover" v-loading="loading">
      <el-table
        :data="paginatedItems"
        stripe
        style="width: 100%"
        :empty-text="t('common.noData')"
      >
        <el-table-column :label="t('common.transaction')" width="150">
          <template #default="{ row }">
            <el-button
              link
              type="primary"
              size="small"
              class="mono-link"
              @click="viewTransaction(row.transaction || row.transfer)"
            >
              {{ truncateHash(row.transaction || row.transfer, 8) }}
            </el-button>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.date')" width="160">
          <template #default="{ row }">{{ formatTimestamp(row.timestamp) }}</template>
        </el-table-column>
        <el-table-column :label="t('asset.asset')" width="160">
          <template #default="{ row }">
            {{ row.assetName || truncateHash(row.asset, 8) }}
          </template>
        </el-table-column>
        <el-table-column :label="t('asset.quantity')" width="140" align="right">
          <template #default="{ row }">{{ formatQNT(row.quantityQNT, row.decimals) }}</template>
        </el-table-column>
        <el-table-column :label="t('common.sender')" width="200" show-overflow-tooltip>
          <template #default="{ row }">
            <span :class="{ highlight: row.senderRS === accountRS }">
              {{ row.senderRS || '-' }}
            </span>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.recipient')" width="200" show-overflow-tooltip>
          <template #default="{ row }">
            <span :class="{ highlight: row.recipientRS === accountRS }">
              {{ row.recipientRS || '-' }}
            </span>
          </template>
        </el-table-column>
      </el-table>
      <div class="pagination-container">
        <el-pagination
          v-model:current-page="currentPage"
          :page-size="pageSize"
          :total="transfers.length"
          layout="prev, pager, next"
          background
          small
        />
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { Connection, Refresh } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { useAccountStore } from '@/stores/modules/account.store'
import { usePolling } from '@/composables/usePolling'
import { qntToQntf, formatTimestamp, truncateHash } from '@/utils/format'

const { t } = useI18n()
const router = useRouter()
const accountStore = useAccountStore()

const accountRS = computed(() => accountStore.accountRS)

// --- State ---
const loading = ref(false)
const transfers = ref<any[]>([])
const currentPage = ref(1)
const pageSize = 20

// --- Computed ---
const paginatedItems = computed(() => {
  const start = (currentPage.value - 1) * pageSize
  return transfers.value.slice(start, start + pageSize)
})

// --- Methods ---
function formatQNT(qnt: string, decimals: number): string {
  if (!qnt || qnt === '0') return '0'
  try {
    return qntToQntf(qnt, decimals)
  } catch {
    return qnt
  }
}

async function loadTransfers() {
  if (!accountRS.value) return
  loading.value = true
  try {
    const result = await nrcsApi.getAssetTransfers(
      undefined,
      accountRS.value,
      0,
      199,
    )
    const allTransfers = result?.transfers || []

    // Enrich with asset names
    const assetCache = new Map<string, any>()
    for (const transfer of allTransfers) {
      const assetId = transfer.asset
      if (assetId && !assetCache.has(assetId)) {
        try {
          const detail = await nrcsApi.getAsset(assetId)
          assetCache.set(assetId, detail)
        } catch {
          assetCache.set(assetId, null)
        }
      }
      const detail = assetCache.get(assetId)
      if (detail) {
        transfer.assetName = detail.name
        transfer.decimals = detail.decimals ?? 0
      } else {
        transfer.decimals = transfer.decimals ?? 0
      }
    }

    transfers.value = allTransfers
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  } finally {
    loading.value = false
  }
}

function viewTransaction(txId: string) {
  if (!txId) return
  router.push({ name: 'transaction', params: { tx: txId } })
}

async function refreshData() {
  currentPage.value = 1
  await loadTransfers()
}

// --- Lifecycle ---
onMounted(() => {
  loadTransfers()
})

usePolling(refreshData, 30000, false)
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;

.transfer-history-page {
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
    .header-actions {
      display: flex;
      gap: 8px;
    }
  }
  .mono-link {
    font-family: monospace;
    font-size: 13px;
  }
  .highlight {
    font-weight: 600;
    color: var(--el-color-primary);
  }
  .pagination-container {
    display: flex;
    justify-content: center;
    margin-top: 16px;
  }
}
</style>
