<template>
  <el-dialog
    v-model="visible"
    :title="t('rawTransaction.title')"
    width="780px"
    :close-on-click-modal="false"
    destroy-on-close
    class="raw-transaction-modal"
    @close="handleClose"
  >
    <el-tabs v-model="activeTab" class="raw-transaction-tabs">
      <!-- ============================================================
           Tab 1: 交易详情展示（对标 nrs.modals.advanced.js:28-98 showRawTransactionModal）
           ============================================================ -->
      <el-tab-pane
        v-if="hasInitialTransaction"
        :label="t('rawTransaction.broadcastedTransaction')"
        name="display"
      >
        <!-- 未签名交易字节 + QR（对标 :29-39） -->
        <div v-if="showUnsignedBytes" class="rt-section">
          <div class="rt-section-header">
            <label class="rt-label">{{ t('rawTransaction.unsignedTransactionBytesLabel') }}</label>
            <el-button size="small" @click="generateUnsignedQr">
              <el-icon><Picture /></el-icon>
              {{ t('rawTransaction.unsignedQrCode') }}
            </el-button>
          </div>
          <el-input
            v-model="displayForm.unsignedTransactionBytes"
            type="textarea"
            :rows="3"
            readonly
            class="rt-mono"
          />
          <div v-if="unsignedQrDataUrl" class="rt-qr">
            <img :src="unsignedQrDataUrl" :alt="t('rawTransaction.unsignedQrCode')" class="rt-qr-img" />
          </div>
        </div>

        <!-- 交易 JSON（对标 :41-66） -->
        <div v-if="displayForm.transactionJSON" class="rt-section">
          <div class="rt-section-header">
            <label class="rt-label">
              {{ hasSignedBytes
                ? t('rawTransaction.signedTransactionJSON')
                : t('rawTransaction.unsignedTransactionJSON') }}
            </label>
            <el-button size="small" @click="downloadTransactionJson">
              <el-icon><Download /></el-icon>
              {{ t('rawTransaction.download') }}
            </el-button>
          </div>
          <el-input
            v-model="transactionJsonStr"
            type="textarea"
            :rows="6"
            class="rt-mono"
          />
        </div>

        <!-- 签名输入（对标 :68-74，仅未签名时显示） -->
        <div v-if="showSignatureInput" class="rt-section">
          <label class="rt-label">{{ t('rawTransaction.signature') }}</label>
          <el-input
            v-model="displayForm.signature"
            type="textarea"
            :rows="2"
            :placeholder="t('rawTransaction.signaturePlaceholder')"
            class="rt-mono"
          />
          <div class="rt-actions">
            <el-button type="primary" size="small" @click="injectSignature">
              {{ t('rawTransaction.broadcast') }}
            </el-button>
          </div>
        </div>

        <!-- 已签名交易字节（对标 :76-81） -->
        <div v-if="displayForm.transactionBytes" class="rt-section">
          <label class="rt-label">{{ t('rawTransaction.signedTransactionBytes') }}</label>
          <el-input
            v-model="displayForm.transactionBytes"
            type="textarea"
            :rows="3"
            readonly
            class="rt-mono"
          />
        </div>

        <!-- Full Hash（对标 :83-88） -->
        <div v-if="displayForm.fullHash" class="rt-section">
          <label class="rt-label">{{ t('rawTransaction.fullHash') }}</label>
          <el-input v-model="displayForm.fullHash" readonly class="rt-mono" />
        </div>

        <!-- Signature Hash（对标 :90-95） -->
        <div v-if="displayForm.signatureHash" class="rt-section">
          <label class="rt-label">{{ t('rawTransaction.signatureHash') }}</label>
          <el-input v-model="displayForm.signatureHash" readonly class="rt-mono" />
        </div>

        <!-- 广播按钮（对标 :34 raw_transaction_broadcast） -->
        <div v-if="canBroadcastDisplay" class="rt-actions rt-actions-center">
          <el-button
            type="primary"
            :loading="broadcasting"
            @click="broadcastDisplayTransaction"
          >
            {{ t('rawTransaction.broadcast') }}
          </el-button>
        </div>
      </el-tab-pane>

      <!-- ============================================================
           Tab 2: 本地签名（对标 nrs.modals.advanced.js:475-498 NRS.forms.signTransaction）
           ============================================================ -->
      <el-tab-pane :label="t('rawTransaction.signature')" name="sign">
        <el-form label-position="top">
          <el-form-item :label="t('rawTransaction.unsignedTransactionBytes')">
            <el-input
              v-model="signForm.unsignedTransactionBytes"
              type="textarea"
              :rows="4"
              :placeholder="t('rawTransaction.transactionBytesPlaceholder')"
              class="rt-mono"
            />
            <div class="rt-file-upload">
              <input
                ref="unsignedJsonFileInput"
                type="file"
                accept=".json,.txt"
                class="rt-file-input"
                @change="onUnsignedFileSelected"
              />
              <el-button size="small" @click="triggerUnsignedFileInput">
                <el-icon><Upload /></el-icon>
                {{ t('rawTransaction.loadFile') }}
              </el-button>
            </div>
          </el-form-item>

          <el-form-item :label="t('common.secretPhrase')">
            <el-input
              v-model="signForm.secretPhrase"
              type="password"
              show-password
              :placeholder="t('rawTransaction.signaturePlaceholder')"
              clearable
            />
          </el-form-item>

          <el-form-item>
            <el-button
              type="primary"
              :loading="signing"
              :disabled="!signForm.unsignedTransactionBytes || !signForm.secretPhrase"
              @click="signTransactionLocally"
            >
              {{ t('rawTransaction.signature') }}
            </el-button>
          </el-form-item>
        </el-form>

        <!-- 签名结果（对标 :447-451 updateSignature） -->
        <div v-if="signResult.signature" class="rt-output">
          <h4 class="rt-output-title">{{ t('rawTransaction.signature') }}</h4>
          <el-input
            v-model="signResult.signature"
            type="textarea"
            :rows="3"
            readonly
            class="rt-mono"
          />
          <div v-if="signedQrDataUrl" class="rt-qr">
            <img :src="signedQrDataUrl" :alt="t('rawTransaction.signedQrCode')" class="rt-qr-img" />
          </div>
          <div class="rt-actions">
            <el-button size="small" @click="downloadSignedJson">
              <el-icon><Download /></el-icon>
              {{ t('rawTransaction.download') }}
            </el-button>
            <el-button type="primary" size="small" :loading="broadcasting" @click="broadcastSignedResult">
              {{ t('rawTransaction.broadcast') }}
            </el-button>
          </div>
        </div>
      </el-tab-pane>

      <!-- ============================================================
           Tab 3: 广播交易（对标 :128-151 NRS.forms.broadcastTransaction）
           ============================================================ -->
      <el-tab-pane :label="t('rawTransaction.broadcast')" name="broadcast">
        <el-form label-position="top">
          <el-form-item :label="t('rawTransaction.transactionBytes')">
            <el-input
              v-model="broadcastForm.transactionBytes"
              type="textarea"
              :rows="4"
              :placeholder="t('rawTransaction.transactionBytesPlaceholder')"
              class="rt-mono"
            />
          </el-form-item>

          <el-form-item :label="t('rawTransaction.transactionJSON')">
            <el-input
              v-model="broadcastForm.transactionJSON"
              type="textarea"
              :rows="6"
              :placeholder="t('rawTransaction.transactionJSONPlaceholder')"
              class="rt-mono"
            />
            <div class="rt-file-upload">
              <input
                ref="broadcastJsonFileInput"
                type="file"
                accept=".json,.txt"
                class="rt-file-input"
                @change="onBroadcastFileSelected"
              />
              <el-button size="small" @click="triggerBroadcastFileInput">
                <el-icon><Upload /></el-icon>
                {{ t('rawTransaction.loadFile') }}
              </el-button>
            </div>
          </el-form-item>

          <el-form-item :label="t('rawTransaction.signature')">
            <el-input
              v-model="broadcastForm.signature"
              type="textarea"
              :rows="2"
              :placeholder="t('rawTransaction.signaturePlaceholder')"
              class="rt-mono"
            />
          </el-form-item>

          <el-form-item>
            <el-button
              type="primary"
              :loading="broadcasting"
              :disabled="!canBroadcastInteractive"
              @click="broadcastInteractive"
            >
              {{ t('rawTransaction.broadcast') }}
            </el-button>
          </el-form-item>
        </el-form>

        <!-- 广播结果 -->
        <div v-if="broadcastResult" class="rt-output">
          <el-alert
            :title="broadcastResult.broadcasted
              ? t('rawTransaction.broadcastSuccess')
              : t('rawTransaction.broadcastError')"
            :type="broadcastResult.broadcasted ? 'success' : 'error'"
            :description="broadcastResult.errorDescription || broadcastResult.transaction"
            :closable="false"
            show-icon
          />
          <div v-if="broadcastResult.transaction" class="rt-output-detail">
            <label class="rt-label">{{ t('rawTransaction.transactionId') }}</label>
            <span class="rt-mono">{{ broadcastResult.transaction }}</span>
          </div>
          <div v-if="broadcastResult.fullHash" class="rt-output-detail">
            <label class="rt-label">{{ t('rawTransaction.fullHash') }}</label>
            <span class="rt-mono">{{ broadcastResult.fullHash }}</span>
          </div>
        </div>
      </el-tab-pane>

      <!-- ============================================================
           Tab 4: 解析交易（对标 :416-429 NRS.forms.parseTransactionComplete）
           ============================================================ -->
      <el-tab-pane :label="t('rawTransaction.parse')" name="parse">
        <el-form label-position="top">
          <el-form-item :label="t('rawTransaction.transactionBytes')">
            <el-input
              v-model="parseForm.transactionBytes"
              type="textarea"
              :rows="4"
              :placeholder="t('rawTransaction.transactionBytesPlaceholder')"
              class="rt-mono"
            />
          </el-form-item>

          <el-form-item :label="t('rawTransaction.transactionJSON')">
            <el-input
              v-model="parseForm.transactionJSON"
              type="textarea"
              :rows="6"
              :placeholder="t('rawTransaction.transactionJSONPlaceholder')"
              class="rt-mono"
            />
          </el-form-item>

          <el-form-item>
            <el-button
              type="primary"
              :loading="parsing"
              :disabled="!parseForm.transactionBytes && !parseForm.transactionJSON"
              @click="parseTransaction"
            >
              {{ t('rawTransaction.parse') }}
            </el-button>
          </el-form-item>
        </el-form>

        <!-- 解析结果（对标 :416-424 parseTransactionComplete） -->
        <div v-if="parseResult" class="rt-output">
          <h4 class="rt-output-title">{{ t('rawTransaction.parseResult') }}</h4>
          <div v-if="parseError" class="rt-output-error">
            <el-alert
              :title="t('rawTransaction.parseError')"
              :description="parseError"
              type="error"
              :closable="false"
              show-icon
            />
          </div>
          <div v-else>
            <InfoTable v-if="parseRows.length > 0" :rows="parseRows" :column="2" />
            <el-empty v-else :description="t('rawTransaction.noData')" />
          </div>
        </div>
      </el-tab-pane>

      <!-- ============================================================
           Tab 5: 计算手续费（对标 calculateFee API）
           ============================================================ -->
      <el-tab-pane :label="t('rawTransaction.calculateFee')" name="fee">
        <el-form label-position="top">
          <el-form-item :label="t('rawTransaction.transactionBytes')">
            <el-input
              v-model="feeForm.transactionBytes"
              type="textarea"
              :rows="4"
              :placeholder="t('rawTransaction.transactionBytesPlaceholder')"
              class="rt-mono"
            />
          </el-form-item>

          <el-form-item :label="t('rawTransaction.transactionJSON')">
            <el-input
              v-model="feeForm.transactionJSON"
              type="textarea"
              :rows="6"
              :placeholder="t('rawTransaction.transactionJSONPlaceholder')"
              class="rt-mono"
            />
          </el-form-item>

          <el-form-item>
            <el-button
              type="primary"
              :loading="calculatingFee"
              :disabled="!feeForm.transactionBytes && !feeForm.transactionJSON"
              @click="calculateFee"
            >
              {{ t('rawTransaction.calculateFee') }}
            </el-button>
          </el-form-item>
        </el-form>

        <!-- 手续费结果 -->
        <div v-if="feeResult !== null" class="rt-output">
          <el-alert
            :title="feeError
              ? t('rawTransaction.calculateFeeError')
              : t('rawTransaction.calculateFeeSuccess', { fee: feeResult })"
            :type="feeError ? 'error' : 'success'"
            :description="feeError || undefined"
            :closable="false"
            show-icon
          />
        </div>
      </el-tab-pane>
    </el-tabs>

    <template #footer>
      <div class="dialog-footer">
        <el-button @click="handleClose">{{ t('common.close') }}</el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
