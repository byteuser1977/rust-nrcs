<template>
  <!--
    RtHashInput —— 引用交易哈希输入 UI 元素。
    对标 NRCS 表单中的 phasingLinkedFullHash 字段（引用交易 phasing 模型）。
  -->
  <div class="rt-hash-input">
    <el-input
      v-model="hashValue"
      :placeholder="t('uiElements.rtHashPlaceholder')"
      :disabled="disabled"
      clearable
      class="rt-hash-field"
      @input="onInput"
      @blur="onBlur"
    >
      <template #prefix><el-icon><Link /></el-icon></template>
    </el-input>
    <p v-if="hint" class="rt-hash-hint" :class="{ 'is-error': hasError }">{{ hint }}</p>
  </div>
</template>

<script setup lang="ts">
/**
 * RtHashInput 组件 —— 引用交易哈希（return transaction hash）输入。
 *
 * 对标 NRCS 表单中的 `phasingLinkedFullHash` 字段：
 *   - 用于 phasing voting model = 1（by transaction）时指定被引用的交易哈希
 *   - 哈希为 64 字符十六进制字符串（SHA256/Keccak256）
 *
 * 提供格式校验：长度 64 + 仅 hex 字符。
 */
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { Link } from '@element-plus/icons-vue'

const props = withDefaults(
  defineProps<{
    /** v-model 绑定的哈希值 */
    modelValue?: string
    /** 是否禁用 */
    disabled?: boolean
    /** 是否必填（影响校验提示） */
    required?: boolean
  }>(),
  {
    modelValue: '',
    disabled: false,
    required: false,
  },
)

const emit = defineEmits<{
  'update:modelValue': [value: string]
  change: [value: string]
  /** 校验结果（true=合法） */
  valid: [valid: boolean]
}>()

const { t } = useI18n()

/** 哈希输入值 */
const hashValue = ref<string>(props.modelValue || '')

/** 是否已触发过校验（blur 后） */
const touched = ref<boolean>(false)

/** 校验是否通过（64 字符 hex） */
const isValid = computed(() => {
  if (!hashValue.value) return !props.required
  return /^[0-9a-fA-F]{64}$/.test(hashValue.value.trim())
})

/** 是否有错误 */
const hasError = computed(() => touched.value && !isValid.value)

/** 提示文本 */
const hint = computed(() => {
  if (!touched.value) return ''
  if (!hashValue.value && props.required) {
    return t('uiElements.rtHashRequired')
  }
  if (hashValue.value && !isValid.value) {
    return t('uiElements.rtHashInvalid')
  }
  return ''
})

// 监听外部 modelValue 变化
watch(
  () => props.modelValue,
  (val) => {
    if ((val || '') !== hashValue.value) {
      hashValue.value = val || ''
    }
  },
)

/**
 * 输入事件。
 */
function onInput(val: string): void {
  hashValue.value = val
  emit('update:modelValue', val)
  emit('change', val)
  if (touched.value) {
    emit('valid', isValid.value)
  }
}

/**
 * 失焦时触发校验。
 */
function onBlur(): void {
  touched.value = true
  emit('valid', isValid.value)
}
</script>

<style scoped lang="scss">
.rt-hash-input {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.rt-hash-field {
  font-family: monospace;
}
.rt-hash-hint {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin: 0;
  &.is-error {
    color: var(--el-color-danger);
  }
}
</style>
