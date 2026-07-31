<template>
  <el-dialog
    v-model="visible"
    :title="t('ledgerInfo.title')"
    width="640px"
    :close-on-click-modal="false"
    destroy-on-close
    class="ledger-info-modal"
    @close="handleClose"
  >
    <!-- 加载中 -->
    <div v-if="loading" class="ledger-info-loading">
      <el-icon class="is-loading"><Loading /></el-icon>
      <span>{{ t('common.loading') }}</span>
    </div>

    <template v-else-if="entry">
      <!-- 错误提示 -->
      <el-alert
        v-if="error"
        :title="error"
        type="error"
        :closable="false"
        show-icon
        class="ledger-info-error"
      />

      <!-- 条目 ID 标题 -->
      <div class="ledger-info-header">
        <span class="ledger-info-label">{{ t('ledgerInfo.entryId') }}:</span>
        <span class="ledger-info-value">{{ entry.ledgerId }}</span>
      </div>

      <!-- 条目详情信息表（对标 nrs.modals.ledger.js:48-82 showLedgerEntryModal） -->
      <InfoTable
        :rows="entryDetailsRows"
        :column="2"
        @navigate="onNavigate"
      />
    </template>

    <!-- 空状态 -->
    <el-empty v-else :description="t('ledgerInfo.notFound')" />

    <template #footer>
      <div class="dialog-footer">
        <el-button @click="handleClose">{{ t('common.close') }}</el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * LedgerEntryInfoModal 组件 —— 账户总账条目详情弹窗。
 *
 * 对标 nrs.modals.ledger.js（88 行）的完整实现。
 *
 * 主要功能：
 *   1. 通过 getAccountLedgerEntry API 拉取单条总账条目详情
 *   2. 格式化展示 eventType / holdingType / timestamp / holding / height / transaction
 *   3. 展示 change（变动金额）和 balance（变动后余额）
 *
 * 对标参考：showLedgerEntryModal(entry, change, balance) 函数
 */
import { ref, watch, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Loading } from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { formatTimestamp } from '@/utils/format'
import { getTransactionLink, getBlockLink, type LinkRef } from '@/utils/transaction-links'
import { createInfoTable, type InfoRow } from '@/utils/transaction-info-table'
import InfoTable from '@/components/base/InfoTable.vue'

const props = defineProps<{
  /** 总账条目 ID（调 API 拉取） */
  ledgerId?: string | null
  /** 预填条目对象（可选，避免重复 API 调用） */
  entry?: any | null
  /** 变动金额（展示用，对标 nrs.modals.ledger.js:34 change） */
  change?: string | null
  /** 变动后余额（展示用，对标 nrs.modals.ledger.js:34 balance） */
  balance?: string | null
}>()

const emit = defineEmits<{
  navigate: [link: LinkRef]
  close: []
}>()

const { t } = useI18n()

const visible = defineModel<boolean>('visible', { default: false })

const loading = ref(false)
const error = ref<string | null>(null)
const entry = ref<any | null>(null)

// ----------------------------------------------------------------
// 计算属性
// ----------------------------------------------------------------

/**
 * 条目详情信息表行（对标 nrs.modals.ledger.js:48-82）。
 *
 * 处理：
 *   - eventType → i18n 小写
 *   - holdingType → i18n 小写
 *   - timestamp → formatTimestamp（entryTime）
 *   - holding → getTransactionLink
 *   - height → getBlockLink
 *   - isTransactionEvent + event → getTransactionLink
 *   - change / balance → 直接展示（_formatted_html 后缀）
 */
const entryDetailsRows = computed<InfoRow[]>(() => {
  if (!entry.value) return []
  const e: Record<string, any> = { ...entry.value }

  // eventType / holdingType 小写后 i18n
  if (e.eventType) {
    e.eventType = t(`ledgerInfo.eventType.${String(e.eventType).toLowerCase()}`, String(e.eventType))
  }
  if (e.holdingType) {
    e.holdingType = t(`ledgerInfo.holdingType.${String(e.holdingType).toLowerCase()}`, String(e.holdingType))
  }

  // 时间戳格式化
  if (e.timestamp) {
    e.entryTime = formatTimestamp(e.timestamp)
  }

  // holding 转链接（对标 :63-66）
  if (e.holding) {
    e.holding_formatted_html = getTransactionLink(e.holding)
    delete e.holding
  }

  // height 转区块链接（对标 :67-69）
  if (e.height !== undefined) {
    e.height_formatted_html = getBlockLink(e.height)
    delete e.block
    delete e.height
  }

  // 交易事件（对标 :70-74）
  if (e.isTransactionEvent) {
    e.transaction_formatted_html = getTransactionLink(e.event)
  }
  delete e.event
  delete e.isTransactionEvent

  // change / balance 直接展示（_formatted_html 后缀）
  // 优先使用 props 传入的值（对标 nrs.modals.ledger.js:75-78）
  if (props.change) {
    e.change_formatted_html = props.change
    delete e.change
  }
  if (props.balance) {
    e.balance_formatted_html = props.balance
    delete e.balance
  }

  return createInfoTable(e, false)
})

// ----------------------------------------------------------------
// 数据加载
// ----------------------------------------------------------------

/**
 * 加载总账条目详情（对标 nrs.modals.ledger.js:43）。
 *
 * @param id 总账条目 ID
 */
async function loadEntry(id: string): Promise<void> {
  loading.value = true
  error.value = null
  try {
    const resp = await nrcsApi.getAccountLedgerEntry(id)
    if (!resp || (resp as any).errorCode) {
      throw new Error((resp as any)?.errorDescription || t('ledgerInfo.notFound'))
    }
    entry.value = resp
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
    entry.value = null
  } finally {
    loading.value = false
  }
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
 * 关闭 modal 并清空状态。
 */
function handleClose(): void {
  visible.value = false
  entry.value = null
  error.value = null
  emit('close')
}

// ----------------------------------------------------------------
// 监听 props 变化自动加载
// ----------------------------------------------------------------

watch(
  () => [props.entry, props.ledgerId, visible.value] as const,
  async ([newEntry, newLedgerId, isVisible]) => {
    if (!isVisible) return
    if (newEntry) {
      entry.value = newEntry
    } else if (newLedgerId) {
      await loadEntry(newLedgerId)
    }
  },
  { immediate: true },
)
</script>

<style scoped lang="scss">
.ledger-info-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 40px 0;
  color: var(--el-text-color-secondary);
}
.ledger-info-error {
  margin-bottom: 12px;
}
.ledger-info-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--el-fill-color-light);
  border-radius: 4px;
  margin-bottom: 12px;
  font-size: 14px;
}
.ledger-info-label {
  color: var(--el-text-color-secondary);
}
.ledger-info-value {
  font-family: monospace;
  font-size: 13px;
  word-break: break-all;
}
.dialog-footer {
  display: flex;
  justify-content: flex-end;
}
</style>