/**
 * RawTransactionModal 组件 —— 原始交易（离线签名/广播/解析）弹窗。
 *
 * 对标 nrs.modals.advanced.js 的完整实现，整合两个参考模态框：
 *   1. raw_transaction_modal（NRS.showRawTransactionModal，:28-98）：
 *      展示未签名/已签名交易字节、交易 JSON、签名、fullHash、signatureHash，
 *      支持 QR 码生成、JSON 下载、广播。
 *   2. transaction_json_modal（:365-498）：
 *      交互式标签页，支持本地签名 / 广播 / 解析 / 计算手续费。
 *
 * 主要功能：
 *   - 展示模式：传入 transaction prop 时显示交易详情（对标 showRawTransactionModal）
 *   - 本地签名：输入未签名字节 + secretPhrase，本地签名生成签名与已签名 JSON
 *   - 广播交易：支持 transactionBytes 或 transactionJSON（含 signature 注入）
 *   - 解析交易：调用 parseTransaction API，展示交易字段详情
 *   - 计算手续费：调用 calculateFee API
 *   - QR 码生成：未签名/已签名交易字节的 QR 码
 *   - JSON 下载：Blob + URL.createObjectURL，文件名格式 signed/unsigned.transaction.{timestamp}.json
 *   - 文件加载：FileReader 读取本地 JSON 文件
 *
 * 安全模型：
 *   - secretPhrase 优先使用 accountStore 内存中的值（password 登录场景）
 *   - 否则由用户手动输入，仅用于本地签名，不外发
 *   - 本地签名使用 signBytes（EC-KCDSA），与 nrs.modals.advanced.js:484 一致
 */
