<template>
  <!--
    SetPhasingOnlyControlModal —— 强制审批（Phasing Only Control）设置弹窗。
    对标 html/modals/account_control.html + nrs.accountcontrol.js。

    功能：
      1. 5 种投票模型 tab 切换（无审批/按账户/按余额/按资产/按货币）
      2. 每种模型对应的字段组（quorum/whitelist/minBalance/holding）
      3. 持续时间与最大手续费（仅非"无审批"tab 显示）
      4. 高级区域：手续费/截止时间/广播/给自己留言
      5. 三步本地签名提交（复用 useNrcsForm.submitForm）
  -->
  <el-dialog
    v-model="visible"
    :title="t('accountControl.setTitle')"
    width="680px"
    :close-on-click-modal="false"
    destroy-on-close
    class="nrcs-modal"
    @closed="onClosed"
  >
    <el-alert
      type="warning"
      :closable="false"
      show-icon
      class="control-warning"
    >
      {{ t('accountControl.warningInfo') }}
    </el-alert>

    <el-form
      ref="formRef"
      :model="form"
      label-position="top"
      class="control-form"
      @submit.prevent
    >
      <!-- 投票模型 Tab 切换 -->
      <el-tabs v-model="activeTab" class="approve-tabs" @tab-change="onTabChange">
        <!-- Tab 1: 无审批 -->
        <el-tab-pane name="-1">
          <template #label>
            <span class="tab-label">
              <el-icon><Close /></el-icon>
              {{ t('accountControl.noApproval') }}
            </span>
          </template>
          <p class="tab-hint">{{ t('accountControl.noApprovalHint') }}</p>
        </el-tab-pane>

        <!-- Tab 2: 按账户 -->
        <el-tab-pane name="0">
          <template #label>
            <span class="tab-label">
              <el-icon><User /></el-icon>
              {{ t('accountControl.byAccount') }}
            </span>
          </template>
          <el-form-item :label="t('accountControl.numberAccounts')">
            <el-input-number
              v-model="form.controlQuorum"
              :min="1"
              :disabled="formLocked"
              controls-position="right"
              class="ac_ue_control_quorum"
            />
          </el-form-item>

          <!-- 白名单账户 -->
          <el-form-item :label="t('accountControl.whitelist')">
            <el-input
              v-model="whitelistText"
              type="textarea"
              :rows="2"
              :placeholder="t('accountControl.whitelistPlaceholder')"
              :disabled="formLocked"
              @input="onWhitelistInput"
            />
          </el-form-item>

          <!-- 最小余额模型 -->
          <el-row :gutter="16">
            <el-col :span="12">
              <el-form-item :label="t('accountControl.minBalanceType')">
                <el-select
                  v-model="form.controlMinBalanceModel"
                  :disabled="formLocked"
                  class="ac_ue_control_min_balance_model"
                  @change="onMinBalanceModelChange"
                >
                  <el-option :label="t('accountControl.mbNone')" :value="0" />
                  <el-option :label="t('accountControl.mbBalance')" :value="1" />
                  <el-option :label="t('accountControl.mbAsset')" :value="2" />
                  <el-option :label="t('accountControl.mbCurrency')" :value="3" />
                </el-select>
              </el-form-item>
            </el-col>
            <el-col :span="12">
              <!-- 最小余额值（根据模型切换） -->
              <el-form-item
                v-if="form.controlMinBalanceModel === 1"
                :label="t('accountControl.minBalance')"
              >
                <el-input
                  v-model="form.controlMinBalanceNXT"
                  :disabled="formLocked"
                  class="ac_ue_control_min_balance_nxt"
                >
                  <template #append>NRC</template>
                </el-input>
              </el-form-item>
              <el-form-item
                v-if="form.controlMinBalanceModel === 2"
                :label="t('accountControl.minBalance')"
              >
                <el-input
                  v-model="form.controlMinBalanceQNTf"
                  :disabled="formLocked"
                >
                  <template #append>{{ t('common.quantity') }}</template>
                </el-input>
              </el-form-item>
              <el-form-item
                v-if="form.controlMinBalanceModel === 3"
                :label="t('accountControl.minBalance')"
              >
                <el-input
                  v-model="form.controlMinBalanceQNTf"
                  :disabled="formLocked"
                >
                  <template #append>{{ t('accountControl.units') }}</template>
                </el-input>
              </el-form-item>
            </el-col>
          </el-row>

          <!-- 可选持有资产/货币（仅 minBalanceModel 为 2/3 时显示） -->
          <el-form-item
            v-if="form.controlMinBalanceModel === 2"
            :label="t('common.asset')"
          >
            <AssetInfoInput
              v-model="form.controlHolding"
              :disabled="formLocked"
              @decimals:update="onHoldingDecimalsUpdate"
            />
          </el-form-item>
          <el-form-item
            v-if="form.controlMinBalanceModel === 3"
            :label="t('accountControl.currency')"
          >
            <CurrencyInfoInput
              v-model="form.controlHoldingCurrencyCode"
              :disabled="formLocked"
              @currencyId:update="onCurrencyIdUpdate"
              @decimals:update="onHoldingDecimalsUpdate"
            />
          </el-form-item>
        </el-tab-pane>

        <!-- Tab 3: 按余额 -->
        <el-tab-pane name="1">
          <template #label>
            <span class="tab-label">
              <el-icon><Money /></el-icon>
              {{ t('accountControl.byBalance') }}
            </span>
          </template>
          <el-form-item :label="t('accountControl.amountNXT')">
            <el-input
              v-model="form.controlQuorumNXT"
              :disabled="formLocked"
              class="ac_ue_control_quorum_nxt"
            >
              <template #append>NRC</template>
            </el-input>
          </el-form-item>

          <el-form-item :label="t('accountControl.whitelist')">
            <el-input
              v-model="whitelistText"
              type="textarea"
              :rows="2"
              :placeholder="t('accountControl.whitelistPlaceholder')"
              :disabled="formLocked"
              @input="onWhitelistInput"
            />
          </el-form-item>
        </el-tab-pane>

        <!-- Tab 4: 按资产持有者 -->
        <el-tab-pane name="2">
          <template #label>
            <span class="tab-label">
              <el-icon><Coin /></el-icon>
              {{ t('accountControl.byAsset') }}
            </span>
          </template>
          <el-form-item :label="t('common.asset')">
            <AssetInfoInput
              v-model="form.controlHolding"
              :disabled="formLocked"
              @decimals:update="onHoldingDecimalsUpdate"
            />
          </el-form-item>

          <el-form-item :label="t('accountControl.assetQuantity')">
            <el-input
              v-model="form.controlQuorumQNTf"
              :disabled="formLocked"
              class="ac_ue_control_quorum_qntf"
            >
              <template #append>{{ t('common.quantity') }}</template>
            </el-input>
          </el-form-item>

          <el-form-item :label="t('accountControl.whitelist')">
            <el-input
              v-model="whitelistText"
              type="textarea"
              :rows="2"
              :placeholder="t('accountControl.whitelistPlaceholder')"
              :disabled="formLocked"
              @input="onWhitelistInput"
            />
          </el-form-item>

          <el-row :gutter="16">
            <el-col :span="12">
              <el-form-item :label="t('accountControl.minBalanceType')">
                <el-select
                  v-model="form.controlMinBalanceModel"
                  :disabled="formLocked"
                  @change="onMinBalanceModelChange"
                >
                  <el-option :label="t('accountControl.mbNone')" :value="0" />
                  <el-option :label="t('accountControl.mbAsset')" :value="2" />
                </el-select>
              </el-form-item>
            </el-col>
            <el-col :span="12">
              <el-form-item
                v-if="form.controlMinBalanceModel === 2"
                :label="t('accountControl.minBalance')"
              >
                <el-input
                  v-model="form.controlMinBalanceQNTf"
                  :disabled="formLocked"
                >
                  <template #append>{{ t('common.quantity') }}</template>
                </el-input>
              </el-form-item>
            </el-col>
          </el-row>
        </el-tab-pane>

        <!-- Tab 5: 按货币持有者 -->
        <el-tab-pane name="3">
          <template #label>
            <span class="tab-label">
              <el-icon><CreditCard /></el-icon>
              {{ t('accountControl.byCurrency') }}
            </span>
          </template>
          <el-form-item :label="t('accountControl.currency')">
            <CurrencyInfoInput
              v-model="form.controlHoldingCurrencyCode"
              :disabled="formLocked"
              @currencyId:update="onCurrencyIdUpdate"
              @decimals:update="onHoldingDecimalsUpdate"
            />
          </el-form-item>

          <el-form-item :label="t('accountControl.currencyUnits')">
            <el-input
              v-model="form.controlQuorumQNTf"
              :disabled="formLocked"
              class="ac_ue_control_quorum_qntf"
            >
              <template #append>{{ t('accountControl.units') }}</template>
            </el-input>
          </el-form-item>

          <el-form-item :label="t('accountControl.whitelist')">
            <el-input
              v-model="whitelistText"
              type="textarea"
              :rows="2"
              :placeholder="t('accountControl.whitelistPlaceholder')"
              :disabled="formLocked"
              @input="onWhitelistInput"
            />
          </el-form-item>

          <el-row :gutter="16">
            <el-col :span="12">
              <el-form-item :label="t('accountControl.minBalanceType')">
                <el-select
                  v-model="form.controlMinBalanceModel"
                  :disabled="formLocked"
                  @change="onMinBalanceModelChange"
                >
                  <el-option :label="t('accountControl.mbNone')" :value="0" />
                  <el-option :label="t('accountControl.mbCurrency')" :value="3" />
                </el-select>
              </el-form-item>
            </el-col>
            <el-col :span="12">
              <el-form-item
                v-if="form.controlMinBalanceModel === 3"
                :label="t('accountControl.minBalance')"
              >
                <el-input
                  v-model="form.controlMinBalanceQNTf"
                  :disabled="formLocked"
                >
                  <template #append>{{ t('accountControl.units') }}</template>
                </el-input>
              </el-form-item>
            </el-col>
          </el-row>
        </el-tab-pane>
      </el-tabs>

      <!-- 持续时间与最大手续费（仅非"无审批"tab 显示） -->
      <div v-if="activeTab !== '-1'" class="phasing-duration-and-fees">
        <el-divider content-position="left">
          {{ t('accountControl.durationAndFees') }}
        </el-divider>

        <el-row :gutter="16">
          <el-col :span="12">
            <el-form-item :label="t('accountControl.minDuration')">
              <el-input-number
                v-model="form.controlMinDuration"
                :min="0"
                :disabled="formLocked"
                controls-position="right"
                class="ac_ue_control_min_duration"
              />
              <span class="input-hint">{{ t('accountControl.blocks') }}</span>
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item :label="t('accountControl.maxDuration')">
              <el-input-number
                v-model="form.controlMaxDuration"
                :min="0"
                :disabled="formLocked"
                controls-position="right"
                class="ac_ue_control_max_duration"
              />
              <span class="input-hint">{{ t('accountControl.blocks') }}</span>
            </el-form-item>
          </el-col>
        </el-row>

        <el-form-item :label="t('accountControl.maxPendingFees')">
          <el-input
            v-model="form.controlMaxFeesNXT"
            :disabled="formLocked"
            :placeholder="t('accountControl.maxFeesPlaceholder')"
            class="ac_ue_control_max_fees"
          >
            <template #append>NRC</template>
          </el-input>
        </el-form-item>
      </div>

      <!-- 高级区域 -->
      <el-divider content-position="left">
        <el-link type="primary" @click="showAdvanced = !showAdvanced">
          {{ t('common.advanced') }}
          <el-icon><ArrowDown v-if="!showAdvanced" /><ArrowUp v-else /></el-icon>
        </el-link>
      </el-divider>

      <div v-show="showAdvanced" class="advanced-section">
        <el-form-item :label="t('common.fee')">
          <el-input
            v-model="form.feeNXT"
            :disabled="formLocked"
            :placeholder="t('common.fee')"
            class="ac_ue_fee"
          >
            <template #append>NRC</template>
          </el-input>
        </el-form-item>

        <el-form-item :label="t('common.deadline')">
          <DeadlinePicker
            v-model="form.deadline"
            :disabled="formLocked"
          />
        </el-form-item>

        <el-form-item>
          <BroadcastToggle
            v-model="form.doNotBroadcast"
            :disabled="formLocked"
          />
        </el-form-item>

        <el-form-item>
          <NoteToSelfInput
            v-model="noteToSelfText"
            :disabled="formLocked"
            @update:enabled="onNoteToSelfToggle"
          />
        </el-form-item>
      </div>

      <!-- 密码短语 -->
      <el-form-item :label="t('common.secretPhrase')" required>
        <el-input
          v-model="form.secretPhrase"
          type="password"
          show-password
          :disabled="formLocked"
          :placeholder="t('common.secretPhraseRequired')"
          class="ac_ue_secret_phrase"
        />
      </el-form-item>
    </el-form>

    <template #footer>
      <el-button @click="visible = false">{{ t('common.cancel') }}</el-button>
      <el-button
        type="primary"
        :loading="submitting"
        :disabled="formLocked"
        class="ac_ue_submit_btn"
        @click="handleSubmit"
      >
        {{ t('common.submit') }}
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * SetPhasingOnlyControlModal 组件 —— 设置账户强制审批（Phasing Only Control）。
 *
 * 对标 html/modals/account_control.html + nrs.accountcontrol.js：
 *   - 5 种投票模型 tab（无审批/按账户/按余额/按资产/按货币）
 *   - 每种模型对应的 quorum/whitelist/minBalance/holding 字段
 *   - 持续时间与最大手续费（controlMinDuration/controlMaxDuration/controlMaxFees）
 *   - 三步本地签名提交（复用 useNrcsForm.submitForm）
 *
 * 用于 AccountControl 页面设置或修改强制审批策略。
 */
