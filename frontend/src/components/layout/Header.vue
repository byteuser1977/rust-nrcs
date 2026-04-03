<template>
  <header class="app-header">
    <div class="header-left">
      <!-- 侧边栏折叠按钮 -->
      <el-button
        class="collapse-btn"
        text
        @click="toggleSidebar"
      >
        <el-icon :size="22">
          <Fold v-if="!isCollapsed" />
          <Expand v-else />
        </el-icon>
      </el-button>

      <!-- 面包屑 -->
      <Breadcrumb v-if="showBreadcrumb" :routes="breadcrumbRoutes" />
    </div>

    <div class="header-right">
      <!-- 全局搜索（可选） -->
      <el-input
        v-if="showSearch"
        v-model="searchQuery"
        placeholder="搜索..."
        :prefix-icon="Search"
        class="header-search"
        clearable
        @clear="handleSearch('')"
        @keyup.enter="handleSearch(searchQuery)"
      />

      <!-- 全屏切换 -->
      <el-tooltip content="全屏" placement="bottom">
        <el-button
          class="header-btn"
          text
          @click="toggleFullscreen"
        >
          <el-icon :size="18">
            <FullScreen v-if="!isFullscreen" />
            <Aim v-else />
          </el-icon>
        </el-button>
      </el-tooltip>

      <!-- 主题切换 -->
      <el-tooltip :content="isDark ? '浅色模式' : '深色模式'" placement="bottom">
        <el-button
          class="header-btn"
          text
          @click="toggleTheme"
        >
          <el-icon :size="18">
            <Sunny v-if="isDark" />
            <Moon v-else />
          </el-icon>
        </el-button>
      </el-tooltip>

      <!-- 通知中心 -->
      <el-badge
        :value="notificationCount"
        :max="99"
        :hidden="notificationCount === 0"
        class="notification-badge"
      >
        <el-tooltip content="通知" placement="bottom">
          <el-button
            class="header-btn"
            text
            @click="showNotifications = true"
          >
            <el-icon :size="18"><Bell /></el-icon>
          </el-button>
        </el-tooltip>
      </el-badge>

      <!-- 语言切换 -->
      <el-dropdown v-if="showLocaleSelector" @command="changeLocale">
        <el-button class="header-btn" text>
          <el-icon><Globe /></el-icon>
          <span class="locale-label">{{ currentLocale }}</span>
        </el-button>
        <template #dropdown>
          <el-dropdown-menu>
            <el-dropdown-item
              v-for="locale in supportedLocales"
              :key="locale.value"
              :command="locale.value"
              :disabled="locale.value === currentLocale"
            >
              {{ locale.label }}
            </el-dropdown-item>
          </el-dropdown-menu>
        </template>
      </el-dropdown>

      <!-- 用户下拉菜单 -->
      <el-dropdown @command="handleCommand">
        <span class="user-dropdown">
          <el-avatar
            :size="32"
            :src="userAvatar"
            class="user-avatar"
          />
          <span class="username">{{ userName }}</span>
          <el-icon class="arrow-icon"><ArrowDown /></el-icon>
        </span>
        <template #dropdown>
          <el-dropdown-menu>
            <el-dropdown-item command="profile">
              <el-icon><User /></el-icon>
              个人资料
            </el-dropdown-item>
            <el-dropdown-item command="settings">
              <el-icon><Setting /></el-icon>
              设置
            </el-dropdown-item>
            <el-dropdown-item divided command="logout">
              <el-icon><SwitchButton /></el-icon>
              退出登录
            </el-dropdown-item>
          </el-dropdown-menu>
        </template>
      </el-dropdown>
    </div>

    <!-- 通知面板 -->
    <el-drawer
      v-model="showNotifications"
      title="通知中心"
      direction="rtl"
      size="360px"
    >
      <template #default>
        <div v-if="notifications.length === 0" class="empty-notifications">
          <el-empty description="暂无通知" />
        </div>
        <el-timeline v-else>
          <el-timeline-item
            v-for="notification in notifications"
            :key="notification.id"
            :timestamp="notification.time"
            :type="notification.type"
            :color="notification.color"
          >
            <div class="notification-item" @click="handleNotificationClick(notification)">
              <div class="notification-title">{{ notification.title }}</div>
              <div class="notification-content">{{ notification.content }}</div>
            </div>
          </el-timeline-item>
        </el-timeline>
      </template>
    </el-drawer>
  </header>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useAppStore } from '@/stores/modules/app.store'
import { useAccountStore } from '@/stores/modules/account.store'
import { ElMessageBox } from 'element-plus'
import Breadcrumb from './Breadcrumb.vue'

interface Props {
  showBreadcrumb?: boolean
  showSearch?: boolean
  showLocaleSelector?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  showBreadcrumb: true,
  showSearch: false,
  showLocaleSelector: true
})

const emit = defineEmits<{
  search: [query: string]
  sidebarToggle: []
}>()

