<template>
  <div class="asset-exchange-page">
    <div class="page-header">
      <h2 class="page-title"><el-icon><TrendCharts /></el-icon> {{ t('asset.exchange') }}</h2>
      <div class="header-actions">
        <el-button type="primary" size="small" @click="showIssueAsset = true">
          <el-icon><Plus /></el-icon> {{ t('asset.issueAsset') }}
        </el-button>
        <el-button size="small" @click="showAddBookmark = true">
          <el-icon><Star /></el-icon> {{ t('asset.addBookmark') }}
        </el-button>
        <el-button size="small" @click="refreshData"><el-icon><Refresh /></el-icon> {{ t('common.refresh') }}</el-button>
      </div>
    </div>

    <el-row :gutter="16">
      <!-- 资产收藏侧栏（对标 nrs.assetexchange.js:311 loadAssetExchangeSidebar） -->
      <el-col :span="6">
        <el-card shadow="hover" class="sidebar-card">
          <template #header>
            <div class="sidebar-header">
              <div class="sidebar-title">
                <span>{{ t('asset.bookmarks') }}</span>
                <el-tag size="small" type="info">{{ bookmarks.length }}</el-tag>
              </div>
              <el-input
                v-model="searchQuery"
                :placeholder="t('asset.searchPlaceholder')"
                size="small"
                clearable
                class="search-input"
              >
                <template #prefix><el-icon><Search /></el-icon></template>
              </el-input>
            </div>
          </template>
          <div class="asset-list" v-loading="loadingBookmarks">
            <!-- 分组展示（对标 nrs.assetexchange.js:356-410 分组渲染） -->
            <template v-for="group in groupedBookmarks" :key="group.name || '__ungrouped__'">
              <div v-if="group.name" class="group-header" @click="toggleGroup(group.name)">
                <span class="group-name">{{ group.name.toUpperCase() }}</span>
                <el-icon class="group-icon">
                  <ArrowDown v-if="!collapsedGroups.has(group.name)" />
                  <ArrowRight v-else />
                </el-icon>
              </div>
              <div v-show="!group.name || !collapsedGroups.has(group.name)">
                <div
                  v-for="asset in group.items"
                  :key="asset.asset"
                  class="asset-item"
                  :class="{
                    active: selectedAsset?.asset === asset.asset,
                    owned: ownedAssetIds.has(asset.asset),
                  }"
                  @click="selectAsset(asset)"
                >
                  <div class="asset-name">{{ asset.name }}</div>
                  <div class="asset-id">{{ truncateHash(asset.asset, 6) }}</div>
                  <div v-if="ownedAssetIds.has(asset.asset)" class="owned-badge">{{ t('asset.owned') }}</div>
                </div>
              </div>
            </template>
            <el-empty
              v-if="filteredBookmarks.length === 0 && !loadingBookmarks"
              :description="t('asset.noBookmarks')"
              :image-size="48"
            />
          </div>
        </el-card>
      </el-col>

      <!-- 主内容区 -->
      <el-col :span="18">
        <!-- 资产详情 -->
        <el-card v-if="selectedAsset" shadow="hover" class="detail-card">
          <template #header>
            <div class="detail-header">
              <div class="detail-title">
                <h3>{{ selectedAsset.name }}</h3>
                <el-tag size="small" type="info">{{ selectedAsset.decimals }} {{ t('asset.decimals') }}</el-tag>
                <el-tag v-if="isBookmarked" size="small" type="warning">
                  <el-icon><Star /></el-icon> {{ t('asset.bookmarked') }}
                </el-tag>
              </div>
              <div class="detail-actions">
                <el-tooltip v-if="!isBookmarked" :content="t('asset.addBookmark')" placement="top">
                  <el-button size="small" circle @click="bookmarkSelectedAsset">
                    <el-icon><Star /></el-icon>
                  </el-button>
                </el-tooltip>
                <el-tooltip v-else :content="t('asset.removeBookmark')" placement="top">
                  <el-button size="small" circle type="danger" @click="removeSelectedAssetBookmark">
                    <el-icon><StarFilled /></el-icon>
                  </el-button>
                </el-tooltip>
                <el-tooltip :content="t('asset.viewAssetDistribution')" placement="top">
                  <el-button size="small" circle @click="showHolders = true">
                    <el-icon><User /></el-icon>
                  </el-button>
                </el-tooltip>
                <el-tooltip :content="t('asset.viewDividendHistory')" placement="top">
                  <el-button size="small" circle @click="showDividends = true">
                    <el-icon><Money /></el-icon>
                  </el-button>
                </el-tooltip>
              </div>
            </div>
          </template>
          <el-descriptions :column="3" border size="small">
            <el-descriptions-item :label="t('common.id')">{{ truncateHash(selectedAsset.asset, 8) }}</el-descriptions-item>
            <el-descriptions-item :label="t('asset.quantity')">{{ formatQNT(selectedAsset.quantityQNT, selectedAsset.decimals) }}</el-descriptions-item>
            <el-descriptions-item :label="t('common.issuer')">{{ selectedAsset.issuerRS }}</el-descriptions-item>
            <el-descriptions-item v-if="yourBalance" :label="t('asset.myAssets')">
              {{ formatQNT(yourBalance.quantityQNT, selectedAsset.decimals) }}
            </el-descriptions-item>
            <el-descriptions-item v-if="selectedAsset.description" :label="t('asset.description')" :span="3">
              {{ selectedAsset.description }}
            </el-descriptions-item>
          </el-descriptions>
          <div class="action-buttons">
            <el-button type="primary" size="small" @click="openBuyOrder">
              <el-icon><ShoppingCart /></el-icon> {{ t('asset.buy') }}
            </el-button>
            <el-button type="success" size="small" @click="openSellOrder">
              <el-icon><Sell /></el-icon> {{ t('asset.sell') }}
            </el-button>
            <el-button size="small" @click="openTransfer">
              <el-icon><Sort /></el-icon> {{ t('asset.transfer') }}
            </el-button>
          </div>
        </el-card>

        <!-- 订单簿（对标 nrs.assetexchange.js:717 loadAssetOrders，含预期订单） -->
        <el-row :gutter="16" v-if="selectedAsset">
          <el-col :span="12">
            <el-card shadow="hover">
              <template #header>
                <div class="card-header-row">
                  <h3 class="card-title">{{ t('asset.buyOrders') }}</h3>
                  <el-tooltip :content="t('asset.buyOrdersHint')" placement="top">
                    <el-icon class="hint-icon"><InfoFilled /></el-icon>
                  </el-tooltip>
                </div>
              </template>
              <el-table
                :data="bidOrders"
                stripe
                size="small"
                v-loading="loadingOrders"
                :empty-text="t('common.noData')"
                @row-click="(row: any) => onOrderRowClick(row, 'sell')"
              >
                <el-table-column :label="t('asset.price')" width="120" align="right">
                  <template #default="{ row }">
                    <span class="clickable">{{ formatOrderPrice(row.priceNQT, selectedAsset.decimals) }} NRC</span>
                  </template>
                </el-table-column>
                <el-table-column :label="t('asset.quantity')" width="100" align="right">
                  <template #default="{ row }">{{ formatQNT(row.quantityQNT, selectedAsset.decimals) }}</template>
                </el-table-column>
                <el-table-column :label="t('asset.total')" width="120" align="right">
                  <template #default="{ row }">{{ calculateOrderTotal(row.quantityQNT, row.priceNQT) }} NRC</template>
                </el-table-column>
                <el-table-column :label="t('asset.expected')" width="80" align="center">
                  <template #default="{ row }">
                    <el-tag v-if="row.isExpected" size="small" type="warning">{{ t('asset.expectedShort') }}</el-tag>
                  </template>
                </el-table-column>
              </el-table>
            </el-card>
          </el-col>
          <el-col :span="12">
            <el-card shadow="hover">
              <template #header>
                <div class="card-header-row">
                  <h3 class="card-title">{{ t('asset.sellOrders') }}</h3>
                  <el-tooltip :content="t('asset.sellOrdersHint')" placement="top">
                    <el-icon class="hint-icon"><InfoFilled /></el-icon>
                  </el-tooltip>
                </div>
              </template>
              <el-table
                :data="askOrders"
                stripe
                size="small"
                v-loading="loadingOrders"
                :empty-text="t('common.noData')"
                @row-click="(row: any) => onOrderRowClick(row, 'buy')"
              >
                <el-table-column :label="t('asset.price')" width="120" align="right">
                  <template #default="{ row }">
                    <span class="clickable">{{ formatOrderPrice(row.priceNQT, selectedAsset.decimals) }} NRC</span>
                  </template>
                </el-table-column>
                <el-table-column :label="t('asset.quantity')" width="100" align="right">
                  <template #default="{ row }">{{ formatQNT(row.quantityQNT, selectedAsset.decimals) }}</template>
                </el-table-column>
                <el-table-column :label="t('asset.total')" width="120" align="right">
                  <template #default="{ row }">{{ calculateOrderTotal(row.quantityQNT, row.priceNQT) }} NRC</template>
                </el-table-column>
                <el-table-column :label="t('asset.expected')" width="80" align="center">
                  <template #default="{ row }">
                    <el-tag v-if="row.isExpected" size="small" type="warning">{{ t('asset.expectedShort') }}</el-tag>
                  </template>
                </el-table-column>
              </el-table>
            </el-card>
          </el-col>
        </el-row>

        <!-- 交易历史 -->
        <el-card v-if="selectedAsset" shadow="hover" class="mt-16">
          <template #header>
            <div class="section-header">
              <h3 class="card-title">{{ t('asset.tradeHistory') }}</h3>
              <el-radio-group v-model="tradeFilter" size="small" @change="loadTrades">
                <el-radio-button value="everyone">{{ t('dashboard.total') }}</el-radio-button>
                <el-radio-button value="you">{{ t('dashboard.owned') }}</el-radio-button>
              </el-radio-group>
            </div>
          </template>
          <el-table :data="trades" stripe size="small" v-loading="loadingTrades" :empty-text="t('common.noData')">
            <el-table-column :label="t('common.date')" width="160">
              <template #default="{ row }">{{ formatTimestamp(row.timestamp) }}</template>
            </el-table-column>
            <el-table-column :label="t('asset.type')" width="80">
              <template #default="{ row }">
                <el-tag size="small" :type="row.buyerRS === accountRS ? 'success' : 'danger'">
                  {{ row.buyerRS === accountRS ? t('asset.buy') : t('asset.sell') }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column :label="t('asset.quantity')" width="100" align="right">
              <template #default="{ row }">{{ formatQNT(row.quantityQNT, selectedAsset.decimals) }}</template>
            </el-table-column>
            <el-table-column :label="t('asset.price')" width="120" align="right">
              <template #default="{ row }">{{ formatOrderPrice(row.priceNQT, selectedAsset.decimals) }} NRC</template>
            </el-table-column>
            <el-table-column :label="t('common.buyer')" min-width="160">
              <template #default="{ row }">{{ row.buyerRS }}</template>
            </el-table-column>
            <el-table-column :label="t('common.seller')" min-width="160">
              <template #default="{ row }">{{ row.sellerRS }}</template>
            </el-table-column>
          </el-table>
        </el-card>

        <!-- 未选择资产 -->
        <el-empty v-if="!selectedAsset" :description="t('asset.selectAsset')" :image-size="80" />
      </el-col>
    </el-row>

    <!-- 弹窗 -->
    <AssetOrderModal
      v-model:visible="showOrderModal"
      :asset="selectedAsset"
      :order-type="orderType"
      :your-asset-balance-q-n-t="yourBalance?.quantityQNT"
      :prefill="orderPrefill"
      @success="onOrderSuccess"
    />
    <TransferAssetModal
      v-model:visible="showTransferModal"
      :asset="selectedAsset"
      :your-balance-q-n-t="yourBalance?.quantityQNT"
      @success="refreshData"
    />
    <IssueAssetModal v-model:visible="showIssueAsset" @success="onIssueSuccess" />
    <AddAssetBookmarkModal v-model:visible="showAddBookmark" @success="onBookmarkAdded" />
    <AssetHoldersModal v-model:visible="showHolders" :asset="selectedAsset" />
    <AssetDividendHistoryModal v-model:visible="showDividends" :asset="selectedAsset" />
  </div>
</template>

<script setup lang="ts">
/**
 * AssetExchange 视图 —— 资产交易页面。
 *
 * 对标 nrs.assetexchange.js（2109 行）的核心功能：
 *   - 资产收藏侧栏（IndexedDB 持久化，对标 :311 loadAssetExchangeSidebar）
 *   - 分组展示与折叠（对标 :356-410 分组渲染）
 *   - 搜索过滤（对标 :872-904 assetExchangeSearch）
 *   - 订单簿（含预期订单，对标 :717 loadAssetOrders）
 *   - 订单点击填充（对标 :929-976 行点击事件）
 *   - 交易历史（对标 :809 getAssetTradeHistory）
 *   - 资产持有人（对标 :1164 getAssetAccounts）
 *   - 股息历史（对标 :764 getAssetDividendHistory）
 *   - 自动收藏持有资产（对标 :89-127 检查 unconfirmedAssetBalances）
 */
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  TrendCharts, Plus, Refresh, Search, ShoppingCart, Sell, Sort,
  Star, StarFilled, User, Money, InfoFilled, ArrowDown, ArrowRight,
} from '@element-plus/icons-vue'
import { nrcsApi, type NrcsAsset } from '@/api/modules/nrcs.api'
import { useAccountStore } from '@/stores/modules/account.store'
import { usePolling } from '@/composables/usePolling'
import { useAssetBookmarks, type AssetBookmark } from '@/composables/useAssetBookmarks'
import {
  formatTimestamp, formatOrderPricePerWholeQNT, calculateOrderTotalNQT,
  qntToQntf, truncateHash,
} from '@/utils/format'
import AssetOrderModal from '@/components/modals/AssetOrderModal.vue'
import TransferAssetModal from '@/components/modals/TransferAssetModal.vue'
import IssueAssetModal from '@/components/modals/IssueAssetModal.vue'
import AddAssetBookmarkModal from '@/components/modals/AddAssetBookmarkModal.vue'
import AssetHoldersModal from '@/components/modals/AssetHoldersModal.vue'
import AssetDividendHistoryModal from '@/components/modals/AssetDividendHistoryModal.vue'

