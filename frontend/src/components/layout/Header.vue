<template>
  <header class="nrcs-header">
    <div class="header-left">
      <button class="collapse-toggle" @click="toggleSidebar" :title="isCollapsed ? '展开' : '折叠'">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
          <path v-if="!isCollapsed" d="M3 12h18M3 6h18M3 18h18" stroke="#94a3b8" stroke-width="1.8" stroke-linecap="round"/>
          <path v-else d="M4 6h16v12H4z M15 3l6 6-6 6" stroke="#94a3b8" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      </button>
      <Breadcrumb v-if="showBreadcrumb" :routes="breadcrumbRoutes" />
    </div>

    <div class="header-actions">
      <div class="action-group">
        <button class="action-btn action-btn--primary" @click="showSendMoneyDialog = true" :title="t('header.sendNRC')">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none"><line x1="22" y1="2" x2="11" y2="13" stroke="#0b1121" stroke-width="2" stroke-linecap="round"/><polygon points="22,2 15,9 11,16" fill="#0b1121"/></svg>
          <span class="action-label">{{ t('header.sendNRC') }}</span>
        </button>

        <button class="action-btn" @click="showSendMessageDialog = true" :title="t('header.sendMessage')">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none"><path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z" stroke="#94a3b8" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/></svg>
        </button>
      </div>

      <div class="divider-v"></div>

      <button class="icon-btn" @click="navigateTo('/settings/blocks')" :title="t('header.clientStatus')">
        <svg width="17" height="17" viewBox="0 0 24 24" fill="none"><rect x="2" y="3" width="20" height="14" rx="2" stroke="#94a3b8" stroke-width="1.8"/><line x1="8" y1="21" x2="16" y2="21" stroke="#94a3b8" stroke-width="1.8" stroke-linecap="round"/><line x1="12" y1="17" x2="12" y2="21" stroke="#94a3b8" stroke-width="1.8"/></svg>
      </button>

      <button class="icon-btn icon-btn--badge" @click="navigateTo('/dashboard/transactions')" :title="t('header.unconfirmedTxs')">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none"><circle cx="12" cy="12" r="10" stroke="#94a3b8" stroke-width="1.8"/><polyline points="12,6 12,12 16,14" stroke="#94a3b8" stroke-width="1.8" stroke-linecap="round"/></svg>
        <span class="btn-badge" v-if="unconfirmedCount > 0">{{ unconfirmedCount > 99 ? '99+' : unconfirmedCount }}</span>
      </button>

      <button class="icon-btn icon-btn--badge" @click="showNotifications = true" :title="t('header.notifications')">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none"><path d="M18 8A6 6 0 006 8c0 7-3 9-3 9h18c0-2-3-4-3-9zM13.73 21a2 2 0 01-3.46 0" stroke="#94a3b8" stroke-width="1.8" stroke-linecap="round"/></svg>
        <span class="btn-badge btn-badge--pulse" v-if="notificationCount > 0">{{ notificationCount > 99 ? '99+' : notificationCount }}</span>
      </button>

      <button class="icon-btn" @click="navigateTo('/contacts')" :title="t('header.contacts')">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none"><path d="M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2" stroke="#94a3b8" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/><circle cx="12" cy="7" r="4" stroke="#94a3b8" stroke-width="1.8"/></svg>
      </button>

      <el-dropdown trigger="click" @command="handleSettingsCommand">
        <button class="icon-btn" :title="t('header.notifications')">
          <svg width="17" height="17" viewBox="0 0 24 24" fill="none"><circle cx="12" cy="12" r="3" stroke="#94a3b8" stroke-width="1.8"/><path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.82 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-2 2 2 2 0 01-2-2v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010 2.82l.06.06a1.65 1.65 0 00.33 1.82 1.65 1.65 0 001 1.51V21a2 2 0 002 2 2 2 0 002-2v-.09a1.65 1.65 0 001-1.51 1.65 1.65 0 00.33-1.82l.06-.06a2 2 0 012.82 0 2 2 0 010-2.82l-.06-.06A1.65 1.65 0 009 4.68a1.65 1.65 0 00-1-1.51V3a2 2 0 012-2 2 2 0 002 2v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 00-.33 1.82l-.06.06a2 2 0 01-2.82 0z" stroke="#94a3b8" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/></svg>
        </button>
        <template #dropdown>
          <el-dropdown-menu>
            <template #default>
              <!-- Group: Network -->
              <div class="dd-group-label">{{ t('header.clientStatus') }}</div>
              <el-dropdown-item command="blocks">{{ t('settings.blocks') }}</el-dropdown-item>
              <el-dropdown-item command="peers">{{ t('settings.peers') }}</el-dropdown-item>
              <el-dropdown-item command="generators">{{ t('settings.generators') }}</el-dropdown-item>
              <el-dropdown-item command="scheduled-transactions">{{ t('settings.scheduledTransactions') }}</el-dropdown-item>
              <el-dropdown-item command="monitors">{{ t('settings.monitors') }}</el-dropdown-item>
              <!-- Divider -->
              <div class="dd-divider"></div>
              <el-dropdown-item command="plugins">{{ t('settings.plugins') }}</el-dropdown-item>
              <!-- Divider -->
              <div class="dd-divider"></div>
              <div class="dd-group-label">{{ t('header.accountSettings') }}</div>
              <el-dropdown-item command="account-settings">{{ t('settings.accountSettings') }}</el-dropdown-item>
              <!-- Divider -->
              <div class="dd-divider"></div>
              <div class="dd-group-label">Tools</div>
              <el-dropdown-item command="generate-token">{{ t('settings.tokenGenerator') }}</el-dropdown-item>
              <el-dropdown-item command="generate-hallmark">{{ t('settings.hallmarkGenerator') }}</el-dropdown-item>
              <el-dropdown-item command="calculate-hash">{{ t('settings.hashCalculator') }}</el-dropdown-item>
              <el-dropdown-item command="transaction-operations">{{ t('settings.transactionOperations') }}</el-dropdown-item>
              <!-- Divider -->
              <div class="dd-divider"></div>
              <el-dropdown-item command="api-console">{{ t('settings.apiConsole') }} →</el-dropdown-item>
            </template>
          </el-dropdown-menu>
        </template>
      </el-dropdown>

      <el-dropdown trigger="click" @command="handleLogoutCommand">
        <button class="user-chip" :title="t('header.logout')">
          <div class="chip-avatar">N</div>
          <svg width="10" height="10" viewBox="0 0 24 24" fill="none"><path d="M6 9l6 6 6-6" stroke="#94a3b8" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/></svg>
        </button>
        <template #dropdown>
          <el-dropdown-menu>
            <template #default>
              <el-dropdown-item command="logout">{{ t('header.logout') }}</el-dropdown-item>
              <el-dropdown-item command="logout-stop-forging">{{ t('header.logoutStopForging') }}</el-dropdown-item>
              <el-dropdown-item command="logout-clear-data">{{ t('header.logoutClearData') }}</el-dropdown-item>
            </template>
          </el-dropdown-menu>
        </template>
      </el-dropdown>
    </div>

    <!-- Send Money Modal -->
    <SendMoneyModal v-model:visible="showSendMoneyDialog" @success="onSendMoneySuccess" />

    <!-- Send Message Modal -->
    <SendMessageModal v-model:visible="showSendMessageDialog" @success="onSendMessageSuccess" />

    <!-- Notifications Drawer -->
    <el-drawer v-model="showNotifications" title="Notifications" direction="rtl" size="380px">
      <div v-if="notifications.length === 0" style="padding: 40px 0;"><el-empty description="No notifications" /></div>
    </el-drawer>
  </header>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useAppStore } from '@/stores/app'
