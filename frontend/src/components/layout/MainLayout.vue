<template>
  <div class="shell" :class="{ 'shell--nav-collapsed': isCollapsed }">
    <Header :show-breadcrumb="true" class="topbar" @toggle-sidebar="toggleSidebar" />
    <Sidebar class="nav" @open-forging="openForgingModal" />
    <main class="content">
      <router-view v-slot="{ Component }">
        <transition name="dashboard-enter" mode="out-in">
          <component :is="Component" />
        </transition>
      </router-view>
    </main>

    <!-- 共享的 ForgingModal（Sidebar 锻造状态点击触发） -->
    <ForgingModal v-model:visible="showForgingModal" :mode="forgingModalMode" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import Header from './Header.vue'
import Sidebar from './Sidebar.vue'
import ForgingModal from '@/components/modals/ForgingModal.vue'
import { useForging } from '@/composables/useForging'
import { useAppStore } from '@/stores/app'

const appStore = useAppStore()
const isCollapsed = computed(() => appStore.isSidebarCollapsed)

const { forgingStatus, checkForgingPreconditions } = useForging()

/** 共享 ForgingModal 显示状态（由 Sidebar 锻造状态点击触发） */
const showForgingModal = ref(false)
const forgingModalMode = ref<'start' | 'stop'>('start')

const toggleSidebar = () => {
  appStore.toggleSidebar()
}

/**
 * 打开锻造 Modal（由 Sidebar 锻造状态指示器点击触发）。
 * 先执行前置校验，通过后根据当前状态显示 start/stop modal。
 */
function openForgingModal(): void {
  const error = checkForgingPreconditions()
  if (error) {
    // 前置校验失败时不打开 modal，由调用方（Sidebar）处理提示
    return
  }
  forgingModalMode.value = forgingStatus.value === 'forging' ? 'stop' : 'start'
  showForgingModal.value = true
}
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;

.shell {
  --shell-pad: 16px;
  --shell-gap: 16px;
  --shell-nav-width: #{$sidebar-width};
  --shell-topbar-height: #{$header-height};
  --shell-focus-duration: 200ms;
  --shell-focus-ease: var(--ease-out);

  height: 100vh;
  display: grid;
  grid-template-columns: var(--shell-nav-width) minmax(0, 1fr);
  grid-template-rows: var(--shell-topbar-height) 1fr;
  grid-template-areas:
    "topbar topbar"
    "nav content";
  gap: 0;
  animation: dashboard-enter 0.4s $ease-out;
  transition: grid-template-columns var(--shell-focus-duration) $ease-out;
  overflow: hidden;
  background: $bg;

  &--nav-collapsed {
    grid-template-columns: 0px minmax(0, 1fr);

    .nav {
      width: 0;
      padding: 0;
      border-width: 0;
      overflow: hidden;
      pointer-events: none;
      opacity: 0;
    }
  }

  @supports (height: 100dvh) {
    height: 100dvh;
  }
}

.topbar {
  grid-area: topbar;
  position: sticky;
  top: 0;
  z-index: $z-header;
}

.nav {
  grid-area: nav;
  overflow-y: auto;
  overflow-x: hidden;
  padding: $space-md ($space-sm + 4px);
  background: $bg;
  scrollbar-width: none;
  transition:
    width var(--shell-focus-duration) $ease-out,
    padding var(--shell-focus-duration) $ease-out,
    opacity var(--shell-focus-duration) $ease-out;
  min-height: 0;

  &::-webkit-scrollbar {
    display: none;
  }
}

.content {
  grid-area: content;
  padding: 12px 16px 32px;
  display: block;
  min-height: 0;
  overflow-y: auto;
  overflow-x: hidden;
  background: $bg-content;

  &::-webkit-scrollbar {
    width: 8px;
    height: 8px;
  }

  &::-webkit-scrollbar-track {
    background: transparent;
  }

  &::-webkit-scrollbar-thumb {
    background: $border-default;
    border-radius: $radius-full;
  }

  &::-webkit-scrollbar-thumb:hover {
    background: $border-strong;
  }
}

@keyframes dashboard-enter {
  from {
    opacity: 0;
    transform: translateY(12px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.dashboard-enter-active,
.dashboard-enter-leave-active {
  transition: opacity $duration-normal ease, transform $duration-normal ease;
}

.dashboard-enter-from {
  opacity: 0;
  transform: translateY(12px);
}

.dashboard-enter-leave-to {
  opacity: 0;
  transform: translateY(-12px);
}

@media (max-width: 1100px) {
  .shell {
    --shell-pad: 12px;
    --shell-gap: 12px;
    grid-template-columns: 1fr;
    grid-template-rows: auto auto 1fr;
    grid-template-areas:
      "topbar"
      "nav"
      "content";
  }

  .nav {
    position: static;
    max-height: none;
    display: flex;
    gap: 6px;
    overflow-x: auto;
    border-right: none;
    border-bottom: 1px solid $border-default;
    padding: 10px 14px;
    background: $bg;
  }

  .topbar {
    position: static;
    padding: 12px 14px;
    gap: 10px;
  }

  .content {
    padding: 12px 14px 32px;
  }
}
</style>
