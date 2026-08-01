<template>
  <!--
    DeadlinePicker —— 交易截止时间选择器 UI 元素。
    对标参考表单中的 deadline 输入（分钟，默认 1440 = 24 小时）。
  -->
  <div class="deadline-picker">
    <el-input-number
      v-model="deadlineValue"
      :min="1"
      :max="65535"
      :step="60"
      controls-position="right"
      :disabled="disabled"
      class="deadline-input"
      @change="onChange"
    />
    <span class="deadline-unit">{{ t('uiElements.minutes') }}</span>
    <span class="deadline-hint">{{ deadlineHint }}</span>
  </div>
</template>

<script setup lang="ts">
/**
 * DeadlinePicker 组件 —— 交易截止时间（分钟）输入。
 *
 * 对标 NRCS 表单中的 deadline 字段：
 *   - NRCS API 接收分钟数（默认 1440 = 24 小时）
 *   - 上限 65535（MAX_UNSIGNED_SHORT_JAVA，对标 nrs.modals.balanceleasing.js 中的上限校验）
 *
 * `useNrcsForm` 在提交时会将小时制输入转为分钟，本组件直接以分钟为单位，
 * 同时展示等价的小时/天提示，便于用户理解。
 */
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'

const props = withDefaults(
  defineProps<{
    /** v-model 绑定的截止时间（分钟） */
    modelValue?: number | string
    /** 是否禁用 */
    disabled?: boolean
  }>(),
  {
    modelValue: 1440,
    disabled: false,
  },
)

const emit = defineEmits<{
  'update:modelValue': [value: number]
  change: [value: number]
}>()

const { t } = useI18n()

/** 截止时间值（分钟） */
const deadlineValue = ref<number>(
  props.modelValue !== undefined && props.modelValue !== ''
    ? Number(props.modelValue)
    : 1440,
)

/** 等价时间提示（小时/天） */
const deadlineHint = computed(() => {
  const minutes = deadlineValue.value
  if (!minutes || minutes <= 0) return ''
  const hours = minutes / 60
  if (hours < 48) {
    return `≈ ${hours.toFixed(1)} ${t('uiElements.hours')}`
  }
  const days = hours / 24
  return `≈ ${days.toFixed(1)} ${t('uiElements.days')}`
})

// 监听外部 modelValue 变化
watch(
  () => props.modelValue,
  (val) => {
    const num = val !== undefined && val !== '' ? Number(val) : 1440
    if (num !== deadlineValue.value) {
      deadlineValue.value = num
    }
  },
)

/**
 * 值变化时 emit（对标 deadline 字段提交）。
 */
function onChange(val: number): void {
  deadlineValue.value = val
  emit('update:modelValue', val)
  emit('change', val)
}
</script>

<style scoped lang="scss">
.deadline-picker {
  display: flex;
  align-items: center;
  gap: 8px;
}
.deadline-input {
  flex: 1;
}
.deadline-unit {
  font-size: 13px;
  color: var(--el-text-color-secondary);
  white-space: nowrap;
}
.deadline-hint {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}
</style>
