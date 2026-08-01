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

      <el-form-item v-if="needsSecretPhrase" :label="t('sendMessage.secretPhrase')" prop="secretPhrase">
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
/**
 * SendMessageModal 组件 —— 发送消息弹窗。
 *
 * 对标 nrs.messages.js 的 sendMessage 表单提交。
 *
 * 安全模型：secretPhrase 不随请求外发，通过 useNrcsForm 三步本地签名流程提交：
 *   1. 发送 doNotSign 请求获取 unsignedTransactionBytes
 *   2. 本地验证 + 签名（verifyAndSignTransactionBytes）
 *   3. 广播已签名交易（broadcastTransactionBytes）
 */
import { ref, reactive, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, type FormInstance, type FormRules } from 'element-plus'
import { User, Lock } from '@element-plus/icons-vue'
import { useAccountStore } from '@/stores/modules/account.store'
import { useNrcsForm } from '@/composables/useNrcsForm'

const { t } = useI18n()
const accountStore = useAccountStore()
const { submitForm } = useNrcsForm()

const visible = defineModel<boolean>('visible', { default: false })

/** 外部预填收款人（如从联系人列表点击"发送消息"时传入 RS 地址） */
const props = defineProps<{ recipient?: string }>()

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

/**
 * 弹窗打开时预填收款人（对标 nrs.messages.js 从联系人入口打开 send_message_modal）。
 */
watch(visible, (open) => {
  if (open && props.recipient) {
    form.recipient = props.recipient
  }
})

/**
 * 是否需要显示 secretPhrase 输入框。
 *
 * 当 accountStore 已有 secretPhrase（password 登录模式）时隐藏输入框，
 * 直接使用内存中的值；否则显示输入框让用户手动输入。
 */
const needsSecretPhrase = computed(() => !accountStore.hasSecretPhrase)

const rules = computed<FormRules>(() => ({
  recipient: [{ required: true, message: t('sendMessage.recipientRequired'), trigger: 'blur' }],
  message: [{ required: true, message: t('sendMessage.messageRequired'), trigger: 'blur' }],
  feeNQT: [{ required: true, message: t('sendMessage.feeRequired'), trigger: 'blur' }],
  deadline: [{ required: true, message: t('sendMessage.deadlineRequired'), trigger: 'blur' }],
  secretPhrase: needsSecretPhrase.value
    ? [{ required: true, message: t('sendMessage.secretPhraseRequired'), trigger: 'blur' }]
    : [],
}))

/**
 * 提交发送消息（通过 useNrcsForm 三步本地签名流程）。
 *
 * 安全模型：secretPhrase 不随请求外发，由 useNrcsForm 在本地签名。
 */
const handleSubmit = async () => {
  if (!formRef.value) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    isSubmitting.value = true
    try {
      // 获取 secretPhrase：优先 accountStore 内存值，否则用用户输入
      const secretPhrase = accountStore.hasSecretPhrase
        ? accountStore.secretPhrase
        : form.secretPhrase

      if (!secretPhrase) {
        ElMessage.warning(t('sendMessage.secretPhraseRequired'))
        return
      }

      // 构造表单数据（feeNQT 由 useNrcsForm 自动转换 NXT→NQT）
      const data: Record<string, any> = {
        secretPhrase,
        recipient: form.recipient,
        feeNXT: form.feeNQT,
        deadline: form.deadline,
      }

      if (form.encryptMessage) {
        // 加密消息：使用 messageToEncrypt（服务端会加密后返回 unsignedTransactionBytes）
        data.messageToEncrypt = form.message
        data.messageToEncryptIsText = form.messageIsText
      } else {
        // 普通文本消息
        data.message = form.message
        data.messageIsText = form.messageIsText
      }

      // 通过 useNrcsForm 三步本地签名流程提交（secretPhrase 不外发）
      await submitForm('sendMessage', data, {
        successMessage: t('sendMessage.success'),
      })

      emit('success')
      handleClose()
    } catch (err: any) {
      // useNrcsForm 已通过 ElMessage 显示错误，此处仅记录日志
      console.error('Send message failed:', err)
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
