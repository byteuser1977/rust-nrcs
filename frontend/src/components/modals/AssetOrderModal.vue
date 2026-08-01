<template>
  <el-dialog
    v-model="visible"
    :title="orderType === 'buy' ? t('asset.buyAsset') : t('asset.sellAsset')"
    width="520px"
    :close-on-click-modal="false"
    destroy-on-close
    class="asset-order-modal"
    @close="handleClose"
  >
    <!-- 订单摘要 -->
    <div class="order-summary">
      <div class="summary-row">
        <span class="label">{{ t('asset.assetName') }}</span>
        <span class="value">{{ asset?.name }}</span>
      </div>
      <div class="summary-row">
        <span class="label">{{ t('asset.orderType') }}</span>
        <span class="value" :class="orderType === 'buy' ? 'text-success' : 'text-danger'">
          {{ orderType === 'buy' ? t('asset.buy') : t('asset.sell') }}
        </span>
      </div>
      <div class="summary-row" v-if="asset">
        <span class="label">{{ t('asset.availableBalance') }}</span>
        <span class="value text-mono">
          <template v-if="orderType === 'sell'">
            {{ formatQNT(yourAssetBalanceQNT, asset.decimals) }}
          </template>
          <template v-else>
            {{ accountBalanceNXT }} NRC
          </template>
        </span>
      </div>
    </div>

    <el-divider />

    <el-form ref="formRef" :model="form" :rules="rules" label-position="top" class="order-form">
      <el-form-item :label="t('asset.quantity')" prop="quantity">
        <el-input v-model="form.quantity" type="number" :placeholder="t('asset.quantityPlaceholder')" clearable>
          <template #append>{{ t('asset.shares') }}</template>
        </el-input>
      </el-form-item>

      <el-form-item :label="t('asset.pricePerShare')" prop="pricePerShare">
        <el-input v-model="form.pricePerShare" type="number" :placeholder="t('asset.pricePlaceholder')" clearable>
          <template #append>NRC</template>
        </el-input>
      </el-form-item>

      <el-form-item>
        <div class="total-display">
          <span class="label">{{ t('asset.total') }}</span>
          <span class="value text-mono" :class="{ 'text-danger': isTotalExceedsBalance }">
            {{ totalAmount }} NRC
          </span>
        </div>
      </el-form-item>

      <el-form-item :label="t('asset.fee')" prop="feeNQT">
        <el-input v-model="form.feeNQT" placeholder="1" type="number" clearable>
          <template #append>NRC</template>
        </el-input>
      </el-form-item>

      <el-form-item :label="t('asset.deadline')" prop="deadline">
        <el-input v-model="form.deadline" placeholder="1440" type="number" clearable>
          <template #append>{{ t('asset.minutes') }}</template>
        </el-input>
      </el-form-item>

      <el-form-item v-if="needsSecretPhrase" :label="t('asset.secretPhrase')" prop="secretPhrase">
        <el-input v-model="form.secretPhrase" type="password" :placeholder="t('asset.secretPhrasePlaceholder')" show-password clearable>
          <template #prefix>
            <el-icon><Lock /></el-icon>
          </template>
        </el-input>
      </el-form-item>
    </el-form>

    <template #footer>
      <div class="dialog-footer">
        <el-button @click="handleClose">{{ t('common.cancel') }}</el-button>
        <el-button
          :type="orderType === 'buy' ? 'success' : 'danger'"
          :loading="isSubmitting"
          :disabled="isTotalExceedsBalance"
          @click="handleSubmit"
        >
          {{ orderType === 'buy' ? t('asset.confirmBuy') : t('asset.confirmSell') }}
        </el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * AssetOrderModal 组件 —— 资产买卖下单弹窗。
 *
 * 对标 nrs.assetexchange.js 中的 placeBidOrder / placeAskOrder 流程。
 *
 * 安全模型：secretPhrase 不随请求外发，通过 useNrcsForm 三步本地签名流程提交：
 *   1. 发送 doNotSign 请求获取 unsignedTransactionBytes
 *   2. 本地验证 + 签名（verifyAndSignTransactionBytes）
 *   3. 广播已签名交易（broadcastTransactionBytes）
 *
 * 价格换算（对标 NRS.convertToNQT + NRS.convertToQNT）：
 *   - quantityQNT = qntfToQnt(quantity, decimals)  // 用户输入的小数 → base units
 *   - priceNQTPerShare = pricePerShare × 10^(8-decimals)  // NXT/整股 → NQT/QNT
 */
