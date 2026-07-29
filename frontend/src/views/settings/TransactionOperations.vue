<template>
  <div class="page-container">
    <div class="page-header">
      <h2 class="page-title"><el-icon><Operation /></el-icon> {{ t('settings.transactionOperations') }}</h2>
    </div>
    <el-card shadow="hover">
      <el-tabs v-model="mode">
        <!-- Tab 1: Parse Transaction -->
        <el-tab-pane :label="t('settings.parseTransaction')" name="parse">
          <el-form label-width="160px" style="max-width: 620px">
            <el-form-item :label="t('settings.transactionBytes')">
              <el-input v-model="txBytes" type="textarea" :rows="4" placeholder="Hex transaction bytes" />
            </el-form-item>
            <el-form-item :label="t('settings.transactionJSON')">
              <el-input v-model="txJson" type="textarea" :rows="4" placeholder="Or paste transaction JSON" />
            </el-form-item>
            <el-form-item>
              <el-button type="primary" @click="parseTx" :loading="parsing">
                {{ t('settings.parse') }}
              </el-button>
            </el-form-item>
          </el-form>
          <el-descriptions v-if="parsed" :column="2" border style="margin-top: 16px">
            <el-descriptions-item :label="t('common.transaction')" :span="2">
              <span class="mono-text-sm">{{ parsed.transaction }}</span>
            </el-descriptions-item>
            <el-descriptions-item :label="t('common.type')">
              {{ parsed.type }}.{{ parsed.subtype }}
            </el-descriptions-item>
            <el-descriptions-item :label="t('common.timestamp')">
              {{ formatTimestamp(parsed.timestamp) }}
            </el-descriptions-item>
            <el-descriptions-item :label="t('common.amount') + ' (NRC)'">
              {{ formatNrc(parsed.amountNQT) }}
            </el-descriptions-item>
            <el-descriptions-item :label="t('common.fee') + ' (NRC)'">
              {{ formatNrc(parsed.feeNQT) }}
            </el-descriptions-item>
            <el-descriptions-item :label="t('common.sender')" :span="2">
              {{ parsed.senderRS || parsed.sender }}
            </el-descriptions-item>
            <el-descriptions-item :label="t('common.recipient')" :span="2">
              {{ parsed.recipientRS || parsed.recipient || '-' }}
            </el-descriptions-item>
            <el-descriptions-item :label="t('settings.signature')" :span="2">
              <span class="mono-text-sm">{{ parsed.signature || '-' }}</span>
            </el-descriptions-item>
            <el-descriptions-item :label="t('settings.fullHash')" :span="2">
              <span class="mono-text-sm">{{ parsed.fullHash || '-' }}</span>
            </el-descriptions-item>
          </el-descriptions>
          <!-- Raw JSON display -->
          <div v-if="parsed" style="margin-top: 12px">
            <el-collapse>
              <el-collapse-item :title="t('settings.rawJSON')">
                <pre class="raw-json">{{ JSON.stringify(parsed, null, 2) }}</pre>
              </el-collapse-item>
            </el-collapse>
          </div>
        </el-tab-pane>

        <!-- Tab 2: Broadcast Transaction -->
        <el-tab-pane :label="t('settings.broadcastTransaction')" name="broadcast">
          <el-form label-width="160px" style="max-width: 620px">
            <el-form-item :label="t('settings.transactionBytes')">
              <el-input v-model="broadcastBytes" type="textarea" :rows="4" placeholder="Signed transaction bytes to broadcast" />
            </el-form-item>
            <el-form-item>
              <el-button type="primary" @click="broadcastTx" :loading="broadcasting">
                {{ t('settings.broadcast') }}
              </el-button>
            </el-form-item>
          </el-form>
          <el-alert
            v-if="broadcastResult"
            :title="t('common.operationSuccess')"
            :description="broadcastResult"
            type="success"
            :closable="false"
            show-icon
            style="margin-top: 16px"
          />
        </el-tab-pane>

        <!-- Tab 3: Sign Transaction -->
        <el-tab-pane :label="t('settings.signTransaction')" name="sign">
          <el-form label-width="160px" style="max-width: 620px">
            <el-form-item :label="t('settings.unsignedBytes')">
              <el-input v-model="unsignedBytes" type="textarea" :rows="4" placeholder="Unsigned transaction bytes" />
            </el-form-item>
            <el-form-item :label="t('settings.unsignedJSON')">
              <el-input v-model="unsignedJson" type="textarea" :rows="4" placeholder="Or paste unsigned transaction JSON" />
            </el-form-item>
            <el-form-item :label="t('common.secretPhrase')">
              <el-input v-model="signSecretPhrase" type="password" show-password placeholder="Enter secret phrase to sign" />
            </el-form-item>
            <el-form-item>
              <el-button type="primary" @click="signTx" :loading="signing">
                {{ t('settings.sign') }}
              </el-button>
            </el-form-item>
          </el-form>
          <div v-if="signedResult" style="margin-top: 16px">
            <el-form-item :label="t('settings.signedBytes')" label-width="160px">
              <div class="signed-display">
                <el-input :model-value="signedResult" readonly type="textarea" :rows="4" />
                <el-button size="small" type="primary" plain @click="copySigned">
                  <el-icon><CopyDocument /></el-icon> {{ t('common.copy') }}
                </el-button>
              </div>
            </el-form-item>
            <el-form-item :label="t('settings.fullHash')" label-width="160px" v-if="signedFullHash">
              <span class="mono-text-sm">{{ signedFullHash }}</span>
            </el-form-item>
          </div>
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
import { formatTimestamp, formatNrc } from '@/utils/format'

