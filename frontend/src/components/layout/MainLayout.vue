<template>
  <el-container class="main-layout">
    <!-- 顶部导航栏 -->
    <Header :show-breadcrumb="true" class="layout-header" />

    <el-container class="main-container">
      <!-- 侧边栏 -->
      <Sidebar
        :routes="menuRoutes"
        class="layout-sidebar"
      />

      <!-- 主内容区 -->
      <el-main class="layout-main">
        <router-view v-slot="{ Component }">
          <transition name="fade" mode="out-in">
            <component :is="Component" />
          </transition>
        </router-view>
      </el-main>
    </el-container>
  </el-container>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAppStore } from '@/stores/modules/app.store'
import { useAccountStore } from '@/stores/modules/account.store'
import Header from './Header.vue'
import Sidebar from './Sidebar.vue'
import type { AppRouteRecordRaw } from '@/types/router'

const router = useRouter()
const appStore = useAppStore()
const accountStore = useAccountStore()

// 计算当前用户可访问的路由菜单
const menuRoutes = computed<AppRouteRecordRaw[]>(() => {
  const allRoutes = router.getRoutes()
  const userRoles = accountStore.userRoles

  return allRoutes.filter(route => {
    // 只显示需要认证的路由
    if (route.meta?.requireAuth === false) {
      return false
    }

    // 检查隐藏标记
    if (route.meta?.hidden) {
      return false
    }

    // 检查权限（如果没有meta.roles，则显示给所有登录用户）
    if (route.meta?.roles && userRoles.length > 0) {
      return userRoles.some(role => route.meta?.roles?.includes(role))
    }

    return true
  })
})
</script>

<style scoped lang="scss">
.main-layout {
  height: 100vh;
  width: 100vw;
}

.main-container {
  height: calc(100vh - 60px);
}

.layout-sidebar {
  background: var(--sidebar-bg, #fff);
  border-right: 1px solid var(--border-color, #e4e7ed);
  transition: width 0.3s ease;
  overflow-y: auto;

  &.el-menu--collapse {
    width: 64px;
  }
}

.layout-main {
  background: var(--content-bg, #f2f3f5);
  padding: 16px;
  overflow-y: auto;
}

.layout-header {
  flex-shrink: 0;
}

// 页面切换动画
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

// 侧边栏暗色模式
html.dark {
  .layout-sidebar {
    background: #1f2937;
    border-right-color: #374151;
  }
}
</style>