const { t } = useI18n()
const accountStore = useAccountStore()
const {
  bookmarks,
  isLoading: loadingBookmarks,
  loadBookmarks,
  saveBookmarks,
  removeBookmark,
  fromApiResponse,
  cacheAsset,
  searchBookmarks,
  groupNames,
} = useAssetBookmarks()

const accountRS = computed(() => accountStore.accountRS)

// --- 状态 ---
const searchQuery = ref('')
const loadingOrders = ref(false)
const loadingTrades = ref(false)
const selectedAsset = ref<NrcsAsset | null>(null)
const yourBalance = ref<{ quantityQNT: string } | null>(null)
const bidOrders = ref<any[]>([])
const askOrders = ref<any[]>([])
const trades = ref<any[]>([])
const tradeFilter = ref('everyone')
/** 已持有资产 ID 集合（用于侧栏 owned 标记，对标 NRS.accountInfo.unconfirmedAssetBalances） */
const ownedAssetIds = ref<Set<string>>(new Set())
/** 折叠的分组名称集合 */
const collapsedGroups = ref<Set<string>>(new Set())

// 弹窗
const showOrderModal = ref(false)
const showTransferModal = ref(false)
const showIssueAsset = ref(false)
const showAddBookmark = ref(false)
const showHolders = ref(false)
const showDividends = ref(false)
const orderType = ref<'buy' | 'sell'>('buy')
/** 订单弹窗预填值（点击订单簿行时填充） */
const orderPrefill = ref<{ quantity?: string; pricePerShare?: string } | null>(null)

