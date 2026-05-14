<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Cpu /></el-icon> {{ t('settings.generators') }}</h2>
      <div class="header-actions">
        <el-button type="success" size="small" @click="startForgingAction">
          <el-icon><VideoPlay /></el-icon> Start Forging
        </el-button>
        <el-button type="danger" size="small" @click="stopForgingAction">
          <el-icon><VideoPause /></el-icon> Stop Forging
        </el-button>
        <el-button type="primary" size="small" @click="refreshData">
          <el-icon><Refresh /></el-icon> {{ t('common.refresh') }}
        </el-button>
      </div>
    </div>

    <el-row :gutter="16" class="stats-row" v-loading="isLoading">
      <el-col :xs="12" :sm="8" v-for="s in genStats" :key="s.label">
        <div class="stat-box">
          <div class="stat-value">{{ s.value }}</div>
          <div class="stat-label">{{ s.label }}</div>
        </div>
      </el-col>
    </el-row>

    <el-card shadow="hover" style="margin-top: 16px">
      <el-table :data="generators" style="width: 100%" v-loading="isLoading" :empty-text="t('common.noData')">
        <el-table-column label="Account RS" min-width="200">
          <template #default="{ row }">
            <span class="mono-text">{{ row.accountRS }}</span>
          </template>
        </el-table-column>
        <el-table-column label="Effective Balance" width="150" align="right">
          <template #default="{ row }">
            {{ row.effectiveBalanceNXT ? formatAmount(row.effectiveBalanceNXT) : '-' }}
          </template>
        </el-table-column>
        <el-table-column label="Deadline" width="100" align="right">
          <template #default="{ row }">
            <span class="mono-text">{{ row.deadline }}</span>
          </template>
        </el-table-column>
        <el-table-column label="Hit Time" width="100" align="right">
          <template #default="{ row }">
            <span class="mono-text">{{ row.hitTime }}</span>
          </template>
        </el-table-column>
        <el-table-column label="Countdown" width="120">
          <template #default="{ row }">
            <span :class="['countdown', row.deadline < 30 ? 'countdown-soon' : '']">
              {{ countdownText(row.deadline) }}
            </span>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- Passphrase Dialog -->
    <el-dialog v-model="passphraseDialogVisible" :title="passphraseAction === 'start' ? 'Start Forging' : 'Stop Forging'" width="400px" destroy-on-close>
      <el-form label-width="120px">
        <el-form-item label="Secret Phrase">
          <el-input v-model="passphrase" type="password" show-password placeholder="Enter your secret phrase" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="passphraseDialogVisible = false">{{ t('common.cancel') }}</el-button>
        <el-button type="primary" @click="submitForging" :loading="isForgingAction">
          {{ t('common.submit') }}
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'
import type { NrcsGenerator } from '@/api/modules/nrcs.api'

const { t } = useI18n()

const isLoading = ref(false)
const isForgingAction = ref(false)
const generators = ref<NrcsGenerator[]>([])
const blockTime = ref(0)
const blockHeight = ref(0)
const passphraseDialogVisible = ref(false)
const passphraseAction = ref<'start' | 'stop'>('start')
const passphrase = ref('')
let refreshTimer: number | null = null

const genStats = computed(() => [
  { label: 'Last Block', value: blockTime.value ? formatNrcsTime(blockTime.value) : '-' },
  { label: 'Height', value: blockHeight.value ? `#${blockHeight.value.toLocaleString()}` : '-' },
  { label: 'Active Forgers', value: generators.value.length.toString() },
])

onMounted(async () => {
  await refreshData()
  refreshTimer = window.setInterval(refreshData, 5000)
})

onUnmounted(() => {
  if (refreshTimer) { clearInterval(refreshTimer); refreshTimer = null }
})

async function refreshData() {
  try {
    isLoading.value = true
    const [forging, status] = await Promise.allSettled([
      nrcsApi.getForging(),
      nrcsApi.getBlockchainStatus()
    ])
    if (forging.status === 'fulfilled') {
      generators.value = forging.value.generators || []
    }
    if (status.status === 'fulfilled') {
      blockTime.value = status.value.time
      blockHeight.value = status.value.lastBlockHeight
    }
  } catch (error) {
    console.error('Failed to load generators:', error)
  } finally {
    isLoading.value = false
  }
}

function formatNrcsTime(timestamp?: number): string {
  if (!timestamp) return ''
  const epoch = new Date(Date.UTC(2013, 10, 24, 12, 0, 0))
  return new Date(epoch.getTime() + timestamp * 1000).toLocaleString()
}

function formatAmount(nqt?: string): string {
  if (!nqt) return '0.00'
  return (Number(BigInt(nqt)) / 100000000).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })
}

function countdownText(deadline: number): string {
  if (deadline <= 0) return 'Now'
  const mins = Math.floor(deadline / 60)
  const secs = deadline % 60
  return `${mins}m ${secs}s`
}

function startForgingAction() {
  passphraseAction.value = 'start'
  passphrase.value = ''
  passphraseDialogVisible.value = true
}

function stopForgingAction() {
  passphraseAction.value = 'stop'
  passphrase.value = ''
  passphraseDialogVisible.value = true
}

async function submitForging() {
  if (!passphrase.value) {
    ElMessage.warning('Please enter your secret phrase')
    return
  }
  isForgingAction.value = true
  try {
    if (passphraseAction.value === 'start') {
      await nrcsApi.startForging(passphrase.value)
      ElMessage.success('Forging started')
    } else {
      await nrcsApi.stopForging(passphrase.value)
      ElMessage.success('Forging stopped')
    }
    passphraseDialogVisible.value = false
    await refreshData()
  } catch (e: any) {
    ElMessage.error(e?.description || 'Forging action failed')
  } finally {
    isForgingAction.value = false
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
.stats-row {
  .stat-box {
    text-align: center;
    padding: 16px 8px;
    background: #f5f7fa;
    border-radius: 8px;
    .stat-value {
      font-size: 20px;
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
.countdown {
  font-family: 'Roboto Mono', monospace;
  color: #67c23a;
  &-soon {
    color: #e6a23c;
    font-weight: bold;
  }
}
</style>
