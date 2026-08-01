<template>
  <el-dialog
    v-model="visible"
    :title="t('currencyExchange.title', { code: currency?.code || '' })"
    width="960px"
    :close-on-click-modal="false"
    destroy-on-close
    class="currency-exchange-modal"
    @open="onOpen"
    @closed="onClosed"
  >
    <!-- 货币信息概览 -->
    <div class="currency-summary" v-if="currency">
      <el-descriptions :column="4" size="small" border>
        <el-descriptions-item :label="t('common.name')">
          {{ currency.name || '-' }}
        </el-descriptions-item>
        <el-descriptions-item :label="t('currencyExchange.decimals')">
          {{ currency.decimals || 0 }}
        </el-descriptions-item>
        <el-descriptions-item :label="t('currencyExchange.currentSupply')">
          {{ formatQNT(currency.currentSupplyQNT, currency.decimals) }}
        </el-descriptions-item>
        <el-descriptions-item :label="t('currencyExchange.yourBalance')">
          <span class="text-mono">{{ yourUnitsDisplay }}</span>
        </el-descriptions-item>
      </el-descriptions>
    </div>

    <el-tabs v-model="activeTab" class="exchange-tabs">
      <!-- Tab: 买入（NRC → 货币） -->
      <el-tab-pane name="buy">
        <template #label>
          <span class="tab-label text-success">
            <el-icon><Top /></el-icon>
            {{ t('currencyExchange.buy') }} (NRC → {{ currency?.code || '' }})
          </span>
        </template>

        <el-row :gutter="16">
          <!-- 买入表单 -->
          <el-col :span="10">
            <el-card shadow="never" class="order-form-card buy-form-card">
              <template #header>
                <span class="form-header text-success">{{ t('currencyExchange.buyCurrency') }}</span>
              </template>
              <div class="balance-hint">
                {{ t('currencyExchange.nrcBalance') }}:
                <span class="text-mono">{{ nrcBalanceDisplay }} NRC</span>
              </div>
              <el-form label-position="top" size="small" @submit.prevent>
                <el-form-item :label="t('currencyExchange.units')">
                  <el-input v-model="buyForm.units" type="number" @input="recalcBuy">
                    <template #append>{{ currency?.code || '' }}</template>
                  </el-input>
                </el-form-item>
                <el-form-item :label="t('currencyExchange.maxRate')">
                  <el-input v-model="buyForm.rate" type="number" @input="recalcBuy">
                    <template #append>{{ currency?.code }}/NRC</template>
                  </el-input>
                </el-form-item>
                <el-form-item :label="t('currencyExchange.total')">
                  <el-input :model-value="buyForm.total" readonly>
                    <template #append>NRC</template>
                  </el-input>
                </el-form-item>
                <el-button
                  type="success"
                  class="submit-btn"
                  :loading="submitting"
                  :disabled="!canSubmitBuy"
                  @click="submitBuy"
                >
                  {{ t('currencyExchange.buy') }}
                </el-button>
              </el-form>
            </el-card>
          </el-col>

          <!-- 卖单列表（买入时参考） -->
          <el-col :span="14">
            <el-card shadow="never" class="offers-card">
              <template #header>
                <span>{{ t('currencyExchange.sellOffers') }}</span>
                <el-tag size="small" type="info" class="offers-count">{{ sellOffers.length }}</el-tag>
              </template>
              <el-table
                :data="sellOffers"
                size="small"
                :empty-text="t('currencyExchange.noSellOffers')"
                @row-click="(row: any) => fillBuyFromOffer(row)"
                row-key="offer"
              >
                <el-table-column :label="t('currencyExchange.account')" min-width="140">
                  <template #default="{ row }">
                    <span class="text-mono">{{ truncateHash(row.accountRS || row.account, 10) }}</span>
                  </template>
                </el-table-column>
                <el-table-column :label="t('currencyExchange.units')" width="110" align="right">
                  <template #default="{ row }">
                    <span class="text-mono">{{ formatQNT(row.supply, currency?.decimals || 0) }}</span>
                  </template>
                </el-table-column>
                <el-table-column :label="t('currencyExchange.limit')" width="110" align="right">
                  <template #default="{ row }">
                    <span class="text-mono">{{ formatQNT(row.limit, currency?.decimals || 0) }}</span>
                  </template>
                </el-table-column>
                <el-table-column :label="t('currencyExchange.rate')" width="120" align="right">
                  <template #default="{ row }">
                    <span class="text-mono text-success">{{ formatRate(row.rateNQT) }}</span>
                  </template>
                </el-table-column>
              </el-table>
            </el-card>
          </el-col>
        </el-row>
      </el-tab-pane>

      <!-- Tab: 卖出（货币 → NRC） -->
      <el-tab-pane name="sell">
        <template #label>
          <span class="tab-label text-danger">
            <el-icon><Bottom /></el-icon>
            {{ t('currencyExchange.sell') }} ({{ currency?.code || '' }} → NRC)
          </span>
        </template>

        <el-row :gutter="16">
          <!-- 卖出表单 -->
          <el-col :span="10">
            <el-card shadow="never" class="order-form-card sell-form-card">
              <template #header>
                <span class="form-header text-danger">{{ t('currencyExchange.sellCurrency') }}</span>
              </template>
              <div class="balance-hint">
                {{ t('currencyExchange.currencyBalance') }}:
                <span class="text-mono">{{ yourUnitsDisplay }} {{ currency?.code || '' }}</span>
              </div>
              <el-form label-position="top" size="small" @submit.prevent>
                <el-form-item :label="t('currencyExchange.units')">
                  <el-input v-model="sellForm.units" type="number" @input="recalcSell">
                    <template #append>{{ currency?.code || '' }}</template>
                  </el-input>
                </el-form-item>
                <el-form-item :label="t('currencyExchange.minRate')">
                  <el-input v-model="sellForm.rate" type="number" @input="recalcSell">
                    <template #append>{{ currency?.code }}/NRC</template>
                  </el-input>
                </el-form-item>
                <el-form-item :label="t('currencyExchange.total')">
                  <el-input :model-value="sellForm.total" readonly>
                    <template #append>NRC</template>
                  </el-input>
                </el-form-item>
                <el-button
                  type="danger"
                  class="submit-btn"
                  :loading="submitting"
                  :disabled="!canSubmitSell"
                  @click="submitSell"
                >
                  {{ t('currencyExchange.sell') }}
                </el-button>
              </el-form>
            </el-card>
          </el-col>

          <!-- 买单列表（卖出时参考） -->
          <el-col :span="14">
            <el-card shadow="never" class="offers-card">
              <template #header>
                <span>{{ t('currencyExchange.buyOffers') }}</span>
                <el-tag size="small" type="info" class="offers-count">{{ buyOffers.length }}</el-tag>
              </template>
              <el-table
                :data="buyOffers"
                size="small"
                :empty-text="t('currencyExchange.noBuyOffers')"
                @row-click="(row: any) => fillSellFromOffer(row)"
                row-key="offer"
              >
                <el-table-column :label="t('currencyExchange.account')" min-width="140">
                  <template #default="{ row }">
                    <span class="text-mono">{{ truncateHash(row.accountRS || row.account, 10) }}</span>
                  </template>
                </el-table-column>
                <el-table-column :label="t('currencyExchange.units')" width="110" align="right">
                  <template #default="{ row }">
                    <span class="text-mono">{{ formatQNT(row.supply, currency?.decimals || 0) }}</span>
                  </template>
                </el-table-column>
                <el-table-column :label="t('currencyExchange.limit')" width="110" align="right">
                  <template #default="{ row }">
                    <span class="text-mono">{{ formatQNT(row.limit, currency?.decimals || 0) }}</span>
                  </template>
                </el-table-column>
                <el-table-column :label="t('currencyExchange.rate')" width="120" align="right">
                  <template #default="{ row }">
                    <span class="text-mono text-danger">{{ formatRate(row.rateNQT) }}</span>
                  </template>
                </el-table-column>
              </el-table>
            </el-card>
          </el-col>
        </el-row>
      </el-tab-pane>

      <!-- Tab: 发布兑换报价 -->
      <el-tab-pane name="publish">
        <template #label>
          <span class="tab-label">
            <el-icon><Promotion /></el-icon>
            {{ t('currencyExchange.publishOffer') }}
          </span>
        </template>

        <el-card shadow="never" class="publish-form-card">
          <el-alert
            type="info"
            :closable="false"
            show-icon
            class="publish-hint"
          >
            {{ t('currencyExchange.publishHint') }}
          </el-alert>
          <el-form label-position="top" @submit.prevent>
            <el-row :gutter="16">
              <el-col :span="12">
                <el-form-item :label="t('currencyExchange.buyRate')">
                  <el-input v-model="publishForm.buyRate" type="number">
                    <template #append>NRC/{{ currency?.code || '' }}</template>
                  </el-input>
                </el-form-item>
                <el-form-item :label="t('currencyExchange.totalBuyLimit')">
                  <el-input v-model="publishForm.totalBuyLimit" type="number">
                    <template #append>{{ currency?.code || '' }}</template>
                  </el-input>
                </el-form-item>
              </el-col>
              <el-col :span="12">
                <el-form-item :label="t('currencyExchange.sellRate')">
                  <el-input v-model="publishForm.sellRate" type="number">
                    <template #append>NRC/{{ currency?.code || '' }}</template>
                  </el-input>
                </el-form-item>
                <el-form-item :label="t('currencyExchange.totalSellLimit')">
                  <el-input v-model="publishForm.totalSellLimit" type="number">
                    <template #append>{{ currency?.code || '' }}</template>
                  </el-input>
                </el-form-item>
              </el-col>
            </el-row>
            <el-row :gutter="16">
              <el-col :span="12">
                <el-form-item :label="t('currencyExchange.expirationHeight')">
                  <BlockHeightPicker v-model="publishForm.expirationHeight" />
                </el-form-item>
              </el-col>
              <el-col :span="12">
                <el-form-item :label="t('common.fee')">
                  <el-input v-model="publishForm.feeNXT">
                    <template #append>NRC</template>
                  </el-input>
                </el-form-item>
              </el-col>
            </el-row>
            <el-button
              type="primary"
              :loading="submitting"
              :disabled="!canPublish"
              @click="submitPublish"
            >
              {{ t('currencyExchange.publishOffer') }}
            </el-button>
          </el-form>
        </el-card>
      </el-tab-pane>
    </el-tabs>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { Top, Bottom, Promotion } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { useAccountStore } from '@/stores/modules/account.store'