const { t } = useI18n()

const mode = ref('parse')

// Parse tab
const txBytes = ref('')
const txJson = ref('')
const parsed = ref<any>(null)
const parsing = ref(false)

// Broadcast tab
const broadcastBytes = ref('')
const broadcasting = ref(false)
const broadcastResult = ref('')

// Sign tab
const unsignedBytes = ref('')
const unsignedJson = ref('')
const signSecretPhrase = ref('')
const signedResult = ref('')
const signedFullHash = ref('')
const signing = ref(false)

async function parseTx() {
  if (!txBytes.value.trim() && !txJson.value.trim()) {
    ElMessage.warning(t('validation.required'))
    return
  }
  parsing.value = true
  try {
    parsed.value = await nrcsApi.parseTransaction(
      txBytes.value.trim() || undefined,
      txJson.value.trim() || undefined,
    )
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.operationFailed'))
  } finally {
    parsing.value = false
  }
}

async function broadcastTx() {
  if (!broadcastBytes.value.trim()) {
    ElMessage.warning(t('validation.required'))
    return
  }
  broadcasting.value = true
  try {
    const result = await nrcsApi.broadcastTransaction(broadcastBytes.value.trim())
    broadcastResult.value = JSON.stringify(result, null, 2)
    ElMessage.success(t('common.operationSuccess'))
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.operationFailed'))
  } finally {
    broadcasting.value = false
  }
}

async function signTx() {
  if (!signSecretPhrase.value.trim()) {
    ElMessage.warning(t('validation.required'))
    return
  }
  if (!unsignedBytes.value.trim() && !unsignedJson.value.trim()) {
    ElMessage.warning(t('validation.required'))
    return
  }
  signing.value = true
  try {
    const result = await nrcsApi.signTransaction(
      unsignedBytes.value.trim() || undefined,
      unsignedJson.value.trim() || undefined,
      signSecretPhrase.value.trim(),
    )
    signedResult.value = (result as any)?.signedTransactionBytes || (result as any)?.transactionBytes || JSON.stringify(result)
    signedFullHash.value = (result as any)?.fullHash || (result as any)?.transaction || ''
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.operationFailed'))
  } finally {
    signing.value = false
  }
}

async function copySigned() {
  try {
    await navigator.clipboard.writeText(signedResult.value)
    ElMessage.success(t('common.copied'))
  } catch {
    ElMessage.error(t('common.copyFailed'))
  }
}
</script>

<style scoped lang="scss">
.page-container {
  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
    .page-title {
      font-size: 18px;
      font-weight: 600;
      display: flex;
      align-items: center;
      gap: 8px;
      margin: 0;
    }
  }
}
.mono-text-sm {
  font-family: 'Roboto Mono', monospace;
  font-size: 12px;
  color: #606266;
  word-break: break-all;
}
.raw-json {
  background: #f5f7fa;
  padding: 12px;
  border-radius: 6px;
  font-family: 'Roboto Mono', monospace;
  font-size: 12px;
  overflow-x: auto;
  max-height: 400px;
  overflow-y: auto;
}
.signed-display {
  display: flex;
  flex-direction: column;
  gap: 8px;
  width: 100%;
}
</style>
