<template>
  <div class="nrcs-dashboard">
    <div class="dashboard-grid animate-stagger">
      <!-- 页面标题区 -->
      <header class="dash-header">
        <div class="dash-header-left">
          <h1 class="dash-title">{{ t('dashboard.title') }}</h1>
          <span class="version-badge" v-if="blockchainStatus.version">v{{ blockchainStatus.version }}</span>
        </div>
        <div class="dash-header-right">
          <div class="network-status" :class="{ online: isConnected }">
            <span class="status-dot"></span>
            {{ isConnected ? 'Online' : 'Offline' }}
          </div>
          <div class="block-info" v-if="blockHeight > 0">
            <span class="block-label">Height</span>
            <span class="block-value">#{{ blockHeight.toLocaleString() }}</span>
          </div>
        </div>
      </header>

      <!-- 第一行: 核心信息卡片 -->
      <div class="info-cards-row">
        <div class="stat-card stat-card--balance" @click="showAccountDetails">
          <div class="stat-card__glow"></div>
          <div class="stat-card__icon-wrap">
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none"><path d="M21 7.5V18a2 2 0 01-2 2H5a2 2 0 01-2-2V6a2 2 0 012-2h11l4 3.5z" stroke="#00d4ff" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/></svg>
          </div>
          <div class="stat-card__body">
            <span class="stat-card__label">Balance</span>
            <span class="stat-card__value text-mono">{{ accountBalance }}</span>
            <span class="stat-card__unit">NRC</span>
          </div>
          <div class="stat-card__trend">
            <svg width="14" height="14" viewBox="0 0 16 16" fill="#10b981"><path d="M8 12l-6-6h12z" opacity=".6"/><path d="M8 4l6 6H2z"/></svg>
          </div>
        </div>

        <div class="stat-card stat-card--assets" @click="navigateTo('/assets/my-assets')">
          <div class="stat-card__glow"></div>
          <div class="stat-card__icon-wrap">
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none"><path d="M23 6l-9.5 9.5-5-5L1 18" stroke="#10b981" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/><path d="M17 6h6v6" stroke="#10b981" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/></svg>
          </div>
          <div class="stat-card__body">
            <span class="stat-card__label">Assets Value</span>
            <span class="stat-card__value text-mono">{{ assetsValue }}</span>
            <span class="stat-card__unit">NRC · {{ assetsCount }} assets</span>
          </div>
        </div>

        <div class="stat-card stat-card--currency" @click="navigateTo('/monetary/currencies')">
          <div class="stat-card__glow"></div>
          <div class="stat-card__icon-wrap">
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none"><circle cx="12" cy="12" r="9" stroke="#f59e0b" stroke-width="1.8"/><path d="M15 9h-3a2 2 0 100 4h0a2 2 0 110 4H9" stroke="#f59e0b" stroke-width="1.8" stroke-linecap="round"/></svg>
          </div>
          <div class="stat-card__body">
            <span class="stat-card__label">Currencies</span>
            <span class="stat-card__value text-mono">{{ currenciesValue }}</span>
            <span class="stat-card__unit">NRC · {{ currenciesCount }} currencies</span>
          </div>
        </div>

        <div class="stat-card stat-card--marketplace" @click="navigateTo('/marketplace/purchased')">
          <div class="stat-card__glow"></div>
          <div class="stat-card__icon-wrap">
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none"><path d="M6 2L3 6v14a2 2 0 002 2h14a2 2 0 002-2V6l-3-4zM3 6h18M16 10a4 4 0 01-8 0" stroke="#8b5cf6" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/></svg>
          </div>
          <div class="stat-card__body">
            <span class="stat-card__label">Purchased</span>
            <span class="stat-card__value">{{ purchasedProducts }}</span>
            <span class="stat-card__unit">products</span>
          </div>
        </div>
      </div>

      <!-- 第二行: 可选信息卡片 -->
      <div class="info-cards-row info-cards-row--secondary" v-if="showOptionalCards">
        <div class="stat-card stat-card--compact" @click="navigateTo('/marketplace/pending-orders')">
          <div class="stat-card__icon-wrap icon-sm">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none"><rect x="3" y="4" width="18" height="16" rx="2" stroke="#94a3b8" stroke-width="1.8"/><line x1="7" y1="8" x2="17" y2="8" stroke="#94a3b8" stroke-width="1.8"/><line x1="7" y1="12" x2="13" y2="12" stroke="#94a3b8" stroke-width="1.8"/></svg>
          </div>
          <div class="stat-card__body">
            <span class="stat-card__value">{{ pendingSales }} / {{ completedSales }}</span>
            <span class="stat-card__label">Orders (pending / done)</span>
          </div>
        </div>

        <div class="stat-card stat-card--compact" @click="navigateTo('/messages')">
          <div class="stat-card__icon-wrap icon-sm">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none"><path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z" stroke="#94a3b8" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/></svg>
          </div>
          <div class="stat-card__body">
            <span class="stat-card__value">{{ messageCount }}</span>
            <span class="stat-card__label">Messages</span>
          </div>
        </div>

        <div class="stat-card stat-card--compact" @click="navigateTo('/aliases')">
          <div class="stat-card__icon-wrap icon-sm">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none"><path d="M19 21l-7-5-7 5V5a2 2 0 012-2h10a2 2 0 012 2z" stroke="#94a3b8" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/></svg>
          </div>
          <div class="stat-card__body">
            <span class="stat-card__value">{{ aliasCount }}</span>
            <span class="stat-card__label">Aliases</span>
          </div>
        </div>

        <div class="stat-card stat-card--compact stat-card--time" @click="navigateTo('/settings/blocks')">
          <div class="stat-card__icon-wrap icon-sm">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none"><circle cx="12" cy="12" r="10" stroke="#94a3b8" stroke-width="1.8"/><polyline points="12,6 12,12 16,14" stroke="#94a3b8" stroke-width="1.8" stroke-linecap="round"/></svg>
          </div>
          <div class="stat-card__body">
            <span class="stat-card__value text-mono">{{ lastBlockTime }}</span>
            <span class="stat-card__label">Last Updated · #{{ blockHeight }}</span>
          </div>
        </div>
      </div>

      <!-- 最近交易列表 -->
      <section class="transactions-section el-card">
        <div class="tx-section-header">
          <div class="tx-title-group">
            <div class="tx-icon-ring">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none"><polyline points="22,12 18,12 15,21 9,3 6,12 2,12" stroke="#00d4ff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>
            </div>
            <div>
              <h2 class="tx-section-title">{{ t('dashboard.recentTransactions') }}</h2>
              <p class="tx-section-subtitle">Latest blockchain activity for your account</p>
            </div>
          </div>
        </div>

        <div class="tx-table-wrapper">
          <el-table :data="recentTransactions" style="width: 100%" v-loading="isLoadingTransactions"
                     :empty-text="t('common.noData')" :row-class-name="txRowClassName"
                     element-loading-background="rgba(18, 20, 26, 0.85)">
            <el-table-column :label="t('dashboard.date')" width="110">
              <template #default="{ row }">
                <span class="text-muted text-sm">{{ formatTimestamp(row.timestamp) }}</span>
              </template>
            </el-table-column>
            <el-table-column width="40" align="center">
              <template #default="{ row }">
                <span class="msg-indicator" :class="{ active: hasAttachment(row) }">
                  <svg width="13" height="13" viewBox="0 0 24 24" fill="none"><path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z" stroke="currentColor" stroke-width="2"/><polyline points="22,6 12,13 2,6" stroke="currentColor" stroke-width="2"/></svg>
                </span>
              </template>
            </el-table-column>
            <el-table-column :label="t('dashboard.type')" width="100">
              <template #default="{ row }">
                <span class="tx-type-badge" :class="typeColorClass(row.type)">
                  {{ getTransactionType(row.type, row.subtype) }}
                </span>
              </template>
            </el-table-column>
            <el-table-column :label="t('dashboard.amount')" width="140" align="right">
              <template #default="{ row }">
                <span class="tx-amount" :class="getAmountClass(row)" dir="ltr">
                  {{ formatAmount(row.amountNQT) }}
                </span>
              </template>
            </el-table-column>
            <el-table-column :label="t('dashboard.fee')" width="100" align="right">
              <template #default="{ row }">
                <span class="tx-fee text-muted text-sm text-mono">{{ formatAmount(row.feeNQT) }}</span>
              </template>
            </el-table-column>
            <el-table-column :label="t('dashboard.account')" min-width="200">
              <template #default="{ row }">
                <span class="tx-account text-mono clickable text-accent" @click="goToAccount(row)">
                  {{ getCounterparty(row) }}
                </span>
              </template>
            </el-table-column>
            <el-table-column width="40" align="center">
              <template #default="{ row }">
                <span class="phasing-dot" v-if="row.phasing" title="Phasing">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none"><gavel stroke="#f59e0b" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>
                </span>
              </template>
            </el-table-column>
            <el-table-column :label="t('dashboard.height')" width="90" align="center">
              <template #default="{ row }">
                <span class="text-muted text-sm text-mono" v-if="row.height">#{{ row.height }}</span>
                <span class="text-muted text-sm" v-else>—</span>
              </template>
            </el-table-column>
            <el-table-column :label="t('dashboard.confirmations')" width="90" align="center">
              <template #default="{ row }">
                <span v-if="row.confirmations !== undefined && row.confirmations >= 0"
                      class="conf-tag" :class="row.confirmations > 10 ? 'conf-tag--confirmed' : 'conf-tag--pending'">
                  {{ row.confirmations }}
                </span>
                <span v-else class="conf-tag conf-tag--unconfirmed">—</span>
              </template>
            </el-table-column>
          </el-table>
        </div>

        <div class="tx-footer">
          <el-button type="primary" link size="large" @click="navigateTo('/dashboard/transactions')">
            View All Transactions →
          </el-button>
          <el-divider direction="vertical" />
          <el-button type="primary" link size="large" @click="navigateTo('/dashboard/ledger')">
            Account Ledger →
          </el-button>
        </div>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useAppStore } from '@/stores/app'
