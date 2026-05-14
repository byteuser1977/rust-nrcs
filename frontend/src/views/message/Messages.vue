<template>
  <div class="page-container messages-page">
    <div class="page-header">
      <h2 class="page-title"><el-icon><ChatDotRound /></el-icon> {{ t('messages.title') }}</h2>
      <div class="header-actions">
        <el-button type="primary" size="small" @click="showSend = true"><el-icon><EditPen /></el-icon> {{ t('messages.send') }}</el-button>
        <el-button size="small" @click="refreshData"><el-icon><Refresh /></el-icon> {{ t('common.refresh') }}</el-button>
      </div>
    </div>
    <el-card shadow="hover" v-loading="loading" class="messages-card">
      <div v-if="!items.length && !loading" class="empty-state">
        <el-icon :size="48"><ChatDotRound /></el-icon>
        <p>{{ t('messages.noMessages') }}</p>
      </div>
      <div v-else class="message-list">
        <div v-for="msg in items" :key="msg.transaction" class="message-item" :class="{ unread: !msg.isRead }">
          <div class="msg-header">
            <span class="msg-sender text-mono">{{ msg.senderRS }}</span>
            <span v-if="msg.recipientRS" class="msg-arrow">&rarr;</span>
            <span v-if="msg.recipientRS" class="msg-recipient text-mono">{{ msg.recipientRS }}</span>
            <span class="msg-time">{{ formatDate(msg.timestamp) }}</span>
          </div>
          <div class="msg-body" v-if="msg.message">
            <p>{{ msg.isText ? msg.message : `[${t('messages.encrypted')}]` }}</p>
          </div>
          <div class="msg-body" v-if="msg.attachment">
            <el-tag size="small" type="info">{{ t('messages.attachment') }}: {{ msg.attachment?.name || 'file' }}</el-tag>
          </div>
          <div class="msg-actions">
            <el-button size="small" text type="primary" @click="replyTo(msg)">{{ t('messages.reply') }}</el-button>
          </div>
        </div>
      </div>
      <div class="pagination-container" v-if="total > 20">
        <el-pagination v-model:current-page="page" :page-size="20" :total="total" layout="prev,pager,next" @current-change="refreshData" />
      </div>
    </el-card>
    <SendMessageModal v-model:visible="showSend" @success="refreshData" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'
import SendMessageModal from '@/components/modals/SendMessageModal.vue'

const { t } = useI18n()
const loading = ref(false)
const items = ref<any[]>([])
const total = ref(0)
const page = ref(1)
const showSend = ref(false)

function formatDate(ts?: number) {
  if (!ts) return ''
  const epoch = new Date(Date.UTC(2013, 10, 24, 12, 0, 0))
  return new Date(epoch.getTime() + ts * 1000).toLocaleString()
}

onMounted(() => refreshData())

async function refreshData() {
  loading.value = true
  try {
    const accountId = localStorage.getItem('nrcs_account_id') || ''
    const result = await nrcsApi.getAccountMessages(accountId, (page.value - 1) * 20, page.value * 20 - 1)
    items.value = result.messages || result.transactions || []
    total.value = items.value.length
  } catch (e: any) { ElMessage.error(e?.message || t('common.loadError')) }
  finally { loading.value = false }
}

function replyTo(msg: any) { showSend.value = true }
</script>
<style scoped lang="scss">
@use '@/assets/styles/variables' as *;
.messages-page { max-width: 900px; margin: 0 auto; }
.page-container { .page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; .page-title { font-size: 18px; font-weight: 600; display: flex; align-items: center; gap: 8px; margin: 0; } .header-actions { display: flex; gap: 8px; } } }
.messages-card { min-height: 400px; }
.message-item { border-bottom: 1px solid $border-subtle; padding: $space-md $space-md; transition: background 0.2s; &.unread { background: rgba($primary, 0.04); } &:hover { background: rgba(255,255,255,0.03); } }
.msg-header { display: flex; align-items: center; gap: $space-sm; margin-bottom: $space-xs; font-size: $font-size-sm; .msg-sender, .msg-recipient { font-weight: 500; color: $primary; } .msg-arrow { color: $text-muted; } .msg-time { margin-left: auto; color: $text-muted; font-size: $font-size-xs; } }
.msg-body { margin: $space-xs 0; p { margin: 0; color: $text-secondary; line-height: 1.5; } }
.msg-actions { margin-top: $space-xs; }
.empty-state { display: flex; flex-direction: column; align-items: center; justify-content: center; padding: 80px 0; color: $text-muted; gap: $space-md; }
.pagination-container { display: flex; justify-content: center; padding: $space-lg 0 $space-md; }
</style>
