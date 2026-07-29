<template>
  <div class="peer-list-page">
    <el-card shadow="never">
      <template #header>
        <div class="card-header">
          <div class="header-left">
            <el-icon><Connection /></el-icon>
            <span>{{ t('node.peerList') }}</span>
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

      <!-- Peer Table -->
      <el-table :data="peers" style="width: 100%" v-loading="loading" row-key="address">
        <el-table-column :label="t('node.address')" min-width="240">
          <template #default="{ row }">
            <span class="mono-text">{{ row.announcedAddress || row.address || '-' }}</span>
          </template>
        </el-table-column>

        <el-table-column :label="t('node.state')" width="120">
          <template #default="{ row }">
            <el-tag
              :type="row.state === 1 ? 'success' : row.state === 2 ? 'info' : 'danger'"
              size="small"
            >
              {{ peerStateText(row.state) }}
            </el-tag>
          </template>
        </el-table-column>

        <el-table-column :label="t('node.announceAddress')" min-width="200">
          <template #default="{ row }">
            <span class="mono-text">{{ row.announcedAddress || row.address || '-' }}</span>
          </template>
        </el-table-column>

        <el-table-column :label="t('node.software')" width="130">
          <template #default="{ row }">
            {{ row.application || row.software || '-' }}
          </template>
        </el-table-column>

        <el-table-column :label="t('node.version')" width="100">
          <template #default="{ row }">
            <span class="mono-text">{{ row.version || '-' }}</span>
          </template>
        </el-table-column>

        <el-table-column :label="t('node.platform')" width="130">
          <template #default="{ row }">
            {{ row.platform || '-' }}
          </template>
        </el-table-column>

        <el-table-column :label="t('node.lastUpdated')" width="180">
          <template #default="{ row }">
            {{ formatPeerTime(row.lastUpdated) }}
          </template>
        </el-table-column>
      </el-table>

      <div class="peer-count-info">
        {{ t('node.totalPeers') }}: <strong>{{ peers.length }}</strong>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Connection, Refresh } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import type { NrcsPeer } from '@/api/modules/nrcs.api'
import { usePolling } from '@/composables/usePolling'
import { formatTimestamp } from '@/utils/format'

const { t } = useI18n()

const POLL_INTERVAL = 30000

const peers = ref<NrcsPeer[]>([])
const loading = ref(false)

async function fetchPeers() {
  try {
    loading.value = true
    const res = await nrcsApi.getPeers(undefined, undefined, true)
    peers.value = res.peers || []
  } catch (err) {
    console.error('[PeerList] Failed to fetch peers:', err)
  } finally {
    loading.value = false
  }
}

function peerStateText(state: number): string {
  switch (state) {
    case 0: return t('node.peerState.nonConnected')
    case 1: return t('node.peerState.connected')
    case 2: return t('node.peerState.disconnected')
    default: return `${state}`
  }
}

function formatPeerTime(ts?: number): string {
  if (!ts) return '-'
  return formatTimestamp(ts)
}

const { isPolling, lastPollTime, refreshNow } = usePolling(fetchPeers, POLL_INTERVAL, true)
</script>

<style scoped lang="scss">
.peer-list-page {
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
    color: #606266;
  }

  .peer-count-info {
    margin-top: 16px;
    text-align: right;
    font-size: 13px;
    color: #909399;

    strong {
      color: #303133;
      font-family: 'Roboto Mono', monospace;
    }
  }
}
</style>