import { nrcsApi } from '@/api/modules/nrcs.api'
import type { NrcsTransaction, NrcsUnconfirmedTransaction, NrcsBlockchainStatus } from '@/api/modules/nrcs.api'

const router = useRouter()
const { t } = useI18n()
const appStore = useAppStore()

const isLoadingTransactions = ref(false)
const dashboardMessage = ref('')

const accountRS = ref(localStorage.getItem('nrcs_account_rs') || '')
const accountId = ref(localStorage.getItem('nrcs_account_id') || '')

const accountBalance = ref('0.00')
const assetsValue = ref('0.00')
const assetsCount = ref(0)
const currenciesValue = ref('0.00')
const currenciesCount = ref(0)
const purchasedProducts = ref(0)
const pendingSales = ref(0)
const completedSales = ref(0)
const messageCount = ref(0)
const aliasCount = ref(0)
const blockHeight = ref(0)
const lastBlockTime = ref('')
const blockchainStatus = ref<Partial<NrcsBlockchainStatus>>({})
const recentTransactions = ref<any[]>([])
let pollTimer: number | null = null

const isConnected = computed(() => appStore.isConnected)

const showOptionalCards = computed(() => {
  return assetsCount.value > 0 || currenciesCount.value > 0 || purchasedProducts.value > 0 || messageCount.value > 0 || aliasCount.value > 0
})

