import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { nrcsGet, nrcsPost, isRequirePost } from '@/api/nrcs-client'
import { useAccountStore } from '@/stores/modules/account.store'
import { convertNumericToRSAccountFormat } from '@/utils/converters'
import { convertRSToNumericAccount } from '@/utils/nrs-address'
import { translateServerError as translateNrcsError } from '@/utils/nrcs-errors'
import { getPublicKey } from '@/utils/nrcs-crypto'
import {
  getECBlock,
  verifyAndSignTransactionBytes,
  broadcastTransactionBytes,
  type TransactionFormData,
  type VerifyOptions,
} from '@/utils/nrcs-signing'

/**
 * Composable that provides the NRCS form submission pipeline.
 * Ported from nrs.forms.js (804 lines) + nrs.server.js processAjaxRequest.
 *
 * 安全模型：secretPhrase 仅在客户端本地使用，绝不随请求外发。
 * 三步签名流程：
 *   1. 发送 doNotSign 请求获取 unsignedTransactionBytes
 *   2. 本地验证 + 签名（verifyAndSignTransactionBytes）
 *   3. 广播已签名交易（broadcastTransactionBytes）
 */
export function useNrcsForm() {
  const { t } = useI18n()
  const accountStore = useAccountStore()

  const isSubmitting = ref(false)
  const errorMessage = ref('')
  const feeNQT = ref('')
  const isRecalculatingFee = ref(false)

  /** Lock modal — disable buttons, show spinner */
  function lockForm() {
    isSubmitting.value = true
    errorMessage.value = ''
  }

  /** Unlock modal — enable buttons, hide spinner */
  function unlockForm() {
    isSubmitting.value = false
  }

  /**
   * Convert NXT fields to NQT (multiply by 10^8).
   * The NRCS API expects amounts in NQT (smallest unit).
   */
  function convertNxtToNqt(data: Record<string, any>): Record<string, any> {
    const nxtFields = [
      'feeNXT', 'amountNXT', 'priceNXT', 'refundNXT', 'discountNXT',
      'minReservePerUnitNXT', 'reserveSupplyNXT', 'initialSupplyNXT',
      'maxSupplyNXT', 'broadcastFeeNXT',
    ]

    const result = { ...data }
    for (const field of nxtFields) {
      if (result[field] !== undefined && result[field] !== '') {
        const value = result[field]
        delete result[field]
        const nqtField = field.replace('NXT', 'NQT')
        // Multiply by 10^8
        result[nqtField] = (BigInt(Math.floor(parseFloat(value) * 1e8))).toString()
      }
    }
    return result
  }

  /**
   * Convert QNTf (fractional quantity) fields to QNT (base units).
   * Multiplies by 10^decimals.
   */
  function convertQntfToQnt(data: Record<string, any>, decimals: number): Record<string, any> {
    const qntfFields = ['quantityQNTf', 'unitsQNTf', 'amountPerShareQNTf']

    const result = { ...data }
    for (const field of qntfFields) {
      if (result[field] !== undefined && result[field] !== '') {
        const value = result[field]
        delete result[field]
        const qntField = field.replace('QNTf', 'QNT')
        result[qntField] = (BigInt(Math.floor(parseFloat(value) * Math.pow(10, decimals)))).toString()
      }
    }
    return result
  }

  /**
   * Set mandatory default params (feeNQT, deadline).
   */
  function setMandatoryParams(data: Record<string, any>): Record<string, any> {
    const result = { ...data }
    if (!result.feeNQT && !result.feeNXT) {
      result.feeNQT = '0'
    }
    if (!result.deadline) {
      result.deadline = '1440' // 24 hours in minutes
    }
    return result
  }

  /**
   * Convert hours-based deadline to minutes (the NRCS API expects minutes).
   */
  function convertDeadline(data: Record<string, any>): Record<string, any> {
    const result = { ...data }
    if (result.deadline && typeof result.deadline === 'string') {
      const hours = parseFloat(result.deadline)
      if (hours > 0) {
        result.deadline = String(Math.ceil(hours * 60))
      }
    }
    // deadline 可能是数字类型（来自表单 reactive）
    if (typeof result.deadline === 'number') {
      result.deadline = String(result.deadline)
    }
    return result
  }

  /**
   * Remove empty/null/undefined values from data object.
   */
  function cleanEmptyValues(data: Record<string, any>): Record<string, any> {
    const result = { ...data }
    for (const key of Object.keys(result)) {
      if (result[key] === undefined || result[key] === null || result[key] === '') {
        delete result[key]
      }
    }
    return result
  }

  /**
   * Validate recipient address format.
   */
  function validateRecipient(recipient: string): boolean {
    if (!recipient) return false
    // Accept NRCS-XXXX-XXXX-XXXX-XXXXX format
    if (/^NRCS\-/i.test(recipient)) return true
    // Accept numeric account ID
    if (/^\d{1,20}$/.test(recipient)) return true
    return false
  }

  /**
   * 将 RS 地址转换为数字 ID（参考 nrs.server.js addAddressData）。
   *
   * 如果 data.recipient 是 NRCS-XXX 格式，转换为数字 ID 以便与服务端返回的
   * unsignedTransactionBytes 中的数字 recipient 进行比对。
   * 同时保留 recipientRS 用于显示。
   */
  function addAddressData(data: Record<string, any>): Record<string, any> {
    const result = { ...data }
    if ('recipient' in result && result.recipient) {
      if (/^NRCS\-/i.test(result.recipient)) {
        result.recipientRS = result.recipient
        try {
          result.recipient = convertRSToNumericAccount(result.recipient)
        } catch {
          // 转换失败时保持原值，服务端会处理
        }
      } else {
        try {
          result.recipientRS = convertNumericToRSAccountFormat(result.recipient)
        } catch {
          // 转换失败时忽略
        }
      }
    }
    return result
  }

  /**
   * 添加缺失的默认字段（参考 nrs.server.js addMissingData）。
   *
   * 服务端构造 unsignedTransactionBytes 时，如果表单没有 amountNQT/recipient，
   * 会使用默认值 0。客户端验证时需要同样的默认值才能匹配。
   */
  function addMissingData(data: Record<string, any>): Record<string, any> {
    const result = { ...data }
    if (!('amountNQT' in result)) {
      result.amountNQT = '0'
    }
    if (!('recipient' in result)) {
      result.recipient = '0' // GENESIS
    }
    return result
  }

  /**
   * 判断请求类型是否需要直接提交 secretPhrase（不使用本地签名）。
   *
   * 参考 nrs.server.js isSubmitPassphrase：
   * getAccountId 和 decodeToken 需要服务端处理 secretPhrase，不能本地签名。
   */
  function isSubmitPassphrase(requestType: string): boolean {
    return requestType === 'getAccountId' || requestType === 'decodeToken'
  }

  /**
   * 判断请求是否为查询类型（GET，不需要签名）。
   */
  function isQueryRequest(requestType: string, data: Record<string, any>): boolean {
    // 查询请求不需要 secretPhrase
    if (!data.secretPhrase && !accountStore.secretPhrase) return true
    // calculateFee 不需要签名
    if (requestType === 'calculateFee') return true
    // doNotSign 标记
    if (data.doNotSign) return true
    return false
  }

  /**
   * 本地签名提交路径（三步签名流程）。
   *
   * 参考 nrs.server.js processAjaxRequest 的 isVolatile 分支：
   *   1. 移除 secretPhrase，添加 publicKey + EC block
   *   2. 发送请求获取 unsignedTransactionBytes
   *   3. 本地验证 + 签名（verifyAndSignTransactionBytes）
   *   4. 广播已签名交易（broadcastTransactionBytes）
   *
   * @param requestType NRCS API 请求类型
   * @param data 已处理的表单数据（含 secretPhrase）
   * @param secretPhrase 本地密钥短语
   * @param options 成功/错误回调
   * @returns API 响应或广播结果
   */
  async function submitWithLocalSigning(
    requestType: string,
    data: Record<string, any>,
    secretPhrase: string,
    options?: {
      successMessage?: string
      errorMessage?: string
      onSuccess?: (response: any) => void
      onError?: (error: any) => void
    },
  ): Promise<any> {
    // ── 第 1 步：准备请求数据 ──────────────────────────────────────────────
    // 移除 secretPhrase，添加 publicKey（参考 nrs.server.js line 291-311）
    const signingData: Record<string, any> = { ...data }
    delete signingData.secretPhrase

    const publicKey = accountStore.publicKey || getPublicKey(secretPhrase)
    signingData.publicKey = publicKey

    // 添加 EC block 参数（参考 NRS.getECBlock）
    // TODO: 阶段 1.x 接入全局 isTestNet 标志（constants.store / app.store）
    const ecBlock = getECBlock(false)
    signingData.ecBlockId = ecBlock.id
    signingData.ecBlockHeight = ecBlock.height

    // ── 第 2 步：发送请求获取 unsignedTransactionBytes ─────────────────────
    const response = await nrcsPost(requestType, signingData)

    // 检查服务端错误
    if (response.errorCode || response.error) {
      const errorMsg = response.errorDescription || response.error || t('common.operationFailed')
      throw { message: errorMsg, errorCode: response.errorCode, ...response }
    }

    // 如果服务端没有返回 unsignedTransactionBytes（非交易请求），直接返回响应
    if (!response.unsignedTransactionBytes) {
      const successMsg = options?.successMessage || t('common.operationSuccess')
      ElMessage.success(successMsg)
      if (options?.onSuccess) options.onSuccess(response)
      return response
    }

    // ── 第 3 步：本地验证 + 签名 ───────────────────────────────────────────
    // 准备验证数据：转换 RS 地址 + 添加缺失字段（参考 addAddressData + addMissingData）
    let verifyData = addAddressData(signingData)
    verifyData = addMissingData(verifyData)

    const verifyOptions: VerifyOptions = {
      accountPublicKey: publicKey,
      isVerifyECBlock: false, // EC block 由服务端填充，不严格校验
      isTestNet: false,
    }

    const signResult = verifyAndSignTransactionBytes(
      response.unsignedTransactionBytes,
      requestType,
      verifyData as TransactionFormData,
      response,
      secretPhrase,
      verifyOptions,
    )

    if (!signResult.ok) {
      throw {
        message: signResult.errorDescription || t('common.operationFailed'),
        errorCode: signResult.errorCode,
      }
    }

    // ── 第 4 步：广播或返回已签名交易 ──────────────────────────────────────
    // broadcast=false 模式：不广播，返回已签名 payload（用于"不广播"调试模式）
    if (data.broadcast === 'false' || data.broadcast === false) {
      const result = {
        ...response,
        transactionBytes: signResult.payload,
        signature: signResult.signature,
        broadcasted: false,
      }
      const successMsg = options?.successMessage || t('common.operationSuccess')
      ElMessage.success(successMsg)
      if (options?.onSuccess) options.onSuccess(result)
      return result
    }

    // 正常模式：广播已签名交易
    const broadcastResult = await broadcastTransactionBytes(
      signResult.payload!,
      response,
      verifyData as TransactionFormData,
      {},
    )

    if (!broadcastResult.broadcasted) {
      throw {
        message: broadcastResult.errorDescription || t('common.operationFailed'),
        errorCode: broadcastResult.errorCode,
      }
    }

    const successMsg = options?.successMessage || t('common.operationSuccess')
    ElMessage.success(successMsg)
    if (options?.onSuccess) options.onSuccess(broadcastResult)
    return broadcastResult
  }

  /**
   * Submit a form to the NRCS API.
   *
   * 自动检测是否需要本地签名：
   *   - 如果有 secretPhrase 且不是查询/提交密码请求 → 走三步本地签名流程
   *   - 否则 → 直接发送请求（GET 查询、calculateFee 等）
   *
   * @param requestType The NRCS API request type (e.g., 'sendMoney', 'issueAsset')
   * @param data Form data as a key-value object
   * @param options.successMessage i18n key for success toast
   * @param options.errorMessage i18n key for error toast
   * @returns The API response data on success, or throws on failure
   */
  async function submitForm(
    requestType: string,
    data: Record<string, any>,
    options?: {
      successMessage?: string
      errorMessage?: string
      onSuccess?: (response: any) => void
      onError?: (error: any) => void
    },
  ): Promise<any> {
    lockForm()

    try {
      // Validate recipient if present
      if (data.recipient && !validateRecipient(data.recipient)) {
        throw new Error(t('validation.invalidAddress'))
      }

      // Build the request data through the pipeline
      let processedData = { ...data }

      // Set mandatory defaults
      processedData = setMandatoryParams(processedData)

      // Convert NXT fields to NQT
      processedData = convertNxtToNqt(processedData)

      // Convert deadline from hours to minutes
      processedData = convertDeadline(processedData)

      // Remove empty/null/undefined values
      processedData = cleanEmptyValues(processedData)

      // Determine signing mode
      const secretPhrase = processedData.secretPhrase || accountStore.secretPhrase
      const submitPassphrase = isSubmitPassphrase(requestType)
      const needsLocalSigning = !!secretPhrase && !submitPassphrase && !processedData.doNotSign

      if (needsLocalSigning) {
        // ── 本地签名路径（三步签名流程） ──────────────────────────────────
        // secretPhrase 仅在客户端本地使用，绝不随请求外发
        const signingData = { ...processedData }
        if (!signingData.secretPhrase && accountStore.secretPhrase) {
          signingData.secretPhrase = accountStore.secretPhrase
        }

        return await submitWithLocalSigning(requestType, signingData, secretPhrase, options)
      } else {
        // ── 直接发送路径（查询、getAccountId、calculateFee 等） ────────────
        // 对于 getAccountId/decodeToken，需要发送 secretPhrase 到服务端
        if (submitPassphrase && accountStore.secretPhrase && !processedData.secretPhrase) {
          processedData.secretPhrase = accountStore.secretPhrase
        }

        const response = isRequirePost(requestType, processedData)
          ? await nrcsPost(requestType, processedData)
          : await nrcsGet(requestType, processedData)

        // Success handling
        unlockForm()

        const successMsg = options?.successMessage || t('common.operationSuccess')
        ElMessage.success(successMsg)

        if (options?.onSuccess) {
          options.onSuccess(response)
        }

        return response
      }
    } catch (error: any) {
      unlockForm()

      const message = error?.message || error?.errorDescription || options?.errorMessage || t('common.operationFailed')
      errorMessage.value = message
      ElMessage.error(message)

      if (options?.onError) {
        options.onError(error)
      }

      throw error
    }
  }

  /**
   * Calculate the fee for a transaction without broadcasting it.
   *
   * 不发送 secretPhrase 到服务端，使用 doNotSign + publicKey 模式。
   */
  async function calculateFee(
    requestType: string,
    data: Record<string, any>,
  ): Promise<string> {
    isRecalculatingFee.value = true

    try {
      let processedData: Record<string, any> = { ...data, feeNQT: '0', deadline: '1440' }
      // 移除 secretPhrase，不发送到服务端
      delete processedData.secretPhrase
      // 添加 publicKey（如果已登录）
      if (accountStore.publicKey) {
        processedData.publicKey = accountStore.publicKey
      }
      processedData = convertNxtToNqt(processedData)

      // Remove empty values
      for (const key of Object.keys(processedData)) {
        if (processedData[key] === undefined || processedData[key] === null || processedData[key] === '') {
          delete processedData[key]
        }
      }

      const response = await nrcsGet<{ feeNQT: string }>('calculateFee', {
        transactionJSON: JSON.stringify({ ...processedData, requestType }),
      })

      const fee = response?.feeNQT || '0'
      feeNQT.value = fee
      return fee
    } catch (error: any) {
      console.error('Fee calculation failed:', error)
      feeNQT.value = ''
      return ''
    } finally {
      isRecalculatingFee.value = false
    }
  }

  /**
   * Translate a server error response to a user-friendly message.
   * Ported from the 200-line switch in nrs.util.js.
   */
  function translateServerError(errorDescription: string | undefined): string {
    if (!errorDescription) return t('common.unknownError')
    return translateNrcsError({ errorDescription, errorCode: undefined })
  }

  return {
    // State
    isSubmitting,
    errorMessage,
    feeNQT,
    isRecalculatingFee,
    // Actions
    submitForm,
    calculateFee,
    translateServerError,
    lockForm,
    unlockForm,
  }
}