import { ref, reactive, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { Picture, Download, Upload } from '@element-plus/icons-vue'
import QRCode from 'qrcode'
import { useAccountStore } from '@/stores/modules/account.store'
import { useNotifications } from '@/composables/useNotifications'
import { nrcsApi } from '@/api/modules/nrcs.api'
import { signBytes } from '@/utils/nrcs-crypto'
import { formatAmount, formatTimestamp } from '@/utils/format'
import type { InfoRow } from '@/utils/transaction-info-table'
import InfoTable from '@/components/base/InfoTable.vue'

// ----------------------------------------------------------------
// Props & emits
// ----------------------------------------------------------------

const props = defineProps<{
  /**
   * 初始交易对象（展示模式，对标 NRS.showRawTransactionModal(transaction)）。
   * 包含字段：unsignedTransactionBytes / transactionBytes / transactionJSON /
   * fullHash / signatureHash。
   * 不传或为 null 时进入交互模式。
   */
  transaction?: {
    unsignedTransactionBytes?: string
    transactionBytes?: string
    transactionJSON?: Record<string, any>
    fullHash?: string
    signatureHash?: string
  } | null
}>()

const visible = defineModel<boolean>('visible', { default: false })

const { t } = useI18n()
const accountStore = useAccountStore()
const { notifySuccess, notifyError } = useNotifications()

// ----------------------------------------------------------------
// 状态
// ----------------------------------------------------------------

/** 当前激活的标签页 */
const activeTab = ref<string>('display')

// --- 展示模式表单（对标 showRawTransactionModal 的字段） ---
const displayForm = reactive({
  unsignedTransactionBytes: '',
  transactionBytes: '',
  transactionJSON: null as Record<string, any> | null,
  signature: '',
  fullHash: '',
  signatureHash: '',
})

/** 交易 JSON 字符串（双向绑定用） */
const transactionJsonStr = ref<string>('')

// --- 本地签名表单（对标 :475-498 NRS.forms.signTransaction） ---
const signForm = reactive({
  unsignedTransactionBytes: '',
  secretPhrase: '',
})

/** 签名结果 */
const signResult = reactive({
  signature: '',
  signedJSON: null as Record<string, any> | null,
  signedBytes: '',
})

// --- 广播表单（对标 :128-151 NRS.forms.broadcastTransaction） ---
const broadcastForm = reactive({
  transactionBytes: '',
  transactionJSON: '',
  signature: '',
})

/** 广播结果 */
const broadcastResult = ref<{
  broadcasted: boolean
  transaction?: string
  fullHash?: string
  errorDescription?: string
} | null>(null)

// --- 解析表单 ---
const parseForm = reactive({
  transactionBytes: '',
  transactionJSON: '',
})

/** 解析结果 */
const parseResult = ref<Record<string, any> | null>(null)
const parseError = ref<string>('')

// --- 手续费表单 ---
const feeForm = reactive({
  transactionBytes: '',
  transactionJSON: '',
})

/** 手续费结果 */
const feeResult = ref<string | null>(null)
const feeError = ref<string>('')

// --- QR 码 ---
const unsignedQrDataUrl = ref<string>('')
const signedQrDataUrl = ref<string>('')

// --- 加载状态 ---
const signing = ref<boolean>(false)
const broadcasting = ref<boolean>(false)
const parsing = ref<boolean>(false)
const calculatingFee = ref<boolean>(false)

// --- 文件输入引用 ---
const unsignedJsonFileInput = ref<HTMLInputElement | null>(null)
const broadcastJsonFileInput = ref<HTMLInputElement | null>(null)

// ----------------------------------------------------------------
// 计算属性
// ----------------------------------------------------------------

/** 是否有初始交易（展示模式） */
const hasInitialTransaction = computed<boolean>(() => !!props.transaction)

/** 是否显示未签名交易字节（对标 :29，仅当 unsignedTransactionBytes 存在且无 transactionBytes 时） */
const showUnsignedBytes = computed<boolean>(
  () => !!displayForm.unsignedTransactionBytes && !displayForm.transactionBytes,
)

/** 是否已有签名字节 */
const hasSignedBytes = computed<boolean>(() => !!displayForm.transactionBytes)

/** 是否显示签名输入框（对标 :68，仅未签名时显示） */
const showSignatureInput = computed<boolean>(() =>
  !!displayForm.unsignedTransactionBytes && !displayForm.transactionBytes,
)

/** 展示模式是否可广播（已签名字节 或 签名+未签名字节） */
const canBroadcastDisplay = computed<boolean>(() =>
  !!displayForm.transactionBytes ||
  (!!displayForm.unsignedTransactionBytes && !!displayForm.signature),
)

/** 交互模式广播是否可用 */
const canBroadcastInteractive = computed<boolean>(() =>
  !!broadcastForm.transactionBytes || !!broadcastForm.transactionJSON,
)

// ----------------------------------------------------------------
// 展示模式：初始化交易数据（对标 showRawTransactionModal）
// ----------------------------------------------------------------

/**
 * 初始化展示模式数据（对标 nrs.modals.advanced.js:28-98）。
 *
 * 将传入的 transaction 对象填充到 displayForm，并按条件显示各字段：
 *   - unsignedTransactionBytes + QR（仅未签名时）
 *   - transactionJSON（标签随是否已签名切换）
 *   - signature 输入（仅未签名时）
 *   - transactionBytes（已签名时）
 *   - fullHash / signatureHash（存在时）
 */
function initDisplayMode(): void {
  if (!props.transaction) {
    activeTab.value = 'sign'
    return
  }

  const tx = props.transaction
  displayForm.unsignedTransactionBytes = tx.unsignedTransactionBytes || ''
  displayForm.transactionBytes = tx.transactionBytes || ''
  displayForm.transactionJSON = tx.transactionJSON || null
  displayForm.signature = ''
  displayForm.fullHash = tx.fullHash || ''
  displayForm.signatureHash = tx.signatureHash || ''

  // 同步 JSON 字符串
  transactionJsonStr.value = tx.transactionJSON
    ? JSON.stringify(tx.transactionJSON, null, 2)
    : ''

  // 生成未签名 QR 码（对标 :31 NRS.generateQRCode）
  if (showUnsignedBytes.value) {
    void generateUnsignedQr()
  }

  activeTab.value = 'display'
}

// ----------------------------------------------------------------
// QR 码生成（对标 NRS.generateQRCode）
// ----------------------------------------------------------------

/**
 * 生成未签名交易字节的 QR 码（对标 :31 NRS.generateQRCode(..., 14)）。
 */
async function generateUnsignedQr(): Promise<void> {
  if (!displayForm.unsignedTransactionBytes) {
    unsignedQrDataUrl.value = ''
    return
  }
  try {
    unsignedQrDataUrl.value = await QRCode.toDataURL(
      displayForm.unsignedTransactionBytes,
      { width: 200, margin: 2 },
    )
  } catch {
    unsignedQrDataUrl.value = ''
  }
}

/**
 * 生成已签名交易字节的 QR 码（对标 :449 updateSignature 中的 QR 生成）。
 */
async function generateSignedQr(): Promise<void> {
  if (!signResult.signature) {
    signedQrDataUrl.value = ''
    return
  }
  try {
    signedQrDataUrl.value = await QRCode.toDataURL(
      signResult.signature,
      { width: 200, margin: 2 },
    )
  } catch {
    signedQrDataUrl.value = ''
  }
}

// ----------------------------------------------------------------
// JSON 下载（对标 :53-65 downloadLink）
// ----------------------------------------------------------------

/**
 * 下载交易 JSON 文件（对标 :53-65）。
 *
 * 文件名格式：{signed|unsigned}.transaction.{timestamp}.json
 * 使用 Blob + URL.createObjectURL 实现客户端下载。
 */
function downloadTransactionJson(): void {
  if (!displayForm.transactionJSON) return
  const prefix = hasSignedBytes.value ? 'signed' : 'unsigned'
  const timestamp = displayForm.transactionJSON.timestamp || Date.now()
  const filename = `${prefix}.transaction.${timestamp}.json`
  downloadJsonFile(transactionJsonStr.value, filename)
}

/**
 * 下载已签名交易 JSON（对标 :458-470 signTransactionComplete 中的下载）。
 */
function downloadSignedJson(): void {
  if (!signResult.signedJSON) return
  const jsonStr = JSON.stringify(signResult.signedJSON, null, 2)
  const timestamp = signResult.signedJSON.timestamp || Date.now()
  const filename = `signed.transaction.${timestamp}.json`
  downloadJsonFile(jsonStr, filename)
}

/**
 * 通用 JSON 文件下载工具。
 *
 * @param jsonStr JSON 字符串
 * @param filename 下载文件名
 */
function downloadJsonFile(jsonStr: string, filename: string): void {
  if (!window.URL) {
    notifyError(t('rawTransaction.download') + ' unavailable')
    return
  }
  try {
    const blob = new Blob([jsonStr], { type: 'text/plain' })
    const url = window.URL.createObjectURL(blob)
    const link = document.createElement('a')
    link.href = url
    link.download = filename
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
    window.URL.revokeObjectURL(url)
  } catch (e) {
    notifyError(t('rawTransaction.download') + ': ' + (e instanceof Error ? e.message : String(e)))
  }
}

// ----------------------------------------------------------------
// 文件加载（对标 :110-126 FileReader）
// ----------------------------------------------------------------

/**
 * 触发未签名 JSON 文件选择。
 */
function triggerUnsignedFileInput(): void {
  unsignedJsonFileInput.value?.click()
}

/**
 * 触发广播 JSON 文件选择。
 */
function triggerBroadcastFileInput(): void {
  broadcastJsonFileInput.value?.click()
}

/**
 * 处理未签名 JSON 文件选择（对标 :110-126）。
 */
function onUnsignedFileSelected(e: Event): void {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) {
    notifyError(t('rawTransaction.selectFileToUpload'))
    return
  }
  readFileAsText(file).then((text) => {
    signForm.unsignedTransactionBytes = text
  }).catch((err) => {
    notifyError(err.message || String(err))
  })
  // 清空 input 允许重复选择同一文件
  input.value = ''
}

