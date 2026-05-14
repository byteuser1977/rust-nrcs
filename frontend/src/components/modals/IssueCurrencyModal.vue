<template>
  <el-dialog v-model="visible" :title="t('monetary.issueCurrency')" width="600px" :close-on-click-modal="false" destroy-on-close class="nrcs-modal" @close="handleClose">
    <el-form ref="formRef" :model="form" :rules="rules" label-position="top">
      <el-row :gutter="16">
        <el-col :span="12">
          <el-form-item :label="t('monetary.name')" prop="name">
            <el-input v-model="form.name" maxlength="10" show-word-limit clearable />
          </el-form-item>
        </el-col>
        <el-col :span="12">
          <el-form-item :label="t('monetary.code')" prop="code">
            <el-input v-model="form.code" maxlength="5" show-word-limit clearable style="text-transform:uppercase" />
          </el-form-item>
        </el-col>
      </el-row>
      <el-form-item :label="t('monetary.description')" prop="description">
        <el-input v-model="form.description" type="textarea" :rows="2" />
      </el-form-item>
      <div class="options-grid">
        <el-checkbox v-model="form.typeExchangeable">{{ t('monetary.exchangeable') }}</el-checkbox>
        <el-checkbox v-model="form.typeControllable">{{ t('monetary.controllable') }}</el-checkbox>
        <el-checkbox v-model="form.typeReservable">{{ t('monetary.reservable') }}</el-checkbox>
        <el-checkbox v-model="form.typeClaimable">{{ t('monetary.claimable') }}</el-checkbox>
        <el-checkbox v-model="form.typeMintable">{{ t('monetary.mintable') }}</el-checkbox>
        <el-checkbox v-model="form.typeNonShuffleable">{{ t('monetary.nonShuffleable') }}</el-checkbox>
      </div>
      <template v-if="form.typeReservable">
        <el-form-item :label="t('monetary.minReserveSupply')"><el-input v-model="form.minReservePerUnitNQT" type="number" placeholder="0" clearable /></el-form-item>
      </template>
      <template v-if="form.typeMintable">
        <el-form-item :label="t('monetary.minDifficulty')"><el-input v-model="form.minDifficulty" type="number" clearable /></el-form-item>
        <el-form-item :label="t('monetary.maxDifficulty')"><el-input v-model="form.maxDifficulty" type="number" clearable /></el-form-item>
        <el-form-item :label="t('monetary.algorithm')">
          <el-select v-model="form.algorithm" style="width:100%">
            <el-option v-for="a in algorithms" :key="a" :label="a" :value="a" />
          </el-select>
        </el-form-item>
      </template>
      <el-row :gutter="16">
        <el-col :span="8"><el-form-item :label="t('monetary.initialSupply')" prop="initialSupply"><el-input v-model="form.initialSupply" type="number" clearable /></el-form-item></el-col>
        <el-col :span="8"><el-form-item :label="t('monetary.totalSupply')"><el-input v-model="form.maxSupply" type="number" clearable /></el-form-item></el-col>
        <el-col :span="8"><el-form-item :label="t('monetary.decimals')" prop="decimals"><el-input-number v-model="form.decimals" :min="0" :max="8" style="width:100%" /></el-form-item></el-col>
      </el-row>
      <el-form-item :label="t('monetary.issuanceHeight')"><el-input v-model="form.issuanceHeight" type="number" clearable /></el-form-item>
      <el-row :gutter="16">
        <el-col :span="12"><el-form-item :label="t('monetary.fee')" prop="feeNQT"><el-input v-model="form.feeNQT" placeholder="1000" type="number" clearable><template #append>NRC</template></el-input></el-form-item></el-col>
        <el-col :span="12"><el-form-item :label="t('monetary.deadline')" prop="deadline"><el-input v-model="form.deadline" placeholder="1440" type="number" clearable><template #append>{{ t('common.minutes') }}</template></el-input></el-form-item></el-col>
      </el-row>
      <el-form-item :label="t('common.secretPhrase')" prop="secretPhrase">
        <el-input v-model="form.secretPhrase" type="password" show-password clearable />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="handleClose">{{ t('common.cancel') }}</el-button>
      <el-button type="primary" :loading="loading" @click="handleSubmit">{{ t('monetary.issueCurrency') }}</el-button>
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
const algorithms = ['SHA256', 'SHA3', 'Scrypt', 'Keccak25']

const form = reactive({
  name: '', code: '', description: '',
  typeExchangeable: true, typeControllable: false, typeReservable: false, typeClaimable: false, typeMintable: false, typeNonShuffleable: false,
  minReservePerUnitNQT: '', minDifficulty: '', maxDifficulty: '', algorithm: 'SHA256',
  initialSupply: '', maxSupply: '', decimals: 0, issuanceHeight: '',
  feeNQT: '1000', deadline: '1440', secretPhrase: ''
})

const rules = computed<FormRules>(() => ({
  name: [{ required: true, message: t('monetary.nameRequired'), trigger: 'blur' }, { min: 3, max: 10, message: t('monetary.nameLength'), trigger: 'blur' }],
  code: [{ required: true, message: t('monetary.codeRequired'), trigger: 'blur' }, { min: 3, max: 5, message: t('monetary.codeLength'), trigger: 'blur' }],
  initialSupply: [{ required: true, message: t('monetary.supplyRequired'), trigger: 'blur' }],
  decimals: [{ required: true, message: t('monetary.decimalsRequired'), trigger: 'blur' }],
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
      const typeVal = (form.typeExchangeable ? 1 : 0) | (form.typeControllable ? 2 : 0) | (form.typeReservable ? 4 : 0) | (form.typeClaimable ? 8 : 0) | (form.typeMintable ? 16 : 0) | (form.typeNonShuffleable ? 32 : 0)
      await nrcsApi.issueCurrency({ secretPhrase: form.secretPhrase, code: form.code.toUpperCase(), name: form.name, description: form.description, type: typeVal, initialSupply: form.initialSupply, maxSupply: form.maxSupply, decimals: form.decimals, issuanceHeight: Number(form.issuanceHeight) || 0, minReservePerUnitNQT: form.minReservePerUnitNQT, minDifficulty: form.minDifficulty, maxDifficulty: form.maxDifficulty, algorithm: form.algorithm, feeNQT: String(Math.round(Number(form.feeNQT) * 1e8)), deadline: Number(form.deadline) } as any)
      ElMessage.success(t('monetary.issueSuccess'))
      emit('success'); handleClose()
    } catch (err: any) { ElMessage.error(err?.message || t('monetary.issueError')) }
    finally { loading.value = false }
  })
}

function handleClose() { formRef.value?.resetFields(); visible.value = false }
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;
.options-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: $space-sm; margin-bottom: $space-md; }
</style>
