<template>
  <div class="nrcs-dashboard">
    <div class="dashboard-grid animate-stagger">
      <!-- Header -->
      <header class="dash-header">
        <div class="dash-header-left">
          <h1 class="dash-title">{{ t('dashboard.title') }}</h1>
          <span class="version-badge" v-if="blockchainStatus.version">v{{ blockchainStatus.version }}</span>
        </div>
        <div class="dash-header-right">
          <div class="network-status" :class="{ online: isConnected }">
            <span class="status-dot"></span>
            {{ isConnected ? t('dashboard.online') : t('dashboard.offline') }}
          </div>
          <div class="block-info" v-if="blockHeight > 0">
            <span class="block-label">{{ t('sidebar.height') }}</span>
            <span class="block-value">#{{ blockHeight.toLocaleString() }}</span>
          </div>
        </div>
      </header>

      <!-- Stat Tiles Row -->
      <div class="info-cards-row">
        <el-card class="stat-tile stat-tile--balance" shadow="hover" @click="$router.push('/dashboard')">
          <div class="stat-tile__icon">
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none">
              <circle cx="12" cy="12" r="9.5" stroke="#ff5c5c" stroke-width="1.8"/>
              <text x="12" y="16" text-anchor="middle" fill="#ff5c5c" font-size="11" font-weight="700">$</text>
            </svg>
          </div>
          <div class="stat-tile__body">
            <span class="stat-tile__label">{{ t('dashboard.accountBalance') }}</span>
            <span class="stat-tile__value text-mono">{{ accountBalance }}</span>
            <span class="stat-tile__unit">NRC</span>
          </div>
        </el-card>

        <el-card class="stat-tile stat-tile--assets" shadow="hover" @click="$router.push('/assets/my-assets')">
          <div class="stat-tile__icon">
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none">
              <rect x="3" y="3" width="18" height="18" rx="3" stroke="#22c55e" stroke-width="1.8"/>
              <line x1="8" y1="10" x2="16" y2="10" stroke="#22c55e" stroke-width="1.5"/>
              <line x1="8" y1="14" x2="12" y2="14" stroke="#22c55e" stroke-width="1.5"/>
            </svg>
          </div>
          <div class="stat-tile__body">
            <span class="stat-tile__label">{{ t('menu.assets') }}</span>
            <span class="stat-tile__value text-mono">{{ assetsCount }}</span>
            <span class="stat-tile__unit">{{ t('dashboard.owned') }}</span>
          </div>
        </el-card>

        <el-card class="stat-tile stat-tile--currency" shadow="hover" @click="$router.push('/monetary/currencies')">
          <div class="stat-tile__icon">
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none">
              <circle cx="12" cy="12" r="9" stroke="#f59e0b" stroke-width="1.8"/>
              <path d="M15 9h-3a2 2 0 100 4h1a2 2 0 010 4H9" stroke="#f59e0b" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
          </div>
          <div class="stat-tile__body">
            <span class="stat-tile__label">{{ t('menu.currencies') }}</span>
            <span class="stat-tile__value text-mono">{{ currenciesCount }}</span>
            <span class="stat-tile__unit">{{ t('dashboard.owned') }}</span>
          </div>
        </el-card>

        <el-card class="stat-tile stat-tile--messages" shadow="hover" @click="$router.push('/messages')">
          <div class="stat-tile__icon">
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none">
              <path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z" stroke="#8b5cf6" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
          </div>
          <div class="stat-tile__body">
            <span class="stat-tile__label">{{ t('dashboard.messages') }}</span>
            <span class="stat-tile__value text-mono">{{ messageCount }}</span>
            <span class="stat-tile__unit">{{ t('dashboard.total') }}</span>
          </div>
        </el-card>
      </div>

      <div class="info-cards-row">
        <el-card class="stat-tile stat-tile--aliases" shadow="hover" @click="$router.push('/aliases')">
          <div class="stat-tile__icon">
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none">
              <path d="M19 21l-7-5-7 5V5a2 2 0 012-2h10a2 2 0 012 2z" stroke="#14b8a6" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
          </div>
          <div class="stat-tile__body">
            <span class="stat-tile__label">{{ t('dashboard.aliases') }}</span>
            <span class="stat-tile__value text-mono">{{ aliasCount }}</span>
            <span class="stat-tile__unit">{{ t('dashboard.registered') }}</span>
          </div>
        </el-card>

        <el-card class="stat-tile stat-tile--height" shadow="hover" @click="$router.push('/settings/blocks')">
          <div class="stat-tile__icon">
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none">
              <rect x="4" y="6" width="16" height="13" rx="2" stroke="#3b82f6" stroke-width="1.8"/>
              <line x1="8" y1="3" x2="8" y2="6" stroke="#3b82f6" stroke-width="1.8" stroke-linecap="round"/>
              <line x1="16" y1="3" x2="16" y2="6" stroke="#3b82f6" stroke-width="1.8" stroke-linecap="round"/>
            </svg>
          </div>
          <div class="stat-tile__body">
            <span class="stat-tile__label">{{ t('sidebar.height') }}</span>
            <span class="stat-tile__value text-mono">{{ blockHeight.toLocaleString() }}</span>
            <span class="stat-tile__unit">{{ lastBlockTime }}</span>
          </div>
        </el-card>

        <el-card class="stat-tile stat-tile--peers" shadow="hover" @click="$router.push('/settings/peers')">
          <div class="stat-tile__icon">
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none">
              <circle cx="7" cy="8" r="3" stroke="#94a3b8" stroke-width="1.8"/>
              <circle cx="17" cy="8" r="3" stroke="#94a3b8" stroke-width="1.8"/>
              <path d="M2 19c0-3 2-5 5-5h1M22 19c0-3-2-5-5-5h-1" stroke="#94a3b8" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
          </div>
          <div class="stat-tile__body">
            <span class="stat-tile__label">{{ t('settings.peers') }}</span>
            <span class="stat-tile__value text-mono">{{ peerCount }}</span>
            <span class="stat-tile__unit">{{ t('dashboard.connected') }}</span>
          </div>
        </el-card>

        <el-card class="stat-tile stat-tile--polls" shadow="hover" @click="$router.push('/voting/active-polls')">
          <div class="stat-tile__icon">
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none">
              <rect x="3" y="7" width="18" height="13" rx="2" stroke="#ec4899" stroke-width="1.8"/>
              <line x1="8" y1="12" x2="16" y2="12" stroke="#ec4899" stroke-width="1.5" stroke-linecap="round"/>
              <line x1="8" y1="16" x2="13" y2="16" stroke="#ec4899" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
          </div>
          <div class="stat-tile__body">
            <span class="stat-tile__label">{{ t('dashboard.activePolls') }}</span>
            <span class="stat-tile__value text-mono">{{ pollCount }}</span>
            <span class="stat-tile__unit">{{ t('dashboard.active') }}</span>
          </div>
        </el-card>
      </div>

      <!-- Blockchain Download Progress -->
      <div v-if="blockchainStatus.isDownloading" class="download-progress glass-panel">
        <div class="download-progress__header">
          <el-icon class="is-loading"><Loading /></el-icon>
          <span>{{ t('dashboard.downloadInProgress') }}</span>
        </div>
        <el-progress
          :percentage="downloadProgress"
          :stroke-width="8"
          :show-text="true"
        />
        <span class="download-progress__feeder" v-if="blockchainStatus.lastBlockchainFeederHeight">
          {{ t('dashboard.peerHeight') }}: {{ blockchainStatus.lastBlockchainFeederHeight.toLocaleString() }} |
          {{ t('dashboard.localHeight') }}: {{ (blockchainStatus.lastBlockHeight || 0).toLocaleString() }}
        </span>
      </div>

      <!-- Recent Transactions -->
      <section class="transactions-section el-card">
        <div class="tx-section-header">
          <div class="tx-title-group">
            <h2 class="tx-section-title">{{ t('dashboard.recentTransactions') }}</h2>
          </div>
        </div>

        <div class="tx-table-wrapper">
          <el-table
            :data="recentTransactions"
            style="width: 100%"
            v-loading="isLoadingTransactions"
            :empty-text="t('common.noData')"
            :row-class-name="txRowClassName"
            @row-click="showTransactionDetail"
          >
            <el-table-column :label="t('common.date')" width="170">
              <template #default="{ row }">
                <span class="text-muted text-sm">{{ formatTimestamp(row.timestamp) }}</span>
              </template>
            </el-table-column>
            <el-table-column :label="t('common.type')" width="110">
              <template #default="{ row }">
                <span class="tx-type-badge" :class="typeColorClass(row.type)">
                  {{ getTxTypeLabel(row.type, row.subtype) }}
                </span>
              </template>
            </el-table-column>
            <el-table-column :label="t('common.amount')" width="150" align="right">
              <template #default="{ row }">
                <span class="tx-amount" :class="getAmountClass(row)">
                  {{ formatAmount(row.amountNQT) }}
                </span>
              </template>
            </el-table-column>
            <el-table-column :label="t('common.fee')" width="120" align="right">
              <template #default="{ row }">
                <span class="text-muted text-sm text-mono">{{ formatAmount(row.feeNQT) }}</span>
              </template>
            </el-table-column>
            <el-table-column :label="t('common.account')" min-width="200">
              <template #default="{ row }">
                <span class="tx-account text-mono text-accent">
                  {{ getCounterparty(row) }}
                </span>
              </template>
            </el-table-column>
            <el-table-column :label="t('dashboard.height')" width="100" align="center">
              <template #default="{ row }">
                <span class="text-muted text-sm text-mono" v-if="row.height">#{{ row.height }}</span>
                <el-tag type="warning" size="small" v-else>{{ t('dashboard.pending') }}</el-tag>
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
          </el-table>
        </div>

        <div class="tx-footer">
          <el-button type="primary" link size="large" @click="$router.push('/dashboard/transactions')">
            {{ t('dashboard.viewAllTransactions') }}
          </el-button>
          <el-divider direction="vertical" />
          <el-button type="primary" link size="large" @click="$router.push('/dashboard/ledger')">
            {{ t('dashboard.accountLedger') }}
          </el-button>
        </div>
      </section>

      <!-- Recent Blocks -->
      <section class="blocks-section el-card" v-if="recentBlocks.length > 0">
        <div class="tx-section-header">
          <h2 class="tx-section-title">{{ t('dashboard.recentBlocks') }}</h2>
        </div>
        <div class="blocks-list">
          <div
            v-for="block in recentBlocks"
            :key="block.block"
            class="block-item"
            @click="showBlockDetail(block)"
          >
            <span class="block-height text-mono text-accent">#{{ block.height.toLocaleString() }}</span>
            <span class="block-info-text">{{ block.numberOfTransactions }} tx</span>
            <span class="block-info-text text-mono">{{ formatAmount(block.totalAmountNQT) }} NRC</span>
            <span class="block-time text-muted text-sm">{{ formatTimestamp(block.timestamp) }}</span>
          </div>
        </div>
      </section>
    </div>

    <!-- Transaction Detail Dialog -->
    <el-dialog
      v-model="txDetailVisible"
      :title="t('dashboard.transactionDetails')"
      width="700px"
      destroy-on-close
    >
      <TransactionDetailPanel v-if="selectedTransaction" :transaction="selectedTransaction" />
      <el-empty v-else :description="t('dashboard.noTransactionSelected')" />
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { Loading } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { useAppStore } from '@/stores/app'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { formatTimestamp as fmtTimestamp, formatNrc, nqtToNxt } from '@/utils/format'
import type { NrcsTransaction, NrcsUnconfirmedTransaction, NrcsBlockchainStatus, NrcsBlock } from '@/api/modules/nrcs.api'
import TransactionDetailPanel from '@/components/base/TransactionDetailPanel.vue'

