<template>
  <!--
    NoteToSelfInput —— "给自己留言"加密消息 UI 元素。
    对标 nrs.modals.js:74-80 的 .add_note_to_self 复选框：
      勾选后展示 .optional_note 文本区域。
  -->
  <div class="note-to-self-input">
    <el-checkbox v-model="enabled" :disabled="disabled" @change="onToggle">
      {{ t('uiElements.addNoteToSelf') }}
    </el-checkbox>
    <el-input
      v-if="enabled"
      v-model="noteValue"
      type="textarea"
      :rows="2"
      :placeholder="t('uiElements.noteToSelfPlaceholder')"
      :disabled="disabled"
      class="optional_note"
      @input="onInput"
    />
    <p v-if="enabled" class="note-hint">{{ t('uiElements.noteToSelfHint') }}</p>
  </div>
</template>

<script setup lang="ts">
/**
 * NoteToSelfInput 组件 —— "给自己留言"加密消息输入。
 *
 * 对标 nrs.modals.js:74-80 的 `.add_note_to_self`：
 *   - 复选框勾选后展示 `.optional_note` 文本区域
 *   - 内容会作为 `encryptToSelfMessage` 附件随交易提交（本地加密）
 *
 * 用于 sendMoney/sendMessage 等交易表单，允许用户附加一条只有自己能解密的备注。
 */
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

const props = withDefaults(
  defineProps<{
    /** v-model 绑定的留言内容 */
    modelValue?: string
    /** 是否启用（复选框状态） */
    enabled?: boolean
    /** 是否禁用 */
    disabled?: boolean
  }>(),
  {
    modelValue: '',
    enabled: false,
    disabled: false,
  },
)

const emit = defineEmits<{
  'update:modelValue': [value: string]
  'update:enabled': [value: boolean]
  change: [value: string]
}>()

const { t } = useI18n()

/** 是否启用留言（.add_note_to_self） */
const enabled = ref<boolean>(props.enabled)

/** 留言内容（.optional_note） */
const noteValue = ref<string>(props.modelValue)

// 监听外部 modelValue/enabled 变化
watch(
  () => props.modelValue,
  (val) => {
    if (val !== noteValue.value) noteValue.value = val
  },
)
watch(
  () => props.enabled,
  (val) => {
    if (val !== enabled.value) enabled.value = val
  },
)

/**
 * 复选框切换（对标 :74-80 .add_note_to_self change）。
 */
function onToggle(val: boolean): void {
  enabled.value = val
  emit('update:enabled', val)
  if (!val) {
    noteValue.value = ''
    emit('update:modelValue', '')
  }
}

/**
 * 留言内容输入。
 */
function onInput(val: string): void {
  noteValue.value = val
  emit('update:modelValue', val)
  emit('change', val)
}
</script>

<style scoped lang="scss">
.note-to-self-input {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.optional_note {
  margin-top: 4px;
}
.note-hint {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin: 0;
}
</style>