import { ref, reactive, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import {
  Close, User, Money, Coin, CreditCard,
  ArrowDown, ArrowUp,
} from '@element-plus/icons-vue'
import {
  DeadlinePicker,
  BroadcastToggle,
  NoteToSelfInput,
  AssetInfoInput,
  CurrencyInfoInput,
} from '@/components/modals/ui-elements'
import { useNrcsForm } from '@/composables/useNrcsForm'
import { useAccountStore } from '@/stores/modules/account.store'

const props = defineProps<{ modelValue: boolean }>()
const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  success: []
}>()

const { t } = useI18n()
const accountStore = useAccountStore()
const { submitForm } = useNrcsForm()

/** 弹窗可见性 */
const visible = computed({
  get: () => props.modelValue,
  set: (val) => emit('update:modelValue', val),
})

/** 当前激活的投票模型 tab（-1=无审批/0=账户/1=余额/2=资产/3=货币） */
const activeTab = ref<string>('-1')

/** 是否显示高级区域 */
const showAdvanced = ref(false)

/** 是否正在提交 */
const submitting = ref(false)

/** 表单是否锁定（提交中） */
const formLocked = computed(() => submitting.value)

/** 白名单文本（逗号分隔） */
const whitelistText = ref('')

/** 给自己留言文本 */
const noteToSelfText = ref('')

