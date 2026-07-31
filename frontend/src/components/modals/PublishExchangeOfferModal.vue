<template>
  <el-dialog v-model="visible" :title="t('modal.publishExchangeOffer')" width="600px" :close-on-click-modal="false" destroy-on-close class="nrcs-modal" @close="handleClose">
    <div v-if="currency" class="order-summary mb-4">
      <div class="summary-row"><span class="label">{{ t('monetary.currency') }}</span><span class="value">{{ currencyDisplay }}</span></div>
    </div>
    <el-form ref="formRef" :model="form" :rules="rules" label-position="top">
      <el-form-item :label="t('monetary.currencyCode')">
        <el-input :model-value="currencyCode" disabled />
      </el-form-item>
      <el-row :gutter="16">
        <el-col :span="12">
          <el-form-item :label="t('monetary.buyRate')" prop="buyRate">
            <el-input v-model="form.buyRate" type="number" placeholder="0.00" clearable><template #append>NRC</template></el-input>
          </el-form-item>
        </el-col>
        <el-col :span="12">
          <el-form-item :label="t('monetary.sellRate')" prop="sellRate">
            <el-input v-model="form.sellRate" type="number" placeholder="0.00" clearable><template #append>NRC</template></el-input>
          </el-form-item>
        </el-col>
      </el-row>
      <el-row :gutter="16">
        <el-col :span="12">
          <el-form-item :label="t('monetary.buyLimit')" prop="buyLimit">
            <el-input v-model="form.buyLimit" type="number" placeholder="0" clearable>
              <template #append>{{ currencyCode }}</template>
            </el-input>
          </el-form-item>
        </el-col>
        <el-col :span="12">
          <el-form-item :label="t('monetary.sellLimit')" prop="sellLimit">
            <el-input v-model="form.sellLimit" type="number" placeholder="0" clearable>
              <template #append>{{ currencyCode }}</template>
            </el-input>
          </el-form-item>
        </el-col>
      </el-row>
      <el-form-item :label="t('voting.finishHeight')" prop="expirationHeight">
        <el-input v-model="form.expirationHeight" type="number" :placeholder="t('voting.finishHeight')" clearable />
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
      <el-button type="primary" :loading="loading" @click="handleSubmit">{{ t('modal.publishExchangeOffer') }}</el-button>
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
const props = defineProps<{ currency: { currency: string; code: string } }>()
const visible = defineModel<boolean>('visible', { default: false })
const emit = defineEmits<{ (e: 'success'): void }>()
const formRef = ref<FormInstance>()
const loading = ref(false)

const currencyCode = computed(() => props.currency?.code || '')
const currencyDisplay = computed(() => {
  const c = props.currency
  if (!c) return ''
  return c.code ? `${c.code}` : c.currency
})

const form = reactive({
  buyRate: '',
  sellRate: '',
  buyLimit: '',
  sellLimit: '',
  expirationHeight: '',
  feeNQT: '1',
  deadline: '1440',
  secretPhrase: ''
})

watch(visible, (val) => { if (val) { form.secretPhrase = accountStore.secretPhrase || '' } })

const rules = computed<FormRules>(() => ({
  buyRate: [{ required: true, message: t('error.fieldNotANumber', { field: t('monetary.buyRate') }), trigger: 'blur' }],
  sellRate: [{ required: true, message: t('error.fieldNotANumber', { field: t('monetary.sellRate') }), trigger: 'blur' }],
  buyLimit: [{ required: true, message: t('error.fieldNotANumber', { field: t('monetary.buyLimit') }), trigger: 'blur' }],
  sellLimit: [{ required: true, message: t('error.fieldNotANumber', { field: t('monetary.sellLimit') }), trigger: 'blur' }],
  expirationHeight: [{ required: true, message: t('error.heightInvalid'), trigger: 'blur' }],
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
      // Convert buyRate/sellRate from NRC display to NQT
      const buyRateNQT = String(Math.round(Number(form.buyRate) * 1e8))
      const sellRateNQT = String(Math.round(Number(form.sellRate) * 1e8))

      await (nrcsApi as any).publishExchangeOffer({
        secretPhrase: form.secretPhrase,
        currency: props.currency.currency,
        buyRateNQT,
        sellRateNQT,
        totalBuyLimit: Number(form.buyLimit),
        totalSellLimit: Number(form.sellLimit),
        expirationHeight: Number(form.expirationHeight),
        feeNQT: String(Math.round(Number(form.feeNQT) * 1e8)),
        deadline: Number(form.deadline)
      })
      ElMessage.success(t('success.publishExchangeOffer'))
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
.mb-4 { margin-bottom: $space-lg; }
</style>
