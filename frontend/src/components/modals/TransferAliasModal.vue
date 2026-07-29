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
      <el-form-item :label="t('common.secretPhrase')" prop="secretPhrase">
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
import { ref, reactive, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, type FormInstance, type FormRules } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { useAccountStore } from '@/stores/modules/account.store'

const { t } = useI18n()
const accountStore = useAccountStore()
const props = defineProps<{ alias: { aliasName: string } }>()
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
  secretPhrase: accountStore.secretPhrase || ''
})

const rules = computed<FormRules>(() => ({
  recipient: [{ required: true, message: t('error.invalidRecipient'), trigger: 'blur' }],
  feeNQT: [{ required: true, message: t('alias.feeRequired'), trigger: 'blur' }],
  deadline: [{ required: true, message: t('alias.deadlineRequired'), trigger: 'blur' }],
  secretPhrase: [{ required: true, message: t('common.secretPhraseRequired'), trigger: 'blur' }]
}))

async function handleSubmit() {
  if (!formRef.value) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    loading.value = true
    try {
      const params: any = {
        secretPhrase: form.secretPhrase,
        aliasName: aliasName.value,
        priceNQT: '1',
        buyer: form.recipient,
        feeNQT: String(Math.round(Number(form.feeNQT) * 1e8)),
        deadline: Number(form.deadline)
      }
      if (form.message) {
        params.message = form.message
      }
      await nrcsApi.sellAlias(params)
      ElMessage.success(t('success.transferAlias'))
      emit('success')
      handleClose()
    } catch (err: any) {
      ElMessage.error(err?.message || t('error.transferAliasError'))
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
<style scoped lang="scss">@use '@/assets/styles/variables' as *;</style>
