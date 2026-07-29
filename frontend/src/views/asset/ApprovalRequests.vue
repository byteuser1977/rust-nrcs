<template>
  <div class="approval-requests-page">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Checked /></el-icon> {{ t('asset.approvalRequests') }}</h2>
      <div class="header-actions">
        <el-button size="small" @click="refreshData"><el-icon><Refresh /></el-icon> {{ t('common.refresh') }}</el-button>
      </div>
    </div>

    <!-- Asset filter -->
    <el-card shadow="hover" class="filter-card">
      <el-row :gutter="16" align="middle">
        <el-col :span="8">
          <el-select
            v-model="selectedAssetFilter"
            :placeholder="t('asset.allAssets')"
            clearable
            filterable
            style="width: 100%"
            @change="onFilterChange"
          >
            <el-option
              v-for="a in userAssets"
              :key="a.asset"
              :label="a.name || truncateHash(a.asset, 8)"
              :value="a.asset"
            />
          </el-select>
        </el-col>
      </el-row>
    </el-card>

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
              @click="viewTransaction(row.transaction)"
            >
              {{ truncateHash(row.transaction || row.fullHash, 8) }}
            </el-button>
          </template>
        </el-table-column>
        <el-table-column :label="t('asset.asset')" width="140">
          <template #default="{ row }">
            {{ row.assetName || truncateHash(row.asset, 6) }}
          </template>
        </el-table-column>
        <el-table-column :label="t('common.type')" width="120">
          <template #default="{ row }">
            {{ formatTxType(row.type, row.subtype) }}
          </template>
        </el-table-column>
        <el-table-column :label="t('common.sender')" width="200" show-overflow-tooltip>
          <template #default="{ row }">
            {{ row.senderRS || '-' }}
          </template>
        </el-table-column>
        <el-table-column :label="t('common.amount')" width="120" align="right">
          <template #default="{ row }">
            {{ formatAmountFromAttachment(row.attachment) }}
          </template>
        </el-table-column>
        <el-table-column :label="t('common.deadline')" width="120" align="right">
          <template #default="{ row }">
            {{ row.phasingFinishHeight || '-' }}
          </template>
        </el-table-column>
        <el-table-column :label="t('common.date')" width="160">
          <template #default="{ row }">{{ formatTimestamp(row.timestamp) }}</template>
        </el-table-column>
        <el-table-column :label="t('common.actions')" width="100" fixed="right">
          <template #default="{ row }">
            <el-button
              size="small"
              type="primary"
              text
              :disabled="!canApprove(row)"
              @click="approveTx(row)"
            >
              {{ t('asset.approve') }}
            </el-button>
          </template>
        </el-table-column>
      </el-table>
      <div class="pagination-container">
        <el-pagination
          v-model:current-page="currentPage"
          :page-size="pageSize"
          :total="filteredRequests.length"
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
import { ElMessage, ElMessageBox } from 'element-plus'
import { Checked, Refresh } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { useAccountStore } from '@/stores/modules/account.store'
import { usePolling } from '@/composables/usePolling'
import { formatTimestamp, truncateHash, qntToQntf, formatAmount } from '@/utils/format'

const { t } = useI18n()
const router = useRouter()
const accountStore = useAccountStore()

const accountRS = computed(() => accountStore.accountRS)
const secretPhrase = computed(() => accountStore.secretPhrase)

// --- State ---
const loading = ref(false)
const allRequests = ref<any[]>([])
const userAssets = ref<any[]>([])
const selectedAssetFilter = ref<string | undefined>(undefined)
const currentPage = ref(1)
const pageSize = 20

// --- Computed ---
const filteredRequests = computed(() => {
  let items = allRequests.value
  if (selectedAssetFilter.value) {
    items = items.filter((r) => {
      const attachment = r.attachment || {}
      return attachment.asset === selectedAssetFilter.value
    })
  }
  return items
})

const paginatedItems = computed(() => {
  const start = (currentPage.value - 1) * pageSize
  return filteredRequests.value.slice(start, start + pageSize)
})

// --- Methods ---
function formatTxType(type: number, subtype: number): string {
  const key = `txType.subtype_${type}_${subtype}`
  const translated = t(key)
  if (translated !== key) return translated
  return `${type}.${subtype}`
}

function formatAmountFromAttachment(attachment: Record<string, any> | undefined): string {
  if (!attachment) return '-'
  if (attachment.quantityQNT) {
    try {
      return qntToQntf(attachment.quantityQNT, attachment.decimals ?? 0)
    } catch {
      return attachment.quantityQNT
    }
  }
  if (attachment.amountNQT) {
    return formatAmount(attachment.amountNQT)
  }
  if (attachment.priceNQTPerShare) {
    return formatAmount(attachment.priceNQTPerShare)
  }
  return '-'
}

function canApprove(row: any): boolean {
  return !!secretPhrase.value
}

async function loadPhasedTransactions() {
  if (!accountRS.value) return
  loading.value = true
  try {
    const result = await nrcsApi.getAccountPhasedTransactions(accountRS.value, 0, 199)
    const txs = result?.transactions || []
    allRequests.value = txs
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  } finally {
    loading.value = false
  }
}

async function loadUserAssets() {
  if (!accountRS.value) return
  try {
    const result = await nrcsApi.getAccountAssets(accountRS.value)
    const balances = (result as any)?.assetBalances || (result as any)?.accountAssets || []
    // Load asset names
    const enriched: any[] = []
    for (const bal of balances) {
      try {
        const detail = await nrcsApi.getAsset(bal.asset)
        enriched.push({
          asset: bal.asset,
          name: detail?.name,
          decimals: detail?.decimals ?? 0,
        })
      } catch {
        enriched.push({ asset: bal.asset, name: undefined, decimals: 0 })
      }
    }
    userAssets.value = enriched
  } catch {
    userAssets.value = []
  }
}

async function approveTx(row: any) {
  if (!secretPhrase.value) {
    ElMessage.warning(t('asset.enterSecretPhrase'))
    return
  }

  try {
    await ElMessageBox.prompt(
      t('asset.enterSecretPhrase'),
      t('asset.approve'),
      {
        confirmButtonText: t('common.confirm'),
        cancelButtonText: t('common.cancel'),
        inputType: 'password',
        inputValue: secretPhrase.value,
      },
    ).then(async ({ value }) => {
      if (!value) {
        ElMessage.warning(t('common.secretPhraseRequired'))
        return
      }
      await nrcsApi.approveTransaction({
        secretPhrase: value,
        transaction: row.transaction || row.fullHash,
        feeNQT: '100000000',
        deadline: 1440,
      })
      ElMessage.success(t('common.operationSuccess'))
      refreshData()
    }).catch(() => {})
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.operationFailed'))
  }
}

function viewTransaction(txId: string) {
  if (!txId) return
  router.push({ name: 'transaction', params: { tx: txId } })
}

function onFilterChange() {
  currentPage.value = 1
}

async function refreshData() {
  currentPage.value = 1
  await Promise.all([loadPhasedTransactions(), loadUserAssets()])
}

// --- Lifecycle ---
onMounted(() => {
  refreshData()
})

usePolling(refreshData, 30000, false)
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;

.approval-requests-page {
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
  .filter-card {
    margin-bottom: 16px;
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