// --- 计算属性 ---
/** 按搜索关键词过滤后的收藏 */
const filteredBookmarks = computed(() => searchBookmarks(searchQuery.value))

/** 按分组组织收藏（对标 nrs.assetexchange.js:328 排序 + :356 分组渲染） */
const groupedBookmarks = computed(() => {
  const groups: { name: string; items: AssetBookmark[] }[] = []
  // 已命名的分组
  for (const name of groupNames.value) {
    const items = filteredBookmarks.value.filter((b) => b.groupName === name)
    items.sort((a, b) => a.name.toLowerCase().localeCompare(b.name.toLowerCase()))
    groups.push({ name, items })
  }
  // 未分组
  const ungrouped = filteredBookmarks.value.filter((b) => !b.groupName)
  ungrouped.sort((a, b) => a.name.toLowerCase().localeCompare(b.name.toLowerCase()))
  if (ungrouped.length > 0) {
    groups.push({ name: '', items: ungrouped })
  }
  return groups
})

/** 当前选中的资产是否已收藏 */
const isBookmarked = computed(() => {
  if (!selectedAsset.value) return false
  return bookmarks.value.some((b) => b.asset === selectedAsset.value!.asset)
})

// --- 方法 ---
/** 格式化 QNT 为可读数量 */
function formatQNT(qnt: string | undefined, decimals: number): string {
  if (!qnt) return '0'
  try { return qntToQntf(qnt, decimals) } catch { return qnt }
}

