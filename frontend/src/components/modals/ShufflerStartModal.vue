<template>
  <el-dialog v-model="visible" :title="t('shuffling.startShuffler')" width="480px" :close-on-click-modal="false" destroy-on-close @close="handleClose">
    <el-form ref="formRef" :model="form" label-width="120px" label-position="top">
      <el-form-item :label="t('shuffling.shufflingId')">
        <span class="text-mono">{{ shuffling?.shuffling }}</span>
      </el-form-item>
      <el-form-item :label="t('shuffling.shufflingFullHash')" v-if="shuffling?.shufflingFullHash">
        <span class="text-mono text-sm">{{ truncate(shuffling.shufflingFullHash, 12) }}</span>
      </el-form-item>
      <el-form-item :label="t('login.secretPhrase')" prop="secretPhrase" :rules="[{ required: true, message: t('login.secretPhraseRequired') }]">
        <el-input v-model="form.secretPhrase" type="password" show-password />
      </el-form-item>
      <el-form-item :label="t('shuffling.recipientSecretPhrase')" v-if="form.secretPhrase">
        <el-input v-model="form.recipientSecretPhrase" type="password" show-password :placeholder="t('common.optional')" />
      </el-form-item>
      <el-form-item :label="t('common.fee')" prop="feeNQT">
        <el-input v-model="form.feeNQT" placeholder="1"><template #append>NRC</template></el-input>
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="handleClose">{{ t('common.cancel') }}</el-button>
      <el-button type="primary" :loading="submitting" @click="handleSubmit">{{ t('shuffling.startShuffler') }}</el-button>
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

const props = defineProps<{ shuffling: any }>()
const visible = defineModel<boolean>('visible', { default: false })
const emit = defineEmits<{ (e: 'success'): void }>()

const formRef = ref()
const submitting = ref(false)
const form = reactive({ secretPhrase: '', recipientSecretPhrase: '', feeNQT: '1' })

function truncate(hash: string, len: number) {
  return hash ? hash.slice(0, len) + '...' + hash.slice(-len) : ''
}

watch(visible, (val) => {
  if (val) {
    form.secretPhrase = accountStore.secretPhrase || ''
    form.recipientSecretPhrase = ''
    form.feeNQT = '1'
  }
})

async function handleSubmit() {
  if (!formRef.value) return
  submitting.value = true
  try {
    await nrcsApi.startShuffler({
      secretPhrase: form.secretPhrase,
      shufflingFullHash: props.shuffling?.shufflingFullHash,
      recipientSecretPhrase: form.recipientSecretPhrase || undefined,
      feeNQT: String(Math.round(Number(form.feeNQT) * 1e8)),
      deadline: 1440,
    } as any)
    ElMessage.success(t('common.operationSuccess'))
    emit('success')
    visible.value = false
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.operationFailed'))
  } finally {
    submitting.value = false
  }
}

function handleClose() { formRef.value?.resetFields(); visible.value = false }
</script>

<style scoped>
.text-mono { font-family: monospace; }
.text-sm { font-size: 12px; }
</style>
