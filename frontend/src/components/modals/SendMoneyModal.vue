<template>
  <el-dialog
    v-model="visible"
    :title="t('header.sendNRC')"
    width="520px"
    :close-on-click-modal="false"
    destroy-on-close
    class="send-money-modal"
    @close="handleClose"
  >
    <el-form ref="formRef" :model="form" :rules="rules" label-width="100px" label-position="top">
      <el-form-item :label="t('sendMoney.recipient')" prop="recipient">
        <el-input
          v-model="form.recipient"
          :placeholder="t('sendMoney.recipientPlaceholder')"
          clearable
          @blur="onRecipientBlur"
          @clear="onRecipientClear"
        >
          <template #prefix>
            <el-icon><User /></el-icon>
          </template>
        </el-input>

        <!-- 收款人实时校验结果（对标 nrs.recipient.js 的 .account_info callout） -->
        <div v-if="recipientCheck.result.value" class="recipient-info" :class="recipientInfoClass">
          <div class="recipient-info-icon">
            <svg v-if="recipientCheck.result.value.type === 'success'" width="16" height="16" viewBox="0 0 24 24" fill="none">
              <path d="M22 11.08V12a10 10 0 11-5.93-9.14" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
              <path d="M22 4L12 14.01l-3-3" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
            <svg v-else-if="recipientCheck.result.value.type === 'info'" width="16" height="16" viewBox="0 0 24 24" fill="none">
              <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="2"/>
              <line x1="12" y1="16" x2="12" y2="12" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
              <circle cx="12" cy="8" r="1" fill="currentColor"/>
            </svg>
            <svg v-else-if="recipientCheck.result.value.type === 'warning'" width="16" height="16" viewBox="0 0 24 24" fill="none">
              <path d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
              <line x1="12" y1="9" x2="12" y2="13" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
              <circle cx="12" cy="17" r="1" fill="currentColor"/>
            </svg>
            <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none">
              <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="2"/>
              <line x1="15" y1="9" x2="9" y2="15" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
              <line x1="9" y1="9" x2="15" y2="15" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
            </svg>
          </div>
          <div class="recipient-info-content">
            <span class="recipient-info-message" v-html="recipientCheck.result.value.message"></span>

            <!-- 纠正建议列表（对标 nrs.recipient.js:234-246 的 .malformed_address） -->
            <div
              v-if="recipientCheck.result.value.suggestions && recipientCheck.result.value.suggestions.length > 0"
              class="suggestions-list"
            >
              <button
                v-for="(suggestion, idx) in recipientCheck.result.value.suggestions"
                :key="idx"
                class="suggestion-item"
                @click="applySuggestion(suggestion.address)"
              >
                <span v-html="suggestion.formatted"></span>
              </button>
            </div>
          </div>
        </div>

        <!-- 商家信息提示（对标 nrs.recipient.js:372-383 checkForMerchant） -->
        <div v-if="recipientCheck.result.value?.merchantInfo" class="merchant-info">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
            <path d="M3 9l1-5h16l1 5M5 9v11a1 1 0 001 1h12a1 1 0 001-1V9M9 13h6" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
          <span>该账户为商家，已自动勾选附加消息</span>
        </div>
      </el-form-item>

      <!-- 收款方公钥输入（无公钥账户时显示，对标 nrs.recipient.js:218 recipient_public_key） -->
      <el-form-item
        v-if="recipientCheck.result.value?.showRecipientPublicKey"
        :label="t('sendMoney.recipientPublicKey')"
        prop="recipientPublicKey"
      >
        <el-input
          v-model="form.recipientPublicKey"
          placeholder="收款方公钥（64 字符十六进制）"
          clearable
        >
          <template #prefix>
            <el-icon><Key /></el-icon>
          </template>
        </el-input>
      </el-form-item>

      <el-form-item :label="t('sendMoney.amount')" prop="amountNQT">
        <el-input v-model="form.amountNQT" :placeholder="t('sendMoney.amountPlaceholder')" type="number" clearable>
          <template #append>NRC</template>
        </el-input>
        <div class="balance-hint">
          <span>{{ t('sendMoney.available') }}:</span>
          <span class="balance-value" @click="fillMaxAmount">{{ accountBalance }} NRC</span>
        </div>
      </el-form-item>

      <el-form-item :label="t('sendMoney.fee')" prop="feeNQT">
        <el-input v-model="form.feeNQT" placeholder="1" type="number" clearable>
          <template #append>NRC</template>
        </el-input>
      </el-form-item>

      <el-form-item :label="t('sendMoney.deadline')" prop="deadline">
        <el-input v-model="form.deadline" :placeholder="t('sendMoney.deadlinePlaceholder')" type="number" clearable>
          <template #append>{{ t('sendMoney.minutes') }}</template>
        </el-input>
      </el-form-item>

      <el-form-item :label="t('sendMoney.message')" prop="message">
        <el-input v-model="form.message" type="textarea" :rows="3" :placeholder="t('sendMoney.messagePlaceholder')" />
      </el-form-item>

      <el-form-item>
        <el-checkbox v-model="form.messageIsText">{{ t('sendMoney.messageIsText') }}</el-checkbox>
      </el-form-item>

      <el-form-item :label="t('sendMoney.secretPhrase')" prop="secretPhrase">
        <el-input v-model="form.secretPhrase" type="password" :placeholder="t('sendMoney.secretPhrasePlaceholder')" show-password clearable>
          <template #prefix>
            <el-icon><Lock /></el-icon>
          </template>
        </el-input>
      </el-form-item>
    </el-form>

    <template #footer>
      <div class="dialog-footer">
        <el-button @click="handleClose">{{ t('common.cancel') }}</el-button>
        <el-button type="primary" :loading="isSubmitting" @click="handleSubmit">
          {{ t('sendMoney.submit') }}
        </el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * SendMoneyModal 组件 —— 发送 NRC 转账弹窗。
 *
 * 对标 nrs.forms.js 的 sendMoney 表单提交 + nrs.recipient.js 的收款人实时校验。
 *
 * 安全模型：secretPhrase 不随请求外发，通过 useNrcsForm 三步本地签名流程提交：
 *   1. 发送 doNotSign 请求获取 unsignedTransactionBytes
 *   2. 本地验证 + 签名（verifyAndSignTransactionBytes）
 *   3. 广播已签名交易（broadcastTransactionBytes）
 */
