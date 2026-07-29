<template>
  <div class="approval-requests-page">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Checked /></el-icon> {{ t('monetary.approvalRequests') }}</h2>
      <div class="header-actions">
        <el-select
          v-model="selectedCurrency"
          :placeholder="t('monetary.currency')"
          size="small"
          clearable
          filterable
          style="width: 180px"
          @change="onFilterChange"
        >
          <el-option
            v-for="c in currencyList"
            :key="c.currency"
            :label="c.code || c.name"
            :value="c.currency"
          />
        </el-select>
        <el-button size="small" @click="refreshData"><el-icon><Refresh /></el-icon> {{ t('common.refresh') }}</el-button>
      </div>
    </div>

    <el-card shadow="hover" v-loading="pagination.isLoading.value">
      <el-table
        :data="items"
        stripe
        size="small"
        :empty-text="t('common.noData')"
      >
        <el-table-column :label="t('common.transaction')" width="180">
          <template #default="{ row }">
            <span v-if="row.transaction">{{ truncateHash(row.transaction, 8) }}</span>
            <span v-else>-</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('monetary.currency')" width="100">
          <template #default="{ row }">
            {{ getCurrencyAttachment(row)?.code || getCurrencyAttachment(row)?.name || '-' }}
          </template>
        </el-table-column>
        <el-table-column :label="t('common.type')" width="120">
          <template #default="{ row }">{{ getTypeLabel(row.type, row.subtype) }}</template>
        </el-table-column>
        <el-table-column prop="senderRS" :label="t('common.sender')" min-width="160" />
        <el-table-column :label="t('asset.finishHeight')" width="110" align="right">
          <template #default="{ row }">{{ row.phasingFinishHeight || row.finishHeight || '-' }}</template>
        </el-table-column>
        <el-table-column :label="t('common.date')" width="160">
          <template #default="{ row }">{{ formatTimestamp(row.timestamp) }}</template>
        </el-table-column>
        <el-table-column :label="t('common.actions')" width="100" fixed="right">
          <template #default="{ row }">
            <el-button
              size="small"
              type="primary"
              @click="openApproveModal(row)"
            >
              {{ t('asset.approve') }}
            </el-button>
          </template>
        </el-table-column>
      </el-table>

      <div class="pagination-container" v-if="pagination.total.value > 0">
        <el-pagination
          v-model:current-page="pagination.currentPage.value"
          :page-size="pagination.pageSize.value"
          :total="pagination.total.value"
          layout="prev, pager, next"
          @current-change="onPageChange"
        />
      </div>
    </el-card>

    <!-- Approve Modal -->
    <el-dialog
      v-model="showApproveModal"
      :title="t('asset.approve')"
      width="420px"
      :close-on-click-modal="false"
      destroy-on-close
      @close="handleApproveClose"
    >
      <el-form ref="approveFormRef" :model="approveForm" :rules="approveRules" label-position="top">
        <el-form-item :label="t('common.transaction')">
          <el-input :model-value="approvingTransaction?.transaction || ''" disabled />
        </el-form-item>
        <el-form-item :label="t('common.secretPhrase')" prop="secretPhrase">
          <el-input
            v-model="approveForm.secretPhrase"
            type="password"
            show-password
            clearable
            :placeholder="t('asset.enterSecretPhrase')"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showApproveModal = false">{{ t('common.cancel') }}</el-button>
        <el-button type="primary" :loading="approving" @click="handleApprove">
          {{ t('asset.approve') }}
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, type FormInstance, type FormRules } from 'element-plus'
import { Checked, Refresh } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { useAccountStore } from '@/stores/modules/account.store'
import { usePagination } from '@/composables/usePagination'
import { formatTimestamp, truncateHash } from '@/utils/format'

const { t } = useI18n()
const accountStore = useAccountStore()

const accountRS = computed(() => accountStore.accountRS)

// Currency type: 5 is Monetary System in NRCS
const CURRENCY_TRANSACTION_TYPES = [5]

// --- State ---
const items = ref<any[]>([])
const currencyList = ref<any[]>([])
const selectedCurrency = ref('')

