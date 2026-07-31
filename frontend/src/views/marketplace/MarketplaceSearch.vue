<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><ShoppingCart /></el-icon> {{ t('marketplace.title') }}</h2>
      <div class="header-actions">
        <el-button type="primary" size="small" @click="showListing = true">
          <el-icon><Plus /></el-icon> {{ t('marketplace.listProduct') }}
        </el-button>
        <el-button size="small" @click="refreshData">
          <el-icon><Refresh /></el-icon> {{ t('common.refresh') }}
        </el-button>
      </div>
    </div>

    <el-row :gutter="16">
      <!-- Main content -->
      <el-col :span="18">
        <!-- Search & filters -->
        <el-card shadow="hover" class="search-card">
          <el-row :gutter="12">
            <el-col :span="8">
              <el-input
                v-model="filters.query"
                :placeholder="t('marketplace.searchPlaceholder', 'Search products...')"
                clearable
                @change="onFilterChange"
                @keyup.enter="onFilterChange"
              >
                <template #prefix>
                  <el-icon><Search /></el-icon>
                </template>
              </el-input>
            </el-col>
            <el-col :span="6">
              <el-input
                v-model="filters.seller"
                :placeholder="t('marketplace.sellerPlaceholder')"
                clearable
                @change="onFilterChange"
              >
                <template #prefix>
                  <el-icon><User /></el-icon>
                </template>
              </el-input>
            </el-col>
            <el-col :span="6">
              <el-input
                v-model="filters.tag"
                :placeholder="t('marketplace.tagPlaceholder')"
                clearable
                @change="onFilterChange"
              >
                <template #prefix>
                  <el-icon><PriceTag /></el-icon>
                </template>
              </el-input>
            </el-col>
            <el-col :span="4">
              <el-checkbox v-model="filters.inStockOnly" @change="onFilterChange" style="margin-top: 6px">
                {{ t('marketplace.inStockOnly') }}
              </el-checkbox>
            </el-col>
          </el-row>
        </el-card>

        <!-- Tag cloud -->
        <el-card shadow="hover" class="tag-cloud-card" v-if="tags.length > 0">
          <template #header>
            <span class="card-header-title">{{ t('marketplace.popularTags', 'Popular Tags') }}</span>
          </template>
          <div class="tag-cloud">
            <el-tag
              v-for="tag in tags"
              :key="tag.name"
              :type="filters.tag === tag.name ? 'primary' : 'info'"
              :class="['cloud-tag', { active: filters.tag === tag.name }]"
              :style="{ fontSize: Math.max(12, Math.min(20, 12 + tag.count * 2)) + 'px' }"
              effect="plain"
              @click="selectTag(tag.name)"
            >
              {{ tag.name }}
              <sup class="tag-count">{{ tag.count }}</sup>
            </el-tag>
          </div>
        </el-card>

        <!-- Product grid -->
        <el-card shadow="hover" v-loading="loading">
          <template #header>
            <div class="card-header-row">
              <span class="card-header-title">{{ t('marketplace.products', 'Products') }} ({{ total }})</span>
            </div>
          </template>
          <div v-if="items.length === 0 && !loading" class="empty-products">
            <el-empty :description="t('common.noData')" />
          </div>
          <div v-else class="product-grid">
            <div
              v-for="product in items"
              :key="product.goods"
              class="product-card"
              @click="openPurchase(product)"
            >
              <div class="product-card-body">
                <h4 class="product-name">{{ product.name }}</h4>
                <p class="product-desc" v-if="product.description">{{ product.description }}</p>
                <div class="product-tags" v-if="product.tags">
                  <el-tag
                    v-for="t in parseTags(product.tags)"
                    :key="t"
                    size="small"
                    type="info"
                    class="product-tag"
                  >
                    {{ t }}
                  </el-tag>
                </div>
              </div>
              <div class="product-card-footer">
                <span class="product-price">{{ formatNQT(product.priceNQT) }} NRC</span>
                <span class="product-qty">
                  {{ t('marketplace.quantity') }}: <strong>{{ product.quantity }}</strong>
                </span>
              </div>
              <div class="product-card-seller">
                <el-icon size="14"><User /></el-icon>
                <span class="seller-text">{{ product.sellerRS || truncateHash(product.seller) }}</span>
              </div>
            </div>
          </div>
          <div class="pagination-container" v-if="total > 20">
            <el-pagination
              v-model:current-page="page"
              :page-size="20"
              :total="total"
              layout="prev,pager,next"
              @current-change="refreshData"
            />
          </div>
        </el-card>
      </el-col>

      <!-- Sidebar -->
      <el-col :span="6">
        <!-- Stats bar -->
        <el-card shadow="hover" class="sidebar-card">
          <template #header>
            <span class="card-header-title">{{ t('marketplace.stats', 'Market Stats') }}</span>
          </template>
          <div class="stat-list">
            <div class="stat-item">
              <div class="stat-icon"><el-icon><Goods /></el-icon></div>
              <div class="stat-info">
                <span class="stat-num">{{ totalProducts }}</span>
                <span class="stat-label">{{ t('marketplace.totalProducts', 'Total Products') }}</span>
              </div>
            </div>
            <div class="stat-item">
              <div class="stat-icon"><el-icon><ShoppingCart /></el-icon></div>
              <div class="stat-info">
                <span class="stat-num">{{ totalPurchases }}</span>
                <span class="stat-label">{{ t('marketplace.totalPurchases', 'Total Purchases') }}</span>
              </div>
            </div>
            <div class="stat-item">
              <div class="stat-icon"><el-icon><PriceTag /></el-icon></div>
              <div class="stat-info">
                <span class="stat-num">{{ totalTags }}</span>
                <span class="stat-label">{{ t('marketplace.totalTags', 'Total Tags') }}</span>
              </div>
            </div>
          </div>
        </el-card>

        <!-- Recent listings -->
        <el-card shadow="hover" class="sidebar-card" v-if="recentListings.length > 0">
          <template #header>
            <span class="card-header-title">{{ t('marketplace.recentListings', 'Recent Listings') }}</span>
          </template>
          <div class="sidebar-list">
            <div v-for="rl in recentListings" :key="rl.goods" class="sidebar-list-item" @click="openPurchase(rl)">
              <span class="sidebar-item-name">{{ rl.name }}</span>
              <span class="sidebar-item-price">{{ formatNQT(rl.priceNQT) }} NRC</span>
              <span class="sidebar-item-seller">{{ rl.sellerRS || truncateHash(rl.seller) }}</span>
            </div>
          </div>
        </el-card>

        <!-- Recent purchases -->
        <el-card shadow="hover" class="sidebar-card" v-if="recentPurchases.length > 0">
          <template #header>
            <span class="card-header-title">{{ t('marketplace.recentPurchases', 'Recent Purchases') }}</span>
          </template>
          <div class="sidebar-list">
            <div v-for="rp in recentPurchases" :key="rp.purchase" class="sidebar-list-item">
              <span class="sidebar-item-name">{{ rp.name }}</span>
              <span class="sidebar-item-price">{{ formatNQT(rp.priceNQT) }} NRC</span>
              <span class="sidebar-item-seller">{{ rp.buyerRS || truncateHash(rp.buyer) }}</span>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <DGSListingModal v-model:visible="showListing" @success="refreshData" />
    <PurchaseProductModal v-model:visible="showPurchase" :product="selectedProduct" @success="refreshData" />
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { formatNrc, truncateHash, formatTimestamp } from '@/utils/format'
import DGSListingModal from '@/components/modals/DGSListingModal.vue'
import PurchaseProductModal from '@/components/modals/PurchaseProductModal.vue'

