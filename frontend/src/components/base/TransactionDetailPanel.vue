<template>
  <div class="tx-detail-panel">
    <!-- 加载中 -->
    <div v-if="state.loading" class="tx-detail-loading">
      <el-icon class="is-loading"><Loading /></el-icon>
      <span>{{ t('common.loading') }}</span>
    </div>

    <template v-else-if="state.transaction">
      <!-- 错误提示 -->
      <el-alert
        v-if="state.error"
        :title="state.error"
        type="error"
        :closable="false"
        show-icon
        class="tx-detail-error"
      />

      <!-- 交易 ID 标题 -->
      <div class="tx-detail-header">
        <span class="tx-detail-id-label">{{ t('transaction.id') }}:</span>
        <span class="tx-detail-id-value">{{ transactionId }}</span>
      </div>

      <!-- 顶部消息区（对标 transaction_info_output_top，仅消息类交易） -->
      <div v-if="state.topMessage" class="tx-detail-message-top">
        <div class="tx-detail-message-label">
          <el-icon><Unlock /></el-icon>
          <span>{{ t('transaction.publicMessage') }}</span>
        </div>
        <div class="tx-detail-message-content">{{ state.topMessage.text }}</div>
        <!-- 消息元数据 -->
        <div class="tx-detail-message-meta">
          <span><strong>{{ t('transaction.from') }}:</strong>
            <TransactionLink v-if="senderLink" :link="senderLink" @navigate="onNavigate" />
          </span>
          <span><strong>{{ t('transaction.to') }}:</strong>
            <TransactionLink v-if="recipientLink" :link="recipientLink" @navigate="onNavigate" />
          </span>
        </div>
      </div>

      <!-- 标签页 -->
      <el-tabs v-model="activeTab" class="tx-detail-tabs">
        <!-- 交易详情标签页 -->
        <el-tab-pane :label="t('transaction.details')" name="details">
          <!-- 交易基本信息表 -->
          <InfoTable
            :rows="state.transactionDetails"
            :column="2"
            @navigate="onNavigate"
          />

          <!-- 附件信息表 -->
          <InfoTable
            v-if="state.attachmentRows.length > 0"
            :rows="state.attachmentRows"
            :column="2"
            @navigate="onNavigate"
          />

          <!-- 底部消息区（对标 transaction_info_output_bottom） -->
          <div v-if="state.bottomMessage" class="tx-detail-message-bottom">
            <div class="tx-detail-message-label">
              <el-icon><Unlock /></el-icon>
              <span>{{ t('transaction.publicMessage') }}</span>
            </div>
            <div class="tx-detail-message-content">{{ state.bottomMessage.text }}</div>
          </div>

          <!-- 消息哈希 -->
          <div v-if="state.messageHash" class="tx-detail-hash">
            <strong>{{ t('transaction.hash') }}:</strong>
            <code>{{ state.messageHash }}</code>
          </div>

          <!-- 加密消息解密区 -->
          <div v-if="state.hasEncryptedMessage" class="tx-detail-decrypt">
            <div class="tx-detail-decrypt-header">
              <el-icon><Lock /></el-icon>
              <span>{{ t('transaction.encryptedMessage') }}</span>
            </div>

            <!-- 已解密的消息 -->
            <div
              v-for="result in state.decryptionResults"
              :key="result.fieldName"
              class="tx-detail-decrypt-result"
            >
              <div class="tx-detail-decrypt-label">{{ t(`transaction.${result.label}`) }}</div>
              <div v-if="result.success" class="tx-detail-decrypt-message">{{ result.message }}</div>
              <div v-else class="tx-detail-decrypt-error">
                {{ t('transaction.decryptFailed') }}: {{ result.error }}
              </div>
            </div>

            <!-- sharedKey 输入（解密失败时） -->
            <div v-if="state.needsSharedKey" class="tx-detail-shared-key">
              <el-input
                v-model="sharedKeyInput"
                :placeholder="t('transaction.sharedKeyPlaceholder')"
                size="small"
              />
              <el-button size="small" type="primary" @click="onDecryptWithSharedKey">
                {{ t('transaction.decrypt') }}
              </el-button>
            </div>
          </div>
        </el-tab-pane>

        <!-- Phasing 详情标签页 -->
        <el-tab-pane
          v-if="hasPhasingDetails"
          :label="t('transaction.phasingDetails')"
          name="phasing"
        >
          <InfoTable
            :rows="state.phasingDetails"
            :column="2"
            @navigate="onNavigate"
          />
        </el-tab-pane>

        <!-- 操作标签页 -->
        <el-tab-pane :label="t('transaction.actions')" name="actions">
          <div class="tx-detail-actions">
            <el-button
              v-if="!isSentByCurrentAccount"
              size="small"
              @click="onSendMoney"
            >
              {{ t('transaction.sendMoney') }}
            </el-button>
            <el-button
              v-if="!isSentByCurrentAccount"
              size="small"
              @click="onSendMessage"
            >
              {{ t('transaction.sendMessage') }}
            </el-button>
            <el-button
              v-if="!isSentByCurrentAccount"
              size="small"
              @click="onAddContact"
            >
              {{ t('transaction.addContact') }}
            </el-button>
            <el-button
              v-if="canApproveTransaction"
              size="small"
              type="primary"
              @click="onApproveTransaction"
            >
              {{ t('transaction.approve') }}
            </el-button>
            <el-button
              v-if="canExtendData"
              size="small"
              @click="onExtendData"
            >
              {{ t('transaction.extendData') }}
            </el-button>
          </div>
        </el-tab-pane>
      </el-tabs>
    </template>

    <!-- 空状态 -->
    <el-empty v-else :description="t('transaction.notFound')" />
  </div>
