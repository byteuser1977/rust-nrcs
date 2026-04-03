import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useAccountStore } from '@/stores/account'
import { useAccountApi } from '@/api/modules/account'

vi.mock('@/api/modules/account')

describe('AccountStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('should initialize with default state', () => {
    const store = useAccountStore()
    expect(store.currentAccount).toBeNull()
    expect(store.accounts).toEqual([])
    expect(store.balances).toEqual({})
  })

  it('should fetch account successfully', async () => {
    const mockAccount = {
      id: 1,
      pubkey: '02abc123...',
      balance: 1000,
      nonce: 0
    }
    
    vi.mocked(useAccountApi).mockReturnValue({
      getAccount: vi.fn().mockResolvedValue(mockAccount as any)
    } as any)
    
    const store = useAccountStore()
    await store.fetchAccount(1)
    
    expect(store.currentAccount).toEqual(mockAccount)
  })

  it('should update balance correctly', () => {
    const store = useAccountStore()
    
    store.updateBalance(1, 1000)
    expect(store.balances[1]).toBe(1000)
    
    store.updateBalance(1, 500)
    expect(store.balances[1]).toBe(1500)
  })

  it('should clear state on logout', () => {
    const store = useAccountStore()
    
    store.currentAccount = { id: 1, pubkey: 'test', balance: 100, nonce: 0 } as any
    store.accounts = [{ id: 1, balance: 100 } as any]
    store.balances = { 1: 100 }
    
    store.clearState()
    
    expect(store.currentAccount).toBeNull()
    expect(store.accounts).toEqual([])
    expect(store.balances).toEqual({})
  })

  it('should track mulitple accounts', () => {
    const store = useAccountStore()
    
    store.addAccount({ id: 1, pubkey: 'a', balance: 100, nonce: 0 } as any)
    store.addAccount({ id: 2, pubkey: 'b', balance: 200, nonce: 0 } as any)
    
    expect(store.accounts).toHaveLength(2)
    expect(store.balances[1]).toBe(100)
    expect(store.balances[2]).toBe(200)
  })
})