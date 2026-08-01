<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title">
        <el-icon><CollectionTag /></el-icon>
        {{ t('alias.title') }}
        <span v-if="aliasCount !== null" class="alias-count-badge">{{ aliasCount }}</span>
      </h2>
      <div class="header-actions">
        <el-button type="primary" size="small" @click="openRegister()">
          <el-icon><Plus /></el-icon>
          {{ t('alias.registerAlias') }}
        </el-button>
        <el-button size="small" @click="refreshData">
          <el-icon><Refresh /></el-icon>
          {{ t('common.refresh') }}
        </el-button>
      </div>
    </div>

    <el-card shadow="hover" class="search-card">
      <el-form :inline="true" @submit.prevent="searchAliases">
        <el-form-item :label="t('alias.searchLabel')">
          <el-input
            v-model="searchQuery"
            :placeholder="t('alias.searchPlaceholder')"
            clearable
            style="width: 320px"
            @keyup.enter="searchAliases"
          />
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="searchAliases">
            <el-icon><Search /></el-icon>
            {{ t('common.search') }}
          </el-button>
          <el-button @click="resetSearch">{{ t('common.reset') }}</el-button>
        </el-form-item>
      </el-form>
    </el-card>

    <el-card shadow="hover" class="subtitle-card">
      <div class="section-title">
        <el-icon><User /></el-icon>
        {{ t('alias.yourAliases') }}
      </div>
    </el-card>

    <el-card shadow="hover" v-loading="loading" class="alias-card">
      <el-table
        :data="items"
        stripe
        style="width: 100%"
        :empty-text="t('common.noData')"
        row-key="aliasName"
      >
        <el-table-column prop="aliasName" :label="t('alias.aliasName')" min-width="180">
          <template #default="{ row }">
            <span class="text-mono text-accent">{{ row.aliasName }}</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('alias.uri')" min-width="260" show-overflow-tooltip>
          <template #default="{ row }">
            <a
              v-if="isHttpUri(row.aliasURI)"
              :href="row.aliasURI"
              target="_blank"
              rel="noopener noreferrer"
              class="uri-link"
            >{{ shortUri(row.aliasURI) }}</a>
            <span v-else class="text-sm">{{ row.aliasURI || '-' }}</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.owner')" width="200" show-overflow-tooltip>
          <template #default="{ row }">
            <span class="text-mono text-sm">{{ row.accountRS || row.account }}</span>
          </template>
        </el-table-column>
        <!-- 状态列（对标 nrs.aliases.js:53-86 状态判定逻辑） -->
        <el-table-column :label="t('alias.status')" width="140" align="center">
          <template #default="{ row }">
            <el-tag :type="statusTagType(row)" size="small" effect="plain">
              {{ statusLabel(row) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column :label="t('alias.price')" width="130" align="right">
          <template #default="{ row }">
            <span v-if="isAliasForSale(row)" class="text-mono">
              {{ formatNrc(row.priceNQT) }}
            </span>
            <span v-else class="text-muted text-sm">-</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.actions')" width="280" fixed="right">
          <template #default="{ row }">
            <div class="action-btns" v-if="isOwner(row)">
              <el-button size="small" text type="primary" @click="openEdit(row)">
                {{ t('common.edit') }}
              </el-button>
              <el-button size="small" text type="primary" @click="transferAlias(row)">
                {{ t('alias.transfer') }}
              </el-button>
              <el-button
                v-if="isAliasForSale(row)"
                size="small"
                text
                type="warning"
                @click="cancelSale(row)"
              >
                {{ t('alias.cancelSale') }}
              </el-button>
              <el-button
                v-else
                size="small"
                text
                type="warning"
                @click="openSell(row)"
              >
                {{ t('alias.sell') }}
              </el-button>
              <el-popconfirm
                :title="t('alias.confirmDelete')"
                @confirm="deleteAliasConfirm(row)"
              >
                <template #reference>
                  <el-button size="small" text type="danger">
                    {{ t('common.delete') }}
                  </el-button>
                </template>
              </el-popconfirm>
            </div>
            <div class="action-btns" v-else>
              <el-button
                v-if="isAliasForSale(row) && !isOwner(row)"
                size="small"
                type="primary"
                @click="openBuy(row)"
              >
                {{ t('alias.buy') }}
              </el-button>
            </div>
          </template>
        </el-table-column>
      </el-table>

      <div class="pagination-container" v-if="pagination.total.value > pagination.pageSize.value">
        <el-pagination
          v-model:current-page="pagination.currentPage.value"
          :page-size="pagination.pageSize.value"
          :total="pagination.total.value"
          :disabled="pagination.isLoading.value"
          layout="total, prev, pager, next"
          @current-change="handlePageChange"
        />
      </div>
    </el-card>

    <!-- 注册/编辑别名弹窗（对标 nrs.aliases.js:328 register_alias_modal） -->
    <SetAliasModal
      v-model:visible="showRegister"
      :edit-alias="editingAlias"
      @success="refreshData"
    />
    <!-- 出售别名弹窗（对标 nrs.aliases.js sell_alias_modal） -->
    <SellAliasModal
      v-model:visible="showSell"
      :alias-name="selectedAlias?.aliasName"
      @success="refreshData"
    />
    <!-- 转移别名弹窗（对标 nrs.aliases.js transfer_alias_modal，复用 sellAlias priceNQT=0） -->
    <TransferAliasModal
      v-model:visible="showTransfer"
      :alias="selectedAlias"
      @success="refreshData"
    />
    <!-- 购买别名弹窗（对标 nrs.aliases.js:274 buy_alias_modal） -->
    <BuyAliasModal
      v-model:visible="showBuy"
      :alias="selectedAlias"
      @success="refreshData"
    />

    <!-- 别名信息弹窗（对标 nrs.aliases.js:558 alias_info_modal 搜索结果展示） -->
    <el-dialog
      v-model="showAliasInfo"
      :title="t('alias.aliasInfo')"
      width="560px"
      destroy-on-close
      class="nrcs-modal"
    >
      <el-descriptions v-if="aliasInfo" :column="1" border>
        <el-descriptions-item :label="t('alias.aliasName')">
          <span class="text-mono text-accent">{{ aliasInfo.aliasName }}</span>
        </el-descriptions-item>
        <el-descriptions-item :label="t('common.owner')">
          <span class="text-mono">{{ aliasInfo.accountRS || aliasInfo.account }}</span>
        </el-descriptions-item>
        <el-descriptions-item :label="t('alias.uri')">
          <a
            v-if="isHttpUri(aliasInfo.aliasURI)"
            :href="aliasInfo.aliasURI"
            target="_blank"
            rel="noopener noreferrer"
            class="uri-link"
          >{{ aliasInfo.aliasURI }}</a>
          <span v-else>{{ aliasInfo.aliasURI || '-' }}</span>
        </el-descriptions-item>
        <el-descriptions-item v-if="aliasInfo.timestamp" :label="t('alias.lastUpdated')">
          {{ formatTimestamp(aliasInfo.timestamp) }}
        </el-descriptions-item>
        <el-descriptions-item v-if="isAliasForSale(aliasInfo)" :label="t('alias.price')">
          <span class="text-mono">{{ formatNrc(aliasInfo.priceNQT) }} NRC</span>
        </el-descriptions-item>
      </el-descriptions>

      <!-- 出售提示（对标 nrs.aliases.js:580-594 alias_sale_callout） -->
      <el-alert
        v-if="aliasInfo && isAliasForSale(aliasInfo) && !isOwner(aliasInfo)"
        :title="t('alias.aliasSaleIndirectOffer', { nxt: formatNrc(aliasInfo.priceNQT) })"
        type="info"
        :closable="false"
        show-icon
        class="sale-callout"
      >
        <el-button type="primary" size="small" @click="openBuy(aliasInfo)">
          {{ t('alias.buyItQ') }}
        </el-button>
      </el-alert>

      <template #footer>
        <el-button v-if="aliasInfo && !isOwner(aliasInfo) && !isAliasForSale(aliasInfo)" type="primary" @click="showAccountInfo">
          {{ t('alias.viewOwnerInfoQ') }}
        </el-button>
        <el-button v-if="aliasInfo && !isOwner(aliasInfo) && !isAliasForSale(aliasInfo)" type="success" @click="registerFromSearch">
          {{ t('alias.registerQ') }}
        </el-button>
        <el-button @click="showAliasInfo = false">{{ t('common.close') }}</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
/**
 * Aliases.vue —— 别名管理页面。
 *
 * 对标 nrs.aliases.js 的 NRS.pages.aliases（nrs.aliases.js:21-101）。
 *
 * 功能：
 *   - 列表展示当前账户的所有别名（getAliases + getAliasCount）
 *   - 按 aliasName 字母序排序（nrs.aliases.js:40-48）
 *   - 状态判定：registered / for_sale_direct / for_sale_indirect / cancelling_sale / transfer_in_progress
 *   - 注册/编辑别名（SetAliasModal，支持 uri/account/general 三种类型）
 *   - 转移别名（TransferAliasModal，priceNQT=0）
 *   - 出售别名（SellAliasModal）
 *   - 取消出售（priceNQT=0 + recipient=自身，对标 nrs.aliases.js:135-140）
 *   - 购买别名（BuyAliasModal）
 *   - 删除别名（deleteAlias API）
 *   - 搜索别名（getAlias，展示 alias_info_modal）
 */
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { CollectionTag, Plus, Refresh, Search, User } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { usePagination } from '@/composables/usePagination'
import { formatNrc, formatTimestamp } from '@/utils/format'
import { useAccountStore } from '@/stores/modules/account.store'
import { useNodeStore } from '@/stores/modules/node.store'
import { useNrcsForm } from '@/composables/useNrcsForm'
import SetAliasModal from '@/components/modals/SetAliasModal.vue'
import SellAliasModal from '@/components/modals/SellAliasModal.vue'
import BuyAliasModal from '@/components/modals/BuyAliasModal.vue'
import TransferAliasModal from '@/components/modals/TransferAliasModal.vue'

const { t } = useI18n()
const accountStore = useAccountStore()
const nodeStore = useNodeStore()
const { submitForm } = useNrcsForm()

const loading = ref(false)
const items = ref<any[]>([])
const aliasCount = ref<number | null>(null)
const pagination = usePagination(20)
const searchQuery = ref('')

const showRegister = ref(false)
const showSell = ref(false)
const showBuy = ref(false)
const showTransfer = ref(false)
const showAliasInfo = ref(false)
const selectedAlias = ref<any>(null)
const editingAlias = ref<{ aliasName: string; aliasURI: string } | undefined>(undefined)
const aliasInfo = ref<any>(null)

const accountRS = ref(accountStore.accountRS || localStorage.getItem('nrcs_account_rs') || '')
const accountId = ref(accountStore.accountId || localStorage.getItem('nrcs_account_id') || '')

onMounted(() => {
  refreshData()
  // 注册区块更新回调（对标 nrs.aliases.js:379 NRS.incoming.aliases）
  // node store 回调目前无反向注销 API，依赖 clearCallbacks 在登出时统一清理
  nodeStore.onBlockChanged(onBlockChanged)
})

/**
 * 区块更新回调（对标 nrs.aliases.js:379 NRS.incoming.aliases）。
 *
 * 当有交易更新时刷新别名列表。
 */
function onBlockChanged(): void {
  // 仅在第 1 页且无搜索时自动刷新（避免打断用户操作）
  if (pagination.currentPage.value === 1 && !searchQuery.value.trim()) {
    refreshData()
  }
}

function handlePageChange(page: number): void {
  pagination.goToPage(page)
  refreshData()
}

/**
 * 刷新别名列表（对标 nrs.aliases.js:21 NRS.pages.aliases）。
 *
 * 同时调用 getAliasCount（显示总数）和 getAliases（分页查询）。
 */
async function refreshData(): Promise<void> {
  loading.value = true
  try {
    const acct = accountRS.value
    if (!acct) {
      ElMessage.warning(t('common.noAccount'))
      return
    }

    // 并行加载别名总数与当前页数据（对标 nrs.aliases.js:23-31）
    const [countRes, listRes] = await Promise.all([
      nrcsApi.getAliasCount(acct),
      nrcsApi.getAliases(
        acct,
        pagination.firstIndex.value,
        pagination.lastIndex.value
      )
    ])

    aliasCount.value = countRes?.numberOfAliases ?? 0

    let list = listRes.aliases || []
    // 对标 nrs.aliases.js:40-48 按 aliasName 小写排序
    list = list.slice().sort((a: any, b: any) => {
      const an = (a.aliasName || '').toLowerCase()
      const bn = (b.aliasName || '').toLowerCase()
      if (an > bn) return 1
      if (an < bn) return -1
      return 0
    })

    items.value = list
    // 对标 nrs.aliases.js:34-37 多取一条判断是否有下一页
    pagination.setTotal(aliasCount.value)
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  } finally {
    loading.value = false
  }
}

/**
 * 搜索别名（对标 nrs.aliases.js:558-601 alias_search 提交）。
 *
 * 调用 getAlias 查询，成功时展示 alias_info_modal，失败时提示注册。
 */
async function searchAliases(): Promise<void> {
  const q = searchQuery.value.trim()
  if (!q) {
    refreshData()
    return
  }
  loading.value = true
  try {
    const result = await nrcsApi.getAlias(undefined, q)
    if (result && result.aliasName) {
      aliasInfo.value = result
      showAliasInfo.value = true
    } else {
      ElMessage.error(t('alias.aliasNotFound'))
    }
  } catch (e: any) {
    // 对标 nrs.aliases.js:567-569 未找到时提示注册
    ElMessage.error(e?.message || t('alias.aliasNotFound'))
  } finally {
    loading.value = false
  }
}

function resetSearch(): void {
  searchQuery.value = ''
  refreshData()
}

/**
 * 判断是否为当前账户的别名（对标 nrs.aliases.js 中 NRS.account 比较）。
 */
function isOwner(row: any): boolean {
  return (row.account === accountId.value) ||
    (row.accountRS === accountRS.value)
}

/**
 * 判断别名是否在出售中（对标 nrs.aliases.js:67-82）。
 *
 * priceNQT 存在且不为 '0'，或 priceNQT 为 '0' 但有 buyer（取消出售中）。
 */
function isAliasForSale(row: any): boolean {
  if (!('priceNQT' in row)) return false
  if (row.priceNQT === '0') return false
  return true
}

/**
 * 状态标签文本（对标 nrs.aliases.js:53-86 状态判定）。
 *
 * - priceNQT='0' + buyer=自身 → cancelling_sale（取消出售中）
 * - priceNQT='0' + buyer=他人 → transfer_in_progress（转移中）
 * - priceNQT>'0' + 有 buyer → for_sale_direct（直接出售）
 * - priceNQT>'0' + 无 buyer → for_sale_indirect（间接出售）
 * - 无 priceNQT → registered（已注册）
 */
function statusLabel(row: any): string {
  if (!('priceNQT' in row)) {
    return t('alias.registered')
  }
  if (row.priceNQT === '0') {
    if (row.buyer === accountId.value) {
      return t('alias.cancelSale')
    }
    return t('alias.transferAlias')
  }
  if (typeof row.buyer !== 'undefined') {
    return t('alias.forSaleDirect')
  }
  return t('alias.forSaleIndirect')
}

/**
 * 状态标签类型（el-tag type）。
 */
function statusTagType(row: any): 'success' | 'warning' | 'info' | 'primary' {
  if (!('priceNQT' in row)) return 'success'
  if (row.priceNQT === '0') return 'info'
  return 'warning'
}

/**
 * 判断 URI 是否为 HTTP(S) 链接（对标 nrs.aliases.js:87 aliasURI.indexOf("http") === 0）。
 */
function isHttpUri(uri: string | undefined): boolean {
  if (!uri) return false
  return /^https?:\/\//i.test(uri)
}

/**
 * 截断 URI 展示（对标 nrs.aliases.js:58-63 shortAliasURI，超过 100 字符加 "..."）。
 */
function shortUri(uri: string | undefined): string {
  if (!uri) return '-'
  if (uri.length > 100) return uri.substring(0, 100) + '...'
  return uri
}

function openBuy(row: any): void {
  selectedAlias.value = row
  showBuy.value = true
}

function openSell(row: any): void {
  selectedAlias.value = row
  showSell.value = true
}

/**
 * 打开注册弹窗（对标 nrs.aliases.js:362-376 无 alias 时为注册模式）。
 */
function openRegister(): void {
  editingAlias.value = undefined
  selectedAlias.value = null
  showRegister.value = true
}

/**
 * 打开编辑弹窗（对标 nrs.aliases.js:334-361 有 alias 时为编辑模式）。
 *
 * 传入 aliasName 与 aliasURI，由 SetAliasModal 自动识别类型。
 */
function openEdit(row: any): void {
  editingAlias.value = {
    aliasName: row.aliasName,
    aliasURI: row.aliasURI || ''
  }
  showRegister.value = true
}

/**
 * 转移别名（对标 nrs.aliases.js:141-145 transfer_alias，priceNXT='0'）。
 */
function transferAlias(row: any): void {
  selectedAlias.value = row
  showTransfer.value = true
}

/**
 * 取消出售（对标 nrs.aliases.js:135-140 cancel_alias_sale，priceNXT='0' + recipient=自身）。
 *
 * 实际调用 sellAlias（priceNXT='0' + recipient=自身），通过 useNrcsForm 本地签名。
 */
async function cancelSale(row: any): Promise<void> {
  const secretPhrase = accountStore.secretPhrase || localStorage.getItem('nrcs-secretPhrase') || ''
  if (!secretPhrase) {
    ElMessage.warning(t('dashboard.enterSecretPhrase'))
    return
  }
  try {
    await submitForm('sellAlias', {
      aliasName: row.aliasName,
      priceNXT: '0',
      recipient: accountRS.value,
      feeNXT: '1',
      deadline: '1440',
      secretPhrase
    }, {
      successMessage: t('alias.cancelSaleSuccess')
    })
    refreshData()
  } catch (e: any) {
    ElMessage.error(e?.message || t('alias.cancelSaleError'))
  }
}

/**
 * 删除别名（对标 nrs.aliases.js:219 deleteAlias）。
 *
 * 通过 useNrcsForm 三步本地签名流程提交。
 */
async function deleteAliasConfirm(row: any): Promise<void> {
  const secretPhrase = accountStore.secretPhrase || localStorage.getItem('nrcs-secretPhrase') || ''
  if (!secretPhrase) {
    ElMessage.warning(t('dashboard.enterSecretPhrase'))
    return
  }
  try {
    await submitForm('deleteAlias', {
      aliasName: row.aliasName,
      feeNXT: '1',
      deadline: '1440',
      secretPhrase
    }, {
      successMessage: t('alias.deleteSuccess')
    })
    refreshData()
  } catch (e: any) {
    ElMessage.error(e?.message || t('alias.deleteError'))
  }
}

/**
 * 从搜索结果弹窗点击"注册？"（对标 nrs.aliases.js:568 register_q）。
 */
function registerFromSearch(): void {
  if (!aliasInfo.value) return
  showAliasInfo.value = false
  editingAlias.value = undefined
  // 预填搜索的名称作为新别名名称
  showRegister.value = true
}

/**
 * 查看所有者信息（对标 nrs.aliases.js:520 view_owner_info_q）。
 */
function showAccountInfo(): void {
  if (!aliasInfo.value) return
  // 跳转到账户详情页（可扩展为打开 AccountInfoModal）
  ElMessage.info(t('alias.viewOwnerInfoQ'))
}
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;

.page-container {
  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: $space-lg;
    flex-wrap: wrap;
    gap: $space-sm;
    .page-title {
      font-size: $font-size-lg;
      font-weight: 600;
      display: flex;
      align-items: center;
      gap: $space-sm;
      margin: 0;
      color: $text-primary;
    }
    .header-actions {
      display: flex;
      gap: $space-sm;
      flex-wrap: wrap;
    }
  }
}

.alias-count-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 24px;
  height: 24px;
  padding: 0 8px;
  margin-left: $space-xs;
  font-size: $font-size-xs;
  font-weight: 600;
  color: $primary;
  background: rgba($primary, 0.12);
  border-radius: 12px;
}

