<template>
  <el-container class="sub-layout">
    <!-- 顶部Tabs导航（可选） -->
    <el-header v-if="showTabs" class="tabs-header" height="48px">
      <el-tabs
        v-model="activeTab"
        type="card"
        closable
        @tab-remove="removeTab"
        @tab-click="handleTabClick"
      >
        <el-tab-pane
          v-for="tab in tabs"
          :key="tab.path"
          :label="tab.title"
          :name="tab.path"
        />
      </el-tabs>
    </el-header>

    <!-- 面包屑导航 -->
    <el-header class="breadcrumb-header" height="52px">
      <div class="breadcrumb-container">
        <el-breadcrumb separator="/">
          <el-breadcrumb-item :to="{ path: '/' }">首页</el-breadcrumb-item>
          <el-breadcrumb-item v-for="(item, index) in breadcrumbs" :key="index">
            {{ item.name }}
          </el-breadcrumb-item>
        </el-breadcrumb>

        <div class="header-actions">
          <slot name="header-actions" />
        </div>
      </div>
    </el-header>

    <!-- 主内容区 -->
    <el-main class="main-content">
      <router-view v-slot="{ Component }">
        <transition name="fade" mode="out-in">
          <component :is="Component" />
        </transition>
      </router-view>
    </el-main>
  </el-container>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAppStore } from '@/stores/app'

const route = useRoute()
const router = useRouter()
const appStore = useAppStore()

// Tabs 管理
const tabs = ref<Array<{ path: string; title: string }>>([])
const activeTab = ref('')

// 显示 tabs
const showTabs = computed(() => tabs.value.length > 1)

// 计算面包屑
const breadcrumbs = computed(() => {
  const matched = route.matched.filter(item => item.meta?.title)
  return matched.map(item => ({
    name: item.meta?.title || '',
    path: item.path
  }))
})

// 监听路由变化
watch(
  () => route.path,
  (newPath) => {
    activeTab.value = newPath

    // 添加或更新 tab
    const title = route.meta?.title || route.name?.toString() || 'Untitled'
    const existingTab = tabs.value.find(tab => tab.path === newPath)

    if (!existingTab) {
      tabs.value.push({ path: newPath, title })
    }
  },
  { immediate: true }
)

// 移除 tab
const removeTab = (targetPath: string) => {
  const index = tabs.value.findIndex(tab => tab.path === targetPath)
  if (index > -1) {
    tabs.value.splice(index, 1)

    // 如果关闭的是当前激活的 tab，跳转到最后一个 tab
    if (activeTab.value === targetPath) {
      const lastTab = tabs.value[Math.max(0, index - 1)]
      if (lastTab) {
        router.push(lastTab.path)
      } else {
        router.push('/')
      }
    }
  }
}

// Tab 点击
const handleTabClick = (tab: any) => {
  router.push(tab.paneName)
}
</script>

<style lang="scss" scoped>
.sub-layout {
  height: 100%;
}

.tabs-header {
  padding: 0 16px;
  background: #fff;
  border-bottom: 1px solid #e4e7ed;

  :deep(.el-tabs) {
    height: 100%;

    .el-tabs__header {
      margin: 0;
      border-bottom: none;
    }

    .el-tabs__nav-wrap::after {
      display: none;
    }
  }
}

.breadcrumb-header {
  background: #fff;
  border-bottom: 1px solid #e4e7ed;
  padding: 0 16px;

  .breadcrumb-container {
    display: flex;
    justify-content: space-between;
    align-items: center;
    height: 100%;
  }

  .header-actions {
    display: flex;
    gap: 8px;
  }
}

.main-content {
  background: #f2f3f5;
  padding: 16px;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

html.dark {
  .tabs-header,
  .breadcrumb-header {
    background: #1f2937;
    border-bottom-color: #374151;
  }
}
</style>