</template>

<script setup lang="ts">
/**
 * TransactionDetailPanel 组件 —— 交易详情面板。
 *
 * 对标 nrs.modals.transaction.js（1653 行）的完整实现。
 *
 * 主要功能：
 *   1. 显示交易基本信息（ID/类型/金额/手续费/发送方/接收方/区块/确认数/时间戳等）
 *   2. 按交易类型渲染附件信息（Payment/Messaging/Asset/Marketplace/Monetary/Shuffling/TaggedData）
 *   3. 显示 Phasing 详情（投票模型/法定人数/白名单/关联交易等）
 *   4. 显示公开消息（含 legacy hex 解码）
 *   5. 解密加密消息（encryptedMessage/encryptToSelfMessage，本地解密）
 *   6. 操作按钮（发送资金/发送消息/添加联系人/审批交易/扩展数据）
 *
 * 使用 useTransactionDetail composable 处理业务逻辑，组件仅负责渲染。
 */
import { ref, watch, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Loading, Unlock, Lock } from '@element-plus/icons-vue'
import type { NrcsTransaction } from '@/api/modules/nrcs.api'
import type { LinkRef } from '@/utils/transaction-links'
import { getAccountLink } from '@/utils/transaction-links'
import { useTransactionDetail } from '@/composables/useTransactionDetail'
import InfoTable from './InfoTable.vue'
import TransactionLink from './TransactionLink.vue'

const props = defineProps<{
  /** 交易对象或交易 ID */
  transaction: NrcsTransaction | string | null
  /** 共享密钥（用于解密加密消息） */
  sharedKey?: string
  /** 是否显示为 modal（false 时作为内嵌面板） */
  modal?: boolean
}>()

const emit = defineEmits<{
  /** 链接导航事件（对标 NRS.modalStack） */
  navigate: [link: LinkRef]
  /** 发送资金 */
  sendMoney: [recipient: string]
  /** 发送消息 */
  sendMessage: [recipient: string]
  /** 添加联系人 */
  addContact: [account: string]
  /** 审批交易 */
  approveTransaction: [transactionId: string, fullHash: string]
  /** 扩展数据 */
  extendData: [transactionId: string]
}>()

const { t } = useI18n()
const {
  state,
  sharedKeyInput,
  transactionId,
  hasPhasingDetails,
  canApproveTransaction,
  canExtendData,
  isSentByCurrentAccount,
  showTransactionModal,
  decryptWithSharedKey,
} = useTransactionDetail()

/** 当前激活的标签页 */
const activeTab = ref<'details' | 'phasing' | 'actions'>('details')

/** 发送方链接（用于顶部消息区） */
const senderLink = computed(() => {
  if (!state.value.transaction) return null
  const link = getAccountLink(state.value.transaction, 'sender')
  return typeof link === 'string' ? null : link
})

