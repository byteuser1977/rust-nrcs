/**
 * NRCS Server Error Translation Utility
 *
 * Ported from nrs.util.js translateServerError() function (~200+ lines).
 * Maps NRCS blockchain API error codes and messages to user-friendly
 * localized messages.
 */

import type { ComposerTranslation } from 'vue-i18n'

let _t: ComposerTranslation | null = null

/** Set the i18n translation function (call once during app init) */
export function setNrcsErrorTranslator(t: ComposerTranslation): void {
  _t = t
}

const t = (key: string, fallback?: string): string => {
  if (_t) return _t(key) || fallback || key
  return fallback || key
}

/**
 * Translates a server error response to a human-readable message.
 * Ported from the 200+ line error code switch in nrs.util.js.
 */
export function translateServerError(response: any): string {
  if (!response) return t('error.unknownError', 'Unknown error')

  const errorCode = response.errorCode
  const errorDesc = (response.errorDescription || '').toLowerCase()

  // Map of error code -> i18n keys
  switch (errorCode) {
    case 1:
      if (errorDesc.includes('incorrect')) return t('error.invalidSignature', 'Incorrect signature')
      if (errorDesc.includes('block')) return t('error.blockNotAccepted', 'Block not accepted')
      return t('error.badRequest', 'Bad request')
    case 2:
      return t('error.badContract', 'Contract not accepted')
    case 3:
      return t('error.unexpectedError', 'Unexpected server error')
    case 4:
      if (errorDesc.includes('signature')) return t('error.missingSignature', 'Transaction not signed')
      if (errorDesc.includes('referenced')) return t('error.missingReferencedTransaction', 'Referenced transaction not found')
      if (errorDesc.includes('not yet available')) return t('error.featureNotAvailable', 'Feature not available')
      if (errorDesc.includes('not enabled')) return t('error.featureNotAvailable', 'Feature not available')
      return t('error.invalidTransaction', 'Invalid transaction')
    case 5:
      if (errorDesc.includes('public key')) return t('error.publicKeyNotFound', 'Public key not found')
      if (errorDesc.includes('funds')) return t('error.notEnoughFunds', 'Not enough funds')
      if (errorDesc.includes('balance')) return t('error.notEnoughFunds', 'Insufficient balance')
      if (errorDesc.includes('download')) return t('error.downloadInProgress', 'Blockchain download in progress')
      if (errorDesc.includes('light client')) return t('error.lightClientNotAllowed', 'Operation not allowed for light client')
      return t('error.accountError', 'Account error')
    case 6:
      if (errorDesc.includes('funds')) return t('error.notEnoughFunds', 'Not enough funds')
      if (errorDesc.includes('asset')) return t('error.assetNotExist', 'Asset does not exist')
      if (errorDesc.includes('order')) return t('error.orderNotExist', 'Order does not exist')
      if (errorDesc.includes('currency')) return t('error.currencyNotExist', 'Currency does not exist')
      if (errorDesc.includes('goods')) return t('error.goodsNotExist', 'Goods do not exist')
      if (errorDesc.includes('purchase')) return t('error.purchaseNotExist', 'Purchase does not exist')
      if (errorDesc.includes('poll')) return t('error.pollNotExist', 'Poll does not exist')
      if (errorDesc.includes('shuffling')) return t('error.shufflingNotExist', 'Shuffling does not exist')
      if (errorDesc.includes('alias')) return t('error.aliasAlreadyRegistered', 'Alias error')
      if (errorDesc.includes('tagged data')) return t('error.taggedDataNotExist', 'Tagged data does not exist')
      return t('error.notEnoughFunds', 'Not enough funds')
    case 7:
      return t('error.transactionNotAllowed', 'Transaction not allowed at current state')
    case 8:
      if (errorDesc.includes('asset')) return t('error.assetNotExist', 'Asset does not exist')
      if (errorDesc.includes('currency')) return t('error.currencyNotExist', 'Currency does not exist')
      if (errorDesc.includes('goods')) return t('error.goodsNotExist', 'Goods not found')
      if (errorDesc.includes('purchase')) return t('error.purchaseNotExist', 'Purchase not found')
      if (errorDesc.includes('poll')) return t('error.pollNotExist', 'Poll not found')
      if (errorDesc.includes('shuffling')) return t('error.shufflingNotExist', 'Shuffling not found')
      if (errorDesc.includes('alias')) return t('error.aliasNotExist', 'Alias not found')
      if (errorDesc.includes('tagged data')) return t('error.taggedDataNotExist', 'Tagged data not found')
      return t('error.notFound', 'Not found')
    case 9:
      return t('error.featureNotAvailable', 'Feature not available')

    // Higher error codes
    default:
      break
  }

  // Error description pattern matching
  const patterns: [string, string][] = [
    ['not enough funds', 'error.notEnoughFunds'],
    ['insufficient balance', 'error.notEnoughFunds'],
    ['invalid secret phrase', 'error.invalidSecretPhrase'],
    ['secret phrase does not match', 'error.secretPhraseDoesNotMatch'],
    ['passphrase does not match', 'error.secretPhraseDoesNotMatch'],
    ['invalid passphrase', 'error.invalidSecretPhrase'],
    ['public key not found', 'error.publicKeyNotFound'],
    ['public key has not been', 'error.publicKeyNotFound'],
    ['alias already registered', 'error.aliasAlreadyRegistered'],
    ['alias not for sale', 'error.aliasNotForSale'],
    ['alias does not belong', 'error.invalidAlias'],
    ['asset does not exist', 'error.assetNotExist'],
    ['unknown asset', 'error.assetNotExist'],
    ['order does not exist', 'error.orderNotExist'],
    ['currency does not exist', 'error.currencyNotExist'],
    ['unknown currency', 'error.currencyNotExist'],
    ['not enough currency', 'error.insufficientSupply'],
    ['insufficient currency', 'error.insufficientSupply'],
    ['poll does not exist', 'error.pollNotExist'],
    ['already voted', 'error.alreadyVoted'],
    ['poll is finished', 'error.pollFinished'],
    ['poll has finished', 'error.pollFinished'],
    ['goods does not exist', 'error.goodsNotExist'],
    ['goods not found', 'error.goodsNotExist'],
    ['not enough goods', 'error.notEnoughGoods'],
    ['insufficient quantity', 'error.notEnoughGoods'],
    ['purchase does not exist', 'error.purchaseNotExist'],
    ['purchase not found', 'error.purchaseNotExist'],
    ['shuffling does not exist', 'error.shufflingNotExist'],
    ['shuffling not found', 'error.shufflingNotExist'],
    ['shuffling already registered', 'error.shufflingAlreadyRegistered'],
    ['invalid recipient', 'error.invalidRecipient'],
    ['recipient account does not have', 'error.invalidRecipient'],
    ['invalid amount', 'error.invalidAmount'],
    ['invalid fee', 'error.invalidFee'],
    ['blockchain not ready', 'error.blockchainNotReady'],
    ['downloading blockchain', 'error.downloadInProgress'],
    ['blockchain is downloading', 'error.downloadInProgress'],
    ['not available', 'error.featureNotAvailable'],
    ['feature not available', 'error.featureNotAvailable'],
    ['not allowed', 'error.operationNotAllowed'],
    ['transaction not signed', 'error.transactionNotSign'],
    ['invalid transaction', 'error.invalidTransaction'],
    ['invalid transaction bytes', 'error.invalidTransactionBytes'],
    ['referenced transaction', 'error.invalidReferencedTransaction'],
    ['maximum number', 'error.maximumExceeded'],
    ['too many', 'error.maximumExceeded'],
    ['deadline', 'error.invalidDeadline'],
    ['deadline must be', 'error.invalidDeadline'],
    ['ask order', 'error.orderNotExist'],
    ['bid order', 'error.orderNotExist'],
    ['cannot delete', 'error.operationNotAllowed'],
    ['is not exchangeable', 'error.currencyNotExchangeable'],
    ['is not reservable', 'error.currencyNotReservable'],
    ['is not claimable', 'error.currencyNotClaimable'],
    ['is not mintable', 'error.currencyNotMintable'],
    ['already issued', 'error.currencyAlreadyIssued'],
    ['reserve', 'error.currencyReserveError'],
    ['claim', 'error.currencyClaimError'],
    ['no data', 'error.taggedDataNotExist'],
    ['prunable', 'error.prunableDataNotAvailable'],
    ['phasing', 'error.phasingError'],
    ['control', 'error.accountControlError'],
  ]

  for (const [pattern, i18nKey] of patterns) {
    if (errorDesc.includes(pattern)) {
      return t(i18nKey, response.errorDescription || 'Server error')
    }
  }

  return response.errorDescription || t('error.serverError', 'Server error')
}

/**
 * Shorthand: extract and translate error from a caught exception.
 */
export function getErrorMessage(error: any): string {
  if (!error) return t('error.unknownError', 'Unknown error')
  if (typeof error === 'string') return error
  if (error.errorCode !== undefined || error.errorDescription !== undefined) {
    return translateServerError(error)
  }
  return error.message || error.description || t('error.serverError', 'Server error')
}
