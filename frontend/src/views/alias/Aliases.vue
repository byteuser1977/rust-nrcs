<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title">
        <el-icon><Bookmark /></el-icon>
        {{ t('alias.title') }}
      </h2>
      <div class="header-actions">
        <el-button type="primary" size="small" @click="showRegister = true">
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
      <el-form :inline="true">
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
            <span class="text-sm">{{ row.aliasURI || '-' }}</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.owner')" width="200" show-overflow-tooltip>
          <template #default="{ row }">
            <span class="text-mono text-sm">{{ row.accountRS || row.account }}</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('alias.status')" width="130" align="center">
          <template #default="{ row }">
            <el-tag v-if="isAliasForSale(row)" type="warning" size="small" effect="plain">
              {{ t('alias.forSale') }}
            </el-tag>
            <el-tag v-else-if="row.buyer && row.buyer !== '0'" type="info" size="small" effect="plain">
              {{ t('alias.sold') }}
            </el-tag>
            <el-tag v-else type="success" size="small" effect="plain">
              {{ t('alias.registered') }}
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
        <el-table-column :label="t('common.actions')" width="240" fixed="right">
          <template #default="{ row }">
            <div class="action-btns" v-if="isOwner(row)">
              <el-button size="small" text type="primary" @click="editAlias(row)">
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

    <SetAliasModal v-model:visible="showRegister" @success="refreshData" />
    <SellAliasModal v-model:visible="showSell" :alias-name="selectedAlias?.aliasName" @success="refreshData" />
    <BuyAliasModal v-model:visible="showBuy" :alias="selectedAlias" @success="refreshData" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { Bookmark, Plus, Refresh, Search, User } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { usePagination } from '@/composables/usePagination'
import { formatNrc } from '@/utils/format'
import { useAccountStore } from '@/stores/modules/account.store'
import SetAliasModal from '@/components/modals/SetAliasModal.vue'
import SellAliasModal from '@/components/modals/SellAliasModal.vue'
import BuyAliasModal from '@/components/modals/BuyAliasModal.vue'

const { t } = useI18n()
const accountStore = useAccountStore()

const loading = ref(false)
const items = ref<any[]>([])
const pagination = usePagination(20)
const searchQuery = ref('')
const showRegister = ref(false)
const showSell = ref(false)
const showBuy = ref(false)
const selectedAlias = ref<any>(null)

const accountRS = ref(accountStore.accountRS || localStorage.getItem('nrcs_account_rs') || '')
const accountId = ref(accountStore.accountId || localStorage.getItem('nrcs_account_id') || '')

onMounted(() => {
  refreshData()
})

function handlePageChange(page: number) {
  pagination.goToPage(page)
  refreshData()
}

async function refreshData() {
  loading.value = true
  try {
    const acct = accountRS.value
    if (!acct) {
      ElMessage.warning(t('common.noAccount'))
      return
    }
    const result = await nrcsApi.getAliases(
      acct,
      pagination.firstIndex.value,
      pagination.lastIndex.value
    )
    const list = result.aliases || []
    items.value = list
    pagination.setTotalFromList(list.length)
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  } finally {
    loading.value = false
  }
}

async function searchAliases() {
  if (!searchQuery.value.trim()) {
    refreshData()
    return
  }
  loading.value = true
  try {
    const result = await nrcsApi.getAlias(undefined, searchQuery.value.trim())
    items.value = result ? [result] : []
    pagination.setTotal(items.value.length)
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  } finally {
    loading.value = false
  }
}

function resetSearch() {
  searchQuery.value = ''
  refreshData()
}

function isOwner(row: any): boolean {
  return (row.account === accountId.value) ||
    (row.accountRS === accountRS.value)
}

function isAliasForSale(row: any): boolean {
  return row.priceNQT && row.priceNQT !== '0' && (!row.buyer || row.buyer === '0')
}

function openBuy(row: any) {
  selectedAlias.value = row
  showBuy.value = true
}

function openSell(row: any) {
  selectedAlias.value = row
  showSell.value = true
}

function editAlias(row: any) {
  showRegister.value = true
}

function transferAlias(row: any) {
  openSell(row)
}

async function cancelSale(row: any) {
  const secretPhrase = accountStore.secretPhrase || localStorage.getItem('nrcs-secretPhrase') || ''
  if (!secretPhrase) {
    ElMessage.warning(t('dashboard.enterSecretPhrase'))
    return
  }
  try {
    await nrcsApi.sellAlias({
      secretPhrase,
      aliasName: row.aliasName,
      priceNQT: '0',
      feeNQT: '100000000',
      deadline: 1440
    })
    ElMessage.success(t('common.operationSuccess'))
    refreshData()
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  }
}

async function deleteAliasConfirm(row: any) {
  const secretPhrase = accountStore.secretPhrase || localStorage.getItem('nrcs-secretPhrase') || ''
  if (!secretPhrase) {
    ElMessage.warning(t('dashboard.enterSecretPhrase'))
    return
  }
  try {
    await nrcsApi.deleteAlias({
      secretPhrase,
      aliasName: row.aliasName,
      feeNQT: '100000000',
      deadline: 1440
    })
    ElMessage.success(t('common.operationSuccess'))
    refreshData()
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  }
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
    }
  }
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
</style>