const router = useRouter()
const { t } = useI18n()
const appStore = useAppStore()

// --- State ---
const isLoadingTransactions = ref(false)
const isLoadingCounts = ref(false)

const accountRS = ref(localStorage.getItem('nrcs_account_rs') || '')
const accountId = ref(localStorage.getItem('nrcs_account_id') || '')

const accountBalance = ref('0.00')
const assetsCount = ref(0)
const currenciesCount = ref(0)
const messageCount = ref(0)
const aliasCount = ref(0)
const peerCount = ref(0)
const pollCount = ref(0)
const blockHeight = ref(0)
const lastBlockTime = ref('')
const blockchainStatus = ref<Partial<NrcsBlockchainStatus>>({})
const recentTransactions = ref<(NrcsTransaction | NrcsUnconfirmedTransaction)[]>([])
const recentBlocks = ref<NrcsBlock[]>([])

const txDetailVisible = ref(false)
const selectedTransaction = ref<NrcsTransaction | NrcsUnconfirmedTransaction | null>(null)

let pollTimer: number | null = null

// --- Computed ---
const isConnected = computed(() => appStore.isConnected)

const downloadProgress = computed(() => {
  const feeder = blockchainStatus.value.lastBlockchainFeederHeight || 1
  const local = blockchainStatus.value.lastBlockHeight || 1
  if (feeder <= local) return 100
  return Math.round((local / feeder) * 100)
})

