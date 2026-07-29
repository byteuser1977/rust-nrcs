<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title">
        <el-icon><Setting /></el-icon>
        {{ t('dashboard.accountProperties') }}
      </h2>
      <div class="header-actions">
        <el-button type="primary" size="small" @click="showSetProperty = true">
          <el-icon><Plus /></el-icon>
          {{ t('dashboard.setProperty') }}
        </el-button>
        <el-button size="small" @click="refreshData">
          <el-icon><Refresh /></el-icon>
          {{ t('common.refresh') }}
        </el-button>
      </div>
    </div>

    <el-card shadow="hover" class="property-card" v-loading="loading">
      <div class="property-tabs">
        <el-radio-group v-model="filterMode" @change="applyFilter">
          <el-radio-button value="incoming">{{ t('dashboard.incomingProperties') }}</el-radio-button>
          <el-radio-button value="outgoing">{{ t('dashboard.outgoingProperties') }}</el-radio-button>
        </el-radio-group>
      </div>

      <el-table
        :data="filteredProperties"
        stripe
        style="width: 100%"
        :empty-text="t('common.noData')"
        row-key="transaction"
      >
        <el-table-column :label="t('common.account')" width="220" show-overflow-tooltip>
          <template #default="{ row }">
            <span class="text-mono text-accent">
              {{ filterMode === 'outgoing' ? row.recipientRS : row.setterRS }}
            </span>
          </template>
        </el-table-column>
        <el-table-column prop="property" :label="t('common.property')" min-width="160">
          <template #default="{ row }">
            <el-tag size="small" type="info" effect="plain">{{ row.property }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.value')" min-width="220" show-overflow-tooltip>
          <template #default="{ row }">
            <span class="text-sm">{{ row.value || '-' }}</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.actions')" width="120" fixed="right" v-if="filterMode === 'outgoing'">
          <template #default="{ row }">
            <el-popconfirm
              :title="t('dashboard.confirmDeleteProperty')"
              :confirm-button-text="t('common.confirm')"
              :cancel-button-text="t('common.cancel')"
              @confirm="deleteProperty(row)"
            >
              <template #reference>
                <el-button size="small" text type="danger">
                  {{ t('common.delete') }}
                </el-button>
              </template>
            </el-popconfirm>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <SetAccountPropertyModal v-model:visible="showSetProperty" @success="refreshData" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { Setting, Plus, Refresh } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { useAccountStore } from '@/stores/modules/account.store'
import SetAccountPropertyModal from '@/components/modals/SetAccountPropertyModal.vue'

const { t } = useI18n()
const accountStore = useAccountStore()

const loading = ref(false)
const properties = ref<any[]>([])
const filterMode = ref<'incoming' | 'outgoing'>('incoming')
const showSetProperty = ref(false)

const accountRS = ref(
  accountStore.accountRS || localStorage.getItem('nrcs_account_rs') || ''
)

const filteredProperties = computed(() => {
  if (!properties.value.length) return []
  const myAccount = accountRS.value
  return properties.value.filter((p: any) => {
    if (filterMode.value === 'incoming') {
      return p.recipientRS === myAccount && p.setterRS !== myAccount
    }
    return p.setterRS === myAccount
  })
})

onMounted(() => {
  refreshData()
})

function applyFilter() {}

async function refreshData() {
  loading.value = true
  try {
    const acct = accountRS.value
    if (!acct) {
      ElMessage.warning(t('common.noAccount'))
      return
    }
    const result = await nrcsApi.getAccountProperties(acct)
    properties.value = (result as any).properties || []
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  } finally {
    loading.value = false
  }
}

async function deleteProperty(row: any) {
  const secretPhrase = accountStore.secretPhrase || localStorage.getItem('nrcs-secretPhrase') || ''
  if (!secretPhrase) {
    ElMessage.warning(t('dashboard.enterSecretPhrase'))
    return
  }
  try {
    await nrcsApi.setAccountProperty({
      secretPhrase,
      recipient: row.recipientRS || row.recipient,
      property: row.property,
      value: '',
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

.property-tabs {
  margin-bottom: $space-lg;
}

.property-card {
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

.text-mono {
  font-family: $font-mono;
  font-size: $font-size-sm;
}

.text-accent {
  color: $primary;
}

.text-sm {
  font-size: $font-size-sm;
  color: $text-secondary;
}
</style>
