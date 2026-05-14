<template>
  <el-dialog v-model="visible" :title="t('marketplace.changeQuantity')" width="480px" :close-on-click-modal="false" destroy-on-close class="nrcs-modal" @close="handleClose">
    <div v-if="goods" class="mb-4"><span class="text-muted">{{ goods.name }}</span></div>
    <el-form ref="formRef" :model="form" label-position="top">
      <el-form-item :label="t('marketplace.quantityDelta')" prop="deltaQuantity">
        <el-input v-model="form.deltaQuantity" type="number" placeholder="Use negative to decrease" clearable />
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
      <el-button type="primary" :loading="loading" @click="handleSubmit">{{ t('marketplace.changeQuantity') }}</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, type FormInstance } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'

const { t } = useI18n()
const props = defineProps<{ goods?: any }>()
const visible = defineModel<boolean>('visible', { default: false })
const emit = defineEmits<{ (e: 'success'): void }>()
const formRef = ref<FormInstance>()
const loading = ref(false)
const form = reactive({ deltaQuantity: '', feeNQT: '1', deadline: '1440', secretPhrase: '' })

async function handleSubmit() {
  if (!formRef.value) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    loading.value = true
    try {
      await nrcsApi.dgsQuantityChange({ secretPhrase: form.secretPhrase, goods: props.goods?.goods, deltaQuantity: Number(form.deltaQuantity), feeNQT: String(Math.round(Number(form.feeNQT) * 1e8)), deadline: Number(form.deadline) })
      ElMessage.success(t('marketplace.quantityChangeSuccess'))
      emit('success'); handleClose()
    } catch (err: any) { ElMessage.error(err?.message || t('marketplace.quantityChangeError')) }
    finally { loading.value = false }
  })
}
function handleClose() { formRef.value?.resetFields(); visible.value = false }
</script>
<style scoped lang="scss">
@use '@/assets/styles/variables' as *;
.mb-4 { margin-bottom: $space-md; }
.text-muted { color: $text-muted; }
</style>