import { useNrcsForm } from '@/composables/useNrcsForm'
import {
  formatQuantity,
  formatOrderPricePerWholeQNT,
  qntfToQnt,
  nxtToNqt,
  nqtToNxt,
  truncateHash,
} from '@/utils/format'
import { BlockHeightPicker } from '@/components/modals/ui-elements'

/** 货币详情（对标 getCurrency 响应） */
interface CurrencyInfo {
  currency: string
  code: string
  name?: string
  decimals: number
  currentSupplyQNT?: string
  type?: number
}

const props = defineProps<{
  /** 货币详情 */
  currency: CurrencyInfo | null
}>()

const visible = defineModel<boolean>('visible', { default: false })
const emit = defineEmits<{ (e: 'success'): void }>()

const { t } = useI18n()
const accountStore = useAccountStore()
const { submitForm } = useNrcsForm()

/** 当前激活的 tab */
const activeTab = ref<'buy' | 'sell' | 'publish'>('buy')

/** 是否正在提交 */
const submitting = ref(false)

/** 卖单列表（买入时参考，按价格升序） */
const sellOffers = ref<any[]>([])

/** 买单列表（卖出时参考，按价格降序） */
const buyOffers = ref<any[]>([])

/** 当前账户持有的货币单位（QNT） */
const yourUnitsQNT = ref<string>('0')