const { t } = useI18n()
const loading = ref(false)
const items = ref<any[]>([])
const total = ref(0)
const page = ref(1)
const tags = ref<Array<{ name: string; count: number }>>([])
const totalProducts = ref(0)
const totalPurchases = ref(0)
const totalTags = ref(0)
const recentListings = ref<any[]>([])
const recentPurchases = ref<any[]>([])
const showListing = ref(false)
const showPurchase = ref(false)
const selectedProduct = ref<any>(null)

const filters = reactive({
  query: '',
  seller: '',
  tag: '',
  inStockOnly: true,
})

let filterTimeout: ReturnType<typeof setTimeout> | null = null

function formatNQT(nqt: string) { return formatNrc(nqt) }

function parseTags(tagsStr: string): string[] {
  if (!tagsStr) return []
  return tagsStr.split(',').map((t) => t.trim()).filter(Boolean)
}

onMounted(() => {
  refreshData()
  loadTagCloud()
  loadStats()
  loadRecentListings()
  loadRecentPurchases()
})

function onFilterChange() {
  if (filterTimeout) clearTimeout(filterTimeout)
  filterTimeout = setTimeout(() => {
    page.value = 1
    refreshData()
  }, 300)
}

function selectTag(tagName: string) {
  if (filters.tag === tagName) {
    filters.tag = ''
  } else {
    filters.tag = tagName
  }
  onFilterChange()
}

