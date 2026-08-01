<template>
  <header class="nrcs-header">
    <!-- 分叉警告 Banner（对标 nrs.js:1713/1719 fork_warning 显示） -->
    <div v-if="forkWarning" class="fork-warning-banner" :class="{ 'fork-base-target': forkWarning === 'fork_warning_base_target' }">
      <el-icon><Warning /></el-icon>
      <span>{{ forkWarning === 'fork_warning' ? t('dashboard.forkWarning') : t('dashboard.forkWarningBaseTarget') }}</span>
    </div>

    <!-- 区块链下载进度条（对标 nrs.js:1646-1695 updateBlockchainDownloadProgress） -->
    <div v-if="isDownloading && downloadProgress" class="download-progress-bar">
      <span class="download-label">{{ t('dashboard.downloadInProgress') }}</span>
      <el-progress
        :percentage="downloadProgress.percentageTotal"
        :stroke-width="4"
        :show-text="false"
        class="download-progress"
      />
      <span class="download-percent">{{ downloadProgress.percentageTotal }}%</span>
      <span v-if="downloadProgress.blocksLeft > 0" class="download-blocks-left">
        {{ downloadProgress.blocksLeft }} {{ t('dashboard.blocksLeft') }}
      </span>
    </div>

    <div class="header-main-row">
      <div class="header-left">
        <button class="collapse-toggle" @click="toggleSidebar" :title="isCollapsed ? t('header.expand') : t('header.collapse')">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
            <path v-if="!isCollapsed" d="M3 12h18M3 6h18M3 18h18" stroke="#94a3b8" stroke-width="1.8" stroke-linecap="round"/>
            <path v-else d="M4 6h16v12H4z M15 3l6 6-6 6" stroke="#94a3b8" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </button>
        <Breadcrumb v-if="showBreadcrumb" :routes="breadcrumbRoutes" />
      </div>

      <div class="header-actions">
        <!-- 测试网标识（对标 NRS.isTestnet 的 UI 标识） -->
        <div v-if="isTestnet" class="testnet-badge" :title="t('login.testnetWarning')">
          <span class="testnet-dot"></span>
          {{ t('header.testnet') }}
        </div>

        <!-- 节点状态指示器（对标 nrs.js dashboard 状态展示） -->
        <div class="node-status-indicator" :title="nodeStatusTooltip">
          <span class="status-dot" :class="{ online: serverConnected, offline: !serverConnected }"></span>
          <span class="status-height text-mono" v-if="currentHeight > 0">#{{ currentHeight }}</span>
          <span class="status-height text-mono" v-else>{{ t('sidebar.disconnected') }}</span>
        </div>

        <!-- 锻造状态指示器（对标 nrs.modals.forging.js forgingIndicator） -->
        <div
          v-if="canForge"
          class="forging-indicator"
          :class="forgingState"
          :title="forgingTooltip || t('forging.tooltipNotStarted')"
          @click="onForgingIndicatorClick"
        >
          <span class="forging-dot" :style="{ background: forgingIndicatorColor }"></span>
          <span class="forging-label">{{ forgingStatusLabel }}</span>
        </div>

        <div class="divider-v"></div>

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

        <button class="icon-btn" @click="showClientStatusModal = true" :title="t('header.clientStatus')">
          <svg width="17" height="17" viewBox="0 0 24 24" fill="none"><rect x="2" y="3" width="20" height="14" rx="2" stroke="#94a3b8" stroke-width="1.8"/><line x1="8" y1="21" x2="16" y2="21" stroke="#94a3b8" stroke-width="1.8" stroke-linecap="round"/><line x1="12" y1="17" x2="12" y2="21" stroke="#94a3b8" stroke-width="1.8"/></svg>
        </button>

        <button class="icon-btn icon-btn--badge" @click="navigateTo('/dashboard/transactions')" :title="t('header.unconfirmedTxs')">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none"><circle cx="12" cy="12" r="10" stroke="#94a3b8" stroke-width="1.8"/><polyline points="12,6 12,12 16,14" stroke="#94a3b8" stroke-width="1.8" stroke-linecap="round"/></svg>
          <span class="btn-badge" v-if="unconfirmedCount > 0">{{ unconfirmedCount > 99 ? '99+' : unconfirmedCount }}</span>
        </button>

        <button class="icon-btn icon-btn--badge" @click="openNotifications" :title="t('header.notifications')">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none"><path d="M18 8A6 6 0 006 8c0 7-3 9-3 9h18c0-2-3-4-3-9zM13.73 21a2 2 0 01-3.46 0" stroke="#94a3b8" stroke-width="1.8" stroke-linecap="round"/></svg>
          <span class="btn-badge btn-badge--pulse" v-if="notificationCount > 0">{{ notificationCount > 99 ? '99+' : notificationCount }}</span>
        </button>

        <button class="icon-btn" @click="navigateTo('/contacts')" :title="t('header.contacts')">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none"><path d="M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2" stroke="#94a3b8" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/><circle cx="12" cy="7" r="4" stroke="#94a3b8" stroke-width="1.8"/></svg>
        </button>

        <el-dropdown trigger="click" @command="handleSettingsCommand">
          <button class="icon-btn" :title="t('header.tools')">
            <svg width="17" height="17" viewBox="0 0 24 24" fill="none"><circle cx="12" cy="12" r="3" stroke="#94a3b8" stroke-width="1.8"/><path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.82 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-2 2 2 2 0 01-2-2v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.82l.06-.06a1.65 1.65 0 00.33-1.82 1.65 1.65 0 00-1.51-1H3a2 2 0 01-2-2 2 2 0 012-2h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 010-2.82 2 2 0 012.82 0l.06.06a1.65 1.65 0 001.82.33H9a1.65 1.65 0 001-1.51V3a2 2 0 012-2 2 2 0 012 2v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.82 0 2 2 0 010 2.82l-.06.06a1.65 1.65 0 00-.33 1.82V9a1.65 1.65 0 001.51 1H21a2 2 0 012 2 2 2 0 01-2 2h-.09a1.65 1.65 0 00-1.51 1z" stroke="#94a3b8" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/></svg>
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
                <el-dropdown-item command="debug-console">{{ t('settings.debugConsole') }}</el-dropdown-item>
                <el-dropdown-item command="api-console">{{ t('settings.apiConsole') }} →</el-dropdown-item>
              </template>
            </el-dropdown-menu>
          </template>
        </el-dropdown>

        <el-dropdown trigger="click" @command="handleLogoutCommand">
          <button class="user-chip" :title="t('header.logout')">
            <div class="chip-avatar">{{ accountInitial }}</div>
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
    </div>

    <!-- Send Money Modal -->
    <SendMoneyModal v-model:visible="showSendMoneyDialog" @success="onSendMoneySuccess" />

    <!-- Send Message Modal -->
    <SendMessageModal v-model:visible="showSendMessageDialog" @success="onSendMessageSuccess" />

    <!-- Forging Modal -->
    <ForgingModal v-model:visible="showForgingModal" :mode="forgingModalMode" @success="onForgingSuccess" />

    <!-- Notifications Drawer -->
    <el-drawer
      v-model="showNotifications"
      :title="t('header.notifications')"
      direction="rtl"
      size="400px"
      @open="onNotificationsOpen"
    >
      <div class="notifications-container">
        <!-- 通知分类列表（对标 nrs.notifications.js updateNotificationUI） -->
        <div v-if="notificationCategories.length > 0" class="notification-categories">
          <div
            v-for="cat in notificationCategories"
            :key="cat.key"
            class="notification-category-item"
            @click="onNotificationCategoryClick(cat)"
          >
            <div class="cat-info">
              <el-tag :type="cat.type as any" size="small" effect="dark">
                {{ cat.count }}
              </el-tag>
              <span class="cat-label">{{ cat.label }}</span>
            </div>
            <el-icon class="cat-arrow"><ArrowRight /></el-icon>
          </div>
        </div>

        <el-divider v-if="notificationCategories.length > 0" />

        <!-- 未确认交易（对标 setUnconfirmedNotifications） -->
        <div v-if="unconfirmedCount > 0" class="notification-section">
          <h4 class="section-title">{{ t('header.unconfirmedTxs') }}</h4>
          <div class="section-item" @click="navigateTo('/dashboard/transactions')">
            <el-tag type="warning" size="small">{{ unconfirmedCount }}</el-tag>
            <span>{{ t('header.unconfirmedTxsHint') }}</span>
          </div>
        </div>

        <!-- Phased 交易（对标 setPhasingNotifications） -->
        <div v-if="phasingCount > 0" class="notification-section">
          <h4 class="section-title">{{ t('transaction.phased') }}</h4>
          <div class="section-item" @click="navigateTo('/dashboard/approval-requests')">
            <el-tag type="info" size="small">{{ phasingCount }}</el-tag>
            <span>{{ t('header.phasedTxHint') }}</span>
          </div>
        </div>

        <!-- Shuffling（对标 setShufflingNotifications） -->
        <div v-if="shufflingCount > 0" class="notification-section">
          <h4 class="section-title">{{ t('menu.activeShufflings') }}</h4>
          <div class="section-item" @click="navigateTo('/shuffling/active')">
            <el-tag type="success" size="small">{{ shufflingCount }}</el-tag>
            <span>{{ t('header.shufflingHint') }}</span>
          </div>
        </div>

        <!-- 空状态 -->
        <div v-if="notificationCategories.length === 0 && unconfirmedCount === 0 && phasingCount === 0 && shufflingCount === 0" class="no-notifications">
          <el-empty :description="t('header.noNotifications')" />
        </div>

        <!-- 全部已读按钮（对标 resetNotificationState） -->
        <div v-if="totalNotificationCount > 0" class="mark-all-read">
          <el-button text type="primary" @click="markAllNotificationsRead">
            <el-icon><Check /></el-icon>
            {{ t('header.markAllRead') }}
          </el-button>
        </div>
      </div>
    </el-drawer>

    <!-- 客户端状态模态框（对标 nrs.header.js client_status_modal） -->
    <el-dialog
      v-model="showClientStatusModal"
      :title="t('header.clientStatus')"
      width="560px"
      destroy-on-close
      class="nrcs-modal"
    >
      <el-descriptions :column="1" border>
        <el-descriptions-item :label="t('header.connectionStatus')">
          <el-tag :type="serverConnected ? 'success' : 'danger'" size="small">
            {{ serverConnected ? t('sidebar.connected') : t('sidebar.disconnected') }}
          </el-tag>
        </el-descriptions-item>
        <el-descriptions-item :label="t('dashboard.height')">
          <span class="text-mono">#{{ currentHeight }}</span>
        </el-descriptions-item>
        <el-descriptions-item :label="t('header.nodeVersion')">
          <span class="text-mono">{{ nodeVersion }}</span>
        </el-descriptions-item>
        <el-descriptions-item :label="t('header.application')">
          {{ application }}
        </el-descriptions-item>
        <el-descriptions-item v-if="isLightClient" :label="t('header.clientType')">
          <el-tag type="info" size="small">{{ t('header.lightClient') }}</el-tag>
        </el-descriptions-item>
        <el-descriptions-item v-if="isApiProxy" :label="t('header.apiProxy')">
          <el-tag type="warning" size="small">{{ t('header.apiProxyMode') }}</el-tag>
        </el-descriptions-item>
        <el-descriptions-item v-if="isDownloading && downloadProgress" :label="t('header.downloadProgress')">
          {{ downloadProgress.percentageTotal }}% ({{ downloadProgress.blocksLeft }} {{ t('dashboard.blocksLeft') }})
        </el-descriptions-item>
        <el-descriptions-item v-if="cumulativeDifficulty" :label="t('header.cumulativeDifficulty')">
          <span class="text-mono">{{ cumulativeDifficulty }}</span>
        </el-descriptions-item>
      </el-descriptions>

      <!-- API 代理节点设置（对标 nrs.header.js setAPIProxyPeer/blacklistAPIProxyPeer） -->
      <div v-if="isApiProxy" class="api-proxy-section">
        <el-divider content-position="left">{{ t('header.apiProxyPeer') }}</el-divider>
        <el-input
          v-model="apiProxyPeerInput"
          :placeholder="t('header.apiProxyPeerPlaceholder')"
          size="default"
          clearable
        />
        <div class="api-proxy-actions">
          <el-button
            type="primary"
            size="small"
            :disabled="!apiProxyPeerInput || apiProxyPeerInput === apiProxyPeer"
            :loading="settingProxy"
            @click="setApiProxyPeer"
          >
            {{ t('header.setApiProxyPeer') }}
          </el-button>
          <el-button
            type="danger"
            size="small"
            :disabled="!apiProxyPeer"
            :loading="blacklistingProxy"
            @click="blacklistApiProxyPeer"
          >
            {{ t('header.blacklistApiProxyPeer') }}
          </el-button>
        </div>
      </div>
    </el-dialog>
  </header>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useAppStore } from '@/stores/app'
