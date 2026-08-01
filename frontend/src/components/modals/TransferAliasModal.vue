<template>
  <el-dialog v-model="visible" :title="t('alias.transferAlias')" width="520px" :close-on-click-modal="false" destroy-on-close class="nrcs-modal" @close="handleClose">
    <el-form ref="formRef" :model="form" :rules="rules" label-position="top">
      <el-form-item :label="t('alias.aliasName')">
        <el-input :model-value="aliasName" disabled />
      </el-form-item>
      <el-form-item :label="t('common.recipient')" prop="recipient">
        <el-input v-model="form.recipient" :placeholder="t('common.recipient')" clearable />
      </el-form-item>
      <el-form-item :label="t('messages.message')">
        <el-input v-model="form.message" type="textarea" :rows="2" :placeholder="t('common.optional')" clearable />
      </el-form-item>
      <el-row :gutter="16">
        <el-col :span="12">
          <el-form-item :label="t('common.fee')" prop="feeNQT">
            <el-input v-model="form.feeNQT" placeholder="1" type="number" clearable><template #append>NRC</template></el-input>
          </el-form-item>
        </el-col>
        <el-col :span="12">
          <el-form-item :label="t('common.deadline')" prop="deadline">
            <el-input v-model="form.deadline" placeholder="1440" type="number" clearable><template #append>{{ t('common.minutes') }}</template></el-input>
          </el-form-item>
        </el-col>
      </el-row>
      <el-form-item v-if="needsSecretPhrase" :label="t('common.secretPhrase')" prop="secretPhrase">
        <el-input v-model="form.secretPhrase" type="password" show-password clearable />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="handleClose">{{ t('common.cancel') }}</el-button>
      <el-button type="primary" :loading="loading" @click="handleSubmit">{{ t('alias.transferAlias') }}</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * TransferAliasModal 组件 —— 转移别名弹窗。
 *
 * 对标 nrs.aliases.js 的 sellAlias 表单（transfer_alias 模式，nrs.aliases.js:141-145）。
 *
 * 转移本质是 sellAlias 的特例：priceNXT='0' + 指定 recipient。
 *
 * 安全模型：secretPhrase 不随请求外发，由 useNrcsForm 在本地签名。
 */
import { ref, reactive, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { type FormInstance, type FormRules } from 'element-plus'
import { useAccountStore } from '@/stores/modules/account.store'
import { useNrcsForm } from '@/composables/useNrcsForm'

const { t } = useI18n()
const accountStore = useAccountStore()
const { submitForm } = useNrcsForm()

const props = defineProps<{ alias?: { aliasName?: string } }>()
const visible = defineModel<boolean>('visible', { default: false })
const emit = defineEmits<{ (e: 'success'): void }>()
const formRef = ref<FormInstance>()
const loading = ref(false)
const aliasName = computed(() => props.alias?.aliasName || '')

const form = reactive({
  recipient: '',
  message: '',
  feeNQT: '1',
  deadline: '1440',
  secretPhrase: ''
})

const needsSecretPhrase = computed(() => !accountStore.hasSecretPhrase)

/** 弹窗打开时预填 secretPhrase */
watch(visible, (val) => {
  if (val) {
    form.secretPhrase = accountStore.secretPhrase || ''
  }
})

const rules = computed<FormRules>(() => ({
  recipient: [{ required: true, message: t('validation.invalidAddress'), trigger: 'blur' }],
  feeNQT: [{ required: true, message: t('alias.feeRequired'), trigger: 'blur' }],
  deadline: [{ required: true, message: t('alias.deadlineRequired'), trigger: 'blur' }],
  secretPhrase: needsSecretPhrase.value
    ? [{ required: true, message: t('common.secretPhraseRequired'), trigger: 'blur' }]
    : []
}))

/**
 * 提交转移别名（通过 useNrcsForm 三步本地签名流程）。
 *
 * 转移 = sellAlias with priceNXT='0' + recipient（对标 nrs.aliases.js:141-145）。
 */
async function handleSubmit(): Promise<void> {
  if (!formRef.value) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    loading.value = true
    try {
      const data: Record<string, any> = {
        aliasName: aliasName.value,
        priceNXT: '0',
        recipient: form.recipient,
        feeNXT: form.feeNQT,
        deadline: form.deadline,
        secretPhrase: form.secretPhrase
      }
      if (form.message) {
        data.message = form.message
      }
      await submitForm('sellAlias', data, {
        successMessage: t('alias.transferSuccess')
      })
      emit('success')
      handleClose()
    } catch (err: any) {
      console.error('Transfer alias failed:', err)
    } finally {
      loading.value = false
    }
  })
}

function handleClose(): void {
  formRef.value?.resetFields()
  Object.assign(form, {
    recipient: '',
    message: '',
    feeNQT: '1',
    deadline: '1440',
    secretPhrase: ''
  })
  visible.value = false
}
</script>
<style scoped lang="scss">@use '@/assets/styles/variables' as *;</style>