/** 是否启用给自己留言 */
const noteToSelfEnabled = ref(false)

/** 持有资产/货币的精度（用于 QNT 转换） */
const holdingDecimals = ref(0)

/** 表单数据 */
const form = reactive({
  // 投票模型（由 tab 决定）
  controlVotingModel: -1,
  // 法定人数（按账户模式）
  controlQuorum: 1,
  // 法定人数（按余额模式，NXT 字符串）
  controlQuorumNXT: '',
  // 法定人数（按资产/货币模式，QNT 字符串）
  controlQuorumQNTf: '',
  // 白名单账户（RS 数组）
  controlWhitelisted: [] as string[],
  // 最小余额模型（0=无/1=余额/2=资产/3=货币）
  controlMinBalanceModel: 0,
  // 最小余额（NXT）
  controlMinBalanceNXT: '',
  // 最小余额（QNT）
  controlMinBalanceQNTf: '',
  // 持有资产/货币 ID
  controlHolding: '',
  // 持有货币代码
  controlHoldingCurrencyCode: '',
  // 最小持续时间（区块数）
  controlMinDuration: 0,
  // 最大持续时间（区块数）
  controlMaxDuration: 0,
  // 最大手续费（NXT）
  controlMaxFeesNXT: '',
  // 手续费（NXT）
  feeNXT: '1',
  // 截止时间（分钟）
  deadline: 1440,
  // 不广播
  doNotBroadcast: false,
  // 密码短语
  secretPhrase: '',
})

