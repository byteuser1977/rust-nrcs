<template>
  <span class="info-value">
    <!-- 字符串 / 数字 -->
    <span v-if="isPrimitive" class="info-value-text">{{ String(value) }}</span>

    <!-- HTML 字符串（已转义，对标 _formatted_html 后缀字段） -->
    <span v-else-if="isHtml" class="info-value-html" v-html="htmlValue"></span>

    <!-- 单个链接（LinkRef） -->
    <TransactionLink v-else-if="isLink" :link="value as LinkRef" />

    <!-- 链接数组 -->
    <template v-else-if="isLinkArray">
      <span v-for="(link, idx) in linkArrayValue" :key="idx" class="info-value-list-item">
        <TransactionLink :link="link" />
        <span v-if="idx < linkArrayValue.length - 1" class="info-value-separator">, </span>
      </span>
    </template>

    <!-- 数量二元组 [quantityQNT, decimals] -->
    <span v-else-if="isQuantity" class="info-value-quantity">
      {{ formatQuantityDisplay(quantityValue.quantity, quantityValue.decimals) }}
    </span>

    <!-- 金额二元组 [amountNQT, decimals] -->
    <span v-else-if="isAmount" class="info-value-amount">
      {{ formatAmountDisplay(amountValue.amount) }}
    </span>

    <!-- 兜底：JSON 字符串 -->
    <span v-else class="info-value-fallback">{{ JSON.stringify(value) }}</span>
  </span>
</template>

<script setup lang="ts">
/**
 * InfoValue 组件 —— 渲染交易详情信息表中的单个值。
 *
 * 支持的值类型（对标 InfoValue 联合类型）：
 *   - string / number → 纯文本
 *   - { html: string } → HTML 字符串（已转义）
 *   - LinkRef → 可点击链接（TransactionLink 子组件）
 *   - LinkRef[] → 链接列表
 *   - { quantity, decimals } → 资产数量（按精度格式化）
 *   - { amount, decimals? } → 金额（NQT → NRC）
 */
import { computed } from 'vue'
import type { InfoValue, LinkRef } from '@/utils/transaction-links'
import { formatQuantity, formatAmount } from '@/utils/format'
import TransactionLink from './TransactionLink.vue'

const props = defineProps<{ value: InfoValue }>()

// 类型守卫（computed）
const isPrimitive = computed(() => typeof props.value === 'string' || typeof props.value === 'number')
const isHtml = computed(() =>
  props.value !== null &&
  typeof props.value === 'object' &&
  !Array.isArray(props.value) &&
  typeof (props.value as { html?: unknown }).html === 'string',
)
const isLink = computed(() => isLinkRef(props.value))
const isLinkArray = computed(() =>
  Array.isArray(props.value) && props.value.length > 0 && props.value.every(isLinkRef),
)
const isQuantity = computed(() =>
  props.value !== null &&
  typeof props.value === 'object' &&
  !Array.isArray(props.value) &&
  typeof (props.value as { quantity?: unknown }).quantity === 'string' &&
  typeof (props.value as { decimals?: unknown }).decimals === 'number' &&
  !('amount' in (props.value as object)),
)
const isAmount = computed(() =>
  props.value !== null &&
  typeof props.value === 'object' &&
  !Array.isArray(props.value) &&
  typeof (props.value as { amount?: unknown }).amount === 'string',
)

/** 安全提取的 HTML 字符串 */
const htmlValue = computed(() => (props.value as { html?: string }).html ?? '')

/** 安全提取的链接数组 */
const linkArrayValue = computed(() => (Array.isArray(props.value) ? (props.value as LinkRef[]) : []))

/** 安全提取的数量对象 */
const quantityValue = computed(() => {
  const v = props.value as { quantity?: string; decimals?: number }
  return {
    quantity: v.quantity ?? '0',
    decimals: v.decimals ?? 0,
  }
})

/** 安全提取的金额对象 */
const amountValue = computed(() => {
  const v = props.value as { amount?: string }
  return { amount: v.amount ?? '0' }
})

/**
 * 类型守卫：判断对象是否为 LinkRef。
 */
function isLinkRef(obj: unknown): boolean {
  return (
    obj !== null &&
    typeof obj === 'object' &&
    !Array.isArray(obj) &&
    typeof (obj as LinkRef).type === 'string' &&
    typeof (obj as LinkRef).value === 'string' &&
    !('html' in (obj as object)) &&
    !('quantity' in (obj as object)) &&
    !('amount' in (obj as object))
  )
}

/**
 * 格式化资产数量展示（对标 NRS.formatQuantity）。
 */
function formatQuantityDisplay(quantity: string, decimals: number): string {
  return formatQuantity(quantity, decimals)
}

/**
 * 格式化金额展示（对标 NRS.formatAmount，NQT → NRC）。
 */
function formatAmountDisplay(amountNQT: string): string {
  return `${formatAmount(amountNQT)} NRC`
}
</script>

<style scoped lang="scss">
.info-value {
  display: inline;
  word-break: break-all;
}
.info-value-text {
  color: inherit;
}
.info-value-html {
  display: inline;
}
.info-value-separator {
  color: var(--el-text-color-secondary);
  margin: 0 2px;
}
.info-value-quantity,
.info-value-amount {
  font-variant-numeric: tabular-nums;
}
</style>