import { ref, reactive, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, type FormInstance, type FormRules } from 'element-plus'
import { Lock } from '@element-plus/icons-vue'
import { useAccountStore } from '@/stores/modules/account.store'
import { useNrcsForm } from '@/composables/useNrcsForm'
import { qntToQntf, qntfToQnt, nxtToNqt, calculateOrderTotalNQT } from '@/utils/format'
import type { NrcsAsset } from '@/api/modules/nrcs.api'

const { t } = useI18n()
const accountStore = useAccountStore()
const { submitForm } = useNrcsForm()

const props = defineProps<{
  /** 资产信息 */
  asset: NrcsAsset | null
  /** 订单类型：buy=买单（placeBidOrder），sell=卖单（placeAskOrder） */
  orderType: 'buy' | 'sell'
  /** 用户持有的该资产数量 QNT（卖单时用于余额校验，可选） */
  yourAssetBalanceQNT?: string
  /** 预填值（点击订单簿行时填充，可选） */
  prefill?: { quantity?: string; pricePerShare?: string } | null
}>()

const visible = defineModel<boolean>('visible', { default: false })
const emit = defineEmits<{ (e: 'success'): void }>()

const formRef = ref<FormInstance>()
const isSubmitting = ref(false)

const form = reactive({
  quantity: '',
  pricePerShare: '',
  feeNQT: '1',
  deadline: '1440',
  secretPhrase: '',
})

/** 账户余额（NXT），买单时用于余额校验 */
const accountBalanceNXT = computed(() => {
  const nqt = accountStore.balanceNQT || '0'
  return (Number(nqt) / 1e8).toFixed(2)
})

/**
 * 是否需要显示 secretPhrase 输入框。
 *
 * 当 accountStore 已有 secretPhrase（password 登录模式）时隐藏输入框，
 * 直接使用内存中的值；否则显示输入框让用户手动输入。
 */
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

/** 订单总金额（NXT） */
const totalAmount = computed(() => {
  const qty = Number(form.quantity) || 0
  const price = Number(form.pricePerShare) || 0
  if (qty <= 0 || price <= 0) return '0.00000000'
  return (qty * price).toFixed(8)
})

/** 总金额对应的 NQT（用于余额校验） */
const totalNQT = computed(() => {
  try {
    const qty = Number(form.quantity) || 0
    const price = Number(form.pricePerShare) || 0
    if (qty <= 0 || price <= 0) return '0'
    // 总 NXT → NQT
    return nxtToNqt((qty * price).toFixed(8))
  } catch {
    return '0'
  }
})

/** 买单时检查总金额是否超过账户余额 */
const isTotalExceedsBalance = computed(() => {
  if (props.orderType !== 'buy') return false
  const balanceNQT = BigInt(accountStore.balanceNQT || '0')
  try {
    return BigInt(totalNQT.value) > balanceNQT
  } catch {
    return false
  }
})

