<template>
  <!--
    AssetInfoInput —— 资产 ID 输入 + 自动查询名称/精度 UI 元素。
    对标 nrs.modals.uielements.js:123-155 的 add_asset_modal_ui_element。
  -->
  <div class="asset-info-input" data-modal-ui-element="add_asset_modal_ui_element">
    <el-input
      v-model="assetIdValue"
      :placeholder="t('uiElements.assetIdPlaceholder')"
      :disabled="disabled"
      clearable
      class="aam_ue_asset_id_input"
      @input="onInput"
    >
      <template #prefix><el-icon><Coin /></el-icon></template>
    </el-input>

    <!-- 资产名称展示（.aam_ue_asset_name） -->
    <div v-if="assetIdValue" class="aam_ue_asset_name asset-info-display">
      <el-icon v-if="loading" class="is-loading"><Loading /></el-icon>
      <span v-else-if="assetName" class="asset-info-name">
        {{ assetName }}
        <span class="asset-info-decimals">
          ({{ t('uiElements.decimals') }}: {{ assetDecimals }})
        </span>
      </span>
      <span v-else class="asset-info-not-existing">{{ t('uiElements.notExisting') }}</span>
    </div>

    <!-- 精度隐藏字段（.aam_ue_asset_decimals_input） -->
    <input
      v-if="assetDecimals !== null"
      type="hidden"
      class="aam_ue_asset_decimals_input"
      :value="assetDecimals"
    />
  </div>
</template>

<script setup lang="ts">
/**
 * AssetInfoInput 组件 —— 资产 ID 输入框 + 自动查询资产信息。
 *
 * 对标 nrs.modals.uielements.js:123-155 的 add_asset_modal_ui_element：
 *   - 用户输入资产 ID 后，延迟 1 秒调用 `getAsset` 查询
 *   - 查询成功：展示资产名称 + 精度，并 emit 精度值
 *   - 查询失败/不存在：展示 "Not existing"，清空精度
 *
 * 用于 dividend payment（按资产派息）、asset property 设置等需要按资产 ID
 * 反查精度的表单。
 */
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { Coin, Loading } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules'

const props = withDefaults(
  defineProps<{
    /** v-model 绑定的资产 ID */
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
  /** 资产信息查询成功时触发（精度） */
  'decimals:update': [decimals: number]
  /** 资产信息查询完成（成功/失败） */
  loaded: [info: { name: string; decimals: number } | null]
}>()

const { t } = useI18n()

/** 资产 ID 输入值（.aam_ue_asset_id_input） */
const assetIdValue = ref<string>(props.modelValue || '')

/** 查询得到的资产名称 */
const assetName = ref<string>('')

/** 查询得到的资产精度（.aam_ue_asset_decimals_input） */
const assetDecimals = ref<number | null>(null)

/** 是否正在查询 */
const loading = ref(false)

/** 延迟定时器（对标 _delay） */
let debounceTimer: ReturnType<typeof setTimeout> | null = null

// 监听外部 modelValue 变化
watch(
  () => props.modelValue,
  (val) => {
    if ((val || '') !== assetIdValue.value) {
      assetIdValue.value = val || ''
      if (assetIdValue.value) {
        scheduleQuery()
      } else {
        resetInfo()
      }
    }
  },
)

/**
 * 输入事件（对标 :151-155 keyup → _delay(_loadAssetInfoForId, 1000)）。
 */
function onInput(val: string): void {
  assetIdValue.value = val
  emit('update:modelValue', val)
  if (val) {
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
  debounceTimer = setTimeout(loadAssetInfo, props.debounceMs)
}

/**
 * 查询资产信息（对标 :132-149 _loadAssetInfoForId）。
 *
 * 调用 `getAsset` API，成功时填充名称/精度并 emit，
 * 失败时展示 "Not existing" 并清空精度。
 */
async function loadAssetInfo(): Promise<void> {
  const id = assetIdValue.value.trim()
  if (!id) {
    resetInfo()
    return
  }

  loading.value = true
  try {
    const response: any = await nrcsApi.getAsset(id)
    if (response && response.asset && !response.errorCode) {
      assetName.value = String(response.name || '')
      assetDecimals.value = parseInt(String(response.decimals), 10) || 0
      emit('decimals:update', assetDecimals.value)
      emit('loaded', { name: assetName.value, decimals: assetDecimals.value })
    } else {
      // 不存在（对标 _setAssetInfoNotExisting）
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
 * 重置资产信息（对标 _setAssetInfoNotExisting）。
 */
function resetInfo(): void {
  assetName.value = ''
  assetDecimals.value = null
}
</script>

<style scoped lang="scss">
.asset-info-input {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.asset-info-display {
  font-size: 13px;
  min-height: 20px;
}
.asset-info-name {
  color: var(--el-text-color-primary);
  font-weight: 500;
}
.asset-info-decimals {
  color: var(--el-text-color-secondary);
  font-weight: 400;
  margin-left: 4px;
}
.asset-info-not-existing {
  color: var(--el-color-danger);
}
</style>