import { useNodeStore } from '@/stores/modules/node.store'
import { useAccountStore } from '@/stores/modules/account.store'
import { useNotifications } from '@/composables/useNotifications'
import { useForging } from '@/composables/useForging'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Warning, ArrowRight, Check } from '@element-plus/icons-vue'
import Breadcrumb from './Breadcrumb.vue'
import SendMoneyModal from '@/components/modals/SendMoneyModal.vue'
import SendMessageModal from '@/components/modals/SendMessageModal.vue'
import ForgingModal from '@/components/modals/ForgingModal.vue'
import { nrcsApi } from '@/api/modules'
import { getAdminPassword } from '@/utils/feature-detection'

interface Props { showBreadcrumb?: boolean }
withDefaults(defineProps<Props>(), { showBreadcrumb: true })
const emit = defineEmits<{ sidebarToggle: [] }>()

const router = useRouter()
const { t } = useI18n()
const appStore = useAppStore()
const nodeStore = useNodeStore()
const accountStore = useAccountStore()
const {
  unconfirmedCount,
  totalNotificationCount,
  phasingCount,
  shufflingCount,
  transactionNotifyStates,
  resetNotificationState,
} = useNotifications()
const {
  forgingStatus,
  forgingStatusLabel,
  forgingTooltip,
  canForge,
  checkForgingPreconditions,
  stopForging,
  updateForgingStatus,
} = useForging()

