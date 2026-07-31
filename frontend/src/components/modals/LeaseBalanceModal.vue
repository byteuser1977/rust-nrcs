<template>
  <el-dialog v-model="visible" :title="t('forging.leaseBalance')" width="520px" :close-on-click-modal="false" destroy-on-close class="nrcs-modal" @close="handleClose">
    <el-alert type="warning" :closable="false" show-icon class="mb-4">
      {{ t('forging.leaseWarning') }}
    </el-alert>
    <el-form ref="formRef" :model="form" :rules="rules" label-position="top">
      <el-form-item :label="t('common.recipient')" prop="recipient">
        <el-input v-model="form.recipient" placeholder="NRCS-XXXX-XXXX-XXXX-XXXXX" clearable />
      </el-form-item>
      <el-form-item :label="t('forging.period')" prop="period">
        <el-input
          v-model="form.period"
          type="number"
          :min="MIN_PERIOD"
          :max="MAX_PERIOD"
          placeholder="65535"
          clearable
          @input="onPeriodChange"
        />
        <!-- 周期→天数换算提示（对标 nrs.modals.balanceleasing.js:26-32 setLeaseBalanceHelp） -->
        <span v-if="periodHelp" class="hint-text" :class="{ 'hint-text--error': periodExceeded }">
          {{ periodHelp }}
        </span>
      </el-form-item>
      <el-row :gutter="16">
        <el-col :span="12"><el-form-item :label="t('common.fee')" prop="feeNQT"><el-input v-model="form.feeNQT" placeholder="1" type="number" clearable><template #append>NRC</template></el-input></el-form-item></el-col>
        <el-col :span="12"><el-form-item :label="t('common.deadline')" prop="deadline"><el-input v-model="form.deadline" placeholder="1440" type="number" clearable><template #append>{{ t('common.minutes') }}</template></el-input></el-form-item></el-col>
      </el-row>
      <el-form-item :label="t('common.secretPhrase')" prop="secretPhrase">
        <el-input v-model="form.secretPhrase" type="password" show-password clearable />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="handleClose">{{ t('common.cancel') }}</el-button>
      <el-button type="primary" :loading="loading" @click="handleSubmit">{{ t('forging.leaseBalance') }}</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * LeaseBalanceModal 组件 —— 余额出租弹窗。
 *
 * 对标 nrs.modals.balanceleasing.js（50 行）的完整实现。
 *
 * 主要功能：
 *   1. 输入承租人地址、租期（区块数）、手续费、截止时间、密码短语
 *   2. 租期实时换算为天数提示（setLeaseBalanceHelp：days = period / 1440）
 *   3. 租期范围校验：最小 1440，最大 MAX_UNSIGNED_SHORT_JAVA（65535）
 *   4. 超过上限时显示错误提示（error_lease_balance_period）
 *   5. 提交后调用 leaseBalance API，成功后刷新账户信息
 *
 * 对标参考：
 *   - setLeaseBalanceHelp(period)：:26-32
 *   - modal show 时设置 min=1440/max=MAX_UNSIGNED_SHORT_JAVA：:34-39
 *   - period change 事件：:41-47
 */
import { ref, reactive, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, type FormInstance, type FormRules } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'

const { t } = useI18n()
const visible = defineModel<boolean>('visible', { default: false })
const emit = defineEmits<{ (e: 'success'): void }>()
const formRef = ref<FormInstance>()
const loading = ref(false)

/** 最小租期（区块数，对标 NRS.constants 中 lease_balance_modal 的 min 属性） */
const MIN_PERIOD = 1440
/** 最大租期（区块数，对标 NRS.constants.MAX_UNSIGNED_SHORT_JAVA = 65535） */
const MAX_PERIOD = 65535

const form = reactive({ recipient: '', period: String(MAX_PERIOD), feeNQT: '1', deadline: '1440', secretPhrase: '' })

/** 租期是否超过上限（对标 nrs.modals.balanceleasing.js:42） */
const periodExceeded = ref(false)

/**
 * 租期→天数换算提示文本（对标 setLeaseBalanceHelp：:26-32）。
 *
 * days = Math.round(period / 1440)（1440 个区块约等于 1 天）。
 * 当 period 超过 MAX_UNSIGNED_SHORT_JAVA 时返回错误提示。
 */
const periodHelp = computed<string>(() => {
  const period = Number(form.period)
  if (!form.period || isNaN(period)) return ''
  if (period > MAX_PERIOD) {
    return t('forging.leasePeriodError')
  }
  const days = Math.round(period / 1440)
  return t('forging.leasePeriodHelp', { blocks: String(period), days: String(days) })
})

const rules = computed<FormRules>(() => ({
  recipient: [{ required: true, message: t('forging.recipientRequired'), trigger: 'blur' }],
  period: [
    { required: true, message: t('forging.periodRequired'), trigger: 'blur' },
    {
      validator: (_rule: any, value: string, callback: (e?: Error) => void) => {
        const num = Number(value)
        if (isNaN(num) || num < MIN_PERIOD) {
          callback(new Error(t('forging.leasePeriodError')))
        } else if (num > MAX_PERIOD) {
          callback(new Error(t('forging.leasePeriodError')))
        } else {
          callback()
        }
      },
      trigger: 'blur',
    },
  ],
  feeNQT: [{ required: true, message: t('forging.feeRequired'), trigger: 'blur' }],
  deadline: [{ required: true, message: t('forging.deadlineRequired'), trigger: 'blur' }],
  secretPhrase: [{ required: true, message: t('forging.secretPhraseRequired'), trigger: 'blur' }],
}))

/**
 * 租期输入变化时更新超限状态（对标 nrs.modals.balanceleasing.js:41-47）。
 *
 * 当值超过 MAX_UNSIGNED_SHORT_JAVA 时标记超限，UI 显示错误样式。
 */
function onPeriodChange(): void {
  const period = Number(form.period)
  periodExceeded.value = !isNaN(period) && period > MAX_PERIOD
}

async function handleSubmit() {
  if (!formRef.value) return
  // 提交前再次校验租期上限（对标 :42-46）
  const period = Number(form.period)
  if (period > MAX_PERIOD) {
    ElMessage.error(t('forging.leasePeriodError'))
    return
  }
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    loading.value = true
    try {
      await nrcsApi.leaseBalance(form.secretPhrase, form.recipient, Number(form.period), String(Math.round(Number(form.feeNQT) * 1e8)), Number(form.deadline))
      ElMessage.success(t('forging.leaseSuccess'))
      emit('success'); handleClose()
    } catch (err: any) { ElMessage.error(err?.message || t('forging.leaseError')) }
    finally { loading.value = false }
  })
}
function handleClose() { formRef.value?.resetFields(); visible.value = false }
</script>
<style scoped lang="scss">
@use '@/assets/styles/variables' as *;
.hint-text {
  font-size: $font-size-xs;
  color: $text-muted;
  margin-top: 4px;
  display: block;
  &--error {
    color: $danger;
  }
}
.mb-4 { margin-bottom: $space-lg; }
</style>