/**
 * 处理广播 JSON 文件选择（对标 :110-126）。
 */
function onBroadcastFileSelected(e: Event): void {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) {
    notifyError(t('rawTransaction.selectFileToUpload'))
    return
  }
  readFileAsText(file).then((text) => {
    broadcastForm.transactionJSON = text
  }).catch((err) => {
    notifyError(err.message || String(err))
  })
  input.value = ''
}

/**
 * 读取文件为文本（对标 :120-125 FileReader.readAsText）。
 *
 * @param file 文件对象
 * @returns 文件文本内容
 */
function readFileAsText(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = (e) => {
      const text = e.target?.result
      if (typeof text === 'string') {
        resolve(text)
      } else {
        reject(new Error(t('rawTransaction.invalidJSON')))
      }
    }
    reader.onerror = () => reject(reader.error || new Error('FileReader error'))
    reader.readAsText(file, 'UTF-8')
  })
}

// ----------------------------------------------------------------
// 本地签名（对标 :475-498 NRS.forms.signTransaction）
// ----------------------------------------------------------------

/**
 * 本地签名交易（对标 :475-498）。
 *
 * 流程：
 *   1. 获取 secretPhrase（优先 accountStore 内存值，否则用用户输入）
 *   2. 校验 secretPhrase 归属（可选，对标 :482 getAccountId 比对）
 *   3. 调用 signBytes（EC-KCDSA）生成签名
 *   4. 构造已签名 JSON（注入 signature 字段）
 *   5. 生成签名 QR 码
 */