import { ref, reactive, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, type FormInstance, type FormRules } from 'element-plus'
import { User, Lock, Key } from '@element-plus/icons-vue'
import { useAccountStore } from '@/stores/modules/account.store'
import { useNrcsForm } from '@/composables/useNrcsForm'
import { useRecipientCheck } from '@/composables/useRecipientCheck'

const { t } = useI18n()
const accountStore = useAccountStore()
const recipientCheck = useRecipientCheck()
const { submitForm } = useNrcsForm()

const visible = defineModel<boolean>('visible', { default: false })

/** 外部预填收款人（如从联系人列表点击"发送"时传入 RS 地址） */
const props = defineProps<{ recipient?: string }>()

const emit = defineEmits<{ (e: 'success'): void }>()

const formRef = ref<FormInstance>()
const isSubmitting = ref(false)

const form = reactive({
  recipient: '',
  recipientPublicKey: '',
  amountNQT: '',
  feeNQT: '1',
  deadline: '1440',
  message: '',
  messageIsText: true,
  secretPhrase: ''
})

const accountBalance = computed(() => {
  const nqt = accountStore.balanceNQT || '0'
  return (Number(nqt) / 1e8).toFixed(2)
})

/** 校验结果样式类（对标 nrs.recipient.js callout-info/danger/warning） */
const recipientInfoClass = computed(() => {
  const type = recipientCheck.result.value?.type
  if (!type) return ''
  return `is-${type}`
})

