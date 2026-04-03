import { describe, it, expect, vi, beforeEach } from 'vitest'
import { useAccountApi } from '@/api/modules/account'
import type { Account, CreateAccountRequest, TransferRequest } from '@/types/account'

// Mock axios
vi.mock('axios', () => {
  return {
    default: {
      create: vi.fn(() => ({
        get: vi.fn().mockResolvedValue({ data: { code: 0, data: {} as any } }),
        post: vi.fn().mockResolvedValue({ data: { code: 0, data: {} as any } })
      }))
    }
  }
})

describe('AccountApi', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('should create account', async () => {
    const api = useAccountApi()
    const req: CreateAccountRequest = {
      pubkey: '02abc123...'
    }
    
    const result = await api.createAccount(req)
    expect(result).toBeDefined()
  })

  it('should get account by id', async () => {
    const api = useAccountApi()
    const accountId = 1
    
    const result = await api.getAccount(accountId)
    expect(result).toBeDefined()
  })

  it('should get balance', async () => {
    const api = useAccountApi()
    const accountId = 1
    
    const result = await api.getBalance(accountId)
    expect(result).toBeGreaterThanOrEqual(0)
  })

  it('should transfer funds', async () => {
    const api = useAccountApi()
    const req: TransferRequest = {
      from: 1,
      to: 2,
      amount: 100,
      nonce: 1,
      gas_price: 1,
      gas_limit: 10000,
      signature: 'signature...'
    }
    
    const result = await api.transfer(req)
    expect(result).toBeDefined()
  })
})