<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Collection /></el-icon> {{ t('alias.title') }}</h2>
      <div class="header-actions">
        <el-button type="primary" size="small" @click="showRegister = true"><el-icon><Plus /></el-icon> {{ t('alias.registerAlias') }}</el-button>
        <el-button size="small" @click="refreshData"><el-icon><Refresh /></el-icon> {{ t('common.refresh') }}</el-button>
      </div>
    </div>
    <el-card shadow="hover" class="mb-4">
      <el-form :inline="true">
        <el-form-item :label="t('alias.searchLabel')">
          <el-input v-model="searchQuery" :placeholder="t('alias.searchPlaceholder')" clearable style="width:300px" @change="searchAliases" />
        </el-form-item>
        <el-form-item>
          <el-button @click="searchAliases">{{ t('common.search') }}</el-button>
        </el-form-item>
      </el-form>
    </el-card>
    <el-card shadow="hover" v-loading="loading">
      <el-table :data="items" stripe style="width:100%" :empty-text="t('common.noData')">
        <el-table-column prop="aliasName" :label="t('alias.aliasName')" min-width="180" />
        <el-table-column prop="aliasURI" :label="t('alias.uri')" min-width="240" show-overflow-tooltip />
        <el-table-column prop="accountRS" :label="t('common.owner')" width="200" show-overflow-tooltip />
        <el-table-column :label="t('alias.price')" width="120" align="right">
          <template #default="{ row }">{{ row.priceNQT ? (Number(row.priceNQT) / 1e8).toFixed(2) + ' NRC' : '-' }}</template>
        </el-table-column>
        <el-table-column :label="t('common.actions')" width="180" fixed="right">
          <template #default="{ row }">
            <el-button v-if="!row.buyer || row.buyer === '0'" size="small" text type="primary" @click="openBuy(row)">{{ t('alias.buy') }}</el-button>
            <el-button v-if="isOwner(row)" size="small" text type="warning" @click="openSell(row)">{{ t('alias.sell') }}</el-button>
          </template>
        </el-table-column>
      </el-table>
      <div class="pagination-container" v-if="total > 20">
        <el-pagination v-model:current-page="page" :page-size="20" :total="total" layout="prev,pager,next" @current-change="refreshData" />
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
import { nrcsApi } from '@/api/modules/nrcs.api'
import SetAliasModal from '@/components/modals/SetAliasModal.vue'
import SellAliasModal from '@/components/modals/SellAliasModal.vue'
import BuyAliasModal from '@/components/modals/BuyAliasModal.vue'

const { t } = useI18n()
const loading = ref(false)
const items = ref<any[]>([])
const total = ref(0)
const page = ref(1)
const searchQuery = ref('')
const showRegister = ref(false)
const showSell = ref(false)
const showBuy = ref(false)
const selectedAlias = ref<any>(null)

onMounted(() => refreshData())

async function refreshData() {
  loading.value = true
  try {
    const accountId = localStorage.getItem('nrcs_account_id') || ''
    const result = await nrcsApi.getAliases(accountId, (page.value - 1) * 20, page.value * 20 - 1)
    items.value = result.aliases || []
    total.value = items.value.length
  } catch (e: any) { ElMessage.error(e?.message || t('common.loadError')) }
  finally { loading.value = false }
}

async function searchAliases() {
  if (!searchQuery.value) { refreshData(); return }
  loading.value = true
  try {
    const result = await nrcsApi.getAlias(searchQuery.value)
    items.value = result ? [result] : []
    total.value = items.value.length
  } catch (e: any) { ElMessage.error(e?.message || t('common.loadError')) }
  finally { loading.value = false }
}

function isOwner(row: any) {
  const accountId = localStorage.getItem('nrcs_account_id')
  return accountId && (row.account === accountId || row.accountRS === localStorage.getItem('nrcs_account_rs'))
}

function openBuy(row: any) { selectedAlias.value = row; showBuy.value = true }
function openSell(row: any) { selectedAlias.value = row; showSell.value = true }
</script>
<style scoped lang="scss">
@use '@/assets/styles/variables' as *;
.page-container { .page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; .page-title { font-size: 18px; font-weight: 600; display: flex; align-items: center; gap: 8px; margin: 0; } .header-actions { display: flex; gap: 8px; } } .pagination-container { display: flex; justify-content: center; margin-top: 16px; } }
.mb-4 { margin-bottom: $space-md; }
</style>
