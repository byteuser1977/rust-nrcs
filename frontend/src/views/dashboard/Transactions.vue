<template>
  <!--
    Transactions —— 我的交易页面（完整版）。
    对标 nrs.transactions.js（1118 行）。

    功能：
      1. 交易列表（确认 + 未确认合并 + 去重 + 排序）
      2. 类型导航过滤（all/phased/payment/messaging/asset/marketplace/currency/voting/data）
      3. Phased 交易状态徽章（pending/approved/rejected）+ 审批按钮
      4. 交易详情（复用 TransactionDetailPanel 组件，支持附件/加密消息/phasing 详情）
      5. 分页
  -->
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title">
        <el-icon><List /></el-icon>
        {{ t('dashboard.transactions') }}
      </h2>
      <div class="header-actions">
        <el-badge :value="phasedCount" :hidden="phasedCount === 0" type="warning">
          <el-button
            :type="activeFilter === 'phased' ? 'primary' : 'default'"
            size="small"
            @click="setFilter('phased')"
          >
            <el-icon><Finished /></el-icon>
            {{ t('transaction.phased') }}
          </el-button>
        </el-badge>
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
        :data="displayedTransactions"
        style="width: 100%"
        :empty-text="emptyText"
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
        <!-- Phasing 状态列（对标 nrs.transactions.js:272 addPhasedTransactionHTML） -->
        <el-table-column :label="t('transaction.phasingStatus')" width="120" align="center">
          <template #default="{ row }">
            <el-tag
              v-if="isPhased(row)"
              :type="phasingTagType(row)"
              size="small"
              effect="dark"
            >
              {{ phasingStatusLabel(row) }}
            </el-tag>
            <span v-else class="text-muted text-sm">-</span>
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
        <el-table-column :label="t('common.actions')" width="120" align="center" fixed="right">
          <template #default="{ row }">
            <!-- 审批按钮（对标 nrs.transactions.js:292-312 approve_transaction_btn） -->
            <el-button
              v-if="isPhased(row) && canApprove(row)"
              size="small"
              type="success"
              text
              @click.stop="approveTransaction(row)"
            >
              <el-icon><Check /></el-icon>
              {{ t('transaction.approve') }}
            </el-button>
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

    <!-- 交易详情弹窗（复用 TransactionDetailPanel） -->
    <el-dialog
      v-model="txDetailVisible"
      :title="t('dashboard.transactionDetails')"
      width="800px"
      destroy-on-close
      class="nrcs-modal"
    >
      <TransactionDetailPanel
        :transaction="selectedTransaction"
        @approve-transaction="onApproveFromDetail"
        @send-money="onSendMoney"
        @send-message="onSendMessage"
        @add-contact="onAddContact"
      />
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
/**
 * Transactions 组件 —— 我的交易页面（完整版）。
 *
 * 对标 nrs.transactions.js（1118 行）的核心功能：
 *   - handleIncomingTransactions：确认 + 未确认交易合并 + 去重 + 排序
 *   - buildTransactionsTypeNavi：类型导航过滤
 *   - addPhasingInfoToTransactionRows + addPhasedTransactionHTML：phased 交易状态展示
 *   - displayPhasedTransactions：phased 交易过滤视图
 *   - updateApprovalRequests：审批请求计数
 *
 * phasing 状态来源：对 phased 交易调用 getPhasingPoll API 获取 result（0=pending/1=approved/2=rejected）。
 */
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { List, Refresh, Finished, Check } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import type { NrcsTransaction, NrcsUnconfirmedTransaction } from '@/api/modules/nrcs.api'
import { usePagination } from '@/composables/usePagination'
import { formatTimestamp, formatAmount } from '@/utils/format'
import { useAccountStore } from '@/stores/modules/account.store'
import TransactionDetailPanel from '@/components/base/TransactionDetailPanel.vue'

const { t } = useI18n()
const accountStore = useAccountStore()