/** 格式化订单价格 */
function formatOrderPrice(price: string, decimals: number): string {
  try { return formatOrderPricePerWholeQNT(price, decimals) } catch { return price }
}

/** 计算订单总金额 */
function calculateOrderTotal(quantityQNT: string, priceNQT: string): string {
  try { return calculateOrderTotalNQT(quantityQNT, priceNQT) } catch { return '0' }
}

/** 折叠/展开分组 */
function toggleGroup(name: string): void {
  if (collapsedGroups.value.has(name)) {
    collapsedGroups.value.delete(name)
  } else {
    collapsedGroups.value.add(name)
  }
}

/**
 * 加载资产收藏 + 自动收藏持有资产。
 *
 * 对标 NRS.pages.asset_exchange（:79-129）：
 *   1. 从 IndexedDB 加载已收藏资产
 *   2. 检查账户的 unconfirmedAssetBalances，将未收藏的持有资产自动加入收藏
 */
async function loadAssetBookmarks(): Promise<void> {
  await loadBookmarks()

  // 自动收藏持有资产（对标 :89-127）
  if (accountRS.value) {
    try {
      const accountAssets = await nrcsApi.getAccountAssets(accountRS.value)
      const balances = accountAssets?.assetBalances || []
      const ownedIds = new Set<string>()
      const newBookmarks: AssetBookmark[] = []

      for (const balance of balances) {
        const assetId = String(balance.asset)
        ownedIds.add(assetId)
        // 若持有但未收藏，自动加入收藏（对标 :101-120 saveAssetBookmarks）
        if (!bookmarks.value.some((b) => b.asset === assetId)) {
          try {
            const detail = await nrcsApi.getAsset(assetId)
            if (detail && !(detail as any).errorCode) {
              newBookmarks.push(fromApiResponse(detail as NrcsAsset))
            }
          } catch {
            // 单个资产查询失败不阻断
          }
        }
      }
      ownedAssetIds.value = ownedIds

      if (newBookmarks.length > 0) {
        await saveBookmarks(newBookmarks)
      }
    } catch (e) {
      // getAccountAssets 失败不阻断收藏加载
      console.warn('Failed to load account assets:', e)
    }
  }
}

