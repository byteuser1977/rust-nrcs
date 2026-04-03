import type { RouteRecordRaw } from 'vue-router'
import type { UserRole } from './enums'

/**
 * 路由元信息扩展
 */
export interface RouteMeta {
  title: string
  requiresAuth?: boolean
  roles?: UserRole[]
  breadcrumb?: BreadcrumbItem[]
  icon?: string  // Element Plus 图标名称
  keepAlive?: boolean  // 是否缓存
}

/**
 * 面包屑项
 */
export interface BreadcrumbItem {
  path: string
  meta: {
    title: string
  }
}

/**
 * 路由守卫钩子参数
 */
export interface NavigationGuard {
  to: RouteLocationNormalizedLoaded
  from: RouteLocationNormalizedLoaded
  next: (to?: RouteLocationRaw | false | (() => any)) => void
}

/**
 * 路由位置（Vue Router 内部类型，简化版）
 */
export interface RouteLocationNormalizedLoaded {
  fullPath: string
  path: string
  params: Record<string, string>
  query: Record<string, string>
  name: string | null
  meta: RouteMeta | undefined
  matched: Array<{ meta: RouteMeta }>
}

/**
 * 路由定义增强版
 */
export interface AppRouteRecordRaw extends Omit<RouteRecordRaw, 'meta'> {
  meta: RouteMeta
  children?: AppRouteRecordRaw[]
}