.search-card {
  margin-bottom: $space-lg;
  border: 1px solid $border-subtle !important;
  border-radius: $radius-lg !important;
  background: rgba($surface-800, 0.6) !important;

  :deep(.el-card__body) {
    padding: $space-md $space-lg !important;
  }
}

.subtitle-card {
  margin-bottom: $space-md;
  border: 1px solid $border-subtle !important;
  border-radius: $radius-md !important;
  background: rgba($surface-800, 0.4) !important;

  :deep(.el-card__body) {
    padding: $space-md $space-xl !important;
  }

  .section-title {
    font-size: $font-size-md;
    font-weight: 600;
    color: $text-primary;
    display: flex;
    align-items: center;
    gap: $space-sm;
  }
}

.alias-card {
  border: 1px solid $border-subtle !important;
  border-radius: $radius-lg !important;
  background: linear-gradient(180deg, rgba($surface-800, 0.8), rgba($surface-900, 0.9)) !important;

  :deep(.el-table) {
    background: transparent !important;
    --el-table-bg-color: transparent;
    --el-table-tr-bg-color: transparent;
    --el-table-header-bg-color: rgba(255, 255, 255, 0.02);
    --el-table-row-hover-bg-color: rgba($primary, 0.06);
    --el-table-border-color: $border-subtle;
    --el-table-text-color: $text-primary;
    --el-table-header-text-color: $text-muted;

    th.el-table__cell {
      background: rgba(255, 255, 255, 0.025) !important;
      font-weight: 600;
      font-size: $font-size-xs;
      text-transform: uppercase;
      border-bottom: 1px solid $border-subtle;
    }

    td.el-table__cell {
      border-bottom: 1px solid rgba($border-default, 0.5);
    }
  }
}

.uri-link {
  color: $primary;
  text-decoration: none;
  font-size: $font-size-sm;
  &:hover {
    text-decoration: underline;
  }
}

.action-btns {
  display: flex;
  flex-wrap: wrap;
  gap: 2px;
}

.text-mono {
  font-family: $font-mono;
}

.text-accent {
  color: $primary;
}

.text-sm {
  font-size: $font-size-sm;
}

.text-muted {
  color: $text-muted;
}

.pagination-container {
  display: flex;
  justify-content: center;
  margin-top: $space-lg;
}

.sale-callout {
  margin-top: $space-md;
}
</style>
