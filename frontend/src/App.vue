<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useAppStore } from '@/stores/modules/app.store'
import { useUiStore } from '@/stores/modules/ui.store'
import { useAccountStore } from '@/stores/modules/account.store'
import { ElMessage, ElMessageBox } from 'element-plus'

const route = useRoute()
const router = useRouter()
const { locale, fallbackLocale } = useI18n()
const appStore = useAppStore()
const uiStore = useUiStore()
const accountStore = useAccountStore()

// 计算属性
const isAuthenticated = computed(() => accountStore.isLoggedIn)
const currentRouteName = computed(() => route.name)
const sidebarCollapsed = computed(() => uiStore.sidebarCollapsed)

// 切换侧边栏
const toggleSidebar = () => {
  uiStore.toggleSidebar()
}

// 登出
const handleLogout = async () => {
  try {
    await ElMessageBox.confirm('确定要退出登录吗？', '提示', {
      confirmButtonText: '退出',
      cancelButtonText: '取消',
      type: 'warning'
    })

    await accountStore.logout()
    ElMessage.success('已退出登录')
    router.push({ name: 'login' })
  } catch {
    // 用户取消操作
  }
}

// 切换语言
const handleLanguageChange = async (lang: string) => {
  try {
    await loadLanguageAsync(lang)
    locale.value = lang
    appStore.setLanguage(lang)
    ElMessage.success(`已切换至 ${fallbackLocale === lang ? 'English' : '中文'}`)
  } catch (error) {
    ElMessage.error('语言切换失败')
  }
}
</script>

<template>
  <el-config-provider :locale="locale === 'zh-CN' ? undefined : {}">
    <div class="app-container" :class="{ 'sidebar-collapsed': sidebarCollapsed }">
      <!-- 顶部导航栏 -->
      <header class="app-header">
        <div class="header-left">
          <el-button
            link
            @click="toggleSidebar"
            class="collapse-btn"
          >
            <el-icon :size="24">
              <component :is="sidebarCollapsed ? 'Expand' : 'Fold'" />
            </el-icon>
          </el-button>

          <div class="logo">
            <img src="@/assets/images/logo.svg" alt="NRCS" class="logo-img" />
            <span v-show="!sidebarCollapsed" class="logo-text">NRCS</span>
          </div>
        </div>

        <div class="header-center">
          <!-- 顶部导航标签（可选） -->
          <nav class="top-nav">
            <router-link
              v-for="item in route.meta?.breadcrumb || []"
              :key="item.path"
              :to="item.path"
              class="nav-item"
            >
              {{ item.meta?.title }}
            </router-link>
          </nav>
        </div>

        <div class="header-right">
          <!-- 语言切换 -->
          <el-dropdown @command="handleLanguageChange">
            <el-button link>
              <el-icon><Globe /></el-icon>
            </el-button>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item
                  command="zh-CN"
                  :disabled="locale === 'zh-CN'"
                >
                  简体中文
                </el-dropdown-item>
                <el-dropdown-item
                  command="en-US"
                  :disabled="locale === 'en-US'"
                >
                  English
                </el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>

          <!-- 用户信息 -->
          <el-dropdown v-if="isAuthenticated" @command="handleLogout">
            <div class="user-info">
              <el-avatar :size="32" :src="accountStore.userInfo?.avatar">
                {{ accountStore.userInfo?.name?.charAt(0) || 'U' }}
              </el-avatar>
              <span class="user-name">{{ accountStore.userInfo?.name || 'User' }}</span>
            </div>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item command="logout">
                  <el-icon><SwitchButton /></el-icon>
                  退出登录
                </el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>

          <!-- 未登录显示登录按钮 -->
          <router-link v-else to="/login">
            <el-button type="primary">登录</el-button>
          </router-link>
        </div>
      </header>

      <!-- 主内容区 -->
      <div class="app-main">
        <!-- 侧边栏菜单（桌面端） -->
        <aside class="sidebar" v-show="!sidebarCollapsed">
          <el-menu
            :default-active="String(route.name)"
            router
            mode="vertical"
            :collapse="sidebarCollapsed"
          >
            <el-menu-item index="dashboard">
              <el-icon><Odometer /></el-icon>
              <template #title>{{ $t('menu.dashboard') }}</template>
            </el-menu-item>

            <el-menu-item index="account">
              <el-icon><User /></el-icon>
              <template #title>{{ $t('menu.account') }}</template>
            </el-menu-item>

            <el-menu-item index="transaction">
              <el-icon><TrendCharts /></el-icon>
              <template #title>{{ $t('menu.transaction') }}</template>
            </el-menu-item>

            <el-menu-item index="contract">
              <el-icon><Files /></el-icon>
              <template #title>{{ $t('menu.contract') }}</template>
            </el-menu-item>

            <el-menu-item index="node">
              <el-icon><Monitor /></el-icon>
              <template #title>{{ $t('menu.node') }}</template>
            </el-menu-item>
          </el-menu>
        </aside>

        <!-- 内容区域 -->
        <main class="content">
          <router-view v-slot="{ Component }">
            <transition name="fade" mode="out-in">
              <component :is="Component" />
            </transition>
          </router-view>
        </main>
      </div>
    </div>
  </el-config-provider>
</template>

<style lang="scss">
@use "@/assets/styles/variables" as *;

.app-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;

  &.sidebar-collapsed {
    .sidebar {
      width: 64px;
    }
    .content {
      margin-left: 64px;
    }
  }
}

// 头部样式
.app-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 60px;
  padding: 0 16px;
  background: $header-bg;
  border-bottom: 1px solid $border-color;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  z-index: 100;

  .header-left {
    display: flex;
    align-items: center;
    gap: 12px;

    .logo {
      display: flex;
      align-items: center;
      gap: 8px;

      .logo-img {
        height: 32px;
        width: auto;
      }

      .logo-text {
        font-size: 18px;
        font-weight: 600;
        color: $primary-color;
      }
    }
  }

  .header-center {
    flex: 1;
    display: flex;
    justify-content: center;

    .top-nav {
      display: flex;
      gap: 16px;

      .nav-item {
        color: $text-secondary;
        text-decoration: none;
        font-size: 14px;
        padding: 8px 12px;
        border-radius: 4px;
        transition: all 0.2s;

        &:hover,
        &.router-link-active {
          color: $primary-color;
          background: rgba($primary-color, 0.1);
        }
      }
    }
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 16px;

    .user-info {
      display: flex;
      align-items: center;
      gap: 8px;
      cursor: pointer;
      padding: 4px 8px;
      border-radius: 20px;
      transition: background 0.2s;

      &:hover {
        background: rgba(255, 255, 255, 0.1);
      }

      .user-name {
        font-size: 14px;
        color: $text-primary;
      }
    }
  }
}

// 主体布局
.app-main {
  display: flex;
  flex: 1;
  overflow: hidden;

  .sidebar {
    width: 240px;
    background: $sidebar-bg;
    border-right: 1px solid $border-color;
    transition: width 0.3s ease;
    overflow-y: auto;

    .el-menu {
      border-right: none;
      background: transparent;

      .el-menu-item {
        color: $text-secondary;

        &:hover {
          background: rgba($primary-color, 0.05);
        }

        &.is-active {
          color: $primary-color;
          background: rgba($primary-color, 0.1);
          border-right: 3px solid $primary-color;
        }
      }
    }
  }

  .content {
    flex: 1;
    margin-left: 240px;
    padding: 24px;
    overflow-y: auto;
    background: $content-bg;
    transition: margin-left 0.3s ease;
  }
}

// 页面切换动画
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