/** 接收方链接（用于顶部消息区） */
const recipientLink = computed(() => {
  if (!state.value.transaction) return null
  const link = getAccountLink(state.value.transaction, 'recipient')
  return typeof link === 'string' ? null : link
})

// 监听 transaction prop 变化，自动加载
watch(
  () => props.transaction,
  async (newTx) => {
    if (newTx) {
      activeTab.value = 'details'
      await showTransactionModal(newTx, props.sharedKey)
    }
  },
  { immediate: true },
)

/**
 * 处理链接导航事件（转发给父组件）。
 */
function onNavigate(link: LinkRef): void {
  emit('navigate', link)
}

/**
 * 发送资金给当前交易的对方。
 */
function onSendMoney(): void {
  const recipient = state.value.transaction?.recipientRS
  if (recipient) emit('sendMoney', recipient)
}

/**
 * 发送消息给当前交易的对方。
 */
function onSendMessage(): void {
  const recipient = state.value.transaction?.recipientRS
  if (recipient) emit('sendMessage', recipient)
}

/**
 * 添加当前交易发送方为联系人。
 */
function onAddContact(): void {
  const account = state.value.transaction?.senderRS
  if (account) emit('addContact', account)
}

/**
 * 审批当前交易。
 */
function onApproveTransaction(): void {
  const tx = state.value.transaction
  if (tx?.transaction && tx.fullHash) {
    emit('approveTransaction', tx.transaction, tx.fullHash)
  }
}

/**
 * 扩展当前交易的数据（仅 TaggedDataUpload）。
 */
function onExtendData(): void {
  const tx = state.value.transaction
  if (tx?.transaction) emit('extendData', tx.transaction)
}

/**
 * 用 sharedKey 解密加密消息。
 */
async function onDecryptWithSharedKey(): Promise<void> {
  if (sharedKeyInput.value) {
    await decryptWithSharedKey(sharedKeyInput.value)
  }
}
</script>

<style scoped lang="scss">
.tx-detail-panel {
  padding: 12px;
}
.tx-detail-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 40px 0;
  color: var(--el-text-color-secondary);
}
.tx-detail-error {
  margin-bottom: 12px;
}
.tx-detail-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--el-fill-color-light);
  border-radius: 4px;
  margin-bottom: 12px;
  font-size: 14px;
}
.tx-detail-id-label {
  color: var(--el-text-color-secondary);
}
.tx-detail-id-value {
  font-family: monospace;
  font-size: 12px;
  word-break: break-all;
}
.tx-detail-tabs {
  margin-top: 8px;
}
.tx-detail-message-top,
.tx-detail-message-bottom {
  padding: 12px;
  background: var(--el-fill-color-lighter);
  border-radius: 4px;
  margin-bottom: 12px;
}
.tx-detail-message-label {
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--el-text-color-secondary);
  font-size: 13px;
  margin-bottom: 8px;
}
.tx-detail-message-content {
  padding: 8px;
  background: var(--el-bg-color);
  border-radius: 4px;
  word-break: break-all;
  white-space: pre-wrap;
}
.tx-detail-message-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
  margin-top: 8px;
  font-size: 13px;
  color: var(--el-text-color-regular);
}
.tx-detail-hash {
  margin-top: 12px;
  font-size: 12px;
  word-break: break-all;
  code {
    font-family: monospace;
    color: var(--el-text-color-secondary);
  }
}
.tx-detail-decrypt {
  margin-top: 16px;
  padding: 12px;
  border: 1px solid var(--el-border-color-light);
  border-radius: 4px;
}
.tx-detail-decrypt-header {
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--el-text-color-secondary);
  font-size: 13px;
  margin-bottom: 8px;
}
.tx-detail-decrypt-result {
  margin-bottom: 8px;
}
.tx-detail-decrypt-label {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-bottom: 4px;
}
.tx-detail-decrypt-message {
  padding: 8px;
  background: var(--el-fill-color-lighter);
  border-radius: 4px;
  word-break: break-all;
  white-space: pre-wrap;
}
.tx-detail-decrypt-error {
  color: var(--el-color-danger);
  font-size: 12px;
}
.tx-detail-shared-key {
  display: flex;
  gap: 8px;
  margin-top: 8px;
}
.tx-detail-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  padding: 12px 0;
}
</style>