/**
 * 选择资产并加载详情。
 *
 * 对标 NRS.loadAsset（参考 :522-700）：
 *   - 加载资产详情（getAsset）
 *   - 加载用户持仓（getAccountAssets）
 *   - 加载订单簿（含预期订单）
 *   - 加载交易历史
 */
async function selectAsset(asset: AssetBookmark | NrcsAsset): Promise<void> {
  // 兼容 AssetBookmark 与 NrcsAsset 两种类型
  const assetId = String(asset.asset)
  const baseInfo: NrcsAsset = {
    asset: assetId,
    name: asset.name,
    description: (asset as any).description || '',
    quantityQNT: (asset as any).quantityQNT || '',
    decimals: (asset as any).decimals || 0,
    issuer: (asset as any).issuer || (asset as any).account || '',
    issuerRS: (asset as any).issuerRS || (asset as any).accountRS || '',
  }
  selectedAsset.value = baseInfo
  yourBalance.value = null
  bidOrders.value = []
  askOrders.value = []
  trades.value = []

  if (!accountRS.value) return

  // 加载资产详情
  try {
    const detail = await nrcsApi.getAsset(assetId)
    if (detail && !(detail as any).errorCode) {
      selectedAsset.value = { ...baseInfo, ...(detail as NrcsAsset) }
    }
  } catch {
    // 详情加载失败保留基础信息
  }

  // 加载用户持仓
  try {
    const accountAssets = await nrcsApi.getAccountAssets(accountRS.value)
    const balances = accountAssets?.assetBalances || []
    const found = balances.find((b: any) => b.asset === assetId)
    if (found) {
      yourBalance.value = { quantityQNT: found.quantityQNT || found.unconfirmedQuantityQNT }
    }
  } catch {
    // 持仓查询失败不阻断
  }

  // 加载订单簿（含预期订单）
  await loadOrders(assetId)

  // 加载交易历史
  loadTrades()
}