const router = useRouter()
const { locale } = useI18n()
const appStore = useAppStore()
const accountStore = useAccountStore()

// 状态
const searchQuery = ref('')
const showNotifications = ref(false)
const notifications = ref([
  // TODO: 从 API 加载通知
])

// 计算属性
const isCollapsed = computed(() => appStore.isSidebarCollapsed)
const isDark = computed(() => appStore.isDark)
const userName = computed(() => accountStore.displayName || 'Admin')
const userAvatar = computed(() => accountStore.userInfo?.avatar || '')
const notificationCount = computed(() => notifications.value.length)
const currentLocale = computed(() => locale.value)
const supportedLocales = computed(() => [
  { label: '简体中文', value: 'zh-CN' },
  { label: 'English', value: 'en-US' }
])

// 面包屑路由
const breadcrumbRoutes = computed(() => {
  const matched = router.currentRoute.value.matched
  return matched.map(route => ({
    path: route.path,
    title: route.meta?.title || '未命名'
  }))
})

// 方法
const toggleSidebar = () => {
  appStore.toggleSidebar()
  emit('sidebarToggle')
}

const toggleFullscreen = () => {
  if (!document.fullscreenElement) {
    document.documentElement.requestFullscreen()
  } else {
    document.exitFullscreen()
  }
}

const toggleTheme = () => {
  appStore.toggleTheme()
}

const changeLocale = (localeCode: string) => {
  locale.value = localeCode
  // 可以保存到 localStorage
  localStorage.setItem('locale', localeCode)
}

const handleSearch = (query: string) => {
  emit('search', query)
}

const handleCommand = async (command: string) => {
  switch (command) {
    case 'profile':
      await router.push('/account/profile')
      break
    case 'settings':
      // 打开设置对话框
      break
    case 'logout':
      try {
        await ElMessageBox.confirm('确定要退出登录吗？', '提示', {
          type: 'warning'
        })
        await accountStore.logout()
        await router.push('/login')
      } catch {
        // 用户取消
      }
      break
  }
}

const handleNotificationClick = (notification: any) => {
  // 标记为已读，跳转到相关页面
  showNotifications.value = false
}

// 加载通知
const loadNotifications = async () => {
  // TODO: 调用通知 API
}

// 监听 locale 从本地存储恢复
watch(
  () => props.showLocaleSelector,
  (val) => {
    if (val) {
      const savedLocale = localStorage.getItem('locale')
      if (savedLocale && savedLocale !== locale.value) {
        locale.value = savedLocale
      }
    }
  },
  { immediate: true }
)

defineExpose({
  loadNotifications
})
</script>

<style scoped lang="scss">
.app-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 60px;
  padding: 0 16px;
  background: #fff;
  border-bottom: 1px solid #e4e7ed;
  box-shadow: 0 1px 4px rgba(0, 21, 41, 0.08);

  .header-left {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 8px;

    .header-btn {
      padding: 8px;
      color: #606266;

      &:hover {
        color: #409eff;
        background: #ecf5ff;
      }
    }

    .header-search {
      width: 200px;
      margin-right: 8px;
    }

    .locale-label {
      margin-left: 4px;
      font-size: 14px;
    }

    .notification-badge {
      .el-badge__content {
        transform: translateY(-50%) translateX(50%);
      }
    }

    .user-dropdown {
      display: flex;
      align-items: center;
      gap: 8px;
      margin-left: 8px;
      padding: 4px 8px;
      border-radius: 4px;
      cursor: pointer;
      transition: background 0.3s;

      &:hover {
        background: #f5f7fa;
      }

      .username {
        font-size: 14px;
        color: #303133;
        max-width: 100px;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
      }

      .arrow-icon {
        font-size: 12px;
        color: #909399;
      }
    }
  }
}

.empty-notifications {
  padding: 40px 0;
}

.notification-item {
  padding: 8px 12px;
  background: #f5f7fa;
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.3s;

  &:hover {
    background: #ecf5ff;
  }

  .notification-title {
    font-weight: 500;
    margin-bottom: 4px;
    color: #303133;
  }

  .notification-content {
    font-size: 13px;
    color: #606266;
    line-height: 1.4;
  }
}

// 深色模式
html.dark {
  .app-header {
    background: #1f2937;
    border-bottom-color: #374151;

    .header-btn {
      color: #d1d5db;

      &:hover {
        color: #409eff;
        background: rgba(64, 158, 255, 0.1);
      }
    }

    .user-dropdown {
      &:hover {
        background: rgba(255, 255, 255, 0.1);
      }

      .username {
        color: #d1d5db;
      }
    }
  }

  .notification-item {
    background: rgba(255, 255, 255, 0.05);

    &:hover {
      background: rgba(64, 158, 255, 0.1);
    }

    .notification-title,
    .notification-content {
      color: #d1d5db;
    }
  }
}
</style>
