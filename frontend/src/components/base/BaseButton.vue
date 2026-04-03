<template>
  <el-button
    v-bind="buttonProps"
    :loading="loading"
    :disabled="disabled || loading"
    @click="handleClick"
  >
    <template v-if="$slots.icon" #icon>
      <slot name="icon" />
    </template>
    <slot />
  </el-button>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { ButtonProps } from 'element-plus'

interface Props {
  // 按钮类型
  type?: ButtonProps['type']
  // 按钮尺寸
  size?: ButtonProps['size']
  // 朴素按钮
  plain?: boolean
  // 圆角按钮
  round?: boolean
  // 圆形按钮
  circle?: boolean
  // 加载状态
  loading?: boolean
  // 禁用状态
  disabled?: boolean
  // 图标位置
  iconPosition?: 'left' | 'right'
  // 自动调整宽度
  autoWidth?: boolean
  // 阻止重复点击
  preventReclick?: boolean
  // 重复点击间隔（ms）
  reclickInterval?: number
}

const props = withDefaults(defineProps<Props>(), {
  type: 'default',
  size: 'default',
  plain: false,
  round: false,
  circle: false,
  loading: false,
  disabled: false,
  iconPosition: 'left',
  autoWidth: false,
  preventReclick: true,
  reclickInterval: 1000
})

const emit = defineEmits<{
  click: [event: MouseEvent]
}>()

// 自动计算按钮样式属性
const buttonProps = computed<ButtonProps>(() => ({
  type: props.type,
  size: props.size,
  plain: props.plain,
  round: props.round,
  circle: props.circle,
  autoWidth: props.autoWidth
}))

// 节流控制
let lastClickTime = 0

const handleClick = (event: MouseEvent) => {
  if (props.disabled || props.loading) return

  if (props.preventReclick) {
    const now = Date.now()
    if (now - lastClickTime < props.reclickInterval) {
      return
    }
    lastClickTime = now
  }

  emit('click', event)
}
</script>

<style scoped lang="scss">
// 允许图标位置自定义
:deep(.el-button) {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;

  &.is-circle {
    padding: 0;
  }
}
</style>
