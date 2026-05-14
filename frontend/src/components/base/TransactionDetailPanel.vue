<template>
  <div class="tx-detail-panel" v-if="transaction">
    <el-descriptions :column="2" border size="small">
      <el-descriptions-item :label="t('transaction.id')" :span="2">{{ transaction.transaction || transaction.id }}</el-descriptions-item>
      <el-descriptions-item :label="t('transaction.type')">{{ transaction.type }} / {{ transaction.subtype }}</el-descriptions-item>
      <el-descriptions-item :label="t('transaction.amount')">{{ formatNQT(transaction.amountNQT) }} NRC</el-descriptions-item>
      <el-descriptions-item :label="t('transaction.fee')">{{ formatNQT(transaction.feeNQT) }} NRC</el-descriptions-item>
      <el-descriptions-item :label="t('transaction.deadline')">{{ transaction.deadline }}</el-descriptions-item>
      <el-descriptions-item :label="t('transaction.sender')" :span="2">{{ transaction.senderRS }}</el-descriptions-item>
      <el-descriptions-item :label="t('transaction.recipient')" :span="2">{{ transaction.recipientRS || '-' }}</el-descriptions-item>
      <el-descriptions-item :label="t('transaction.block')">{{ transaction.height || '-' }}</el-descriptions-item>
      <el-descriptions-item :label="t('transaction.confirmations')">{{ transaction.confirmations || 0 }}</el-descriptions-item>
      <el-descriptions-item :label="t('transaction.signature')" :span="2" class="text-mono">
        {{ (transaction.signature || '').slice(0, 64) }}...
      </el-descriptions-item>
      <el-descriptions-item :label="t('transaction.date')">{{ formatDate(transaction.timestamp) }}</el-descriptions-item>
      <el-descriptions-item :label="t('transaction.ecBlock')">{{ transaction.ecBlockHeight || '-' }}</el-descriptions-item>
      <el-descriptions-item :label="t('transaction.fullHash')" :span="2" class="text-mono">
        {{ (transaction.fullHash || '').slice(0, 64) }}...
      </el-descriptions-item>
      <el-descriptions-item v-if="transaction.attachment" :label="t('transaction.attachment')" :span="2">
        {{ JSON.stringify(transaction.attachment).slice(0, 200) }}
      </el-descriptions-item>
    </el-descriptions>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
defineProps<{ transaction: any }>()

function formatNQT(nqt?: string) { return nqt ? (Number(nqt) / 1e8).toFixed(2) : '0.00' }
function formatDate(ts?: number) {
  if (!ts) return ''
  const epoch = new Date(Date.UTC(2013, 10, 24, 12, 0, 0))
  return new Date(epoch.getTime() + ts * 1000).toLocaleString()
}
</script>
<style scoped lang="scss">
@use '@/assets/styles/variables' as *;
.tx-detail-panel { padding: $space-md; }
.text-mono { font-family: monospace; font-size: 12px; }
</style>
