<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Plus /></el-icon> {{ t('voting.createPoll') }}</h2>
      <div class="header-actions">
        <el-button type="primary" size="small" @click="showCreate = true">
          <el-icon><Plus /></el-icon> {{ t('voting.createPoll') }}
        </el-button>
        <el-button size="small" @click="refreshData">
          <el-icon><Refresh /></el-icon> {{ t('common.refresh') }}
        </el-button>
      </div>
    </div>

    <!-- Introduction -->
    <el-card shadow="hover" class="intro-card">
      <div class="intro-content">
        <h3 class="intro-title">{{ t('voting.introduction', 'How Polls Work') }}</h3>
        <p class="intro-text">
          {{ t('voting.introText',
            'Polls on the NRCS blockchain allow you to create decentralized voting for your community. '
            + 'You can set multiple voting models — by account, by NRC balance, by asset holdings, or by currency holdings. '
            + 'Each poll has a defined finish height (block number) after which voting closes. '
            + 'Votes are weighted according to the voting model selected when creating the poll.'
          ) }}
        </p>
        <el-divider />
        <div class="voting-models">
          <h4>{{ t('voting.votingModel') }}</h4>
          <el-row :gutter="16">
            <el-col :span="6">
              <div class="model-card">
                <el-icon size="24"><User /></el-icon>
                <span class="model-name">{{ t('voting.modelAccount', 'Account') }}</span>
                <span class="model-desc">{{ t('voting.modelAccountDesc', 'One account = One vote') }}</span>
              </div>
            </el-col>
            <el-col :span="6">
              <div class="model-card">
                <el-icon size="24"><Money /></el-icon>
                <span class="model-name">{{ t('voting.modelBalance', 'NRC Balance') }}</span>
                <span class="model-desc">{{ t('voting.modelBalanceDesc', 'Weighted by NRC holdings') }}</span>
              </div>
            </el-col>
            <el-col :span="6">
              <div class="model-card">
                <el-icon size="24"><TrendCharts /></el-icon>
                <span class="model-name">{{ t('voting.modelAsset', 'Asset') }}</span>
                <span class="model-desc">{{ t('voting.modelAssetDesc', 'Weighted by asset holdings') }}</span>
              </div>
            </el-col>
            <el-col :span="6">
              <div class="model-card">
                <el-icon size="24"><Coin /></el-icon>
                <span class="model-name">{{ t('voting.modelCurrency', 'Currency') }}</span>
                <span class="model-desc">{{ t('voting.modelCurrencyDesc', 'Weighted by currency holdings') }}</span>
              </div>
            </el-col>
          </el-row>
        </div>
        <el-divider />
        <h4>{{ t('voting.yourPolls', 'Your Polls') }}</h4>
      </div>
    </el-card>

    <el-card shadow="hover" v-loading="loading" style="margin-top: 16px">
      <el-table :data="items" stripe style="width: 100%" :empty-text="t('common.noData')">
        <el-table-column prop="name" :label="t('voting.name')" min-width="180" />
        <el-table-column prop="description" :label="t('voting.description')" min-width="220" show-overflow-tooltip />
        <el-table-column :label="t('voting.finishHeight')" width="130" align="right">
          <template #default="{ row }">{{ row.finishHeight || '-' }}</template>
        </el-table-column>
        <el-table-column :label="t('voting.status', 'Status')" width="120" align="center">
          <template #default="{ row }">
            <el-tag :type="row._finished ? 'info' : 'success'" size="small">
              {{ row._finished ? t('voting.finished', 'Finished') : t('voting.active', 'Active') }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.date')" width="160">
          <template #default="{ row }">{{ formatTimestamp(row.timestamp) }}</template>
        </el-table-column>
      </el-table>
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

    <CreatePollModal v-model:visible="showCreate" @success="refreshData" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { formatTimestamp } from '@/utils/format'
import { useAccountStore } from '@/stores/modules/account.store'
import CreatePollModal from '@/components/modals/CreatePollModal.vue'

const { t } = useI18n()
const accountStore = useAccountStore()
const loading = ref(false)
const items = ref<any[]>([])
const total = ref(0)
const page = ref(1)
const showCreate = ref(false)

onMounted(() => refreshData())

async function refreshData() {
  loading.value = true
  try {
    const accountId = accountStore.accountId || accountStore.accountRS
    if (!accountId) {
      items.value = []
      total.value = 0
      return
    }
    const result = await nrcsApi.getBlockchainTransactions(
      accountId,
      (page.value - 1) * 20,
      page.value * 20 - 1,
      1,
      2
    )
    const txs = (result as any).transactions || []
    let currentHeight = 0
    try {
      const status = await nrcsApi.getBlockchainStatus()
      currentHeight = (status as any).numberOfBlocks || (status as any).lastBlockHeight || 0
    } catch { /* ignore */ }

    items.value = txs.map((tx: any) => {
      const att = tx.attachment || {}
      return {
        name: att.name || tx.transaction?.substring(0, 16) || 'Unknown',
        description: att.description || '',
        finishHeight: att.finishHeight || att.finish_height || 0,
        transaction: tx.transaction,
        senderRS: tx.senderRS,
        timestamp: tx.timestamp,
        _finished: (att.finishHeight || 0) > 0 && (att.finishHeight || 0) <= currentHeight,
      }
    })
    total.value = items.value.length
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  } finally {
    loading.value = false
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

.intro-card {
  .intro-content {
    .intro-title {
      font-size: $font-size-lg;
      font-weight: 600;
      margin-bottom: $space-sm;
    }
    .intro-text {
      font-size: $font-size-sm;
      line-height: 1.6;
      color: $text-secondary;
      margin-bottom: $space-md;
    }
  }
}

.voting-models {
  h4 {
    margin-bottom: $space-md;
    font-size: $font-size-base;
    font-weight: 600;
  }
  .model-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    padding: $space-md;
    border: 1px solid $border-subtle;
    border-radius: $radius-md;
    gap: $space-sm;
    transition: border-color $duration-fast;

    &:hover {
      border-color: $primary;
    }

    .model-name {
      font-size: $font-size-sm;
      font-weight: 600;
      color: $text-primary;
    }
    .model-desc {
      font-size: $font-size-xs;
      color: $text-muted;
    }
  }
}
</style>
