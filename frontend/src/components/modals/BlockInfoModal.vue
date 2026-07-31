<template>
  <el-dialog
    v-model="visible"
    :title="t('blockInfo.title')"
    width="900px"
    :close-on-click-modal="false"
    destroy-on-close
    class="block-info-modal"
    @close="handleClose"
  >
    <!-- 加载中 -->
    <div v-if="loading" class="block-info-loading">
      <el-icon class="is-loading"><Loading /></el-icon>
      <span>{{ t('common.loading') }}</span>
    </div>

    <template v-else-if="block">
      <!-- 错误提示 -->
      <el-alert
        v-if="error"
        :title="error"
        type="error"
        :closable="false"
        show-icon
        class="block-info-error"
      />

      <!-- 区块 ID 标题 -->
      <div class="block-info-header">
        <span class="block-info-height-label">{{ t('sidebar.height') }}:</span>
        <span class="block-info-height-value">#{{ block.height.toLocaleString() }}</span>
      </div>

      <!-- 标签页 -->
      <el-tabs v-model="activeTab" class="block-info-tabs">
        <!-- 区块详情标签页 -->
        <el-tab-pane :label="t('blockInfo.details')" name="details">
          <InfoTable
            :rows="blockDetailsRows"
            :column="2"
            @navigate="onNavigate"
          />
        </el-tab-pane>

        <!-- 区块交易标签页 -->
        <el-tab-pane :label="`${t('blockInfo.transactions')} (${blockTransactions.length})`" name="transactions">
          <div v-if="blockTransactions.length > 0" class="block-info-transactions">
            <el-table :data="blockTransactions" stripe size="small" @row-click="onTransactionClick">
              <el-table-column prop="transactionIndex" :label="t('blockInfo.index')" width="60" />
              <el-table-column :label="t('blockInfo.timestamp')" width="120">
                <template #default="{ row }">
                  {{ formatTimestamp(row.timestamp) }}
                </template>
              </el-table-column>
              <el-table-column :label="t('blockInfo.amount')" width="120">
                <template #default="{ row }">
                  {{ formatAmount(row.amountNQT) }} NRC
                </template>
              </el-table-column>
              <el-table-column :label="t('blockInfo.fee')" width="100">
                <template #default="{ row }">
                  {{ formatAmount(row.feeNQT) }} NRC
                </template>
              </el-table-column>
              <el-table-column :label="t('blockInfo.sender')">
                <template #default="{ row }">
                  <TransactionLink
                    v-if="getSenderLink(row)"
                    :link="getSenderLink(row)!"
                    @navigate="onNavigate"
                  />
                </template>
              </el-table-column>
              <el-table-column :label="t('blockInfo.recipient')">
                <template #default="{ row }">
                  <TransactionLink
                    v-if="getRecipientLink(row)"
                    :link="getRecipientLink(row)!"
                    @navigate="onNavigate"
                  />
                </template>
              </el-table-column>
            </el-table>
          </div>
          <el-empty v-else :description="t('blockInfo.noTransactions')" />
        </el-tab-pane>

        <!-- 已执行的 Phased 交易标签页 -->
        <el-tab-pane
          v-if="executedPhasedTransactions.length > 0"
          :label="`${t('blockInfo.executedPhased')} (${executedPhasedTransactions.length})`"
          name="executedPhased"
        >
          <el-table :data="executedPhasedTransactions" stripe size="small" @row-click="onTransactionClick">
            <el-table-column :label="t('blockInfo.timestamp')" width="120">
              <template #default="{ row }">
                {{ formatTimestamp(row.timestamp) }}
              </template>
            </el-table-column>
            <el-table-column :label="t('blockInfo.height')" width="100">
              <template #default="{ row }">
                <TransactionLink
                  v-if="getBlockLinkForRow(row)"
                  :link="getBlockLinkForRow(row)!"
                  @navigate="onNavigate"
                />
              </template>
            </el-table-column>
            <el-table-column :label="t('blockInfo.status')">
              <template #default="{ row }">
                {{ getPhasedStatus(row) }}
              </template>
            </el-table-column>
          </el-table>
        </el-tab-pane>
      </el-tabs>
    </template>

    <!-- 空状态 -->
    <el-empty v-else :description="t('blockInfo.notFound')" />

    <template #footer>
      <div class="dialog-footer">
        <el-button @click="handleClose">{{ t('common.close') }}</el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * BlockInfoModal 组件 —— 区块详情弹窗。
 *
 * 对标 nrs.modals.block.js（127 行）的完整实现。
 *
 * 主要功能：
 *   1. 显示区块基本信息（高度/生成者/时间戳/前块/后块/总金额/总手续费/版本等）
 *   2. 显示区块包含的交易列表（点击可打开交易详情 modal）
 *   3. 显示已执行的 Phased 交易列表（executedPhasedTransactions，若存在）
 *
 * 支持两种入参：
 *   - block: NrcsBlock 对象（直接展示，不调 API）
 *   - height / blockId: 数字或字符串（调 getBlock API 拉取）
 *
 * 对标参考：showBlockModal(block) 函数
 */