/** 买入表单 */
const buyForm = reactive({
  units: '0',
  rate: '0',
  total: '0',
})

/** 卖出表单 */
const sellForm = reactive({
  units: '0',
  rate: '0',
  total: '0',
})

/** 发布兑换报价表单（对标 publishExchangeOffer） */
const publishForm = reactive({
  buyRate: '0',
  sellRate: '0',
  totalBuyLimit: '0',
  totalSellLimit: '0',
  expirationHeight: 0,
  feeNXT: '1',
})

/** NRC 余额展示 */
const nrcBalanceDisplay = computed(() =>
  nqtToNxt(accountStore.balanceNQT || '0')
)

/** 持有的货币单位展示 */
const yourUnitsDisplay = computed(() => {
  if (!props.currency) return '0'
  return formatQuantity(yourUnitsQNT.value, props.currency.decimals)
})

/** 是否可提交买单 */
const canSubmitBuy = computed(() => {
  return Number(buyForm.units) > 0 && Number(buyForm.rate) > 0 && !submitting.value
})

/** 是否可提交卖单 */
const canSubmitSell = computed(() => {
  return Number(sellForm.units) > 0 && Number(sellForm.rate) > 0 && !submitting.value
})

/** 是否可发布报价 */
const canPublish = computed(() => {
  return (
    (Number(publishForm.buyRate) > 0 || Number(publishForm.sellRate) > 0) &&
    publishForm.expirationHeight > 0 &&
    !submitting.value
  )
})

/**
 * 弹窗打开时加载数据。
 */
async function onOpen(): Promise<void> {
  await Promise.all([loadOffers(), loadAccountCurrency()])
}

/**
 * 弹窗关闭时重置。
 */