// Approve modal
const showApproveModal = ref(false)
const approving = ref(false)
const approvingTransaction = ref<any>(null)
const approveFormRef = ref<FormInstance>()
const approveForm = ref({ secretPhrase: '' })

const approveRules = computed<FormRules>(() => ({
  secretPhrase: [{ required: true, message: t('asset.enterSecretPhrase'), trigger: 'blur' }],
}))

const pagination = usePagination(15)

// --- Methods ---
function getCurrencyAttachment(row: any): any {
  return row?.attachment || {}
}

function getTypeLabel(type: number, subtype: number): string {
  // Currency transaction type is 5
  if (type === 5) {
    const subtypeMap: Record<number, string> = {
      0: t('monetary.issueCurrency') || 'Currency Issuance',
      1: t('monetary.transferHistory') || 'Reserve Increase',
      2: t('monetary.transferHistory') || 'Reserve Claim',
      3: t('monetary.transferHistory') || 'Currency Transfer',
      4: t('monetary.exchangeHistory') || 'Publish Offer',
      5: t('monetary.buy') || 'Exchange Buy',
      6: t('monetary.sell') || 'Exchange Sell',
      7: t('monetary.exchange') || 'Currency Mint',
      8: t('monetary.exchange') || 'Currency Delete',
    }
    return subtypeMap[subtype] || `${type}.${subtype}`
  }
  return `${type}.${subtype}`
}

async function loadCurrenciesForFilter() {
  try {
    if (accountRS.value) {
      const result = await nrcsApi.getAccountCurrencies(accountRS.value, 0, 99)
      const balances = result?.currencyBalances || []
      currencyList.value = balances.map((b: any) => ({
        currency: b.currency || b.code,
        code: b.code,
        name: b.name || b.code,
      }))
    }
  } catch {
    currencyList.value = []
  }
}

async function loadPhasedTransactions() {
  pagination.isLoading.value = true
  try {
    if (!accountRS.value) {
      items.value = []
      return
    }
    const result = await nrcsApi.getAccountPhasedTransactions(
      accountRS.value,
      pagination.firstIndex.value,
      pagination.lastIndex.value,
    )
    let allTxs = (result as any).transactions || []

    // Filter by currency type (type 5 = Monetary System)
    allTxs = allTxs.filter((tx: any) =>
      CURRENCY_TRANSACTION_TYPES.includes(tx.type),
    )

    // Apply currency filter if selected
    if (selectedCurrency.value) {
      allTxs = allTxs.filter((tx: any) => {
        const att = tx.attachment || {}
        return att.currency === selectedCurrency.value
      })
    }

    items.value = allTxs
    pagination.setTotalFromList(allTxs.length)
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  } finally {
    pagination.isLoading.value = false
  }
}

function onFilterChange() {
  pagination.reset()
  loadPhasedTransactions()
}

function onPageChange() {
  loadPhasedTransactions()
}

function openApproveModal(row: any) {
  approvingTransaction.value = row
  approveForm.value.secretPhrase = ''
  showApproveModal.value = true
}

async function handleApprove() {
  if (!approveFormRef.value) return
  await approveFormRef.value.validate(async (valid) => {
    if (!valid) return
    approving.value = true
    try {
      await nrcsApi.approveTransaction({
        secretPhrase: approveForm.value.secretPhrase,
        transaction: approvingTransaction.value?.transaction || '',
        feeNQT: '100000000',
        deadline: 1440,
      })
      ElMessage.success(t('common.operationSuccess'))
      showApproveModal.value = false
      loadPhasedTransactions()
    } catch (err: any) {
      ElMessage.error(err?.message || t('common.operationFailed'))
    } finally {
      approving.value = false
    }
  })
}

function handleApproveClose() {
  approveFormRef.value?.resetFields()
  approveForm.value.secretPhrase = ''
}

async function refreshData() {
  await Promise.all([loadCurrenciesForFilter(), loadPhasedTransactions()])
}

// --- Lifecycle ---
onMounted(() => {
  refreshData()
})

watch([() => pagination.currentPage.value, () => pagination.pageSize.value], () => {
  loadPhasedTransactions()
})
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
  .pagination-container {
    display: flex;
    justify-content: center;
    margin-top: 16px;
  }
}
</style>
