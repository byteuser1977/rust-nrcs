<template>
  <el-dialog v-model="visible" :title="t('datacloud.uploadData')" width="560px" :close-on-click-modal="false" destroy-on-close class="nrcs-modal" @close="handleClose">
    <el-form ref="formRef" :model="form" :rules="rules" label-position="top">
      <el-form-item :label="t('datacloud.name')" prop="name">
        <el-input v-model="form.name" :placeholder="t('datacloud.namePlaceholder')" maxlength="100" clearable />
      </el-form-item>
      <el-form-item :label="t('datacloud.description')">
        <el-input v-model="form.description" type="textarea" :rows="3" :placeholder="t('datacloud.descriptionPlaceholder')" />
      </el-form-item>
      <el-form-item :label="t('datacloud.tags')">
        <el-input v-model="form.tags" :placeholder="t('datacloud.tagsPlaceholder')" maxlength="100" />
      </el-form-item>
      <el-form-item :label="t('datacloud.channel')">
        <el-input v-model="form.channel" :placeholder="t('datacloud.channelPlaceholder')" clearable />
      </el-form-item>
      <el-form-item :label="t('datacloud.data')" prop="data">
        <el-input v-model="form.data" type="textarea" :rows="4" :placeholder="t('datacloud.dataPlaceholder')" />
        <el-checkbox v-model="form.isText" class="mt-2">{{ t('datacloud.dataIsText') }}</el-checkbox>
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
      <el-button type="primary" :loading="loading" @click="handleSubmit">{{ t('datacloud.uploadData') }}</el-button>
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
const form = reactive({ name: '', description: '', tags: '', channel: '', data: '', isText: true, feeNQT: '1', deadline: '1440', secretPhrase: '' })

const rules = computed<FormRules>(() => ({
  name: [{ required: true, message: t('datacloud.nameRequired'), trigger: 'blur' }],
  data: [{ required: true, message: t('datacloud.dataRequired'), trigger: 'blur' }],
  feeNQT: [{ required: true, message: t('datacloud.feeRequired'), trigger: 'blur' }],
  deadline: [{ required: true, message: t('datacloud.deadlineRequired'), trigger: 'blur' }],
  secretPhrase: [{ required: true, message: t('common.secretPhraseRequired'), trigger: 'blur' }]
}))

async function handleSubmit() {
  if (!formRef.value) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    loading.value = true
    try {
      const params: any = { secretPhrase: form.secretPhrase, name: form.name, description: form.description, tags: form.tags, channel: form.channel, data: form.data, isText: form.isText, feeNQT: String(Math.round(Number(form.feeNQT) * 1e8)), deadline: Number(form.deadline) }
      await nrcsApi.uploadTaggedData(params)
      ElMessage.success(t('datacloud.uploadSuccess'))
      emit('success'); handleClose()
    } catch (err: any) { ElMessage.error(err?.message || t('datacloud.uploadError')) }
    finally { loading.value = false }
  })
}
function handleClose() { formRef.value?.resetFields(); visible.value = false }
</script>
<style scoped lang="scss">
@use '@/assets/styles/variables' as *;
.mt-2 { margin-top: $space-sm; }
</style>