// --- Lifecycle ---
onMounted(async () => {
  await loadAllData()
  startPolling()
})

onUnmounted(() => stopPolling())

// --- Data Loading ---
async function loadAllData() {
  if (!accountRS.value && !accountId.value) return
  try {
    await Promise.all([
      loadBlockchainStatus(),
      loadAccountInfo(),
      loadRecentTransactions(),
      loadRecentBlocks(),
      loadCounts()
    ])
  } catch (error) {
    console.error('Failed to load dashboard data:', error)
  }
}

async function loadBlockchainStatus() {
  try {
    const status = await nrcsApi.getBlockchainStatus()
    blockchainStatus.value = status
    if (status.lastBlockHeight) {
      blockHeight.value = status.lastBlockHeight
      appStore.blockHeight = status.lastBlockHeight
    }
    if (status.time) {
      lastBlockTime.value = formatTimestamp(status.time)
    }
    appStore.isConnected = true
  } catch {
    appStore.isConnected = false
  }
}

async function loadAccountInfo() {
  try {
    const account = await nrcsApi.getAccount(accountRS.value || accountId.value)
    const nrc = Number(nqtToNxt(account.balanceNQT || '0'))
    accountBalance.value = nrc.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })
    if (account.accountRS) {
      accountRS.value = account.accountRS
      localStorage.setItem('nrcs_account_rs', account.accountRS)
    }
    if (account.account) {
      accountId.value = account.account
      localStorage.setItem('nrcs_account_id', account.account)
    }
  } catch (error) {
    console.error('Failed to load account:', error)
  }
}