/**
 * 加载订单簿（含预期订单）。
 *
 * 对标 NRS.loadAssetOrders（:717-762）：
 *   并行拉取真实订单（getBidOrders/getAskOrders）与预期订单
 *   （getExpectedBidOrders/getExpectedAskOrders），合并后按价格排序。
 *
 * @param assetId 资产 ID
 */
async function loadOrders(assetId: string): Promise<void> {
  loadingOrders.value = true
  try {
    const [bidsRes, expectedBidsRes, asksRes, expectedAsksRes] = await Promise.all([
      nrcsApi.getBidOrders(assetId, 0, 25).catch(() => ({ bidOrders: [] })),
      nrcsApi.getExpectedBidOrders(assetId).catch(() => ({ bidOrders: [] })),
      nrcsApi.getAskOrders(assetId, 0, 25).catch(() => ({ askOrders: [] })),
      nrcsApi.getExpectedAskOrders(assetId).catch(() => ({ askOrders: [] })),
    ])

    // 合并真实 + 预期订单，标记 isExpected
    const bids = [
      ...((bidsRes as any).bidOrders || []).map((o: any) => ({ ...o, isExpected: false })),
      ...((expectedBidsRes as any).bidOrders || []).map((o: any) => ({ ...o, isExpected: true })),
    ]
    const asks = [
      ...((asksRes as any).askOrders || []).map((o: any) => ({ ...o, isExpected: false })),
      ...((expectedAsksRes as any).askOrders || []).map((o: any) => ({ ...o, isExpected: true })),
    ]

    // 排序：bid 降序（价格高在前），ask 升序（价格低在前）
    bids.sort((a, b) => Number(b.priceNQT) - Number(a.priceNQT))
    asks.sort((a, b) => Number(a.priceNQT) - Number(b.priceNQT))

    bidOrders.value = bids
    askOrders.value = asks
  } catch (e) {
    console.error('Failed to load orders:', e)
  } finally {
    loadingOrders.value = false
  }
}

/**
 * 加载交易历史。
 *
 * 对标 NRS.getAssetTradeHistory（:809-864）。
 */
async function loadTrades(): Promise<void> {
  if (!selectedAsset.value) return
  loadingTrades.value = true
  try {
    const options: any = { firstIndex: 0, lastIndex: 50 }
    if (tradeFilter.value === 'you' && accountRS.value) {
      options.account = accountRS.value
    }
    const result = await nrcsApi.getTrades(selectedAsset.value.asset, 0, 50)
    const allTrades = result?.trades || []
    if (tradeFilter.value === 'you' && accountRS.value) {
      trades.value = allTrades.filter(
        (tr: any) => tr.buyerRS === accountRS.value || tr.sellerRS === accountRS.value,
      )
    } else {
      trades.value = allTrades
    }
  } catch (e) {
    console.error('Failed to load trades:', e)
  } finally {
    loadingTrades.value = false
  }
}

/**
 * 订单簿行点击：填充下单表单。
 *
 * 对标 nrs.assetexchange.js:929-976 行点击事件：
 *   - 点击卖单 → 填充买单（buy）表单
 *   - 点击买单 → 填充卖单（sell）表单
 *   - 自动填充价格和数量
 *
 * @param row 订单行数据
 * @param type 触发的订单类型：'buy' 或 'sell'
 */
function onOrderRowClick(row: any, type: 'buy' | 'sell'): void {
  if (!selectedAsset.value) return
  orderType.value = type
  orderPrefill.value = {
    quantity: formatQNT(row.quantityQNT, selectedAsset.value.decimals),
    pricePerShare: formatOrderPrice(row.priceNQT, selectedAsset.value.decimals),
  }
  showOrderModal.value = true
}

function openBuyOrder(): void {
  orderType.value = 'buy'
  orderPrefill.value = null
  showOrderModal.value = true
}

function openSellOrder(): void {
  orderType.value = 'sell'
  orderPrefill.value = null
  showOrderModal.value = true
}

function openTransfer(): void {
  showTransferModal.value = true
}

/** 订单成功回调：刷新订单簿与交易历史 */
function onOrderSuccess(): void {
  loadTrades()
  if (selectedAsset.value) {
    loadOrders(selectedAsset.value.asset)
  }
}

/** 发行资产成功回调：刷新收藏 */
function onIssueSuccess(): void {
  loadAssetBookmarks()
}