/** 全部交易（确认 + 未确认合并） */
const transactions = ref<(NrcsTransaction | NrcsUnconfirmedTransaction)[]>([])
/** 分页 */
const pagination = usePagination(15)
/** 当前激活的过滤器 */
const activeFilter = ref('all')
/** 交易详情弹窗可见性 */
const txDetailVisible = ref(false)
/** 选中的交易 */
const selectedTransaction = ref<NrcsTransaction | NrcsUnconfirmedTransaction | null>(null)

/** phased 交易的轮询状态缓存：transactionId → poll result */
const phasingPolls = ref<Record<string, { result: number; yesVotes: number; noVotes: number }>>({})

/** 当前账户已投票的 phased 交易集合 */
const votedTransactions = ref<Set<string>>(new Set())

/** phased 交易数量（用于徽章） */
const phasedCount = computed(() =>
  transactions.value.filter((tx: any) => isPhased(tx)).length
)

const accountRS = computed(() => accountStore.accountRS || localStorage.getItem('nrcs_account_rs') || '')

/** 类型过滤器（对标 buildTransactionsTypeNavi） */
const typeFilters = computed(() => [
  { value: 'all', label: t('common.all') },
  { value: '0', label: t('txType.payment') || 'Payment' },
  { value: '1', label: t('txType.messaging') || 'Messaging' },
  { value: '2', label: t('txType.coloredCoins') || 'Asset' },
  { value: '3', label: t('txType.marketplace') || 'Marketplace' },
  { value: '5', label: t('txType.accountProperty') || 'Property' },
  { value: '6', label: t('txType.monetarySystem') || 'Currency' },
  { value: '7', label: t('txType.voting') || 'Voting' },
  { value: '20', label: t('txType.taggedData') || 'Data Cloud' },
])

/** 空状态文本 */
const emptyText = computed(() => {
  if (activeFilter.value === 'phased') return t('transaction.noPhasedTransactions')
  return t('common.noData')
})

/** 根据过滤器筛选显示的交易 */
const displayedTransactions = computed(() => {
  if (activeFilter.value === 'all') return transactions.value
  if (activeFilter.value === 'phased') {
    return transactions.value.filter((tx: any) => isPhased(tx))
  }
  const typeNum = Number(activeFilter.value)
  return transactions.value.filter((tx: any) => tx.type === typeNum)
})

onMounted(() => {
  refreshData()
})

function setFilter(value: string) {
  activeFilter.value = value
  if (value !== 'phased') {
    pagination.reset()
    refreshData()
  }
}

function handlePageChange(page: number) {
  pagination.goToPage(page)
  refreshData()
}

/**
 * 刷新数据：拉取确认 + 未确认交易（对标 handleIncomingTransactions）。
 * 合并、去重、排序，并为 phased 交易拉取轮询状态。
 */
async function refreshData(): Promise<void> {
  pagination.isLoading.value = true
  try {
    const acct = accountRS.value
    if (!acct) {
      ElMessage.warning(t('common.noAccount'))
      return
    }

    const typeParam = activeFilter.value !== 'all' && activeFilter.value !== 'phased'
      ? Number(activeFilter.value)
      : undefined

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

    // 合并 + 标记未确认（对标 handleIncomingTransactions）
    const combined: any[] = [
      ...(unconfirmedTxs.map((tx: any) => ({
        ...tx,
        confirmations: -1,
        height: undefined,
        _isUnconfirmed: true
      }))),
      ...confirmedTxs
    ]

    // 去重（按 transaction ID）
    const seen = new Set<string>()
    const deduped = combined.filter((tx) => {
      const id = tx.transaction
      if (id && seen.has(id)) return false
      if (id) seen.add(id)
      return true
    })

    transactions.value = deduped
    pagination.setTotalFromList(confirmedTxs.length)

    // 为 phased 交易拉取轮询状态（对标 addPhasingInfoToTransactionRows）
    await loadPhasingPolls(deduped)
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  } finally {
    pagination.isLoading.value = false
  }
}