function onClosed(): void {
  activeTab.value = 'buy'
  sellOffers.value = []
  buyOffers.value = []
  yourUnitsQNT.value = '0'
  buyForm.units = '0'
  buyForm.rate = '0'
  buyForm.total = '0'
  sellForm.units = '0'
  sellForm.rate = '0'
  sellForm.total = '0'
  publishForm.buyRate = '0'
  publishForm.sellRate = '0'
  publishForm.totalBuyLimit = '0'
  publishForm.totalSellLimit = '0'
  publishForm.expirationHeight = 0
  publishForm.feeNXT = '1'
}

/**
 * 加载买卖单报价（对标 nrs.monetarysystem.js:359 loadCurrencyOffers）。
 * 合并真实报价与预期报价，按价格排序。
 */
async function loadOffers(): Promise<void> {
  if (!props.currency?.currency) return
  const currencyId = props.currency.currency
  const decimals = props.currency.decimals

  const [sellRes, expectedSellRes, buyRes, expectedBuyRes] = await Promise.allSettled([
    nrcsApi.getSellOffers(currencyId, 0, 100),
    nrcsApi.getExpectedSellOffers(currencyId),
    nrcsApi.getBuyOffers(currencyId, 0, 100),
    nrcsApi.getExpectedBuyOffers(currencyId),
  ])

  // 合并真实卖单 + 预期卖单，按 rateNQT 升序（对标 nrs.monetarysystem.js:394-403）
  const realSell = sellRes.status === 'fulfilled' ? (sellRes.value?.offers || []) : []
  const expectedSell = expectedSellRes.status === 'fulfilled' ? (expectedSellRes.value?.offers || []) : []
  sellOffers.value = [...realSell, ...expectedSell].sort((a, b) => {
    return Number(a.rateNQT) - Number(b.rateNQT)
  })

  // 合并真实买单 + 预期买单，按 rateNQT 降序
  const realBuy = buyRes.status === 'fulfilled' ? (buyRes.value?.offers || []) : []
  const expectedBuy = expectedBuyRes.status === 'fulfilled' ? (expectedBuyRes.value?.offers || []) : []
  buyOffers.value = [...realBuy, ...expectedBuy].sort((a, b) => {
    return Number(b.rateNQT) - Number(a.rateNQT)
  })

  // 自动填充最优价格（对标 nrs.monetarysystem.js:338-341）
  if (sellOffers.value.length > 0) {
    buyForm.rate = formatOrderPricePerWholeQNT(sellOffers.value[0].rateNQT, decimals)
    recalcBuy()
  }
  if (buyOffers.value.length > 0) {
    sellForm.rate = formatOrderPricePerWholeQNT(buyOffers.value[0].rateNQT, decimals)
    recalcSell()
  }
}

/**
 * 加载当前账户持有的货币余额（对标 getAccountCurrencies）。
 */
async function loadAccountCurrency(): Promise<void> {
  if (!props.currency?.currency || !accountStore.accountRS) return
  try {
    const res = await nrcsApi.getAccountCurrencies(accountStore.accountRS, 0, 100)
    const found = (res?.currencyBalances || []).find(
      (b: any) => b.currency === props.currency?.currency
    )
    yourUnitsQNT.value = found?.balanceQNT || '0'
  } catch {
    yourUnitsQNT.value = '0'
  }
}

/**
 * 格式化 QNT 为可读数量（对标 NRS.formatQuantity）。
 */
function formatQNT(qnt: string | undefined, decimals: number): string {
  if (!qnt) return '0'
  return formatQuantity(qnt, decimals)
}

/**
 * 格式化费率（对标 NRS.formatOrderPricePerWholeQNT）。
 */
function formatRate(rateNQT: string): string {
  if (!props.currency) return rateNQT
  return formatOrderPricePerWholeQNT(rateNQT, props.currency.decimals)
}

/**
 * 重新计算买入总额。
 */
function recalcBuy(): void {
  const units = Number(buyForm.units) || 0
  const rate = Number(buyForm.rate) || 0
  buyForm.total = (units * rate).toFixed(8)
}

/**
 * 重新计算卖出总额。
 */
function recalcSell(): void {
  const units = Number(sellForm.units) || 0
  const rate = Number(sellForm.rate) || 0
  sellForm.total = (units * rate).toFixed(8)
}

/**
 * 点击卖单报价填充买入表单（对标 nrs.monetarysystem.js:339 rate.val）。
 */
function fillBuyFromOffer(offer: any): void {
  if (!props.currency) return
  buyForm.rate = formatOrderPricePerWholeQNT(offer.rateNQT, props.currency.decimals)
  if (Number(buyForm.units) === 0) {
    buyForm.units = formatQuantity(offer.supply, props.currency.decimals)
  }
  recalcBuy()
}

