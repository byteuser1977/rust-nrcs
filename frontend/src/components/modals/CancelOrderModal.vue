<template>
  <el-dialog v-model="visible" :title="t('asset.cancelOrder')" width="450px" :close-on-click-modal="false" destroy-on-close @close="handleClose">
    <el-form ref="formRef" :model="form" label-width="100px" label-position="top">
      <el-form-item v-if="order" :label="t('asset.orderInfo')">
        <el-descriptions :column="1" border size="small">
          <el-descriptions-item :label="t('asset.asset')">{{ order.name || order.asset }}</el-descriptions-item>
          <el-descriptions-item :label="t('asset.quantity')">{{ order.quantityQNT || '—' }}</el-descriptions-item>
          <el-descriptions-item :label="t('asset.price')">{{ formatOrderPrice(order.priceNQT, order.decimals || 0) }} NRC</el-descriptions-item>
        </el-descriptions>
      </el-form-item>
      <el-form-item :label="t('login.secretPhrase')" prop="secretPhrase" :rules="[{ required: true, message: t('login.secretPhraseRequired') }]">
        <el-input v-model="form.secretPhrase" type="password" show-password :placeholder="t('login.secretPhraseRequired')" />
      </el-form-item>
      <el-form-item :label="t('common.fee')" prop="feeNQT">
        <el-input v-model="form.feeNQT" placeholder="1">
          <template #append>NRC</template>
        </el-input>
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="handleClose">{{ t('common.cancel') }}</el-button>
      <el-button type="danger" :loading="submitting" @click="handleSubmit">{{ t('common.confirm') }}</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, reactive, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { useAccountStore } from '@/stores/modules/account.store'
import { formatOrderPricePerWholeQNT } from '@/utils/format'

const { t } = useI18n()
const accountStore = useAccountStore()

const props = defineProps<{
  order: any
  orderType: 'bid' | 'ask'
}>()

const visible = defineModel<boolean>('visible', { default: false })
const emit = defineEmits<{ (e: 'success'): void }>()

const formRef = ref()
const submitting = ref(false)
const form = reactive({ secretPhrase: '', feeNQT: '1' })

function formatOrderPrice(price: string, decimals: number) {
  try { return formatOrderPricePerWholeQNT(price, decimals) } catch { return price }
}

watch(visible, (val) => {
  if (val) {
    form.secretPhrase = accountStore.secretPhrase || ''
    form.feeNQT = '1'
  }
})

async function handleSubmit() {
  if (!formRef.value) return
  await formRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    submitting.value = true
    try {
      const data: any = {
        secretPhrase: form.secretPhrase || accountStore.secretPhrase,
        order: props.order?.order || props.order?.bidOrder || props.order?.askOrder,
        feeNQT: String(Math.round(Number(form.feeNQT) * 1e8)),
        deadline: 1440,
      }
      if (props.orderType === 'bid') {
        await nrcsApi.cancelBidOrder(data)
      } else {
        await nrcsApi.cancelAskOrder(data)
      }
      ElMessage.success(t('asset.cancelSuccess'))
      emit('success')
      visible.value = false
    } catch (e: any) {
      ElMessage.error(e?.message || t('asset.cancelError'))
    } finally {
      submitting.value = false
    }
  })
}

function handleClose() {
  formRef.value?.resetFields()
  visible.value = false
}
</script>
