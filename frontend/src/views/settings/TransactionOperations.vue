<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Operation /></el-icon> {{ t('settings.transactionOperations') }}</h2>
    </div>
    <el-card shadow="hover">
      <el-tabs v-model="mode">
        <el-tab-pane :label="t('settings.parseTransaction')" name="parse">
          <el-form label-width="140px">
            <el-form-item :label="t('settings.transactionBytes')">
              <el-input v-model="txBytes" type="textarea" :rows="4" placeholder="Hex transaction bytes" />
            </el-form-item>
            <el-form-item :label="t('settings.transactionJSON')">
              <el-input v-model="txJson" type="textarea" :rows="4" placeholder="Or paste transaction JSON" />
            </el-form-item>
            <el-form-item>
              <el-button type="primary" @click="parseTx">{{ t('settings.parse') }}</el-button>
            </el-form-item>
          </el-form>
          <el-descriptions v-if="parsed" :column="2" border style="margin-top:16px">
            <el-descriptions-item :label="t('common.transaction')" :span="2">{{ parsed.transaction }}</el-descriptions-item>
            <el-descriptions-item :label="t('common.type')">{{ parsed.type }}.{{ parsed.subtype }}</el-descriptions-item>
            <el-descriptions-item :label="t('common.amount')">{{ formatNQT(parsed.amountNQT) }} NRC</el-descriptions-item>
            <el-descriptions-item :label="t('common.fee')">{{ formatNQT(parsed.feeNQT) }} NRC</el-descriptions-item>
            <el-descriptions-item :label="t('common.sender')">{{ parsed.senderRS }}</el-descriptions-item>
            <el-descriptions-item :label="t('common.recipient')" :span="2">{{ parsed.recipientRS }}</el-descriptions-item>
          </el-descriptions>
        </el-tab-pane>
        <el-tab-pane :label="t('settings.broadcastTransaction')" name="broadcast">
          <el-form label-width="140px">
            <el-form-item :label="t('settings.transactionBytes')">
              <el-input v-model="broadcastBytes" type="textarea" :rows="4" placeholder="Signed transaction bytes to broadcast" />
            </el-form-item>
            <el-form-item>
              <el-button type="primary" @click="broadcastTx">{{ t('settings.broadcast') }}</el-button>
            </el-form-item>
          </el-form>
        </el-tab-pane>
      </el-tabs>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'

const { t } = useI18n()
const mode = ref('parse')
const txBytes = ref('')
const txJson = ref('')
const parsed = ref<any>(null)
const broadcastBytes = ref('')

function formatNQT(nqt: string) { return (Number(nqt || '0') / 1e8).toFixed(2) }

async function parseTx() {
  try {
    parsed.value = await nrcsApi.parseTransaction(txBytes.value || undefined, txJson.value || undefined)
  } catch (e: any) { ElMessage.error(e?.message || 'Failed to parse transaction') }
}

async function broadcastTx() {
  if (!broadcastBytes.value) return
  try {
    const result = await nrcsApi.broadcastTransaction(broadcastBytes.value)
    ElMessage.success(`Broadcast: ${(result as any).transaction || 'OK'}`)
  } catch (e: any) { ElMessage.error(e?.message || 'Failed to broadcast') }
}
</script>
<style scoped lang="scss">
@use '@/assets/styles/variables' as *;
.page-container { .page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; .page-title { font-size: 18px; font-weight: 600; display: flex; align-items: center; gap: 8px; margin: 0; } } }
</style>
