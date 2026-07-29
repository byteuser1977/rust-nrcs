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
        <el-table-column :label="t('common.actions')" width="160" fixed="right">
          <template #default="{ row }">
            <el-button v-if="row.type & 1" size="small" type="primary" link @click="openExchange(row)">
              <el-icon><TrendCharts /></el-icon> {{ t('monetary.exchange') || 'Exchange' }}
            </el-button>
            <el-button size="small" type="success" link @click="openTransfer(row)">
              <el-icon><Sort /></el-icon> {{ t('monetary.transfer') || 'Transfer' }}
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- Modals -->
    <CurrencyOrderModal
      v-model:visible="showOrderModal"
      :order-type="orderType"
      :currency-id="selectedCurrency?.currency || ''"
      :currency-code="selectedCurrency?.code || ''"
      :decimals="selectedCurrency?.decimals || 0"
      :units="orderUnits"
      :rate="orderRate"
      @success="onOrderSuccess"
    />
    <CurrencyTransferModal
      v-model:visible="showTransferModal"
      @success="refreshData"
    />
    <IssueCurrencyModal v-model:visible="showIssue" @success="refreshData" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { Coin, Plus, Refresh, Search, TrendCharts, Sort } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { useAccountStore } from '@/stores/modules/account.store'
import { usePolling } from '@/composables/usePolling'
import { qntToQntf, truncateHash } from '@/utils/format'
import CurrencyOrderModal from '@/components/modals/CurrencyOrderModal.vue'
import CurrencyTransferModal from '@/components/modals/CurrencyTransferModal.vue'
import IssueCurrencyModal from '@/components/modals/IssueCurrencyModal.vue'

const { t } = useI18n()
const accountStore = useAccountStore()

const accountRS = computed(() => accountStore.accountRS)

// --- State ---
const loading = ref(false)
const currencies = ref<any[]>([])
const searchQuery = ref('')
const viewMode = ref<'my' | 'all'>('all')
const showIssue = ref(false)

// Order/Transfer modals
const showOrderModal = ref(false)
const showTransferModal = ref(false)
const selectedCurrency = ref<any>(null)
const orderType = ref<'buy' | 'sell'>('buy')
const orderUnits = ref('0')
const orderRate = ref('0')

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
  orderType.value = 'buy'
  orderUnits.value = '0'
  orderRate.value = '0'
  showOrderModal.value = true
}

function openTransfer(currency: any) {
  selectedCurrency.value = currency
  showTransferModal.value = true
}

function onOrderSuccess() {
  refreshData()
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
