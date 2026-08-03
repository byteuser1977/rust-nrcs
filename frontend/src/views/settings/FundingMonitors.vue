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

    <!-- 监控列表（对标 nrs.monitors.js:72 NRS.pages.funding_monitors） -->
    <el-card shadow="hover" v-loading="isLoading">
      <template v-if="monitors.length === 0 && !isLoading">
        <el-empty :description="t('settings.noActiveMonitors')" />
      </template>
      <el-table v-else :data="monitors" style="width: 100%" :empty-text="t('common.noData')">
        <el-table-column :label="t('common.account')" min-width="180">
          <template #default="{ row }">
            <span class="mono-text cursor-pointer" @click="showMonitorStatus(row)">{{ row.accountRS || truncateHash(row.account, 8) }}</span>
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
          <template #default>
            <el-tag size="small" type="success">{{ t('common.active') }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.actions')" width="160" fixed="right">
          <template #default="{ row }">
            <el-button size="small" type="primary" link @click="showMonitorStatus(row)">
              {{ t('settings.monitorStatus') }}
            </el-button>
            <el-button size="small" type="danger" link @click="openStopDialog(row)">
              {{ t('settings.stopMonitor') }}
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- 启动监控弹窗（对标 nrs.monitors.js:115 startFundingMonitor） -->
    <el-dialog v-model="startDialogVisible" :title="t('settings.startMonitor')" width="520px" destroy-on-close>
      <el-form ref="startFormRef" :model="monitorForm" :rules="monitorRules" label-position="top">
        <el-form-item :label="t('settings.property')" prop="property">
          <el-input v-model="monitorForm.property" placeholder="funding" />
        </el-form-item>
        <el-row :gutter="16">
          <el-col :span="12">
            <el-form-item :label="t('common.amount') + ' (NRC)'" prop="amount">
              <el-input v-model="monitorForm.amount" placeholder="0.00" type="number" />
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item :label="t('settings.threshold') + ' (NRC)'" prop="threshold">
              <el-input v-model="monitorForm.threshold" placeholder="0.00" type="number" />
            </el-form-item>
          </el-col>
        </el-row>
        <el-row :gutter="16">
          <el-col :span="12">
            <el-form-item :label="t('settings.interval') + ' (' + t('settings.numberOfBlocks') + ')'" prop="interval">
              <el-input-number v-model="monitorForm.interval" :min="1" :max="100000" controls-position="right" style="width:100%" />
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item :label="t('common.fee') + ' (NRC)'" prop="feeNQT">
              <el-input v-model="monitorForm.feeNQT" placeholder="1" type="number" />
            </el-form-item>
          </el-col>
        </el-row>
        <el-form-item v-if="needsSecretPhrase" :label="t('common.secretPhrase')" prop="secretPhrase">
          <el-input v-model="monitorForm.secretPhrase" type="password" show-password />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="startDialogVisible = false">{{ t('common.cancel') }}</el-button>
        <el-button type="primary" @click="submitStartMonitor" :loading="isStarting">
          {{ t('common.submit') }}
        </el-button>
      </template>
    </el-dialog>

    <!-- 停止监控弹窗（对标 nrs.monitors.js:120 stop_funding_monitor_modal） -->
    <el-dialog v-model="stopDialogVisible" :title="t('settings.stopMonitor')" width="460px" destroy-on-close>
      <el-form ref="stopFormRef" :model="stopForm" :rules="stopRules" label-position="top">
        <el-form-item :label="t('common.account')">
          <el-input v-model="stopForm.account" disabled />
        </el-form-item>
        <el-form-item :label="t('settings.property')">
          <el-input v-model="stopForm.property" disabled />
        </el-form-item>
        <el-form-item v-if="needsSecretPhrase" :label="t('common.secretPhrase')" prop="secretPhrase">
          <el-input v-model="stopForm.secretPhrase" type="password" show-password />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="stopDialogVisible = false">{{ t('common.cancel') }}</el-button>
        <el-button type="danger" @click="submitStopMonitor" :loading="isStopping">
          {{ t('settings.stopMonitor') }}
        </el-button>
      </template>
    </el-dialog>

    <!-- 监控状态详情抽屉（对标 nrs.monitors.js:140 goToMonitor + funding_monitor_status） -->
    <el-drawer
      v-model="statusDrawerVisible"
      :title="t('settings.fundingMonitorStatus')"
      direction="rtl"
      size="640px"
      destroy-on-close
    >
      <div v-if="currentMonitor" class="monitor-status-container">
        <el-descriptions :column="1" border class="monitor-info">
          <el-descriptions-item :label="t('settings.fundingAccount')">
            <span class="mono-text">{{ currentMonitor.accountRS || currentMonitor.account }}</span>
          </el-descriptions-item>
          <el-descriptions-item :label="t('settings.controlProperty')">
            {{ currentMonitor.property }}
          </el-descriptions-item>
          <el-descriptions-item :label="t('common.amount')">
            {{ formatNrc(currentMonitor.amount) }} NRC
          </el-descriptions-item>
          <el-descriptions-item :label="t('settings.threshold')">
            {{ formatNrc(currentMonitor.threshold) }} NRC
          </el-descriptions-item>
          <el-descriptions-item :label="t('settings.interval')">
            {{ currentMonitor.interval }} {{ t('settings.numberOfBlocks') }}
          </el-descriptions-item>
        </el-descriptions>

        <div class="monitored-section-header">
          <span>{{ t('settings.monitoredAccounts') }}</span>
          <el-button type="primary" size="small" @click="openAddMonitoredDialog">
            <el-icon><Plus /></el-icon>
            {{ t('settings.addMonitoredAccount') }}
          </el-button>
        </div>

        <el-table
          v-loading="isLoadingMonitored"
          :data="monitoredAccounts"
          style="width: 100%"
          :empty-text="t('settings.noMonitoredAccounts')"
          size="small"
        >
          <el-table-column :label="t('common.account')" min-width="180">
            <template #default="{ row }">
              <span class="mono-text">{{ row.recipientRS || row.recipient }}</span>
            </template>
          </el-table-column>
          <el-table-column :label="t('common.amount')" width="120" align="right">
            <template #default="{ row }">
              {{ getMonitoredValue(row, 'amount', currentMonitor.amount) }}
            </template>
          </el-table-column>
          <el-table-column :label="t('settings.threshold')" width="120" align="right">
            <template #default="{ row }">
              {{ getMonitoredValue(row, 'threshold', currentMonitor.threshold) }}
            </template>
          </el-table-column>
          <el-table-column :label="t('settings.interval')" width="100" align="right">
            <template #default="{ row }">
              {{ getMonitoredValue(row, 'interval', currentMonitor.interval) }}
            </template>
          </el-table-column>
          <el-table-column :label="t('common.actions')" width="80" fixed="right">
            <template #default="{ row }">
              <el-popconfirm
                :title="t('settings.removeMonitoredAccountConfirm')"
                @confirm="removeMonitoredAccount(row)"
              >
                <template #reference>
                  <el-button size="small" type="danger" link>
                    {{ t('settings.removeMonitoredAccount') }}
                  </el-button>
                </template>
              </el-popconfirm>
            </template>
          </el-table-column>
        </el-table>
      </div>
    </el-drawer>

    <!-- 添加被监控账户弹窗（对标 nrs.monitors.js:198 add_monitored_account_modal） -->
    <el-dialog v-model="addMonitoredDialogVisible" :title="t('settings.addMonitoredAccount')" width="500px" destroy-on-close>
      <el-form ref="addMonitoredFormRef" :model="addMonitoredForm" :rules="addMonitoredRules" label-position="top">
        <el-form-item :label="t('common.recipient')" prop="recipient">
          <el-input v-model="addMonitoredForm.recipient" :placeholder="t('contacts.accountRSPlaceholder')" />
        </el-form-item>
        <el-row :gutter="16">
          <el-col :span="12">
            <el-form-item :label="t('common.amount') + ' (NRC)'">
              <el-input v-model="addMonitoredForm.amount" placeholder="0.00" type="number" />
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item :label="t('settings.threshold') + ' (NRC)'">
              <el-input v-model="addMonitoredForm.threshold" placeholder="0.00" type="number" />
            </el-form-item>
          </el-col>
        </el-row>
        <el-form-item :label="t('settings.interval') + ' (' + t('settings.numberOfBlocks') + ')'">
          <el-input-number v-model="addMonitoredForm.interval" :min="1" :max="100000" controls-position="right" style="width:100%" />
        </el-form-item>
        <el-form-item v-if="needsSecretPhrase" :label="t('common.secretPhrase')" prop="secretPhrase">
          <el-input v-model="addMonitoredForm.secretPhrase" type="password" show-password />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="addMonitoredDialogVisible = false">{{ t('common.cancel') }}</el-button>
        <el-button type="primary" @click="submitAddMonitoredAccount" :loading="isAddingMonitored">
          {{ t('common.submit') }}
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
/**
 * FundingMonitors.vue —— 资金监控页面。
 *
 * 对标 nrs.monitors.js 的完整功能：
 *   - 列表展示当前账户的资金监控（getFundingMonitor + adminPassword）
 *   - 启动监控（startFundingMonitor，含 property/amount/threshold/interval）
 *   - 停止监控（stopFundingMonitor，含 secretPhrase/adminPassword）
 *   - 监控状态详情（funding_monitor_status，展示被监控账户列表）
 *   - 添加被监控账户（setAccountProperty，value 为 JSON 含 amount/threshold/interval）
 *   - 移除被监控账户（deleteAccountProperty）
 *   - 区块更新自动刷新（incoming.funding_monitors）
 *
 * 安全模型：secretPhrase 不随请求外发，通过 useNrcsForm 本地签名。
 */
import { ref, reactive, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, type FormInstance, type FormRules } from 'element-plus'
import { View, Plus, Refresh } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { formatNrc, truncateHash } from '@/utils/format'
import { useAccountStore } from '@/stores/modules/account.store'
import { useNodeStore } from '@/stores/modules/node.store'
import { useNrcsForm } from '@/composables/useNrcsForm'
import { getAdminPassword } from '@/utils/feature-detection'

const { t } = useI18n()
const accountStore = useAccountStore()
const nodeStore = useNodeStore()
const { submitForm } = useNrcsForm()

const isLoading = ref(false)
const isStarting = ref(false)
const isStopping = ref(false)
const isLoadingMonitored = ref(false)
const isAddingMonitored = ref(false)
const monitors = ref<any[]>([])
const monitoredAccounts = ref<any[]>([])
const currentMonitor = ref<any>(null)

const startDialogVisible = ref(false)
const stopDialogVisible = ref(false)
const statusDrawerVisible = ref(false)
const addMonitoredDialogVisible = ref(false)

const startFormRef = ref<FormInstance>()
const stopFormRef = ref<FormInstance>()
const addMonitoredFormRef = ref<FormInstance>()

const needsSecretPhrase = computed(() => !accountStore.hasSecretPhrase)

const monitorForm = reactive({
  property: '',
  amount: '',
  threshold: '',
  interval: 100,
  feeNQT: '1',
  secretPhrase: ''
})

const stopForm = reactive({
  account: '',
  property: '',
  secretPhrase: ''
})

const addMonitoredForm = reactive({
  recipient: '',
  amount: '',
  threshold: '',
  interval: 100,
  secretPhrase: ''
})

const monitorRules = computed<FormRules>(() => ({
  property: [{ required: true, message: t('validation.required'), trigger: 'blur' }],
  amount: [{ required: true, message: t('validation.required'), trigger: 'blur' }],
  feeNQT: [{ required: true, message: t('validation.required'), trigger: 'blur' }],
  secretPhrase: needsSecretPhrase.value
    ? [{ required: true, message: t('common.secretPhraseRequired'), trigger: 'blur' }]
    : []
}))

const stopRules = computed<FormRules>(() => ({
  secretPhrase: needsSecretPhrase.value
    ? [{ required: true, message: t('common.secretPhraseRequired'), trigger: 'blur' }]
    : []
}))

const addMonitoredRules = computed<FormRules>(() => ({
  recipient: [{ required: true, message: t('validation.invalidAddress'), trigger: 'blur' }],
  secretPhrase: needsSecretPhrase.value
    ? [{ required: true, message: t('common.secretPhraseRequired'), trigger: 'blur' }]
    : []
}))

onMounted(() => {
  refreshData()
  // 注册区块更新回调（对标 nrs.monitors.js:68 NRS.incoming.funding_monitors）
  // node store 回调目前无反向注销 API，依赖 clearCallbacks 在登出时统一清理
  nodeStore.onBlockChanged(() => {
    // 仅在抽屉关闭时自动刷新监控列表，避免打断用户操作
    if (!statusDrawerVisible.value) {
      refreshData()
    }
  })
})

/**
 * 加载资金监控列表（对标 nrs.monitors.js:72 NRS.pages.funding_monitors）。
 *
 * 使用 adminPassword 查询本地节点上的所有监控。
 */
async function refreshData(): Promise<void> {
  isLoading.value = true
  try {
    const adminPassword = getAdminPassword()
    const result = await nrcsApi.getFundingMonitor({
      account: accountStore.accountRS,
      adminPassword: adminPassword || undefined
    })
    monitors.value = result.monitors || []
  } catch (error: any) {
    console.error('Failed to load monitors:', error)
    // 不弹错误提示，避免轮询时刷屏
  } finally {
    isLoading.value = false
  }
}

/**
 * 打开启动监控弹窗。
 */
function showStartDialog(): void {
  Object.assign(monitorForm, {
    property: '',
    amount: '',
    threshold: '',
    interval: 100,
    feeNQT: '1',
    secretPhrase: accountStore.secretPhrase || ''
  })
  startDialogVisible.value = true
}

/**
 * 提交启动监控（对标 nrs.monitors.js:115 startFundingMonitor）。
 *
 * 通过 useNrcsForm 本地签名，secretPhrase 不外发。
 */
async function submitStartMonitor(): Promise<void> {
  if (!startFormRef.value) return
  await startFormRef.value.validate(async (valid) => {
    if (!valid) return
    isStarting.value = true
    try {
      await submitForm('startFundingMonitor', {
        property: monitorForm.property,
        amount: monitorForm.amount,
        threshold: monitorForm.threshold || '0',
        interval: monitorForm.interval,
        feeNXT: monitorForm.feeNQT,
        deadline: '1440',
        secretPhrase: monitorForm.secretPhrase
      }, {
        successMessage: t('settings.monitorStarted')
      })
      startDialogVisible.value = false
      refreshData()
    } catch (e: any) {
      ElMessage.error(e?.message || t('settings.startMonitorError'))
    } finally {
      isStarting.value = false
    }
  })
}

/**
 * 打开停止监控弹窗（对标 nrs.monitors.js:120 stop_funding_monitor_modal show.bs.modal）。
 */
function openStopDialog(monitor: any): void {
  Object.assign(stopForm, {
    account: monitor.accountRS || monitor.account,
    property: monitor.property,
    secretPhrase: accountStore.secretPhrase || ''
  })
  stopDialogVisible.value = true
}

/**
 * 提交停止监控（对标 nrs.monitors.js:135 stopFundingMonitor）。
 *
 * 通过 useNrcsForm 本地签名，secretPhrase 不外发。
 */
async function submitStopMonitor(): Promise<void> {
  if (!stopFormRef.value) return
  await stopFormRef.value.validate(async (valid) => {
    if (!valid) return
    isStopping.value = true
    try {
      await submitForm('stopFundingMonitor', {
        property: stopForm.property,
        account: stopForm.account,
        feeNXT: '1',
        deadline: '1440',
        secretPhrase: stopForm.secretPhrase
      }, {
        successMessage: t('settings.monitorStopped')
      })
      stopDialogVisible.value = false
      refreshData()
    } catch (e: any) {
      ElMessage.error(e?.message || t('settings.stopMonitorError'))
    } finally {
      isStopping.value = false
    }
  })
}

/**
 * 展示监控状态详情（对标 nrs.monitors.js:140 goToMonitor + funding_monitor_status）。
 *
 * 从 getAccountProperties 查询被监控账户列表（setter=监控账户, property=监控属性）。
 */
async function showMonitorStatus(monitor: any): Promise<void> {
  currentMonitor.value = monitor
  statusDrawerVisible.value = true
  await loadMonitoredAccounts()
}

/**
 * 加载被监控账户列表（对标 nrs.monitors.js:150-196 NRS.pages.funding_monitor_status）。
 *
 * 调用 getAccountProperties，参数 setter=监控账户、property=监控属性。
 */
async function loadMonitoredAccounts(): Promise<void> {
  if (!currentMonitor.value) return
  isLoadingMonitored.value = true
  try {
    const result = await nrcsApi.getAccountProperties({
      setter: currentMonitor.value.account,
      property: currentMonitor.value.property
    })
    monitoredAccounts.value = result.properties || []
  } catch (error: any) {
    console.error('Failed to load monitored accounts:', error)
    monitoredAccounts.value = []
  } finally {
    isLoadingMonitored.value = false
  }
}

/**
 * 获取被监控账户的覆盖值（对标 nrs.monitors.js:49-66 jsondata.monitoredAccount）。
 *
 * 被监控账户的 value 字段为 JSON，包含 amount/threshold/interval 的覆盖值。
 * 若 value 中存在覆盖值则用覆盖值，否则用监控器的默认值。
 */
function getMonitoredValue(row: any, field: string, defaultValue: any): string {
  try {
    const value = row.value ? JSON.parse(row.value) : {}
    if (value[field] != null) {
      if (field === 'amount' || field === 'threshold') {
        return formatNrc(value[field])
      }
      return String(value[field])
    }
  } catch {
    // JSON 解析失败，使用默认值
  }
  if (field === 'amount' || field === 'threshold') {
    return formatNrc(defaultValue)
  }
  return String(defaultValue ?? '-')
}

/**
 * 打开添加被监控账户弹窗（对标 nrs.monitors.js:198 add_monitored_account_modal）。
 *
 * 预填当前监控器的默认值。
 */
function openAddMonitoredDialog(): void {
  if (!currentMonitor.value) return
  Object.assign(addMonitoredForm, {
    recipient: '',
    amount: (Number(currentMonitor.value.amount) / 1e8).toString(),
    threshold: (Number(currentMonitor.value.threshold) / 1e8).toString(),
    interval: currentMonitor.value.interval,
    secretPhrase: accountStore.secretPhrase || ''
  })
  addMonitoredDialogVisible.value = true
}

/**
 * 提交添加被监控账户（对标 nrs.monitors.js:198-229 add_monitored_account_modal）。
 *
 * 通过 setAccountProperty 设置属性，value 为 JSON 含 amount/threshold/interval。
 * 若所有值都等于监控器默认值，则 value 为空字符串。
 */
async function submitAddMonitoredAccount(): Promise<void> {
  if (!addMonitoredFormRef.value || !currentMonitor.value) return
  await addMonitoredFormRef.value.validate(async (valid) => {
    if (!valid) return
    isAddingMonitored.value = true
    try {
      // 构造 value JSON（对标 nrs.monitors.js:210-228）
      const value: Record<string, any> = {}
      const defaultAmountNqt = String(currentMonitor.value.amount)
      const defaultThresholdNqt = String(currentMonitor.value.threshold)
      const defaultInterval = currentMonitor.value.interval

      const inputAmountNqt = String(Math.round(Number(addMonitoredForm.amount) * 1e8))
      const inputThresholdNqt = String(Math.round(Number(addMonitoredForm.threshold) * 1e8))

      if (inputAmountNqt !== defaultAmountNqt) value.amount = inputAmountNqt
      if (inputThresholdNqt !== defaultThresholdNqt) value.threshold = inputThresholdNqt
      if (addMonitoredForm.interval !== defaultInterval) value.interval = addMonitoredForm.interval

      const valueStr = Object.keys(value).length > 0 ? JSON.stringify(value) : ''

      await submitForm('setAccountProperty', {
        recipient: addMonitoredForm.recipient,
        property: currentMonitor.value.property,
        value: valueStr,
        feeNXT: '1',
        deadline: '1440',
        secretPhrase: addMonitoredForm.secretPhrase
      }, {
        successMessage: t('settings.addMonitoredAccountSuccess')
      })

      addMonitoredDialogVisible.value = false
      await loadMonitoredAccounts()
    } catch (e: any) {
      ElMessage.error(e?.message || t('settings.addMonitoredAccountError'))
    } finally {
      isAddingMonitored.value = false
    }
  })
}

/**
 * 移除被监控账户（对标 nrs.monitors.js:231 remove_monitored_account_modal）。
 *
 * 通过 deleteAccountProperty 删除属性。
 */
async function removeMonitoredAccount(row: any): Promise<void> {
  if (!currentMonitor.value) return
  try {
    const secretPhrase = accountStore.secretPhrase || ''
    if (!secretPhrase) {
      ElMessage.warning(t('dashboard.enterSecretPhrase'))
      return
    }
    await submitForm('deleteAccountProperty', {
      recipient: row.recipient,
      property: currentMonitor.value.property,
      feeNXT: '1',
      deadline: '1440',
      secretPhrase
    }, {
      successMessage: t('settings.removeMonitoredAccountSuccess')
    })
    await loadMonitoredAccounts()
  } catch (e: any) {
    ElMessage.error(e?.message || t('settings.removeMonitoredAccountError'))
  }
}
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;

.page-container {
  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: $space-lg;
    flex-wrap: wrap;
    gap: $space-sm;
    .page-title {
      font-size: $font-size-lg;
      font-weight: 600;
      display: flex;
      align-items: center;
      gap: $space-sm;
      margin: 0;
      color: $text-primary;
    }
    .header-actions {
      display: flex;
      gap: $space-sm;
      flex-wrap: wrap;
    }
  }
}

.mono-text {
  font-family: $font-mono;
  font-size: $font-size-sm;
  color: $text-secondary;
}

.cursor-pointer {
  cursor: pointer;
  &:hover {
    color: $primary;
  }
}

.monitor-status-container {
  padding: 0 $space-md;

  .monitor-info {
    margin-bottom: $space-lg;
  }

  .monitored-section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: $space-md;
    font-size: $font-size-md;
    font-weight: 600;
    color: $text-primary;
  }
}
</style>
