<template>
  <el-dialog v-model="visible" :title="t('monetary.transferCurrency')" width="520px" :close-on-click-modal="false" destroy-on-close class="nrcs-modal" @close="handleClose">
    <el-form ref="formRef" :model="form" :rules="rules" label-position="top">
      <el-form-item :label="t('monetary.currency')" prop="currency">
        <el-input v-model="form.currency" :placeholder="t('monetary.currencyIdPlaceholder')" clearable />
      </el-form-item>
      <el-form-item :label="t('common.recipient')" prop="recipient">
        <el-input v-model="form.recipient" placeholder="NRCS-XXXX-XXXX-XXXX-XXXXX" clearable />
      </el-form-item>
      <el-form-item :label="t('monetary.units')" prop="units">
        <el-input v-model="form.units" type="number" placeholder="1" clearable />
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
      <el-button type="primary" :loading="loading" @click="handleSubmit">{{ t('monetary.transferCurrency') }}</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, reactive, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, type FormInstance, type FormRules } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'

const { t } = useI18n()
const visible = defineModel<boolean>('visible', { default: false })
const emit = defineEmits<{ (e: 'success'): void }>()
const formRef = ref<FormInstance>()
const loading = ref(false)
const form = reactive({ currency: '', recipient: '', units: '', feeNQT: '1', deadline: '1440', secretPhrase: '' })

const rules = computed<FormRules>(() => ({
  currency: [{ required: true, message: t('monetary.currencyRequired'), trigger: 'blur' }],
  recipient: [{ required: true, message: t('monetary.recipientRequired'), trigger: 'blur' }],
  units: [{ required: true, message: t('monetary.unitsRequired'), trigger: 'blur' }],
  feeNQT: [{ required: true, message: t('monetary.feeRequired'), trigger: 'blur' }],
  deadline: [{ required: true, message: t('monetary.deadlineRequired'), trigger: 'blur' }],
  secretPhrase: [{ required: true, message: t('common.secretPhraseRequired'), trigger: 'blur' }]
}))

async function handleSubmit() {
  if (!formRef.value) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    loading.value = true
    try {
      await nrcsApi.transferCurrency({ secretPhrase: form.secretPhrase, currency: form.currency, recipient: form.recipient, unitsQNT: form.units, feeNQT: String(Math.round(Number(form.feeNQT) * 1e8)), deadline: Number(form.deadline) })
      ElMessage.success(t('monetary.transferSuccess'))
      emit('success'); handleClose()
    } catch (err: any) { ElMessage.error(err?.message || t('monetary.transferError')) }
    finally { loading.value = false }
  })
}
function handleClose() { formRef.value?.resetFields(); visible.value = false }
</script>
<style scoped lang="scss">@use '@/assets/styles/variables' as *;</style>
