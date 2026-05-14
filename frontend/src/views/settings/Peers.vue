<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Connection /></el-icon> {{ t('settings.peers') }}</h2>
      <div class="header-actions">
        <el-button type="success" size="small" @click="showAddDialog">
          <el-icon><Plus /></el-icon> Add Peer
        </el-button>
        <el-button type="primary" size="small" @click="refreshData">
          <el-icon><Refresh /></el-icon> {{ t('common.refresh') }}
        </el-button>
      </div>
    </div>

    <el-row :gutter="16" class="stats-row" v-loading="isLoading">
      <el-col :xs="12" :sm="6" v-for="s in peerStats" :key="s.label">
        <div class="stat-box">
          <div class="stat-value">{{ s.value }}</div>
          <div class="stat-label">{{ s.label }}</div>
        </div>
      </el-col>
    </el-row>

    <el-card shadow="hover" style="margin-top: 16px">
      <el-table :data="peers" style="width: 100%" v-loading="isLoading" :empty-text="t('common.noData')">
        <el-table-column label="Address" min-width="160">
          <template #default="{ row }">
            <span class="mono-text">{{ row.address }}</span>
          </template>
        </el-table-column>
        <el-table-column prop="port" label="Port" width="70" />
        <el-table-column label="State" width="100">
          <template #default="{ row }">
            <el-tag :type="peerStateTag(row.state)" size="small">{{ peerStateText(row.state) }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column label="Announced Address" min-width="180">
          <template #default="{ row }">
            <span class="mono-text">{{ row.announcedAddress || '-' }}</span>
          </template>
        </el-table-column>
        <el-table-column label="Software" min-width="120">
          <template #default="{ row }">
            {{ row.application || row.software || '-' }}
          </template>
        </el-table-column>
        <el-table-column label="Version" min-width="100">
          <template #default="{ row }">
            {{ row.version || '-' }}
          </template>
        </el-table-column>
        <el-table-column label="Platform" min-width="100">
          <template #default="{ row }">
            {{ row.platform || '-' }}
          </template>
        </el-table-column>
        <el-table-column label="Last Updated" width="170">
          <template #default="{ row }">
            {{ row.lastUpdated ? formatNrcsTime(row.lastUpdated) : '-' }}
          </template>
        </el-table-column>
        <el-table-column label="Actions" width="200" fixed="right">
          <template #default="{ row }">
            <el-button size="small" type="primary" link @click="showPeerDetail(row)">Details</el-button>
            <el-button size="small" type="warning" link @click="connectPeer(row)">Connect</el-button>
            <el-button size="small" type="danger" link @click="blacklistPeerAction(row)">Blacklist</el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- Add Peer Dialog -->
    <el-dialog v-model="addDialogVisible" title="Add Peer" width="450px" destroy-on-close>
      <el-form :model="addForm" label-width="120px">
        <el-form-item label="Peer Address">
          <el-input v-model="addForm.peer" placeholder="e.g. 192.168.1.1:7874" />
        </el-form-item>
        <el-form-item label="Admin Password">
          <el-input v-model="addForm.adminPassword" type="password" placeholder="Optional" show-password />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="addDialogVisible = false">{{ t('common.cancel') }}</el-button>
        <el-button type="primary" @click="submitAddPeer" :loading="isAdding">{{ t('common.submit') }}</el-button>
      </template>
    </el-dialog>

    <!-- Peer Detail Dialog -->
    <el-dialog v-model="detailDialogVisible" title="Peer Detail" width="600px" destroy-on-close>
      <template v-if="detailPeer">
        <el-descriptions :column="2" border size="small">
          <el-descriptions-item label="Address">{{ detailPeer.address }}</el-descriptions-item>
          <el-descriptions-item label="Port">{{ detailPeer.port }}</el-descriptions-item>
          <el-descriptions-item label="State">{{ peerStateText(detailPeer.state) }}</el-descriptions-item>
          <el-descriptions-item label="Share Address">{{ detailPeer.shareAddress ? 'Yes' : 'No' }}</el-descriptions-item>
          <el-descriptions-item label="Announced">{{ detailPeer.announcedAddress || '-' }}</el-descriptions-item>
          <el-descriptions-item label="Software">{{ detailPeer.software || detailPeer.application || '-' }}</el-descriptions-item>
          <el-descriptions-item label="Version">{{ detailPeer.version || '-' }}</el-descriptions-item>
          <el-descriptions-item label="Platform">{{ detailPeer.platform || '-' }}</el-descriptions-item>
          <el-descriptions-item label="Last Updated" :span="2">
            {{ detailPeer.lastUpdated ? formatNrcsTime(detailPeer.lastUpdated) : '-' }}
          </el-descriptions-item>
        </el-descriptions>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, ElMessageBox } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'
