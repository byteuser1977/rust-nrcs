<template>
  <el-dialog v-model="visible" :title="t('marketplace.delete')" width="400px" :close-on-click-modal="false" destroy-on-close @close="handleClose">
    <el-form ref="formRef" :model="form" label-width="100px" label-position="top">
      <el-form-item v-if="product" :label="t('marketplace.product')">
        <span>{{ product.name }}</span>
      </el-form-item>
      <el-form-item :label="t('login.secretPhrase')" prop="secretPhrase">
        <el-input v-model="form.secretPhrase" type="password" show-password />
      </el-form-item>
      <el-form-item :label="t('common.fee')" prop="feeNQT">
        <el-input v-model="form.feeNQT" placeholder="1"><template #append>NRC</template></el-input>
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="handleClose">{{ t('common.cancel') }}</el-button>
      <el-button type="danger" :loading="submitting" @click="handleSubmit">{{ t('marketplace.deleteConfirm') }}</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, reactive, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { useAccountStore } from '@/stores/modules/account.store'

const { t } = useI18n()
const accountStore = useAccountStore()

const props = defineProps<{ product: any }>()
const visible = defineModel<boolean>('visible', { default: false })
const emit = defineEmits<{ (e: 'success'): void }>()

const formRef = ref()
const submitting = ref(false)
const form = reactive({ secretPhrase: '', feeNQT: '1' })

watch(visible, (val) => {
  if (val) {
    form.secretPhrase = accountStore.secretPhrase || ''
    form.feeNQT = '1'
  }
})

async function handleSubmit() {
  if (!formRef.value) return
  submitting.value = true
  try {
    await nrcsApi.dgsDelisting({
      secretPhrase: form.secretPhrase || accountStore.secretPhrase,
      goods: props.product?.goods,
      feeNQT: String(Math.round(Number(form.feeNQT) * 1e8)),
      deadline: 1440,
    } as any)
    ElMessage.success(t('marketplace.deleteSuccess'))
    emit('success')
    visible.value = false
  } catch (e: any) {
    ElMessage.error(e?.message || t('marketplace.deleteError'))
  } finally {
    submitting.value = false
  }
}

function handleClose() { formRef.value?.resetFields(); visible.value = false }
</script>
