<template>
  <el-dialog v-model="visible" :title="t('monetary.reserveCurrency')" width="560px" :close-on-click-modal="false" destroy-on-close class="nrcs-modal" @close="handleClose">
    <el-form ref="formRef" :model="form" :rules="rules" label-position="top">
      <!-- 货币信息（对标 reserve_currency_code/currency 显示） -->
      <el-descriptions :column="2" border size="small" class="mb-16">
        <el-descriptions-item :label="t('monetary.code')">{{ currency?.code || '-' }}</el-descriptions-item>
        <el-descriptions-item :label="t('monetary.decimals')">{{ currencyInfo.decimals }}</el-descriptions-item>
        <el-descriptions-item :label="t('monetary.minReservePerUnit')">
          {{ formatNqtToNrc(minReservePerUnitNqt) }} NRC
        </el-descriptions-item>
        <el-descriptions-item :label="t('monetary.currentReservePerUnit')">
          {{ formatNqtToNrc(currentReservePerUnitNqt) }} NRC
        </el-descriptions-item>
        <el-descriptions-item :label="t('monetary.reserveSupply')">
          {{ formatQNT(currencyInfo.reserveSupply, currencyInfo.decimals) }}
        </el-descriptions-item>
        <el-descriptions-item :label="t('monetary.initialSupply')">
          {{ formatQNT(currencyInfo.initialSupply, currencyInfo.decimals) }}
        </el-descriptions-item>
      </el-descriptions>

      <el-form-item :label="t('monetary.totalAmountNXT')" prop="amountNXT">
        <el-input v-model="form.amountNXT" type="number" :placeholder="t('monetary.totalAmountPlaceholder')" clearable @blur="recalculate">
          <template #append>NRC</template>
        </el-input>
      </el-form-item>
      <el-form-item :label="t('monetary.amountPerUnit')">
        <el-input :model-value="perUnitDisplay" disabled>
          <template #append>NRC / {{ t('monetary.unit') }}</template>
        </el-input>
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
      <el-button type="primary" :loading="loading" @click="handleSubmit">{{ t('monetary.reserveCurrency') }}</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * CurrencyReserveModal.vue —— 增加货币储备（reserveIncrease）弹窗。
 *
 * 对标参考 `nrs.monetarysystem.js:1466` 的 `#reserve_currency_modal` 与
 * `nrs.monetarysystem.js:1518` 的 `NRS.forms.currencyReserveIncrease`。
 *
 * 流程：
 *   1. 打开时调用 `getCurrency` 获取最新储备信息（minReserve/currentReserve/resSupply/initialSupply）
 *   2. 用户输入总 NRC 金额，blur 时按参考 `reserve_currency_amount.blur` 计算每单位 NQT 与总额
 *   3. 提交时计算 `amountPerUnitNQT = (totalNQT / resSupply) / 10^decimals`
 *      （对标 `calculatePricePerWholeQNT(convertToNQT(amount), decimals)`）
 *   4. 通过 `useNrcsForm.submitForm('currencyReserveIncrease', ...)` 本地签名，secretPhrase 不外发
 *
 * 仅当 `isReservable(type) && issuanceHeight > lastBlockHeight` 时可操作（对标参考 disabled 条件）。
 */
import { ref, reactive, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, type FormInstance, type FormRules } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { useAccountStore } from '@/stores/modules/account.store'
import { useNodeStore } from '@/stores/modules/node.store'
import { useNrcsForm } from '@/composables/useNrcsForm'
import { formatNqtToNrc, qntToQntf } from '@/utils/format'

const props = defineProps<{ currency: any }>()
const visible = defineModel<boolean>('visible', { default: false })
const emit = defineEmits<{ (e: 'success'): void }>()

const { t } = useI18n()
const accountStore = useAccountStore()
const nodeStore = useNodeStore()
const { submitForm } = useNrcsForm()

const formRef = ref<FormInstance>()
const loading = ref(false)
const needsSecretPhrase = computed(() => !accountStore.hasSecretPhrase)

/** 货币最新信息（getCurrency 返回） */
const currencyInfo = reactive({
  currency: '',
  decimals: 0,
  minReservePerUnitNQT: '0',
  currentReservePerUnitNQT: '0',
  reserveSupply: '0',
  initialSupply: '0',
})

const form = reactive({
  amountNXT: '',
  feeNXT: '1',
  deadline: '1440',
  secretPhrase: '',
})