async function loadRecentTransactions() {
  try {
    isLoadingTransactions.value = true
    const account = accountRS.value || accountId.value
    if (!account) return

    const [confirmedResult, unconfirmedResult] = await Promise.allSettled([
      nrcsApi.getBlockchainTransactions(account, 0, 9),
      nrcsApi.getUnconfirmedTransactions(account)
    ])

    const confirmedTxs = confirmedResult.status === 'fulfilled'
      ? (confirmedResult.value?.transactions || [])
      : []
    const unconfirmedTxs = unconfirmedResult.status === 'fulfilled'
      ? (unconfirmedResult.value?.unconfirmedTransactions || [])
      : []

    recentTransactions.value = [
      ...unconfirmedTxs.map((tx: NrcsUnconfirmedTransaction) => ({ ...tx, confirmations: -1, height: undefined })),
      ...confirmedTxs
    ].slice(0, 10)
  } catch (error) {
    console.error('Failed to load transactions:', error)
  } finally {
    isLoadingTransactions.value = false
  }
}

async function loadRecentBlocks() {
  try {
    const result = await nrcsApi.getBlocks(0, 4, false)
    recentBlocks.value = result?.blocks || []
  } catch (error) {
    console.error('Failed to load blocks:', error)
  }
}

async function loadCounts() {
  isLoadingCounts.value = true
  const account = accountRS.value || accountId.value
  if (!account) { isLoadingCounts.value = false; return }

  try {
    const [assetsRes, currenciesRes, messagesRes, aliasesRes, peersRes, pollsRes] = await Promise.allSettled([
      nrcsApi.getAccountAssets(account),
      nrcsApi.getAccountCurrencies(account),
      nrcsApi.getAccountMessages(account, 0, 0),
      nrcsApi.getAliases(account, 0, 0),
      nrcsApi.getPeers('CONNECTED', false, true),
      nrcsApi.getPolls(0, 99)
    ])

    if (assetsRes.status === 'fulfilled') {
      assetsCount.value = (assetsRes.value?.assetBalances || []).length
    }
    if (currenciesRes.status === 'fulfilled') {
      currenciesCount.value = (currenciesRes.value?.currencyBalances || []).length
    }
    if (messagesRes.status === 'fulfilled') {
      messageCount.value = (messagesRes.value?.messages || []).length
    }
    if (aliasesRes.status === 'fulfilled') {
      aliasCount.value = (aliasesRes.value?.aliases || []).length
    }
    if (peersRes.status === 'fulfilled') {
      peerCount.value = (peersRes.value?.peers || []).length
    }
    if (pollsRes.status === 'fulfilled') {
      const now = await nrcsApi.getBlockchainStatus().catch(() => ({} as any))
      const currentHeight = now?.numberOfBlocks || 0
      pollCount.value = ((pollsRes.value as any)?.polls || []).filter((p: any) => p.finishHeight > currentHeight).length
    }
  } catch (error) {
    console.error('Failed to load counts:', error)
  } finally {
    isLoadingCounts.value = false
  }
}

