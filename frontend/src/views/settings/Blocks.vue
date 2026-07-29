<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Grid /></el-icon> {{ t('settings.blocks') }}</h2>
      <div class="header-actions">
        <el-radio-group v-model="blockFilter" size="small" @change="handleFilterChange">
          <el-radio-button value="all">{{ t('settings.allBlocks') }}</el-radio-button>
          <el-radio-button value="forged">{{ t('settings.forgedByYou') }}</el-radio-button>
        </el-radio-group>
        <el-button type="primary" size="small" @click="refreshData">
          <el-icon><Refresh /></el-icon> {{ t('common.refresh') }}
        </el-button>
      </div>
    </div>

    <el-row :gutter="16" class="stats-row" v-loading="isLoading">
      <el-col :xs="12" :sm="6" v-for="stat in stats" :key="stat.label">
        <div class="stat-box">
          <div class="stat-value">{{ stat.value }}</div>
          <div class="stat-label">{{ stat.label }}</div>
        </div>
      </el-col>
    </el-row>

    <el-card shadow="hover" style="margin-top: 16px">
      <el-table
        :data="displayBlocks"
        style="width: 100%"
        v-loading="isLoading"
        :empty-text="t('common.noData')"
        @expand-change="handleExpand"
        :row-key="(row: NrcsBlock) => row.block"
      >
        <el-table-column type="expand">
          <template #default="{ row }">
            <el-descriptions :column="2" border size="small" style="margin: 8px 0">
              <el-descriptions-item :label="t('settings.previousBlockHash')">
                <span class="mono-text-sm">{{ row.previousBlockHash || '-' }}</span>
              </el-descriptions-item>
              <el-descriptions-item :label="t('settings.nextBlockHash')">
                <span class="mono-text-sm">{{ row.nextBlock || '-' }}</span>
              </el-descriptions-item>
              <el-descriptions-item :label="t('settings.blockSignature')" :span="2">
                <span class="mono-text-sm">{{ row.blockSignature || '-' }}</span>
              </el-descriptions-item>
              <el-descriptions-item :label="t('settings.generationSignature')" :span="2">
                <span class="mono-text-sm">{{ row.generationSignature || '-' }}</span>
              </el-descriptions-item>
              <el-descriptions-item :label="t('settings.payloadHash')" :span="2">
                <span class="mono-text-sm">{{ row.payloadHash || '-' }}</span>
              </el-descriptions-item>
              <el-descriptions-item :label="t('settings.cumulativeDifficulty')">
                <span class="mono-text-sm">{{ row.cumulativeDifficulty || '-' }}</span>
              </el-descriptions-item>
              <el-descriptions-item :label="t('settings.baseTarget')">
                {{ row.baseTarget || '-' }}
              </el-descriptions-item>
            </el-descriptions>
          </template>
        </el-table-column>
        <el-table-column prop="height" :label="t('dashboard.height')" width="100" sortable />
        <el-table-column :label="t('common.date')" width="180" sortable prop="timestamp">
          <template #default="{ row }">
            {{ formatTimestamp(row.timestamp) }}
          </template>
        </el-table-column>
        <el-table-column :label="t('common.amount') + ' (NRC)'" width="140" align="right">
          <template #default="{ row }">
            {{ formatNrc(row.totalAmountNQT) }}
          </template>
        </el-table-column>
        <el-table-column :label="t('common.fee') + ' (NRC)'" width="120" align="right">
          <template #default="{ row }">
            {{ formatNrc(row.totalFeeNQT) }}
          </template>
        </el-table-column>
        <el-table-column :label="t('settings.numberOfTransactions')" width="80" align="center">
          <template #default="{ row }">
            <el-button
              v-if="row.numberOfTransactions > 0"
              link
              type="primary"
              size="small"
              @click.stop="showTransactions(row)"
            >
              {{ row.numberOfTransactions }}
            </el-button>
            <span v-else>0</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('settings.generator')" min-width="180">
          <template #default="{ row }">
            <el-tooltip :content="row.generatorRS" placement="top">
              <span class="mono-text">{{ truncateHash(row.generatorRS, 8) }}</span>
            </el-tooltip>
          </template>
        </el-table-column>
        <el-table-column :label="t('settings.payloadLength')" width="100" align="right">
          <template #default="{ row }">
            {{ row.payloadLength || 0 }} {{ t('common.bytes') }}
          </template>
        </el-table-column>
        <el-table-column :label="t('settings.baseTarget')" width="110" align="right">
          <template #default="{ row }">
            {{ row.baseTarget ? Number(row.baseTarget).toLocaleString() : '-' }}
          </template>
        </el-table-column>
      </el-table>

      <div class="pagination-container" v-if="totalBlocks > pageSize">
        <el-pagination
          v-model:current-page="currentPage"
          :page-size="pageSize"
          :total="totalBlocks"
          layout="prev, pager, next"
          @current-change="handlePageChange"
        />
      </div>
    </el-card>

    <!-- Transactions Dialog -->
    <el-dialog
      v-model="txDialogVisible"
      :title="t('common.transaction') + ' - ' + t('dashboard.height') + ' #' + selectedBlockHeight"
      width="900px"
      destroy-on-close
    >
      <el-table :data="blockTransactions" style="width: 100%" v-loading="txLoading" size="small" max-height="500" :empty-text="t('common.noData')">
        <el-table-column :label="t('common.transaction')" min-width="180">
          <template #default="{ row }">
            <el-tooltip :content="row.transaction" placement="top">
              <span class="mono-text-sm clickable">{{ truncateHash(row.transaction, 10) }}</span>
            </el-tooltip>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.type')" width="80">
          <template #default="{ row }">
            <span>{{ row.type }}.{{ row.subtype }}</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.sender')" min-width="160">
          <template #default="{ row }">
            <span class="mono-text-sm">{{ truncateHash(row.senderRS || row.sender, 8) }}</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.recipient')" min-width="160">
          <template #default="{ row }">
            <span class="mono-text-sm">{{ truncateHash(row.recipientRS || row.recipient, 8) }}</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.amount')" width="120" align="right">
          <template #default="{ row }">
            {{ formatNrc(row.amountNQT) }}
          </template>
        </el-table-column>
        <el-table-column :label="t('common.fee')" width="100" align="right">
          <template #default="{ row }">
            {{ formatNrc(row.feeNQT) }}
          </template>
        </el-table-column>
      </el-table>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { useAccountStore } from '@/stores/modules/account.store'
