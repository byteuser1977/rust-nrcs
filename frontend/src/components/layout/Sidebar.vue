<template>
  <div class="nrcs-sidebar" :class="{ collapsed: isCollapsed }">
    <!-- 用户信息面板 -->
    <div class="sidebar-user-panel" v-if="!isCollapsed">
      <div class="user-avatar-ring">
        <div class="user-avatar">{{ accountInitial }}</div>
      </div>
      <div class="user-info">
        <span class="user-label">{{ t('sidebar.yourAccount') }}</span>
        <!-- 账户名（对标 nrs.js accountInfo.name 显示） -->
        <span v-if="accountName" class="user-account-name" :title="accountName">{{ accountName }}</span>
        <div class="user-account-row" :title="accountRS" @click="copyAccountId">
          <span class="user-account-id text-mono">{{ accountRS }}</span>
          <button class="copy-btn" :title="t('common.copy')">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none"><rect x="9" y="9" width="13" height="13" rx="2" stroke="currentColor" stroke-width="2"/><path d="M5 15V5a2 2 0 012-2h10" stroke="currentColor" stroke-width="2"/></svg>
          </button>
        </div>
      </div>

      <div class="user-balance-bar" @click="showAccountDetails">
        <span class="balance-amount text-mono">{{ formattedBalance }}</span>
        <span class="balance-unit">NRC</span>
      </div>

      <div class="user-meta-row">
        <span class="meta-item" :class="{ online: isConnected }">
          <i class="meta-dot"></i>
          {{ connectionStatusText }}
        </span>
        <span class="meta-divider"></span>
        <span class="meta-item text-mono">#{{ blockHeight }}</span>
      </div>

      <!-- 锻造状态指示器（对标 nrs.modals.forging.js forgingIndicator） -->
      <div v-if="canForge" class="user-forging-row" :class="forgingState" :title="forgingTooltip" @click="$emit('openForging')">
        <span class="forging-dot" :style="{ background: forgingIndicatorColor }"></span>
        <span class="forging-text">{{ forgingStatusLabel }}</span>
      </div>

      <!-- 测试网标识（对标 NRS.isTestnet 的 UI 标识） -->
      <div v-if="isTestnet" class="testnet-row">
        <span class="testnet-dot"></span>
        <span class="testnet-text">{{ t('header.testnet') }}</span>
      </div>

      <div class="sidebar-search-wrap">
        <el-input v-model="searchId" :placeholder="t('sidebar.searchById')" size="small" clearable @keyup.enter="handleSearch">
          <template #prefix>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none"><circle cx="11" cy="11" r="8" stroke="#64748b" stroke-width="2"/><path d="M21 21l-4.35-4.35" stroke="#64748b" stroke-width="2" stroke-linecap="round"/></svg>
          </template>
        </el-input>
      </div>
    </div>

    <!-- 菜单区域 -->
    <nav class="sidebar-nav">
      <el-menu :default-active="activeMenu" :default-openeds="defaultOpeneds" :collapse="isCollapsed"
              :unique-opened="false" :router="false" :collapse-transition="false"
              class="nrcs-menu">
        <template v-for="menu in sidebarMenus" :key="menu.path">
          <el-menu-item v-if="!menu.children || menu.children.length === 0" :index="menu.path" @click="handleMenuClick(menu)">
            <component :is="'el-icon'" v-if="menu.icon"><component :is="menu.icon" /></component>
            <template #title>
              <span>{{ menu.title }}</span>
              <el-badge v-if="menu.badge" :value="menu.badge" :max="99" class="menu-badge" />
            </template>
          </el-menu-item>
          <el-sub-menu v-else :index="menu.path">
            <template #title>
              <component :is="'el-icon'" v-if="menu.icon"><component :is="menu.icon" /></component>
              <span>{{ menu.title }}</span>
              <el-badge v-if="menu.badge" :value="menu.badge" :max="99" class="menu-badge" />
            </template>
            <template v-for="child in menu.children" :key="child.path">
              <div v-if="child.isDivider" class="menu-section-header">
                <span>{{ child.title }}</span>
              </div>
              <el-menu-item v-else-if="child.isButton" :index="child.path" class="action-menu-item" @click="handleButtonClick(child)">
                <component :is="'el-icon'" v-if="child.icon"><component :is="child.icon" /></component>
                <template #title><span class="action-text">{{ child.title }}</span></template>
              </el-menu-item>
              <el-menu-item v-else :index="child.path" @click="handleMenuClick(child)">
                <component :is="'el-icon'" v-if="child.icon"><component :is="child.icon" /></component>
                <template #title>
                  <span>{{ child.title }}</span>
                  <el-badge v-if="child.badge" :value="child.badge" :max="99" class="menu-badge" />
                </template>
              </el-menu-item>
            </template>
          </el-sub-menu>
        </template>
      </el-menu>
    </nav>

    <!-- 底部品牌 -->
    <div class="sidebar-footer" v-if="!isCollapsed">
      <div class="brand-mark">
        <span class="brand-letter">N</span>
        <span class="brand-name">NRCS</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useAppStore } from '@/stores/app'
