<template>
  <div class="page-container messages-layout">
    <div class="page-header">
      <h2 class="page-title">
        <el-icon><ChatDotRound /></el-icon>
        {{ t('messages.title') }}
      </h2>
      <div class="header-actions">
        <el-button type="primary" size="small" @click="showSend = true">
          <el-icon><EditPen /></el-icon>
          {{ t('messages.send') }}
        </el-button>
        <el-button size="small" @click="showDecryptModal = true">
          <el-icon><Lock /></el-icon>
          {{ t('messages.decryptAll') }}
        </el-button>
        <el-button size="small" @click="refreshData">
          <el-icon><Refresh /></el-icon>
          {{ t('common.refresh') }}
        </el-button>
      </div>
    </div>

    <div class="messages-body">
      <!-- Left sidebar: conversation partners -->
      <div class="conversation-sidebar">
        <el-card shadow="hover" class="sidebar-card">
          <div class="sidebar-header">
            <el-input
              v-model="partnerFilter"
              :placeholder="t('messages.searchPartner')"
              clearable
              size="small"
              prefix-icon="Search"
            />
          </div>
          <div class="partner-list" v-if="filteredPartners.length">
            <div
              v-for="partner in filteredPartners"
              :key="partner.accountRS"
              class="partner-item"
              :class="{ active: selectedPartner === partner.accountRS }"
              @click="selectPartner(partner.accountRS)"
            >
              <div class="partner-avatar">
                <el-icon :size="20"><User /></el-icon>
              </div>
              <div class="partner-info">
                <span class="partner-name text-mono">{{ truncateRS(partner.accountRS) }}</span>
                <span class="partner-count text-muted">{{ partner.count }} {{ t('messages.messages') }}</span>
              </div>
              <div class="partner-last" v-if="partner.lastTimestamp">
                <span class="text-muted text-xs">{{ formatShortTime(partner.lastTimestamp) }}</span>
              </div>
            </div>
          </div>
          <div class="empty-state" v-else-if="messages.length === 0">
            <el-icon :size="32"><ChatDotRound /></el-icon>
            <p>{{ t('messages.noMessages') }}</p>
          </div>
        </el-card>
      </div>

      <!-- Right panel: message thread -->
      <div class="message-thread">
        <el-card shadow="hover" class="thread-card" v-if="selectedPartner">
          <div class="thread-header">
            <div class="thread-partner">
              <el-icon :size="18"><User /></el-icon>
              <span class="text-mono text-accent">{{ selectedPartner }}</span>
            </div>
          </div>

          <div class="thread-messages" ref="threadContainer">
            <div v-if="currentThread.length === 0" class="empty-state">
              <p>{{ t('messages.noConversation') }}</p>
            </div>
            <template v-for="(msg, idx) in currentThread" :key="msg.transaction || idx">
              <div v-if="showDateSeparator(idx, currentThread)" class="date-separator">
                <span>{{ formatShortDate(msg.timestamp) }}</span>
              </div>
              <div class="thread-message" :class="{ 'is-sent': isSentByMe(msg) }">
                <div class="msg-bubble" :class="{ 'is-sent': isSentByMe(msg) }">
                  <div class="msg-meta">
                    <span class="msg-direction text-xs">
                      {{ msg.senderRS === accountRS ? t('messages.to') : t('messages.from') }}
                      <span class="text-mono">{{ msg.senderRS === accountRS ? msg.recipientRS : msg.senderRS }}</span>
                    </span>
                    <span class="msg-time text-muted text-xs">{{ formatShortTime(msg.timestamp) }}</span>
                  </div>
                  <div class="msg-content" v-if="decryptedMessages[msg.transaction] !== undefined">
                    <p>{{ decryptedMessages[msg.transaction] || t('messages.emptyMessage') }}</p>
                  </div>
                  <div class="msg-content" v-else-if="msg.attachment?.message">
                    <p>{{ msg.attachment.message }}</p>
                  </div>
                  <div class="msg-content" v-else-if="msg.attachment?.encryptedMessage">
                    <div class="encrypted-badge">
                      <el-icon><Lock /></el-icon>
                      <span>{{ t('messages.encrypted') }}</span>
                      <el-button size="small" text type="primary" @click="decryptMessage(msg)">
                        {{ t('messages.decrypt') }}
                      </el-button>
                    </div>
                  </div>
                  <div class="msg-content" v-else>
                    <span class="text-muted">-</span>
                  </div>
                </div>
              </div>
            </template>
            <div v-if="decryptingTx" class="decrypting-spacer">
              <span class="text-muted text-sm">{{ t('common.loading') }}...</span>
            </div>
          </div>

          <div class="thread-reply">
            <el-input
              v-model="replyText"
              type="textarea"
              :rows="2"
              :placeholder="t('messages.replyPlaceholder')"
              :disabled="sendingReply"
            />
            <div class="reply-actions">
              <el-checkbox v-model="replyEncrypt" size="small">{{ t('messages.encrypt') }}</el-checkbox>
              <el-button
                type="primary"
                size="small"
                :loading="sendingReply"
                :disabled="!replyText.trim()"
                @click="sendReply"
              >
                <el-icon><Position /></el-icon>
                {{ t('messages.send') }}
              </el-button>
            </div>
          </div>
        </el-card>

        <div class="empty-state full-height" v-else>
          <el-icon :size="64"><ChatDotRound /></el-icon>
          <h3>{{ t('messages.selectConversation') }}</h3>
          <p class="text-muted">{{ t('messages.selectConversationHint') }}</p>
        </div>
      </div>
    </div>

    <SendMessageModal v-model:visible="showSend" @success="refreshData" />
    <DecryptMessagesModal v-model:visible="showDecryptModal" @decrypt="handleBatchDecrypt" />
  </div>