async function refreshData() {
  loading.value = true
  try {
    const result = await nrcsApi.searchDGSGoods(
      filters.query || undefined,
      filters.tag || undefined,
      filters.seller || undefined,
      (page.value - 1) * 20,
      page.value * 20 - 1
    )
    const goods = (result as any).goods || []
    items.value = filters.inStockOnly
      ? goods.filter((g: any) => !g.delisted && g.quantity > 0)
      : goods
    total.value = items.value.length
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  } finally {
    loading.value = false
  }
}

async function loadTagCloud() {
  try {
    const tagRes = await nrcsApi.getDGSTags(filters.inStockOnly, 0, 50)
    const tagList = (tagRes as any).tags || []
    // Build tag cloud with simple counts
    const tagMap = new Map<string, number>()
    for (const tag of tagList) {
      const name = tag.trim()
      if (name) {
        tagMap.set(name, (tagMap.get(name) || 0) + 1)
      }
    }
    tags.value = Array.from(tagMap.entries())
      .map(([name, count]) => ({ name, count }))
      .sort((a, b) => b.count - a.count)
      .slice(0, 30)
  } catch { /* ignore */ }
}

async function loadStats() {
  try {
    const [goodsRes, tagCountRes] = await Promise.all([
      nrcsApi.getDGSGoods(undefined, 0, 0, true).catch(() => null),
      nrcsApi.getDGSTagCount(true).catch(() => null),
    ])
    totalProducts.value = (goodsRes as any)?.goods?.length || 0
    totalTags.value = (tagCountRes as any)?.numberOfTags || 0
  } catch { /* ignore */ }

  try {
    const countRes = await nrcsApi.getDGSPurchaseCount(undefined, undefined, undefined).catch(() => null)
    totalPurchases.value = (countRes as any)?.numberOfPurchases || 0
  } catch { /* ignore */ }
}

async function loadRecentListings() {
  try {
    const res = await nrcsApi.getDGSGoods(undefined, 0, 9, true)
    const goods = (res as any).goods || []
    recentListings.value = goods.filter((g: any) => !g.delisted).slice(0, 5)
  } catch { /* ignore */ }
}

async function loadRecentPurchases() {
  try {
    const res = await nrcsApi.getDGSPurchases(undefined, undefined, 0, 9, true)
    recentPurchases.value = ((res as any).purchases || []).slice(0, 5)
  } catch { /* ignore */ }
}

function openPurchase(row: any) {
  selectedProduct.value = row
  showPurchase.value = true
}
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;

.page-container {
  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: $space-md;
    .page-title {
      font-size: 18px;
      font-weight: 600;
      display: flex;
      align-items: center;
      gap: $space-sm;
      margin: 0;
    }
    .header-actions {
      display: flex;
      gap: $space-sm;
    }
  }
  .pagination-container {
    display: flex;
    justify-content: center;
    margin-top: $space-md;
  }
}

