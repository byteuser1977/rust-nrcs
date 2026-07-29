<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title">
        <el-icon><List /></el-icon>
        {{ t('dashboard.transactions') }}
      </h2>
      <div class="header-actions">
        <el-button size="small" @click="refreshData">
          <el-icon><Refresh /></el-icon>
          {{ t('common.refresh') }}
        </el-button>
      </div>
    </div>

    <el-card shadow="hover" class="filter-card">
      <div class="filter-pills">
        <el-button
          v-for="f in typeFilters"
          :key="f.value"
          :type="activeFilter === f.value ? 'primary' : 'default'"
          size="small"
          round
          @click="setFilter(f.value)"
        >
          {{ f.label }}
        </el-button>
      </div>
    </el-card>

    <el-card shadow="hover" v-loading="pagination.isLoading.value" class="tx-card">
      <el-table
        :data="transactions"
        style="width: 100%"
        :empty-text="t('common.noData')"
        :row-class-name="txRowClassName"
        @row-click="showTransactionDetail"
        row-key="transaction"
      >
        <el-table-column :label="t('common.date')" width="170">
          <template #default="{ row }">
            <span class="text-muted text-sm">{{ formatTimestamp(row.timestamp) }}</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.type')" width="130">
          <template #default="{ row }">
            <span class="tx-type-badge" :class="typeColorClass(row.type)">
              {{ getTxTypeLabel(row.type, row.subtype) }}
            </span>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.amount')" width="150" align="right">
          <template #default="{ row }">
            <span class="tx-amount text-mono" :class="getAmountClass(row)">
              {{ formatNrcAmount(row.amountNQT) }}
            </span>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.fee')" width="120" align="right">
          <template #default="{ row }">
            <span class="text-muted text-sm text-mono">{{ formatNrcAmount(row.feeNQT) }}</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.account')" min-width="200">
          <template #default="{ row }">
            <span class="text-mono text-accent">
              {{ getCounterparty(row) }}
            </span>
          </template>
        </el-table-column>
        <el-table-column :label="t('dashboard.height')" width="90" align="center">
          <template #default="{ row }">
            <span v-if="row.height" class="text-muted text-sm text-mono">#{{ row.height }}</span>
            <el-tag v-else type="warning" size="small">{{ t('dashboard.pending') }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column :label="t('dashboard.confirmations')" width="120" align="center">
          <template #default="{ row }">
            <span
              v-if="row.confirmations !== undefined && row.confirmations >= 0"
              class="conf-tag"
              :class="row.confirmations > 10 ? 'conf-tag--confirmed' : 'conf-tag--pending'"
            >
              {{ row.confirmations }}
            </span>
            <span v-else class="conf-tag conf-tag--unconfirmed">{{ t('dashboard.unconfirmed') }}</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.status')" width="80" align="center">
          <template #default="{ row }">
            <el-icon v-if="!row.height" class="status-icon status-unconfirmed"><Clock /></el-icon>
            <el-icon v-else-if="row.confirmations && row.confirmations >= 10" class="status-icon status-confirmed"><CircleCheck /></el-icon>
            <el-icon v-else class="status-icon status-pending"><Loading /></el-icon>
          </template>
        </el-table-column>
      </el-table>

      <div class="pagination-container">
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

    <el-dialog
      v-model="txDetailVisible"
      :title="t('dashboard.transactionDetails')"
      width="700px"
      destroy-on-close
    >
      <div class="tx-detail" v-if="selectedTransaction">
        <el-descriptions :column="2" border size="small">
          <el-descriptions-item
            v-for="key in detailKeys"
            :key="key"
            :label="key"
          >
            {{ formatDetailValue((selectedTransaction as any)[key]) }}
          </el-descriptions-item>
        </el-descriptions>
      </div>
      <el-empty v-else :description="t('dashboard.noTransactionSelected')" />
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { List, Refresh, Clock, CircleCheck, Loading } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import type { NrcsTransaction, NrcsUnconfirmedTransaction } from '@/api/modules/nrcs.api'
import { usePagination } from '@/composables/usePagination'
import { formatTimestamp, formatAmount } from '@/utils/format'
import { useAccountStore } from '@/stores/modules/account.store'

const { t } = useI18n()
const accountStore = useAccountStore()

const transactions = ref<(NrcsTransaction | NrcsUnconfirmedTransaction)[]>([])
const pagination = usePagination(15)
const activeFilter = ref('all')
const txDetailVisible = ref(false)
const selectedTransaction = ref<NrcsTransaction | NrcsUnconfirmedTransaction | null>(null)

const accountRS = computed(() => accountStore.accountRS || localStorage.getItem('nrcs_account_rs') || '')

const typeFilters = computed(() => [
  { value: 'all', label: t('common.all') },
  { value: '0', label: t('txType.payment') || 'Payment' },
  { value: '2', label: t('txType.asset') || 'Asset' },
  { value: '5', label: t('txType.currency') || 'Currency' },
  { value: '3', label: t('txType.marketplace') || 'Marketplace' },
  { value: '7', label: t('txType.voting') || 'Voting' },
  { value: '1', label: t('txType.messaging') || 'Messaging' },
  { value: '6', label: t('txType.taggedData') || 'Data Cloud' },
])

const detailKeys = [
  'transaction', 'type', 'subtype', 'senderRS', 'recipientRS',
  'amountNQT', 'feeNQT', 'height', 'confirmations', 'timestamp',
  'block', 'signature', 'fullHash'
]

onMounted(() => {
  refreshData()
})

function setFilter(value: string) {
  activeFilter.value = value
  pagination.reset()
  refreshData()
}

function handlePageChange(page: number) {
  pagination.goToPage(page)
  refreshData()
}

async function refreshData() {
  pagination.isLoading.value = true
  try {
    const acct = accountRS.value
    if (!acct) {
      ElMessage.warning(t('common.noAccount'))
      return
    }

    const typeParam = activeFilter.value !== 'all' ? Number(activeFilter.value) : undefined

    const [confirmedResult, unconfirmedResult] = await Promise.allSettled([
      nrcsApi.getBlockchainTransactions(
        acct,
        pagination.firstIndex.value,
        pagination.lastIndex.value,
        typeParam
      ),
      nrcsApi.getUnconfirmedTransactions(acct)
    ])

    const confirmedTxs = confirmedResult.status === 'fulfilled'
      ? (confirmedResult.value?.transactions || [])
      : []
    const unconfirmedTxs = unconfirmedResult.status === 'fulfilled'
      ? (unconfirmedResult.value?.unconfirmedTransactions || [])
      : []

    const combined: any[] = [
      ...(unconfirmedTxs.map((tx: any) => ({
        ...tx,
        confirmations: -1,
        height: undefined,
        _isUnconfirmed: true
      }))),
      ...confirmedTxs
    ]

    transactions.value = combined
    pagination.setTotalFromList(confirmedTxs.length)
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  } finally {
    pagination.isLoading.value = false
  }
}

function formatNrcAmount(nqt?: string): string {
  if (!nqt || nqt === '0') return '0'
  return formatAmount(nqt)
}

function formatDetailValue(val: any): string {
  if (val === undefined || val === null) return '-'
  if (typeof val === 'object') return JSON.stringify(val)
  return String(val)
}

function getTxTypeLabel(type?: number, subtype?: number): string {
  if (type === undefined || type === null) return t('txType.unknown')
  const keyMap: Record<number, Record<number, string>> = {
    0: { 0: 'txType.payment' },
    1: { 0: 'txType.messaging', 1: 'txType.aliasAssignment', 2: 'txType.pollCreation', 3: 'txType.voteCasting', 4: 'txType.accountInfo' },
    2: { 0: 'txType.assetIssuance', 1: 'txType.assetTransfer', 2: 'txType.askOrder', 3: 'txType.bidOrder', 4: 'txType.assetAskCancel', 5: 'txType.assetBidCancel' },
    3: { 0: 'txType.marketListing', 1: 'txType.marketDelisting', 2: 'txType.marketPriceChange', 3: 'txType.marketQtyChange', 4: 'txType.marketPurchase', 5: 'txType.marketDelivery', 6: 'txType.marketFeedback', 7: 'txType.marketRefund' },
    4: { 0: 'txType.accountInfo', 1: 'txType.aliasAssignment', 2: 'txType.aliasSell', 3: 'txType.aliasBuy' },
    5: { 0: 'txType.accountProperty', 1: 'txType.accountPropertyDelete' },
    6: { 0: 'txType.currencyIssuance', 1: 'txType.reserveIncrease', 2: 'txType.reserveClaim', 3: 'txType.currencyTransfer', 4: 'txType.publishOffer', 5: 'txType.exchangeBuy', 6: 'txType.exchangeSell', 7: 'txType.currencyMint', 8: 'txType.currencyDelete' },
    7: { 0: 'txType.pollCreation', 1: 'txType.voteCasting' },
    8: { 0: 'txType.phasingVote' },
    20: { 0: 'txType.taggedData' }
  }
  const subKeyMap = keyMap[type ?? -1]
  if (subKeyMap) {
    const key = subKeyMap[subtype ?? 0]
    if (key) return t(key as any)
  }
  return `${t('txType.type')} ${type ?? '?'}.${subtype ?? 0}`
}

function typeColorClass(type?: number): string {
  const map: Record<number, string> = {
    0: 'type-payment', 1: 'type-message', 2: 'type-asset',
    3: 'type-market', 4: 'type-system', 5: 'type-property',
    6: 'type-currency', 7: 'type-vote', 8: 'type-phasing',
    20: 'type-data'
  }
  return map[type ?? -1] || ''
}

function getAmountClass(tx: any): string {
  if (!tx.amountNQT || tx.amountNQT === '0') return ''
  const myAccount = accountRS.value
  if (tx.recipient === myAccount || tx.recipientRS === myAccount) return 'amount-in'
  if (tx.sender === myAccount || tx.senderRS === myAccount) return 'amount-out'
  return ''
}

function getCounterparty(tx: any): string {
  const myAccount = accountRS.value
  if (tx.sender === myAccount || tx.senderRS === myAccount) {
    return tx.recipientRS || tx.recipient || '-'
  }
  return tx.senderRS || tx.sender || '-'
}

function txRowClassName({ row }: { row: any }): string {
  return row._isUnconfirmed ? 'tx-row-unconfirmed' : ''
}

function showTransactionDetail(row: NrcsTransaction | NrcsUnconfirmedTransaction) {
  selectedTransaction.value = row
  txDetailVisible.value = true
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

.filter-card {
  margin-bottom: $space-lg;
  border: 1px solid $border-subtle !important;
  border-radius: $radius-lg !important;
  background: rgba($surface-800, 0.6) !important;

  :deep(.el-card__body) {
    padding: $space-md $space-lg !important;
  }
}

.filter-pills {
  display: flex;
  flex-wrap: wrap;
  gap: $space-sm;
}

.tx-card {
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

    tr.el-table__row {
      cursor: pointer;
      &:hover > td { background: rgba($primary, 0.04) !important; }
    }
  }

  :deep(.tx-row-unconfirmed) > td {
    background: rgba($warning, 0.04) !important;
  }
}

.tx-type-badge {
  display: inline-block;
  font-size: $font-size-xs;
  font-weight: 600;
  padding: 3px 10px;
  border-radius: $radius-full;

  &.type-payment   { background: rgba(#00d4ff, 0.1); color: #5ee7ff; }
  &.type-message   { background: rgba(#8b5cf6, 0.1); color: #a78bfa; }
  &.type-asset     { background: rgba(#10b981, 0.1); color: #34d399; }
  &.type-market    { background: rgba(#f59e0b, 0.1); color: #fbbf24; }
  &.type-system    { background: rgba(#64748b, 0.15); color: #94a3b8; }
  &.type-property  { background: rgba(#ec4899, 0.1); color: #f472b6; }
  &.type-currency  { background: rgba(#06b6d4, 0.1); color: #22d3ee; }
  &.type-vote      { background: rgba(#84cc16, 0.1); color: #a3e635; }
  &.type-phasing   { background: rgba(#f97316, 0.1); color: #fb923c; }
  &.type-data      { background: rgba(#6366f1, 0.1); color: #818cf8; }
}

.tx-amount {
  font-weight: 600;
  &.amount-in  { color: $success; }
  &.amount-out { color: $danger; }
}

.text-muted {
  color: $text-muted;
}

.text-sm {
  font-size: $font-size-sm;
}

.text-mono {
  font-family: $font-mono;
}

.text-accent {
  color: $primary;
}

.pagination-container {
  display: flex;
  justify-content: center;
  margin-top: $space-lg;
}

.conf-tag {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-family: $font-mono;
  font-size: $font-size-xs;
  font-weight: 600;
  min-width: 28px;
  height: 22px;
  padding: 0 6px;
  border-radius: $radius-full;
  &--confirmed   { background: $success-subtle; color: $success; }
  &--pending     { background: $warning-subtle; color: $warning; }
  &--unconfirmed { background: rgba(100, 116, 139, 0.15); color: $text-muted; }
}

.status-icon {
  &.status-confirmed { color: $success; }
  &.status-pending   { color: $warning; }
  &.status-unconfirmed { color: $text-muted; }
}

.tx-detail {
  :deep(.el-descriptions__label) {
    font-weight: 600;
  }
}
</style>
