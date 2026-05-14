<template>
  <el-dialog v-model="visible" :title="t('voting.createPoll')" width="600px" :close-on-click-modal="false" destroy-on-close class="nrcs-modal" @close="handleClose">
    <el-form ref="formRef" :model="form" :rules="rules" label-position="top">
      <el-form-item :label="t('voting.name')" prop="name"><el-input v-model="form.name" maxlength="100" clearable /></el-form-item>
      <el-form-item :label="t('voting.description')" prop="description"><el-input v-model="form.description" type="textarea" :rows="3" /></el-form-item>
      <el-row :gutter="16">
        <el-col :span="12">
          <el-form-item :label="t('voting.votingModel')" prop="votingModel">
            <el-select v-model="form.votingModel" style="width:100%">
              <el-option v-for="(val, key) in votingModels" :key="key" :label="key" :value="val" />
            </el-select>
          </el-form-item>
        </el-col>
        <el-col :span="12">
          <el-form-item v-if="form.votingModel >= 2" :label="t('voting.holding')"><el-input v-model="form.holding" /></el-form-item>
        </el-col>
      </el-row>
      <el-row :gutter="16">
        <el-col :span="12">
          <el-form-item :label="t('voting.minBalanceModel')">
            <el-select v-model="form.minBalanceModel" style="width:100%">
              <el-option :label="t('voting.none')" :value="0" />
              <el-option label="NRC" :value="1" />
              <el-option :label="t('voting.asset')" :value="2" />
              <el-option :label="t('voting.currency')" :value="3" />
            </el-select>
          </el-form-item>
        </el-col>
        <el-col :span="12">
          <el-form-item v-if="form.minBalanceModel > 0" :label="t('voting.minBalance')"><el-input v-model="form.minBalance" type="number" /></el-form-item>
        </el-col>
      </el-row>
      <el-form-item :label="t('voting.finishHeight')" prop="finishHeight">
        <el-input v-model="form.finishHeight" type="number" :placeholder="t('voting.finishHeightPlaceholder')" clearable>
          <template #append><el-button @click="form.finishHeight = String(currentHeight + 1440)">+1440</el-button></template>
        </el-input>
        <span class="hint-text">{{ t('voting.currentHeight') }}: {{ currentHeight }}</span>
      </el-form-item>
      <el-form-item :label="t('voting.options')" prop="options">
        <div v-for="(opt, idx) in form.options" :key="idx" class="option-row">
          <el-input v-model="form.options[idx]" :placeholder="t('voting.option') + ' ' + (idx + 1)" clearable style="flex:1" />
          <el-button v-if="form.options.length > 2" @click="removeOption(idx)" type="danger" text><el-icon><Delete /></el-icon></el-button>
        </div>
        <el-button @click="addOption" size="small" class="mt-2">{{ t('voting.addOption') }}</el-button>
      </el-form-item>
      <el-row :gutter="16">
        <el-col :span="8"><el-form-item :label="t('voting.minChoices')"><el-input-number v-model="form.minNumberOfOptions" :min="1" style="width:100%" /></el-form-item></el-col>
        <el-col :span="8"><el-form-item :label="t('voting.maxChoices')"><el-input-number v-model="form.maxNumberOfOptions" :min="1" style="width:100%" /></el-form-item></el-col>
        <el-col :span="8"><el-form-item :label="t('voting.minRangeValue')"><el-input-number v-model="form.minRangeValue" :min="0" style="width:100%" /></el-form-item></el-col>
      </el-row>
      <el-row :gutter="16">
        <el-col :span="12"><el-form-item :label="t('common.fee')" prop="feeNQT"><el-input v-model="form.feeNQT" placeholder="10" type="number" clearable><template #append>NRC</template></el-input></el-form-item></el-col>
        <el-col :span="12"><el-form-item :label="t('common.deadline')" prop="deadline"><el-input v-model="form.deadline" placeholder="1440" type="number" clearable><template #append>{{ t('common.minutes') }}</template></el-input></el-form-item></el-col>
      </el-row>
      <el-form-item :label="t('common.secretPhrase')" prop="secretPhrase">
        <el-input v-model="form.secretPhrase" type="password" show-password clearable />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="handleClose">{{ t('common.cancel') }}</el-button>
      <el-button type="primary" :loading="loading" @click="handleSubmit">{{ t('voting.createPoll') }}</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, type FormInstance, type FormRules } from 'element-plus'
import { Delete } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'

const { t } = useI18n()
const visible = defineModel<boolean>('visible', { default: false })
const emit = defineEmits<{ (e: 'success'): void }>()
const formRef = ref<FormInstance>()
const loading = ref(false)
const currentHeight = ref(0)
const votingModels = ref<Record<string,number>>({})

const form = reactive({
  name: '', description: '', votingModel: 0, holding: '',
  minBalanceModel: 0, minBalance: '',
  finishHeight: '', options: ['', ''],
  minNumberOfOptions: 1, maxNumberOfOptions: 1, minRangeValue: 0,
  feeNQT: '10', deadline: '1440', secretPhrase: ''
})

const rules = computed<FormRules>(() => ({
  name: [{ required: true, message: t('voting.nameRequired'), trigger: 'blur' }],
  finishHeight: [{ required: true, message: t('voting.finishHeightRequired'), trigger: 'blur' }],
  feeNQT: [{ required: true, message: t('voting.feeRequired'), trigger: 'blur' }],
  deadline: [{ required: true, message: t('voting.deadlineRequired'), trigger: 'blur' }],
  secretPhrase: [{ required: true, message: t('common.secretPhraseRequired'), trigger: 'blur' }]
}))

onMounted(async () => {
  try { const c = await nrcsApi.getConstants(); votingModels.value = c.votingModels || {} } catch {}
  try { const s = await nrcsApi.getBlockchainStatus(); currentHeight.value = (s as any).numberOfBlocks || 0 } catch {}
})

function addOption() { form.options.push('') }
function removeOption(idx: number) { if (form.options.length > 2) form.options.splice(idx, 1) }

async function handleSubmit() {
  if (!formRef.value) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    loading.value = true
    try {
      const filteredOpts = form.options.filter(o => o.trim())
      await nrcsApi.createPoll({
        secretPhrase: form.secretPhrase, name: form.name, description: form.description,
        finishHeight: Number(form.finishHeight), votingModel: form.votingModel,
        minNumberOfOptions: form.minNumberOfOptions, maxNumberOfOptions: form.maxNumberOfOptions,
        minRangeValue: form.minRangeValue, options: filteredOpts,
        feeNQT: String(Math.round(Number(form.feeNQT) * 1e8)), deadline: Number(form.deadline),
        holding: form.holding, minBalance: form.minBalance, minBalanceModel: form.minBalanceModel
      } as any)
      ElMessage.success(t('voting.createSuccess'))
      emit('success'); handleClose()
    } catch (err: any) { ElMessage.error(err?.message || t('voting.createError')) }
    finally { loading.value = false }
  })
}
function handleClose() { formRef.value?.resetFields(); visible.value = false }
</script>
<style scoped lang="scss">
@use '@/assets/styles/variables' as *;
.option-row { display: flex; gap: $space-sm; margin-bottom: $space-sm; align-items: center; }
.hint-text { font-size: $font-size-xs; color: $text-muted; margin-top: 4px; display: block; }
.mt-2 { margin-top: $space-sm; }
</style>