import { ElMessage, ElMessageBox } from 'element-plus'
import Breadcrumb from './Breadcrumb.vue'
import SendMoneyModal from '@/components/modals/SendMoneyModal.vue'
import SendMessageModal from '@/components/modals/SendMessageModal.vue'

interface Props { showBreadcrumb?: boolean }
withDefaults(defineProps<Props>(), { showBreadcrumb: true })
const emit = defineEmits<{ sidebarToggle: [] }>()

const router = useRouter()
const { t } = useI18n()
const appStore = useAppStore()
const isCollapsed = computed(() => appStore.isSidebarCollapsed)
const unconfirmedCount = ref(0)
const notificationCount = ref(0)
const notifications = ref<any[]>([])
const showSendMoneyDialog = ref(false)
const showSendMessageDialog = ref(false)

const onSendMoneySuccess = () => {
  ElMessage.success(t('sendMoney.success'))
}

const onSendMessageSuccess = () => {
  ElMessage.success(t('sendMessage.success'))
}

const showNotifications = ref(false)

const breadcrumbRoutes = computed(() => {
  return router.currentRoute.value.matched.map(r => ({ path: r.path, title: (r.meta?.title as string) || '' }))
})

function toggleSidebar() { appStore.toggleSidebar(); emit('sidebarToggle') }
function navigateTo(path: string) { router.push(path) }

function handleSettingsCommand(cmd: string) {
  const map: Record<string, string> = {
    blocks: '/settings/blocks', peers: '/settings/peers', generators: '/settings/generators',
    'scheduled-transactions': '/settings/scheduled-transactions', monitors: '/settings/monitors',
    plugins: '/settings/plugins', 'account-settings': '/settings/account',
    'generate-token': '/settings/token', 'generate-hallmark': '/settings/hallmark',
    'calculate-hash': '/settings/hash-calculator', 'transaction-operations': '/settings/transaction-operations',
    'api-console': '/test'
  }
  if (cmd === 'api-console') window.open(map[cmd], '_blank')
  else if (map[cmd]) router.push(map[cmd])
}

