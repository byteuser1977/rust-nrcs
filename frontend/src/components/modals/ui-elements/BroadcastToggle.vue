<template>
  <!--
    BroadcastToggle —— 是否广播交易开关 UI 元素。
    对标 nrs.modals.js:82-91 的 .do_not_broadcast 复选框：
      勾选后展示 .optional_do_not_sign（离线签名模式）。
  -->
  <div class="broadcast-toggle">
    <el-checkbox v-model="doNotBroadcast" :disabled="disabled" @change="onChange">
      {{ t('uiElements.doNotBroadcast') }}
    </el-checkbox>
    <div v-if="doNotBroadcast" class="optional_do_not_sign">
      <el-checkbox v-model="doNotSign" :disabled="disabled" @change="onDoNotSignChange">
        {{ t('uiElements.doNotSign') }}
      </el-checkbox>
      <p class="broadcast-hint">{{ t('uiElements.doNotBroadcastHint') }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
/**
 * BroadcastToggle 组件 —— "不广播"/"不签名"开关。
 *
 * 对标 nrs.modals.js:82-98 的 `.do_not_broadcast` 与 `.do_not_sign` 复选框：
 *   - 勾选"不广播"（doNotBroadcast）→ 展示"不签名"选项（离线签名模式）
 *   - 勾选"不签名"（doNotSign）→ 调用方应禁用 secretPhrase 输入并要求 publicKey
 *
 * 用于 RawTransactionModal 离线签名场景：用户可生成未签名/已签名但不广播的交易字节。
 */
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

const props = withDefaults(
  defineProps<{
    /** v-model 绑定的"不广播"状态 */
    modelValue?: boolean
    /** 是否禁用 */
    disabled?: boolean
  }>(),
  {
    modelValue: false,
    disabled: false,
  },
)

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  /** "不签名"状态变化（doNotSign） */
  'doNotSignChange': [value: boolean]
  change: [value: boolean]
}>()

const { t } = useI18n()

/** 不广播（.do_not_broadcast） */
const doNotBroadcast = ref<boolean>(props.modelValue)

/** 不签名（.do_not_sign） */
const doNotSign = ref<boolean>(false)

// 监听外部 modelValue 变化
watch(
  () => props.modelValue,
  (val) => {
    if (val !== doNotBroadcast.value) {
      doNotBroadcast.value = val
      if (!val) doNotSign.value = false
    }
  },
)

/**
 * "不广播"变化时 emit（对标 :82-91 .do_not_broadcast change）。
 */
function onChange(val: boolean): void {
  doNotBroadcast.value = val
  if (!val) {
    // 取消广播时重置不签名状态（对标 :87-90 隐藏并取消勾选）
    doNotSign.value = false
    emit('doNotSignChange', false)
  }
  emit('update:modelValue', val)
  emit('change', val)
}

/**
 * "不签名"变化时 emit（对标 :93-104 .do_not_sign change）。
 */
function onDoNotSignChange(val: boolean): void {
  doNotSign.value = val
  emit('doNotSignChange', val)
}
</script>

<style scoped lang="scss">
.broadcast-toggle {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.optional_do_not_sign {
  padding-left: 24px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.broadcast-hint {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin: 0;
}
</style>
