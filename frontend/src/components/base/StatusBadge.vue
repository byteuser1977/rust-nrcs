<template>
  <el-tag
    :type="tagType"
    :effect="effect"
    :size="size"
    :round="round"
  >
    <slot name="icon" />
    <slot>{{ displayText }}</slot>
  </el-tag>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  // 状态值
  value: string | number | boolean
  // 状态类型映射（可选）
  typeMap?: Record<string, { type: 'success' | 'warning' | 'danger' | 'info'; text?: string }>
  // 默认类型（未匹配时使用）
  defaultType?: 'success' | 'warning' | 'danger' | 'info'
  // 效果：light / dark / plain
  effect?: 'light' | 'dark' | 'plain'
  // 尺寸
  size?: 'large' | 'default' | 'small'
  // 是否圆角
  round?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  defaultType: 'info',
  effect: 'light',
  size: 'default',
  round: false,
  typeMap: () => ({})
})

const statusToString = (val: any): string => {
  if (val === null || val === undefined) return 'unknown'
  if (typeof val === 'boolean') return val ? 'true' : 'false'
  return String(val).toLowerCase()
}

const tagType = computed(() => {
  const status = statusToString(props.value)
  const mapping = props.typeMap[status]

  if (mapping) {
    return mapping.type
  }

  // 内置常见状态映射
  switch (status) {
    case 'success':
    case 'succeeded':
    case 'completed':
    case 'true':
    case 'active':
    case 'enabled':
      return 'success'

    case 'warning':
    case 'pending':
    case 'processing':
    case 'waiting':
    case 'syncing':
      return 'warning'

    case 'error':
    case 'failed':
    case 'danger':
    case 'critical':
    case 'false':
    case 'inactive':
    case 'disabled':
      return 'danger'

    case 'info':
    case 'notice':
    case 'alert':
      return 'info'

    default:
      return props.defaultType
  }
})

const displayText = computed(() => {
  const status = statusToString(props.value)
  const mapping = props.typeMap[status]

  if (mapping?.text) {
    return mapping.text
  }

  // 默认文本映射
  switch (status) {
    case 'success':
    case 'succeeded':
    case 'completed':
      return '成功'
    case 'warning':
    case 'pending':
    case 'processing':
      return '处理中'
    case 'error':
    case 'failed':
    case 'danger':
      return '失败'
    case 'info':
      return '提示'
    case 'true':
      return '是'
    case 'false':
      return '否'
    case 'active':
      return '活跃'
    case 'inactive':
      return '不活跃'
    default:
      return status
  }
})
</script>
