<template>
  <el-dialog
    v-model="visible"
    :title="t('marketplace.purchaseProduct')"
    width="480px"
    :close-on-click-modal="false"
    destroy-on-close
    class="purchase-product-modal"
    @close="handleClose"
  >
    <div class="product-summary">
      <div class="summary-row">
        <span class="label">{{ t('marketplace.productName') }}</span>
        <span class="value">{{ product?.name }}</span>
      </div>
      <div class="summary-row">
        <span class="label">{{ t('marketplace.seller') }}</span>
        <span class="value text-mono">{{ product?.sellerRS }}</span>
      </div>
      <div class="summary-row">
        <span class="label">{{ t('marketplace.price') }}</span>
        <span class="value price-value">{{ formatPrice(product?.priceNQT) }} NRC</span>
      </div>
    </div>

    <el-form ref="formRef" :model="form" :rules="rules" label-position="top">
      <el-form-item :label="t('marketplace.quantity')" prop="quantity">
        <el-input-number v-model="form.quantity" :min="1" :max="product?.quantity || 1" controls-position="right" />
      </el-form-item>

      <el-form-item :label="t('marketplace.deliveryDeadline')" prop="deliveryDeadlineHours">
        <el-input v-model="form.deliveryDeadlineHours" :placeholder="t('marketplace.deliveryDeadlinePlaceholder')" type="number" clearable>
          <template #append>{{ t('marketplace.hours') }}</template>
        </el-input>
      </el-form-item>

      <el-form-item :label="t('marketplace.fee')" prop="feeNQT">
        <el-input v-model="form.feeNQT" placeholder="1" type="number" clearable>
          <template #append>NRC</template>
        </el-input>
      </el-form-item>

      <el-form-item :label="t('marketplace.secretPhrase')" prop="secretPhrase">
        <el-input v-model="form.secretPhrase" type="password" show-password clearable>
          <template #prefix>
            <el-icon><Lock /></el-icon>
          </template>
        </el-input>
      </el-form-item>
    </el-form>

    <template #footer>
      <div class="dialog-footer">
        <el-button @click="handleClose">{{ t('common.cancel') }}</el-button>
        <el-button type="primary" :loading="isSubmitting" @click="handleSubmit">
          {{ t('marketplace.confirmPurchase') }}
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
import { nrcsApi, type NrcsDGSProduct } from '@/api/modules/nrcs.api'

const { t } = useI18n()

const props = defineProps<{
  product: NrcsDGSProduct | null
}>()

const visible = defineModel<boolean>('visible', { default: false })
const emit = defineEmits<{ (e: 'success'): void }>()

const formRef = ref<FormInstance>()
const isSubmitting = ref(false)

const form = reactive({
  quantity: 1,
  deliveryDeadlineHours: '168',
  feeNQT: '1',
  secretPhrase: ''
})

const formatPrice = (priceNQT?: string) => {
  if (!priceNQT) return '0.00'
  return (Number(priceNQT) / 1e8).toFixed(2)
}

const rules = computed<FormRules>(() => ({
  quantity: [{ required: true, message: t('marketplace.quantityRequired'), trigger: 'change' }],
  deliveryDeadlineHours: [{ required: true, message: t('marketplace.deadlineRequired'), trigger: 'blur' }],
  feeNQT: [{ required: true, message: t('marketplace.feeRequired'), trigger: 'blur' }],
  secretPhrase: [{ required: true, message: t('marketplace.secretPhraseRequired'), trigger: 'blur' }]
}))

const handleSubmit = async () => {
  if (!formRef.value || !props.product) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    isSubmitting.value = true
    try {
      const feeNQT = String(Math.round(Number(form.feeNQT) * 1e8))
      await nrcsApi.dgsPurchase({
        secretPhrase: form.secretPhrase,
        goods: props.product!.goods,
        quantity: form.quantity,
        priceNQT: props.product!.priceNQT,
        feeNQT,
        deadline: Number(form.deliveryDeadlineHours) * 60
      })
      ElMessage.success(t('marketplace.purchaseSuccess'))
      emit('success')
      handleClose()
    } catch (err: any) {
      ElMessage.error(err?.message || t('marketplace.purchaseError'))
    } finally {
      isSubmitting.value = false
    }
  })
}

const handleClose = () => {
  formRef.value?.resetFields()
  Object.assign(form, { quantity: 1, deliveryDeadlineHours: '168', feeNQT: '1', secretPhrase: '' })
  visible.value = false
}
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;

.product-summary {
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

    .label {
      color: $text-muted;
      font-size: $font-size-sm;
    }

    .value {
      color: $text-primary;
      font-weight: 500;
    }

    .price-value {
      color: $primary;
      font-weight: 700;
    }
  }
}

.text-mono { font-family: $font-mono; }

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: $space-sm;
}
</style>