const rules = computed<FormRules>(() => ({
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
  pricePerShare: [
    { required: true, message: t('asset.priceRequired'), trigger: 'blur' },
    {
      validator: (_rule: any, value: string, callback: (err?: Error) => void) => {
        const num = Number(value)
        if (isNaN(num) || num <= 0) {
          callback(new Error(t('asset.priceInvalid')))
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
    ? [{ required: true, message: t('asset.secretPhraseRequired'), trigger: 'blur' }]
    : [],
}))

/**
 * 提交订单（通过 useNrcsForm 三步本地签名流程）。
 *
 * 价格/数量转换：
 *   - quantityQNT = qntfToQnt(form.quantity, decimals)
 *   - priceNQTPerShare = form.pricePerShare × 10^(8-decimals)
 */
const handleSubmit = async () => {
  if (!formRef.value || !props.asset) return
  // 捕获到局部常量，避免在下方闭包中 TS 重新放宽 nullable 收窄
  const asset = props.asset
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    if (isTotalExceedsBalance.value) {
      ElMessage.error(t('asset.insufficientBalance'))
      return
    }
    isSubmitting.value = true
    try {
      const decimals = asset.decimals
      // 获取 secretPhrase：优先 accountStore 内存值，否则用用户输入
      const secretPhrase = accountStore.hasSecretPhrase
        ? accountStore.secretPhrase
        : form.secretPhrase

      if (!secretPhrase) {
        ElMessage.warning(t('asset.secretPhraseRequired'))
        return
      }

      // 数量 QNTf → QNT（base units）
      const quantityQNT = qntfToQnt(form.quantity, decimals)
      // 价格 NXT/整股 → NQT/QNT
      // priceNQTPerShare = priceNXT × 10^(8-decimals)
      const priceNXT = form.pricePerShare
      const priceNQT = nxtToNqt(priceNXT)
      const priceNQTPerShare = String(BigInt(priceNQT) / BigInt(Math.pow(10, decimals)))

      const requestType = props.orderType === 'buy' ? 'placeBidOrder' : 'placeAskOrder'
      const data: Record<string, any> = {
        secretPhrase,
        asset: asset.asset,
        quantityQNT,
        priceNQTPerShare,
        feeNXT: form.feeNQT,
        deadline: form.deadline,
      }

      await submitForm(requestType, data, {
        successMessage: props.orderType === 'buy' ? t('asset.buySuccess') : t('asset.sellSuccess'),
      })

      emit('success')
      handleClose()
    } catch (err: any) {
      // useNrcsForm 已通过 ElMessage 显示错误，此处仅记录日志
      console.error('Place asset order failed:', err)
    } finally {
      isSubmitting.value = false
    }
  })
}

const handleClose = () => {
  formRef.value?.resetFields()
  Object.assign(form, {
    quantity: '',
    pricePerShare: '',
    feeNQT: '1',
    deadline: '1440',
    secretPhrase: '',
  })
  visible.value = false
}

/**
 * 监听 visible 打开：
 *   - 若为卖单且未传入 yourAssetBalanceQNT，提示用户
 *   - 若传入 prefill（点击订单簿行触发），自动填充数量与价格
 */
watch(visible, (open) => {
  if (open) {
    // 应用预填值（对标 nrs.assetexchange.js:943-945 行点击填充）
    if (props.prefill) {
      form.quantity = props.prefill.quantity || ''
      form.pricePerShare = props.prefill.pricePerShare || ''
    }
    if (props.orderType === 'sell' && !props.yourAssetBalanceQNT) {
      // 卖单但无持仓信息，仍允许提交（由服务端校验）
    }
  }
})
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;

.order-summary {
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid $border-subtle;
  border-radius: $radius-md;
  padding: $space-lg;

  .summary-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: $space-xs 0;

    .label {
      color: $text-muted;
      font-size: $font-size-sm;
    }

    .value {
      color: $text-primary;
      font-weight: 500;
    }
  }
}

.text-success { color: $success; }
.text-danger { color: $danger; }
.text-mono { font-family: $font-mono; }

.total-display {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
  padding: $space-sm $space-md;
  background: var(--el-fill-color-light);
  border-radius: $radius-sm;

  .label {
    color: $text-muted;
    font-size: $font-size-sm;
  }

  .value {
    font-size: $font-size-lg;
    font-weight: 700;
    color: $primary;
  }
}

.order-form {
  margin-top: $space-md;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: $space-sm;
}
</style>
