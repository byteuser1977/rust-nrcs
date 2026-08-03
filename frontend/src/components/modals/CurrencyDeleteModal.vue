<template>
  <el-dialog
    v-model="visible"
    :title="t('monetary.deleteCurrencyTitle', { code: currency?.code || '' })"
    width="480px"
    :close-on-click-modal="false"
    destroy-on-close
    class="nrcs-modal"
    @close="handleClose"
  >
    <el-form ref="formRef" :model="form" :rules="rules" label-position="top">
      <!-- 货币信息（对标 delete_currency_modal 显示） -->
      <el-descriptions :column="1" border size="small" class="mb-16">
        <el-descriptions-item :label="t('monetary.code')">
          {{ currency?.code || '-' }}
        </el-descriptions-item>
        <el-descriptions-item :label="t('common.name')">
          {{ currency?.name || '-' }}
        </el-descriptions-item>
        <el-descriptions-item :label="t('monetary.currentSupply')">
          {{ formatQNT(currency?.currentSupply, currency?.decimals ?? 0) }}
        </el-descriptions-item>
      </el-descriptions>

      <el-alert
        :title="t('monetary.deleteConfirm')"
        type="warning"
        :closable="false"
        show-icon
        class="mb-16"
      />

      <el-row :gutter="16">
        <el-col :span="12">
          <el-form-item :label="t('common.fee')" prop="feeNXT">
            <el-input v-model="form.feeNXT" placeholder="1" type="number" clearable>
              <template #append>NRC</template>
            </el-input>
          </el-form-item>
        </el-col>
        <el-col :span="12">
          <el-form-item :label="t('common.deadline')" prop="deadline">
            <el-input v-model="form.deadline" placeholder="1440" type="number" clearable>
              <template #append>{{ t('common.minutes') }}</template>
            </el-input>
          </el-form-item>
        </el-col>
      </el-row>
      <el-form-item v-if="needsSecretPhrase" :label="t('common.secretPhrase')" prop="secretPhrase">
        <el-input v-model="form.secretPhrase" type="password" show-password clearable />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="handleClose">{{ t('common.cancel') }}</el-button>
      <el-button type="danger" :loading="loading" @click="handleSubmit">
        {{ t('monetary.delete') }}
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * CurrencyDeleteModal.vue —— 删除货币（deleteCurrency）弹窗。
 *
 * 对标参考 `nrs.server.js:1235` 的 deleteCurrency 交易验证（type=5, subtype=8）。
 *
 * 流程：
 *   1. 仅发行者（issuer === 当前账户）可删除
 *   2. 通过 `useNrcsForm.submitForm('deleteCurrency', { currency, ... })` 本地签名
 *   3. secretPhrase 不外发，客户端本地签名
 *
 * 删除条件（对标 Java CurrencyDeleteAttachment）：货币未被使用（无交易/无持有者）。
 */
import { ref, reactive, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, type FormInstance, type FormRules } from 'element-plus'
import { useAccountStore } from '@/stores/modules/account.store'
import { useNrcsForm } from '@/composables/useNrcsForm'
import { qntToQntf } from '@/utils/format'

const props = defineProps<{ currency: any }>()
const visible = defineModel<boolean>('visible', { default: false })
const emit = defineEmits<{ (e: 'success'): void }>()

const { t } = useI18n()
const accountStore = useAccountStore()
const { submitForm } = useNrcsForm()

const formRef = ref<FormInstance>()
const loading = ref(false)
const needsSecretPhrase = computed(() => !accountStore.hasSecretPhrase)

const form = reactive({
  feeNXT: '1',
  deadline: '1440',
  secretPhrase: '',
})

const rules = computed<FormRules>(() => ({
  feeNXT: [{ required: true, message: t('validation.required'), trigger: 'blur' }],
  deadline: [{ required: true, message: t('validation.required'), trigger: 'blur' }],
  secretPhrase: needsSecretPhrase.value
    ? [{ required: true, message: t('common.secretPhraseRequired'), trigger: 'blur' }]
    : [],
}))

/**
 * 打开时重置表单（对标 delete_currency_modal show.bs.modal）。
 */
watch(visible, (val) => {
  if (!val) return
  form.feeNXT = '1'
  form.deadline = '1440'
  form.secretPhrase = accountStore.secretPhrase || ''
})

/** 格式化 QNT 数量（对标 NRS.formatQuantity） */
function formatQNT(qnt: string | undefined, decimals: number): string {
  if (!qnt) return '0'
  try {
    return qntToQntf(qnt, decimals)
  } catch {
    return qnt
  }
}

/**
 * 提交删除货币（对标 NRS.forms.deleteCurrency）。
 *
 * deleteCurrency 交易仅包含 currency ID（8 字节），对标 server.js:1235-1244。
 */
async function handleSubmit(): Promise<void> {
  if (!formRef.value) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    loading.value = true
    try {
      await submitForm(
        'deleteCurrency',
        {
          currency: props.currency?.currency,
          feeNXT: form.feeNXT,
          deadline: form.deadline,
          secretPhrase: form.secretPhrase,
        },
        {
          successMessage: t('monetary.deleteCurrencySuccess'),
        },
      )
      emit('success')
      handleClose()
    } catch (e: any) {
      ElMessage.error(e?.message || t('monetary.deleteCurrencyError'))
    } finally {
      loading.value = false
    }
  })
}

function handleClose(): void {
  formRef.value?.resetFields()
  visible.value = false
}
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;
.mb-16 {
  margin-bottom: 16px;
}
</style>