/**
 * 是否需要显示 secretPhrase 输入框。
 *
 * 当 accountStore 已有 secretPhrase（password 登录模式）时隐藏输入框，
 * 直接使用内存中的值；否则显示输入框让用户手动输入。
 */
const needsSecretPhrase = computed(() => !accountStore.hasSecretPhrase)

const rules = computed<FormRules>(() => ({
  recipient: [{ required: true, message: t('sendMoney.recipientRequired'), trigger: 'blur' }],
  amountNQT: [
    { required: true, message: t('sendMoney.amountRequired'), trigger: 'blur' },
    {
      validator: (_rule: any, value: string, callback: (err?: Error) => void) => {
        const num = Number(value)
        if (isNaN(num) || num <= 0) callback(new Error(t('sendMoney.amountInvalid')))
        else callback()
      },
      trigger: 'blur'
    }
  ],
  feeNQT: [{ required: true, message: t('sendMoney.feeRequired'), trigger: 'blur' }],
  deadline: [{ required: true, message: t('sendMoney.deadlineRequired'), trigger: 'blur' }],
  secretPhrase: needsSecretPhrase.value
    ? [{ required: true, message: t('sendMoney.secretPhraseRequired'), trigger: 'blur' }]
    : [],
}))

/**
 * 收款人输入框失焦校验
 *
 * 对标 nrs.recipient.js:27-29 的 blur → checkRecipient 触发链。
 * 空输入时清空校验结果。
 */
async function onRecipientBlur(): Promise<void> {
  const value = form.recipient.trim()
  if (!value) {
    recipientCheck.reset()
    return
  }
  await recipientCheck.checkRecipient(value, {
    selfAccountRS: accountStore.accountRS,
    requestType: 'sendMoney',
  })

  // 若解析出 convertedAccount（联系人/alias 转换后的 RS），自动填充
  const converted = recipientCheck.result.value?.convertedAccount
  if (converted && converted !== value) {
    form.recipient = converted
  }

  // 商家信息检测：自动勾选消息
  if (recipientCheck.result.value?.merchantInfo && !form.message) {
    form.messageIsText = true
  }
}

/** 清空收款人输入时重置校验 */
function onRecipientClear(): void {
  recipientCheck.reset()
  form.recipientPublicKey = ''
}

/**
 * 应用纠正建议
 *
 * 对标 nrs.recipient.js:195-197 correctAddressMistake：
 * 点击建议地址后填入输入框并重新校验。
 */
async function applySuggestion(address: string): Promise<void> {
  form.recipient = address
  await recipientCheck.correctAddressMistake(address, {
    selfAccountRS: accountStore.accountRS,
    requestType: 'sendMoney',
  })
}

const fillMaxAmount = () => {
  const balanceNQT = accountStore.balanceNQT || '0'
  const feeNQT = Number(form.feeNQT || 1) * 1e8
  const maxAmount = Math.max(0, Number(balanceNQT) - feeNQT) / 1e8
  form.amountNQT = maxAmount.toFixed(8)
}

/**
 * 监听 visible 打开：若外部传入 recipient prop 则预填并触发校验
 *
 * 对标 nrs.recipient.js:51-64 的 modal show 事件：
 * 从 invoker data-account 预填收款人并触发 checkRecipient。
 */
watch(visible, async (open) => {
  if (open && props.recipient) {
    form.recipient = props.recipient
    await onRecipientBlur()
  }
})

