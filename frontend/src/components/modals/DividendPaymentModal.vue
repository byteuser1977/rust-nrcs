<template>
  <el-dialog
    v-model="visible"
    :title="t('dividendPayment.title')"
    width="520px"
    :close-on-click-modal="false"
    destroy-on-close
    class="dividend-payment-modal"
    @close="handleClose"
  >
    <el-form ref="formRef" :model="form" :rules="rules" label-width="160px" label-position="top">
      <!-- 资产信息（只读） -->
      <el-form-item :label="t('dividendPayment.asset')">
        <el-input :model-value="assetInfo?.name ?? assetId" readonly>
          <template #append>
            <span class="asset-decimals">{{ t('dividendPayment.decimals') }}: {{ assetInfo?.decimals ?? 0 }}</span>
          </template>
        </el-input>
      </el-form-item>

      <!-- 每股分红金额 -->
      <el-form-item :label="t('dividendPayment.amountPerShare')" prop="amountNXTPerShare">
        <el-input
          v-model="form.amountNXTPerShare"
          :placeholder="t('dividendPayment.amountPerSharePlaceholder')"
          type="number"
          clearable
          @blur="previewDividend"
        >
          <template #append>NRC</template>
        </el-input>
      </el-form-item>

      <!-- 持有人高度 -->
      <el-form-item :label="t('dividendPayment.height')" prop="height">
        <el-input
          v-model="form.height"
          :placeholder="t('dividendPayment.heightPlaceholder')"
          type="number"
          clearable
          @blur="previewDividend"
        />
      </el-form-item>

      <!-- 手续费 -->
      <el-form-item :label="t('dividendPayment.fee')" prop="feeNQT">
        <el-input v-model="form.feeNQT" placeholder="1" type="number" clearable>
          <template #append>NRC</template>
        </el-input>
      </el-form-item>

      <!-- 截止时间 -->
      <el-form-item :label="t('dividendPayment.deadline')" prop="deadline">
        <el-input v-model="form.deadline" :placeholder="t('dividendPayment.deadlinePlaceholder')" type="number" clearable>
          <template #append>{{ t('common.minutes') }}</template>
        </el-input>
      </el-form-item>

      <!-- 密码短语 -->
      <el-form-item :label="t('dividendPayment.secretPhrase')" prop="secretPhrase">
        <el-input v-model="form.secretPhrase" type="password" :placeholder="t('dividendPayment.secretPhrasePlaceholder')" show-password clearable>
          <template #prefix>
            <el-icon><Lock /></el-icon>
          </template>
        </el-input>
      </el-form-item>

      <!-- 分红预览（对标 nrs.modals.dividendpayment.js:70-114） -->
      <el-alert
        v-if="preview.message"
        :title="preview.message"
        :type="preview.type"
        :closable="false"
        show-icon
        class="dividend-payment-preview"
      />
    </el-form>

    <template #footer>
      <div class="dialog-footer">
        <el-button @click="handleClose">{{ t('common.cancel') }}</el-button>
        <el-button type="primary" :loading="isSubmitting" @click="handleSubmit">
          {{ t('dividendPayment.submit') }}
        </el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * DividendPaymentModal 组件 —— 资产分红支付弹窗。
 *
 * 对标 nrs.modals.dividendpayment.js（117 行）的完整实现。
 *
 * 主要功能：
 *   1. 输入每股分红金额 + 持有人高度
 *   2. 实时预览：调 getAssetAccounts 拉取持有人列表，计算总金额和接收人数
 *   3. 提交时将 amountNXTPerShare 转换为 amountNQTPerQNT（按资产精度）
 *   4. 校验分红高度不早于资产发行高度
 *
 * 关键转换（对标 nrs.modals.dividendpayment.js:31-34）：
 *   amountNQTPerQNT = calculatePricePerWholeQNT(
 *     convertToNQT(amountNXTPerShare),
 *     asset.decimals
 *   )
 *
 * 对标参考：NRS.forms.dividendPayment 函数
 */
import { ref, reactive, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, type FormInstance, type FormRules } from 'element-plus'
import { Lock } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import type { NrcsAsset } from '@/api/modules/nrcs.api'

const props = defineProps<{
  /** 资产 ID */
  assetId: string
  /** 资产信息（可选，避免重复 API 调用） */
  assetInfo?: NrcsAsset | null
}>()

const emit = defineEmits<{ (e: 'success'): void }>()

const { t } = useI18n()

const visible = defineModel<boolean>('visible', { default: false })

const formRef = ref<FormInstance>()
const isSubmitting = ref(false)
const localAssetInfo = ref<NrcsAsset | null>(null)

