<template>
  <div class="deletes-history-page">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Delete /></el-icon> {{ t('asset.deletesHistory') }}</h2>
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
        <el-table-column :label="t('common.issuer')" width="200" show-overflow-tooltip>
          <template #default="{ row }">
            {{ row.senderRS || '-' }}
          </template>
        </el-table-column>
        <el-table-column :label="t('dashboard.height')" width="100" align="right">
          <template #default="{ row }">{{ row.height || '-' }}</template>
        </el-table-column>
      </el-table>
      <div class="pagination-container">
        <el-pagination
          v-model:current-page="currentPage"
          :page-size="pageSize"
          :total="deletes.length"
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
import { Delete, Refresh } from '@element-plus/icons-vue'
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
const deletes = ref<any[]>([])
const currentPage = ref(1)
const pageSize = 20

// --- Computed ---
const paginatedItems = computed(() => {
  const start = (currentPage.value - 1) * pageSize
  return deletes.value.slice(start, start + pageSize)
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

async function loadDeletes() {
  if (!accountRS.value) return
  loading.value = true
  try {
    // Asset delete (shares) transactions: type=2, subtype=7
    // Also try getAssetTransfers which may include delete entries
    const [txResult, transferResult] = await Promise.allSettled([
      nrcsApi.getBlockchainTransactions(accountRS.value, 0, 199, 2, 7),
      nrcsApi.getAssetTransfers(undefined, accountRS.value, 0, 199),
    ])

    let allEntries: any[] = []

    // Collect delete-share transactions (type=2, subtype=7)
    if (txResult.status === 'fulfilled') {
      const txs = txResult.value?.transactions || []
      for (const tx of txs) {
        const attachment = tx.attachment || {}
        allEntries.push({
          transaction: tx.transaction || tx.signature,
          timestamp: tx.timestamp,
          height: tx.height,
          senderRS: tx.senderRS,
          asset: attachment.asset,
          quantityQNT: attachment.quantityQNT || '0',
          decimals: 0,
        })
      }
    }

    // Also include asset transfers where quantityQNT may be '0' (deletes)
    // or filter by specific sender
    if (transferResult.status === 'fulfilled') {
      const xfers = transferResult.value?.transfers || []
      for (const x of xfers) {
        // Only include if sender is the current account (issuer deleting shares)
        if (x.senderRS === accountRS.value && !allEntries.some((e) => e.transaction === (x.transaction || x.transfer))) {
          allEntries.push({
            transaction: x.transaction || x.transfer,
            timestamp: x.timestamp,
            height: x.height,
            senderRS: x.senderRS,
            asset: x.asset,
            quantityQNT: x.quantityQNT || '0',
            decimals: x.decimals ?? 0,
          })
        }
      }
    }

    // Sort by timestamp descending
    allEntries.sort((a, b) => (b.timestamp || 0) - (a.timestamp || 0))

    // Enrich with asset names
    const assetCache = new Map<string, any>()
    for (const entry of allEntries) {
      const assetId = entry.asset
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
        entry.assetName = detail.name
        entry.decimals = detail.decimals ?? entry.decimals
      }
    }

    deletes.value = allEntries
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
  await loadDeletes()
}

// --- Lifecycle ---
onMounted(() => {
  loadDeletes()
})

usePolling(refreshData, 30000, false)
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;

.deletes-history-page {
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
  .pagination-container {
    display: flex;
    justify-content: center;
    margin-top: 16px;
  }
}
</style>
