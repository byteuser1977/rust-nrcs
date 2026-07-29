<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Connection /></el-icon> {{ t('settings.peers') }}</h2>
      <div class="header-actions">
        <el-button type="success" size="small" @click="showAddDialog">
          <el-icon><Plus /></el-icon> {{ t('settings.addPeer') }}
        </el-button>
        <el-button type="primary" size="small" @click="refreshNow">
          <el-icon><Refresh /></el-icon> {{ t('common.refresh') }}
        </el-button>
      </div>
    </div>

    <el-row :gutter="16" class="stats-row">
      <el-col :xs="12" :sm="6" v-for="s in peerStats" :key="s.label">
        <div class="stat-box">
          <div class="stat-value">{{ s.value }}</div>
          <div class="stat-label">{{ s.label }}</div>
        </div>
      </el-col>
    </el-row>

    <el-card shadow="hover" style="margin-top: 16px">
      <el-table :data="peers" style="width: 100%" v-loading="isLoading" :empty-text="t('common.noData')">
        <el-table-column :label="t('common.address')" min-width="180">
          <template #default="{ row }">
            <span class="mono-text">{{ row.address }}:{{ row.port || 7874 }}</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.state')" width="120">
          <template #default="{ row }">
            <el-tag :type="peerStateTag(row.state)" size="small">{{ peerStateText(row.state) }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column :label="t('settings.weight')" width="80" align="right">
          <template #default="{ row }">
            {{ row.weight ?? '-' }}
          </template>
        </el-table-column>
        <el-table-column :label="t('settings.downloaded')" width="110" align="right">
          <template #default="{ row }">
            {{ row.downloadedVolume != null ? formatVolume(row.downloadedVolume) : '-' }}
          </template>
        </el-table-column>
        <el-table-column :label="t('settings.uploaded')" width="110" align="right">
          <template #default="{ row }">
            {{ row.uploadedVolume != null ? formatVolume(row.uploadedVolume) : '-' }}
          </template>
        </el-table-column>
        <el-table-column :label="t('common.software')" min-width="120">
          <template #default="{ row }">
            {{ row.application || row.software || '-' }}
          </template>
        </el-table-column>
        <el-table-column :label="t('common.platform')" min-width="100">
          <template #default="{ row }">
            {{ row.platform || '-' }}
          </template>
        </el-table-column>
        <el-table-column :label="t('common.version')" min-width="100">
          <template #default="{ row }">
            <el-tag size="small" v-if="row.version">{{ row.version }}</el-tag>
            <span v-else>-</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('settings.services')" min-width="140">
          <template #default="{ row }">
            <div class="services-cell">
              <el-tag
                v-for="svc in parseServices(row.services)"
                :key="svc"
                size="small"
                type="info"
                class="svc-tag"
              >
                {{ svc }}
              </el-tag>
              <span v-if="!row.services || parseServices(row.services).length === 0">-</span>
            </div>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.actions')" width="120" fixed="right">
          <template #default="{ row }">
            <el-button size="small" type="danger" link @click="blacklistPeerAction(row)">
              {{ t('settings.blacklist') }}
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- Add Peer Dialog -->
    <el-dialog v-model="addDialogVisible" :title="t('settings.addPeer')" width="450px" destroy-on-close>
      <el-form :model="addForm" label-width="130px">
        <el-form-item :label="t('settings.peerAddress')">
          <el-input v-model="addForm.peer" placeholder="e.g. 192.168.1.1:7874" />
        </el-form-item>
        <el-form-item :label="t('settings.adminPassword')">
          <el-input v-model="addForm.adminPassword" type="password" show-password placeholder="" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="addDialogVisible = false">{{ t('common.cancel') }}</el-button>
        <el-button type="primary" @click="submitAddPeer" :loading="isAdding">{{ t('common.submit') }}</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, ElMessageBox } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { usePolling } from '@/composables/usePolling'
import { formatVolume } from '@/utils/format'
import type { NrcsPeer } from '@/api/modules/nrcs.api'

const { t } = useI18n()

const isLoading = ref(false)
const isAdding = ref(false)
const peers = ref<NrcsPeer[]>([])
const addDialogVisible = ref(false)
const addForm = ref({ peer: '', adminPassword: '' })

const peerStats = computed(() => {
  const list = peers.value
  const connected = list.filter(p => p.state === 1).length
  const upToDate = list.filter(p => p.version && p.state === 1).length
  const blacklisted = list.filter(p => p.state === 2).length
  return [
    { label: t('settings.connectedPeers'), value: connected },
    { label: t('settings.upToDatePeers'), value: upToDate },
    { label: t('settings.totalKnown'), value: list.length },
    { label: t('settings.blacklisted'), value: blacklisted },
  ]
})

function peerStateText(state: number): string {
  const map: Record<number, string> = {
    1: t('dashboard.connected'),
    2: t('settings.blacklisted'),
    0: t('sidebar.disconnected'),
  }
  return map[state] ?? t('common.unknownError')
}

function peerStateTag(state: number): 'success' | 'danger' | 'info' | 'warning' {
  const map: Record<number, string> = { 1: 'success', 2: 'danger', 0: 'info' }
  return (map[state] || 'info') as 'success' | 'danger' | 'info' | 'warning'
}

function parseServices(services: string | string[] | undefined): string[] {
  if (!services) return []
  if (Array.isArray(services)) return services
  if (typeof services === 'string') return services.split(/[,;]/).map(s => s.trim()).filter(Boolean)
  return []
}

async function fetchPeers() {
  try {
    const result = await nrcsApi.getPeers('', true, true)
    peers.value = result.peers || []
  } catch (error) {
    console.error('Failed to load peers:', error)
  }
}

async function refreshData() {
  isLoading.value = true
  try {
    await fetchPeers()
  } finally {
    isLoading.value = false
  }
}

const { refreshNow } = usePolling(refreshData, 30000, true)

async function blacklistPeerAction(peer: NrcsPeer) {
  try {
    await ElMessageBox.confirm(
      `${t('settings.blacklistPeerConfirm', { address: peer.address })}`,
      t('common.confirm'),
      { type: 'warning' }
    )
    await nrcsApi.blacklistPeer(peer.address)
    ElMessage.success(t('common.operationSuccess'))
    fetchPeers()
  } catch {
    // User cancelled
  }
}

async function submitAddPeer() {
  if (!addForm.value.peer.trim()) {
    ElMessage.warning(t('validation.required'))
    return
  }
  isAdding.value = true
  try {
    await nrcsApi.addPeer(addForm.value.peer.trim())
    ElMessage.success(t('common.operationSuccess'))
    addDialogVisible.value = false
    addForm.value = { peer: '', adminPassword: '' }
    fetchPeers()
  } catch (e: any) {
    ElMessage.error(e?.description || t('common.operationFailed'))
  } finally {
    isAdding.value = false
  }
}

function showAddDialog() {
  addForm.value = { peer: '', adminPassword: '' }
  addDialogVisible.value = true
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
}
.services-cell {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  .svc-tag {
    margin: 0;
  }
}
</style>
