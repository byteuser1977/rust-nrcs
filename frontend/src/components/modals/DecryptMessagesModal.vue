<template>
  <el-dialog v-model="visible" :title="t('messages.decryptMessages')" width="480px" :close-on-click-modal="false" destroy-on-close class="nrcs-modal" @close="handleClose">
    <el-alert
      :title="t('messages.passphraseRequiredForDecryption')"
      type="info"
      :closable="false"
      show-icon
      class="mb-4"
    />
    <el-form ref="formRef" :model="form" :rules="rules" label-position="top">
      <el-form-item :label="t('common.secretPhrase')" prop="secretPhrase">
        <el-input v-model="form.secretPhrase" type="password" show-password clearable />
      </el-form-item>
      <el-form-item :label="t('form.sharedKey')">
        <el-input v-model="form.sharedKey" type="password" show-password clearable :placeholder="t('messages.sharedKeyInfo')" />
      </el-form-item>
      <el-form-item>
        <el-checkbox v-model="form.rememberPassword">{{ t('settings.rememberDecryptionPassphrase') }}</el-checkbox>
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="handleClose">{{ t('common.cancel') }}</el-button>
      <el-button type="primary" :loading="loading" @click="handleSubmit">{{ t('messages.decrypt') }}</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, type FormInstance, type FormRules } from 'element-plus'
import { useAccountStore } from '@/stores/modules/account.store'

const { t } = useI18n()
const accountStore = useAccountStore()
const visible = defineModel<boolean>('visible', { default: false })
const emit = defineEmits<{
  (e: 'success'): void
  (e: 'decrypt', payload: { secretPhrase: string; sharedKey: string }): void
}>()
const formRef = ref<FormInstance>()
const loading = ref(false)

const form = reactive({
  secretPhrase: '',
  sharedKey: '',
  rememberPassword: false
})

watch(visible, (val) => { if (val) { form.secretPhrase = accountStore.secretPhrase || '' } })

const rules = computed<FormRules>(() => ({
  secretPhrase: [
    {
      validator: (_rule: any, _value: string, callback: any) => {
        if (!form.secretPhrase && !form.sharedKey) {
          callback(new Error(t('messages.passphraseRequiredForDecryption')))
        } else {
          callback()
        }
      },
      trigger: 'blur'
    }
  ]
}))

async function handleSubmit() {
  if (!formRef.value) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    loading.value = true
    try {
      if (form.rememberPassword) {
        accountStore.secretPhrase = form.secretPhrase
      }
      emit('decrypt', {
        secretPhrase: form.secretPhrase,
        sharedKey: form.sharedKey
      })
      ElMessage.success(t('success.decryptMessages'))
      emit('success')
      handleClose()
    } catch (err: any) {
      ElMessage.error(err?.message || t('error.decryptionFailed'))
    } finally {
      loading.value = false
    }
  })
}

function handleClose() {
  formRef.value?.resetFields()
  visible.value = false
}
</script>
<style scoped lang="scss">
@use '@/assets/styles/variables' as *;
.mb-4 { margin-bottom: $space-lg; }
</style>
