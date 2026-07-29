<template>
  <div class="node-status-page">
    <!-- Status Cards -->
    <el-row :gutter="20" style="margin-bottom: 20px;">
      <el-col :xs="24" :sm="12" :lg="6">
        <el-card shadow="hover" class="metric-card">
          <div class="metric">
            <span class="metric-label">{{ t('node.currentHeight') }}</span>
            <span class="metric-value">{{ status?.lastBlockHeight?.toLocaleString() || '0' }}</span>
          </div>
        </el-card>
      </el-col>

      <el-col :xs="24" :sm="12" :lg="6">
        <el-card shadow="hover" class="metric-card">
          <div class="metric">
            <span class="metric-label">{{ t('node.connectedPeers') }}</span>
            <span class="metric-value">{{ state?.numberOfPeers?.toLocaleString() || '0' }}</span>
          </div>
        </el-card>
      </el-col>

      <el-col :xs="24" :sm="12" :lg="6">
        <el-card shadow="hover" class="metric-card">
          <div class="metric">
            <span class="metric-label">{{ t('node.version') }}</span>
            <span class="metric-value version-text">{{ status?.version || '-' }}</span>
          </div>
        </el-card>
      </el-col>

      <el-col :xs="24" :sm="12" :lg="6">
        <el-card shadow="hover" class="metric-card">
          <div class="metric">
            <span class="metric-label">{{ t('node.uptime') }}</span>
            <span class="metric-value">{{ formattedUptime }}</span>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- Blockchain Status -->
    <el-row :gutter="20" style="margin-bottom: 20px;">
      <el-col :span="24">
        <el-card shadow="never">
          <template #header>
            <div class="card-header">
              <span>{{ t('node.blockchainStatus') }}</span>
              <el-button
                size="small"
                text
                type="primary"
                :loading="refreshing"
                @click="refreshData"
              >
                <el-icon><Refresh /></el-icon>
                {{ t('common.refresh') }}
              </el-button>
            </div>
          </template>
          <el-descriptions :column="2" border>
            <el-descriptions-item :label="t('node.state')">
              <el-tag
                :type="status?.blockchainState === 'UP_TO_DATE' ? 'success' : 'warning'"
                size="small"
              >
                {{ status?.blockchainState || '-' }}
              </el-tag>
            </el-descriptions-item>
            <el-descriptions-item :label="t('node.isScanning')">
              <el-tag :type="status?.isScanning ? 'warning' : 'success'" size="small">
                {{ status?.isScanning ? t('common.yes') : t('common.no') }}
              </el-tag>
            </el-descriptions-item>
            <el-descriptions-item :label="t('node.isDownloading')">
              <el-tag :type="status?.isDownloading ? 'warning' : 'success'" size="small">
                {{ status?.isDownloading ? t('common.yes') : t('common.no') }}
              </el-tag>
            </el-descriptions-item>
            <el-descriptions-item :label="t('node.cumulativeDifficulty')">
              <span class="mono-text">{{ truncateMiddle(status?.cumulativeDifficulty, 20) }}</span>
            </el-descriptions-item>
            <el-descriptions-item :label="t('node.totalBlocks')">
              {{ state?.numberOfBlocks?.toLocaleString() || '0' }}
            </el-descriptions-item>
            <el-descriptions-item :label="t('node.totalTransactions')">
              {{ state?.numberOfTransactions?.toLocaleString() || '0' }}
            </el-descriptions-item>
            <el-descriptions-item :label="t('node.lastBlock')">
              <span class="mono-text">{{ truncateHash(status?.lastBlock, 8) }}</span>
            </el-descriptions-item>
            <el-descriptions-item :label="t('node.application')">
              {{ status?.application || '-' }}
            </el-descriptions-item>
            <el-descriptions-item :label="t('node.lastBlockchainFeeder')">
              <span class="mono-text">{{ truncateMiddle(status?.lastBlockchainFeeder, 16) }}</span>
            </el-descriptions-item>
            <el-descriptions-item :label="t('node.isTestnet')">
              <el-tag :type="status?.isTestnet ? 'info' : 'success'" size="small">
                {{ status?.isTestnet ? t('common.yes') : t('common.no') }}
              </el-tag>
            </el-descriptions-item>
            <el-descriptions-item :label="t('node.maxRollback')">
              {{ status?.maxRollback || 0 }}
            </el-descriptions-item>
            <el-descriptions-item :label="t('node.lastBlockchainFeederHeight')">
              {{ status?.lastBlockchainFeederHeight?.toLocaleString() || '0' }}
            </el-descriptions-item>
          </el-descriptions>
        </el-card>
      </el-col>
    </el-row>

    <!-- Polling Status -->
    <el-row :gutter="20">
      <el-col :span="24">
        <el-card shadow="never">
          <template #header>
            <span>{{ t('node.monitorControl') }}</span>
          </template>
          <div class="monitor-controls">
            <span class="polling-info" v-if="isPolling">
              {{ t('node.autoRefresh', { seconds: POLL_INTERVAL / 1000 }) }}
            </span>
            <span class="last-poll" v-if="lastPollTime">
              {{ t('node.lastRefresh') }}: {{ formatTime(lastPollTime) }}
            </span>
          </div>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Refresh } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import type { NrcsBlockchainStatus } from '@/api/modules/nrcs.api'
