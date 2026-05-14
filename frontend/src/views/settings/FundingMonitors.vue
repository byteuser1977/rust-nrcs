<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><View /></el-icon> {{ t('settings.monitors') }}</h2>
      <div class="header-actions">
        <el-button type="success" size="small" @click="showStartDialog">
          <el-icon><Plus /></el-icon> Start Monitor
        </el-button>
        <el-button type="primary" size="small" @click="refreshData">
          <el-icon><Refresh /></el-icon> {{ t('common.refresh') }}
        </el-button>
      </div>
    </div>

    <el-card shadow="hover" v-loading="isLoading">
      <template v-if="monitors.length === 0 && !isLoading">
        <el-empty description="No active funding monitors" />
      </template>
      <el-table v-else :data="monitors" style="width: 100%" :empty-text="t('common.noData')">
        <el-table-column label="Account" min-width="180">
          <template #default="{ row }">
            <span class="mono-text">{{ row.accountRS || truncate(row.account, 14) }}</span>
          </template>
        </el-table-column>
        <el-table-column label="Property" min-width="120">
          <template #default="{ row }">
            {{ row.property || '-' }}
          </template>
        </el-table-column>
        <el-table-column label="Amount" width="130" align="right">
          <template #default="{ row }">
            {{ formatNrcsAmount(row.amount) }}
          </template>
        </el-table-column>
        <el-table-column label="Threshold" width="130" align="right">
          <template #default="{ row }">
            {{ formatNrcsAmount(row.threshold) }}
          </template>
        </el-table-column>
        <el-table-column label="Interval" width="100" align="right">
          <template #default="{ row }">
            {{ row.interval || '-' }}
          </template>
        </el-table-column>
        <el-table-column label="Monitored Accounts" width="140">
          <template #default="{ row }">
            <el-tag size="small" type="info" v-if="row.monitoredAccounts">
              {{ Array.isArray(row.monitoredAccounts) ? row.monitoredAccounts.length : 0 }} accounts
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="Actions" width="100" fixed="right">
          <template #default="{ row }">
            <el-popconfirm title="Stop this funding monitor?" @confirm="stopMonitor(row)">
              <template #reference>
                <el-button size="small" type="danger" link>Stop</el-button>
              </template>
            </el-popconfirm>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- Start Monitor Dialog -->
    <el-dialog v-model="startDialogVisible" title="Start Funding Monitor" width="500px" destroy-on-close>
      <el-form :model="monitorForm" label-width="130px">
        <el-form-item label="Secret Phrase"><el-input v-model="monitorForm.secretPhrase" type="password" show-password /></el-form-item>
        <el-form-item label="Property"><el-input v-model="monitorForm.property" placeholder="e.g. funding" /></el-form-item>
        <el-form-item label="Amount"><el-input v-model="monitorForm.amount" placeholder="NRC amount" /></el-form-item>
        <el-form-item label="Fee (NQT)"><el-input v-model="monitorForm.feeNQT" placeholder="Fee" /></el-form-item>
        <el-form-item label="Deadline (min)"><el-input-number v-model="monitorForm.deadline" :min="1" :max="1440" /></el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="startDialogVisible = false">{{ t('common.cancel') }}</el-button>
        <el-button type="primary" @click="submitStartMonitor" :loading="isStarting">{{ t('common.submit') }}</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'

const { t } = useI18n()

const isLoading = ref(false)
const isStarting = ref(false)
const monitors = ref<any[]>([])
const startDialogVisible = ref(false)
const monitorForm = ref({ secretPhrase: '', property: '', amount: '', feeNQT: '100000000', deadline: 1440 })

onMounted(() => { refreshData() })

async function refreshData() {
  isLoading.value = true
  try {
    const result = await nrcsApi.getFundingMonitor()
    monitors.value = result.monitors || []
  } catch (error) {
    console.error('Failed to load monitors:', error)
  } finally {
    isLoading.value = false
  }
}

function showStartDialog() {
  monitorForm.value = { secretPhrase: '', property: '', amount: '', feeNQT: '100000000', deadline: 1440 }
  startDialogVisible.value = true
}

async function submitStartMonitor() {
  if (!monitorForm.value.secretPhrase || !monitorForm.value.property) {
    ElMessage.warning('Please fill in required fields')
    return
  }
  isStarting.value = true
  try {
    await nrcsApi.startFundingMonitor({
      secretPhrase: monitorForm.value.secretPhrase,
      property: monitorForm.value.property,
      amount: monitorForm.value.amount,
      feeNQT: monitorForm.value.feeNQT,
      deadline: monitorForm.value.deadline
    })
    ElMessage.success('Funding monitor started')
    startDialogVisible.value = false
    refreshData()
  } catch (e: any) {
    ElMessage.error(e?.description || 'Failed to start monitor')
  } finally {
    isStarting.value = false
  }
}

async function stopMonitor(monitor: any) {
  try {
    await nrcsApi.stopFundingMonitor({
      secretPhrase: '',
      property: monitor.property || '',
      feeNQT: '100000000',
      deadline: 1440
    })
    ElMessage.success('Monitor stopped')
    refreshData()
  } catch (e: any) {
    ElMessage.error(e?.description || 'Failed to stop monitor')
  }
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
    .header-actions {
      display: flex;
      gap: 8px;
    }
  }
}
.mono-text {
  font-family: 'Roboto Mono', monospace;
  font-size: 13px;
  color: #606266;
}
</style>
