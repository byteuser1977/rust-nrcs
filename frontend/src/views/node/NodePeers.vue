<template>
  <div class="node-peers-page">
    <el-card shadow="never">
      <template #header>
        <div class="card-header">
          <div class="header-left">
            <el-icon><Connection /></el-icon>
            <span>{{ t('node.peers') }}</span>
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

      <!-- Stats -->
      <el-row :gutter="20" style="margin-bottom: 20px;">
        <el-col :xs="24" :sm="8" :md="6">
          <div class="stat-box">
            <div class="stat-value">{{ peers.length }}</div>
            <div class="stat-label">{{ t('node.totalPeers') }}</div>
          </div>
        </el-col>
        <el-col :xs="24" :sm="8" :md="6">
          <div class="stat-box">
            <div class="stat-value">{{ connectedCount }}</div>
            <div class="stat-label">{{ t('node.connected') }}</div>
          </div>
        </el-col>
        <el-col :xs="24" :sm="8" :md="6">
          <div class="stat-box">
            <div class="stat-value">{{ softwareCount }}</div>
            <div class="stat-label">{{ t('node.uniqueSoftware') }}</div>
          </div>
        </el-col>
      </el-row>

      <!-- Peer Table -->
      <el-table :data="peers" style="width: 100%" v-loading="loading" row-key="address">
        <el-table-column :label="t('node.address')" min-width="220">
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

        <el-table-column :label="t('node.software')" width="120">
          <template #default="{ row }">
            {{ row.application || row.software || '-' }}
          </template>
        </el-table-column>

        <el-table-column :label="t('node.version')" width="100">
          <template #default="{ row }">
            <span class="mono-text">{{ row.version || '-' }}</span>
          </template>
        </el-table-column>

        <el-table-column :label="t('node.platform')" width="120">
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
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
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
    console.error('[NodePeers] Failed to fetch peers:', err)
  } finally {
    loading.value = false
  }
}

const connectedCount = computed(() => peers.value.filter(p => p.state === 1).length)
const softwareCount = computed(() => {
  const apps = new Set(peers.value.filter(p => p.application).map(p => p.application))
  return apps.size
})

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
.node-peers-page {
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

  .stat-box {
    text-align: center;
    padding: 16px;
    background: #f5f7fa;
    border-radius: 8px;

    .stat-value {
      font-size: 24px;
      font-weight: bold;
      color: #409eff;
      font-family: 'Roboto Mono', monospace;
    }

    .stat-label {
      font-size: 13px;
      color: #909399;
      margin-top: 4px;
    }
  }

  .mono-text {
    font-family: 'Roboto Mono', monospace;
    font-size: 13px;
    color: #606266;
  }
}
</style>
