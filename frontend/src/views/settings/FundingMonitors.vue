<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><View /></el-icon> {{ t('settings.monitors') }}</h2>
      <div class="header-actions">
        <el-button type="success" size="small" @click="showStartDialog">
          <el-icon><Plus /></el-icon> {{ t('settings.startMonitor') }}
        </el-button>
        <el-button type="primary" size="small" @click="refreshData">
          <el-icon><Refresh /></el-icon> {{ t('common.refresh') }}
        </el-button>
      </div>
    </div>

    <el-card shadow="hover" v-loading="isLoading">
      <template v-if="monitors.length === 0 && !isLoading">
        <el-empty :description="t('settings.noActiveMonitors')" />
      </template>
      <el-table v-else :data="monitors" style="width: 100%" :empty-text="t('common.noData')">
        <el-table-column :label="t('common.account')" min-width="180">
          <template #default="{ row }">
            <span class="mono-text">{{ row.accountRS || truncateHash(row.account, 8) }}</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('settings.property')" min-width="120">
          <template #default="{ row }">
            {{ row.property || '-' }}
          </template>
        </el-table-column>
        <el-table-column :label="t('common.amount')" width="130" align="right">
          <template #default="{ row }">
            {{ row.amount != null ? formatNrc(row.amount) : '-' }}
          </template>
        </el-table-column>
        <el-table-column :label="t('settings.threshold')" width="130" align="right">
          <template #default="{ row }">
            {{ row.threshold != null ? formatNrc(row.threshold) : '-' }}
          </template>
        </el-table-column>
        <el-table-column :label="t('settings.interval')" width="100" align="right">
          <template #default="{ row }">
            {{ row.interval ?? '-' }}
          </template>
        </el-table-column>
        <el-table-column :label="t('common.status')" width="110">
          <template #default="{ row }">
            <el-tag size="small" type="success">{{ t('common.active') }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.actions')" width="100" fixed="right">
          <template #default="{ row }">
            <el-button size="small" type="danger" link @click="stopMonitorAction(row)">
              {{ t('common.stop') }}
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- Start Monitor Dialog -->
    <el-dialog v-model="startDialogVisible" :title="t('settings.startMonitor')" width="520px" destroy-on-close>
      <el-form :model="monitorForm" label-width="140px">
        <el-form-item :label="t('common.secretPhrase')">
          <el-input v-model="monitorForm.secretPhrase" type="password" show-password />
        </el-form-item>
        <el-form-item :label="t('settings.property')">
          <el-input v-model="monitorForm.property" placeholder="e.g. funding" />
        </el-form-item>
        <el-form-item :label="t('common.amount') + ' (NRC)'">
          <el-input v-model="monitorForm.amount" placeholder="0.00" />
        </el-form-item>
        <el-form-item :label="t('settings.threshold') + ' (NRC)'">
          <el-input v-model="monitorForm.threshold" placeholder="0.00" />
        </el-form-item>
        <el-form-item :label="t('settings.interval') + ' (blocks)'">
          <el-input-number v-model="monitorForm.interval" :min="1" :max="100000" controls-position="right" />
        </el-form-item>
        <el-form-item :label="t('common.fee') + ' (NQT)'">
          <el-input v-model="monitorForm.feeNQT" placeholder="100000000" />
        </el-form-item>
        <el-form-item :label="t('settings.deadline') + ' (min)'">
          <el-input-number v-model="monitorForm.deadline" :min="1" :max="1440" controls-position="right" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="startDialogVisible = false">{{ t('common.cancel') }}</el-button>
        <el-button type="primary" @click="submitStartMonitor" :loading="isStarting">
          {{ t('common.submit') }}
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { formatNrc, truncateHash } from '@/utils/format'

const { t } = useI18n()

const isLoading = ref(false)
const isStarting = ref(false)
const monitors = ref<any[]>([])
const startDialogVisible = ref(false)
const monitorForm = ref({
  secretPhrase: '',
  property: '',
  amount: '',
  threshold: '',
  interval: 100,
  feeNQT: '100000000',
  deadline: 1440,
})

onMounted(() => {
  refreshData()
})

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
  monitorForm.value = {
    secretPhrase: '',
    property: '',
    amount: '',
    threshold: '',
    interval: 100,
    feeNQT: '100000000',
    deadline: 1440,
  }
  startDialogVisible.value = true
}

async function submitStartMonitor() {
  if (!monitorForm.value.secretPhrase || !monitorForm.value.property) {
    ElMessage.warning(t('validation.required'))
    return
  }
  isStarting.value = true
  try {
    await nrcsApi.startFundingMonitor({
      secretPhrase: monitorForm.value.secretPhrase,
      property: monitorForm.value.property,
      amount: monitorForm.value.amount || '0',
      feeNQT: monitorForm.value.feeNQT,
      deadline: monitorForm.value.deadline,
    } as any)
    ElMessage.success(t('common.operationSuccess'))
    startDialogVisible.value = false
    refreshData()
  } catch (e: any) {
    ElMessage.error(e?.description || t('common.operationFailed'))
  } finally {
    isStarting.value = false
  }
}

async function stopMonitorAction(monitor: any) {
  try {
    await nrcsApi.stopFundingMonitor({
      secretPhrase: '',
      property: monitor.property || '',
      feeNQT: '100000000',
      deadline: 1440,
    })
    ElMessage.success(t('common.operationSuccess'))
    refreshData()
  } catch (e: any) {
    ElMessage.error(e?.description || t('common.operationFailed'))
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
}
.mono-text {
  font-family: 'Roboto Mono', monospace;
  font-size: 13px;
  color: #606266;
}
</style>
