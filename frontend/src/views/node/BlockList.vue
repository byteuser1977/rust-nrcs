<template>
  <div class="block-list-page">
    <!-- 统计概览（对标 nrs.blocks.js:287-332 blocksPageLoaded 的统计计算） -->
    <el-row :gutter="12" class="stats-row">
      <el-col :span="6">
        <el-card shadow="never" class="stat-card">
          <div class="stat-label">{{ t('blocks.transactionsPerHour') }}</div>
          <div class="stat-value text-mono">{{ stats.transactionsPerHour }}</div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card shadow="never" class="stat-card">
          <div class="stat-label">{{ t('blocks.averageGenerationTime') }}</div>
          <div class="stat-value text-mono">{{ stats.averageGenerationTime }}s</div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card shadow="never" class="stat-card">
          <div class="stat-label">{{ t('blocks.averageFee') }}</div>
          <div class="stat-value text-mono">{{ stats.averageFee }} NRC</div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card shadow="never" class="stat-card">
          <div class="stat-label">{{ t('blocks.averageAmount') }}</div>
          <div class="stat-value text-mono">{{ stats.averageAmount }} NRC</div>
        </el-card>
      </el-col>
    </el-row>

    <el-card shadow="never">
      <template #header>
        <div class="card-header">
          <div class="header-left">
            <!-- 区块类型切换（对标 nrs.blocks.js:348 blocks_page_type 切换） -->
            <el-radio-group v-model="blocksPageType" size="small" @change="onTypeChange">
              <el-radio-button value="all_blocks">{{ t('blocks.allBlocks') }}</el-radio-button>
              <el-radio-button value="forged_blocks" :disabled="!accountRS">{{ t('blocks.forgedBlocks') }}</el-radio-button>
            </el-radio-group>
          </div>
          <div class="header-right">
            <el-input
              v-model="searchQuery"
              :placeholder="t('node.searchByHeightOrHash')"
              style="width: 280px;"
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
        @row-click="onRowClick"
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

        <el-table-column :label="t('node.height')" width="110" prop="height">
          <template #default="{ row }">
            <el-link
              type="primary"
              :underline="false"
              :style="{ fontWeight: row.numberOfTransactions > 0 ? 'bold' : 'normal' }"
              @click.stop="showBlockInfo(row)"
            >
              <span class="mono-text">{{ row.height?.toLocaleString() }}</span>
            </el-link>
          </template>
        </el-table-column>

        <el-table-column :label="t('node.timestamp')" width="180" prop="timestamp">
          <template #default="{ row }">
            {{ formatBlockTime(row.timestamp) }}
          </template>
        </el-table-column>

        <el-table-column :label="t('node.amount')" width="140" align="right">
          <template #default="{ row }">
            {{ formatNrc(row.totalAmountNQT) }} NRC
          </template>
        </el-table-column>

        <el-table-column :label="t('node.fee')" width="120" align="right">
          <template #default="{ row }">
            {{ formatNrc(row.totalFeeNQT) }} NRC
          </template>
        </el-table-column>

        <el-table-column :label="t('node.txCount')" width="80" prop="numberOfTransactions" align="center" />

        <el-table-column :label="t('node.generator')" width="180">
          <template #default="{ row }">
            <el-tooltip :content="row.generatorRS || row.generator" placement="top">
              <span class="mono-text">{{ truncateHash(row.generatorRS || row.generator, 8) }}</span>
            </el-tooltip>
          </template>
        </el-table-column>

        <el-table-column :label="t('node.payloadLength')" width="120" align="right">
          <template #default="{ row }">
            {{ formatVolume(row.payloadLength) }}
          </template>
        </el-table-column>

        <!-- 基础目标百分比（对标 nrs.blocks.js:283 baseTargetPercent） -->
        <el-table-column :label="t('node.baseTargetPercent')" width="120" align="right">
          <template #default="{ row }">
            <span class="mono-text">{{ baseTargetPercent(row) }}%</span>
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

    <!-- 区块详情弹窗（对标 nrs.blocks.js:276 show_block_modal_action） -->
    <BlockInfoModal v-model:visible="showBlockModal" :block-id="selectedBlockId" />
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { Search, Refresh, Loading } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import type { NrcsBlock, NrcsTransaction } from '@/api/modules/nrcs.api'
import { usePagination } from '@/composables/usePagination'
import { truncateHash, formatNrc, formatTimestamp } from '@/utils/format'
import { getTypeName } from '@/constants/transaction-types'
import { useNodeStore } from '@/stores/modules/node.store'
import { useAccountStore } from '@/stores/modules/account.store'
import BlockInfoModal from '@/components/modals/BlockInfoModal.vue'

