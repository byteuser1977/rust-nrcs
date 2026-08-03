<template>
  <el-dialog v-model="visible" :title="t('monetary.claimCurrency')" width="520px" :close-on-click-modal="false" destroy-on-close class="nrcs-modal" @close="handleClose">
    <el-form ref="formRef" :model="form" :rules="rules" label-position="top">
      <el-descriptions :column="1" border size="small" class="mb-16">
        <el-descriptions-item :label="t('monetary.code')">{{ currency?.code || '-' }}</el-descriptions-item>
        <el-descriptions-item :label="t('monetary.availableUnits')">
          {{ availableUnits }}
        </el-descriptions-item>
        <el-descriptions-item :label="t('monetary.claimRate')">
          {{ claimRateDisplay }} [NRC/{{ currency?.code || '?' }}]
        </el-descriptions-item>
      </el-descriptions>

      <el-form-item :label="t('monetary.unitsToClaim')" prop="units">
        <el-input v-model="form.units" type="number" :placeholder="t('monetary.unitsPlaceholder')" clearable />
      </el-form-item>
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
      <el-button type="primary" :loading="loading" @click="handleSubmit">{{ t('monetary.claimCurrency') }}</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * CurrencyClaimModal.vue —— 领取货币储备（reserveClaim）弹窗。
 *
 * 对标参考 `nrs.monetarysystem.js:1529` 的 `#claim_currency_modal` 与
 * `nrs.monetarysystem.js:1571` 的 `NRS.forms.currencyReserveClaim`。
 *
 * 流程：
 *   1. 打开时调用 `getAccountCurrencies` 获取可用 units，`getCurrency` 获取 currentReservePerUnitNQT
 *   2. 用户输入 units（QNTf），提交时 `convertToQNT(units, decimals)` 转 QNT
 *   3. 通过 `submitForm('currencyReserveClaim', {currency, unitsQNT, ...})` 本地签名
 *
 * 仅当 `isClaimable(type) && issuanceHeight <= lastBlockHeight` 时可操作（对标参考 disabled 条件）。
 */
import { ref, reactive, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, type FormInstance, type FormRules } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { useAccountStore } from '@/stores/modules/account.store'
import { useNrcsForm } from '@/composables/useNrcsForm'
import { formatNqtToNrc, qntToQntf, qntfToQnt } from '@/utils/format'

const props = defineProps<{ currency: any }>()
const visible = defineModel<boolean>('visible', { default: false })
const emit = defineEmits<{ (e: 'success'): void }>()

const { t } = useI18n()
const accountStore = useAccountStore()
const { submitForm } = useNrcsForm()

const formRef = ref<FormInstance>()
const loading = ref(false)
const needsSecretPhrase = computed(() => !accountStore.hasSecretPhrase)

const decimals = ref(0)
const availableUnits = ref('0')
const claimRateNqt = ref('0')

const form = reactive({
  units: '',
  feeNXT: '1',
  deadline: '1440',
  secretPhrase: '',
})

const rules = computed<FormRules>(() => ({
  units: [{ required: true, message: t('validation.required'), trigger: 'blur' }],
  feeNXT: [{ required: true, message: t('validation.required'), trigger: 'blur' }],
  deadline: [{ required: true, message: t('validation.required'), trigger: 'blur' }],
  secretPhrase: needsSecretPhrase.value
    ? [{ required: true, message: t('common.secretPhraseRequired'), trigger: 'blur' }]
    : [],
}))

/** claimRate 展示（对标 claimRate = currentReservePerUnitNQT * 10^decimals） */
const claimRateDisplay = computed(() => {
  try {
    const v = BigInt(claimRateNqt.value || '0')
    const factor = 10n ** BigInt(decimals.value)
    return formatNqtToNrc((v * factor).toString())
  } catch {
    return '0'
  }
})

/**
 * 打开时加载可用 units 与 claimRate（对标 claim_currency_modal show.bs.modal）。
 */
watch(visible, async (val) => {
  if (!val || !props.currency?.currency) return
  form.units = ''
  form.feeNXT = '1'
  form.deadline = '1440'
  form.secretPhrase = accountStore.secretPhrase || ''
  availableUnits.value = '0'
  claimRateNqt.value = '0'
  decimals.value = props.currency.decimals ?? 0
  try {
    // 对标 getAccountCurrencies 获取可用 units
    const acctResult = await nrcsApi.getAccountCurrencies(accountStore.accountRS, 0, 999)
    const bal = (acctResult?.currencyBalances || []).find(
      (b: any) => b.currency === props.currency.currency,
    )
    if (bal?.units) {
      availableUnits.value = qntToQntf(bal.units, decimals.value)
    } else if (bal?.unconfirmedUnits) {
      availableUnits.value = qntToQntf(bal.unconfirmedUnits, decimals.value)
    }
  } catch {
    // 忽略，保留 0
  }
  try {
    // 对标 getCurrency 获取 currentReservePerUnitNQT
    const detail = await nrcsApi.getCurrency(props.currency.currency)
    decimals.value = detail.decimals ?? decimals.value
    claimRateNqt.value = detail.currentReservePerUnitNQT || '0'
  } catch {
    // 忽略
  }
})

/**
 * 提交领取储备（对标 NRS.forms.currencyReserveClaim）。
 *
 * unitsQNT = convertToQNT(units, decimals)（对标参考 data.units = NRS.convertToQNT(data.units, decimals)）。
 */
async function handleSubmit(): Promise<void> {
  if (!formRef.value) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    loading.value = true
    try {
      const unitsQNT = qntfToQnt(form.units, decimals.value)
      await submitForm('currencyReserveClaim', {
        currency: props.currency.currency,
        unitsQNT,
        feeNXT: form.feeNXT,
        deadline: form.deadline,
        secretPhrase: form.secretPhrase,
      }, {
        successMessage: t('monetary.claimSuccess'),
      })
      emit('success')
      handleClose()
    } catch (e: any) {
      ElMessage.error(e?.message || t('monetary.claimError'))
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
.mb-16 { margin-bottom: 16px; }
</style>
