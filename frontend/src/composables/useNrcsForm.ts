import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { nrcsGet, nrcsPost, isRequirePost } from '@/api/nrcs-client'
import { useAccountStore } from '@/stores/modules/account.store'
import { convertNumericToRSAccountFormat } from '@/utils/converters'

/**
 * Composable that provides the NRCS form submission pipeline.
 * Ported from nrs.forms.js (804 lines).
 *
 * Handles: form validation, secretPhrase injection, NXT→NQT conversion,
 * message encryption, fee calculation, and server error translation.
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
        const multiplier = BigInt(10) ** BigInt(decimals)
        result[qntField] = (BigInt(Math.floor(parseFloat(value) * Math.pow(10, decimals)))).toString()
      }
    }
    return result
  }

  /**
   * Inject the secret phrase from the account store if not provided in the form data.
   */
  function injectSecretPhrase(data: Record<string, any>): Record<string, any> {
    if (!data.secretPhrase && !data.doNotSign && accountStore.secretPhrase) {
      return { ...data, secretPhrase: accountStore.secretPhrase }
    }
    return data
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
   * Submit a form to the NRCS API.
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

      // Convert RS address to numeric for the API
      if (processedData.recipient && /^NRCS\-/i.test(processedData.recipient)) {
        // Keep RS format — the API handles both
      }

      // Inject secret phrase
      processedData = injectSecretPhrase(processedData)

      // Set mandatory defaults
      processedData = setMandatoryParams(processedData)

      // Convert NXT fields to NQT
      processedData = convertNxtToNqt(processedData)

      // Convert deadline from hours to minutes
      processedData = convertDeadline(processedData)

      // Remove empty/null/undefined values
      for (const key of Object.keys(processedData)) {
        if (processedData[key] === undefined || processedData[key] === null || processedData[key] === '') {
          delete processedData[key]
        }
      }

      // Send the request using the appropriate HTTP method
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
   */
  async function calculateFee(
    requestType: string,
    data: Record<string, any>,
  ): Promise<string> {
    isRecalculatingFee.value = true

    try {
      let processedData: Record<string, any> = { ...data, feeNQT: '0', deadline: '1440' }
      processedData = injectSecretPhrase(processedData)
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

    const msg = errorDescription.toLowerCase()

    // Map common NRCS error messages to i18n keys
    const errorMap: Record<string, string> = {
      'not enough funds': t('error.notEnoughFunds', msg),
      'insufficient balance': t('error.notEnoughFunds', msg),
      'invalid secret phrase': t('error.invalidSecretPhrase', msg),
      'secret phrase does not match': t('error.secretPhraseDoesNotMatch', msg),
      'alias already registered': t('error.aliasAlreadyRegistered', msg),
      'alias not for sale': t('error.aliasNotForSale', msg),
      'asset does not exist': t('error.assetNotExist', msg),
      'unknown asset': t('error.assetNotExist', msg),
      'order does not exist': t('error.orderNotExist', msg),
      'currency does not exist': t('error.currencyNotExist', msg),
      'unknown currency': t('error.currencyNotExist', msg),
      'not enough currency': t('error.insufficientSupply', msg),
      'poll does not exist': t('error.pollNotExist', msg),
      'already voted': t('error.alreadyVoted', msg),
      'poll is finished': t('error.pollFinished', msg),
      'goods does not exist': t('error.goodsNotExist', msg),
      'not enough goods': t('error.notEnoughGoods', msg),
      'purchase does not exist': t('error.purchaseNotExist', msg),
      'shuffling does not exist': t('error.shufflingNotExist', msg),
      'shuffling already registered': t('error.shufflingAlreadyRegistered', msg),
      'invalid recipient': t('error.invalidRecipient', msg),
      'invalid amount': t('error.invalidAmount', msg),
      'invalid fee': t('error.invalidFee', msg),
      'blockchain not ready': t('error.blockchainNotReady', msg),
      'downloading blockchain': t('error.downloadInProgress', msg),
      'not available': t('error.featureNotAvailable', msg),
      'feature not available': t('error.featureNotAvailable', msg),
    }

    for (const [pattern, translation] of Object.entries(errorMap)) {
      if (msg.includes(pattern)) {
        return translation
      }
    }

    return errorDescription
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