.search-card {
  margin-bottom: $space-md;
}

.tag-cloud-card {
  margin-bottom: $space-md;
  .tag-cloud {
    display: flex;
    flex-wrap: wrap;
    gap: $space-sm;
    .cloud-tag {
      cursor: pointer;
      user-select: none;
      transition: transform $duration-fast, opacity $duration-fast;
      &:hover {
        transform: scale(1.08);
        opacity: 0.9;
      }
      &.active {
        border-color: $primary;
      }
      .tag-count {
        font-size: 10px;
        margin-left: 2px;
      }
    }
  }
}

.product-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: $space-md;
}

.product-card {
  border: 1px solid $border-subtle;
  border-radius: $radius-md;
  padding: $space-md;
  cursor: pointer;
  transition: box-shadow $duration-fast, border-color $duration-fast;
  display: flex;
  flex-direction: column;
  gap: $space-sm;

  &:hover {
    box-shadow: 0 2px 12px rgba(0, 0, 0, 0.1);
    border-color: $primary;
  }

  .product-card-body {
    flex: 1;
    .product-name {
      font-size: $font-size-base;
      font-weight: 600;
      color: $text-primary;
      margin: 0 0 $space-xs;
    }
    .product-desc {
      font-size: $font-size-xs;
      color: $text-muted;
      margin: 0 0 $space-sm;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
    .product-tags {
      display: flex;
      flex-wrap: wrap;
      gap: 4px;
      .product-tag {
        margin: 0;
      }
    }
  }

  .product-card-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    .product-price {
      font-size: $font-size-lg;
      font-weight: 700;
      color: $primary;
    }
    .product-qty {
      font-size: $font-size-sm;
      color: $text-muted;
      strong { color: $text-primary; }
    }
  }

  .product-card-seller {
    display: flex;
    align-items: center;
    gap: $space-xs;
    font-size: $font-size-xs;
    color: $text-muted;
    .seller-text {
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
  }
}

.sidebar-card {
  margin-bottom: $space-md;
}

.card-header-title {
  font-size: $font-size-sm;
  font-weight: 600;
  color: $text-primary;
}

.card-header-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.stat-list {
  .stat-item {
    display: flex;
    align-items: center;
    gap: $space-sm;
    padding: $space-sm 0;
    border-bottom: 1px solid $border-subtle;

    &:last-child {
      border-bottom: none;
    }

    .stat-icon {
      width: 36px;
      height: 36px;
      border-radius: $radius-md;
      background: rgba(64, 158, 255, 0.1);
      display: flex;
      align-items: center;
      justify-content: center;
      color: $primary;
    }

    .stat-info {
      display: flex;
      flex-direction: column;
      .stat-num {
        font-size: $font-size-lg;
        font-weight: 700;
        color: $text-primary;
      }
      .stat-label {
        font-size: $font-size-xs;
        color: $text-muted;
      }
    }
  }
}

.sidebar-list {
  .sidebar-list-item {
    display: flex;
    flex-wrap: wrap;
    gap: $space-xs;
    padding: $space-sm 0;
    border-bottom: 1px solid $border-subtle;
    cursor: pointer;
    transition: background $duration-fast;

    &:last-child {
      border-bottom: none;
    }

    &:hover {
      background: $bg-hover;
      border-radius: $radius-sm;
    }

    .sidebar-item-name {
      flex: 1 1 100%;
      font-size: $font-size-sm;
      font-weight: 500;
      color: $text-primary;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }

    .sidebar-item-price {
      font-size: $font-size-sm;
      font-weight: 600;
      color: $primary;
    }

    .sidebar-item-seller {
      font-size: $font-size-xs;
      color: $text-muted;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
  }
}

.empty-products {
  padding: 40px 0;
  text-align: center;
}
</style>
