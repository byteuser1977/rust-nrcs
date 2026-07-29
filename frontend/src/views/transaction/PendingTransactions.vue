<template>
  <div class="pending-all-page">
    <el-card shadow="never">
      <template #header>
        <div class="card-header">
          <div class="header-left">
            <el-icon><Clock /></el-icon>
            <span>{{ t('transaction.allPending') }}</span>
            <el-tag v-if="pendingTxs.length > 0" size="small" type="warning" style="margin-left: 8px;">
              {{ pendingTxs.length }}
            </el-tag>
          </div>
          <div class="header-right">
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

      <el-table :data="pendingTxs" style="width: 100%" v-loading="loading">
        <el-table-column :label="t('transaction.id')" width="160">
          <template #default="{ row }">
            <el-tooltip :content="row.transaction" placement="top">
              <span class="mono-text">{{ truncateHash(row.transaction, 8) }}</span>
            </el-tooltip>
          </template>
        </el-table-column>

        <el-table-column :label="t('transaction.date')" width="170">
          <template #default="{ row }">
            {{ formatBlockTime(row.timestamp) }}
          </template>
        </el-table-column>

        <el-table-column :label="t('transaction.type')" width="130">
          <template #default="{ row }">
            <el-tag size="small" type="info">
              {{ getSubTypeName(row.type, row.subtype) }}
            </el-tag>
          </template>
        </el-table-column>

        <el-table-column :label="t('transaction.amount')" width="160" align="right">
          <template #default="{ row }">
            <span :class="parseFloat(row.amountNQT || '0') > 0 ? 'amount-positive' : 'amount-zero'">
              {{ formatNrc(row.amountNQT) }} NRC
            </span>
          </template>
        </el-table-column>

        <el-table-column :label="t('transaction.fee')" width="120" align="right">
          <template #default="{ row }">
            {{ formatNrc(row.feeNQT) }}
          </template>
        </el-table-column>

        <el-table-column :label="t('transaction.sender')" width="180">
          <template #default="{ row }">
            <el-tooltip :content="row.senderRS || row.sender" placement="top">
              <span class="mono-text">{{ truncateHash(row.senderRS || row.sender, 8) }}</span>
            </el-tooltip>
          </template>
        </el-table-column>

        <el-table-column :label="t('transaction.recipient')" width="180">
          <template #default="{ row }">
            <el-tooltip v-if="row.recipientRS || row.recipient" :content="row.recipientRS || row.recipient" placement="top">
              <span class="mono-text">{{ truncateHash(row.recipientRS || row.recipient, 8) }}</span>
            </el-tooltip>
            <span v-else class="text-muted">-</span>
          </template>
        </el-table-column>
      </el-table>

      <div v-if="pendingTxs.length === 0 && !loading" class="empty-state">
        <el-empty :description="t('transaction.noPendingNetwork')" />
      </div>

      <div class="polling-info" v-if="isPolling">
        {{ t('node.autoRefresh', { seconds: POLL_INTERVAL / 1000 }) }}
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Clock, Refresh } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import type { NrcsUnconfirmedTransaction } from '@/api/modules/nrcs.api'
import { usePolling } from '@/composables/usePolling'
import { truncateHash, formatNrc, formatTimestamp } from '@/utils/format'
import { getSubTypeName } from '@/constants/transaction-types'

const { t } = useI18n()

const POLL_INTERVAL = 15000

const pendingTxs = ref<NrcsUnconfirmedTransaction[]>([])
const loading = ref(false)

async function fetchPending() {
  try {
    loading.value = true
    const res = await nrcsApi.getUnconfirmedTransactions()
    pendingTxs.value = res.unconfirmedTransactions || []
  } catch (err) {
    console.error('[PendingTransactions] Failed to fetch:', err)
  } finally {
    loading.value = false
  }
}

function formatBlockTime(ts?: number): string {
  if (!ts) return '-'
  return formatTimestamp(ts)
}

const { isPolling, lastPollTime, refreshNow } = usePolling(fetchPending, POLL_INTERVAL, true)
</script>

<style scoped lang="scss">
.pending-all-page {
  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;

    .header-left {
      display: flex;
      align-items: center;
      gap: 8px;
      font-size: 16px;
      font-weight: 600;
    }
  }

  .mono-text {
    font-family: 'Roboto Mono', monospace;
    font-size: 13px;
  }

  .text-muted {
    color: #c0c4cc;
  }

  .amount-positive {
    color: #67c23a;
    font-family: 'Roboto Mono', monospace;
    font-size: 13px;
  }

  .amount-zero {
    color: #909399;
    font-family: 'Roboto Mono', monospace;
    font-size: 13px;
  }

  .empty-state {
    padding: 40px 0;
  }

  .polling-info {
    margin-top: 12px;
    text-align: right;
    font-size: 12px;
    color: #c0c4cc;
  }
}
</style>