import { ref, watch, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Loading } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import type { NrcsBlock, NrcsTransaction } from '@/api/modules/nrcs.api'
import { formatAmount, formatTimestamp } from '@/utils/format'
import { getAccountLink, getBlockLink, type LinkRef } from '@/utils/transaction-links'
import { createInfoTable, type InfoRow } from '@/utils/transaction-info-table'
import InfoTable from '@/components/base/InfoTable.vue'
import TransactionLink from '@/components/base/TransactionLink.vue'

const props = defineProps<{
  /** 区块对象（直接展示，不调 API） */
  block?: NrcsBlock | null
  /** 区块高度（与 blockId 二选一，调 API 拉取） */
  height?: number | string | null
  /** 区块 ID（与 height 二选一，调 API 拉取） */
  blockId?: string | null
}>()

const emit = defineEmits<{
  /** 链接导航事件（对标 NRS.modalStack） */
  navigate: [link: LinkRef]
  /** 关闭 modal */
  close: []
}>()

const { t } = useI18n()

const visible = defineModel<boolean>('visible', { default: false })

/** 加载状态 */
const loading = ref(false)
/** 错误信息 */
const error = ref<string | null>(null)
/** 当前展示的区块 */
const block = ref<NrcsBlock | null>(null)
/** 当前激活的标签页 */
const activeTab = ref<'details' | 'transactions' | 'executedPhased'>('details')

/** 区块基本信息表行（对标 blockDetails → createInfoTable） */
const blockDetailsRows = computed<InfoRow[]>(() => {
  if (!block.value) return []
  const b = block.value
  const details: Record<string, any> = { ...b }
  delete details.transactions
  delete details.executedPhasedTransactions

  // generator 转链接
  details.generator_formatted_html = getAccountLink(b, 'account')
  delete details.generator
  delete details.generatorRS

  // previousBlock 转链接
  if (b.previousBlock) {
    details.previous_block_formatted_html = getBlockLink(b.height - 1, b.previousBlock)
    delete details.previousBlock
  }

  // nextBlock 转链接
  if (b.nextBlock) {
    details.next_block_formatted_html = getBlockLink(b.height + 1, b.nextBlock)
    delete details.nextBlock
  }

  // 时间戳格式化
  if (b.timestamp) {
    details.blockGenerationTime = formatTimestamp(b.timestamp)
  }

  return createInfoTable(details, false)
})

/** 区块包含的交易列表（对标 block.transactions） */
const blockTransactions = computed<NrcsTransaction[]>(() => {
  return block.value?.transactions ?? []
})

/** 已执行的 Phased 交易列表（对标 block.executedPhasedTransactions） */
const executedPhasedTransactions = computed<NrcsTransaction[]>(() => {
  return (block.value as any)?.executedPhasedTransactions ?? []
})

// ----------------------------------------------------------------
// 数据加载
// ----------------------------------------------------------------

