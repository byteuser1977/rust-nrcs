<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Cpu /></el-icon> {{ t('settings.generators') }}</h2>
      <div class="header-actions">
        <el-button type="success" size="small" @click="startForgingAction">
          <el-icon><VideoPlay /></el-icon> {{ t('settings.startForging') }}
        </el-button>
        <el-button type="danger" size="small" @click="stopForgingAction">
          <el-icon><VideoPause /></el-icon> {{ t('settings.stopForging') }}
        </el-button>
        <el-button type="primary" size="small" @click="refreshNow">
          <el-icon><Refresh /></el-icon> {{ t('common.refresh') }}
        </el-button>
      </div>
    </div>

    <el-row :gutter="16" class="stats-row">
      <el-col :xs="12" :sm="8" v-for="s in genStats" :key="s.label">
        <div class="stat-box">
          <div class="stat-value">{{ s.value }}</div>
          <div class="stat-label">{{ s.label }}</div>
        </div>
      </el-col>
    </el-row>

    <el-card shadow="hover" style="margin-top: 16px">
      <el-table :data="generators" style="width: 100%" v-loading="isLoading" :empty-text="t('common.noData')">
        <el-table-column :label="t('common.account')" min-width="200">
          <template #default="{ row }">
            <span class="mono-text">{{ row.accountRS }}</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('settings.effectiveBalance') + ' (NRC)'" width="160" align="right">
          <template #default="{ row }">
            {{ row.effectiveBalanceNXT != null ? formatAmount(row.effectiveBalanceNXT) : '-' }}
          </template>
        </el-table-column>
        <el-table-column :label="t('settings.hitTime')" width="100" align="right">
          <template #default="{ row }">
            <span class="mono-text">{{ row.hitTime }}</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('settings.deadline')" width="100" align="right">
          <template #default="{ row }">
            <span class="mono-text">{{ row.deadline }}</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('settings.countdown')" width="130">
          <template #default="{ row }">
            <span :class="['countdown', row.deadline < 30 ? 'countdown-soon' : '']">
              {{ countdownText(row.deadline) }}
            </span>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- Start/Stop Forging Passphrase Dialog -->
    <el-dialog
      v-model="passphraseDialogVisible"
      :title="passphraseAction === 'start' ? t('settings.startForging') : t('settings.stopForging')"
      width="420px"
      destroy-on-close
    >
      <el-form label-width="130px">
        <el-form-item :label="t('common.secretPhrase')">
          <el-input v-model="passphrase" type="password" show-password :placeholder="t('settings.enterSecretPhrase')" />
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
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { usePolling } from '@/composables/usePolling'
import { formatTimestamp } from '@/utils/format'
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

const genStats = computed(() => [
  {
    label: t('dashboard.height'),
    value: blockHeight.value ? `#${blockHeight.value.toLocaleString()}` : '-',
  },
  {
    label: t('settings.activeForgers'),
    value: generators.value.length.toString(),
  },
  {
    label: t('settings.lastBlock'),
    value: blockTime.value ? formatTimestamp(blockTime.value) : '-',
  },
])

async function fetchGenerators() {
  try {
    const [forging, status] = await Promise.allSettled([
      nrcsApi.getForging(),
      nrcsApi.getBlockchainStatus(),
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
  }
}

async function refreshData() {
  isLoading.value = true
  try {
    await fetchGenerators()
  } finally {
    isLoading.value = false
  }
}

const { refreshNow } = usePolling(refreshData, 10000, true)

function countdownText(deadline: number): string {
  if (deadline <= 0) return t('settings.now')
  const mins = Math.floor(deadline / 60)
  const secs = deadline % 60
  if (mins > 0) return `${mins}m ${secs}s`
  return `${secs}s`
}

function formatAmount(value: number | undefined): string {
  if (value == null) return '-'
  return value.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })
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
  if (!passphrase.value.trim()) {
    ElMessage.warning(t('settings.enterSecretPhrase'))
    return
  }
  isForgingAction.value = true
  try {
    if (passphraseAction.value === 'start') {
      await nrcsApi.startForging(passphrase.value.trim())
      ElMessage.success(t('settings.forgingStarted'))
    } else {
      await nrcsApi.stopForging(passphrase.value.trim())
      ElMessage.success(t('settings.forgingStopped'))
    }
    passphraseDialogVisible.value = false
    await fetchGenerators()
  } catch (e: any) {
    ElMessage.error(e?.description || t('common.operationFailed'))
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