const { t } = useI18n()
const router = useRouter()
const nodeStore = useNodeStore()
const accountStore = useAccountStore()

/** 区块列表数据 */
const blocks = ref<(NrcsBlock & { _transactions?: NrcsTransaction[] })[]>([])
/** 是否加载中 */
const loading = ref(false)
/** 搜索关键词 */
const searchQuery = ref('')
/** 当前展开的区块 ID */
const expandedBlock = ref<string>('')
/** 展开行加载中 */
const expandedLoading = ref(false)
/** 区块页面类型（对标 nrs.blocks.js:220 blocksPageType） */
const blocksPageType = ref<'all_blocks' | 'forged_blocks'>('all_blocks')

/** 区块详情弹窗可见性 */
const showBlockModal = ref(false)
/** 选中的区块 ID（传给 BlockInfoModal） */
const selectedBlockId = ref<string>('')

/** 当前账户 RS（锻造区块视图需要） */
const accountRS = computed(() => accountStore.accountRS || localStorage.getItem('nrcs_account_rs') || '')

/** 分页 */
const pagination = usePagination(20)

/** 统计数据（对标 nrs.blocks.js:287-332） */
const stats = reactive({
  transactionsPerHour: 0,
  averageGenerationTime: 0,
  averageFee: '0.00',
  averageAmount: '0.00',
})

/** node store 区块变化回调取消注册函数 */
let unregisterBlockChanged: (() => void) | null = null

/**
 * 拉取区块列表（对标 nrs.blocks.js:219 NRS.pages.blocks）。
 * 根据当前 blocksPageType 选择 getBlocks 或 getAccountBlocks。
 */
async function fetchBlocks(): Promise<void> {
  try {
    loading.value = true
    const firstIdx = pagination.firstIndex.value
    const lastIdx = pagination.lastIndex.value

    let resultBlocks: NrcsBlock[] = []
    if (blocksPageType.value === 'forged_blocks' && accountRS.value) {
      const res = await nrcsApi.getAccountBlocks(accountRS.value, firstIdx, lastIdx, false)
      resultBlocks = res.blocks || []
    } else {
      const res = await nrcsApi.getBlocks(firstIdx, lastIdx, false)
      resultBlocks = res.blocks || []
    }

    blocks.value = resultBlocks as (NrcsBlock & { _transactions?: NrcsTransaction[] })[]

    // 分页总数估算（对标 nrs.blocks.js:230-233 hasMorePages 逻辑）
    if (resultBlocks.length < pagination.pageSize.value) {
      pagination.setTotal(firstIdx + resultBlocks.length)
    } else {
      pagination.setTotal(pagination.total.value + pagination.pageSize.value)
    }

    // 计算统计数据（对标 nrs.blocks.js:264-332 blocksPageLoaded）
    computeStats(resultBlocks)
  } catch (err) {
    console.error('[BlockList] Failed to fetch blocks:', err)
  } finally {
    loading.value = false
  }
}

/**
 * 计算区块统计（对标 nrs.blocks.js:264-332 NRS.blocksPageLoaded）。
 * 包含每小时交易数、平均生成时间、平均手续费、平均金额。
 */