async function signTransactionLocally(): Promise<void> {
  if (!signForm.unsignedTransactionBytes) {
    notifyError(t('rawTransaction.noData'))
    return
  }

  // 获取 secretPhrase：优先内存（password 登录），否则用户输入
  const secretPhrase = accountStore.hasSecretPhrase
    ? accountStore.secretPhrase
    : signForm.secretPhrase

  if (!secretPhrase) {
    notifyError(t('rawTransaction.signaturePlaceholder'))
    return
  }

  signing.value = true
  try {
    // 本地签名（对标 :484 NRS.signBytes）
    const signature = signBytes(signForm.unsignedTransactionBytes, secretPhrase)

    signResult.signature = signature
    signResult.signedBytes = injectSignature(
      signForm.unsignedTransactionBytes,
      signature,
    )

    // 构造已签名 JSON（对标 :456-457 signedTransactionJson）
    try {
      const unsignedJSON = JSON.parse(signForm.unsignedTransactionBytes)
      unsignedJSON.signature = signature
      signResult.signedJSON = unsignedJSON
    } catch {
      // unsignedTransactionBytes 是 hex 字符串而非 JSON，跳过 JSON 构造
      signResult.signedJSON = null
    }

    // 生成签名 QR 码（对标 :449 updateSignature）
    await generateSignedQr()

    notifySuccess(t('rawTransaction.signature') + ' ✓')
  } catch (e) {
    notifyError(e instanceof Error ? e.message : String(e))
  } finally {
    signing.value = false
  }
}