const isCollapsed = computed(() => appStore.isSidebarCollapsed)

// --- 动态状态（对标 nrs.header.js / nrs.js 的实时状态展示） ---
/** 总通知数（对标 NRS.notificationCount） */
const notificationCount = computed(() => totalNotificationCount.value)
/** 区块链下载进度百分比（对标 NRS.downloadProgress） */
const downloadProgress = computed(() => nodeStore.downloadProgress)
/** 是否正在下载区块链 */
const isDownloading = computed(() => nodeStore.downloadingBlockchain)
/** 分叉警告（对标 NRS.forkWarning） */
const forkWarning = computed(() => nodeStore.forkWarning)
/** 当前区块高度 */
const currentHeight = computed(() => nodeStore.lastBlockHeight)
/** 锻造状态 */
const forgingState = computed(() => forgingStatus.value)
/** 服务器连接状态 */
const serverConnected = computed(() => nodeStore.serverConnect)
/** 节点版本 */
const nodeVersion = computed(() => nodeStore.nodeVersion)
/** 应用名称 */
const application = computed(() => nodeStore.application)
/** 是否测试网 */
const isTestnet = computed(() => nodeStore.isTestnet || accountStore.isTestNet)
/** 是否轻客户端 */
const isLightClient = computed(() => nodeStore.isLightClient)
/** 是否 API 代理 */
const isApiProxy = computed(() => nodeStore.isApiProxy)
/** 累积难度 */
const cumulativeDifficulty = computed(() => nodeStore.cumulativeDifficulty)
/** 当前 API 代理节点（对标 NRS.state.apiProxyPeer） */
const apiProxyPeer = computed(() => (nodeStore.state as any)?.apiProxyPeer || '')