/**
 * 为 phased 交易批量拉取轮询状态（对标 addPhasingInfoToTransactionRows）。
 * 仅对包含 phasing 附件的交易调用 getPhasingPoll。
 */
async function loadPhasingPolls(txs: any[]): Promise<void> {
  const phasedTxIds: string[] = []
  for (const tx of txs) {
    if (isPhased(tx) && tx.transaction) {
      phasedTxIds.push(tx.transaction)
    }
  }

  if (phasedTxIds.length === 0) return

  // 并行拉取所有 phased 交易的轮询状态
  const promises = phasedTxIds.map(async (txId) => {
    try {
      const [pollResult, voteResult] = await Promise.allSettled([
        nrcsApi.getPhasingPoll(txId, true),
        nrcsApi.getPhasingPollVote(txId, accountRS.value),
      ])

      if (pollResult.status === 'fulfilled' && pollResult.value?.transaction) {
        phasingPolls.value[txId] = {
          result: pollResult.value.result || 0,
          yesVotes: pollResult.value.yesVotes || 0,
          noVotes: pollResult.value.noVotes || 0,
        }
      }

      // 如果当前账户已投票，记录到 votedTransactions
      if (voteResult.status === 'fulfilled' && voteResult.value?.transaction) {
        votedTransactions.value.add(txId)
      }
    } catch {
      // 忽略单个交易的状态拉取失败
    }
  })

  await Promise.allSettled(promises)
}

/**
 * 判断交易是否为 phased 交易（对标 nrs.transactions.js:277 附件检测）。
 */
function isPhased(tx: any): boolean {
  return !!(
    tx.attachment &&
    tx.attachment['version.Phasing'] &&
    tx.attachment.phasingVotingModel !== undefined
  )
}

/**
 * 获取 phased 交易的状态标签类型。
 */
function phasingTagType(tx: any): 'info' | 'success' | 'danger' | 'warning' {
  const poll = tx.transaction ? phasingPolls.value[tx.transaction] : undefined
  if (!poll) return 'info'
  // result: 0=pending, 1=approved, 2=rejected
  if (poll.result === 1) return 'success'
  if (poll.result === 2) return 'danger'
  return 'warning'
}

/**
 * 获取 phased 交易的状态标签文本。
 */
function phasingStatusLabel(tx: any): string {
  const poll = tx.transaction ? phasingPolls.value[tx.transaction] : undefined
  if (!poll) return t('transaction.phasingPending')
  if (poll.result === 1) return t('transaction.phasingApproved')
  if (poll.result === 2) return t('transaction.phasingRejected')
  return t('transaction.phasingPending')
}

/**
 * 判断当前账户是否可以审批该 phased 交易。
 * 条件：phased 交易 + 轮询存在 + result 为 pending + 当前账户未投票。
 */
function canApprove(tx: any): boolean {
  if (!isPhased(tx) || !tx.transaction) return false
  if (votedTransactions.value.has(tx.transaction)) return false
  const poll = phasingPolls.value[tx.transaction]
  if (!poll) return false
  return poll.result === 0 // pending
}

function formatNrcAmount(nqt?: string): string {
  if (!nqt || nqt === '0') return '0'
  return formatAmount(nqt)
}

