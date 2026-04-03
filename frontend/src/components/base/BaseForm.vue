<template>
  <el-form
    ref="formRef"
    :model="formData"
    :rules="formRules"
    :label-width="labelWidth"
    :label-position="labelPosition"
    :size="size"
    :disabled="disabled"
    :validate-on-rule-change="validateOnRuleChange"
    @validate="handleValidate"
    @submit.prevent="handleSubmit"
  >
    <slot />

    <template v-if="$slots.default || fields.length > 0">
      <!-- 自动生成表单项（如果提供 fields 配置） -->
      <template v-if="fields.length > 0">
        <el-form-item
          v-for="field in fields"
          :key="field.prop"
          :prop="field.prop"
          :label="field.label"
          :required="field.required"
        >
          <component
            :is="getFieldComponent(field.type)"
            v-model="formData[field.prop]"
            v-bind="field.props"
            :placeholder="field.placeholder"
            :clearable="field.clearable !== false"
            @change="(val: any) => handleFieldChange(field.prop, val)"
          >
            <!-- 选项类型支持 -->
            <template v-if="field.options" #default>
              <component
                v-for="option in field.options"
                :is="getOptionComponent(field.type)"
                :key="option.value"
                :label="option.value"
                :value="option.value"
                :disabled="option.disabled"
              >
                {{ option.label }}
              </component>
            </template>
          </component>
          <template v-if="field.suffix" #label>
            <span>{{ field.label }}</span>
            <el-tooltip v-if="field.tooltip" :content="field.tooltip" placement="top">
              <el-icon class="field-tooltip"><QuestionFilled /></el-icon>
            </el-tooltip>
          </template>
        </el-form-item>
      </template>

      <!-- 自定义字段插槽 -->
      <slot name="fields" />
    </template>

    <el-form-item v-if="$slots.actions || showSubmitButtons" :label="actionLabel">
      <slot name="actions">
        <BaseButton @click="handleReset">
          {{ resetText }}
        </BaseButton>
        <BaseButton
          type="primary"
          :loading="submitting"
          @click="handleSubmit"
        >
          {{ submitText }}
        </BaseButton>
      </slot>
    </el-form-item>
  </el-form>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch, type Component } from 'vue'
import type { FormInstance, FormRules } from 'element-plus'
import BaseButton from './BaseButton.vue'

interface FormField {
  prop: string
  label: string
  type: 'input' | 'textarea' | 'select' | 'checkbox' | 'radio' | 'switch' | 'date' | 'number' | 'password'
  value?: any
  props?: Record<string, any>
  options?: Array<{ label: string; value: any; disabled?: boolean }>
  required?: boolean
  placeholder?: string
  clearable?: boolean
  tooltip?: string
  suffix?: boolean
}

interface Props {
  // 表单数据（v-model）
  modelValue: Record<string, any>
  // 表单字段配置（可选，自动生成表单项）
  fields?: FormField[]
  // 表单验证规则
  rules?: FormRules
  // 标签宽度
  labelWidth?: string | number
  // 标签位置：left / right / top
  labelPosition?: 'left' | 'right' | 'top'
  // 表单尺寸
  size?: 'large' | 'default' | 'small'
  // 是否禁用
  disabled?: boolean
  // 是否在规则变更时立即验证
  validateOnRuleChange?: boolean
  // 动作区域标签
  actionLabel?: string
  // 提交按钮文本
  submitText?: string
  // 重置按钮文本
  resetText?: string
  // 是否显示提交/重置按钮
  showSubmitButtons?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  fields: () => [],
  rules: () => ({}),
  labelWidth: '120px',
  labelPosition: 'right',
  size: 'default',
  disabled: false,
  validateOnRuleChange: true,
  actionLabel: '',
  submitText: '提交',
  resetText: '重置',
  showSubmitButtons: true
})

const emit = defineEmits<{
  'update:modelValue': [value: Record<string, any>]
  submit: [data: Record<string, any>]
  reset: []
  validate: [prop: string, isValid: boolean, message: string]
}>()

const formRef = ref<FormInstance>()
const formData = reactive<Record<string, any>>({})
const submitting = ref(false)

// 同步 modelValue 到内部表单数据
watch(
  () => props.modelValue,
  (newVal) => {
    Object.assign(formData, newVal)
  },
  { immediate: true, deep: true }
)

// 监听内部变化并emit
watch(formData, (newVal) => {
  emit('update:modelValue', newVal)
}, { deep: true })

// 字段变更处理
const handleFieldChange = (prop: string, value: any) => {
  // 可以在这里添加字段变更的副作用逻辑
}

// 获取字段组件
const getFieldComponent = (type: FormField['type']): Component => {
  const componentMap: Record<FormField['type'], Component> = {
    input: 'el-input',
    textarea: 'el-input',
    select: 'el-select',
    checkbox: 'el-checkbox',
    radio: 'el-radio',
    switch: 'el-switch',
    date: 'el-date-picker',
    number: 'el-input-number',
    password: 'el-input'
  }
  return componentMap[type] || 'el-input'
}

// 获取选项组件
const getOptionComponent = (type: FormField['type']): Component => {
  switch (type) {
    case 'select':
      return 'el-option'
    case 'checkbox':
      return 'el-checkbox-group'
    case 'radio':
      return 'el-radio'
    default:
      return 'el-option'
  }
}

// 表单验证
const validate = async () => {
  if (!formRef.value) return true
  try {
    await formRef.value.validate()
    return true
  } catch (error) {
    return false
  }
}

// 清除验证
const clearValidate = () => {
  formRef.value?.clearValidate()
}

// 重置表单
const resetFields = () => {
  if (!formRef.value) return
  formRef.value.resetFields()
  emit('reset')
}

// 提交处理
const handleSubmit = async () => {
  if (!formRef.value) return

  const isValid = await validate()
  if (!isValid) return

  submitting.value = true
  try {
    emit('submit', { ...formData })
  } finally {
    submitting.value = false
  }
}

// 重置处理
const handleReset = () => {
  resetFields()
  emit('update:modelValue', { ...formData })
}

// 验证事件
const handleValidate = (prop: string, isValid: boolean, message: string) => {
  emit('validate', prop, isValid, message)
}

// 暴露方法给父组件
defineExpose({
  formRef,
  formData,
  validate,
  clearValidate,
  resetFields,
  setFieldsValue: (values: Record<string, any>) => {
    Object.assign(formData, values)
  }
})
</script>

<style scoped lang="scss">
.field-tooltip {
  margin-left: 4px;
  color: #909399;
  cursor: help;
}
</style>