/** 账户首字母（用于头像） */
const accountInitial = computed(() => {
  const rs = accountStore.accountRS
  if (!rs) return 'N'
  // 取 RS 地址最后一段的首字符（NRCS-XXXX-XXXX-XXXX-XXXXX → X）
  const parts = rs.split('-')
  return parts[parts.length - 1].charAt(0).toUpperCase()
})

/** 通知分类列表（用于通知抽屉展示，对标 nrs.notifications.js 分类计数） */
const notificationCategories = computed(() => {
  const cats: { key: string; label: string; count: number; type: string; receiverPage?: string }[] = []
  if (unconfirmedCount.value > 0) {
    cats.push({ key: 'unconfirmed', label: t('header.unconfirmedTxs'), count: unconfirmedCount.value, type: 'warning' })
  }
  if (phasingCount.value > 0) {
    cats.push({ key: 'phasing', label: t('transaction.phased'), count: phasingCount.value, type: 'info' })
  }
  if (shufflingCount.value > 0) {
    cats.push({ key: 'shuffling', label: t('menu.activeShufflings'), count: shufflingCount.value, type: 'success' })
  }
  // 交易类型计数（对标 typeDict.notificationCount）
  for (const [typeKey, state] of Object.entries(transactionNotifyStates.value)) {
    if (state.notificationCount > 0) {
      cats.push({
        key: `type-${typeKey}`,
        label: t(`txType.${typeKey}`) || typeKey,
        count: state.notificationCount,
        type: 'primary',
      })
    }
  }
  if (forkWarning.value) {
    cats.push({ key: 'fork', label: t('dashboard.forkWarning'), count: 1, type: 'danger' })
  }
  return cats
})

