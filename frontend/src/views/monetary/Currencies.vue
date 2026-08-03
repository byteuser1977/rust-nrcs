<template>
  <div class="currencies-page">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Coin /></el-icon> {{ t('monetary.title') }}</h2>
      <div class="header-actions">
        <el-radio-group v-model="viewMode" size="small" @change="onViewModeChange">
          <el-radio-button value="my">{{ t('dashboard.owned') }}</el-radio-button>
          <el-radio-button value="all">{{ t('dashboard.total') }}</el-radio-button>
        </el-radio-group>
        <el-button type="primary" size="small" @click="showIssue = true">
          <el-icon><Plus /></el-icon> {{ t('monetary.issueCurrency') }}
        </el-button>
        <el-button size="small" @click="refreshData"><el-icon><Refresh /></el-icon> {{ t('common.refresh') }}</el-button>
      </div>
    </div>

    <!-- Search -->
    <el-row :gutter="16" class="mb-16">
      <el-col :span="8">
        <el-input
          v-model="searchQuery"
          :placeholder="t('alias.searchPlaceholder')"
          size="small"
          clearable
        >
          <template #prefix><el-icon><Search /></el-icon></template>
        </el-input>
      </el-col>
    </el-row>

    <!-- Currency Table -->
    <el-card shadow="hover" v-loading="loading">
      <el-table
        :data="filteredCurrencies"
        stripe
        size="small"
        :empty-text="t('common.noData')"
        row-key="currency"
        @expand-change="onExpand"
      >
        <el-table-column type="expand">
          <template #default="{ row }">
            <div class="expand-detail">
              <el-descriptions :column="3" border size="small">
                <el-descriptions-item :label="t('common.id')">
                  {{ truncateHash(row.currency, 8) }}
                </el-descriptions-item>
                <el-descriptions-item :label="t('common.issuer')">
                  {{ row.issuerRS || row.issuer }}
                </el-descriptions-item>
                <el-descriptions-item :label="t('common.description')">
                  {{ row.description || '-' }}
                </el-descriptions-item>
                <el-descriptions-item :label="t('monetary.currentSupply')">
                  {{ formatQNT(row.currentSupplyQNT, row.decimals) }}
                </el-descriptions-item>
                <el-descriptions-item :label="t('monetary.maxSupply')" v-if="row.maxSupplyQNT">
                  {{ formatQNT(row.maxSupplyQNT, row.decimals) }}
                </el-descriptions-item>
                <el-descriptions-item :label="t('monetary.reserveSupply')" v-if="row.reserveSupplyQNT">
                  {{ formatQNT(row.reserveSupplyQNT, row.decimals) }}
                </el-descriptions-item>
              </el-descriptions>
            </div>
          </template>
        </el-table-column>
        <el-table-column prop="code" :label="t('monetary.code')" width="90" />
        <el-table-column prop="name" :label="t('common.name')" min-width="140" />
        <el-table-column :label="t('monetary.type')" width="200">
          <template #default="{ row }">
            <el-tag v-if="row.type & 1" size="small" type="primary" class="mr-1">{{ t('monetary.exchangeable') || 'EXCH' }}</el-tag>
            <el-tag v-if="row.type & 2" size="small" type="warning" class="mr-1">{{ t('monetary.controllable') || 'CTRL' }}</el-tag>
            <el-tag v-if="row.type & 4" size="small" type="info" class="mr-1">{{ t('monetary.reservable') || 'RESV' }}</el-tag>
            <el-tag v-if="row.type & 8" size="small" class="mr-1">{{ t('monetary.claimable') || 'CLAM' }}</el-tag>
            <el-tag v-if="row.type & 16" size="small" type="success" class="mr-1">{{ t('monetary.mintable') || 'MINT' }}</el-tag>
            <el-tag v-if="row.type & 32" size="small" type="danger" class="mr-1">{{ t('monetary.nonShuffleable') || 'NSFL' }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column :label="t('monetary.currentSupply')" width="140" align="right">
          <template #default="{ row }">{{ formatQNT(row.currentSupplyQNT, row.decimals) }}</template>
        </el-table-column>
        <el-table-column :label="t('monetary.maxSupply')" width="140" align="right">
          <template #default="{ row }">
            <template v-if="row.maxSupplyQNT">{{ formatQNT(row.maxSupplyQNT, row.decimals) }}</template>
            <template v-else>-</template>
          </template>
        </el-table-column>
        <el-table-column prop="decimals" :label="t('monetary.decimals')" width="80" align="center" />
        <el-table-column :label="t('common.actions')" width="280" fixed="right">
          <template #default="{ row }">
            <el-button v-if="row.type & 1" size="small" type="primary" link @click="openExchange(row)">
              <el-icon><TrendCharts /></el-icon> {{ t('monetary.exchange') || 'Exchange' }}
            </el-button>
            <el-button size="small" type="success" link @click="openTransfer(row)">
              <el-icon><Sort /></el-icon> {{ t('monetary.transfer') || 'Transfer' }}
            </el-button>
            <!-- 储备按钮（对标 reserve_currency_modal，仅 reservable 且未到发行高度） -->
            <el-button
              v-if="isReservable(row.type) && row.issuanceHeight > lastBlockHeight"
              size="small"
              type="info"
              link
              @click="openReserve(row)"
            >
              <el-icon><Wallet /></el-icon> {{ t('monetary.reserve') }}
            </el-button>
            <!-- 领取按钮（对标 claim_currency_modal，仅 claimable 且已到发行高度） -->
            <el-button
              v-if="isClaimable(row.type) && row.issuanceHeight <= lastBlockHeight"
              size="small"
              type="warning"
              link
              @click="openClaim(row)"
            >
              <el-icon><Download /></el-icon> {{ t('monetary.claim') }}
            </el-button>
            <!-- 铸造按钮（对标 mint_currency_modal，仅 mintable） -->
            <el-button
              v-if="isMintable(row.type)"
              size="small"
              type="primary"
              link
              @click="openMint(row)"
            >
              <el-icon><Coin /></el-icon> {{ t('monetary.mint') }}
            </el-button>
            <!-- 删除按钮（对标 delete_currency_modal，仅发行者可操作） -->
            <el-button
              v-if="isIssuer(row)"
              size="small"
              type="danger"
              link
              @click="openDelete(row)"
            >
              <el-icon><Delete /></el-icon> {{ t('monetary.delete') }}
            </el-button>
            <!-- 查看创建者按钮（对标 currency_founders_modal，仅 reservable 且未到发行高度） -->
            <el-button
              v-if="isReservable(row.type) && row.issuanceHeight > lastBlockHeight"
              size="small"
              link
              @click="openFounders(row)"
            >
              <el-icon><User /></el-icon> {{ t('monetary.viewFounders') }}
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- Modals -->
    <CurrencyExchangeModal
      v-model:visible="showExchangeModal"
      :currency="selectedCurrency"
      @success="refreshData"
    />
    <CurrencyTransferModal
      v-model:visible="showTransferModal"
      @success="refreshData"
    />
    <IssueCurrencyModal v-model:visible="showIssue" @success="refreshData" />
    <!-- 货币二级操作弹窗（对标 nrs.monetarysystem.js reserve/claim/mint/delete/founders） -->
    <CurrencyReserveModal
      v-model:visible="showReserveModal"
      :currency="selectedCurrency"
      @success="refreshData"
    />
    <CurrencyClaimModal
      v-model:visible="showClaimModal"
      :currency="selectedCurrency"
      @success="refreshData"
    />
    <CurrencyMintModal
      v-model:visible="showMintModal"
      :currency="selectedCurrency"
      @success="refreshData"
    />
    <CurrencyDeleteModal
      v-model:visible="showDeleteModal"
      :currency="selectedCurrency"
      @success="refreshData"
    />
    <CurrencyFoundersModal
      v-model:visible="showFoundersModal"
      :currency="selectedCurrency"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { Coin, Plus, Refresh, Search, TrendCharts, Sort, Wallet, Download, Delete, User } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { useAccountStore } from '@/stores/modules/account.store'
import { useNodeStore } from '@/stores/modules/node.store'
import { usePolling } from '@/composables/usePolling'
import { qntToQntf, truncateHash } from '@/utils/format'
import CurrencyExchangeModal from '@/components/modals/CurrencyExchangeModal.vue'
import CurrencyTransferModal from '@/components/modals/CurrencyTransferModal.vue'
import IssueCurrencyModal from '@/components/modals/IssueCurrencyModal.vue'
// 货币二级操作弹窗（对标 nrs.monetarysystem.js）
import CurrencyReserveModal from '@/components/modals/CurrencyReserveModal.vue'
import CurrencyClaimModal from '@/components/modals/CurrencyClaimModal.vue'
import CurrencyMintModal from '@/components/modals/CurrencyMintModal.vue'
import CurrencyDeleteModal from '@/components/modals/CurrencyDeleteModal.vue'
import CurrencyFoundersModal from '@/components/modals/CurrencyFoundersModal.vue'

const { t } = useI18n()
const accountStore = useAccountStore()
const nodeStore = useNodeStore()

const accountRS = computed(() => accountStore.accountRS)
/** 最新区块高度（对标 NRS.lastBlockHeight，用于判断 issuanceHeight 是否已到） */
const lastBlockHeight = computed(() => nodeStore.lastBlockHeight || 0)

// --- State ---
const loading = ref(false)
const currencies = ref<any[]>([])
const searchQuery = ref('')
const viewMode = ref<'my' | 'all'>('all')
const showIssue = ref(false)

// Order/Transfer modals
const showExchangeModal = ref(false)
const showTransferModal = ref(false)
const selectedCurrency = ref<any>(null)

// 货币二级操作弹窗状态
const showReserveModal = ref(false)
const showClaimModal = ref(false)
const showMintModal = ref(false)
const showDeleteModal = ref(false)
const showFoundersModal = ref(false)

// --- Computed ---
const filteredCurrencies = computed(() => {
  if (!searchQuery.value) return currencies.value
  const q = searchQuery.value.toLowerCase()
  return currencies.value.filter(
    (c: any) =>
      c.code?.toLowerCase().includes(q) ||
      c.name?.toLowerCase().includes(q),
  )
})

// --- Methods ---
function formatQNT(qnt: string, decimals: number): string {
  if (!qnt) return '0'
  try {
    return qntToQntf(qnt, decimals)
  } catch {
    return qnt
  }
}

function onExpand(row: any, expandedRows: any[]) {
  // Expand tracking — row detail is always shown from template
}

async function loadCurrencies() {
  loading.value = true
  try {
    if (viewMode.value === 'my' && accountRS.value) {
      const result = await nrcsApi.getAccountCurrencies(accountRS.value, 0, 99)
      const balances = result?.currencyBalances || []
      // Build currency list from account currencies
      const currencyList: any[] = []
      for (const bal of balances) {
        try {
          const detail = await nrcsApi.getCurrency(bal.currency || bal.code)
          currencyList.push({
            ...detail,
            currency: bal.currency || detail?.currency,
            code: bal.code || detail?.code,
            name: detail?.name || bal.code || '-',
          })
        } catch {
          currencyList.push({
            currency: bal.currency || bal.code,
            code: bal.code,
            name: bal.code,
            decimals: bal.decimals || 0,
          })
        }
      }
      currencies.value = currencyList
    } else {
      const result = await nrcsApi.getAllCurrencies(0, 99)
      currencies.value = result?.currencies || []
    }
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  } finally {
    loading.value = false
  }
}

function onViewModeChange() {
  searchQuery.value = ''
  loadCurrencies()
}

function openExchange(currency: any) {
  selectedCurrency.value = currency
  showExchangeModal.value = true
}

function openTransfer(currency: any) {
  selectedCurrency.value = currency
  showTransferModal.value = true
}

// ── 货币类型判断（对标 nrs.monetarysystem.js NRS.isReservable/isClaimable/isMintable） ──
/** 是否可储备（对标 NRS.isReservable，type & 4） */
function isReservable(type: number): boolean {
  return (type & 4) !== 0
}

/** 是否可领取（对标 NRS.isClaimable，type & 8） */
function isClaimable(type: number): boolean {
  return (type & 8) !== 0
}

/** 是否可铸造（对标 NRS.isMintable，type & 16） */
function isMintable(type: number): boolean {
  return (type & 16) !== 0
}

/** 是否为发行者（对标参考 only issuer can delete，issuerRS === 当前账户） */
function isIssuer(row: any): boolean {
  const issuerRS = row.issuerRS || row.issuer
  return !!accountRS.value && !!issuerRS && issuerRS === accountRS.value
}

// ── 二级操作入口（对标参考 data-toggle='modal' data-target='..._modal'） ──
function openReserve(currency: any): void {
  selectedCurrency.value = currency
  showReserveModal.value = true
}

function openClaim(currency: any): void {
  selectedCurrency.value = currency
  showClaimModal.value = true
}

function openMint(currency: any): void {
  selectedCurrency.value = currency
  showMintModal.value = true
}

function openDelete(currency: any): void {
  selectedCurrency.value = currency
  showDeleteModal.value = true
}

function openFounders(currency: any): void {
  selectedCurrency.value = currency
  showFoundersModal.value = true
}

async function refreshData() {
  await loadCurrencies()
}

// --- Lifecycle ---
onMounted(() => {
  loadCurrencies()
})

usePolling(refreshData, 30000, false)
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;

.currencies-page {
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
  .mb-16 {
    margin-bottom: 16px;
  }
  .expand-detail {
    padding: 16px 24px;
    background: var(--el-fill-color-lighter);
  }
  .mr-1 {
    margin-right: 4px;
  }
}
</style>