import { usePagination } from '@/composables/usePagination'
import { formatTimestamp, formatNrc, truncateHash } from '@/utils/format'
import type { NrcsBlock, NrcsTransaction } from '@/api/modules/nrcs.api'

const { t } = useI18n()
const accountStore = useAccountStore()

const isLoading = ref(false)
const blocks = ref<NrcsBlock[]>([])
const blockFilter = ref<'all' | 'forged'>('all')

const {
  currentPage,
  pageSize,
  total: totalBlocks,
  firstIndex,
  lastIndex,
  goToPage,
  setTotal,
} = usePagination(15)

// Stats computed from loaded blocks
const stats = computed(() => {
  const list = blocks.value
  if (list.length === 0) {
    return [
      { label: t('settings.avgAmount'), value: '-' },
      { label: t('settings.avgFee'), value: '-' },
      { label: t('settings.txPerBlock'), value: '-' },
      { label: t('settings.blockTime'), value: '-' },
    ]
  }
  const toNrc = (nqt: string) => Number(nqt || '0') / 100_000_000
  const avgAmount = list.reduce((s, b) => s + toNrc(b.totalAmountNQT), 0) / list.length
  const avgFee = list.reduce((s, b) => s + toNrc(b.totalFeeNQT), 0) / list.length
  const totalTx = list.reduce((s, b) => s + (b.numberOfTransactions || 0), 0)
  const timestamps = list.filter(b => b.timestamp).map(b => b.timestamp)
  let avgBlockTime = 0
  if (timestamps.length >= 2) {
    const sorted = [...timestamps].sort((a, b) => b - a)
    const diffs: number[] = []
    for (let i = 0; i < sorted.length - 1; i++) diffs.push(sorted[i] - sorted[i + 1])
    avgBlockTime = diffs.reduce((s, d) => s + d, 0) / diffs.length
  }
  return [
    { label: t('settings.avgAmountPerBlock'), value: avgAmount.toFixed(2) + ' NRC' },
    { label: t('settings.avgFeePerBlock'), value: avgFee.toFixed(4) + ' NRC' },
    { label: t('settings.transactionsPerBlock'), value: (totalTx / list.length).toFixed(1) },
    { label: t('settings.blockTime'), value: avgBlockTime.toFixed(0) + ' s' },
  ]
})