/** 节点状态 tooltip */
const nodeStatusTooltip = computed(() => {
  if (!serverConnected.value) return t('header.serverDisconnected')
  return `${t('sidebar.connected')} · ${t('dashboard.height')}: #${currentHeight.value}`
})

const showSendMoneyDialog = ref(false)
const showSendMessageDialog = ref(false)
const showNotifications = ref(false)
const showForgingModal = ref(false)
const showClientStatusModal = ref(false)
const forgingModalMode = ref<'start' | 'stop'>('start')

/** API 代理节点输入框 */
const apiProxyPeerInput = ref('')
const settingProxy = ref(false)
const blacklistingProxy = ref(false)

// 弹窗打开时同步 API 代理节点输入框
watch(showClientStatusModal, (val) => {
  if (val) {
    apiProxyPeerInput.value = apiProxyPeer.value
  }
})

/** 锻造状态指示器颜色 */
const forgingIndicatorColor = computed(() => {
  if (forgingState.value === 'forging') return '#48AB6C'
  if (forgingState.value === 'not_forging') return '#909399'
  return '#E6A23C'
})

const onSendMoneySuccess = () => {
  ElMessage.success(t('sendMoney.success'))
}

const onSendMessageSuccess = () => {
  ElMessage.success(t('sendMessage.success'))
}

/**
 * 锻造成功回调。
 */
function onForgingSuccess(): void {
  // 状态已在 useForging 中更新，无需额外处理
}

const breadcrumbRoutes = computed(() => {
  return router.currentRoute.value.matched.map(r => ({ path: r.path, title: (r.meta?.title as string) || '' }))
})

function toggleSidebar() { appStore.toggleSidebar(); emit('sidebarToggle') }
function navigateTo(path: string) { router.push(path) }

/**
 * 打开通知抽屉（对标 nrs.notifications.js updateNotificationUI 的展示时机）。
 */
function openNotifications(): void {
  showNotifications.value = true
}

/**
 * 通知抽屉打开时的回调（对标 markAllAsRead 的查看时间戳记录）。
 */
function onNotificationsOpen(): void {
  // 打开通知抽屉时记录查看时间（不重置累积计数，仅标记查看）
}

/**
 * 通知分类点击处理（对标 nrs.notifications.js $subTypeItem.click → goToPage）。
 */
