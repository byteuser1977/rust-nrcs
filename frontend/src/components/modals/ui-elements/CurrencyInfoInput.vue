<template>
  <!--
    CurrencyInfoInput —— 货币代码输入 + 自动查询货币 ID/精度 UI 元素。
    对标 nrs.modals.uielements.js:84-121 的 add_currency_modal_ui_element。
  -->
  <div class="currency-info-input" data-modal-ui-element="add_currency_modal_ui_element">
    <el-input
      v-model="currencyCodeValue"
      :placeholder="t('uiElements.currencyCodePlaceholder')"
      :disabled="disabled"
      clearable
      class="acm_ue_currency_code_input"
      @input="onInput"
    >
      <template #prefix><el-icon><Money /></el-icon></template>
    </el-input>

    <!-- 货币 ID 展示（.acm_ue_currency_id） -->
    <div v-if="currencyCodeValue" class="acm_ue_currency_id currency-info-display">
      <el-icon v-if="loading" class="is-loading"><Loading /></el-icon>
      <span v-else-if="currencyId" class="currency-info-id">
        {{ currencyId }}
        <span class="currency-info-decimals">
          ({{ t('uiElements.decimals') }}: {{ currencyDecimals }})
        </span>
      </span>
      <span v-else class="currency-info-not-existing">{{ t('uiElements.notExisting') }}</span>
    </div>

    <!-- 货币 ID 隐藏字段（.acm_ue_currency_id_input） -->
    <input
      v-if="currencyId"
      type="hidden"
      class="acm_ue_currency_id_input"
      :value="currencyId"
    />
    <!-- 货币精度隐藏字段（.acm_ue_currency_decimals_input） -->
    <input
      v-if="currencyDecimals !== null"
      type="hidden"
      class="acm_ue_currency_decimals_input"
      :value="currencyDecimals"
    />
  </div>
</template>

<script setup lang="ts">
/**
 * CurrencyInfoInput 组件 —— 货币代码输入框 + 自动查询货币信息。
 *
 * 对标 nrs.modals.uielements.js:84-121 的 add_currency_modal_ui_element：
 *   - 用户输入货币代码（≥3 字符）后，延迟 1 秒调用 `getCurrency` 查询
 *   - 查询成功：展示货币 ID + 精度，并 emit ID 与精度
 *   - 查询失败/不存在：展示 "Not existing"，清空 ID/精度
 *
 * 用于 exchange buy/sell、reserve claim 等需要按货币代码反查 ID/精度的表单。
 */
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { Money, Loading } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules'

const props = withDefaults(
  defineProps<{
    /** v-model 绑定的货币代码 */
    modelValue?: string
    /** 是否禁用 */
    disabled?: boolean
    /** 查询延迟（毫秒，对标 _delay 1000ms） */
    debounceMs?: number
  }>(),
  {
    modelValue: '',
    disabled: false,
    debounceMs: 1000,
  },
)

const emit = defineEmits<{
  'update:modelValue': [value: string]
  /** 货币 ID 查询成功时触发 */
  'currencyId:update': [id: string]
  /** 货币精度查询成功时触发 */
  'decimals:update': [decimals: number]
  /** 货币信息查询完成（成功/失败） */
  loaded: [info: { currencyId: string; decimals: number } | null]
}>()

const { t } = useI18n()

/** 货币代码输入值（.acm_ue_currency_code_input） */
const currencyCodeValue = ref<string>(props.modelValue || '')

/** 查询得到的货币 ID（.acm_ue_currency_id_input） */
const currencyId = ref<string>('')

/** 查询得到的货币精度（.acm_ue_currency_decimals_input） */
const currencyDecimals = ref<number | null>(null)

/** 是否正在查询 */
const loading = ref(false)

/** 延迟定时器（对标 _delay） */
let debounceTimer: ReturnType<typeof setTimeout> | null = null

// 监听外部 modelValue 变化
watch(
  () => props.modelValue,
  (val) => {
    if ((val || '') !== currencyCodeValue.value) {
      currencyCodeValue.value = val || ''
      if (currencyCodeValue.value.length >= 3) {
        scheduleQuery()
      } else {
        resetInfo()
      }
    }
  },
)

/**
 * 输入事件（对标 :117-121 keyup → _delay(_loadCurrencyInfoForCode, 1000)）。
 */
function onInput(val: string): void {
  currencyCodeValue.value = val
  emit('update:modelValue', val)
  if (val.length >= 3) {
    scheduleQuery()
  } else {
    resetInfo()
  }
}

/**
 * 安排延迟查询（对标 _delay 1000ms）。
 */
function scheduleQuery(): void {
  if (debounceTimer) clearTimeout(debounceTimer)
  debounceTimer = setTimeout(loadCurrencyInfo, props.debounceMs)
}

/**
 * 查询货币信息（对标 :95-115 _loadCurrencyInfoForCode）。
 *
 * 调用 `getCurrency` API（按 code），成功时填充 ID/精度并 emit，
 * 失败时展示 "Not existing" 并清空。
 */
async function loadCurrencyInfo(): Promise<void> {
  const code = currencyCodeValue.value.trim()
  if (code.length < 3) {
    resetInfo()
    return
  }

  loading.value = true
  try {
    const response: any = await nrcsApi.getCurrency(code)
    if (response && response.currency && !response.errorCode) {
      currencyId.value = String(response.currency)
      currencyDecimals.value = parseInt(String(response.decimals), 10) || 0
      emit('currencyId:update', currencyId.value)
      emit('decimals:update', currencyDecimals.value)
      emit('loaded', {
        currencyId: currencyId.value,
        decimals: currencyDecimals.value,
      })
    } else {
      resetInfo()
      emit('loaded', null)
    }
  } catch {
    resetInfo()
    emit('loaded', null)
  } finally {
    loading.value = false
  }
}

/**
 * 重置货币信息（对标 _setAssetInfoNotExisting）。
 */
function resetInfo(): void {
  currencyId.value = ''
  currencyDecimals.value = null
}
</script>

<style scoped lang="scss">
.currency-info-input {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.currency-info-display {
  font-size: 13px;
  min-height: 20px;
}
.currency-info-id {
  color: var(--el-text-color-primary);
  font-weight: 500;
}
.currency-info-decimals {
  color: var(--el-text-color-secondary);
  font-weight: 400;
  margin-left: 4px;
}
.currency-info-not-existing {
  color: var(--el-color-danger);
}
</style>