import { usePolling } from '@/composables/usePolling'
import { truncateHash, formatTimestamp } from '@/utils/format'

const { t } = useI18n()

const POLL_INTERVAL = 10000
const refreshing = ref(false)

const status = ref<NrcsBlockchainStatus | null>(null)
const state = ref<any>(null)

async function fetchData() {
  try {
    const [blockchainStatus, stateData] = await Promise.all([
      nrcsApi.getBlockchainStatus(),
      nrcsApi.getState(true)
    ])
    status.value = blockchainStatus
    state.value = stateData
  } catch (err) {
    console.error('[NodeStatus] Failed to fetch data:', err)
  }
}

async function refreshData() {
  refreshing.value = true
  try {
    await fetchData()
  } finally {
    refreshing.value = false
  }
}

const { isPolling, lastPollTime, refreshNow } = usePolling(fetchData, POLL_INTERVAL, true)

const formattedUptime = computed(() => {
  if (!status.value) return '-'
  const sec = status.value.time
  const days = Math.floor(sec / 86400)
  const hours = Math.floor((sec % 86400) / 3600)
  const minutes = Math.floor((sec % 3600) / 60)
  const parts: string[] = []
  if (days > 0) parts.push(`${days}d`)
  if (hours > 0) parts.push(`${hours}h`)
  parts.push(`${minutes}m`)
  return parts.join(' ')
})

function truncateMiddle(value?: string, len = 12): string {
  if (!value || value.length <= len) return value || '-'
  return value.slice(0, len) + '...'
}

function formatTime(ts: number): string {
  return formatTimestamp(Math.floor(ts / 1000), false, true)
}
</script>

<style scoped lang="scss">
.node-status-page {
  .metric-card {
    .metric {
      text-align: center;
      padding: 8px;

      .metric-label {
        display: block;
        font-size: 13px;
        color: #909399;
        margin-bottom: 8px;
      }

      .metric-value {
        display: block;
        font-size: 22px;
        font-weight: bold;
        color: #303133;
        font-family: 'Roboto Mono', monospace;

        &.version-text {
          font-size: 16px;
        }
      }
    }
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .mono-text {
    font-family: 'Roboto Mono', monospace;
    font-size: 13px;
  }

  .monitor-controls {
    display: flex;
    align-items: center;
    gap: 16px;

    .polling-info, .last-poll {
      font-size: 13px;
      color: #909399;
    }
  }
}
</style>