function onNotificationCategoryClick(cat: { key: string; receiverPage?: string }): void {
  if (cat.key === 'unconfirmed') {
    navigateTo('/dashboard/transactions')
  } else if (cat.key === 'phasing') {
    navigateTo('/dashboard/approval-requests')
  } else if (cat.key === 'shuffling') {
    navigateTo('/shuffling/active')
  } else if (cat.key === 'fork') {
    navigateTo('/settings/blocks')
  } else if (cat.receiverPage) {
    navigateTo(cat.receiverPage)
  }
  showNotifications.value = false
}

/**
 * 全部通知标记已读（对标 nrs.notifications.js resetNotificationState）。
 */
async function markAllNotificationsRead(): Promise<void> {
  await resetNotificationState()
  ElMessage.success(t('header.markedAllRead'))
}

/**
 * 锻造指示器点击（对标 nrs.modals.forging.js forgingIndicator.click）。
 *
 * 先执行前置校验，通过后根据当前状态显示 start/stop modal。
 */
function onForgingIndicatorClick(): void {
  const error = checkForgingPreconditions()
  if (error) {
    ElMessage.warning(error)
    return
  }
  forgingModalMode.value = forgingState.value === 'forging' ? 'stop' : 'start'
  showForgingModal.value = true
}

function handleSettingsCommand(cmd: string) {
  const map: Record<string, string> = {
    blocks: '/settings/blocks', peers: '/settings/peers', generators: '/settings/generators',
    'scheduled-transactions': '/settings/scheduled-transactions', monitors: '/settings/monitors',
    plugins: '/settings/plugins', 'account-settings': '/settings/account',
    'generate-token': '/settings/token', 'generate-hallmark': '/settings/hallmark',
    'calculate-hash': '/settings/hash-calculator', 'transaction-operations': '/settings/transaction-operations',
    'debug-console': '/settings/debug-console',
    'api-console': '/settings/api-console'
  }
  if (map[cmd]) router.push(map[cmd])
}

/**
 * 登出命令处理（对标 nrs.header.js logout + stopForging）。
 * logout-stop-forging 会先停止锻造再登出。
 */
async function handleLogoutCommand(cmd: string) {
  const confirms: Record<string, () => Promise<boolean>> = {
    logout: () => ElMessageBox.confirm(t('header.logoutConfirm'), t('common.confirm'), { type: 'warning' }).then(() => true).catch(() => false),
    'logout-stop-forging': () => ElMessageBox.confirm(t('header.logoutStopForgingConfirm'), t('common.confirm'), { type: 'warning' }).then(() => true).catch(() => false),
    'logout-clear-data': () => ElMessageBox.confirm(t('header.logoutClearDataConfirm'), t('common.warning'), { type: 'warning' }).then(() => true).catch(() => false)
  }
  if (await confirms[cmd]()) {
    // 停止锻造（对标 nrs.header.js logoutStopForging）
    if (cmd === 'logout-stop-forging') {
      try {
        await stopForging(accountStore.secretPhrase)
      } catch {
        // 忽略停止锻造失败
      }
    }
    // 清除用户数据
    localStorage.removeItem('access_token')
    localStorage.removeItem('refresh_token')
    localStorage.removeItem('nrcs_account_rs')
    localStorage.removeItem('nrcs_balance_nqt')
    localStorage.removeItem('nrcs_secret_phrase')
    if (cmd === 'logout-clear-data') {
      // 清除 IndexedDB 用户数据（对标 NRS.logoutClearData）
      try {
        indexedDB.deleteDatabase('nrcs_storage')
      } catch {
        // 忽略
      }
    }
    await router.push('/login')
  }
}

/**
 * 设置 API 代理节点（对标 nrs.header.js NRS.forms.setAPIProxyPeer）。
 */