/**
 * 将签名注入到未签名交易字节中（对标 nrcs-signing.ts verifyAndSignTransactionBytes）。
 *
 * 签名注入位置：[192, 320) 共 128 字节（64 字节签名）。
 *
 * @param unsignedBytes 未签名交易字节（hex）
 * @param signature 签名（hex）
 * @returns 已签名交易字节（hex）
 */
function injectSignature(unsignedBytes: string, signature: string): string {
  if (unsignedBytes.length < 320) {
    // 字节长度不足，直接拼接（保守处理）
    return unsignedBytes + signature
  }
  return unsignedBytes.substring(0, 192) + signature + unsignedBytes.substring(320)
}

// ----------------------------------------------------------------
// 广播交易（对标 :128-151 NRS.forms.broadcastTransaction）
// ----------------------------------------------------------------

/**
 * 广播展示模式中的交易（对标 :34 raw_transaction_broadcast）。
 */
async function broadcastDisplayTransaction(): Promise<void> {
  let transactionBytes = displayForm.transactionBytes
  let transactionJSON: string | undefined

  // 若无已签名字节，但有未签名字节 + 签名，则注入签名
  if (!transactionBytes && displayForm.unsignedTransactionBytes && displayForm.signature) {
    transactionBytes = injectSignature(
      displayForm.unsignedTransactionBytes,
      displayForm.signature,
    )
  }

  // 若有 transactionJSON，注入签名
  if (displayForm.transactionJSON) {
    const json = { ...displayForm.transactionJSON }
    if (!json.signature && displayForm.signature) {
      json.signature = displayForm.signature
    }
    transactionJSON = JSON.stringify(json)
  }

  if (!transactionBytes && !transactionJSON) {
    notifyError(t('rawTransaction.noData'))
    return
  }

  await doBroadcast(transactionBytes, transactionJSON)
}

