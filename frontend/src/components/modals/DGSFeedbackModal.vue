<template>
  <el-dialog v-model="visible" :title="t('marketplace.feedback')" width="500px" :close-on-click-modal="false" destroy-on-close class="nrcs-modal" @close="handleClose">
    <div v-if="purchase" class="order-summary mb-4">
      <div class="summary-row"><span class="label">{{ t('marketplace.product') }}</span><span class="value">{{ purchase.name }}</span></div>
      <div class="summary-row"><span class="label">{{ t('marketplace.seller') }}</span><span class="value text-mono">{{ purchase.sellerRS }}</span></div>
    </div>
    <el-form ref="formRef" :model="form" :rules="rules" label-position="top">
      <el-form-item :label="t('marketplace.feedbackMessage')" prop="message">
        <el-input v-model="form.message" type="textarea" :rows="4" />
      </el-form-item>
      <el-form-item :label="t('marketplace.feedbackType')">
        <el-select v-model="form.feedbackType" style="width:100%">
          <el-option value="public" :label="t('marketplace.publicFeedback')" />
          <el-option value="private" :label="t('marketplace.privateFeedback')" />
        </el-select>
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
      <el-button type="primary" :loading="loading" @click="handleSubmit">{{ t('marketplace.submitFeedback') }}</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, reactive, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, type FormInstance, type FormRules } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'

const { t } = useI18n()
const props = defineProps<{ purchase: any }>()
const visible = defineModel<boolean>('visible', { default: false })
const emit = defineEmits<{ (e: 'success'): void }>()
const formRef = ref<FormInstance>()
const loading = ref(false)
const form = reactive({ message: '', feedbackType: 'public', feeNQT: '1', deadline: '1440', secretPhrase: '' })

const rules = computed<FormRules>(() => ({
  secretPhrase: [{ required: true, message: t('common.secretPhraseRequired'), trigger: 'blur' }],
  feeNQT: [{ required: true, message: t('alias.feeRequired'), trigger: 'blur' }]
}))

async function handleSubmit() {
  if (!formRef.value) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    loading.value = true
    try {
      await nrcsApi.dgsFeedback({ secretPhrase: form.secretPhrase, purchase: props.purchase.purchase, message: form.message, feedbackType: form.feedbackType, feeNQT: String(Math.round(Number(form.feeNQT) * 1e8)), deadline: Number(form.deadline) } as any)
      ElMessage.success(t('marketplace.feedbackSuccess'))
      emit('success'); handleClose()
    } catch (err: any) { ElMessage.error(err?.message || t('marketplace.feedbackError')) }
    finally { loading.value = false }
  })
}
function handleClose() { formRef.value?.resetFields(); visible.value = false }
</script>
<style scoped lang="scss">
@use '@/assets/styles/variables' as *;
.order-summary { background: rgba(255,255,255,0.02); border: 1px solid $border-subtle; border-radius: $radius-md; padding: $space-lg; }
.summary-row { display: flex; justify-content: space-between; padding: $space-xs 0; .label { color: $text-muted; font-size: $font-size-sm; } .value { color: $text-primary; font-weight: 500; } }
.mb-4 { margin-bottom: $space-lg; }
</style>