const form = reactive({
  amountNXTPerShare: '',
  height: '',
  feeNQT: '1',
  deadline: '1440',
  secretPhrase: '',
})

/** 分红预览（对标 nrs.modals.dividendpayment.js:70-114） */
const preview = reactive<{
  message: string
  type: 'info' | 'warning' | 'error' | 'success'
}>({
  message: '',
  type: 'info',
})

/** 资产精度 */
const decimals = computed(() => localAssetInfo.value?.decimals ?? props.assetInfo?.decimals ?? 0)

const rules = computed<FormRules>(() => ({
  amountNXTPerShare: [
    { required: true, message: t('dividendPayment.amountPerShareRequired'), trigger: 'blur' },
  ],
  height: [
    { required: true, message: t('dividendPayment.heightRequired'), trigger: 'blur' },
    {
      validator: (_rule: any, value: string, callback: (err?: Error) => void) => {
        if (!/^\d+$/.test(value)) {
          callback(new Error(t('dividendPayment.heightInvalid')))
        } else {
          callback()
        }
      },
      trigger: 'blur',
    },
  ],
  feeNQT: [{ required: true, message: t('dividendPayment.feeRequired'), trigger: 'blur' }],
  deadline: [{ required: true, message: t('dividendPayment.deadlineRequired'), trigger: 'blur' }],
  secretPhrase: [{ required: true, message: t('dividendPayment.secretPhraseRequired'), trigger: 'blur' }],
}))

// ----------------------------------------------------------------
// 数据加载
// ----------------------------------------------------------------

/**
 * 加载资产信息（若未通过 props 传入）。
 */
async function loadAssetInfo(): Promise<void> {
  if (props.assetInfo) {
    localAssetInfo.value = props.assetInfo
    return
  }
  try {
    const resp = await nrcsApi.getAsset(props.assetId)
    if (resp && !(resp as any).errorCode) {
      localAssetInfo.value = resp
    }
  } catch {
    // 忽略
  }
}

// ----------------------------------------------------------------
// 分红预览（对标 nrs.modals.dividendpayment.js:70-114）
// ----------------------------------------------------------------

/**
 * 预览分红支付信息。
 *
 * 调用 getAssetAccounts 拉取持有人列表，过滤掉发行方和创世账户后：
 *   - 计算合格持有人数量
 *   - 计算总份额
 *   - 计算总分红金额 = totalQuantityQNT × amountNQTPerQNT / 10^decimals
 *
 * 失败时显示错误信息（对标 :102-111）。
 */
async function previewDividend(): Promise<void> {
  if (!form.amountNXTPerShare || !/^\d+$/.test(form.height)) {
    preview.message = ''
    return
  }
  if (!localAssetInfo.value) {
    preview.message = ''
    return
  }

  try {
    const resp = await nrcsApi.getAssetAccounts(props.assetId, Number(form.height))
    if (!resp || (resp as any).errorCode) {
      const errorCode = (resp as any)?.errorCode
      if (errorCode === 4 || errorCode === 8) {
        preview.message = t('dividendPayment.invalidHeight')
      } else {
        preview.message = t('dividendPayment.previewError', { errorCode: String(errorCode ?? '') })
      }
      preview.type = 'warning'
      return
    }

    const accountAssets = resp.accountAssets ?? []
    // 过滤掉发行方和创世账户（对标 :80-84）
    const issuerRS = localAssetInfo.value.accountRS
    const GENESIS_RS = 'NRCS-7CPH-5A6T-YSVY-9WCAU' // NRCS 创世账户 RS
    const qualified = accountAssets.filter(
      (a: any) => a.accountRS !== issuerRS && a.accountRS !== GENESIS_RS,
    )

    // 计算总份额
    let totalQuantityQNT = BigInt(0)
    for (const a of qualified) {
      totalQuantityQNT += BigInt(a.quantityQNT)
    }

    // 计算总分红金额（对标 :91-92）
    const amountNQTPerQNT = calculateAmountNQTPerQNT(form.amountNXTPerShare, decimals.value)
    const totalNQT = calculateOrderTotal(totalQuantityQNT, amountNQTPerQNT)
    const totalNRC = Number(totalNQT) / 1e8

    preview.message = t('dividendPayment.previewSuccess', {
      amountNXT: totalNRC.toFixed(2),
      totalQuantity: formatQuantity(totalQuantityQNT.toString(), decimals.value),
      recipientCount: String(qualified.length),
    })
    preview.type = 'info'
  } catch (e) {
    preview.message = e instanceof Error ? e.message : String(e)
    preview.type = 'warning'
  }
}