onMounted(async () => {
  await loadDashboardData()
  startPolling()
})

onUnmounted(() => stopPolling())

async function loadDashboardData() {
  if (!accountId.value && !accountRS.value) return
  try {
    await Promise.all([loadAccountInfo(), loadBlockchainStatus(), loadRecentTransactions()])
  } catch (error) {
    console.error('Failed to load dashboard data:', error)
  }
}

async function loadAccountInfo() {
  try {
    const account = await nrcsApi.getAccount(accountRS.value || accountId.value)
    if (account) {
      const nrc = Number(BigInt(account.balanceNQT || '0')) / 100000000
      accountBalance.value = nrc.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })
      localStorage.setItem('nrcs_balance_nqt', account.balanceNQT)
      if (account.accountRS) {
        accountRS.value = account.accountRS
        localStorage.setItem('nrcs_account_rs', account.accountRS)
      }
      if (account.account) {
        accountId.value = account.account
        localStorage.setItem('nrcs_account_id', account.account)
      }
    }
  } catch (error) {
    console.error('Failed to load account:', error)
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
    if (status.time) lastBlockTime.value = formatTimestamp(status.time)
    appStore.isConnected = true
  } catch (error) {
    appStore.isConnected = false
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
    const confirmedTxs = confirmedResult.status === 'fulfilled' ? confirmedResult.value?.transactions || [] : []
    const unconfirmedTxs = unconfirmedResult.status === 'fulfilled' ? unconfirmedResult.value?.unconfirmedTransactions || [] : []
    recentTransactions.value = [...unconfirmedTxs.map((tx: any) => ({ ...tx, confirmations: -1, height: undefined })), ...confirmedTxs]
  } catch (error) {
    console.error('Failed to load transactions:', error)
  } finally {
    isLoadingTransactions.value = false
  }
}

