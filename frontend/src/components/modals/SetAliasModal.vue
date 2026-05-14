<template>
  <el-dialog v-model="visible" :title="isEdit ? t('alias.editAlias') : t('alias.registerAlias')" width="520px" :close-on-click-modal="false" destroy-on-close class="nrcs-modal" @close="handleClose">
    <el-form ref="formRef" :model="form" :rules="rules" label-position="top">
      <el-form-item :label="t('alias.type')" prop="aliasType">
        <el-select v-model="form.aliasType" style="width:100%">
          <el-option value="uri" label="URI" />
          <el-option value="account" :label="t('common.account')" />
          <el-option value="general" :label="t('common.other')" />
        </el-select>
      </el-form-item>
      <el-form-item :label="t('alias.aliasName')" prop="aliasName">
        <el-input v-model="form.aliasName" :placeholder="t('alias.aliasNamePlaceholder')" clearable />
      </el-form-item>
      <el-form-item :label="t('alias.uri')" prop="aliasURI">
        <el-input v-model="form.aliasURI" :placeholder="t('alias.uriPlaceholder')" clearable />
      </el-form-item>
      <el-row :gutter="16">
        <el-col :span="12"><el-form-item :label="t('common.fee')" prop="feeNQT"><el-input v-model="form.feeNQT" placeholder="2" type="number" clearable><template #append>NRC</template></el-input></el-form-item></el-col>
        <el-col :span="12"><el-form-item :label="t('common.deadline')" prop="deadline"><el-input v-model="form.deadline" placeholder="1440" type="number" clearable><template #append>{{ t('common.minutes') }}</template></el-input></el-form-item></el-col>
      </el-row>
      <el-form-item :label="t('common.secretPhrase')" prop="secretPhrase">
        <el-input v-model="form.secretPhrase" type="password" show-password clearable />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="handleClose">{{ t('common.cancel') }}</el-button>
      <el-button type="primary" :loading="loading" @click="handleSubmit">{{ isEdit ? t('alias.editAlias') : t('alias.registerAlias') }}</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, reactive, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, type FormInstance, type FormRules } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'

const { t } = useI18n()
const props = defineProps<{ editAlias?: any }>()
const visible = defineModel<boolean>('visible', { default: false })
const emit = defineEmits<{ (e: 'success'): void }>()
const formRef = ref<FormInstance>()
const loading = ref(false)
const isEdit = computed(() => !!props.editAlias)
const form = reactive({ aliasType: 'uri', aliasName: '', aliasURI: '', feeNQT: '2', deadline: '1440', secretPhrase: '' })

const rules = computed<FormRules>(() => ({
  aliasName: [{ required: true, message: t('alias.nameRequired'), trigger: 'blur' }],
  aliasURI: [{ required: true, message: t('alias.uriRequired'), trigger: 'blur' }],
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
      await nrcsApi.setAlias({ aliasName: form.aliasName, aliasURI: form.aliasURI, feeNQT: String(Math.round(Number(form.feeNQT) * 1e8)), deadline: Number(form.deadline), secretPhrase: form.secretPhrase })
      ElMessage.success(t('alias.registerSuccess'))
      emit('success'); handleClose()
    } catch (err: any) { ElMessage.error(err?.message || t('alias.registerError')) }
    finally { loading.value = false }
  })
}
function handleClose() { formRef.value?.resetFields(); visible.value = false }
</script>
<style scoped lang="scss">@use '@/assets/styles/variables' as *;</style>
