import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { nrcsApi } from '@/api/modules'
import type { NrcsAccount } from '@/api/modules'

const STORAGE_KEY_ACCOUNT_RS = 'nrcs-accountRS'
const STORAGE_KEY_ACCOUNT_ID = 'nrcs-accountId'
const STORAGE_KEY_PUBLIC_KEY = 'nrcs-publicKey'
const STORAGE_KEY_SECRET_PHRASE = 'nrcs-secretPhrase'

export const useAccountStore = defineStore('account', () => {
  // --- State ---
  const accountRS = ref<string>('')
  const accountId = ref<string>('')
  const publicKey = ref<string>('')
  const balanceNQT = ref<string>('0')
  const effectiveBalance = ref<number>(0)
  const unconfirmedBalanceNQT = ref<string>('0')
  const forgedBalanceNQT = ref<string>('0')
  const guaranteedBalanceNQT = ref<string>('0')
  const name = ref<string>('')
  const description = ref<string>('')

  // Secret phrase stored ONLY when user opts in (for forging, sending, etc.)
  const secretPhrase = ref<string>('')

  // --- Getters ---
  const isLoggedIn = computed(() => !!accountRS.value)
  const balanceFormatted = computed(() => {
    const nqt = BigInt(balanceNQT.value || '0')
    return Number(nqt) / 100000000
  })

  // --- Actions ---

  /**
   * Login with a secret phrase (passphrase).
   * Derives the account ID, RS address, and public key from the passphrase,
   * then fetches the full account info including balance.
   */
  async function login(password: string): Promise<void> {
    // Step 1: Derive account from secret phrase
    const derived = await nrcsApi.getAccountId(password)
    if (!derived || !derived.accountRS) {
      throw new Error('Failed to derive account from secret phrase')
    }

    accountRS.value = derived.accountRS
    accountId.value = derived.account
    publicKey.value = derived.publicKey || ''
    secretPhrase.value = password

    // Step 2: Fetch full account info including balance
    try {
      const accountInfo: NrcsAccount = await nrcsApi.getAccount(derived.accountRS)
      balanceNQT.value = accountInfo.balanceNQT || '0'
      unconfirmedBalanceNQT.value = accountInfo.unconfirmedBalanceNQT || '0'
      forgedBalanceNQT.value = accountInfo.forgedBalanceNQT || '0'
      guaranteedBalanceNQT.value = accountInfo.guaranteedBalanceNQT || '0'
      effectiveBalance.value = accountInfo.effectiveBalanceNRCS || 0
      name.value = accountInfo.name || ''
      description.value = accountInfo.description || ''
    } catch {
      // Account may not exist yet — that's fine, just derive and move on
      balanceNQT.value = '0'
      effectiveBalance.value = 0
    }

    persistToStorage()
  }

  /**
   * Login by account RS address only (read-only / watch-only).
   * Does NOT store a secret phrase.
   */
  async function loginByAccount(accountRs: string): Promise<void> {
    const accountInfo: NrcsAccount = await nrcsApi.getAccount(accountRs)
    if (!accountInfo || !accountInfo.accountRS) {
      throw new Error('Account not found')
    }

    accountRS.value = accountInfo.accountRS
    accountId.value = accountInfo.account
    publicKey.value = accountInfo.publicKey || ''
    balanceNQT.value = accountInfo.balanceNQT || '0'
    unconfirmedBalanceNQT.value = accountInfo.unconfirmedBalanceNQT || '0'
    forgedBalanceNQT.value = accountInfo.forgedBalanceNQT || '0'
    guaranteedBalanceNQT.value = accountInfo.guaranteedBalanceNQT || '0'
    effectiveBalance.value = accountInfo.effectiveBalanceNRCS || 0
    name.value = accountInfo.name || ''
    description.value = accountInfo.description || ''
    secretPhrase.value = ''

    persistToStorage()
  }

  /**
   * Refresh account data from the blockchain.
   */
  async function refreshAccount(): Promise<void> {
    if (!accountRS.value) return
    try {
      const accountInfo: NrcsAccount = await nrcsApi.getAccount(accountRS.value)
      balanceNQT.value = accountInfo.balanceNQT || '0'
      unconfirmedBalanceNQT.value = accountInfo.unconfirmedBalanceNQT || '0'
      forgedBalanceNQT.value = accountInfo.forgedBalanceNQT || '0'
      guaranteedBalanceNQT.value = accountInfo.guaranteedBalanceNQT || '0'
      effectiveBalance.value = accountInfo.effectiveBalanceNRCS || 0
      name.value = accountInfo.name || ''
      description.value = accountInfo.description || ''
    } catch (e) {
      console.error('Failed to refresh account:', e)
    }
  }

  /**
   * Logout — clears all state and localStorage.
   */
  function logout(): void {
    accountRS.value = ''
    accountId.value = ''
    publicKey.value = ''
    balanceNQT.value = '0'
    effectiveBalance.value = 0
    unconfirmedBalanceNQT.value = '0'
    forgedBalanceNQT.value = '0'
    guaranteedBalanceNQT.value = '0'
    name.value = ''
    description.value = ''
    secretPhrase.value = ''

    clearStorage()
  }

  // --- Persistence ---

  function persistToStorage(): void {
    localStorage.setItem(STORAGE_KEY_ACCOUNT_RS, accountRS.value)
    localStorage.setItem(STORAGE_KEY_ACCOUNT_ID, accountId.value)
    localStorage.setItem(STORAGE_KEY_PUBLIC_KEY, publicKey.value)
    if (secretPhrase.value) {
      localStorage.setItem(STORAGE_KEY_SECRET_PHRASE, secretPhrase.value)
    }
  }

  function clearStorage(): void {
    localStorage.removeItem(STORAGE_KEY_ACCOUNT_RS)
    localStorage.removeItem(STORAGE_KEY_ACCOUNT_ID)
    localStorage.removeItem(STORAGE_KEY_PUBLIC_KEY)
    localStorage.removeItem(STORAGE_KEY_SECRET_PHRASE)
  }

  /**
   * Restore session from localStorage on app start.
   */
  function initFromStorage(): void {
    const savedRS = localStorage.getItem(STORAGE_KEY_ACCOUNT_RS)
    const savedId = localStorage.getItem(STORAGE_KEY_ACCOUNT_ID)
    const savedPK = localStorage.getItem(STORAGE_KEY_PUBLIC_KEY)
    const savedPhrase = localStorage.getItem(STORAGE_KEY_SECRET_PHRASE)

    if (savedRS) accountRS.value = savedRS
    if (savedId) accountId.value = savedId
    if (savedPK) publicKey.value = savedPK
    if (savedPhrase) secretPhrase.value = savedPhrase

    // If we have a stored session, try to refresh balance
    if (savedRS) {
      refreshAccount()
    }
  }

  // Auto-init from storage
  initFromStorage()

  return {
    // state
    accountRS,
    accountId,
    publicKey,
    balanceNQT,
    effectiveBalance,
    unconfirmedBalanceNQT,
    forgedBalanceNQT,
    guaranteedBalanceNQT,
    name,
    description,
    secretPhrase,
    // getters
    isLoggedIn,
    balanceFormatted,
    // actions
    login,
    loginByAccount,
    refreshAccount,
    logout,
    initFromStorage
  }
})