/** 添加收藏成功回调 */
function onBookmarkAdded(): void {
  loadAssetBookmarks()
}

/** 将当前选中资产加入收藏 */
async function bookmarkSelectedAsset(): Promise<void> {
  if (!selectedAsset.value) return
  try {
    const bookmark = fromApiResponse(selectedAsset.value)
    const added = await saveBookmarks([bookmark])
    if (added.length > 0) {
      ElMessage.success(t('asset.bookmarkAddedSuccess', { count: 1 }))
    } else {
      ElMessage.info(t('asset.bookmarkAlreadyExists'))
    }
  } catch (e: any) {
    ElMessage.error(e?.message || t('asset.bookmarkAddError'))
  }
}

/** 移除当前选中资产的收藏 */
async function removeSelectedAssetBookmark(): Promise<void> {
  if (!selectedAsset.value) return
  // 持有的资产不允许移除收藏（对标 :1267-1281）
  if (ownedAssetIds.value.has(selectedAsset.value.asset)) {
    ElMessage.warning(t('asset.cannotRemoveOwnedBookmark'))
    return
  }
  try {
    await ElMessageBox.confirm(
      t('asset.removeBookmarkConfirm'),
      t('common.confirm'),
      { type: 'warning' },
    )
    await removeBookmark(selectedAsset.value.asset, true)
    ElMessage.success(t('asset.bookmarkRemoved'))
  } catch (e: any) {
    if (e === 'cancel') return
    ElMessage.error(e?.message || t('asset.bookmarkRemoveError'))
  }
}

/** 刷新数据 */
async function refreshData(): Promise<void> {
  await loadAssetBookmarks()
  if (selectedAsset.value) {
    await selectAsset(selectedAsset.value as any)
  }
}

// --- 生命周期 ---
onMounted(() => {
  loadAssetBookmarks()
})

usePolling(refreshData, 30000, false)
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;

.asset-exchange-page {
  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
    .page-title {
      font-size: 18px;
      font-weight: 600;
      display: flex;
      align-items: center;
      gap: 8px;
      margin: 0;
    }
    .header-actions {
      display: flex;
      gap: 8px;
    }
  }
  .sidebar-card {
    .sidebar-header {
      display: flex;
      flex-direction: column;
      gap: 8px;
      .sidebar-title {
        display: flex;
        align-items: center;
        gap: 8px;
        font-weight: 600;
      }
      .search-input { width: 100%; }
    }
    .asset-list {
      max-height: 60vh;
      overflow-y: auto;
      .group-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 6px 12px;
        cursor: pointer;
        font-size: 11px;
        font-weight: 700;
        color: $text-muted;
        text-transform: uppercase;
        border-bottom: 1px solid $border-subtle;
        margin-top: 4px;
        &:hover { background: var(--el-fill-color-light); }
        .group-icon { font-size: 10px; }
      }
      .asset-item {
        padding: 8px 12px;
        cursor: pointer;
        border-radius: 4px;
        margin-bottom: 2px;
        transition: background 0.2s;
        position: relative;
        &:hover { background: var(--el-fill-color-light); }
        &.active {
          background: var(--el-color-primary-light-9);
          border-left: 3px solid var(--el-color-primary);
        }
        &.owned .asset-name::after {
          content: '';
        }
        .asset-name { font-weight: 500; font-size: 13px; }
        .asset-id { font-size: 11px; color: $text-muted; }
        .owned-badge {
          position: absolute;
          top: 4px;
          right: 8px;
          font-size: 10px;
          color: $success;
        }
      }
    }
  }
  .detail-card {
    margin-bottom: 16px;
    .detail-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      .detail-title {
        display: flex;
        align-items: center;
        gap: 8px;
        h3 { margin: 0; font-size: 16px; }
      }
      .detail-actions {
        display: flex;
        gap: 4px;
      }
    }
    .action-buttons {
      margin-top: 12px;
      display: flex;
      gap: 8px;
    }
  }
  .card-header-row {
    display: flex;
    align-items: center;
    gap: 6px;
    .hint-icon {
      color: $text-muted;
      cursor: help;
      font-size: 14px;
    }
  }
  .card-title { margin: 0; font-size: 15px; color: $text-primary; }
  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .mt-16 { margin-top: 16px; }
  .clickable {
    cursor: pointer;
    &:hover { text-decoration: underline; color: $primary; }
  }
}
</style>