async function setApiProxyPeer(): Promise<void> {
  if (!apiProxyPeerInput.value) return
  settingProxy.value = true
  try {
    const adminPassword = getAdminPassword()
    await nrcsApi.setAPIProxyPeer({
      peer: apiProxyPeerInput.value,
      adminPassword,
    } as any)
    ElMessage.success(t('header.setApiProxyPeerSuccess'))
    // 刷新节点状态
    await nodeStore.refreshNow()
  } catch (e: any) {
    ElMessage.error(e?.message || t('header.setApiProxyPeerError'))
  } finally {
    settingProxy.value = false
  }
}

/**
 * 黑名单 API 代理节点（对标 nrs.header.js NRS.forms.blacklistAPIProxyPeer）。
 */
async function blacklistApiProxyPeer(): Promise<void> {
  blacklistingProxy.value = true
  try {
    const adminPassword = getAdminPassword()
    await nrcsApi.blacklistAPIProxyPeer({
      adminPassword,
    } as any)
    ElMessage.success(t('header.blacklistApiProxyPeerSuccess'))
    apiProxyPeerInput.value = ''
    // 刷新节点状态
    await nodeStore.refreshNow()
  } catch (e: any) {
    ElMessage.error(e?.message || t('header.blacklistApiProxyPeerError'))
  } finally {
    blacklistingProxy.value = false
  }
}

// --- 生命周期：锻造状态轮询 ---
let forgingPollingTimer: ReturnType<typeof setInterval> | null = null

onMounted(() => {
  // 初始拉取锻造状态（对标 nrs.modals.forging.js 登录后 updateForgingStatus）
  if (accountStore.isLoggedIn) {
    updateForgingStatus().catch(() => {
      // 忽略初始拉取失败
    })
  }
  // 锻造状态轮询（每 30s 一次，对标 nrs.js 中的 updateForgingStatus 轮询）
  forgingPollingTimer = setInterval(() => {
    if (accountStore.isLoggedIn && serverConnected.value) {
      updateForgingStatus().catch(() => {
        // 忽略轮询失败
      })
    }
  }, 30000)
})

onUnmounted(() => {
  if (forgingPollingTimer) {
    clearInterval(forgingPollingTimer)
    forgingPollingTimer = null
  }
})
</script>