function startPolling() {
  pollTimer = window.setInterval(() => {
    Promise.allSettled([loadBlockchainStatus(), loadRecentTransactions()])
  }, 30000)
}

function stopPolling() {
  if (pollTimer) { clearInterval(pollTimer); pollTimer = null }
}

function formatTimestamp(nrcsTimestamp?: number): string {
  if (!nrcsTimestamp) return ''
  const epochStart = new Date(Date.UTC(2013, 10, 24, 12, 0, 0))
  return new Date(epochStart.getTime() + nrcsTimestamp * 1000).toLocaleString()
}

function formatAmount(nqt?: string): string {
  if (!nqt) return '0.00'
  return (Number(BigInt(nqt)) / 100000000).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })
}

function getTransactionType(type?: number, subtype?: number): string {
  if (type === undefined) return t('dashboard.unknownType')
  const map: Record<number, Record<number, string>> = {
    0: { 0: t('txType.payment'), 1: t('txType.payment') },
    1: { 0: t('txType.messaging') },
    2: { 0: t('txType.coloredCoins'), 1: t('txType.assetTransfer'), 2: t('txType.askOrder'), 3: t('txType.bidOrder'), 4: t('txType.askOrderCancellation'), 5: t('txType.bidOrderCancellation') },
    3: { 0: t('txType.marketplaceListing'), 1: t('txType.marketplaceDelisting'), 2: t('txType.marketplacePriceChange'), 3: t('txType.marketplaceQuantityChange'), 4: t('txType.marketplacePurchase'), 5: t('txType.marketplaceDelivery'), 6: t('txType.marketplaceFeedback'), 7: t('txType.marketplaceRefund') },
    4: { 0: t('txType.accountInfo'), 1: t('txType.aliasAssignment'), 2: t('txType.aliasSell'), 3: t('txType.aliasBuy') },
    5: { 0: t('txType.accountProperty'), 1: t('txType.accountPropertyDelete') },
    6: { 0: t('txType.monetarySystem') },
    7: { 0: t('txType.pollCreation'), 1: t('txType.voteCasting') },
    20: { 0: t('txType.taggedDataUpload') }
  }
  return map[type]?.[subtype ?? 0] || `${t('txType.type')} ${type}.${subtype ?? 0}`
}

function hasAttachment(tx: any): boolean {
  return !!(tx.attachment && (tx.attachment.message || tx.attachment.messageIsText || tx.attachment.encryptedMessage))
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
  if (tx.sender === myAccount || tx.senderRS === myAccount) return tx.recipientRS || tx.recipient || '-'
  return tx.senderRS || tx.sender || '-'
}

function typeColorClass(type?: number): string {
  if (type === 0) return 'type-payment'
  if (type === 1) return 'type-message'
  if (type === 2) return 'type-asset'
  if (type === 3) return 'type-market'
  if (type === 4) return 'type-system'
  if (type === 5) return 'type-property'
  if (type === 6) return 'type-currency'
  if (type === 7) return 'type-vote'
  return ''
}

function txRowClassName({ row }: { row: any }): string {
  return row.confirmations === -1 ? 'tx-row-unconfirmed' : ''
}

function goToAccount(tx: any) {
  const myAccount = accountRS.value || accountId.value
  const target = (tx.sender === myAccount || tx.senderRS === myAccount) ? (tx.recipient || tx.recipientRS) : (tx.sender || tx.senderRS)
  if (target) router.push({ path: '/dashboard', query: { search: target } })
}

function showAccountDetails() { router.push('/dashboard') }

function navigateTo(path: string) { router.push(path) }
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
  gap: $space-xl;
}