// ----------------------------------------------------------------
// 提交处理（对标 NRS.forms.dividendPayment）
// ----------------------------------------------------------------

/**
 * 提交分红支付。
 *
 * 步骤：
 *   1. 校验表单
 *   2. 转换 amountNXTPerShare → amountNQTPerQNT
 *   3. 调 dividendPayment API
 *   4. 成功后关闭 modal 并触发 success 事件
 */
async function handleSubmit(): Promise<void> {
  if (!formRef.value) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    if (!localAssetInfo.value) {
      ElMessage.error(t('dividendPayment.assetInfoMissing'))
      return
    }

    isSubmitting.value = true
    try {
      const feeNQT = String(Math.round(Number(form.feeNQT) * 1e8))
      const amountNQTPerQNT = calculateAmountNQTPerQNT(form.amountNXTPerShare, decimals.value)

      await nrcsApi.dividendPayment({
        secretPhrase: form.secretPhrase,
        asset: props.assetId,
        height: Number(form.height),
        amountNQTPerShare: amountNQTPerQNT,
        feeNQT,
        deadline: Number(form.deadline),
      })

      ElMessage.success(t('success.dividendPayment'))
      emit('success')
      handleClose()
    } catch (err: any) {
      ElMessage.error(err?.message || t('dividendPayment.error'))
    } finally {
      isSubmitting.value = false
    }
  })
}

// ----------------------------------------------------------------
// 辅助函数
// ----------------------------------------------------------------

/**
 * 将每股 NRC 金额转换为 amountNQTPerQNT（对标 NRS.calculatePricePerWholeQNT）。
 *
 * 算法：amountNQTPerQNT = amountNQT / 10^decimals
 *   - amountNQT = amountNXTPerShare × 10^8
 *   - 除以 10^decimals 是因为 QNT 是资产的最小单位，whole QNT = QNT × 10^decimals
 *
 * @param amountNXTPerShare 每股 NRC 金额
 * @param assetDecimals 资产精度
 * @returns amountNQTPerQNT 字符串
 */
function calculateAmountNQTPerQNT(amountNXTPerShare: string, assetDecimals: number): string {
  // amountNQT = NRC × 10^8
  const amountNQT = BigInt(Math.round(Number(amountNXTPerShare) * 1e8))
  // amountNQTPerQNT = amountNQT / 10^decimals
  const divisor = BigInt(10) ** BigInt(assetDecimals)
  return (amountNQT / divisor).toString()
}

/**
 * 计算订单总额（对标 NRS.calculateOrderTotal）。
 *
 * totalNQT = quantityQNT × priceNQTPerQNT
 *
 * @param quantityQNT 数量（BigInt）
 * @param priceNQTPerQNT 单价 NQT/QNT（字符串）
 * @returns 总额 NQT（BigInt）
 */
function calculateOrderTotal(quantityQNT: bigint, priceNQTPerQNT: string): bigint {
  return quantityQNT * BigInt(priceNQTPerQNT)
}

/**
 * 格式化资产数量展示（对标 NRS.formatQuantity）。
 *
 * @param quantityQNT 数量 QNT
 * @param decimals 精度
 * @returns 格式化后的字符串
 */
function formatQuantity(quantityQNT: string, decimals: number): string {
  const q = BigInt(quantityQNT)
  const divisor = BigInt(10) ** BigInt(decimals)
  const whole = q / divisor
  const fraction = q % divisor
  if (decimals === 0) return whole.toString()
  const fractionStr = fraction.toString().padStart(decimals, '0').replace(/0+$/, '')
  return fractionStr ? `${whole}.${fractionStr}` : whole.toString()
}

// ----------------------------------------------------------------
// 关闭/重置
// ----------------------------------------------------------------

/**
 * 关闭 modal 并重置表单。
 */
function handleClose(): void {
  formRef.value?.resetFields()
  Object.assign(form, {
    amountNXTPerShare: '',
    height: '',
    feeNQT: '1',
    deadline: '1440',
    secretPhrase: '',
  })
  preview.message = ''
  preview.type = 'info'
  visible.value = false
}

// ----------------------------------------------------------------
// 监听 visible 变化加载资产信息
// ----------------------------------------------------------------

watch(
  () => visible.value,
  (isVisible) => {
    if (isVisible && props.assetId) {
      loadAssetInfo()
    }
  },
  { immediate: true },
)
</script>

<style scoped lang="scss">
.asset-decimals {
  color: var(--el-text-color-secondary);
  font-size: 12px;
}
.dividend-payment-preview {
  margin-top: 12px;
}
.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
</style>
