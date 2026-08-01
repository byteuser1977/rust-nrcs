<template>
  <Teleport to="body">
    <div
      v-if="modelValue"
      ref="menuRef"
      class="sidebar-context-menu"
      :style="menuStyle"
      @click.stop
      @contextmenu.prevent
    >
      <ul class="context-menu-list" role="menu">
        <li
          v-for="item in visibleItems"
          :key="item.key"
          class="context-menu-item"
          :class="{ 'context-menu-item--danger': item.danger }"
          @click="onSelect(item)"
        >
          <el-icon v-if="item.icon" class="context-menu-icon">
            <component :is="item.icon" />
          </el-icon>
          <span class="context-menu-label">{{ item.label }}</span>
        </li>
        <li v-if="visibleItems.length === 0" class="context-menu-empty">
          {{ t('sidebarContext.noActions') }}
        </li>
      </ul>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, ref, watch, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import type { Component } from 'vue'

/**
 * 上下文菜单项定义。
 * 对标 NRCS sidebar_context.html 中的 `<a data-option="..." data-class="...">` 结构。
 */
export interface ContextMenuItem {
  /** 唯一标识（对应 NRCS data-option，如 "add_to_group"） */
  key: string
  /** 显示文本 */
  label: string
  /** 图标组件（可选） */
  icon?: Component
  /** 是否为危险操作（红色高亮） */
  danger?: boolean
  /**
   * 可见性谓词。对标 NRCS 的 `data-class` 过滤逻辑：
   * 当 item 携带的 class 集合包含 requiredClass 时该项才显示。
   * 此处改为函数形式，由父组件根据上下文数据决定。
   */
  visible?: (ctx: any) => boolean
}

const props = withDefaults(defineProps<{
  /** 是否可见（v-model） */
  modelValue: boolean
  /** 菜单横坐标（pageX） */
  x: number
  /** 菜单纵坐标（pageY） */
  y: number
  /** 菜单项列表 */
  items: ContextMenuItem[]
  /** 上下文数据（传给 visible 谓词与 select 事件） */
  context?: any
  /** 菜单宽度 */
  width?: number
}>(), {
  width: 200,
})

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  /** 选中某项（payload 为 { key, context }） */
  select: [payload: { key: string; context: any }]
  /** 菜单关闭 */
  close: []
}>()

const { t } = useI18n()

const menuRef = ref<HTMLDivElement | null>(null)

/** 实际显示的菜单项（经过 visible 谓词过滤） */
const visibleItems = computed(() => {
  return props.items.filter((item) => {
    if (!item.visible) return true
    return item.visible(props.context)
  })
})

/** 菜单定位样式（自动调整避免溢出视口） */
const menuStyle = computed(() => {
  const w = props.width
  const h = Math.max(visibleItems.value.length * 36 + 12, 40)
  const vw = window.innerWidth
  const vh = window.innerHeight
  const left = props.x + w > vw ? Math.max(0, props.x - w) : props.x
  const top = props.y + h > vh ? Math.max(0, props.y - h) : props.y
  return {
    left: `${left}px`,
    top: `${top}px`,
    width: `${w}px`,
  }
})

/**
 * 选中菜单项。
 */
function onSelect(item: ContextMenuItem): void {
  emit('select', { key: item.key, context: props.context })
  close()
}

/**
 * 关闭菜单。
 */
function close(): void {
  emit('update:modelValue', false)
  emit('close')
}

/**
 * 全局点击/右键/ESC 关闭（对标 nrs.sidebar.js:30 的 click.contextmenu 绑定）。
 */
function onGlobalAction(e: MouseEvent | KeyboardEvent): void {
  if (!props.modelValue) return
  if (e instanceof KeyboardEvent) {
    if (e.key === 'Escape') close()
    return
  }
  // 鼠标点击在菜单外部则关闭
  if (menuRef.value && !menuRef.value.contains(e.target as Node)) {
    close()
  }
}

watch(
  () => props.modelValue,
  (visible) => {
    if (visible) {
      document.addEventListener('click', onGlobalAction, true)
      document.addEventListener('contextmenu', onGlobalAction, true)
      document.addEventListener('keydown', onGlobalAction, true)
    } else {
      document.removeEventListener('click', onGlobalAction, true)
      document.removeEventListener('contextmenu', onGlobalAction, true)
      document.removeEventListener('keydown', onGlobalAction, true)
    }
  }
)

onMounted(() => {
  if (props.modelValue) {
    document.addEventListener('click', onGlobalAction, true)
    document.addEventListener('contextmenu', onGlobalAction, true)
    document.addEventListener('keydown', onGlobalAction, true)
  }
})

onUnmounted(() => {
  document.removeEventListener('click', onGlobalAction, true)
  document.removeEventListener('contextmenu', onGlobalAction, true)
  document.removeEventListener('keydown', onGlobalAction, true)
})
</script>

<style scoped lang="scss">
.sidebar-context-menu {
  position: fixed;
  z-index: 9999;
  background: $bg-card;
  border: 1px solid $border-default;
  border-radius: $radius-md;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.2);
  padding: 4px 0;
  user-select: none;
}

.context-menu-list {
  list-style: none;
  margin: 0;
  padding: 0;
}

.context-menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 14px;
  cursor: pointer;
  font-size: 13px;
  color: $text-primary;
  transition: all $duration-fast ease;

  &:hover {
    background: $bg-hover;
    color: $primary;
  }

  &--danger {
    color: $danger;

    &:hover {
      background: rgba($danger, 0.08);
      color: $danger;
    }
  }

  .context-menu-icon {
    font-size: 14px;
    flex-shrink: 0;
  }

  .context-menu-label {
    flex: 1;
  }
}

.context-menu-empty {
  padding: 8px 14px;
  font-size: 12px;
  color: $text-muted;
  text-align: center;
}
</style>
