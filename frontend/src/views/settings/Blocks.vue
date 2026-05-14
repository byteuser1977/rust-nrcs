<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Grid /></el-icon> {{ t('settings.blocks') }}</h2>
      <el-button type="primary" size="small" @click="refreshData">
        <el-icon><Refresh /></el-icon> {{ t('common.refresh') }}
      </el-button>
    </div>

    <el-row :gutter="16" class="stats-row" v-loading="isLoading">
      <el-col :xs="12" :sm="8" :md="4" v-for="stat in stats" :key="stat.label">
        <div class="stat-box">
          <div class="stat-value">{{ stat.value }}</div>
          <div class="stat-label">{{ stat.label }}</div>
        </div>
      </el-col>
    </el-row>

    <el-card shadow="hover" style="margin-top: 16px">
      <el-table :data="blocks" style="width: 100%" v-loading="isLoading" :empty-text="t('common.noData')">
        <el-table-column prop="height" label="Height" width="100" sortable />
        <el-table-column label="Block ID" min-width="180">
          <template #default="{ row }">
            <el-tooltip :content="row.block" placement="top">
              <span class="mono-text clickable" @click="showBlockDetail(row)">{{ truncate(row.block, 12) }}</span>
            </el-tooltip>
          </template>
        </el-table-column>
        <el-table-column label="Generator" min-width="180">
          <template #default="{ row }">
            <el-tooltip :content="row.generatorRS" placement="top">
              <span class="mono-text">{{ truncate(row.generatorRS, 16) }}</span>
            </el-tooltip>
          </template>
        </el-table-column>
        <el-table-column label="Timestamp" width="180">
          <template #default="{ row }">
            {{ formatNrcsTime(row.timestamp) }}
          </template>
        </el-table-column>
        <el-table-column prop="numberOfTransactions" label="# Tx" width="80" sortable />
        <el-table-column label="Total Amount" width="140" align="right">
          <template #default="{ row }">
            {{ formatNrcsAmount(row.totalAmountNQT) }}
          </template>
        </el-table-column>
        <el-table-column label="Total Fee" width="120" align="right">
          <template #default="{ row }">
            {{ formatNrcsAmount(row.totalFeeNQT) }}
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

    <el-dialog v-model="detailVisible" title="Block Detail" width="800px" destroy-on-close>
      <template v-if="selectedBlock">
        <el-descriptions :column="2" border size="small">
          <el-descriptions-item label="Block">{{ selectedBlock.block }}</el-descriptions-item>
          <el-descriptions-item label="Height">{{ selectedBlock.height }}</el-descriptions-item>
          <el-descriptions-item label="Generator">{{ selectedBlock.generatorRS }}</el-descriptions-item>
          <el-descriptions-item label="Generator ID">{{ truncate(selectedBlock.generator, 16) }}</el-descriptions-item>
          <el-descriptions-item label="Timestamp">{{ formatNrcsTime(selectedBlock.timestamp) }}</el-descriptions-item>
          <el-descriptions-item label="Transactions">{{ selectedBlock.numberOfTransactions }}</el-descriptions-item>
          <el-descriptions-item label="Total Amount">{{ formatNrcsAmount(selectedBlock.totalAmountNQT) }}</el-descriptions-item>
          <el-descriptions-item label="Total Fee">{{ formatNrcsAmount(selectedBlock.totalFeeNQT) }}</el-descriptions-item>
          <el-descriptions-item label="Payload Length">{{ selectedBlock.payloadLength }}</el-descriptions-item>
          <el-descriptions-item label="Version">{{ selectedBlock.version }}</el-descriptions-item>
          <el-descriptions-item label="Base Target">{{ selectedBlock.baseTarget }}</el-descriptions-item>
          <el-descriptions-item label="Cumulative Difficulty">{{ selectedBlock.cumulativeDifficulty }}</el-descriptions-item>
          <el-descriptions-item label="Previous Block" :span="2">
            <span class="mono-text">{{ selectedBlock.previousBlock || '-' }}</span>
          </el-descriptions-item>
          <el-descriptions-item label="Next Block" :span="2">
            <span class="mono-text">{{ selectedBlock.nextBlock || '-' }}</span>
          </el-descriptions-item>
          <el-descriptions-item label="Payload Hash" :span="2">
            <span class="mono-text">{{ selectedBlock.payloadHash }}</span>
          </el-descriptions-item>
          <el-descriptions-item label="Generation Signature" :span="2">
            <span class="mono-text">{{ selectedBlock.generationSignature }}</span>
          </el-descriptions-item>
          <el-descriptions-item label="Block Signature" :span="2">
            <span class="mono-text">{{ selectedBlock.blockSignature }}</span>
          </el-descriptions-item>
          <el-descriptions-item label="Previous Block Hash" :span="2">
            <span class="mono-text">{{ selectedBlock.previousBlockHash }}</span>
          </el-descriptions-item>
        </el-descriptions>

        <h4 style="margin-top: 16px; margin-bottom: 8px">Transactions</h4>
        <el-table :data="selectedBlock.transactions || []" size="small" max-height="300" :empty-text="t('common.noData')">
          <el-table-column prop="type" label="Type" width="80">
            <template #default="{ row }">
              <span>{{ row.type }}.{{ row.subtype }}</span>
            </template>
          </el-table-column>
          <el-table-column label="Sender" min-width="160">
            <template #default="{ row }">
              <span class="mono-text">{{ truncate(row.senderRS || row.sender, 14) }}</span>
            </template>
          </el-table-column>
          <el-table-column label="Recipient" min-width="160">
            <template #default="{ row }">
              <span class="mono-text">{{ truncate(row.recipientRS || row.recipient, 14) }}</span>
            </template>
          </el-table-column>
          <el-table-column label="Amount" width="120" align="right">
            <template #default="{ row }">
              {{ formatNrcsAmount(row.amountNQT) }}
            </template>
          </el-table-column>
          <el-table-column label="Fee" width="100" align="right">
            <template #default="{ row }">
              {{ formatNrcsAmount(row.feeNQT) }}
            </template>
          </el-table-column>
        </el-table>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { nrcsApi } from '@/api/modules/nrcs.api'