// --- Polling ---
function startPolling() {
  pollTimer = window.setInterval(() => {
    Promise.allSettled([
      loadBlockchainStatus(),
      loadRecentTransactions(),
      loadRecentBlocks()
    ])
  }, 30000)
}

function stopPolling() {
  if (pollTimer) { clearInterval(pollTimer); pollTimer = null }
}

// --- Formatting ---
function formatTimestamp(nrcsTimestamp?: number): string {
  return nrcsTimestamp ? fmtTimestamp(nrcsTimestamp) : ''
}

function formatAmount(nqt?: string): string {
  return formatNrc(nqt || '0')
}

function getTxTypeLabel(type?: number, subtype?: number): string {
  if (type === undefined || type === null) return t('txType.unknown')
  const keyMap: Record<number, Record<number, string>> = {
    0: { 0: 'txType.payment' },
    1: { 0: 'txType.messaging', 1: 'txType.aliasAssignment', 2: 'txType.pollCreation', 3: 'txType.voteCasting', 4: 'txType.accountInfo', 5: 'txType.assetIssue', 6: 'txType.assetTransfer', 7: 'txType.assetAskOrder', 8: 'txType.assetBidOrder' },
    2: { 0: 'txType.assetIssuance', 1: 'txType.assetTransfer', 2: 'txType.askOrder', 3: 'txType.bidOrder', 4: 'txType.assetAskCancel', 5: 'txType.assetBidCancel' },
    3: { 0: 'txType.marketListing', 1: 'txType.marketDelisting', 2: 'txType.marketPriceChange', 3: 'txType.marketQtyChange', 4: 'txType.marketPurchase', 5: 'txType.marketDelivery', 6: 'txType.marketFeedback', 7: 'txType.marketRefund' },
    4: { 0: 'txType.accountInfo', 1: 'txType.aliasAssignment', 2: 'txType.aliasSell', 3: 'txType.aliasBuy' },
    5: { 0: 'txType.accountProperty', 1: 'txType.accountPropertyDelete' },
    6: { 0: 'txType.currencyIssuance', 1: 'txType.reserveIncrease', 2: 'txType.reserveClaim', 3: 'txType.currencyTransfer', 4: 'txType.publishOffer', 5: 'txType.exchangeBuy', 6: 'txType.exchangeSell', 7: 'txType.currencyMint', 8: 'txType.currencyDelete' },
    7: { 0: 'txType.pollCreation', 1: 'txType.voteCasting' },
    8: { 0: 'txType.phasingVote' },
    20: { 0: 'txType.taggedData' }
  }
  const key = keyMap[type]?.[subtype ?? 0]
  if (key) return t(key as any)
  return `${t('txType.type')} ${type}.${subtype ?? 0}`
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
  const myAccount = accountRS.value || accountId.value
  if (tx.recipient === myAccount || tx.recipientRS === myAccount) return 'amount-in'
  if (tx.sender === myAccount || tx.senderRS === myAccount) return 'amount-out'
  return ''
}

