<template>
  <!--
    BlockHeightPicker —— 区块高度选择器 UI 元素。
    对标 nrs.modals.uielements.js 中的 block_height_modal_ui_element（:30-81）。

    功能：
      1. 展示当前链高度（.bhm_ue_current_block_height）
      2. "使用当前高度"按钮（.bhm_ue_use_current_block_height）
      3. +/- 按钮按步长调整高度（.bhm_ue_add_height_btn / .bhm_ue_reduce_height_btn）
      4. 高度输入框（.bhm_ue_time_input）
      5. 预计到达时间显示（.bhm_ue_time_estimate，基于 getBlockHeightTimeEstimate）
  -->
  <div class="block-height-picker" data-modal-ui-element="block_height_modal_ui_element">
    <div class="bhm-current">
      <span class="bhm-label">{{ t('uiElements.currentBlockHeight') }}:</span>
      <span class="bhm-current-height">{{ lastBlockHeight || '-' }}</span>
      <el-button
        v-if="lastBlockHeight"
        link
        type="primary"
        size="small"
        class="bhm_ue_use_current_block_height"
        @click="useCurrentHeight"
      >
        {{ t('uiElements.useCurrent') }}
      </el-button>
    </div>

    <div class="bhm-input-row">
      <el-button
        v-if="step > 0"
        circle
        size="small"
        class="bhm_ue_reduce_height_btn"
        :disabled="!modelValue || Number(modelValue) <= step"
        @click="changeHeight(-step)"
      >
        <el-icon><Minus /></el-icon>
      </el-button>
      <el-input
        v-model="heightValue"
        type="number"
        :placeholder="t('uiElements.blockHeightPlaceholder')"
        :disabled="disabled"
        class="bhm_ue_time_input"
        @input="onHeightInput"
      />
      <el-button
        v-if="step > 0"
        circle
        size="small"
        class="bhm_ue_add_height_btn"
        @click="changeHeight(step)"
      >
        <el-icon><Plus /></el-icon>
      </el-button>
    </div>

    <div v-if="timeEstimate !== '-'" class="bhm_ue_time_estimate">
      <el-icon><Clock /></el-icon>
      <span>{{ t('uiElements.estimatedTime') }}: {{ timeEstimate }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
/**
 * BlockHeightPicker 组件 —— 区块高度选择器。
 *
 * 对标 nrs.modals.uielements.js:30-81 的 block_height_modal_ui_element：
 *   - 当前高度展示 + "使用当前"按钮
 *   - 高度输入 + +/- 步进按钮（data-bhm-ue-num-blocks）
 *   - 实时预计到达时间（getBlockHeightTimeEstimate）
 *
 * 用于 phasing finish height、shuffling 注册截止高度等需要指定未来区块高度的场景。
 */
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { Plus, Minus, Clock } from '@element-plus/icons-vue'
import { useBlockHeight } from '@/composables/useBlockHeight'

const props = withDefaults(
  defineProps<{
    /** v-model 绑定的区块高度 */
    modelValue?: number | string
    /** +/- 按钮步长（0 表示隐藏步进按钮） */
    step?: number
    /** 是否禁用 */
    disabled?: boolean
  }>(),
  {
    modelValue: '',
    step: 0,
    disabled: false,
  },
)

const emit = defineEmits<{
  'update:modelValue': [value: string]
  /** 高度变化时同步触发，便于父组件获取时间估算 */
  change: [value: string, estimate: string]
}>()

const { t } = useI18n()
const { lastBlockHeight, getBlockHeightTimeEstimate } = useBlockHeight()

/** 内部高度值（字符串形式，与 NRCS API 一致） */
const heightValue = ref<string>(
  props.modelValue !== undefined && props.modelValue !== '' ? String(props.modelValue) : '',
)

/** 预计到达时间（对标 .bhm_ue_time_estimate） */
const timeEstimate = computed(() =>
  heightValue.value ? getBlockHeightTimeEstimate(heightValue.value) : '-',
)

// 监听外部 modelValue 变化
watch(
  () => props.modelValue,
  (val) => {
    const str = val !== undefined && val !== '' ? String(val) : ''
    if (str !== heightValue.value) {
      heightValue.value = str
    }
  },
)

/**
 * 使用当前链高度（对标 :65-69 .bhm_ue_use_current_block_height 点击）。
 */
function useCurrentHeight(): void {
  if (lastBlockHeight.value) {
    heightValue.value = String(lastBlockHeight.value)
    emitHeight()
  }
}

/**
 * 按步长调整高度（对标 :37-47 _changeBlockHeightFromButton）。
 *
 * @param delta 增量（正数加，负数减）
 */
function changeHeight(delta: number): void {
  const current = parseInt(heightValue.value || '0', 10) || 0
  const next = Math.max(1, current + delta)
  heightValue.value = String(next)
  emitHeight()
}

/**
 * 输入框输入事件（对标 :60-63 keyup → _updateBlockHeightEstimates）。
 */
function onHeightInput(val: number | string): void {
  heightValue.value = val === undefined || val === null ? '' : String(val)
  emitHeight()
}

/**
 * 统一向上 emit 高度变化与时间估算。
 */
function emitHeight(): void {
  emit('update:modelValue', heightValue.value)
  emit('change', heightValue.value, timeEstimate.value)
}
</script>

<style scoped lang="scss">
.block-height-picker {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.bhm-current {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: var(--el-text-color-secondary);
}
.bhm-current-height {
  font-family: monospace;
  font-weight: 600;
  color: var(--el-text-color-primary);
}
.bhm-input-row {
  display: flex;
  align-items: center;
  gap: 8px;
}
.bhm_ue_time_input {
  flex: 1;
}
.bhm_ue_time_estimate {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
}
</style>