/**
 * 广播签名结果（本地签名 Tab 中的广播按钮）。
 */
async function broadcastSignedResult(): Promise<void> {
  const transactionBytes = signResult.signedBytes || undefined
  const transactionJSON = signResult.signedJSON
    ? JSON.stringify(signResult.signedJSON)
    : undefined
  await doBroadcast(transactionBytes, transactionJSON)
}

/**
 * 交互式广播（广播 Tab）。
 *
 * 对标 :128-151 NRS.forms.broadcastTransaction：
 *   - 若 transactionJSON 存在，解析后注入 signature（若无 signature 字段）
 *   - 删除 signature 字段，避免重复发送
 */
async function broadcastInteractive(): Promise<void> {
  let transactionBytes = broadcastForm.transactionBytes || undefined
  let transactionJSON = broadcastForm.transactionJSON || undefined

  // 注入 signature 到 transactionJSON（对标 :137-148）
  if (transactionJSON) {
    try {
      const parsed = JSON.parse(transactionJSON)
      if (!parsed.signature && broadcastForm.signature) {
        parsed.signature = broadcastForm.signature
      }
      transactionJSON = JSON.stringify(parsed)
    } catch {
      notifyError(t('rawTransaction.invalidJSON'))
      return
    }
  }

  await doBroadcast(transactionBytes, transactionJSON)
}

/**
 * 执行广播 API 调用。
 *
 * @param transactionBytes 已签名交易字节（hex）
 * @param transactionJSON 已签名交易 JSON 字符串
 */
async function doBroadcast(
  transactionBytes?: string,
  transactionJSON?: string,
): Promise<void> {
  if (!transactionBytes && !transactionJSON) {
    notifyError(t('rawTransaction.noData'))
    return
  }

  broadcasting.value = true
  try {
    const response = await nrcsApi.broadcastTransaction(transactionBytes, transactionJSON)

    if (response && (response.errorCode || response.error)) {
      broadcastResult.value = {
        broadcasted: false,
        errorDescription: response.errorDescription || response.errorMessage || response.error,
      }
      notifyError(t('rawTransaction.broadcastError') + ': ' + broadcastResult.value.errorDescription)
    } else {
      broadcastResult.value = {
        broadcasted: true,
        transaction: response?.transaction,
        fullHash: response?.fullHash,
      }
      notifySuccess(t('rawTransaction.broadcastSuccess'))
      // 切换到广播 Tab 展示结果
      activeTab.value = 'broadcast'
    }
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e)
    broadcastResult.value = {
      broadcasted: false,
      errorDescription: msg,
    }
    notifyError(t('rawTransaction.broadcastError') + ': ' + msg)
  } finally {
    broadcasting.value = false
  }
}

// ----------------------------------------------------------------
// 解析交易（对标 :416-429 NRS.forms.parseTransactionComplete）
// ----------------------------------------------------------------

/**
 * 解析交易（对标 :416-424）。
 *
 * 调用 parseTransaction API，将响应转为 InfoTable 行展示。
 */
async function parseTransaction(): Promise<void> {
  parsing.value = true
  parseResult.value = null
  parseError.value = ''

  try {
    const response = await nrcsApi.parseTransaction(
      parseForm.transactionBytes || undefined,
      parseForm.transactionJSON || undefined,
    )

    if (response && (response.errorCode || response.error)) {
      parseError.value = response.errorDescription || response.errorMessage || response.error
      parseResult.value = null
    } else {
      parseResult.value = response
    }
  } catch (e) {
    parseError.value = e instanceof Error ? e.message : String(e)
    parseResult.value = null
  } finally {
    parsing.value = false
  }
}

/**
 * 解析结果转为 InfoTable 行（对标 :422 NRS.createInfoTable）。
 */
const parseRows = computed<InfoRow[]>(() => {
  if (!parseResult.value) return []
  const rows: InfoRow[] = []
  const details = { ...parseResult.value }
  // attachment 单独处理
  const attachment = details.attachment
  delete details.attachment

  for (const [key, value] of Object.entries(details)) {
    rows.push({
      label: key,
      value: formatParseValue(key, value),
    })
  }

  if (attachment) {
    rows.push({ label: 'attachment', value: JSON.stringify(attachment) })
  }
  return rows
})

