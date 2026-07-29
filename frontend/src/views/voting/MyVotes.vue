<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Checked /></el-icon> {{ t('voting.myVotes') }}</h2>
      <div class="header-actions">
        <el-button size="small" @click="refreshData">
          <el-icon><Refresh /></el-icon> {{ t('common.refresh') }}
        </el-button>
      </div>
    </div>

    <el-card shadow="hover" v-loading="loading">
      <el-table :data="items" stripe style="width: 100%" :empty-text="t('common.noData')">
        <el-table-column prop="pollName" :label="t('voting.poll')" min-width="180" />
        <el-table-column :label="t('common.transaction')" width="200" show-overflow-tooltip>
          <template #default="{ row }">{{ truncateHash(row.transaction) }}</template>
        </el-table-column>
        <el-table-column :label="t('voting.voteValue', 'Vote')" width="250" show-overflow-tooltip>
          <template #default="{ row }">
            <div v-if="row.voteOptions && row.voteOptions.length > 0" class="vote-values">
              <template v-for="(v, idx) in row.voteOptions" :key="idx">
                <el-tag v-if="row.pollOptions && row.pollOptions[idx]" size="small" type="primary" class="vote-tag">
                  {{ row.pollOptions[idx] }}: {{ v }}
                </el-tag>
                <el-tag v-else size="small" type="primary" class="vote-tag">
                  {{ v }}
                </el-tag>
              </template>
            </div>
            <span v-else-if="row.votesArray">{{ row.votesArray.join(', ') }}</span>
            <span v-else>-</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.date')" width="160">
          <template #default="{ row }">{{ formatTimestamp(row.timestamp) }}</template>
        </el-table-column>
        <el-table-column :label="t('common.actions')" width="120" fixed="right">
          <template #default="{ row }">
            <el-button size="small" text type="primary" @click="viewPoll(row)">{{ t('voting.viewPoll', 'View Poll') }}</el-button>
          </template>
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
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { formatTimestamp, truncateHash } from '@/utils/format'
import { useAccountStore } from '@/stores/modules/account.store'

const { t } = useI18n()
const accountStore = useAccountStore()
const loading = ref(false)
const items = ref<any[]>([])
const total = ref(0)
const page = ref(1)

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
      3
    )
    const txs = (result as any).transactions || []

    // Parse vote data and enrich with poll names
    items.value = await Promise.all(
      txs.map(async (tx: any) => {
        const att = tx.attachment || {}
        const pollId = att.poll || att.pollId || ''
        let pollName = pollId ? truncateHash(pollId) : 'Unknown'
        let pollOptions: string[] = []

        // Try to get poll details for name
        if (pollId) {
          try {
            const pollRes = await nrcsApi.getPoll(pollId)
            pollName = (pollRes as any).name || pollName
            pollOptions = (pollRes as any).options || []
          } catch { /* ignore */ }
        }

        // Parse votes: attachment.vote is an array of numbers/mappings
        const votesArray: any[] = att.vote || att.votes || []

        return {
          pollId,
          pollName,
          pollOptions,
          transaction: tx.transaction,
          sender: tx.sender,
          senderRS: tx.senderRS,
          timestamp: tx.timestamp,
          height: tx.height,
          block: tx.block,
          votesArray,
          voteOptions: pollOptions.map((opt: string, i: number) => votesArray[i]),
        }
      })
    )
    total.value = items.value.length
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  } finally {
    loading.value = false
  }
}

function viewPoll(row: any) {
  if (row.pollId) {
    ElMessage.info(`${t('voting.poll')}: ${row.pollName} (${truncateHash(row.pollId)})`)
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

.vote-values {
  display: flex;
  flex-wrap: wrap;
  gap: $space-xs;
  .vote-tag {
    margin: 0;
  }
}
</style>