<style scoped lang="scss">
.nrcs-header {
  display: flex;
  flex-direction: column;
  background: $header-bg;
  border-bottom: 1px solid $border-subtle;
  backdrop-filter: blur(20px);
  position: relative;
  z-index: $z-header;

  // ── 分叉警告 Banner（对标 nrs.js fork_warning 显示） ──
  .fork-warning-banner {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px $space-xl;
    background: rgba($danger, 0.15);
    color: $danger;
    font-size: $font-size-xs;
    font-weight: 600;
    border-bottom: 1px solid rgba($danger, 0.3);
    animation: pulse-warning 2s infinite;

    &.fork-base-target {
      background: rgba($warning, 0.15);
      color: $warning;
      border-bottom-color: rgba($warning, 0.3);
    }
  }

  @keyframes pulse-warning {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.7; }
  }

  // ── 下载进度条（对标 nrs.js updateBlockchainDownloadProgress） ──
  .download-progress-bar {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 4px $space-xl;
    background: rgba($primary, 0.08);
    font-size: $font-size-xs;
    color: $text-secondary;
    border-bottom: 1px solid rgba($primary, 0.15);

    .download-label { font-weight: 600; color: $primary; }
    .download-progress { flex: 1; max-width: 300px; }
    .download-percent { font-weight: 700; color: $primary; min-width: 36px; }
    .download-blocks-left { color: $text-muted; }
  }

  .header-main-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: $header-height;
    padding: 0 $space-xl;

    &::after {
      content: '';
      position: absolute;
      bottom: 0; left: 0; right: 0;
      height: 1px;
      background: linear-gradient(90deg, transparent, rgba($primary, 0.3), transparent);
    }
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

    // ── 测试网标识 ──
    .testnet-badge {
      display: flex;
      align-items: center;
      gap: 4px;
      padding: 3px 10px;
      border-radius: $radius-full;
      background: rgba($warning, 0.15);
      color: $warning;
      font-size: $font-size-xs;
      font-weight: 700;
      border: 1px solid rgba($warning, 0.3);
      margin-right: 4px;

      .testnet-dot {
        width: 6px; height: 6px;
        border-radius: 50%;
        background: $warning;
        box-shadow: 0 0 4px rgba($warning, 0.6);
      }
    }

    // ── 节点状态指示器 ──
    .node-status-indicator {
      display: flex;
      align-items: center;
      gap: 6px;
      padding: 4px 10px;
      border-radius: $radius-sm;
      background: rgba($bg-elevated, 0.4);
      border: 1px solid $border-subtle;
      cursor: default;
      transition: all $duration-fast ease;

      &:hover { border-color: $border-default; }

      .status-dot {
        width: 7px; height: 7px;
        border-radius: 50%;
        background: $danger;
        box-shadow: 0 0 4px rgba($danger, 0.4);

        &.online { background: $success; box-shadow: 0 0 4px rgba($success, 0.4); }
        &.offline { background: $danger; }
      }

      .status-height {
        font-size: $font-size-xs;
        color: $text-secondary;
        font-weight: 600;
      }
    }

    // ── 锻造状态指示器（对标 forgingIndicator） ──
    .forging-indicator {
      display: flex;
      align-items: center;
      gap: 6px;
      padding: 4px 10px;
      border-radius: $radius-sm;
      background: rgba($bg-elevated, 0.4);
      border: 1px solid $border-subtle;
      cursor: pointer;
      transition: all $duration-fast ease;

      &:hover {
        border-color: $border-glow;
        background: $bg-hover;
      }

      &.forging {
        border-color: rgba($success, 0.4);
        background: rgba($success, 0.08);
      }

      .forging-dot {
        width: 7px; height: 7px;
        border-radius: 50%;
        box-shadow: 0 0 4px currentColor;
      }

      .forging-label {
        font-size: $font-size-xs;
        font-weight: 600;
        color: $text-secondary;
      }

      &.forging .forging-label { color: $success; }
      &.not_forging .forging-label { color: $text-muted; }
    }

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

// ── 通知抽屉样式 ──
.notifications-container {
  padding: 0 4px;

  .notification-categories {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .notification-category-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 12px;
    border-radius: $radius-sm;
    cursor: pointer;
    transition: all $duration-fast ease;
    border: 1px solid transparent;

    &:hover {
      background: $bg-hover;
      border-color: $border-subtle;
    }

    .cat-info {
      display: flex;
      align-items: center;
      gap: 10px;

      .cat-label {
        font-size: $font-size-sm;
        color: $text-primary;
        font-weight: 500;
      }
    }

    .cat-arrow {
      color: $text-muted;
      font-size: 12px;
    }
  }

  .notification-section {
    margin-bottom: 16px;

    .section-title {
      font-size: $font-size-xs;
      color: $text-muted;
      text-transform: uppercase;
      letter-spacing: $letter-spacing-wide;
      margin-bottom: 8px;
      font-weight: 600;
    }

    .section-item {
      display: flex;
      align-items: center;
      gap: 10px;
      padding: 8px 12px;
      border-radius: $radius-sm;
      cursor: pointer;
      transition: all $duration-fast ease;

      &:hover { background: $bg-hover; }

      span {
        font-size: $font-size-sm;
        color: $text-secondary;
      }
    }
  }

  .no-notifications {
    padding: 40px 0;
  }

  .mark-all-read {
    text-align: center;
    margin-top: 16px;
    padding-top: 12px;
    border-top: 1px solid $divider;
  }
}

// ── 客户端状态模态框 ──
.api-proxy-section {
  margin-top: 20px;

  .api-proxy-actions {
    display: flex;
    gap: 8px;
    margin-top: 10px;
  }
}

@keyframes pulse-glow {
  0%, 100% { box-shadow: 0 2px 4px rgba(0, 0, 0, 0.3), 0 0 0 0 rgba($danger, 0.4); }
  50% { box-shadow: 0 2px 4px rgba(0, 0, 0, 0.3), 0 0 0 6px rgba($danger, 0); }
}
</style>