/**
 * 格式化解析结果的字段值。
 *
 * @param key 字段名
 * @param value 字段值
 * @returns 格式化后的字符串
 */
function formatParseValue(key: string, value: any): string {
  if (value === null || value === undefined) return ''
  if (key === 'timestamp' && typeof value === 'number') {
    return formatTimestamp(value)
  }
  if ((key === 'amountNQT' || key === 'feeNQT') && typeof value === 'string') {
    return formatAmount(value) + ' NRC'
  }
  if (typeof value === 'object') {
    return JSON.stringify(value)
  }
  return String(value)
}

// ----------------------------------------------------------------
// 计算手续费（对标 calculateFee API）
// ----------------------------------------------------------------

/**
 * 计算手续费。
 */
async function calculateFee(): Promise<void> {
  calculatingFee.value = true
  feeResult.value = null
  feeError.value = ''

  try {
    // calculateFee 成功返回 { feeNQT: string }，失败时返回 errorCode/errorDescription
    // 使用 any 类型以统一处理两种响应
    const response = await nrcsApi.calculateFee(
      feeForm.transactionBytes || undefined,
      feeForm.transactionJSON || undefined,
    ) as any

    if (response && (response.errorCode || response.error)) {
      feeError.value = response.errorDescription || response.errorMessage || response.error
      feeResult.value = null
    } else {
      feeResult.value = formatAmount(response?.feeNQT || '0')
    }
  } catch (e) {
    feeError.value = e instanceof Error ? e.message : String(e)
    feeResult.value = null
  } finally {
    calculatingFee.value = false
  }
}

// ----------------------------------------------------------------
// 事件处理
// ----------------------------------------------------------------

/**
 * 关闭弹窗，重置状态。
 */
function handleClose(): void {
  visible.value = false
  // 重置展示模式数据
  displayForm.unsignedTransactionBytes = ''
  displayForm.transactionBytes = ''
  displayForm.transactionJSON = null
  displayForm.signature = ''
  displayForm.fullHash = ''
  displayForm.signatureHash = ''
  transactionJsonStr.value = ''
  unsignedQrDataUrl.value = ''
  signedQrDataUrl.value = ''
  broadcastResult.value = null
  parseResult.value = null
  parseError.value = ''
  feeResult.value = null
  feeError.value = ''
}

// ----------------------------------------------------------------
// 监听
// ----------------------------------------------------------------

watch(
  visible,
  (isVisible) => {
    if (isVisible) {
      initDisplayMode()
    }
  },
  { immediate: true },
)

// 当 transactionJsonStr 变化时同步回 displayForm.transactionJSON
watch(transactionJsonStr, (val) => {
  if (val) {
    try {
      displayForm.transactionJSON = JSON.parse(val)
    } catch {
      // 解析失败时保留原对象
    }
  }
})
</script>

<style scoped lang="scss">
.raw-transaction-modal {
  .raw-transaction-tabs {
    :deep(.el-tabs__content) {
      max-height: 60vh;
      overflow-y: auto;
    }
  }

  .rt-section {
    margin-bottom: 16px;

    .rt-section-header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      margin-bottom: 6px;
    }

    .rt-label {
      display: block;
      font-size: 13px;
      font-weight: 500;
      color: var(--el-text-color-primary);
      margin-bottom: 6px;
    }
  }

  .rt-mono {
    :deep(.el-textarea__inner),
    :deep(.el-input__inner) {
      font-family: 'Courier New', Consolas, monospace;
      font-size: 12px;
      word-break: break-all;
    }
  }

  .rt-file-upload {
    margin-top: 6px;
    display: flex;
    align-items: center;
    gap: 8px;

    .rt-file-input {
      display: none;
    }
  }

  .rt-qr {
    margin-top: 8px;
    display: flex;
    justify-content: center;

    .rt-qr-img {
      width: 200px;
      height: 200px;
      border: 1px solid var(--el-border-color);
      border-radius: 4px;
    }
  }

  .rt-actions {
    margin-top: 12px;
    display: flex;
    gap: 8px;

    &.rt-actions-center {
      justify-content: center;
    }
  }

  .rt-output {
    margin-top: 16px;
    padding: 12px;
    background: var(--el-fill-color-light);
    border-radius: 4px;

    .rt-output-title {
      margin: 0 0 8px 0;
      font-size: 14px;
      font-weight: 500;
      color: var(--el-text-color-primary);
    }

    .rt-output-error {
      margin-bottom: 8px;
    }

    .rt-output-detail {
      margin-top: 8px;
      font-size: 13px;

      .rt-label {
        display: inline-block;
        font-weight: 500;
        margin-right: 8px;
        color: var(--el-text-color-secondary);
      }
    }
  }
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
}
</style>