/**
 * Tab 切换时更新投票模型。
 */
function onTabChange(tabName: string): void {
  form.controlVotingModel = parseInt(tabName, 10)
  // 切换 tab 时重置 minBalanceModel 为 0（无）
  form.controlMinBalanceModel = 0
}

/**
 * 白名单输入处理（逗号分隔 → 数组）。
 */
function onWhitelistInput(val: string): void {
  form.controlWhitelisted = val
    .split(/[\n,]/)
    .map((s) => s.trim())
    .filter((s) => s.length > 0)
}

/**
 * 最小余额模型变化时清空对应值。
 */
function onMinBalanceModelChange(): void {
  form.controlMinBalanceNXT = ''
  form.controlMinBalanceQNTf = ''
  if (form.controlMinBalanceModel !== 2) form.controlHolding = ''
  if (form.controlMinBalanceModel !== 3) form.controlHoldingCurrencyCode = ''
}

/**
 * 持有资产精度更新回调。
 */
function onHoldingDecimalsUpdate(decimals: number): void {
  holdingDecimals.value = decimals
}

/**
 * 货币 ID 更新回调（CurrencyInfoInput 返回货币 ID）。
 */
function onCurrencyIdUpdate(currencyId: string): void {
  form.controlHolding = currencyId
}

/**
 * 给自己留言开关切换。
 */
function onNoteToSelfToggle(enabled: boolean): void {
  noteToSelfEnabled.value = enabled
}