// --- Header ---
.dash-header {
  display: flex;
  justify-content: space-between;
  align-items: center;

  .dash-header-left {
    display: flex;
    align-items: baseline;
    gap: $space-md;
  }

  .dash-title {
    font-family: $font-display;
    font-size: $font-size-2xl;
    font-weight: 700;
    letter-spacing: $letter-spacing-tight;
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
    letter-spacing: 0.02em;
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
    letter-spacing: $letter-spacing-wide;

    .status-dot {
      width: 8px; height: 8px;
      border-radius: 50%;
      background: $danger;
      box-shadow: 0 0 6px rgba($danger, 0.5);
      transition: all $duration-normal ease;

      .online & {
        background: $success;
        box-shadow: 0 0 6px rgba($success, 0.5);
      }
    }

    &.online { color: $success; }
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
      text-transform: uppercase;
      letter-spacing: $letter-spacing-wide;
    }

    .block-value {
      font-family: $font-mono;
      font-size: $font-size-sm;
      font-weight: 600;
      color: $primary;
    }
  }
}

// --- Info Cards ---
.info-cards-row {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: $space-lg;

  &--secondary {
    grid-template-columns: repeat(4, 1fr);

    .stat-card {
      padding: $space-md $space-lg;
    }
  }
}

.stat-card {
  position: relative;
  display: flex;
  align-items: center;
  gap: $space-md;
  padding: $space-lg $space-xl;
  border-radius: $radius-lg;
  cursor: pointer;
  overflow: hidden;
  transition: all $duration-slow $ease-out-expo;
  border: 1px solid $border-subtle;
  background: linear-gradient(135deg, rgba($surface-700, 0.6), rgba($surface-800, 0.8));
  backdrop-filter: blur(8px);

  &:hover {
    transform: translateY(-2px);
    border-color: $border-glow;
    box-shadow: $shadow-lg, $shadow-glow;

    .stat-card__glow {
      opacity: 0.08;
    }
  }

  // Glow overlay per card type
  &--balance { --card-accent: #{$primary}; }
  &--assets { --card-accent: #{$success}; }
  &--currency { --card-accent: #{$warning}; }
  &--marketplace { --card-accent: #{$info}; }

  .stat-card__glow {
    position: absolute;
    top: -40%;
    right: -20%;
    width: 200px;
    height: 200px;
    border-radius: 50%;
    background: radial-gradient(circle, var(--card-accent), transparent 70%);
    opacity: 0.05;
    transition: opacity $duration-slow ease;
    pointer-events: none;
  }

  .stat-card__icon-wrap {
    flex-shrink: 0;
    width: 48px;
    height: 48px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: $radius-md;
    background: rgba(var(--card-accent, #{$primary}), 0.08);

    &.icon-sm {
      width: 36px;
      height: 36px;
      border-radius: $radius-sm;
    }
  }

  .stat-card__body {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .stat-card__label {
    font-size: $font-size-xs;
    font-weight: 500;
    color: $text-muted;
    text-transform: uppercase;
    letter-spacing: $letter-spacing-wide;
  }

  .stat-card__value {
    font-family: $font-display;
    font-size: $font-size-xl;
    font-weight: 700;
    line-height: 1.2;
    letter-spacing: $letter-spacing-tight;
    color: $text-primary;
  }

  .stat-card__unit {
    font-size: $font-size-xs;
    color: $text-muted;
  }

  .stat-card__trend {
    flex-shrink: 0;
    color: $success;
  }

  &--compact {
    padding: $space-md $space-lg;
    .stat-card__value { font-size: $font-size-lg; }
  }
}

// --- Transactions Section ---
.transactions-section {
  border: 1px solid $border-subtle !important;
  border-radius: $radius-lg !important;
  overflow: hidden;
  background: linear-gradient(180deg, rgba($surface-800, 0.8), rgba($surface-900, 0.9));
  backdrop-filter: blur(12px);
  box-shadow: $shadow-card;

  :deep(.el-table) {
    --el-table-bg-color: transparent;
    --el-table-tr-bg-color: transparent;
    --el-table-header-bg-color: rgba(255, 255, 255, 0.02);
    --el-table-row-hover-bg-color: rgba($primary, 0.06);
    --el-table-border-color: $border-subtle;
    --el-table-text-color: $text-primary;
    --el-table-header-text-color: $text-muted;

    font-size: $font-size-sm;
    border-radius: 0;

    th.el-table__cell {
      background: rgba(255, 255, 255, 0.025) !important;
      color: $text-muted !important;
      font-weight: 600;
      font-size: $font-size-xs;
      text-transform: uppercase;
      letter-spacing: $letter-spacing-wide;
      padding: $space-md $space-lg;
      border-bottom: 1px solid $border-subtle;
    }

    td.el-table__cell {
      padding: $space-md $space-lg;
      border-bottom: 1px solid rgba($border-default, 0.5);
      transition: all $duration-fast ease;
    }

    tr.el-table__row {
      transition: all $duration-normal ease;

      &:hover > td {
        background: rgba($primary, 0.04) !important;
      }

      &:nth-child(even) {
        td {
          background: rgba(255, 255, 255, 0.008);
        }
      }
    }

    .el-table__empty-block {
      background: transparent;
    }

    // Loading mask - dark background
    .el-loading-mask {
      background-color: rgba(18, 20, 26, 0.9) !important;
      backdrop-filter: blur(8px);
    }

    .el-loading-spinner {
      .circular {
        .path {
          stroke: $primary;
        }
      }
      .el-loading-text {
        color: $text-secondary !important;
      }
    }

    // Empty state
    .el-table__empty-block {
      background: rgba(255, 255, 255, 0.01);
      color: $text-muted;

      .el-table__empty-text {
        color: $text-muted;
      }
    }
  }
}

.tx-section-header {
  padding: $space-xl $space-xl 0;

  .tx-title-group {
    display: flex;
    align-items: center;
    gap: $space-md;
  }

  .tx-icon-ring {
    width: 42px;
    height: 42px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: $radius-md;
    background: $primary-subtle;
    border: 1px solid $border-glow;
  }

  .tx-section-title {
    font-family: $font-display;
    font-size: $font-size-lg;
    font-weight: 600;
    margin: 0;
    color: $text-primary;
  }

  .tx-section-subtitle {
    font-size: $font-size-sm;
    color: $text-muted;
    margin: 2px 0 0;
  }
}

.tx-table-wrapper {
  padding: $space-lg $space-xl 0;
}

// Transaction table custom styles
.msg-indicator {
  display: inline-flex;
  color: transparent;
  transition: color $duration-fast ease;

  &.active {
    color: $primary;
  }
}

.tx-type-badge {
  display: inline-block;
  font-size: $font-size-xs;
  font-weight: 600;
  padding: 3px 10px;
  border-radius: $radius-full;
  letter-spacing: 0.02em;

  &.type-payment   { background: rgba(#00d4ff, 0.1); color: #5ee7ff; }
  &.type-message   { background: rgba(#8b5cf6, 0.1); color: #a78bfa; }
  &.type-asset     { background: rgba(#10b981, 0.1); color: #34d399; }
  &.type-market    { background: rgba(#f59e0b, 0.1); color: #fbbf24; }
  &.type-system    { background: rgba(#64748b, 0.15); color: #94a3b8; }
  &.type-property  { background: rgba(#ec4899, 0.1); color: #f472b6; }
  &.type-currency  { background: rgba(#06b6d4, 0.1); color: #22d3ee; }
  &.type-vote      { background: rgba(#84cc16, 0.1); color: #a3e635; }
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
  max-width: none;
  overflow: visible;
  text-overflow: clip;
  white-space: nowrap;
  display: inline-block;
}

.phasing-dot {
  display: inline-flex;
  color: $warning;
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
  padding: $space-xl $space-2xl;
  border-top: 1px solid $border-subtle;
  margin-top: 0;
  background: rgba(255, 255, 255, 0.015);

  :deep(.el-button) {
    font-size: $font-size-sm;
    font-weight: 600;
    letter-spacing: 0.01em;
    border-radius: $radius-full;
    padding: $space-sm $space-xl;
    transition: all $duration-normal ease;

    &:hover {
      transform: translateY(-1px);
      box-shadow: $shadow-glow;
    }
  }
}

// Unconfirmed row style override
:deep(.tx-row-unconfirmed) {
  > td {
    background: rgba($warning, 0.04) !important;
  }
}

// Responsive
@media (max-width: 1280px) {
  .info-cards-row {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (max-width: 768px) {
  .nrcs-dashboard { padding: $space-md; }
  .info-cards-row,
  .info-cards-row--secondary {
    grid-template-columns: 1fr;
  }
  .dash-header {
    flex-direction: column;
    align-items: flex-start;
    gap: $space-md;
  }
}
</style>