function getTxTypeLabel(type?: number, subtype?: number): string {
  if (type === undefined || type === null) return t('txType.unknown')
  const keyMap: Record<number, Record<number, string>> = {
    0: { 0: 'txType.ordinaryPayment' },
    1: { 0: 'txType.arbitraryMessage', 1: 'txType.aliasAssignment', 2: 'txType.pollCreation', 3: 'txType.voteCasting', 4: 'txType.accountInfo', 5: 'txType.aliasSell', 6: 'txType.aliasBuy', 7: 'txType.aliasDeletion', 8: 'txType.aliasTransfer' },
    2: { 0: 'txType.assetIssuance', 1: 'txType.assetTransfer', 2: 'txType.askOrderPlacement', 3: 'txType.bidOrderPlacement', 4: 'txType.askOrderCancellation', 5: 'txType.bidOrderCancellation', 6: 'txType.dividendPayment' },
    3: { 0: 'txType.digitalGoodsListing', 1: 'txType.digitalGoodsDelisting', 2: 'txType.digitalGoodsPriceChange', 3: 'txType.digitalGoodsQuantityChange', 4: 'txType.digitalGoodsPurchase', 5: 'txType.digitalGoodsDelivery', 6: 'txType.digitalGoodsFeedback', 7: 'txType.digitalGoodsRefund' },
    4: { 0: 'txType.accountControlBalanceLeasing', 1: 'txType.accountControlPhasingOnly' },
    5: { 0: 'txType.setAccountProperty', 1: 'txType.deleteAccountProperty' },
    6: { 0: 'txType.currencyIssuance', 1: 'txType.reserveIncrease', 2: 'txType.reserveClaim', 3: 'txType.currencyTransfer', 4: 'txType.publishExchangeOffer', 5: 'txType.currencyBuy', 6: 'txType.currencySell', 7: 'txType.currencyMint', 8: 'txType.currencyDeletion' },
    7: { 0: 'txType.pollCreation', 1: 'txType.voteCasting' },
    8: { 0: 'txType.phasingVoteCasting' },
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
  const classes: string[] = []
  if (row._isUnconfirmed) classes.push('tx-row-unconfirmed')
  if (isPhased(row)) classes.push('tx-row-phased')
  return classes.join(' ')
}

/**
 * 点击交易行 → 展示详情（对标 showTransactionModal）。
 */
function showTransactionDetail(row: NrcsTransaction | NrcsUnconfirmedTransaction): void {
  selectedTransaction.value = row
  txDetailVisible.value = true
}

/**
 * 审批 phased 交易（对标 approve_transaction_btn 点击）。
 */
async function approveTransaction(tx: any): Promise<void> {
  if (!tx.transaction) return
  const secretPhrase = accountStore.secretPhrase
  if (!secretPhrase) {
    ElMessage.warning(t('common.secretPhraseRequired'))
    return
  }

  try {
    await nrcsApi.approveTransaction({
      secretPhrase,
      transaction: tx.transaction,
      feeNQT: '10000000', // 0.1 NRC default fee
      deadline: 1440,
    })
    ElMessage.success(t('transaction.approveSuccess'))
    votedTransactions.value.add(tx.transaction)
    // 刷新 phasing 状态
    refreshData()
  } catch (e: any) {
    ElMessage.error(e?.message || t('transaction.approveError'))
  }
}

/**
 * TransactionDetailPanel 审批交易回调。
 */
function onApproveFromDetail(transactionId: string, _fullHash: string): void {
  const tx = transactions.value.find((t: any) => t.transaction === transactionId)
  if (tx) {
    approveTransaction(tx)
  }
}

function onSendMoney(recipient: string): void {
  // 导航到发送资金页面/弹窗
  console.log('[Transactions] sendMoney:', recipient)
}

function onSendMessage(recipient: string): void {
  // 导航到发送消息页面/弹窗
  console.log('[Transactions] sendMessage:', recipient)
}

function onAddContact(account: string): void {
  // 导航到联系人页面
  console.log('[Transactions] addContact:', account)
}

// 当过滤器变化且为 phased 时，不需要重新拉取（已在 transactions 中过滤）
watch(activeFilter, (val) => {
  if (val === 'phased') {
    // phased 过滤仅在前端进行，不重新拉取
  }
})
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
      align-items: center;
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

  :deep(.tx-row-phased) > td {
    border-left: 3px solid rgba($warning, 0.5);
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
</style>