/**
 * 构建提交数据。
 * 根据当前 tab 组装对应的字段，过滤无关字段。
 */
function buildSubmitData(): Record<string, any> {
  const data: Record<string, any> = {
    secretPhrase: form.secretPhrase || accountStore.secretPhrase || '',
    controlVotingModel: form.controlVotingModel,
    feeNXT: form.feeNXT,
    deadline: form.deadline,
  }

  // 无审批模式：仅需 votingModel = -1
  if (form.controlVotingModel === -1) {
    return data
  }

  // 持续时间与手续费
  data.controlMinDuration = form.controlMinDuration
  data.controlMaxDuration = form.controlMaxDuration
  data.controlMaxFeesNXT = form.controlMaxFeesNXT

  // 白名单
  if (form.controlWhitelisted.length > 0) {
    data.controlWhitelisted = form.controlWhitelisted
  }

  // 按账户模式
  if (form.controlVotingModel === 0) {
    data.controlQuorum = form.controlQuorum
  }

  // 按余额模式
  if (form.controlVotingModel === 1) {
    data.controlQuorumNXT = form.controlQuorumNXT
  }

  // 按资产/货币模式
  if (form.controlVotingModel === 2 || form.controlVotingModel === 3) {
    data.controlQuorumQNTf = form.controlQuorumQNTf
    data.controlHolding = form.controlHolding
  }

  // 最小余额
  if (form.controlMinBalanceModel > 0) {
    data.controlMinBalanceModel = form.controlMinBalanceModel
    if (form.controlMinBalanceModel === 1) {
      data.controlMinBalanceNXT = form.controlMinBalanceNXT
    } else {
      data.controlMinBalanceQNTf = form.controlMinBalanceQNTf
    }
    if (form.controlMinBalanceModel === 2 || form.controlMinBalanceModel === 3) {
      data.controlHolding = form.controlHolding
    }
  }

  // 不广播
  if (form.doNotBroadcast) {
    data.doNotBroadcast = true
  }

  // 给自己留言
  if (noteToSelfEnabled.value && noteToSelfText.value) {
    data.messageToEncryptToSelf = noteToSelfText.value
    data.encryptToSelfMessageData = noteToSelfText.value
  }

  return data
}

/**
 * 提交表单。
 * 使用 useNrcsForm.submitForm 走三步本地签名流程。
 */
async function handleSubmit(): Promise<void> {
  if (!form.secretPhrase && !accountStore.secretPhrase) {
    ElMessage.warning(t('common.secretPhraseRequired'))
    return
  }

  submitting.value = true
  try {
    const data = buildSubmitData()
    await submitForm('setPhasingOnlyControl', data, {
      successMessage: t('accountControl.submitSuccess'),
      onSuccess: () => {
        emit('success')
        visible.value = false
      },
    })
  } finally {
    submitting.value = false
  }
}

/**
 * 弹窗关闭时重置表单。
 */
function onClosed(): void {
  activeTab.value = '-1'
  showAdvanced.value = false
  whitelistText.value = ''
  noteToSelfText.value = ''
  noteToSelfEnabled.value = false
  holdingDecimals.value = 0
  form.controlVotingModel = -1
  form.controlQuorum = 1
  form.controlQuorumNXT = ''
  form.controlQuorumQNTf = ''
  form.controlWhitelisted = []
  form.controlMinBalanceModel = 0
  form.controlMinBalanceNXT = ''
  form.controlMinBalanceQNTf = ''
  form.controlHolding = ''
  form.controlHoldingCurrencyCode = ''
  form.controlMinDuration = 0
  form.controlMaxDuration = 0
  form.controlMaxFeesNXT = ''
  form.feeNXT = '1'
  form.deadline = 1440
  form.doNotBroadcast = false
  form.secretPhrase = ''
}

// 弹窗打开时预填密码短语
watch(visible, (val) => {
  if (val && accountStore.secretPhrase) {
    form.secretPhrase = accountStore.secretPhrase
  }
})
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;

.control-warning {
  margin-bottom: $space-md;
}

.control-form {
  .approve-tabs {
    margin-bottom: $space-md;

    .tab-label {
      display: inline-flex;
      align-items: center;
      gap: 4px;
      font-size: $font-size-sm;
    }
  }

  .tab-hint {
    color: $text-secondary;
    font-size: $font-size-sm;
    padding: $space-md 0;
  }

  .phasing-duration-and-fees {
    margin-top: $space-md;
  }

  .advanced-section {
    padding: $space-sm 0;
  }

  .input-hint {
    margin-left: $space-xs;
    color: $text-muted;
    font-size: $font-size-xs;
  }
}
</style>
