<template>
  <div class="contract-verify-page">
    <el-card>
      <template #header>
        <span>{{ t('contract.verify') || '交易验证' }}</span>
      </template>

      <el-form :model="form" label-width="120px" style="max-width: 720px">
        <el-form-item :label="t('contract.verifyBytes') || '交易字节（Hex）'">
          <el-input
            v-model="form.txBytes"
            type="textarea"
            :rows="4"
            placeholder="粘贴交易字节（Hex）..."
          />
        </el-form-item>
        <el-form-item :label="t('contract.verifyJson') || '交易 JSON'">
          <el-input
            v-model="form.txJson"
            type="textarea"
            :rows="4"
            placeholder="或粘贴未签名交易 JSON..."
          />
        </el-form-item>
        <el-form-item>
          <el-button type="primary" :loading="verifying" @click="verifyTx">
            {{ t('contract.verifyAction') || '解析验证' }}
          </el-button>
        </el-form-item>
      </el-form>

      <!-- 验证结果 -->
      <el-descriptions v-if="result" :column="2" border style="margin-top: 16px">
        <el-descriptions-item :label="t('common.transaction')" :span="2">
          <span class="mono">{{ result.transaction }}</span>
        </el-descriptions-item>
        <el-descriptions-item :label="t('common.type')">
          {{ result.type }}.{{ result.subtype }}
        </el-descriptions-item>
        <el-descriptions-item :label="t('common.timestamp')">
          {{ formatTimestamp(result.timestamp) }}
        </el-descriptions-item>
        <el-descriptions-item :label="`${t('common.amount')} (NRC)`">
          {{ formatNrc(result.amountNQT) }}
        </el-descriptions-item>
        <el-descriptions-item :label="`${t('common.fee')} (NRC)`">
          {{ formatNrc(result.feeNQT) }}
        </el-descriptions-item>
        <el-descriptions-item :label="t('common.sender')" :span="2">
          {{ result.senderRS || result.sender || '-' }}
        </el-descriptions-item>
        <el-descriptions-item :label="t('common.recipient')" :span="2">
          {{ result.recipientRS || result.recipient || '-' }}
        </el-descriptions-item>
        <el-descriptions-item :label="t('contract.verifySignature') || '签名'" :span="2">
          <span class="mono">{{ result.signature || '-' }}</span>
        </el-descriptions-item>
        <el-descriptions-item :label="t('contract.verifyFullHash') || 'Full Hash'" :span="2">
          <span class="mono">{{ result.fullHash || '-' }}</span>
        </el-descriptions-item>
      </el-descriptions>

      <!-- 原始 JSON -->
      <el-collapse v-if="result" style="margin-top: 12px">
        <el-collapse-item :title="t('contract.verifyRawJson') || '原始 JSON'">
          <pre class="raw-json">{{ JSON.stringify(result, null, 2) }}</pre>
        </el-collapse-item>
      </el-collapse>
    </el-card>
  </div>
</template>

<script setup lang="ts">
/**
 * 交易验证工具页面
 *
 * 注：NRCS 无智能合约（EVM）功能，本页面原为"合约验证"占位实现。
 * 现改为调用真实的 parseTransaction API 验证交易字节/JSON 是否有效，
 * 并展示解析结果（对标 nrs.util.js 的交易解析逻辑）。
 */
import { reactive, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { formatNrc, formatTimestamp } from '@/utils/format'

const { t } = useI18n()

const form = reactive({
  txBytes: '',
  txJson: ''
})

const verifying = ref(false)
const result = ref<any>(null)

/**
 * 解析并验证交易（字节或 JSON 二选一，优先字节）
 */
async function verifyTx() {
  if (!form.txBytes.trim() && !form.txJson.trim()) {
    ElMessage.warning(t('validation.required'))
    return
  }
  verifying.value = true
  result.value = null
  try {
    result.value = await nrcsApi.parseTransaction(
      form.txBytes.trim() || undefined,
      form.txJson.trim() || undefined
    )
    ElMessage.success(t('common.operationSuccess'))
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.operationFailed'))
  } finally {
    verifying.value = false
  }
}
</script>

<style scoped lang="scss">
.contract-verify-page {
  .mono {
    font-family: 'Monaco', 'Consolas', monospace;
    font-size: 12px;
    color: #606266;
    word-break: break-all;
  }

  .raw-json {
    background: #f5f7fa;
    padding: 12px;
    border-radius: 6px;
    font-family: 'Monaco', 'Consolas', monospace;
    font-size: 12px;
    overflow-x: auto;
    max-height: 400px;
    overflow-y: auto;
  }
}
</style>
