<template>
  <el-menu
    :default-active="activeMenu"
    :default-openeds="defaultOpeneds"
    :collapse="isCollapsed"
    :unique-opened="true"
    :router="true"
    :collapse-transition="false"
  >
    <template v-for="route in menuRoutes" :key="route.path">
      <!-- 无子菜单 -->
      <el-menu-item
        v-if="!route.children || route.children.length === 0"
        :index="route.path"
        :route="{ name: route.name }"
      >
        <el-icon v-if="route.meta?.icon">
          <component :is="route.meta.icon" />
        </el-icon>
        <template #title>
          <span class="menu-title">{{ route.meta?.title }}</span>
        </template>
      </el-menu-item>

      <!-- 有子菜单 -->
      <el-sub-menu
        v-else
        :index="route.path"
      >
        <template #title>
          <el-icon v-if="route.meta?.icon">
            <component :is="route.meta.icon" />
          </el-icon>
          <span class="menu-title">{{ route.meta?.title }}</span>
        </template>

        <el-menu-item
          v-for="child in route.children"
          :key="child.path"
          :index="`${route.path}/${child.path}`"
          :route="{ name: child.name }"
        >
          <el-icon v-if="child.meta?.icon">
            <component :is="child.meta.icon" />
          </el-icon>
          <template #title>
            <span class="menu-title">{{ child.meta?.title }}</span>
          </template>
        </el-menu-item>
      </el-sub-menu>
    </template>
  </el-menu>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import { useAppStore } from '@/stores/modules/app.store'
import type { AppRouteRecordRaw } from '@/types/router'

const props = defineProps<{
  routes: AppRouteRecordRaw[]
  collapsed?: boolean
}>()

const route = useRoute()
const appStore = useAppStore()

// 使用 store 的折叠状态，除非 props 传入
const isCollapsed = computed(() => props.collapsed ?? appStore.isSidebarCollapsed)

// 当前激活菜单
const activeMenu = computed(() => route.path)

// 默认展开的子菜单（根据路由自动计算）
const defaultOpeneds = computed(() => {
  const path = route.path
  const segments = path.split('/').filter(Boolean)
  const opened: string[] = []

  // 逐步构建路径，展开所有父级菜单
  for (let i = 1; i <= segments.length - 1; i++) {
    opened.push('/' + segments.slice(0, i).join('/'))
  }

  return opened
})

// 只显示需要认证的菜单
const menuRoutes = computed(() => {
  return props.routes.filter(r =>
    r.meta?.requireAuth !== false &&
    !r.meta?.hidden
  )
})
</script>

<style scoped lang="scss">
.el-menu {
  height: 100%;
  border-right: none;

  &:not(.el-menu--collapse) {
    width: 200px;
  }

  &.el-menu--collapse {
    width: 64px;
  }

  .menu-title {
    font-size: 14px;
  }

  .el-menu-item,
  .el-sub-menu__title {
    height: 50px;
    line-height: 50px;

    .el-icon {
      margin-right: 8px;
      font-size: 18px;
    }
  }

  .el-sub-menu .el-menu-item {
    min-width: 200px;
    padding-left: 48px !important;
  }
}
</style>