import type { NrcsBlock } from '@/api/modules/nrcs.api'

const { t } = useI18n()

const isLoading = ref(false)
const blocks = ref<NrcsBlock[]>([])
const totalBlocks = ref(0)
const currentPage = ref(1)
const pageSize = ref(20)

const detailVisible = ref(false)
const selectedBlock = ref<NrcsBlock | null>(null)

const stats = computed(() => {
  const list = blocks.value
  if (list.length === 0) return []
  const totalNqt = (n: string) => Number(BigInt(n || '0')) / 100000000
  const avgAmount = list.reduce((s, b) => s + totalNqt(b.totalAmountNQT), 0) / list.length
  const avgFee = list.reduce((s, b) => s + totalNqt(b.totalFeeNQT), 0) / list.length
  const totalTx = list.reduce((s, b) => s + (b.numberOfTransactions || 0), 0)
  const timestamps = list.filter(b => b.timestamp).map(b => b.timestamp as number)
  let avgBlockTime = 0
  if (timestamps.length >= 2) {
    const sorted = [...timestamps].sort((a, b) => b - a)
    const diffs: number[] = []
    for (let i = 0; i < sorted.length - 1; i++) diffs.push(sorted[i] - sorted[i + 1])
    avgBlockTime = diffs.reduce((s, d) => s + d, 0) / diffs.length
  }
  const generators = new Set(list.map(b => b.generatorRS))
  return [
    { label: 'Avg Amount (NRC)', value: avgAmount.toFixed(2) },
    { label: 'Avg Fee (NRC)', value: avgFee.toFixed(4) },
    { label: 'Tx / Block', value: (totalTx / list.length).toFixed(1) },
    { label: 'Block Time (s)', value: avgBlockTime.toFixed(0) },
    { label: 'Forged Blocks', value: list.length.toString() },
    { label: 'Unique Generators', value: generators.size.toString() },
  ]
})

onMounted(() => {
  refreshData()
})

async function refreshData() {
  isLoading.value = true
  try {
    const firstIndex = (currentPage.value - 1) * pageSize.value
    const lastIndex = firstIndex + pageSize.value - 1
    const result = await nrcsApi.getBlocks(firstIndex, lastIndex, true)
    blocks.value = result.blocks || []
    totalBlocks.value = blocks.value.length >= pageSize.value ? (currentPage.value + 1) * pageSize.value : blocks.value.length
  } catch (error) {
    console.error('Failed to load blocks:', error)
  } finally {
    isLoading.value = false
  }
}

function showBlockDetail(block: NrcsBlock) {
  selectedBlock.value = block
  detailVisible.value = true
}

function handlePageChange(page: number) {
  currentPage.value = page
  refreshData()
}

function formatNrcsTime(timestamp?: number): string {
  if (!timestamp) return ''
  const epoch = new Date(Date.UTC(2013, 10, 24, 12, 0, 0))
  return new Date(epoch.getTime() + timestamp * 1000).toLocaleString()
}

function formatNrcsAmount(nqt?: string): string {
  if (!nqt) return '0.00'
  return (Number(BigInt(nqt)) / 100000000).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })
}

function truncate(text: string | undefined, len: number): string {
  if (!text) return '-'
  if (text.length <= len + 4) return text
  return text.slice(0, len) + '...' + text.slice(-4)
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
  }
  .pagination-container {
    display: flex;
    justify-content: center;
    margin-top: 16px;
  }
}
.stats-row {
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
  &.clickable {
    color: #409eff;
    cursor: pointer;
    &:hover { text-decoration: underline; }
  }
}
</style>
