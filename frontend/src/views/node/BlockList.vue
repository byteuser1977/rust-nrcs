<template>
  <div class="block-list-page">
    <el-card shadow="never">
      <template #header>
        <div class="card-header">
          <div class="header-left">
            <span>{{ t('node.blockExplorer') }}</span>
          </div>
          <div class="header-right">
            <el-input
              v-model="searchQuery"
              :placeholder="t('node.searchByHeightOrHash')"
              style="width: 320px;"
              size="small"
              clearable
              @keyup.enter="searchBlock"
            >
              <template #append>
                <el-button @click="searchBlock">
                  <el-icon><Search /></el-icon>
                </el-button>
              </template>
            </el-input>
            <el-button
              size="small"
              text
              type="primary"
              style="margin-left: 8px;"
              @click="fetchBlocks"
            >
              <el-icon><Refresh /></el-icon>
              {{ t('common.refresh') }}
            </el-button>
          </div>
        </div>
      </template>

      <!-- Block Table -->
      <el-table
        :data="blocks"
        style="width: 100%"
        v-loading="loading"
        @expand-change="handleExpand"
        row-key="block"
      >
        <el-table-column type="expand">
          <template #default="{ row }">
            <div class="block-expand-detail">
              <el-descriptions :column="2" border size="small" style="margin-bottom: 12px;">
                <el-descriptions-item :label="t('node.previousBlock')">
                  <span class="mono-text">{{ truncateHash(row.previousBlock, 8) }}</span>
                </el-descriptions-item>
                <el-descriptions-item :label="t('node.nextBlock')">
                  <span class="mono-text">{{ truncateHash(row.nextBlock, 8) }}</span>
                </el-descriptions-item>
                <el-descriptions-item :label="t('node.baseTarget')">
                  {{ row.baseTarget || '-' }}
                </el-descriptions-item>
                <el-descriptions-item :label="t('node.payloadLength')">
                  {{ row.payloadLength?.toLocaleString() || '0' }}
                </el-descriptions-item>
                <el-descriptions-item :label="t('node.generationSignature')" :span="2">
                  <span class="mono-text mini">{{ truncateHash(row.generationSignature, 12) }}</span>
                </el-descriptions-item>
              </el-descriptions>

              <h4>{{ t('node.blockTransactions') }} ({{ row.numberOfTransactions || 0 }})</h4>
              <el-table
                v-if="expandedBlock === row.block && row._transactions"
                :data="row._transactions"
                style="width: 100%"
                size="small"
              >
                <el-table-column :label="t('transaction.id')" width="160">
                  <template #default="{ row: tx }">
                    <el-link type="primary" :underline="false" @click="goToTx(tx.transaction)">
                      <span class="mono-text">{{ truncateHash(tx.transaction, 8) }}</span>
                    </el-link>
                  </template>
                </el-table-column>
                <el-table-column :label="t('transaction.type')" width="120">
                  <template #default="{ row: tx }">
                    <el-tag size="small" type="info">{{ getTypeName(tx.type) }}</el-tag>
                  </template>
                </el-table-column>
                <el-table-column :label="t('transaction.sender')" width="160">
                  <template #default="{ row: tx }">
                    <span class="mono-text">{{ truncateHash(tx.senderRS || tx.sender, 8) }}</span>
                  </template>
                </el-table-column>
                <el-table-column :label="t('transaction.recipient')" width="160">
                  <template #default="{ row: tx }">
                    <span class="mono-text">{{ tx.recipientRS ? truncateHash(tx.recipientRS, 8) : '-' }}</span>
                  </template>
                </el-table-column>
                <el-table-column :label="t('transaction.amount')" width="140" align="right">
                  <template #default="{ row: tx }">
                    {{ formatNrc(tx.amountNQT) }} NRC
                  </template>
                </el-table-column>
                <el-table-column :label="t('transaction.fee')" width="100" align="right">
                  <template #default="{ row: tx }">
                    {{ formatNrc(tx.feeNQT) }}
                  </template>
                </el-table-column>
              </el-table>
              <div v-else-if="expandedBlock === row.block && expandedLoading" class="loading-expand">
                <el-icon class="is-loading"><Loading /></el-icon>
                {{ t('common.loading') }}
              </div>
              <span v-else-if="row.numberOfTransactions === 0" class="no-tx">{{ t('common.noData') }}</span>
            </div>
          </template>
        </el-table-column>

        <el-table-column :label="t('node.height')" width="100" prop="height">
          <template #default="{ row }">
            <span class="mono-text">{{ row.height?.toLocaleString() }}</span>
          </template>
        </el-table-column>

        <el-table-column :label="t('node.timestamp')" width="180" prop="timestamp">
          <template #default="{ row }">
            {{ formatBlockTime(row.timestamp) }}
          </template>
        </el-table-column>

        <el-table-column :label="t('node.generator')" width="180">
          <template #default="{ row }">
            <el-tooltip :content="row.generatorRS || row.generator" placement="top">
              <span class="mono-text">{{ truncateHash(row.generatorRS || row.generator, 8) }}</span>
            </el-tooltip>
          </template>
        </el-table-column>

        <el-table-column :label="t('node.txCount')" width="80" prop="numberOfTransactions" align="center" />

        <el-table-column :label="t('node.totalAmount')" width="160" align="right">
          <template #default="{ row }">
            {{ formatNrc(row.totalAmountNQT) }} NRC
          </template>
        </el-table-column>

        <el-table-column :label="t('node.totalFee')" width="120" align="right">
          <template #default="{ row }">
            {{ formatNrc(row.totalFeeNQT) }} NRC
          </template>
        </el-table-column>
      </el-table>

      <!-- Pagination -->
      <div class="pagination-wrapper">
        <el-pagination
          v-model:current-page="pagination.currentPage.value"
          v-model:page-size="pagination.pageSize.value"
          :page-sizes="[10, 20, 50, 100]"
          :total="pagination.total.value"
          layout="total, sizes, prev, pager, next, jumper"
          @current-change="onPageChange"
          @size-change="onSizeChange"
        />
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { Search, Refresh, Loading } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import type { NrcsBlock, NrcsTransaction } from '@/api/modules/nrcs.api'
import { usePagination } from '@/composables/usePagination'
import { truncateHash, formatNrc, formatTimestamp } from '@/utils/format'
import { getTypeName } from '@/constants/transaction-types'