function getCounterparty(tx: any): string {
  const myAccount = accountRS.value || accountId.value
  if (tx.sender === myAccount || tx.senderRS === myAccount) {
    return tx.recipientRS || tx.recipient || tx.recipientId || '-'
  }
  return tx.senderRS || tx.sender || tx.senderId || '-'
}

function txRowClassName({ row }: { row: any }): string {
  return row.confirmations === -1 ? 'tx-row-unconfirmed' : ''
}

// --- Navigation ---
function showTransactionDetail(row: NrcsTransaction | NrcsUnconfirmedTransaction) {
  selectedTransaction.value = row
  txDetailVisible.value = true
}

function showBlockDetail(block: NrcsBlock) {
  router.push(`/settings/blocks?height=${block.height}`)
}
</script>

<style scoped lang="scss">
.nrcs-dashboard {
  padding: $space-xl;
  max-width: 1440px;
  margin: 0 auto;
}

.dashboard-grid {
  display: flex;
  flex-direction: column;
  gap: $space-lg;
}

// --- Header ---
.dash-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: $space-md;

  .dash-header-left {
    display: flex;
    align-items: baseline;
    gap: $space-md;
  }

  .dash-title {
    font-family: $font-display;
    font-size: $font-size-2xl;
    font-weight: 700;
    color: $text-primary;
    margin: 0;
  }

  .version-badge {
    font-size: $font-size-xs;
    font-weight: 500;
    color: $primary;
    background: $primary-subtle;
    padding: 2px 10px;
    border-radius: $radius-full;
  }

  .dash-header-right {
    display: flex;
    align-items: center;
    gap: $space-lg;
  }

  .network-status {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: $font-size-xs;
    font-weight: 600;
    color: $text-muted;
    text-transform: uppercase;

    .status-dot {
      width: 8px; height: 8px;
      border-radius: 50%;
      background: $danger;
      transition: all $duration-normal ease;
    }
  }

  .block-info {
    display: flex;
    align-items: baseline;
    gap: 6px;
    padding: 4px 12px;
    background: rgba(255, 255, 255, 0.03);
    border-radius: $radius-full;
    border: 1px solid $border-subtle;

    .block-label {
      font-size: $font-size-xs;
      color: $text-muted;
    }
    .block-value {
      font-family: $font-mono;
      font-size: $font-size-sm;
      font-weight: 600;
      color: $primary;
    }
  }
}

// --- Stat Tiles ---
.info-cards-row {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: $space-lg;
}

