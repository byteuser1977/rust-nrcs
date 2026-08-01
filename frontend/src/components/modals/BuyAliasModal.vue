<template>
  <el-dialog v-model="visible" :title="t('alias.buyAlias')" width="500px" :close-on-click-modal="false" destroy-on-close class="nrcs-modal" @close="handleClose">
    <el-form ref="formRef" :model="form" :rules="rules" label-position="top">
      <el-form-item :label="t('alias.aliasName')">
        <el-input v-model="aliasName" disabled />
      </el-form-item>
      <el-form-item v-if="aliasPrice" :label="t('alias.price')">
        <el-input :model-value="aliasPrice" disabled><template #append>NRC</template></el-input>
      </el-form-item>
      <el-row :gutter="16">
        <el-col :span="12"><el-form-item :label="t('common.fee')" prop="feeNQT"><el-input v-model="form.feeNQT" placeholder="1" type="number" clearable><template #append>NRC</template></el-input></el-form-item></el-col>
        <el-col :span="12"><el-form-item :label="t('common.deadline')" prop="deadline"><el-input v-model="form.deadline" placeholder="1440" type="number" clearable><template #append>{{ t('common.minutes') }}</template></el-input></el-form-item></el-col>
      </el-row>
      <el-form-item v-if="needsSecretPhrase" :label="t('common.secretPhrase')" prop="secretPhrase">
        <el-input v-model="form.secretPhrase" type="password" show-password clearable />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="handleClose">{{ t('common.cancel') }}</el-button>
      <el-button type="primary" :loading="loading" @click="handleSubmit">{{ t('alias.buy') }}</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * BuyAliasModal 组件 —— 购买别名弹窗。
 *
 * 对标 nrs.aliases.js 的 buyAlias 表单（nrs.aliases.js:274-326）。
 *
 * 安全模型：secretPhrase 不随请求外发，由 useNrcsForm 在本地签名。
 */
import { ref, reactive, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { type FormInstance, type FormRules } from 'element-plus'
import { useAccountStore } from '@/stores/modules/account.store'
import { useNrcsForm } from '@/composables/useNrcsForm'

const { t } = useI18n()
const accountStore = useAccountStore()
const { submitForm } = useNrcsForm()

const props = defineProps<{ alias?: any }>()
const visible = defineModel<boolean>('visible', { default: false })
const emit = defineEmits<{ (e: 'success'): void }>()
const formRef = ref<FormInstance>()
const loading = ref(false)
const aliasName = computed(() => props.alias?.aliasName || '')
const aliasPrice = computed(() => props.alias?.priceNQT ? (Number(props.alias.priceNQT) / 1e8).toFixed(2) : '')

const form = reactive({ feeNQT: '1', deadline: '1440', secretPhrase: '' })

const needsSecretPhrase = computed(() => !accountStore.hasSecretPhrase)

/** 弹窗打开时预填 secretPhrase */
watch(visible, (open) => {
  if (open) {
    form.secretPhrase = accountStore.secretPhrase || ''
  }
})

const rules = computed<FormRules>(() => ({
  feeNQT: [{ required: true, message: t('alias.feeRequired'), trigger: 'blur' }],
  deadline: [{ required: true, message: t('alias.deadlineRequired'), trigger: 'blur' }],
  secretPhrase: needsSecretPhrase.value
    ? [{ required: true, message: t('common.secretPhraseRequired'), trigger: 'blur' }]
    : []
}))

/**
 * 提交购买别名（通过 useNrcsForm 三步本地签名流程）。
 *
 * 购买价格由 props.alias.priceNQT 决定（服务端校验），客户端传入 amountNXT。
 */
async function handleSubmit(): Promise<void> {
  if (!formRef.value) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    loading.value = true
    try {
      await submitForm('buyAlias', {
        aliasName: aliasName.value,
        amountNXT: aliasPrice.value,
        feeNXT: form.feeNQT,
        deadline: form.deadline,
        secretPhrase: form.secretPhrase
      }, {
        successMessage: t('alias.buySuccess')
      })
      emit('success')
      handleClose()
    } catch (err: any) {
      console.error('Buy alias failed:', err)
    } finally {
      loading.value = false
    }
  })
}

function handleClose(): void {
  formRef.value?.resetFields()
  Object.assign(form, { feeNQT: '1', deadline: '1440', secretPhrase: '' })
  visible.value = false
}
</script>
<style scoped lang="scss">@use '@/assets/styles/variables' as *;</style>
