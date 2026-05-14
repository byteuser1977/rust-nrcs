<template>
  <el-dialog v-model="visible" :title="t('marketplace.deliverProduct')" width="500px" :close-on-click-modal="false" destroy-on-close class="nrcs-modal" @close="handleClose">
    <div v-if="purchase" class="order-summary">
      <div class="summary-row"><span class="label">{{ t('marketplace.product') }}</span><span class="value">{{ purchase.name }}</span></div>
      <div class="summary-row"><span class="label">{{ t('marketplace.buyer') }}</span><span class="value text-mono">{{ purchase.buyerRS }}</span></div>
      <div class="summary-row"><span class="label">{{ t('marketplace.quantity') }}</span><span class="value">{{ purchase.quantity }}</span></div>
      <div class="summary-row"><span class="label">{{ t('marketplace.price') }}</span><span class="value">{{ formatNQT(purchase.priceNQT) }} NRC</span></div>
    </div>
    <el-form ref="formRef" :model="form" label-position="top" class="mt-4">
      <el-form-item :label="t('marketplace.goodsData')"><el-input v-model="form.goodsData" type="textarea" :rows="4" /></el-form-item>
      <el-form-item>
        <el-checkbox v-model="form.goodsIsText">{{ t('marketplace.goodsDataIsText') }}</el-checkbox>
      </el-form-item>
      <el-form-item :label="t('marketplace.discount')"><el-input v-model="form.discountNQT" type="number" placeholder="0.00" clearable><template #append>NRC</template></el-input></el-form-item>
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
      <el-button type="primary" :loading="loading" @click="handleSubmit">{{ t('marketplace.deliver') }}</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, type FormInstance } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'

const { t } = useI18n()
const props = defineProps<{ purchase: any }>()
const visible = defineModel<boolean>('visible', { default: false })
const emit = defineEmits<{ (e: 'success'): void }>()
const formRef = ref<FormInstance>()
const loading = ref(false)
const form = reactive({ goodsData: '', goodsIsText: true, discountNQT: '', feeNQT: '1', deadline: '1440', secretPhrase: '' })

function formatNQT(nqt: string) { return (Number(nqt || '0') / 1e8).toFixed(2) }

async function handleSubmit() {
  if (!formRef.value) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    loading.value = true
    try {
      const data: any = { secretPhrase: form.secretPhrase, purchase: props.purchase.purchase, feeNQT: String(Math.round(Number(form.feeNQT) * 1e8)), deadline: Number(form.deadline) }
      if (form.goodsData) { data.goodsData = form.goodsData; data.goodsIsText = form.goodsIsText }
      if (form.discountNQT) data.discountNQT = String(Math.round(Number(form.discountNQT) * 1e8))
      await nrcsApi.dgsDelivery(data)
      ElMessage.success(t('marketplace.deliverSuccess'))
      emit('success'); handleClose()
    } catch (err: any) { ElMessage.error(err?.message || t('marketplace.deliverError')) }
    finally { loading.value = false }
  })
}
function handleClose() { formRef.value?.resetFields(); visible.value = false }
</script>
<style scoped lang="scss">
@use '@/assets/styles/variables' as *;
.order-summary { background: rgba(255,255,255,0.02); border: 1px solid $border-subtle; border-radius: $radius-md; padding: $space-lg; margin-bottom: $space-md; }
.summary-row { display: flex; justify-content: space-between; padding: $space-xs 0; .label { color: $text-muted; font-size: $font-size-sm; } .value { color: $text-primary; font-weight: 500; } }
.mt-4 { margin-top: $space-lg; }
</style>