const rules = computed<FormRules>(() => ({
  amountNXT: [{ required: true, message: t('validation.required'), trigger: 'blur' }],
  feeNXT: [{ required: true, message: t('validation.required'), trigger: 'blur' }],
  deadline: [{ required: true, message: t('validation.required'), trigger: 'blur' }],
  secretPhrase: needsSecretPhrase.value
    ? [{ required: true, message: t('common.secretPhraseRequired'), trigger: 'blur' }]
    : [],
}))

/** minReservePerUnitNQT 按 decimals 放大后的 NQT（对标参考 minReservePerUnitNQT = minReserve * 10^decimals） */
const minReservePerUnitNqt = computed(() => {
  const v = BigInt(currencyInfo.minReservePerUnitNQT || '0')
  const factor = 10n ** BigInt(currencyInfo.decimals)
  return (v * factor).toString()
})
const currentReservePerUnitNqt = computed(() => {
  const v = BigInt(currencyInfo.currentReservePerUnitNQT || '0')
  const factor = 10n ** BigInt(currencyInfo.decimals)
  return (v * factor).toString()
})

/** 每单位 NXT 展示值（对标 reserve_currency_total） */
const perUnitDisplay = computed(() => {
  if (!form.amountNXT) return '0'
  try {
    return formatNqtToNrc(perUnitNqt.value)
  } catch {
    return '0'
  }
})

/** 每 whole unit 的 NQT（对标参考 unitAmountNQT） */
const perUnitNqt = computed(() => {
  if (!form.amountNXT) return '0'
  try {
    const totalNqt = BigInt(Math.round(Number(form.amountNXT) * 1e8))
    const resSupplyWhole = BigInt(Math.round(Number(qntToQntf(currencyInfo.reserveSupply, currencyInfo.decimals))))
    if (resSupplyWhole === 0n) return '0'
    return (totalNqt / resSupplyWhole).toString()
  } catch {
    return '0'
  }
})

/**
 * 打开时加载货币最新信息（对标 reserve_currency_modal show.bs.modal 调用 getCurrency+）。
 */
watch(visible, async (val) => {
  if (!val || !props.currency?.currency) return
  try {
    const detail = await nrcsApi.getCurrency(props.currency.currency)
    currencyInfo.currency = detail.currency || props.currency.currency
    currencyInfo.decimals = detail.decimals ?? 0
    currencyInfo.minReservePerUnitNQT = detail.minReservePerUnitNQT || '0'
    currencyInfo.currentReservePerUnitNQT = detail.currentReservePerUnitNQT || '0'
    currencyInfo.reserveSupply = detail.reserveSupply || '0'
    currencyInfo.initialSupply = detail.initialSupply || '0'
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  }
  form.amountNXT = ''
  form.feeNXT = '1'
  form.deadline = '1440'
  form.secretPhrase = accountStore.secretPhrase || ''
})

function formatQNT(qnt: string, decimals: number): string {
  if (!qnt) return '0'
  try {
    return qntToQntf(qnt, decimals)
  } catch {
    return qnt
  }
}

/** blur 时重算（对标 reserve_currency_amount.blur） */
function recalculate(): void {
  // perUnitDisplay 为 computed，自动更新；此处保留以对标参考的 blur 钩子语义
}

/**
 * 提交增加储备（对标 NRS.forms.currencyReserveIncrease）。
 *
 * amountPerUnitNQT = (totalNQT / resSupply) / 10^decimals
 * 对标 `calculatePricePerWholeQNT(convertToNQT(amount), decimals)`。
 */
async function handleSubmit(): Promise<void> {
  if (!formRef.value) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    loading.value = true
    try {
      const totalNqt = BigInt(Math.round(Number(form.amountNXT) * 1e8))
      const resSupplyWhole = BigInt(Math.round(Number(qntToQntf(currencyInfo.reserveSupply, currencyInfo.decimals))))
      if (resSupplyWhole === 0n) throw new Error(t('monetary.invalidReserveSupply'))
      const unitAmountNqt = totalNqt / resSupplyWhole
      const divisor = 10n ** BigInt(currencyInfo.decimals)
      const amountPerUnitNQT = (unitAmountNqt / divisor).toString()

      await submitForm('currencyReserveIncrease', {
        currency: currencyInfo.currency,
        amountPerUnitNQT,
        feeNXT: form.feeNXT,
        deadline: form.deadline,
        secretPhrase: form.secretPhrase,
      }, {
        successMessage: t('monetary.reserveSuccess'),
      })
      emit('success')
      handleClose()
    } catch (e: any) {
      ElMessage.error(e?.message || t('monetary.reserveError'))
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