function computeStats(blocksList: NrcsBlock[]): void {
  if (blocksList.length === 0) {
    stats.transactionsPerHour = 0
    stats.averageGenerationTime = 0
    stats.averageFee = '0.00'
    stats.averageAmount = '0.00'
    return
  }

  let totalTransactions = 0
  let totalAmountNQT = BigInt(0)
  let totalFeeNQT = BigInt(0)

  for (const b of blocksList) {
    totalTransactions += b.numberOfTransactions || 0
    totalAmountNQT += BigInt(b.totalAmountNQT || '0')
    totalFeeNQT += BigInt(b.totalFeeNQT || '0')
  }

  // 时间跨度（对标 nrs.blocks.js:307-312）
  const startingTime = blocksList[blocksList.length - 1].timestamp
  const endingTime = blocksList[0].timestamp
  const time = endingTime - startingTime

  // 每小时交易数（对标 nrs.blocks.js:322-325）
  stats.transactionsPerHour = time === 0
    ? 0
    : Math.round((totalTransactions / time) * 3600)

  // 平均生成时间（对标 nrs.blocks.js:327）
  stats.averageGenerationTime = Math.round(time / blocksList.length)

  // 平均手续费（NQT → NRC，对标 nrs.blocks.js:317）
  const avgFeeNQT = totalFeeNQT / BigInt(blocksList.length)
  stats.averageFee = (Number(avgFeeNQT) / 1e8).toFixed(2)

  // 平均金额（NQT → NRC，对标 nrs.blocks.js:318）
  const avgAmountNQT = totalAmountNQT / BigInt(blocksList.length)
  stats.averageAmount = (Number(avgAmountNQT) / 1e8).toFixed(2)
}

/**
 * 搜索区块（按高度或哈希）。
 */
async function searchBlock(): Promise<void> {
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
      computeStats([block])
    } else {
      blocks.value = []
      pagination.setTotal(0)
      computeStats([])
    }
  } catch (err) {
    console.error('[BlockList] Block not found:', err)
    blocks.value = []
    pagination.setTotal(0)
    computeStats([])
  } finally {
    loading.value = false
  }
}

/**
 * 展开行：拉取区块交易详情。
 */
async function handleExpand(row: NrcsBlock & { _transactions?: NrcsTransaction[] }, expandedRows: any[]): Promise<void> {
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

/**
 * 区块类型切换（对标 nrs.blocks.js:348 blocks_page_type 切换）。
 */
function onTypeChange(): void {
  pagination.currentPage.value = 1
  fetchBlocks()
}

/**
 * 行点击：打开区块详情弹窗（对标 nrs.blocks.js:276 show_block_modal_action）。
 */
function onRowClick(row: NrcsBlock): void {
  showBlockInfo(row)
}

/**
 * 显示区块详情弹窗。
 */
function showBlockInfo(block: NrcsBlock): void {
  selectedBlockId.value = block.block
  showBlockModal.value = true
}

function onPageChange(): void {
  fetchBlocks()
}

function onSizeChange(): void {
  pagination.currentPage.value = 1
  fetchBlocks()
}

function goToTx(txId?: string): void {
  if (txId) {
    router.push(`/transaction/detail/${txId}`)
  }
}

/**
 * 格式化区块时间戳。
 */
function formatBlockTime(ts: number): string {
  return formatTimestamp(ts)
}

/**
 * 格式化数据体积（对标 nrs.blocks.js:282 NRS.formatVolume）。
 */
function formatVolume(bytes: number): string {
  if (!bytes) return '0 B'
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`
}

/**
 * 计算基础目标百分比（对标 nrs.blocks.js:283 NRS.baseTargetPercent）。
 * 基于创世块基础目标 153722867 做百分比换算。
 */
function baseTargetPercent(block: NrcsBlock): string {
  const GENESIS_BASE_TARGET = 153722867
  const bt = Number(block.baseTarget || '0')
  if (!bt) return '0'
  const percent = Math.round((bt / GENESIS_BASE_TARGET) * 100)
  return String(percent).padStart(4, ' ')
}

onMounted(() => {
  fetchBlocks()
  // 注册区块变化回调：新区块到达时自动刷新（对标 nrs.blocks.js:260 NRS.incoming.blocks）
  nodeStore.onBlockChanged(() => {
    // 仅在当前页为第 1 页时自动刷新，避免打断用户翻页
    if (pagination.currentPage.value === 1 && !searchQuery.value) {
      fetchBlocks()
    }
  })
})

onUnmounted(() => {
  // node store 回调目前无反向注销 API，依赖 clearCallbacks 在登出时统一清理
})
</script>

<style scoped lang="scss">
.block-list-page {
  .stats-row {
    margin-bottom: 12px;

    .stat-card {
      :deep(.el-card__body) {
        padding: 12px 16px;
      }

      .stat-label {
        font-size: 12px;
        color: $text-muted;
        margin-bottom: 4px;
      }

      .stat-value {
        font-size: 18px;
        font-weight: 600;
        color: $text-primary;
      }
    }
  }

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