import { useAccountStore } from '@/stores/modules/account.store'
import { useNodeStore } from '@/stores/modules/node.store'
import { useNotifications } from '@/composables/useNotifications'
import { useForging } from '@/composables/useForging'
import { ElMessage } from 'element-plus'

interface MenuItem {
  path: string
  title: string
  icon?: string
  isButton?: boolean
  isDivider?: boolean
  children?: MenuItem[]
  badge?: number
}

const route = useRoute()
const router = useRouter()
const { t } = useI18n()
const appStore = useAppStore()
const accountStore = useAccountStore()
const nodeStore = useNodeStore()
const { unconfirmedCount, phasingCount, shufflingCount } = useNotifications()
const { forgingStatus, forgingStatusLabel, forgingTooltip, canForge } = useForging()

defineEmits<{ openForging: [] }>()

const isCollapsed = computed(() => appStore.isSidebarCollapsed)
const activeMenu = computed(() => route.path)

// --- 响应式账户信息（从 accountStore 读取，替代 localStorage） ---
const accountRS = computed(() => accountStore.accountRS || 'NRCS-XXXX-XXXX-XXXX-XXXXX')
const accountName = computed(() => accountStore.name || '')
const formattedBalance = computed(() => {
  // 使用 accountStore 的 balanceFormatted（NQT → NRC，BigInt 精确计算）
  const nrc = accountStore.balanceFormatted
  return nrc.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })
})

// --- 响应式节点状态（从 nodeStore 读取，替代 appStore） ---
const isConnected = computed(() => nodeStore.serverConnect)
const connectionStatusText = computed(() =>
  nodeStore.serverConnect ? t('sidebar.connected') : t('sidebar.disconnected'),
)
const blockHeight = computed(() => nodeStore.lastBlockHeight)
const isTestnet = computed(() => nodeStore.isTestnet || accountStore.isTestNet)

// --- 锻造状态（对标 nrs.modals.forging.js） ---
const forgingState = computed(() => forgingStatus.value)
const forgingIndicatorColor = computed(() => {
  if (forgingState.value === 'forging') return '#48AB6C'
  if (forgingState.value === 'not_forging') return '#909399'
  return '#E6A23C'
})

// --- 通知徽章计数（对标 nrs.notifications.js 分类计数） ---
const unconfirmedBadge = computed(() => unconfirmedCount.value)
const phasingBadge = computed(() => phasingCount.value)
const shufflingBadge = computed(() => shufflingCount.value)

/** 账户首字母（用于头像） */
const accountInitial = computed(() => {
  const rs = accountStore.accountRS
  if (!rs) return 'N'
  const parts = rs.split('-')
  return parts[parts.length - 1].charAt(0).toUpperCase()
})

const defaultOpeneds = computed(() => {
  const segments = route.path.split('/').filter(Boolean)
  const opened: string[] = []
  for (let i = 1; i <= segments.length - 1; i++) opened.push('/' + segments.slice(0, i + 1).join('/'))
  return opened
})
const searchId = ref('')

