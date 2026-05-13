<template>
  <el-dialog
    v-model="visible"
    :title="t('header.sendMessage')"
    width="520px"
    :close-on-click-modal="false"
    destroy-on-close
    class="send-message-modal"
    @close="handleClose"
  >
    <el-form ref="formRef" :model="form" :rules="rules" label-width="100px" label-position="top">
      <el-form-item :label="t('sendMessage.recipient')" prop="recipient">
        <el-input v-model="form.recipient" :placeholder="t('sendMessage.recipientPlaceholder')" clearable>
          <template #prefix>
            <el-icon><User /></el-icon>
          </template>
        </el-input>
      </el-form-item>

      <el-form-item :label="t('sendMessage.message')" prop="message">
        <el-input v-model="form.message" type="textarea" :rows="5" :placeholder="t('sendMessage.messagePlaceholder')" />
      </el-form-item>

      <el-form-item>
        <div class="options-row">
          <el-checkbox v-model="form.messageIsText">{{ t('sendMessage.messageIsText') }}</el-checkbox>
          <el-checkbox v-model="form.encryptMessage">{{ t('sendMessage.encryptMessage') }}</el-checkbox>
        </div>
      </el-form-item>

      <el-form-item :label="t('sendMessage.fee')" prop="feeNQT">
        <el-input v-model="form.feeNQT" placeholder="1" type="number" clearable>
          <template #append>NRC</template>
        </el-input>
      </el-form-item>

      <el-form-item :label="t('sendMessage.deadline')" prop="deadline">
        <el-input v-model="form.deadline" :placeholder="t('sendMessage.deadlinePlaceholder')" type="number" clearable>
          <template #append>{{ t('sendMessage.minutes') }}</template>
        </el-input>
      </el-form-item>

      <el-form-item :label="t('sendMessage.secretPhrase')" prop="secretPhrase">
        <el-input v-model="form.secretPhrase" type="password" :placeholder="t('sendMessage.secretPhrasePlaceholder')" show-password clearable>
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
          {{ t('sendMessage.submit') }}
        </el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, type FormInstance, type FormRules } from 'element-plus'
import { User, Lock } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'

const { t } = useI18n()

const visible = defineModel<boolean>('visible', { default: false })
const emit = defineEmits<{ (e: 'success'): void }>()

const formRef = ref<FormInstance>()
const isSubmitting = ref(false)

const form = reactive({
  recipient: '',
  message: '',
  messageIsText: true,
  encryptMessage: true,
  feeNQT: '1',
  deadline: '1440',
  secretPhrase: ''
})

const rules = computed<FormRules>(() => ({
  recipient: [{ required: true, message: t('sendMessage.recipientRequired'), trigger: 'blur' }],
  message: [{ required: true, message: t('sendMessage.messageRequired'), trigger: 'blur' }],
  feeNQT: [{ required: true, message: t('sendMessage.feeRequired'), trigger: 'blur' }],
  deadline: [{ required: true, message: t('sendMessage.deadlineRequired'), trigger: 'blur' }],
  secretPhrase: [{ required: true, message: t('sendMessage.secretPhraseRequired'), trigger: 'blur' }]
}))

import { computed } from 'vue'

const handleSubmit = async () => {
  if (!formRef.value) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    isSubmitting.value = true
    try {
      const feeNQT = String(Math.round(Number(form.feeNQT) * 1e8))
      const data: any = {
        secretPhrase: form.secretPhrase,
        recipient: form.recipient,
        feeNQT,
        deadline: Number(form.deadline),
        message: form.message,
        messageIsText: form.messageIsText
      }
      if (form.encryptMessage) {
        data.messageToEncrypt = form.message
      }
      await nrcsApi.sendMessage(data)
      ElMessage.success(t('sendMessage.success'))
      emit('success')
      handleClose()
    } catch (err: any) {
      ElMessage.error(err?.message || t('sendMessage.error'))
    } finally {
      isSubmitting.value = false
    }
  })
}

const handleClose = () => {
  formRef.value?.resetFields()
  Object.assign(form, {
    recipient: '',
    message: '',
    messageIsText: true,
    encryptMessage: true,
    feeNQT: '1',
    deadline: '1440',
    secretPhrase: ''
  })
  visible.value = false
}

defineExpose({ visible })
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;

.options-row {
  display: flex;
  gap: $space-lg;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: $space-sm;
}
</style>