const handleSubmit = async () => {
  if (!formRef.value) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    // 危险状态禁止提交（对标 nrs.recipient.js 中 callout-danger 时不应发送）
    if (recipientCheck.result.value?.type === 'danger') {
      ElMessage.error('收款地址校验未通过，请修正后再提交')
      return
    }
    isSubmitting.value = true
    try {
      // 获取 secretPhrase：优先 accountStore 内存值，否则用用户输入
      const secretPhrase = accountStore.hasSecretPhrase
        ? accountStore.secretPhrase
        : form.secretPhrase

      if (!secretPhrase) {
        ElMessage.warning(t('sendMoney.secretPhraseRequired'))
        return
      }

      // 构造表单数据（amountNQT/feeNQT 由 useNrcsForm 自动转换 NXT→NQT）
      const data: Record<string, any> = {
        secretPhrase,
        recipient: form.recipient,
        amountNXT: form.amountNQT,
        feeNXT: form.feeNQT,
        deadline: form.deadline,
      }
      // 若收款方无公钥且用户提供了公钥，附加到请求
      if (recipientCheck.result.value?.noPublicKey && form.recipientPublicKey) {
        data.recipientPublicKey = form.recipientPublicKey
      }
      if (form.message) {
        data.message = form.message
        data.messageIsText = form.messageIsText
      }

      // 通过 useNrcsForm 三步本地签名流程提交（secretPhrase 不外发）
      await submitForm('sendMoney', data, {
        successMessage: t('sendMoney.success'),
      })

      emit('success')
      handleClose()
    } catch (err: any) {
      // useNrcsForm 已通过 ElMessage 显示错误，此处仅记录日志
      console.error('Send money failed:', err)
    } finally {
      isSubmitting.value = false
    }
  })
}

const handleClose = () => {
  formRef.value?.resetFields()
  Object.assign(form, {
    recipient: '',
    recipientPublicKey: '',
    amountNQT: '',
    feeNQT: '1',
    deadline: '1440',
    message: '',
    messageIsText: true,
    secretPhrase: ''
  })
  recipientCheck.reset()
  visible.value = false
}

defineExpose({ visible })
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;

.balance-hint {
  margin-top: 4px;
  font-size: $font-size-xs;
  color: $text-muted;

  .balance-value {
    color: $primary;
    cursor: pointer;
    font-weight: 600;
    margin-left: 4px;

    &:hover {
      text-decoration: underline;
    }
  }
}

.recipient-info {
  display: flex;
  align-items: flex-start;
  gap: $space-sm;
  margin-top: $space-sm;
  padding: $space-sm $space-md;
  border-radius: $radius-sm;
  font-size: $font-size-xs;
  line-height: 1.5;
  border: 1px solid transparent;

  .recipient-info-icon {
    flex-shrink: 0;
    margin-top: 1px;
  }

  .recipient-info-content {
    flex: 1;
    word-break: break-word;
  }

  .recipient-info-message {
    display: block;
  }

  .suggestions-list {
    margin-top: $space-xs;
    display: flex;
    flex-direction: column;
    gap: $space-xs;

    .suggestion-item {
      background: transparent;
      border: 1px dashed currentColor;
      border-radius: $radius-sm;
      padding: $space-xs $space-sm;
      cursor: pointer;
      font-family: 'Courier New', monospace;
      font-size: $font-size-xs;
      text-align: left;
      transition: all $duration-fast;

      &:hover {
        background: rgba(255, 255, 255, 0.08);
      }

      :deep(b) {
        color: #fff;
        font-weight: 700;
      }
    }
  }

  // 状态样式（对标 nrs.recipient.js callout-info/danger/warning/success）
  &.is-info {
    background: rgba(59, 130, 246, 0.1);
    border-color: rgba(59, 130, 246, 0.3);
    color: #60a5fa;
  }

  &.is-warning {
    background: $warning-subtle;
    border-color: rgba($warning, 0.3);
    color: $warning;
  }

  &.is-danger {
    background: $danger-subtle;
    border-color: rgba($danger, 0.3);
    color: $danger;
  }

  &.is-success {
    background: $success-subtle;
    border-color: rgba($success, 0.3);
    color: $success;
  }
}

.merchant-info {
  display: flex;
  align-items: center;
  gap: $space-xs;
  margin-top: $space-xs;
  font-size: $font-size-xs;
  color: $warning;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: $space-sm;
}
</style>