const sidebarMenus = computed<MenuItem[]>(() => [
  { path: '/dashboard', title: t('menu.dashboard'), icon: 'Odometer', children: [
    { path: '/dashboard', title: t('menu.dashboardPanel'), icon: 'Monitor' },
    { path: '/dashboard/ledger', title: t('menu.accountLedger'), icon: 'Notebook' },
    { path: '/dashboard/properties', title: t('menu.accountProperties'), icon: 'Setting' },
    { path: '/dashboard/transactions', title: t('menu.myTransactions'), icon: 'List', badge: unconfirmedBadge.value || undefined },
    { path: '/dashboard/approval-requests', title: t('menu.approvalRequests'), icon: 'Finished', badge: phasingBadge.value || undefined }
  ]},
  { path: '/assets', title: t('menu.assets'), icon: 'TrendCharts', children: [
    { path: '/assets/exchange', title: t('menu.assetExchange'), icon: 'Sell' },
    { path: '/assets/trade-history', title: t('menu.tradeHistory'), icon: 'Timer' },
    { path: '/assets/transfer-history', title: t('menu.transferHistory'), icon: 'Sort' },
    { path: '/assets/deletes-history', title: t('menu.deletesHistory'), icon: 'Delete' },
    { path: '/assets/my-assets', title: t('menu.myAssets'), icon: 'Wallet' },
    { path: '/assets/open-orders', title: t('menu.openOrders'), icon: 'Document' },
    { path: '/assets/approval-requests', title: t('menu.approvalRequests'), icon: 'Finished' },
    { path: '/assets/issue', title: t('menu.issueAsset'), icon: 'Plus', isButton: true }
  ]},
  { path: '/monetary', title: t('menu.monetarySystem'), icon: 'Coin', children: [
    { path: '/monetary/currencies', title: t('menu.currencies'), icon: 'Money' },
    { path: '/monetary/exchange-history', title: t('menu.exchangeHistory'), icon: 'Timer' },
    { path: '/monetary/transfer-history', title: t('menu.transferHistory'), icon: 'Sort' },
    { path: '/monetary/approval-requests', title: t('menu.approvalRequests'), icon: 'Finished' },
    { path: '/monetary/issue', title: t('menu.issueCurrency'), icon: 'Plus', isButton: true }
  ]},
  { path: '/voting', title: t('menu.voting'), icon: 'Checked', children: [
    { path: '/voting/active-polls', title: t('menu.activePolls'), icon: 'DataLine' },
    { path: '/voting/followed-polls', title: t('menu.followedPolls'), icon: 'Star' },
    { path: '/voting/my-votes', title: t('menu.myVotes'), icon: 'Select' },
    { path: '/voting/my-polls', title: t('menu.myPolls'), icon: 'EditPen' },
    { path: '/voting/create', title: t('menu.createPoll'), icon: 'Plus', isButton: true }
  ]},
  { path: '/marketplace', title: t('menu.marketplace'), icon: 'ShoppingCart', children: [
    { path: '/marketplace/search', title: t('menu.marketplaceSearch'), icon: 'Search' },
    { path: '/marketplace/purchased', title: t('menu.purchasedProducts'), icon: 'Goods' },
    { path: '/marketplace/my-products', title: t('menu.myStore'), icon: 'Sell', isDivider: true },
    { path: '/marketplace/my-products', title: t('menu.myProducts'), icon: 'Sell' },
    { path: '/marketplace/pending-orders', title: t('menu.pendingOrders'), icon: 'Clock' },
    { path: '/marketplace/completed-orders', title: t('menu.completedOrders'), icon: 'CircleCheck' },
    { path: '/marketplace/list-product', title: t('menu.listProductForSale'), icon: 'Plus', isButton: true }
  ]},
  { path: '/datacloud', title: t('menu.dataCloud'), icon: 'Cloudy', children: [
    { path: '/datacloud/search', title: t('menu.search'), icon: 'Search' },
    { path: '/datacloud/upload', title: t('menu.fileUpload'), icon: 'Upload', isButton: true }
  ]},
  { path: '/messages', title: t('menu.messages'), icon: 'ChatDotRound', children: [
    { path: '/messages', title: t('menu.chat'), icon: 'ChatLineRound' },
    { path: '/messages/my', title: t('menu.myMessages'), icon: 'List' }
  ]},
  { path: '/aliases', title: t('menu.aliases'), icon: 'Bookmark' },
  { path: '/shuffling', title: t('menu.shuffling'), icon: 'Switch', badge: shufflingBadge.value || undefined, children: [
    { path: '/shuffling/active', title: t('menu.activeShufflings'), icon: 'DataLine', badge: shufflingBadge.value || undefined },
    { path: '/shuffling/my', title: t('menu.myShufflings'), icon: 'User' },
    { path: '/shuffling/create', title: t('menu.createShuffling'), icon: 'Plus', isButton: true }
  ]}
])