// Filter blocks for "Forged By You"
const displayBlocks = computed(() => {
  if (blockFilter.value === 'forged' && accountStore.accountRS) {
    return blocks.value.filter(b => b.generatorRS === accountStore.accountRS)
  }
  return blocks.value
})

// Transactions dialog
const txDialogVisible = ref(false)
const txLoading = ref(false)
const blockTransactions = ref<NrcsTransaction[]>([])
const selectedBlockHeight = ref(0)

onMounted(() => {
  refreshData()
})

async function refreshData() {
  isLoading.value = true
  try {
    const result = await nrcsApi.getBlocks(firstIndex.value, lastIndex.value, false)
    blocks.value = result.blocks || []
    if (result.blocks && result.blocks.length >= pageSize.value) {
      setTotal((currentPage.value + 1) * pageSize.value)
    } else {
      setTotal(firstIndex.value + (result.blocks || []).length)
    }
  } catch (error) {
    console.error('Failed to load blocks:', error)
  } finally {
    isLoading.value = false
  }
}

function handleFilterChange() {
  // Filter is applied reactively via displayBlocks
}

function handlePageChange(page: number) {
  goToPage(page)
  refreshData()
}

function handleExpand(row: NrcsBlock, expandedRows: NrcsBlock[]) {
  // Expanding a row — no additional action needed as data is inline
}

async function showTransactions(row: NrcsBlock) {
  selectedBlockHeight.value = row.height
  txDialogVisible.value = true
  txLoading.value = true
  try {
    if (row.transactions && row.transactions.length > 0) {
      blockTransactions.value = row.transactions
    } else {
      // Fetch block with transactions
      const block = await nrcsApi.getBlock(row.block, undefined, undefined, true)
      blockTransactions.value = block.transactions || []
    }
  } catch (error) {
    console.error('Failed to load block transactions:', error)
    blockTransactions.value = []
  } finally {
    txLoading.value = false
  }
}
</script>

<style scoped lang="scss">
.page-container {
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
.stats-row {
  margin-bottom: 8px;
  .stat-box {
    text-align: center;
    padding: 16px 8px;
    background: #f5f7fa;
    border-radius: 8px;
    .stat-value {
      font-size: 22px;
      font-weight: bold;
      color: #409eff;
      font-family: 'Roboto Mono', monospace;
    }
    .stat-label {
      font-size: 12px;
      color: #909399;
      margin-top: 4px;
    }
  }
}
.mono-text {
  font-family: 'Roboto Mono', monospace;
  font-size: 13px;
  color: #606266;
}
.mono-text-sm {
  font-family: 'Roboto Mono', monospace;
  font-size: 12px;
  color: #606266;
  word-break: break-all;
  &.clickable {
    color: #409eff;
    cursor: pointer;
    &:hover { text-decoration: underline; }
  }
}
</style>
