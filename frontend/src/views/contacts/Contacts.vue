<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><UserFilled /></el-icon> {{ t('contacts.title') }}</h2>
      <div class="header-actions">
        <el-button size="small" @click="refreshData"><el-icon><Refresh /></el-icon> {{ t('common.refresh') }}</el-button>
      </div>
    </div>
    <el-card shadow="hover" class="mb-4">
      <el-input v-model="searchQuery" :placeholder="t('contacts.searchPlaceholder')" clearable style="max-width:400px" @change="filterContacts" />
    </el-card>
    <el-card shadow="hover" v-loading="loading">
      <el-table :data="filteredItems" stripe style="width:100%" :empty-text="t('contacts.noContacts')">
        <el-table-column prop="name" :label="t('common.name')" min-width="160">
          <template #default="{ row }">{{ row.accountName || row.accountRS }}</template>
        </el-table-column>
        <el-table-column prop="accountRS" :label="t('common.account')" width="220" show-overflow-tooltip />
        <el-table-column :label="t('common.balance')" width="140" align="right">
          <template #default="{ row }">{{ formatNQT(row.balanceNQT) }} NRC</template>
        </el-table-column>
        <el-table-column prop="description" :label="t('common.description')" min-width="180" show-overflow-tooltip />
        <el-table-column :label="t('common.actions')" width="160" fixed="right">
          <template #default="{ row }">
            <el-button size="small" text type="primary" @click="openSendMoney(row)">{{ t('common.send') }}</el-button>
            <el-button size="small" text type="danger" @click="removeContact(row)">{{ t('common.remove') }}</el-button>
          </template>
        </el-table-column>
      </el-table>
      <div class="pagination-container" v-if="total > 20">
        <el-pagination v-model:current-page="page" :page-size="20" :total="total" layout="prev,pager,next" @current-change="refreshData" />
      </div>
    </el-card>
    <SendMoneyModal v-model:visible="showSend" :recipient="selectedContact" @success="refreshData" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, ElMessageBox } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'
import SendMoneyModal from '@/components/modals/SendMoneyModal.vue'

const { t } = useI18n()
const loading = ref(false)
const items = ref<any[]>([])
const total = ref(0)
const page = ref(1)
const searchQuery = ref('')
const showSend = ref(false)
const selectedContact = ref<any>(null)

const filteredItems = computed(() => {
  if (!searchQuery.value) return items.value
  const q = searchQuery.value.toLowerCase()
  return items.value.filter((c: any) =>
    (c.accountRS || '').toLowerCase().includes(q) ||
    (c.accountName || '').toLowerCase().includes(q)
  )
})

function formatNQT(nqt: string) { return nqt ? (Number(nqt) / 1e8).toFixed(2) : '0.00' }

onMounted(() => refreshData())

async function refreshData() {
  loading.value = true
  try {
    const accountId = localStorage.getItem('nrcs_account_id') || ''
    const result = await nrcsApi.getAccount(accountId)
    if (result && result.accountRS) {
      items.value = [result]
      total.value = 1
    }
  } catch (e: any) { /* contacts may be stored locally */ }
  finally { loading.value = false }
}

function filterContacts() { /* handled by computed */ }
function openSendMoney(row: any) { selectedContact.value = row; showSend.value = true }
function removeContact(row: any) {
  ElMessageBox.confirm(t('contacts.removeConfirm'), t('common.warning'), { type: 'warning' }).then(() => {
    items.value = items.value.filter((c: any) => c.accountRS !== row.accountRS)
    ElMessage.success(t('contacts.removeSuccess'))
  }).catch(() => {})
}
</script>
<style scoped lang="scss">
@use '@/assets/styles/variables' as *;
.page-container { .page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; .page-title { font-size: 18px; font-weight: 600; display: flex; align-items: center; gap: 8px; margin: 0; } .header-actions { display: flex; gap: 8px; } } .pagination-container { display: flex; justify-content: center; margin-top: 16px; } }
.mb-4 { margin-bottom: $space-md; }
</style>