async function handleLogoutCommand(cmd: string) {
  const confirms: Record<string, () => Promise<boolean>> = {
    logout: () => ElMessageBox.confirm('Are you sure you want to log out?', 'Confirm', { type: 'warning' }).then(() => true).catch(() => false),
    'logout-stop-forging': () => ElMessageBox.confirm('Log out and stop forging?', 'Confirm', { type: 'warning' }).then(() => true).catch(() => false),
    'logout-clear-data': () => ElMessageBox.confirm('This will clear all user data. Continue?', 'Warning', { type: 'warning' }).then(() => true).catch(() => false)
  }
  if (await confirms[cmd]()) {
    localStorage.removeItem('access_token'); localStorage.removeItem('refresh_token'); localStorage.removeItem('nrcs_account_rs'); localStorage.removeItem('nrcs_balance_nqt'); localStorage.removeItem('nrcs_secret_phrase')
    await router.push('/login')
  }
}
</script>

<style scoped lang="scss">
.nrcs-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: $header-height;
  padding: 0 $space-xl;
  background: $header-bg;
  border-bottom: 1px solid $border-subtle;
  backdrop-filter: blur(20px);
  position: relative;
  z-index: $z-header;

  &::after {
    content: '';
    position: absolute;
    bottom: 0; left: 0; right: 0;
    height: 1px;
    background: linear-gradient(90deg, transparent, rgba($primary, 0.3), transparent);
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: $space-lg;

    .collapse-toggle {
      width: 34px; height: 34px;
      border: none;
      background: transparent;
      border-radius: $radius-sm;
      cursor: pointer;
      display: flex;
      align-items: center;
      justify-content: center;
      color: $text-muted;
      transition: all $duration-fast ease;

      &:hover { color: $text-primary; background: $bg-hover; }
    }
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 4px;

    .action-group {
      display: flex;
      align-items: center;
      gap: 4px;
    }

    .action-btn {
      display: inline-flex;
      align-items: center;
      gap: 6px;
      padding: 7px 14px;
      border-radius: $radius-md;
      font-size: $font-size-sm;
      font-weight: 600;
      letter-spacing: $letter-spacing-tight;
      cursor: pointer;
      transition: all $duration-normal $ease-out-expo;
      border: 1px solid transparent;
      color: $text-secondary;
      background: transparent;

      &:hover {
        color: $text-primary;
        background: $bg-hover;
        border-color: $border-default;
      }

      &--primary {
        color: $surface-900;
        background: linear-gradient(135deg, $primary, $primary-dark);
        border-color: transparent;
        box-shadow: 0 2px 8px rgba($primary, 0.25);

        &:hover {
          box-shadow: 0 4px 16px rgba($primary, 0.35);
          transform: translateY(-1px);
        }

        .action-label { display: inline; }
      }
    }

    .divider-v {
      width: 1px; height: 20px;
      background: $divider;
      margin: 0 6px;
    }

    .icon-btn {
      width: 36px; height: 36px;
      border: none;
      background: transparent;
      border-radius: $radius-sm;
      cursor: pointer;
      display: flex;
      align-items: center;
      justify-content: center;
      color: $text-muted;
      position: relative;
      transition: all $duration-fast ease;

      &:hover {
        color: $text-primary;
        background: $bg-hover;
      }

      &--badge {
        .btn-badge {
          position: absolute;
          top: -2px; right: -2px;
          min-width: 16px; height: 16px;
          padding: 0 4px;
          border-radius: $radius-full;
          font-size: 10px;
          font-weight: 700;
          display: flex;
          align-items: center;
          justify-content: center;
          background: $danger;
          color: white;
          box-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);

          &--pulse { animation: pulse-glow 2s infinite; }
        }
      }
    }

    .user-chip {
      display: flex;
      align-items: center;
      gap: 6px;
      padding: 4px 10px 4px 4px;
      border-radius: $radius-full;
      border: 1px solid $border-subtle;
      cursor: pointer;
      transition: all $duration-fast ease;
      margin-left: 4px;

      &:hover { border-color: $border-glow; background: $bg-hover; }

      .chip-avatar {
        width: 26px; height: 26px;
        border-radius: $radius-sm;
        background: linear-gradient(135deg, $primary-light, $primary);
        display: flex;
        align-items: center;
        justify-content: center;
        font-family: $font-display;
        font-size: 12px;
        font-weight: 800;
        color: $surface-900;
      }
    }
  }
}
</style>
