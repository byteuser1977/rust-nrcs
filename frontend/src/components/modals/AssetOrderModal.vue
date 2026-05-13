<template>
  <el-dialog
    v-model="visible"
    :title="orderType === 'buy' ? t('asset.buyAsset') : t('asset.sellAsset')"
    width="480px"
    :close-on-click-modal="false"
    destroy-on-close
    class="asset-order-modal"
    @close="handleClose"
  >
    <div class="order-summary">
      <div class="summary-row">
        <span class="label">{{ t('asset.assetName') }}</span>
        <span class="value">{{ assetName }}</span>
      </div>
      <div class="summary-row">
        <span class="label">{{ t('asset.orderType') }}</span>
        <span class="value" :class="orderType === 'buy' ? 'text-success' : 'text-danger'">
          {{ orderType === 'buy' ? t('asset.buy') : t('asset.sell') }}
        </span>
      </div>
      <div class="summary-row">
        <span class="label">{{ t('asset.quantity') }}</span>
        <span class="value text-mono">{{ quantity }}</span>
      </div>
      <div class="summary-row">
        <span class="label">{{ t('asset.pricePerShare') }}</span>
        <span class="value text-mono">{{ pricePerShare }} NRC</span>
      </div>
      <el-divider />
      <div class="summary-row total-row">
        <span class="label">{{ t('asset.total') }}</span>
        <span class="value text-mono total-value">{{ totalAmount }} NRC</span>
      </div>
    </div>

    <el-form ref="formRef" :model="form" :rules="rules" label-position="top" class="order-form">
      <el-form-item :label="t('asset.fee')" prop="feeNQT">
        <el-input v-model="form.feeNQT" placeholder="1" type="number" clearable>
          <template #append>NRC</template>
        </el-input>
      </el-form-item>

      <el-form-item :label="t('asset.deadline')" prop="deadline">
        <el-input v-model="form.deadline" placeholder="1440" type="number" clearable>
          <template #append>{{ t('asset.minutes') }}</template>
        </el-input>
      </el-form-item>

      <el-form-item :label="t('asset.secretPhrase')" prop="secretPhrase">
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
        <el-button
          :type="orderType === 'buy' ? 'success' : 'danger'"
          :loading="isSubmitting"
          @click="handleSubmit"
        >
          {{ orderType === 'buy' ? t('asset.confirmBuy') : t('asset.confirmSell') }}
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
  assetId: string
  assetName: string
  quantity: string
  pricePerShare: string
  decimals: number
}>()

const visible = defineModel<boolean>('visible', { default: false })
const emit = defineEmits<{ (e: 'success'): void }>()

const formRef = ref<FormInstance>()
const isSubmitting = ref(false)

const form = reactive({
  feeNQT: '1',
  deadline: '1440',
  secretPhrase: ''
})

const totalAmount = computed(() => {
  const qty = Number(props.quantity) || 0
  const price = Number(props.pricePerShare) || 0
  return (qty * price).toFixed(8)
})

const rules = computed<FormRules>(() => ({
  feeNQT: [{ required: true, message: t('asset.feeRequired'), trigger: 'blur' }],
  deadline: [{ required: true, message: t('asset.deadlineRequired'), trigger: 'blur' }],
  secretPhrase: [{ required: true, message: t('asset.secretPhraseRequired'), trigger: 'blur' }]
}))

const handleSubmit = async () => {
  if (!formRef.value) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    isSubmitting.value = true
    try {
      const quantityQNT = String(Math.round(Number(props.quantity) * Math.pow(10, props.decimals)))
      const priceNQTPerShare = String(Math.round(Number(props.pricePerShare) * Math.pow(10, 8 - props.decimals)))
      const feeNQT = String(Math.round(Number(form.feeNQT) * 1e8))

      const apiFn = props.orderType === 'buy' ? nrcsApi.placeBidOrder : nrcsApi.placeAskOrder
      await apiFn({
        secretPhrase: form.secretPhrase,
        asset: props.assetId,
        quantityQNT,
        priceNQTPerShare,
        feeNQT,
        deadline: Number(form.deadline)
      })

      ElMessage.success(props.orderType === 'buy' ? t('asset.buySuccess') : t('asset.sellSuccess'))
      emit('success')
      handleClose()
    } catch (err: any) {
      ElMessage.error(err?.message || t('asset.orderError'))
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

    .label {
      color: $text-muted;
      font-size: $font-size-sm;
    }

    .value {
      color: $text-primary;
      font-weight: 500;
    }
  }

  .total-row {
    .total-value {
      font-size: $font-size-lg;
      color: $primary;
      font-weight: 700;
    }
  }
}

.text-success { color: $success; }
.text-danger { color: $danger; }
.text-mono { font-family: $font-mono; }

.order-form {
  margin-top: $space-md;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: $space-sm;
}
</style>