/**
 * 点击买单报价填充卖出表单。
 */
function fillSellFromOffer(offer: any): void {
  if (!props.currency) return
  sellForm.rate = formatOrderPricePerWholeQNT(offer.rateNQT, props.currency.decimals)
  if (Number(sellForm.units) === 0) {
    sellForm.units = formatQuantity(offer.supply, props.currency.decimals)
  }
  recalcSell()
}

/**
 * 提交买单（对标 currencyBuy + 三步本地签名）。
 */
async function submitBuy(): Promise<void> {
  if (!props.currency) return
  submitting.value = true
  try {
    const decimals = props.currency.decimals
    const unitsQNT = qntfToQnt(buyForm.units, decimals)
    const rateNQT = nxtToNqt(buyForm.rate)
    await submitForm('currencyBuy', {
      secretPhrase: accountStore.secretPhrase,
      currency: props.currency.currency,
      unitsQNT,
      rateNQT,
      offerType: 'buy',
      feeNXT: '1',
      deadline: 1440,
    }, {
      successMessage: t('currencyExchange.buySuccess'),
      onSuccess: () => {
        emit('success')
        loadOffers()
        loadAccountCurrency()
      },
    })
  } catch (e: any) {
    ElMessage.error(e?.message || t('currencyExchange.orderError'))
  } finally {
    submitting.value = false
  }
}

/**
 * 提交卖单（对标 currencySell + 三步本地签名）。
 */
async function submitSell(): Promise<void> {
  if (!props.currency) return
  submitting.value = true
  try {
    const decimals = props.currency.decimals
    const unitsQNT = qntfToQnt(sellForm.units, decimals)
    const rateNQT = nxtToNqt(sellForm.rate)
    await submitForm('currencySell', {
      secretPhrase: accountStore.secretPhrase,
      currency: props.currency.currency,
      unitsQNT,
      rateNQT,
      offerType: 'sell',
      feeNXT: '1',
      deadline: 1440,
    }, {
      successMessage: t('currencyExchange.sellSuccess'),
      onSuccess: () => {
        emit('success')
        loadOffers()
        loadAccountCurrency()
      },
    })
  } catch (e: any) {
    ElMessage.error(e?.message || t('currencyExchange.orderError'))
  } finally {
    submitting.value = false
  }
}

/**
 * 发布兑换报价（对标 publishExchangeOffer + 三步本地签名）。
 */
async function submitPublish(): Promise<void> {
  if (!props.currency) return
  submitting.value = true
  try {
    const decimals = props.currency.decimals
    const buyRateNQT = nxtToNqt(publishForm.buyRate)
    const sellRateNQT = nxtToNqt(publishForm.sellRate)
    const totalBuyLimitQNT = qntfToQnt(publishForm.totalBuyLimit, decimals)
    const totalSellLimitQNT = qntfToQnt(publishForm.totalSellLimit, decimals)
    await submitForm('publishExchangeOffer', {
      secretPhrase: accountStore.secretPhrase,
      currency: props.currency.currency,
      buyRateNQT,
      sellRateNQT,
      totalBuyLimit: totalBuyLimitQNT,
      totalSellLimit: totalSellLimitQNT,
      expirationHeight: publishForm.expirationHeight,
      feeNXT: publishForm.feeNXT,
      deadline: 1440,
    }, {
      successMessage: t('currencyExchange.publishSuccess'),
      onSuccess: () => {
        emit('success')
        loadOffers()
      },
    })
  } catch (e: any) {
    ElMessage.error(e?.message || t('currencyExchange.publishError'))
  } finally {
    submitting.value = false
  }
}
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;

.currency-exchange-modal {
  .currency-summary {
    margin-bottom: 12px;
  }

  .exchange-tabs {
    .tab-label {
      display: inline-flex;
      align-items: center;
      gap: 4px;
    }
  }

  .order-form-card,
  .offers-card,
  .publish-form-card {
    :deep(.el-card__header) {
      padding: 8px 12px;
    }

    .form-header {
      font-weight: 600;
    }

    .offers-count {
      margin-left: 8px;
    }
  }

  .order-form-card {
    .balance-hint {
      font-size: 12px;
      color: $text-muted;
      margin-bottom: 12px;
      padding: 6px 10px;
      background: $bg-hover;
      border-radius: $radius-sm;
    }

    .submit-btn {
      width: 100%;
    }
  }

  .publish-hint {
    margin-bottom: 12px;
  }

  .text-success { color: $success; }
  .text-danger { color: $danger; }
  .text-mono { font-family: $font-mono; }
}
</style>