const { t } = useI18n()
const router = useRouter()

const blocks = ref<(NrcsBlock & { _transactions?: NrcsTransaction[] })[]>([])
const loading = ref(false)
const searchQuery = ref('')
const expandedBlock = ref<string>('')
const expandedLoading = ref(false)

const pagination = usePagination(20)

async function fetchBlocks() {
  try {
    loading.value = true
    const firstIdx = pagination.firstIndex.value
    const lastIdx = pagination.lastIndex.value
    const res = await nrcsApi.getBlocks(firstIdx, lastIdx, false)
    blocks.value = res.blocks || []
    if (res.blocks && res.blocks.length < pagination.pageSize.value) {
      pagination.setTotal(firstIdx + (res.blocks?.length || 0))
    } else {
      pagination.setTotal(pagination.total.value + pagination.pageSize.value)
    }
  } catch (err) {
    console.error('[BlockList] Failed to fetch blocks:', err)
  } finally {
    loading.value = false
  }
}

async function searchBlock() {
  const q = searchQuery.value.trim()
  if (!q) {
    fetchBlocks()
    return
  }

  try {
    loading.value = true
    const isNumber = /^\d+$/.test(q)
    const block = await nrcsApi.getBlock(
      isNumber ? undefined : q,
      isNumber ? parseInt(q) : undefined,
      undefined,
      false
    )
    if (block) {
      blocks.value = [block as NrcsBlock]
      pagination.setTotal(1)
    } else {
      blocks.value = []
      pagination.setTotal(0)
    }
  } catch (err) {
    console.error('[BlockList] Block not found:', err)
    blocks.value = []
    pagination.setTotal(0)
  } finally {
    loading.value = false
  }
}

async function handleExpand(row: NrcsBlock & { _transactions?: NrcsTransaction[] }, expandedRows: any[]) {
  const isExpanded = expandedRows.some((r: any) => r.block === row.block)
  if (!isExpanded) {
    expandedBlock.value = ''
    return
  }

  if (row._transactions) {
    expandedBlock.value = row.block
    return
  }

  try {
    expandedLoading.value = true
    expandedBlock.value = row.block
    const blockDetail = await nrcsApi.getBlock(row.block, undefined, undefined, true)
    row._transactions = blockDetail.transactions || []
  } catch (err) {
    console.error('[BlockList] Failed to fetch transactions:', err)
    row._transactions = []
  } finally {
    expandedLoading.value = false
  }
}

function onPageChange() {
  fetchBlocks()
}

function onSizeChange() {
  pagination.currentPage.value = 1
  fetchBlocks()
}

function goToTx(txId?: string) {
  if (txId) {
    router.push(`/transaction/detail/${txId}`)
  }
}

function formatBlockTime(ts: number): string {
  return formatTimestamp(ts)
}

fetchBlocks()
</script>

<style scoped lang="scss">
.block-list-page {
  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;

    .header-left {
      font-size: 16px;
      font-weight: 600;
    }

    .header-right {
      display: flex;
      align-items: center;
    }
  }

  .mono-text {
    font-family: 'Roboto Mono', monospace;
    font-size: 13px;

    &.mini {
      font-size: 11px;
    }
  }

  .block-expand-detail {
    padding: 16px 24px;
    background: #fafafa;

    h4 {
      margin: 0 0 12px 0;
      font-size: 14px;
      color: #303133;
    }

    .loading-expand {
      display: flex;
      align-items: center;
      gap: 8px;
      color: #909399;
      font-size: 13px;
    }

    .no-tx {
      color: #909399;
      font-size: 13px;
    }
  }

  .pagination-wrapper {
    margin-top: 24px;
    display: flex;
    justify-content: flex-end;
  }
}
</style>