import type { NrcsPeer } from '@/api/modules/nrcs.api'

const { t } = useI18n()

const isLoading = ref(false)
const isAdding = ref(false)
const peers = ref<NrcsPeer[]>([])
const addDialogVisible = ref(false)
const detailDialogVisible = ref(false)
const detailPeer = ref<NrcsPeer | null>(null)
const addForm = ref({ peer: '', adminPassword: '' })

const peerStats = computed(() => {
  const list = peers.value
  const connected = list.filter(p => p.state === 1).length
  const blacklisted = list.filter(p => p.state === 2).length
  return [
    { label: 'Total Known', value: list.length },
    { label: 'Connected', value: connected },
    { label: 'Blacklisted', value: blacklisted },
    { label: 'Active', value: connected },
  ]
})

onMounted(() => { refreshData() })

async function refreshData() {
  isLoading.value = true
  try {
    const result = await nrcsApi.getPeers(undefined, true, true)
    peers.value = result.peers || []
  } catch (error) {
    console.error('Failed to load peers:', error)
  } finally {
    isLoading.value = false
  }
}

function peerStateText(state: number): string {
  const map: Record<number, string> = { 1: 'Connected', 2: 'Blacklisted', 0: 'Disconnected' }
  return map[state] || 'Unknown'
}

function peerStateTag(state: number): 'success' | 'danger' | 'info' | 'warning' {
  const map: Record<number, string> = { 1: 'success', 2: 'danger', 0: 'info' }
  return (map[state] || 'info') as any
}

async function showPeerDetail(peer: NrcsPeer) {
  try {
    const result = await nrcsApi.getPeer(peer.address)
    detailPeer.value = result
  } catch {
    detailPeer.value = peer
  }
  detailDialogVisible.value = true
}

async function connectPeer(peer: NrcsPeer) {
  try {
    await nrcsApi.addPeer(peer.announcedAddress || `${peer.address}:${peer.port}`)
    ElMessage.success('Peer added successfully')
    refreshData()
  } catch (e: any) {
    ElMessage.error(e?.description || 'Failed to add peer')
  }
}

async function blacklistPeerAction(peer: NrcsPeer) {
  try {
    await ElMessageBox.confirm(`Blacklist peer ${peer.address}?`, 'Confirm', { type: 'warning' })
    await nrcsApi.blacklistPeer(peer.address)
    ElMessage.success('Peer blacklisted')
    refreshData()
  } catch { /* cancelled */ }
}

async function submitAddPeer() {
  if (!addForm.value.peer) {
    ElMessage.warning('Please enter a peer address')
    return
  }
  isAdding.value = true
  try {
    const params: any = { peer: addForm.value.peer }
    if (addForm.value.adminPassword) params.adminPassword = addForm.value.adminPassword
    await nrcsApi.addPeer(params.peer)
    ElMessage.success('Peer added')
    addDialogVisible.value = false
    addForm.value = { peer: '', adminPassword: '' }
    refreshData()
  } catch (e: any) {
    ElMessage.error(e?.description || 'Failed to add peer')
  } finally {
    isAdding.value = false
  }
}

function showAddDialog() {
  addForm.value = { peer: '', adminPassword: '' }
  addDialogVisible.value = true
}

function formatNrcsTime(timestamp?: number): string {
  if (!timestamp) return ''
  const epoch = new Date(Date.UTC(2013, 10, 24, 12, 0, 0))
  return new Date(epoch.getTime() + timestamp * 1000).toLocaleString()
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
</style>