/**
 * 加载区块详情（对标 nrs.modals.block.js:34-46）。
 *
 * 调用 getBlock API（includeTransactions=true, includeExecutedPhased=true）。
 *
 * @param blockHeight 区块高度（与 blockId 二选一）
 * @param blockId 区块 ID
 */
async function loadBlock(blockHeight?: number | string | null, blockId?: string | null): Promise<void> {
  loading.value = true
  error.value = null
  try {
    const params: any = {
      includeTransactions: true,
      includeExecutedPhased: true,
    }
    if (blockId) {
      params.block = blockId
    } else if (blockHeight !== undefined && blockHeight !== null && blockHeight !== '') {
      params.height = blockHeight
    }
    const resp = await nrcsApi.getBlock(params.block, params.height, undefined, params.includeTransactions)
    if (!resp || (resp as any).errorCode) {
      throw new Error((resp as any)?.errorDescription || t('blockInfo.notFound'))
    }
    block.value = resp
    activeTab.value = 'transactions'
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
    block.value = null
  } finally {
    loading.value = false
  }
}

// ----------------------------------------------------------------
// 链接辅助
// ----------------------------------------------------------------

/**
 * 获取交易的发送方链接。
 */
function getSenderLink(tx: NrcsTransaction): LinkRef | null {
  const link = getAccountLink(tx, 'sender')
  return typeof link === 'string' ? null : link
}

/**
 * 获取交易的接收方链接。
 */
function getRecipientLink(tx: NrcsTransaction): LinkRef | null {
  const link = getAccountLink(tx, 'recipient')
  return typeof link === 'string' ? null : link
}

/**
 * 获取交易所在区块的链接。
 */
function getBlockLinkForRow(tx: NrcsTransaction): LinkRef | null {
  if (!tx.height) return null
  return getBlockLink(tx.height)
}

/**
 * 获取 Phased 交易的执行状态（对标 nrs.modals.block.js:111）。
 */
function getPhasedStatus(tx: NrcsTransaction): string {
  if (!block.value) return ''
  return tx.attachment?.phasingFinishHeight === block.value.height
    ? t('blockInfo.finished')
    : t('blockInfo.approved')
}

// ----------------------------------------------------------------
// 事件处理
// ----------------------------------------------------------------

/**
 * 处理链接导航事件（转发给父组件）。
 */
function onNavigate(link: LinkRef): void {
  emit('navigate', link)
}

/**
 * 点击交易行（对标 transactions table row click）。
 */
function onTransactionClick(row: NrcsTransaction): void {
  if (row.transaction) {
    emit('navigate', {
      type: 'transaction',
      value: row.transaction,
      text: row.transaction,
    })
  }
}

/**
 * 关闭 modal 并清空状态。
 */
function handleClose(): void {
  visible.value = false
  block.value = null
  error.value = null
  activeTab.value = 'details'
  emit('close')
}

// ----------------------------------------------------------------
// 监听 props 变化自动加载
// ----------------------------------------------------------------

watch(
  () => [props.block, props.height, props.blockId, visible.value] as const,
  async ([newBlock, newHeight, newBlockId, isVisible]) => {
    if (!isVisible) return
    if (newBlock) {
      // 直接展示传入的 block 对象
      block.value = newBlock
      activeTab.value = 'transactions'
    } else if (newHeight || newBlockId) {
      // 通过 height/blockId 拉取
      await loadBlock(newHeight, newBlockId)
    }
  },
  { immediate: true },
)
</script>

<style scoped lang="scss">
.block-info-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 40px 0;
  color: var(--el-text-color-secondary);
}
.block-info-error {
  margin-bottom: 12px;
}
.block-info-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--el-fill-color-light);
  border-radius: 4px;
  margin-bottom: 12px;
  font-size: 14px;
}
.block-info-height-label {
  color: var(--el-text-color-secondary);
}
.block-info-height-value {
  font-family: monospace;
  font-size: 14px;
  color: var(--el-color-primary);
  font-weight: 600;
}
.block-info-tabs {
  margin-top: 8px;
}
.block-info-transactions {
  margin-top: 8px;
}
.dialog-footer {
  display: flex;
  justify-content: flex-end;
}
</style>
