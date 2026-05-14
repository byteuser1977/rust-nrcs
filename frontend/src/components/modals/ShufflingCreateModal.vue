<template>
  <el-dialog v-model="visible" :title="t('shuffling.createShuffling')" width="560px" :close-on-click-modal="false" destroy-on-close class="nrcs-modal" @close="handleClose">
    <el-alert type="warning" :closable="false" show-icon class="mb-4">
      {{ t('shuffling.nodeWarning') }}
    </el-alert>
    <el-form ref="formRef" :model="form" :rules="rules" label-position="top">
      <el-row :gutter="16">
        <el-col :span="12">
          <el-form-item :label="t('shuffling.holdingType')">
            <el-select v-model="form.holdingType" style="width:100%">
              <el-option :value="0" label="NRC" />
              <el-option :value="1" :label="t('asset.title')" />
              <el-option :value="2" :label="t('monetary.title')" />
            </el-select>
          </el-form-item>
        </el-col>
        <el-col :span="12">
          <el-form-item v-if="form.holdingType > 0" :label="t('shuffling.holding')">
            <el-input v-model="form.holding" :placeholder="t('shuffling.holdingPlaceholder')" clearable />
          </el-form-item>
        </el-col>
      </el-row>
      <el-row :gutter="16">
        <el-col :span="12">
          <el-form-item :label="t('shuffling.amount')" prop="amount">
            <el-input v-model="form.amount" type="number" placeholder="1000" clearable />
          </el-form-item>
        </el-col>
        <el-col :span="12">
          <el-form-item :label="t('shuffling.participantCount')" prop="participantCount">
            <el-input-number v-model="form.participantCount" :min="2" :max="100" style="width:100%" />
          </el-form-item>
        </el-col>
      </el-row>
      <el-form-item :label="t('shuffling.registrationPeriod')">
        <el-input v-model="form.registrationPeriod" type="number" placeholder="1440" clearable />
        <span class="hint-text">{{ t('shuffling.registrationPeriodHint') }}</span>
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
      <el-button type="primary" :loading="loading" @click="handleSubmit">{{ t('shuffling.createShuffling') }}</el-button>
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
const form = reactive({ holdingType: 0, holding: '', amount: '', participantCount: 3, registrationPeriod: '1440', feeNQT: '1', deadline: '1440', secretPhrase: '' })

const rules = computed<FormRules>(() => ({
  amount: [{ required: true, message: t('shuffling.amountRequired'), trigger: 'blur' }],
  participantCount: [{ required: true, message: t('shuffling.participantRequired'), trigger: 'blur' }],
  feeNQT: [{ required: true, message: t('shuffling.feeRequired'), trigger: 'blur' }],
  deadline: [{ required: true, message: t('shuffling.deadlineRequired'), trigger: 'blur' }],
  secretPhrase: [{ required: true, message: t('common.secretPhraseRequired'), trigger: 'blur' }]
}))

async function handleSubmit() {
  if (!formRef.value) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    loading.value = true
    try {
      await nrcsApi.shufflingCreate({ secretPhrase: form.secretPhrase, amount: form.amount, participantCount: form.participantCount, registrationPeriod: Number(form.registrationPeriod), holdingType: form.holdingType, holding: form.holding, feeNQT: String(Math.round(Number(form.feeNQT) * 1e8)), deadline: Number(form.deadline) })
      ElMessage.success(t('shuffling.createSuccess'))
      emit('success'); handleClose()
    } catch (err: any) { ElMessage.error(err?.message || t('shuffling.createError')) }
    finally { loading.value = false }
  })
}
function handleClose() { formRef.value?.resetFields(); visible.value = false }
</script>
<style scoped lang="scss">
@use '@/assets/styles/variables' as *;
.hint-text { font-size: $font-size-xs; color: $text-muted; margin-top: 4px; display: block; }
.mb-4 { margin-bottom: $space-lg; }
</style>
