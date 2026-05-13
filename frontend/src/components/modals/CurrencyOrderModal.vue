<template>
  <el-dialog
    v-model="visible"
    :title="orderType === 'buy' ? t('currency.buyCurrency') : t('currency.sellCurrency')"
    width="480px"
    :close-on-click-modal="false"
    destroy-on-close
    class="currency-order-modal"
    @close="handleClose"
  >
    <div class="order-summary">
      <div class="summary-row">
        <span class="label">{{ t('currency.currencyName') }}</span>
        <span class="value">{{ currencyCode }}</span>
      </div>
      <div class="summary-row">
        <span class="label">{{ t('currency.orderType') }}</span>
        <span class="value" :class="orderType === 'buy' ? 'text-success' : 'text-danger'">
          {{ orderType === 'buy' ? t('currency.buy') : t('currency.sell') }}
        </span>
      </div>
      <div class="summary-row">
        <span class="label">{{ t('currency.units') }}</span>
        <span class="value text-mono">{{ units }}</span>
      </div>
      <div class="summary-row">
        <span class="label">{{ t('currency.rate') }}</span>
        <span class="value text-mono">{{ rate }} NRC/{{ currencyCode }}</span>
      </div>
      <el-divider />
      <div class="summary-row total-row">
        <span class="label">{{ t('currency.total') }}</span>
        <span class="value text-mono total-value">{{ totalAmount }} NRC</span>
      </div>
    </div>

    <el-form ref="formRef" :model="form" :rules="rules" label-position="top">
      <el-form-item :label="t('currency.fee')" prop="feeNQT">
        <el-input v-model="form.feeNQT" placeholder="1" type="number" clearable>
          <template #append>NRC</template>
        </el-input>
      </el-form-item>
      <el-form-item :label="t('currency.deadline')" prop="deadline">
        <el-input v-model="form.deadline" placeholder="1440" type="number" clearable>
          <template #append>{{ t('currency.minutes') }}</template>
        </el-input>
      </el-form-item>
      <el-form-item :label="t('currency.secretPhrase')" prop="secretPhrase">
        <el-input v-model="form.secretPhrase" type="password" show-password clearable>
          <template #prefix><el-icon><Lock /></el-icon></template>
        </el-input>
      </el-form-item>
    </el-form>

    <template #footer>
      <div class="dialog-footer">
        <el-button @click="handleClose">{{ t('common.cancel') }}</el-button>
        <el-button :type="orderType === 'buy' ? 'success' : 'danger'" :loading="isSubmitting" @click="handleSubmit">
          {{ orderType === 'buy' ? t('currency.confirmBuy') : t('currency.confirmSell') }}
        </el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, reactive, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, type FormInstance, type FormRules } from 'element-plus'
import { Lock } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'

const { t } = useI18n()

const props = defineProps<{
  orderType: 'buy' | 'sell'
  currencyId: string
  currencyCode: string
  units: string
  rate: string
  decimals: number
}>()

const visible = defineModel<boolean>('visible', { default: false })
const emit = defineEmits<{ (e: 'success'): void }>()

const formRef = ref<FormInstance>()
const isSubmitting = ref(false)

const form = reactive({ feeNQT: '1', deadline: '1440', secretPhrase: '' })

const totalAmount = computed(() => {
  const u = Number(props.units) || 0
  const r = Number(props.rate) || 0
  return (u * r).toFixed(8)
})

const rules = computed<FormRules>(() => ({
  feeNQT: [{ required: true, message: t('currency.feeRequired'), trigger: 'blur' }],
  deadline: [{ required: true, message: t('currency.deadlineRequired'), trigger: 'blur' }],
  secretPhrase: [{ required: true, message: t('currency.secretPhraseRequired'), trigger: 'blur' }]
}))

const handleSubmit = async () => {
  if (!formRef.value) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    isSubmitting.value = true
    try {
      const feeNQT = String(Math.round(Number(form.feeNQT) * 1e8))
      const unitsQNT = String(Math.round(Number(props.units) * Math.pow(10, props.decimals)))
      const rateNQT = String(Math.round(Number(props.rate) * Math.pow(10, 8 - props.decimals)))
      await nrcsApi.orderCurrency({
        secretPhrase: form.secretPhrase,
        currency: props.currencyId,
        unitsQNT,
        rateNQT,
        offerType: props.orderType === 'buy' ? 'buy' : 'sell',
        feeNQT,
        deadline: Number(form.deadline)
      })
      ElMessage.success(props.orderType === 'buy' ? t('currency.buySuccess') : t('currency.sellSuccess'))
      emit('success')
      handleClose()
    } catch (err: any) {
      ElMessage.error(err?.message || t('currency.orderError'))
    } finally {
      isSubmitting.value = false
    }
  })
}

const handleClose = () => {
  formRef.value?.resetFields()
  Object.assign(form, { feeNQT: '1', deadline: '1440', secretPhrase: '' })
  visible.value = false
}
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;

.order-summary {
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid $border-subtle;
  border-radius: $radius-md;
  padding: $space-lg;
  margin-bottom: $space-lg;

  .summary-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: $space-xs 0;

    .label { color: $text-muted; font-size: $font-size-sm; }
    .value { color: $text-primary; font-weight: 500; }
  }

  .total-row .total-value {
    font-size: $font-size-lg;
    color: $primary;
    font-weight: 700;
  }
}

.text-success { color: $success; }
.text-danger { color: $danger; }
.text-mono { font-family: $font-mono; }

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: $space-sm;
}
</style>
