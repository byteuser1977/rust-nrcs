<template>
  <div class="my-assets-page">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Present /></el-icon> {{ t('asset.myAssets') }}</h2>
      <div class="header-actions">
        <el-button type="primary" size="small" @click="showIssue = true">
          <el-icon><Plus /></el-icon> {{ t('asset.issueAsset') }}
        </el-button>
        <el-button size="small" @click="refreshData"><el-icon><Refresh /></el-icon> {{ t('common.refresh') }}</el-button>
      </div>
    </div>

    <el-card shadow="hover" v-loading="loading">
      <el-table
        :data="paginatedItems"
        stripe
        style="width: 100%"
        :empty-text="t('common.noData')"
        @expand-change="onExpand"
      >
        <el-table-column type="expand">
          <template #default="{ row }">
            <div class="asset-detail">
              <el-descriptions :column="2" border size="small">
                <el-descriptions-item :label="t('common.id')" :span="2">
                  <span class="mono">{{ row.asset }}</span>
                </el-descriptions-item>
                <el-descriptions-item :label="t('common.issuer')">
                  {{ row.issuerRS || '-' }}
                </el-descriptions-item>
                <el-descriptions-item :label="t('common.description')">
                  {{ row.description || '-' }}
                </el-descriptions-item>
                <el-descriptions-item :label="t('asset.totalSupply')">
                  {{ row.totalQuantityQNT ? formatQNT(row.totalQuantityQNT, row.decimals) : '-' }}
                </el-descriptions-item>
              </el-descriptions>
            </div>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.name')" min-width="140">
          <template #default="{ row }">{{ row.name || truncateHash(row.asset, 6) }}</template>
        </el-table-column>
        <el-table-column :label="t('asset.quantity')" width="140" align="right">
          <template #default="{ row }">{{ formatQNT(row.quantityQNT, row.decimals) }}</template>
        </el-table-column>
        <el-table-column :label="t('asset.unconfirmedQuantity')" width="140" align="right">
          <template #default="{ row }">{{ formatQNT(row.unconfirmedQuantityQNT, row.decimals) }}</template>
        </el-table-column>
        <el-table-column :label="t('asset.decimals')" width="80" align="center">
          <template #default="{ row }">{{ row.decimals || 0 }}</template>
        </el-table-column>
        <el-table-column :label="t('common.actions')" width="160" fixed="right">
          <template #default="{ row }">
            <el-button size="small" text type="primary" @click="openTransfer(row)">
              <el-icon><Sort /></el-icon> {{ t('asset.transfer') }}
            </el-button>
          </template>
        </el-table-column>
      </el-table>
      <div class="pagination-container">
        <el-pagination
          v-model:current-page="currentPage"
          :page-size="pageSize"
          :total="items.length"
          layout="prev, pager, next"
          background
          small
        />
      </div>
    </el-card>

    <TransferAssetModal v-model:visible="showTransfer" @success="refreshData" />
    <IssueAssetModal v-model:visible="showIssue" @success="refreshData" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { Present, Plus, Refresh, Sort } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { useAccountStore } from '@/stores/modules/account.store'
import { usePolling } from '@/composables/usePolling'
import { qntToQntf, truncateHash } from '@/utils/format'
import TransferAssetModal from '@/components/modals/TransferAssetModal.vue'
import IssueAssetModal from '@/components/modals/IssueAssetModal.vue'

const { t } = useI18n()
const accountStore = useAccountStore()

const accountRS = computed(() => accountStore.accountRS)

// --- State ---
const loading = ref(false)
const items = ref<any[]>([])
const showTransfer = ref(false)
const showIssue = ref(false)
const selectedAsset = ref<any>(null)
const currentPage = ref(1)
const pageSize = 20

// --- Computed ---
const paginatedItems = computed(() => {
  const start = (currentPage.value - 1) * pageSize
  return items.value.slice(start, start + pageSize)
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

async function loadAssets() {
  if (!accountRS.value) return
  loading.value = true
  try {
    const result = await nrcsApi.getAccountAssets(accountRS.value)
    const balances = (result as any)?.assetBalances || (result as any)?.accountAssets || []

    // Load asset details (name, decimals, issuer, description) for each holding
    const enriched: any[] = []
    for (const bal of balances) {
      try {
        const detail = await nrcsApi.getAsset(bal.asset)
        enriched.push({
          ...bal,
          name: detail?.name,
          decimals: detail?.decimals ?? bal.decimals ?? 0,
          description: detail?.description,
          issuerRS: detail?.issuerRS,
          totalQuantityQNT: detail?.quantityQNT,
        })
      } catch {
        enriched.push({ ...bal, decimals: bal.decimals ?? 0 })
      }
    }
    items.value = enriched
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  } finally {
    loading.value = false
  }
}

function openTransfer(row: any) {
  selectedAsset.value = row
  showTransfer.value = true
}

function onExpand(row: any, expandedRows: any[]) {
  // Expand callback — detail data is already loaded
}

async function refreshData() {
  currentPage.value = 1
  await loadAssets()
}

// --- Lifecycle ---
onMounted(() => {
  loadAssets()
})

usePolling(refreshData, 30000, false)
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;

.my-assets-page {
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
  .asset-detail {
    padding: 12px 24px;
    .mono {
      font-family: monospace;
      font-size: 13px;
      color: $text-muted;
    }
  }
  .pagination-container {
    display: flex;
    justify-content: center;
    margin-top: 16px;
  }
}
</style>
