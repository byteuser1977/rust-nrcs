<template>
  <el-dialog
    v-model="visible"
    :title="t('asset.transferAsset')"
    width="520px"
    :close-on-click-modal="false"
    destroy-on-close
    class="nrcs-modal"
    @close="handleClose"
  >
    <el-form ref="formRef" :model="form" :rules="rules" label-position="top">
      <el-form-item :label="t('asset.assetName')">
        <el-input :model-value="asset?.name" disabled />
      </el-form-item>
      <el-form-item :label="t('common.recipient')" prop="recipient">
        <el-input v-model="form.recipient" placeholder="NRCS-XXXX-XXXX-XXXX-XXXXX" clearable />
      </el-form-item>
      <el-form-item :label="t('asset.quantity')" prop="quantity">
        <el-input v-model="form.quantity" type="number" :placeholder="t('asset.quantityPlaceholder')" clearable>
          <template #append>{{ t('asset.shares') }}</template>
        </el-input>
        <div v-if="asset" class="balance-hint">
          <span>{{ t('asset.availableBalance') }}:</span>
          <span class="balance-value" @click="fillMaxQuantity">{{ formatQNT(yourBalanceQNT, asset.decimals) }}</span>
        </div>
      </el-form-item>
      <el-row :gutter="16">
        <el-col :span="12">
          <el-form-item :label="t('common.fee')" prop="feeNQT">
            <el-input v-model="form.feeNQT" placeholder="1" type="number" clearable>
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
      <el-button type="primary" :loading="loading" @click="handleSubmit">{{ t('asset.transferAsset') }}</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * TransferAssetModal 组件 —— 资产转账弹窗。
 *
 * 对标 nrs.assetexchange.js 的 transferAsset 流程。
 *
 * 安全模型：secretPhrase 不随请求外发，通过 useNrcsForm 三步本地签名流程提交。
 */
import { ref, reactive, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, type FormInstance, type FormRules } from 'element-plus'
import { useAccountStore } from '@/stores/modules/account.store'
import { useNrcsForm } from '@/composables/useNrcsForm'
import { qntToQntf, qntfToQnt } from '@/utils/format'
import type { NrcsAsset } from '@/api/modules/nrcs.api'

const { t } = useI18n()
const accountStore = useAccountStore()
const { submitForm } = useNrcsForm()

const props = defineProps<{
  /** 资产信息 */
  asset: NrcsAsset | null
  /** 用户持有的该资产数量 QNT（用于"全部"按钮） */
  yourBalanceQNT?: string
}>()

const visible = defineModel<boolean>('visible', { default: false })
const emit = defineEmits<{ (e: 'success'): void }>()
const formRef = ref<FormInstance>()
const loading = ref(false)

const form = reactive({
  recipient: '',
  quantity: '',
  feeNQT: '1',
  deadline: '1440',
  secretPhrase: '',
})

/** 是否需要显示 secretPhrase 输入框 */
const needsSecretPhrase = computed(() => !accountStore.hasSecretPhrase)

/** 格式化 QNT 为可读数量 */
function formatQNT(qnt: string | undefined, decimals: number): string {
  if (!qnt) return '0'
  try {
    return qntToQntf(qnt, decimals)
  } catch {
    return qnt
  }
}

/** 填入最大可转数量 */
function fillMaxQuantity(): void {
  if (!props.asset || !props.yourBalanceQNT) return
  try {
    form.quantity = qntToQntf(props.yourBalanceQNT, props.asset.decimals)
  } catch {
    // ignore
  }
}

const rules = computed<FormRules>(() => ({
  recipient: [{ required: true, message: t('asset.recipientRequired'), trigger: 'blur' }],
  quantity: [
    { required: true, message: t('asset.quantityRequired'), trigger: 'blur' },
    {
      validator: (_rule: any, value: string, callback: (err?: Error) => void) => {
        const num = Number(value)
        if (isNaN(num) || num <= 0) {
          callback(new Error(t('asset.quantityInvalid')))
        } else {
          callback()
        }
      },
      trigger: 'blur',
    },
  ],
  feeNQT: [{ required: true, message: t('asset.feeRequired'), trigger: 'blur' }],
  deadline: [{ required: true, message: t('asset.deadlineRequired'), trigger: 'blur' }],
  secretPhrase: needsSecretPhrase.value
    ? [{ required: true, message: t('common.secretPhraseRequired'), trigger: 'blur' }]
    : [],
}))

async function handleSubmit() {
  if (!formRef.value || !props.asset) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    loading.value = true
    try {
      const decimals = props.asset.decimals
      const secretPhrase = accountStore.hasSecretPhrase
        ? accountStore.secretPhrase
        : form.secretPhrase

      if (!secretPhrase) {
        ElMessage.warning(t('common.secretPhraseRequired'))
        return
      }

      // 数量 QNTf → QNT
      const quantityQNT = qntfToQnt(form.quantity, decimals)

      const data: Record<string, any> = {
        secretPhrase,
        asset: props.asset.asset,
        recipient: form.recipient,
        quantityQNT,
        feeNXT: form.feeNQT,
        deadline: form.deadline,
      }

      await submitForm('transferAsset', data, {
        successMessage: t('asset.transferSuccess'),
      })

      emit('success')
      handleClose()
    } catch (err: any) {
      console.error('Transfer asset failed:', err)
    } finally {
      loading.value = false
    }
  })
}

function handleClose() {
  formRef.value?.resetFields()
  Object.assign(form, {
    recipient: '',
    quantity: '',
    feeNQT: '1',
    deadline: '1440',
    secretPhrase: '',
  })
  visible.value = false
}
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;

.balance-hint {
  margin-top: 4px;
  font-size: $font-size-xs;
  color: $text-muted;

  .balance-value {
    color: $primary;
    cursor: pointer;
    font-weight: 600;
    margin-left: 4px;

    &:hover {
      text-decoration: underline;
    }
  }
}
</style>
