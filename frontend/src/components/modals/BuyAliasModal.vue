<template>
  <el-dialog v-model="visible" :title="t('alias.buyAlias')" width="500px" :close-on-click-modal="false" destroy-on-close class="nrcs-modal" @close="handleClose">
    <el-form ref="formRef" :model="form" label-position="top">
      <el-form-item :label="t('alias.aliasName')">
        <el-input v-model="aliasName" disabled />
      </el-form-item>
      <el-form-item v-if="aliasPrice" :label="t('alias.price')">
        <el-input :model-value="aliasPrice" disabled><template #append>NRC</template></el-input>
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
      <el-button type="primary" :loading="loading" @click="handleSubmit">{{ t('alias.buy') }}</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, reactive, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, type FormInstance } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'

const { t } = useI18n()
const props = defineProps<{ alias?: any }>()
const visible = defineModel<boolean>('visible', { default: false })
const emit = defineEmits<{ (e: 'success'): void }>()
const formRef = ref<FormInstance>()
const loading = ref(false)
const aliasName = computed(() => props.alias?.aliasName || '')
const aliasPrice = computed(() => props.alias?.priceNQT ? (Number(props.alias.priceNQT) / 1e8).toFixed(2) : '')
const form = reactive({ feeNQT: '1', deadline: '1440', secretPhrase: '' })

async function handleSubmit() {
  if (!formRef.value) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    loading.value = true
    try {
      await nrcsApi.buyAlias({ secretPhrase: form.secretPhrase, aliasName: aliasName.value, feeNQT: String(Math.round(Number(form.feeNQT) * 1e8)), deadline: Number(form.deadline) })
      ElMessage.success(t('alias.buySuccess'))
      emit('success'); handleClose()
    } catch (err: any) { ElMessage.error(err?.message || t('alias.buyError')) }
    finally { loading.value = false }
  })
}
function handleClose() { formRef.value?.resetFields(); visible.value = false }
</script>
<style scoped lang="scss">@use '@/assets/styles/variables' as *;</style>
