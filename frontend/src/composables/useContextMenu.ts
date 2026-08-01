/**
 * 上下文菜单（右键菜单）管理组合式函数。
 *
 * 配合 `@/components/common/SidebarContextMenu.vue` 使用，对标 NRCS
 * `nrs.sidebar.js` 中 `.sidebar_context` 的 `contextmenu` 事件绑定逻辑。
 *
 * 用法：
 * ```ts
 * const { menuVisible, menuX, menuY, menuContext, onContextMenu, onMenuSelect } = useContextMenu()
 *
 * function onContextMenu(e: MouseEvent, item: MyItem) {
 *   e.preventDefault()
 *   menuContext.value = item
 *   menuVisible.value = true
 *   menuX.value = e.pageX
 *   menuY.value = e.pageY
 * }
 * ```
 */
import { ref } from 'vue'
import type { ContextMenuItem } from '@/components/common/SidebarContextMenu.vue'

export function useContextMenu<T = any>() {
  /** 菜单是否可见 */
  const menuVisible = ref(false)
  /** 菜单横坐标 */
  const menuX = ref(0)
  /** 菜单纵坐标 */
  const menuY = ref(0)
  /** 当前上下文数据（右键时传入的列表项） */
  const menuContext = ref<T | null>(null) as unknown as { value: T | null }

  /**
   * 打开菜单。
   *
   * @param e - 鼠标事件（用于取坐标）
   * @param ctx - 上下文数据
   */
  function openMenu(e: MouseEvent, ctx: T): void {
    e.preventDefault()
    menuContext.value = ctx
    menuX.value = e.clientX
    menuY.value = e.clientY
    menuVisible.value = true
  }

  /**
   * 关闭菜单。
   */
  function closeMenu(): void {
    menuVisible.value = false
    menuContext.value = null
  }

  /**
   * 生成 @contextmenu 事件处理器（对标 NRCS `.on("contextmenu", "a", ...)`）。
   *
   * @param ctx - 与该列表项绑定的上下文数据
   * @returns 可绑定到 @contextmenu 的事件处理器
   */
  function onContextMenu(ctx: T): (e: MouseEvent) => void {
    return (e: MouseEvent) => openMenu(e, ctx)
  }

  return {
    menuVisible,
    menuX,
    menuY,
    menuContext,
    openMenu,
    closeMenu,
    onContextMenu,
  }
}

export type { ContextMenuItem }
