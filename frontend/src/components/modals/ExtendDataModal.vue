<template>
  <el-dialog v-model="visible" :title="t('modal.extendTaggedData')" width="500px" :close-on-click-modal="false" destroy-on-close class="nrcs-modal" @close="handleClose">
    <div v-if="props.transaction" class="order-summary mb-4">
      <div class="summary-row">
        <span class="label">{{ t('common.transaction') }}</span>
        <span class="value text-mono">{{ transactionId }}</span>
      </div>
      <div v-if="dataName" class="summary-row">
        <span class="label">{{ t('common.name') }}</span>
        <span class="value">{{ dataName }}</span>
      </div>
    </div>
    <el-form ref="formRef" :model="form" :rules="rules" label-position="top">
      <el-form-item :label="t('common.transaction')">
        <el-input :model-value="transactionId" disabled />
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
      <el-button type="primary" :loading="loading" @click="handleSubmit">{{ t('modal.extendTaggedData') }}</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, type FormInstance, type FormRules } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { useAccountStore } from '@/stores/modules/account.store'

const { t } = useI18n()
const accountStore = useAccountStore()
const props = defineProps<{ transaction: { transaction: string; name?: string } }>()
const visible = defineModel<boolean>('visible', { default: false })
const emit = defineEmits<{ (e: 'success'): void }>()
const formRef = ref<FormInstance>()
const loading = ref(false)

const transactionId = computed(() => props.transaction?.transaction || '')
const dataName = computed(() => props.transaction?.name || '')

const form = reactive({
  feeNQT: '1',
  deadline: '1440',
  secretPhrase: ''
})

watch(visible, (val) => { if (val) { form.secretPhrase = accountStore.secretPhrase || '' } })

const rules = computed<FormRules>(() => ({
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
      await nrcsApi.extendTaggedData({
        secretPhrase: form.secretPhrase,
        transaction: transactionId.value,
        data: '',
        feeNQT: String(Math.round(Number(form.feeNQT) * 1e8)),
        deadline: Number(form.deadline)
      })
      ElMessage.success(t('success.extendTaggedData'))
      emit('success')
      handleClose()
    } catch (err: any) {
      ElMessage.error(err?.message || t('error.unknownError'))
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
.order-summary { background: rgba(255,255,255,0.02); border: 1px solid $border-subtle; border-radius: $radius-md; padding: $space-lg; }
.summary-row { display: flex; justify-content: space-between; padding: $space-xs 0; .label { color: $text-muted; font-size: $font-size-sm; } .value { color: $text-primary; font-weight: 500; } }
.text-mono { font-family: monospace; font-size: $font-size-xs; word-break: break-all; }
.mb-4 { margin-bottom: $space-lg; }
</style>
