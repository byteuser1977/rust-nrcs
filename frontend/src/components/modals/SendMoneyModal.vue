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
        <el-input v-model="form.recipient" :placeholder="t('sendMoney.recipientPlaceholder')" clearable>
          <template #prefix>
            <el-icon><User /></el-icon>
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
import { ref, reactive, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, type FormInstance, type FormRules } from 'element-plus'
import { User, Lock } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { useAccountStore } from '@/stores/modules/account.store'

const { t } = useI18n()
const accountStore = useAccountStore()

const visible = defineModel<boolean>('visible', { default: false })
const emit = defineEmits<{ (e: 'success'): void }>()

const formRef = ref<FormInstance>()
const isSubmitting = ref(false)

const form = reactive({
  recipient: '',
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
  secretPhrase: [{ required: true, message: t('sendMoney.secretPhraseRequired'), trigger: 'blur' }]
}))

const fillMaxAmount = () => {
  const balanceNQT = accountStore.balanceNQT || '0'
  const feeNQT = Number(form.feeNQT || 1) * 1e8
  const maxAmount = Math.max(0, Number(balanceNQT) - feeNQT) / 1e8
  form.amountNQT = maxAmount.toFixed(8)
}

const handleSubmit = async () => {
  if (!formRef.value) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    isSubmitting.value = true
    try {
      const amountNQT = String(Math.round(Number(form.amountNQT) * 1e8))
      const feeNQT = String(Math.round(Number(form.feeNQT) * 1e8))
      const data: any = {
        secretPhrase: form.secretPhrase,
        recipient: form.recipient,
        amountNQT,
        feeNQT,
        deadline: Number(form.deadline)
      }
      if (form.message) {
        data.message = form.message
        data.messageIsText = form.messageIsText
      }
      await nrcsApi.sendMoney(data)
      ElMessage.success(t('sendMoney.success'))
      emit('success')
      handleClose()
    } catch (err: any) {
      ElMessage.error(err?.message || t('sendMoney.error'))
    } finally {
      isSubmitting.value = false
    }
  })
}

const handleClose = () => {
  formRef.value?.resetFields()
  Object.assign(form, {
    recipient: '',
    amountNQT: '',
    feeNQT: '1',
    deadline: '1440',
    message: '',
    messageIsText: true,
    secretPhrase: ''
  })
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

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: $space-sm;
}
</style>
