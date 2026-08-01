<template>
  <!--
    HashAlgorithmSelect —— 哈希算法选择器 UI 元素。
    对标 nrs.modals.uielements.js:56-58 的 hash_algorithm_model_modal_ui_element：
      NRS.loadAlgorithmList($algoSelect, ($(this).attr('id') != 'hash_modal'));
  -->
  <el-select
    v-model="algorithmValue"
    :placeholder="t('uiElements.selectAlgorithm')"
    :disabled="disabled"
    clearable
    class="hash-algorithm-select"
    @change="onChange"
  >
    <el-option
      v-for="opt in options"
      :key="opt.value"
      :label="opt.label"
      :value="opt.value"
    />
  </el-select>
</template>

<script setup lang="ts">
/**
 * HashAlgorithmSelect 组件 —— 哈希算法下拉选择。
 *
 * 对标 nrs.modals.uielements.js:56-58 + nrs.constants.js:78 loadAlgorithmList。
 *
 * Vue3 中由 `useConstantsStore().getAlgorithmOptions(isPhasingHash)` 返回选项列表，
 * 绑定到 `<el-select>`，而非参考实现的直接操作 DOM。
 *
 * 使用场景：
 *   - hash_modal（计算哈希）
 *   - phasing 控制中的哈希算法选择
 *   - approveTransaction 的 phasing hash 校验
 */
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useConstantsStore } from '@/stores/modules/constants.store'

const props = withDefaults(
  defineProps<{
    /** v-model 绑定的算法 ID */
    modelValue?: number | string
    /** 是否取 phasing 哈希算法表（对标 loadAlgorithmList 第二参数） */
    isPhasingHash?: boolean
    /** 是否禁用 */
    disabled?: boolean
  }>(),
  {
    modelValue: '',
    isPhasingHash: false,
    disabled: false,
  },
)

const emit = defineEmits<{
  'update:modelValue': [value: number]
  change: [value: number]
}>()

const { t } = useI18n()
const constantsStore = useConstantsStore()

/** 算法选项列表（对标 NRS.loadAlgorithmList 返回值） */
const options = computed(() => constantsStore.getAlgorithmOptions(props.isPhasingHash))

/** 内部值（统一为 number） */
const algorithmValue = ref<number>(
  props.modelValue !== undefined && props.modelValue !== '' ? Number(props.modelValue) : 0,
)

// 监听外部 modelValue 变化
watch(
  () => props.modelValue,
  (val) => {
    const num = val !== undefined && val !== '' ? Number(val) : 0
    if (num !== algorithmValue.value) {
      algorithmValue.value = num
    }
  },
)

/**
 * 选择变化时 emit（对标 loadAlgorithmList 后用户选择算法）。
 */
function onChange(val: number): void {
  emit('update:modelValue', val)
  emit('change', val)
}
</script>

<style scoped lang="scss">
.hash-algorithm-select {
  width: 100%;
}
</style>