.stat-tile {
  cursor: pointer;
  border: 1px solid $border-subtle !important;
  background: linear-gradient(135deg, rgba($surface-700, 0.6), rgba($surface-800, 0.8)) !important;
  border-radius: $radius-lg !important;
  transition: all $duration-slow ease;

  :deep(.el-card__body) {
    display: flex;
    align-items: center;
    gap: $space-md;
    padding: $space-lg $space-xl !important;
  }

  &:hover {
    transform: translateY(-2px);
    border-color: $border-glow !important;
  }

  .stat-tile__icon {
    flex-shrink: 0;
    width: 44px; height: 44px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: $radius-md;
    background: rgba($primary, 0.06);
  }

  .stat-tile__body {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .stat-tile__label {
    font-size: $font-size-xs;
    font-weight: 500;
    color: $text-muted;
    text-transform: uppercase;
  }

  .stat-tile__value {
    font-family: $font-display;
    font-size: $font-size-xl;
    font-weight: 700;
    color: $text-primary;
    line-height: 1.2;
  }

  .stat-tile__unit {
    font-size: $font-size-xs;
    color: $text-muted;
  }
}

// --- Download Progress ---
.download-progress {
  padding: $space-lg $space-xl;
  display: flex;
  flex-direction: column;
  gap: $space-md;

  &__header {
    display: flex;
    align-items: center;
    gap: $space-sm;
    font-size: $font-size-sm;
    font-weight: 600;
    color: $warning;
  }

  &__feeder {
    font-size: $font-size-xs;
    color: $text-muted;
    font-family: $font-mono;
  }
}

// --- Transactions Section ---
.transactions-section {
  border: 1px solid $border-subtle !important;
  border-radius: $radius-lg !important;
  overflow: hidden;
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
      color: $text-muted !important;
      font-weight: 600;
      font-size: $font-size-xs;
      text-transform: uppercase;
      padding: $space-md $space-lg;
      border-bottom: 1px solid $border-subtle;
    }

    td.el-table__cell {
      padding: $space-md $space-lg;
      border-bottom: 1px solid rgba($border-default, 0.5);
    }

    tr.el-table__row {
      cursor: pointer;
      &:hover > td { background: rgba($primary, 0.04) !important; }
    }

    .el-loading-mask {
      background-color: rgba($bg, 0.9) !important;
    }
  }
}

.tx-section-header {
  padding: $space-xl $space-xl 0;
  .tx-section-title {
    font-family: $font-display;
    font-size: $font-size-lg;
    font-weight: 600;
    color: $text-primary;
    margin: 0;
  }
}

.tx-table-wrapper {
  padding: $space-lg $space-xl 0;
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
  font-family: $font-mono;
  font-weight: 600;
  font-size: $font-size-sm;
  &.amount-in  { color: $success; }
  &.amount-out { color: $danger; }
}

.tx-account {
  font-size: $font-size-sm;
  font-family: $font-mono;
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
  &--confirmed   { background: $success-glow; color: $success; }
  &--pending     { background: $warning-glow; color: $warning; }
  &--unconfirmed { background: rgba(100, 116, 139, 0.15); color: $text-muted; }
}

.tx-footer {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: $space-md;
  padding: $space-xl;
  border-top: 1px solid $border-subtle;
  background: rgba(255, 255, 255, 0.015);
}

:deep(.tx-row-unconfirmed) > td {
  background: rgba($warning, 0.04) !important;
}

// --- Recent Blocks ---
.blocks-section {
  border: 1px solid $border-subtle !important;
  border-radius: $radius-lg !important;
  background: linear-gradient(180deg, rgba($surface-800, 0.8), rgba($surface-900, 0.9)) !important;

  :deep(.el-card__body) {
    background: transparent !important;
  }
}

.blocks-list {
  padding: 0 $space-xl $space-xl;
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.block-item {
  display: flex;
  align-items: center;
  gap: $space-lg;
  padding: $space-md $space-lg;
  border-radius: $radius-sm;
  cursor: pointer;
  transition: background $duration-fast ease;
  border: 1px solid transparent;

  &:hover {
    background: rgba($primary, 0.04);
    border-color: rgba($primary, 0.1);
  }

  .block-height {
    font-weight: 600;
    min-width: 80px;
  }

  .block-info-text {
    font-size: $font-size-sm;
    color: $text-secondary;
  }

  .block-time {
    margin-left: auto;
  }
}

// --- Responsive ---
@media (max-width: 1280px) {
  .info-cards-row { grid-template-columns: repeat(2, 1fr); }
}
@media (max-width: 768px) {
  .nrcs-dashboard { padding: $space-md; }
  .info-cards-row { grid-template-columns: 1fr; }
  .dash-header { flex-direction: column; align-items: flex-start; }
}
</style>