function handleMenuClick(item: MenuItem) { router.push(item.path) }
function handleButtonClick(item: MenuItem) { router.push(item.path) }
function copyAccountId() {
  navigator.clipboard.writeText(accountRS.value)
    .then(() => ElMessage.success(t('common.copied')))
    .catch(() => ElMessage.error(t('common.copyFailed')))
}
function showAccountDetails() { router.push('/dashboard') }
function handleSearch() { if (searchId.value.trim()) router.push({ path: '/dashboard', query: { search: searchId.value.trim() } }) }
</script>

<style scoped lang="scss">
.nrcs-sidebar {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: linear-gradient(180deg, rgba(11, 17, 33, 0.98), rgba(11, 17, 33, 1));
  border-right: 1px solid $border-subtle;
  transition: width $duration-slow $ease-out-expo;
  overflow: hidden;

  &.collapsed { width: $sidebar-collapsed-width; }

  .sidebar-user-panel {
    padding: $space-lg;
    border-bottom: 1px solid $border-subtle;
    background: rgba(255, 255, 255, 0.01);

    .user-avatar-ring {
      width: 44px; height: 44px;
      border-radius: $radius-md;
      background: $primary-subtle;
      border: 1px solid $border-glow;
      display: flex;
      align-items: center;
      justify-content: center;
      margin-bottom: $space-sm;

      .user-avatar {
        font-family: $font-display;
        font-size: $font-size-lg;
        font-weight: 700;
        color: $primary;
      }
    }

    .user-info {
      margin-bottom: $space-sm;

      .user-label {
        font-size: $font-size-xs;
        color: $text-muted;
        text-transform: uppercase;
        letter-spacing: $letter-spacing-wide;
        font-weight: 500;
        display: block;
      }

      .user-account-name {
        display: block;
        font-size: $font-size-sm;
        color: $text-primary;
        font-weight: 600;
        margin-top: 2px;
        max-width: 100%;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
      }

      .user-account-row {
        display: flex;
        align-items: center;
        gap: 4px;
        cursor: pointer;
        margin-top: 4px;

        &:hover .user-account-id { color: $primary; }

        .user-account-id {
          font-size: $font-size-xs;
          color: $text-secondary;
          word-break: break-all;
          line-height: 1.4;
          transition: color $duration-fast ease;
        }

        .copy-btn {
          flex-shrink: 0;
          width: 22px; height: 22px;
          border: none;
          background: transparent;
          color: $text-muted;
          cursor: pointer;
          border-radius: $radius-sm;
          display: flex;
          align-items: center;
          justify-content: center;
          transition: all $duration-fast ease;

          &:hover { color: $primary; background: $bg-hover; }
        }
      }
    }

    .user-balance-bar {
      display: inline-flex;
      align-items: baseline;
      gap: 4px;
      padding: 6px 14px;
      background: rgba($primary, 0.06);
      border: 1px solid rgba($primary, 0.15);
      border-radius: $radius-full;
      cursor: pointer;
      margin-bottom: $space-sm;
      transition: all $duration-fast ease;

      &:hover {
        background: rgba($primary, 0.1);
        border-color: $border-glow;
      }

      .balance-amount {
        font-size: $font-size-md;
        font-weight: 700;
        color: $primary;
      }

      .balance-unit {
        font-size: $font-size-xs;
        color: $text-muted;
        font-weight: 500;
      }
    }

    .user-meta-row {
      display: flex;
      align-items: center;
      gap: $space-sm;
      font-size: $font-size-xs;
      color: $text-muted;
      margin-bottom: $space-sm;

      .meta-item {
        display: flex;
        align-items: center;
        gap: 4px;

        &.online { color: $success; }

        .meta-dot {
          width: 6px; height: 6px;
          border-radius: 50%;
          background: $danger;
          box-shadow: 0 0 4px rgba($danger, 0.4);

          .online & { background: $success; box-shadow: 0 0 4px rgba($success, 0.4); }
        }
      }

      .meta-divider {
        width: 1px; height: 10px;
        background: $divider;
      }
    }

    // ── 锻造状态行（对标 nrs.modals.forging.js forgingIndicator） ──
    .user-forging-row {
      display: flex;
      align-items: center;
      gap: 6px;
      padding: 4px 10px;
      border-radius: $radius-sm;
      cursor: pointer;
      margin-bottom: $space-sm;
      transition: all $duration-fast ease;
      border: 1px solid transparent;
      width: fit-content;

      &:hover {
        background: $bg-hover;
        border-color: $border-subtle;
      }

      &.forging {
        background: rgba($success, 0.08);
        border-color: rgba($success, 0.3);
      }

      .forging-dot {
        width: 7px; height: 7px;
        border-radius: 50%;
        box-shadow: 0 0 4px currentColor;
      }

      .forging-text {
        font-size: $font-size-xs;
        font-weight: 600;
        color: $text-secondary;
      }

      &.forging .forging-text { color: $success; }
    }

    // ── 测试网标识 ──
    .testnet-row {
      display: flex;
      align-items: center;
      gap: 6px;
      padding: 3px 10px;
      border-radius: $radius-full;
      background: rgba($warning, 0.12);
      border: 1px solid rgba($warning, 0.3);
      margin-bottom: $space-sm;
      width: fit-content;

      .testnet-dot {
        width: 6px; height: 6px;
        border-radius: 50%;
        background: $warning;
        box-shadow: 0 0 4px rgba($warning, 0.6);
      }

      .testnet-text {
        font-size: $font-size-xs;
        font-weight: 700;
        color: $warning;
      }
    }

    .sidebar-search-wrap {
      :deep(.el-input__wrapper) {
        background: rgba(26, 34, 52, 0.6);
        border-radius: $radius-full;
        box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.1);
      }
    }
  }

  .sidebar-nav {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: $space-sm 0;

    .nrcs-menu {
      border-right: none !important;
      background: transparent !important;

      &:not(.el-menu--collapse) { width: 100%; }

      .action-menu-item .action-text { color: $primary !important; opacity: 0.85; }

      .menu-section-header {
        padding: $space-md $space-lg $space-sm;
        font-size: $font-size-xs;
        font-weight: 600;
        color: $text-muted;
        text-transform: uppercase;
        letter-spacing: $letter-spacing-wide;
        border-top: 1px solid $divider;
        margin-top: 4px;
      }

      // ── 菜单项徽章（对标 nrs.notifications.js 通知计数显示） ──
      .menu-badge {
        margin-left: 8px;

        :deep(.el-badge__content) {
          background-color: $danger;
          border: none;
        }
      }
    }
  }

  .sidebar-footer {
    padding: $space-lg;
    border-top: 1px solid $border-subtle;

    .brand-mark {
      display: flex;
      align-items: center;
      gap: 8px;

      .brand-letter {
        width: 26px; height: 26px;
        display: flex;
        align-items: center;
        justify-content: center;
        background: linear-gradient(135deg, $primary, darken($primary, 20%));
        border-radius: $radius-sm;
        font-family: $font-display;
        font-size: 13px;
        font-weight: 800;
        color: $surface-900;
      }

      .brand-name {
        font-family: $font-display;
        font-size: $font-size-sm;
        font-weight: 700;
        color: $text-muted;
        letter-spacing: 2px;
        text-transform: uppercase;
      }
    }
  }
}
</style>
