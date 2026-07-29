<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title">
        <el-icon><Checked /></el-icon>
        {{ t('dashboard.approvalRequests') }}
      </h2>
      <div class="header-actions">
        <el-button size="small" @click="refreshData">
          <el-icon><Refresh /></el-icon>
          {{ t('common.refresh') }}
        </el-button>
      </div>
    </div>

    <el-card shadow="hover" v-loading="loading" class="approval-card">
      <el-table
        :data="items"
        stripe
        style="width: 100%"
        :empty-text="t('common.noData')"
        row-key="transaction"
      >
        <el-table-column prop="transaction" :label="t('common.transaction')" width="180" show-overflow-tooltip>
          <template #default="{ row }">
            <span class="text-mono text-sm">{{ truncateHash(row.transaction || '', 6) }}</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.type')" width="140">
          <template #default="{ row }">
            <span class="tx-type-badge" :class="typeColorClass(row.type)">
              {{ getTxTypeLabel(row.type, row.subtype) }}
            </span>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.amount')" width="150" align="right">
          <template #default="{ row }">
            <span class="text-mono" :class="getAmountClass(row)">
              {{ formatNrcAmount(row.amountNQT) }}
            </span>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.sender')" width="200" show-overflow-tooltip>
          <template #default="{ row }">
            <span class="text-mono text-accent">{{ row.senderRS || row.sender || '-' }}</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('dashboard.deadline')" width="170">
          <template #default="{ row }">
            <span v-if="row.phasingFinishHeight" class="text-muted text-sm">
              {{ t('dashboard.block') }} #{{ row.phasingFinishHeight }}
            </span>
            <span v-else class="text-muted text-sm">-</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.actions')" width="160" fixed="right">
          <template #default="{ row }">
            <el-button
              size="small"
              type="primary"
              :disabled="approvingTx === row.transaction"
              :loading="approvingTx === row.transaction"
              @click="handleApprove(row)"
            >
              {{ t('dashboard.approve') }}
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <el-dialog
      v-model="approveDialogVisible"
      :title="t('dashboard.approveTransaction')"
      width="420px"
      destroy-on-close
    >
      <el-form label-width="120px">
        <el-form-item :label="t('common.transaction')">
          <span class="text-mono text-sm">{{ truncateHash(approvingTransaction?.transaction || '', 8) }}</span>
        </el-form-item>
        <el-form-item :label="t('dashboard.secretPhrase')">
          <el-input
            v-model="approveSecretPhrase"
            type="password"
            :placeholder="t('dashboard.enterSecretPhrase')"
            show-password
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="approveDialogVisible = false">{{ t('common.cancel') }}</el-button>
        <el-button type="primary" :loading="approvingTx !== null" @click="confirmApprove">
          {{ t('common.confirm') }}
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { Checked, Refresh } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { formatAmount, truncateHash } from '@/utils/format'
import { useAccountStore } from '@/stores/modules/account.store'

const { t } = useI18n()
const accountStore = useAccountStore()

const loading = ref(false)
const items = ref<any[]>([])
const accountRS = ref(accountStore.accountRS || localStorage.getItem('nrcs_account_rs') || '')

const approveDialogVisible = ref(false)
const approvingTransaction = ref<any>(null)
const approvingTx = ref<string | null>(null)
const approveSecretPhrase = ref('')

onMounted(() => {
  refreshData()
})

async function refreshData() {
  loading.value = true
  try {
    const acct = accountRS.value
    if (!acct) {
      ElMessage.warning(t('common.noAccount'))
      return
    }
    const result = await nrcsApi.getAccountPhasedTransactions(acct)
    const list = (result as any).transactions || []
    items.value = list
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  } finally {
    loading.value = false
  }
}

function handleApprove(row: any) {
  approvingTransaction.value = row
  approveSecretPhrase.value = accountStore.secretPhrase || localStorage.getItem('nrcs-secretPhrase') || ''
  approveDialogVisible.value = true
}

async function confirmApprove() {
  if (!approveSecretPhrase.value) {
    ElMessage.warning(t('dashboard.enterSecretPhrase'))
    return
  }
  const tx = approvingTransaction.value
  if (!tx?.transaction) return

  approvingTx.value = tx.transaction
  try {
    await nrcsApi.approveTransaction({
      secretPhrase: approveSecretPhrase.value,
      transaction: tx.transaction,
      feeNQT: '100000000',
      deadline: 1440
    })
    ElMessage.success(t('common.operationSuccess'))
    approveDialogVisible.value = false
    refreshData()
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  } finally {
    approvingTx.value = null
  }
}

function formatNrcAmount(nqt?: string): string {
  if (!nqt || nqt === '0') return '0'
  return formatAmount(nqt)
}

function getTxTypeLabel(type?: number, subtype?: number): string {
  if (type === undefined || type === null) return t('txType.unknown')
  const keyMap: Record<number, Record<number, string>> = {
    0: { 0: 'txType.payment' },
    1: { 0: 'txType.messaging', 1: 'txType.aliasAssignment', 2: 'txType.pollCreation', 3: 'txType.voteCasting', 4: 'txType.accountInfo' },
    2: { 0: 'txType.assetIssuance', 1: 'txType.assetTransfer', 2: 'txType.askOrder', 3: 'txType.bidOrder' },
    3: { 0: 'txType.marketListing', 4: 'txType.marketPurchase' },
    5: { 0: 'txType.currencyIssuance', 3: 'txType.currencyTransfer' },
    7: { 0: 'txType.pollCreation', 1: 'txType.voteCasting' },
    8: { 0: 'txType.phasingVote' }
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
    6: 'type-currency', 7: 'type-vote', 8: 'type-phasing'
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

.approval-card {
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
}

.amount-in  { color: $success; font-weight: 600; }
.amount-out { color: $danger; font-weight: 600; }

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
</style>