</template>

<script setup lang="ts">
/**
 * Messages 页面 —— 消息模块。
 *
 * 对标 nrs.messages.js（645 行）的核心功能：
 *   - getAccountMessages 拉取消息列表
 *   - 客户端本地解密加密消息（NRS.tryToDecryptMessage，:259）
 *     安全模型：secretPhrase 不出客户端，使用 decryptNote 本地解密
 *   - 可修剪消息 getPrunableMessage（:296）
 *   - 二进制消息识别（:111）
 *   - 发送消息通过 useNrcsForm 三步本地签名流程
 */
import { ref, computed, onMounted, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import {
  ChatDotRound, EditPen, Refresh, User, Lock,
  Position, Search
} from '@element-plus/icons-vue'
import { nrcsApi } from '@/api/modules/nrcs.api'
import type { NrcsMessage } from '@/api/modules/nrcs.api'
import { fromEpochTime } from '@/utils/format'
import { useAccountStore } from '@/stores/modules/account.store'
import { useNrcsForm } from '@/composables/useNrcsForm'
import { decryptNote, getPrivateKey } from '@/utils/nrcs-crypto'
import { hexStringToByteArray } from '@/utils/converters'
import SendMessageModal from '@/components/modals/SendMessageModal.vue'
import DecryptMessagesModal from '@/components/modals/DecryptMessagesModal.vue'
import { getAccountId } from '@/utils/nrcs-crypto'

const { t } = useI18n()
const accountStore = useAccountStore()
const { submitForm } = useNrcsForm()

const messages = ref<NrcsMessage[]>([])
const selectedPartner = ref<string>('')
const partnerFilter = ref('')
const showSend = ref(false)
const showDecryptModal = ref(false)
const replyText = ref('')
const replyEncrypt = ref(true)
const sendingReply = ref(false)
const decryptingTx = ref<string | null>(null)
const decryptedMessages = ref<Record<string, string | null>>({})
const threadContainer = ref<HTMLElement | null>(null)

/** 当前账户 RS（直接从 store 读取，不使用 localStorage） */
const accountRS = computed(() => accountStore.accountRS)

interface Partner {
  accountRS: string
  messages: NrcsMessage[]
  count: number
  lastTimestamp?: number
}

const partners = computed<Partner[]>(() => {
  const map = new Map<string, NrcsMessage[]>()
  for (const msg of messages.value) {
    const counterparty = getCounterparty(msg)
    if (!map.has(counterparty)) {
      map.set(counterparty, [])
    }
    map.get(counterparty)!.push(msg)
  }
  return Array.from(map.entries()).map(([accountRS, msgs]) => ({
    accountRS,
    messages: msgs.sort((a, b) => (a.timestamp || 0) - (b.timestamp || 0)),
    count: msgs.length,
    lastTimestamp: msgs.sort((a, b) => (b.timestamp || 0) - (a.timestamp || 0))[0]?.timestamp
  })).sort((a, b) => (b.lastTimestamp || 0) - (a.lastTimestamp || 0))
})

const filteredPartners = computed(() => {
  if (!partnerFilter.value.trim()) return partners.value
  const q = partnerFilter.value.toLowerCase()
  return partners.value.filter(p => p.accountRS.toLowerCase().includes(q))
})

const currentThread = computed(() => {
  if (!selectedPartner.value) return []
  const p = partners.value.find(p => p.accountRS === selectedPartner.value)
  return p?.messages || []
})

onMounted(() => {
  refreshData()
})

function getCounterparty(msg: NrcsMessage): string {
  if (msg.senderRS === accountRS.value) return msg.recipientRS || msg.recipient || ''
  return msg.senderRS || msg.sender || ''
}

function isSentByMe(msg: NrcsMessage): boolean {
  return msg.senderRS === accountRS.value
}

function selectPartner(rs: string) {
  selectedPartner.value = rs
  nextTick(() => {
    scrollToBottom()
  })
}

function truncateRS(rs: string): string {
  return rs.length > 34 ? rs.substring(0, 34) + '...' : rs
}

function scrollToBottom() {
  if (threadContainer.value) {
    threadContainer.value.scrollTop = threadContainer.value.scrollHeight
  }
}

function showDateSeparator(idx: number, thread: NrcsMessage[]): boolean {
  if (idx === 0) return true
  const prev = thread[idx - 1]
  const curr = thread[idx]
  if (!prev?.timestamp || !curr?.timestamp) return false
  return !isSameDay(prev.timestamp, curr.timestamp)
}

function isSameDay(ts1: number, ts2: number): boolean {
  const d1 = new Date(fromEpochTime(ts1))
  const d2 = new Date(fromEpochTime(ts2))
  return d1.getFullYear() === d2.getFullYear() &&
    d1.getMonth() === d2.getMonth() &&
    d1.getDate() === d2.getDate()
}

function formatShortTime(ts?: number): string {
  if (!ts) return ''
  const d = new Date(fromEpochTime(ts))
  const hh = d.getHours().toString().padStart(2, '0')
  const mm = d.getMinutes().toString().padStart(2, '0')
  return `${hh}:${mm}`
}

function formatShortDate(ts?: number): string {
  if (!ts) return ''
  const d = new Date(fromEpochTime(ts))
  const M = d.getMonth() + 1
  const dd = d.getDate()
  const yyyy = d.getFullYear()
  return `${yyyy}/${M.toString().padStart(2, '0')}/${dd.toString().padStart(2, '0')}`
}

/**
 * 拉取消息列表（对标 nrs.messages.js loadPage）。
 *
 * 同时拉取普通消息和可修剪消息，合并后按时间排序。
 */
async function refreshData() {
  try {
    const acct = accountRS.value
    if (!acct) return

    // 拉取普通消息（对标 getAccountMessages）
    const result = await nrcsApi.getAccountMessages(acct, 0, 99)
    const list: NrcsMessage[] = (result as any).messages || (result as any).transactions || []

    // 拉取可修剪消息（对标 getPrunableMessages，:296）
    try {
      const prunableResult = await nrcsApi.getPrunableMessages(acct, 0, 99)
      const prunableList: NrcsMessage[] = ((prunableResult as any).prunableMessages || []).map((m: any) => ({
        transaction: m.transaction,
        sender: m.sender,
        senderRS: m.senderRS,
        recipient: m.recipient,
        recipientRS: m.recipientRS,
        timestamp: m.timestamp,
        attachment: m.encryptedMessage
          ? { encryptedMessage: m.encryptedMessage, 'version.PrunableEncryptedMessage': 1 }
          : { message: m.message, messageIsText: true, 'version.PrunablePlainMessage': 1 },
        senderPublicKey: m.senderPublicKey,
      }))
      list.push(...prunableList)
    } catch {
      // 可修剪消息拉取失败不影响主流程
    }

    // 按时间排序
    list.sort((a, b) => (a.timestamp || 0) - (b.timestamp || 0))
    messages.value = list
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.loadError'))
  }
}

/**
 * 客户端本地解密加密消息（对标 nrs.messages.js:259 NRS.tryToDecryptMessage）。
 *
 * 安全模型：secretPhrase 不出客户端，使用 decryptNote 本地解密。
 *
 * 流程：
 *   1. 从 accountStore 获取 secretPhrase（内存暂存，不持久化）
 *   2. 从消息中获取对方公钥（senderPublicKey / recipientPublicKey）
 *   3. 调用 decryptNote 本地解密
 *   4. 若公钥缺失，尝试通过 getAccountPublicKey 拉取
 *
 * @param msg 待解密的消息
 */
async function decryptMessage(msg: NrcsMessage) {
  // 安全模型：secretPhrase 仅从 accountStore 内存获取，不读 localStorage
  const secretPhrase = accountStore.secretPhrase
  if (!secretPhrase) {
    ElMessage.warning(t('dashboard.enterSecretPhrase'))
    return
  }

  const attachment = msg.attachment
  if (!attachment || !attachment.encryptedMessage) {
    decryptedMessages.value[msg.transaction!] = t('messages.cannotDecrypt')
    return
  }

  decryptingTx.value = msg.transaction || null
  try {
    const encryptedMsg = attachment.encryptedMessage
    const dataHex = encryptedMsg.data ?? encryptedMsg.encryptedMessageData
    const nonceHex = encryptedMsg.nonce ?? encryptedMsg.encryptedMessageNonce
    const isText = encryptedMsg.isText !== false
    const isCompressed = encryptedMsg.isCompressed !== false

    if (!dataHex || !nonceHex) {
      decryptedMessages.value[msg.transaction!] = t('messages.cannotDecrypt')
      return
    }

    // 决定对方公钥：若我是发送方，用接收方公钥；否则用发送方公钥
    let otherPublicKey = isSentByMe(msg)
      ? msg.recipientPublicKey
      : msg.senderPublicKey

    // 公钥缺失时尝试通过 API 拉取（对标 NRS.getAccountPublicKey）
    if (!otherPublicKey) {
      const otherAccount = isSentByMe(msg) ? msg.recipient : msg.sender
      if (otherAccount) {
        try {
          const pkResult = await nrcsApi.getAccountPublicKey(otherAccount)
          otherPublicKey = (pkResult as any).publicKey
        } catch {
          // 拉取失败时继续，解密会失败但不会崩溃
        }
      }
    }

    if (!otherPublicKey) {
      decryptedMessages.value[msg.transaction!] = t('messages.cannotDecrypt')
      return
    }

    // 客户端本地解密（对标 NRS.decryptNote）
    const result = decryptNote(dataHex, {
      nonce: hexStringToByteArray(nonceHex),
      publicKey: hexStringToByteArray(otherPublicKey),
      privateKey: hexStringToByteArray(getPrivateKey(secretPhrase)),
      isText,
      isCompressed,
    }, secretPhrase)

    const text = result.message || t('messages.cannotDecrypt')
    decryptedMessages.value[msg.transaction!] = text
  } catch (e: any) {
    ElMessage.error(e?.message || t('messages.decryptError'))
    decryptedMessages.value[msg.transaction!] = t('messages.cannotDecrypt')
  } finally {
    decryptingTx.value = null
  }
}

/**
 * 发送回复消息（通过 useNrcsForm 三步本地签名流程）。
 *
 * 安全模型：secretPhrase 不随请求外发，由 useNrcsForm 在本地签名。
 *
 * 对标 nrs.messages.js 的 sendMessage 表单提交。
 */
async function sendReply() {
  if (!selectedPartner.value || !replyText.value.trim()) return

  // secretPhrase 仅从 accountStore 内存获取
  const secretPhrase = accountStore.secretPhrase
  if (!secretPhrase) {
    ElMessage.warning(t('dashboard.enterSecretPhrase'))
    return
  }

  sendingReply.value = true
  try {
    // 构造表单数据（对标 nrs.forms.js sendMessage 表单）
    const data: Record<string, any> = {
      recipient: selectedPartner.value,
      feeNQT: '100000000',
      deadline: 1440,
      secretPhrase,
    }

    if (replyEncrypt.value) {
      // 加密消息：使用 messageToEncrypt（服务端会加密后返回 unsignedTransactionBytes）
      data.messageToEncrypt = replyText.value
      data.messageToEncryptIsText = true
    } else {
      // 普通文本消息
      data.message = replyText.value
      data.messageIsText = true
    }

    // 通过 useNrcsForm 三步本地签名流程提交（secretPhrase 不外发）
    await submitForm('sendMessage', data, {
      successMessage: t('common.operationSuccess'),
    })

    replyText.value = ''
    await refreshData()
    nextTick(() => scrollToBottom())
  } catch (e: any) {
    // useNrcsForm 已通过 ElMessage 显示错误，此处仅记录日志
    console.error('Send reply failed:', e)
  } finally {
    sendingReply.value = false
  }
}

/**
 * 批量解密所有加密消息（对标 nrs.encryption.js:550-612 NRS.decryptAllMessages）。
 *
 * 安全模型：secretPhrase 不出客户端，本地逐条解密。
 *
 * 流程：
 *   1. 验证 secretPhrase 对应的账户 ID 与当前账户匹配
 *   2. 遍历所有消息，跳过已解密的和无 encryptedMessage 的
 *   3. 逐条调用 decryptNote 解密，存入 decryptedMessages
 *   4. 汇总成功/失败数量并提示
 *
 * @param payload 包含 secretPhrase 和可选 sharedKey
 */
async function handleBatchDecrypt(payload: { secretPhrase: string; sharedKey: string }) {
  const { secretPhrase, sharedKey } = payload
  const useSharedKey = !secretPhrase && !!sharedKey

  // 验证 secretPhrase（对标参考 :561-567）
  if (!useSharedKey) {
    try {
      const accountId = getAccountId(secretPhrase)
      const currentAccountId = accountStore.accountId?.toString()
      if (currentAccountId && accountId !== currentAccountId) {
        ElMessage.error(t('messages.incorrectPassphrase'))
        return
      }
    } catch {
      ElMessage.error(t('messages.incorrectPassphrase'))
      return
    }
  }

  let success = 0
  let error = 0

  for (const msg of messages.value) {
    // 跳过已解密的和无加密消息的（对标 :574）
    if (!msg.attachment?.encryptedMessage) continue
    if (decryptedMessages.value[msg.transaction!] !== undefined) continue

    try {
      const encryptedMsg = msg.attachment.encryptedMessage
      const dataHex = encryptedMsg.data ?? encryptedMsg.encryptedMessageData
      const nonceHex = encryptedMsg.nonce ?? encryptedMsg.encryptedMessageNonce
      const isText = encryptedMsg.isText !== false
      const isCompressed = encryptedMsg.isCompressed !== false

      if (!dataHex || !nonceHex) {
        error++
        continue
      }

      let options: any = {}
      if (useSharedKey) {
        // 使用 sharedKey 解密（对标 :578-580）
        options.sharedKey = hexStringToByteArray(sharedKey)
      } else {
        // 使用 secretPhrase 解密（对标 :581-583）
        options.nonce = hexStringToByteArray(nonceHex)
        const otherUser = msg.sender === accountStore.accountId?.toString()
          ? msg.recipient
          : msg.sender
        if (otherUser) {
          // 尝试获取对方公钥
          let otherPublicKey = msg.sender === accountStore.accountId?.toString()
            ? msg.recipientPublicKey
            : msg.senderPublicKey
          if (!otherPublicKey) {
            try {
              const pkResult = await nrcsApi.getAccountPublicKey(otherUser)
              otherPublicKey = (pkResult as any).publicKey
            } catch {
              // 获取公钥失败
            }
          }
          if (otherPublicKey) {
            options.publicKey = hexStringToByteArray(otherPublicKey)
          }
        }
        options.privateKey = hexStringToByteArray(getPrivateKey(secretPhrase))
      }

      options.isText = isText
      options.isCompressed = isCompressed

      // 客户端本地解密（对标 :590）
      const result = decryptNote(dataHex, options, secretPhrase)
      decryptedMessages.value[msg.transaction!] = result.message || t('messages.cannotDecrypt')
      success++
    } catch {
      if (!useSharedKey) {
        decryptedMessages.value[msg.transaction!] = t('messages.cannotDecrypt')
      }
      error++
    }
  }

  // 结果提示（对标 :607-611）
  if (success > 0) {
    ElMessage.success(t('messages.batchDecryptResult', { success, error }))
  } else if (error > 0) {
    ElMessage.error(t('messages.batchDecryptFailed'))
  } else {
    ElMessage.info(t('messages.noEncryptedMessages'))
  }
}
</script>

<style scoped lang="scss">
@use '@/assets/styles/variables' as *;

.messages-layout {
  height: calc(100vh - 120px);
  display: flex;
  flex-direction: column;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: $space-lg;
  flex-shrink: 0;
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

.messages-body {
  display: flex;
  gap: $space-lg;
  flex: 1;
  min-height: 0;
}

.conversation-sidebar {
  width: 300px;
  flex-shrink: 0;
  min-height: 0;
}

.sidebar-card {
  height: 100%;
  border: 1px solid $border-subtle !important;
  border-radius: $radius-lg !important;
  background: rgba($surface-800, 0.8) !important;
  display: flex;
  flex-direction: column;

  :deep(.el-card__body) {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: $space-md !important;
  }
}

.sidebar-header {
  margin-bottom: $space-md;
  flex-shrink: 0;
}

.partner-list {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.partner-item {
  display: flex;
  align-items: center;
  gap: $space-sm;
  padding: $space-sm $space-md;
  border-radius: $radius-md;
  cursor: pointer;
  transition: background $duration-fast ease;

  &:hover {
    background: rgba($primary, 0.06);
  }

  &.active {
    background: rgba($primary, 0.12);
    border: 1px solid rgba($primary, 0.2);
  }
}

.partner-avatar {
  width: 36px;
  height: 36px;
  border-radius: $radius-full;
  background: rgba($primary, 0.15);
  display: flex;
  align-items: center;
  justify-content: center;
  color: $primary;
  flex-shrink: 0;
}

.partner-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.partner-name {
  font-size: $font-size-sm;
  font-weight: 500;
  color: $text-primary;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.partner-count {
  font-size: $font-size-xs;
}

.message-thread {
  flex: 1;
  min-height: 0;
  min-width: 0;
}

.thread-card {
  height: 100%;
  border: 1px solid $border-subtle !important;
  border-radius: $radius-lg !important;
  background: linear-gradient(180deg, rgba($surface-800, 0.8), rgba($surface-900, 0.9)) !important;
  display: flex;
  flex-direction: column;

  :deep(.el-card__body) {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: 0 !important;
  }
}

.thread-header {
  padding: $space-md $space-lg;
  border-bottom: 1px solid $border-subtle;
  flex-shrink: 0;
}

.thread-partner {
  display: flex;
  align-items: center;
  gap: $space-sm;
  font-weight: 600;
}

.thread-messages {
  flex: 1;
  overflow-y: auto;
  padding: $space-lg;
  display: flex;
  flex-direction: column;
  gap: $space-sm;
}

.date-separator {
  display: flex;
  justify-content: center;
  margin: $space-md 0;

  span {
    font-size: $font-size-xs;
    color: $text-muted;
    background: rgba($surface-700, 0.8);
    padding: 2px 12px;
    border-radius: $radius-full;
  }
}

.thread-message {
  display: flex;
  &.is-sent {
    justify-content: flex-end;
  }
}

.msg-bubble {
  max-width: 70%;
  padding: $space-sm $space-md;
  border-radius: $radius-md;
  background: rgba($surface-700, 0.8);
  border: 1px solid $border-subtle;

  &.is-sent {
    background: rgba($primary, 0.12);
    border-color: rgba($primary, 0.2);
  }
}

.msg-meta {
  display: flex;
  align-items: center;
  gap: $space-sm;
  margin-bottom: 4px;
}

.msg-direction {
  color: $text-muted;
}

.msg-time {
  margin-left: auto;
}

.msg-content {
  p {
    margin: 0;
    color: $text-primary;
    line-height: 1.5;
    word-break: break-word;
  }
}

.encrypted-badge {
  display: flex;
  align-items: center;
  gap: $space-sm;
  color: $text-muted;
  font-size: $font-size-sm;
}

.thread-reply {
  padding: $space-md $space-lg;
  border-top: 1px solid $border-subtle;
  flex-shrink: 0;
}

.reply-actions {
  display: flex;
  justify-content: flex-end;
  align-items: center;
  gap: $space-sm;
  margin-top: $space-sm;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 0;
  color: $text-muted;
  gap: $space-sm;

  p {
    margin: 0;
    font-size: $font-size-sm;
  }
}

.full-height {
  height: 100%;
  padding: 80px 0;
  gap: $space-lg;

  h3 {
    margin: 0;
    color: $text-primary;
    font-weight: 600;
  }
}

.text-mono {
  font-family: $font-mono;
  font-size: $font-size-sm;
}

.text-accent {
  color: $primary;
}

.text-muted {
  color: $text-muted;
}

.text-xs {
  font-size: $font-size-xs;
}

.text-sm {
  font-size: $font-size-sm;
}

.decrypting-spacer {
  text-align: center;
  padding: $space-sm;
}
</style>
