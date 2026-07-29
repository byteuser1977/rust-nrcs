<template>
  <div class="node-blocks-page">
    <el-card shadow="never">
      <template #header>
        <div class="card-header">
          <div class="header-left">
            <span>{{ t('node.recentBlocks') }}</span>
          </div>
          <div class="header-right">
            <el-select
              v-model="blockLimit"
              style="width: 120px; margin-right: 8px;"
              size="small"
              @change="fetchBlocks"
            >
              <el-option :value="10" label="10" />
              <el-option :value="20" label="20" />
              <el-option :value="50" label="50" />
            </el-select>
            <el-button
              size="small"
              text
              type="primary"
              @click="refreshNow"
            >
              <el-icon><Refresh /></el-icon>
              {{ t('common.refresh') }}
            </el-button>
          </div>
        </div>
      </template>

      <el-table
        :data="blocks"
        style="width: 100%"
        v-loading="loading"
        @expand-change="handleExpand"
        row-key="block"
      >
        <el-table-column type="expand">
          <template #default="{ row }">
            <div class="block-expand-detail">
              <h4>{{ t('node.blockTransactions') }} ({{ row.numberOfTransactions || 0 }})</h4>
              <el-table
                v-if="expandedBlock === row.block && row._transactions"
                :data="row._transactions"
                style="width: 100%"
                size="small"
              >
                <el-table-column label="ID" width="160">
                  <template #default="{ row: tx }">
                    <span class="mono-text">{{ truncateHash(tx.transaction, 8) }}</span>
                  </template>
                </el-table-column>
                <el-table-column :label="t('transaction.type')" width="120">
                  <template #default="{ row: tx }">
                    <el-tag size="small" type="info">{{ getTypeName(tx.type) }}</el-tag>
                  </template>
                </el-table-column>
                <el-table-column :label="t('transaction.sender')" width="160">
                  <template #default="{ row: tx }">
                    <span class="mono-text">{{ truncateHash(tx.senderRS || tx.sender, 8) }}</span>
                  </template>
                </el-table-column>
                <el-table-column :label="t('transaction.recipient')" width="160">
                  <template #default="{ row: tx }">
                    <span class="mono-text">{{ tx.recipientRS ? truncateHash(tx.recipientRS, 8) : '-' }}</span>
                  </template>
                </el-table-column>
                <el-table-column :label="t('transaction.amount')" width="140" align="right">
                  <template #default="{ row: tx }">
                    {{ formatNrc(tx.amountNQT) }} NRC
                  </template>
                </el-table-column>
                <el-table-column :label="t('transaction.fee')" width="100" align="right">
                  <template #default="{ row: tx }">
                    {{ formatNrc(tx.feeNQT) }}
                  </template>
                </el-table-column>
              </el-table>
              <div v-else-if="expandedBlock === row.block && expandedLoading" class="loading-expand">
                <el-icon class="is-loading"><Loading /></el-icon>
                {{ t('common.loading') }}
              </div>
              <span v-else-if="row.numberOfTransactions === 0" class="no-tx">{{ t('common.noData') }}</span>
            </div>
          </template>
        </el-table-column>

        <el-table-column :label="t('node.height')" width="100" prop="height">
          <template #default="{ row }">
            <span class="mono-text">{{ row.height?.toLocaleString() }}</span>
          </template>
        </el-table-column>

        <el-table-column :label="t('node.timestamp')" width="180" prop="timestamp">
          <template #default="{ row }">
            {{ formatBlockTime(row.timestamp) }}
          </template>
        </el-table-column>

        <el-table-column :label="t('node.generator')" width="180">
          <template #default="{ row }">
            <el-tooltip :content="row.generatorRS || row.generator" placement="top">
              <span class="mono-text">{{ truncateHash(row.generatorRS || row.generator, 8) }}</span>
            </el-tooltip>
          </template>
        </el-table-column>

        <el-table-column :label="t('node.txCount')" width="80" prop="numberOfTransactions" align="center" />

        <el-table-column :label="t('node.totalAmount')" width="140" align="right">
          <template #default="{ row }">
            {{ formatNrc(row.totalAmountNQT) }} NRC
          </template>
        </el-table-column>

        <el-table-column :label="t('node.totalFee')" width="120" align="right">
          <template #default="{ row }">
            {{ formatNrc(row.totalFeeNQT) }} NRC
          </template>
        </el-table-column>
      </el-table>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Refresh, Loading } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import type { NrcsBlock, NrcsTransaction } from '@/api/modules/nrcs.api'
import { usePolling } from '@/composables/usePolling'
import { truncateHash, formatNrc, formatTimestamp } from '@/utils/format'
import { getTypeName } from '@/constants/transaction-types'

const { t } = useI18n()

const POLL_INTERVAL = 30000

const blocks = ref<(NrcsBlock & { _transactions?: NrcsTransaction[] })[]>([])
const loading = ref(false)
const blockLimit = ref(20)
const expandedBlock = ref<string>('')
const expandedLoading = ref(false)

async function fetchBlocks() {
  try {
    loading.value = true
    const res = await nrcsApi.getBlocks(0, blockLimit.value - 1, false)
    blocks.value = res.blocks || []
  } catch (err) {
    console.error('[NodeBlocks] Failed to fetch blocks:', err)
  } finally {
    loading.value = false
  }
}

async function handleExpand(row: NrcsBlock & { _transactions?: NrcsTransaction[] }, expandedRows: any[]) {
  const isExpanded = expandedRows.some((r: any) => r.block === row.block)
  if (!isExpanded) {
    expandedBlock.value = ''
    return
  }

  if (row._transactions) {
    expandedBlock.value = row.block
    return
  }

  try {
    expandedLoading.value = true
    expandedBlock.value = row.block
    const blockDetail = await nrcsApi.getBlock(row.block, undefined, undefined, true)
    row._transactions = blockDetail.transactions || []
  } catch (err) {
    console.error('[NodeBlocks] Failed to fetch block transactions:', err)
    row._transactions = []
  } finally {
    expandedLoading.value = false
  }
}

function formatBlockTime(ts: number): string {
  return formatTimestamp(ts)
}

const { isPolling, lastPollTime, refreshNow } = usePolling(fetchBlocks, POLL_INTERVAL, true)
</script>

<style scoped lang="scss">
.node-blocks-page {
  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;

    .header-left {
      font-size: 16px;
      font-weight: 600;
    }

    .header-right {
      display: flex;
      align-items: center;
    }
  }

  .mono-text {
    font-family: 'Roboto Mono', monospace;
    font-size: 13px;
  }

  .block-expand-detail {
    padding: 16px 24px;
    background: #fafafa;

    h4 {
      margin: 0 0 12px 0;
      font-size: 14px;
      color: #303133;
    }

    .loading-expand {
      display: flex;
      align-items: center;
      gap: 8px;
      color: #909399;
      font-size: 13px;
    }

    .no-tx {
      color: #909399;
      font-size: 13px;
    }
  }
}
</style>
