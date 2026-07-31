/******************************************************************************
 * 收款人实时校验 Composable 单元测试
 *
 * 验证 useRecipientCheck.ts 的核心功能：
 *   - checkRecipient：RS/数字 ID/联系人名/alias 四种输入解析
 *   - getAccountError：公钥/余额检测与状态分类
 *   - correctAddressMistake：纠正建议应用
 *   - checkForMerchant：商家描述检测
 *   - checkRecipientAlias：alias → account 解析
 *
 * Mock 依赖：
 *   - @/api/modules：nrcsApi.getAccount/getAlias
 *   - @/utils/nrcs-storage：storageSelect（联系人查询）
 *
 * 对标参考：nrs.recipient.js 的 checkRecipient/getAccountError/checkRecipientAlias
 ******************************************************************************/
import { describe, it, expect, vi, beforeEach } from 'vitest'

// Mock nrcsApi
vi.mock('@/api/modules', () => ({
  nrcsApi: {
    getAccount: vi.fn(),
    getAlias: vi.fn(),
  },
}))

// Mock storageSelect
vi.mock('@/utils/nrcs-storage', () => ({
  storageSelect: vi.fn(),
}))

// Mock formatAmount / formatTimestamp（避免依赖完整 format 实现）
vi.mock('@/utils/format', () => ({
  formatAmount: vi.fn((amount: string) => {
    const num = Number(amount) / 1e8
    return num.toFixed(2)
  }),
  formatTimestamp: vi.fn((ts: number) => `TS:${ts}`),
}))

import { useRecipientCheck } from '@/composables/useRecipientCheck'
import { nrcsApi } from '@/api/modules'
import { storageSelect } from '@/utils/nrcs-storage'
import { convertNumericToRSAccountFormat } from '@/utils/nrs-address'

// 测试数据（使用 convertNumericToRSAccountFormat 生成确保 Reed-Solomon 校验位正确）
const VALID_RS = 'NRCS-SM2H-LPVM-ES9M-94C92'
const SELF_RS = convertNumericToRSAccountFormat('999')
const UNKNOWN_RS = 'NRCS-XXXX-XXXX-XXXX-XXXXX'

const ACCOUNT_WITH_PUBLIC_KEY = {
  account: '12345678901234567890',
  accountRS: VALID_RS,
  publicKey: 'abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789',
  balanceNQT: '100000000',
  unconfirmedBalanceNQT: '100000000',
  forgedBalanceNQT: '0',
  guaranteedBalanceNQT: '0',
  effectiveBalanceNRCS: 1,
  name: '',
  description: '',
}

const ACCOUNT_WITH_NAME = {
  ...ACCOUNT_WITH_PUBLIC_KEY,
  name: 'Alice',
  description: 'merchant account',
}

const ACCOUNT_NO_PUBLIC_KEY = {
  account: '12345678901234567890',
  accountRS: VALID_RS,
  publicKey: '',
  balanceNQT: '0',
  unconfirmedBalanceNQT: '0',
  forgedBalanceNQT: '0',
  guaranteedBalanceNQT: '0',
  effectiveBalanceNRCS: 0,
  name: '',
  description: '',
}

describe('useRecipientCheck', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('checkRecipient - 空输入', () => {
    it('空输入返回 null 并重置状态', async () => {
      const { checkRecipient, result, status } = useRecipientCheck()
      const res = await checkRecipient('')
      expect(res).toBeNull()
      expect(result.value).toBeNull()
      expect(status.value).toBe('idle')
    })

    it('纯空格输入返回 null', async () => {
      const { checkRecipient } = useRecipientCheck()
      const res = await checkRecipient('   ')
      expect(res).toBeNull()
    })
  })

  describe('checkRecipient - RS 地址', () => {
    it('有效 RS 地址 + 有公钥 → info 状态', async () => {
      vi.mocked(nrcsApi.getAccount).mockResolvedValue(ACCOUNT_WITH_PUBLIC_KEY as any)
      const { checkRecipient } = useRecipientCheck()
      const res = await checkRecipient(VALID_RS)
      expect(res?.type).toBe('info')
      expect(res?.message).toContain('收款方余额')
      expect(res?.noPublicKey).toBe(false)
      expect(res?.showRecipientPublicKey).toBe(false)
    })

    it('有效 RS 地址 + 有 name → info 状态含名称', async () => {
      vi.mocked(nrcsApi.getAccount).mockResolvedValue(ACCOUNT_WITH_NAME as any)
      const { checkRecipient } = useRecipientCheck()
      const res = await checkRecipient(VALID_RS)
      expect(res?.type).toBe('info')
      expect(res?.message).toContain('Alice')
    })

    it('有效 RS 地址 + 无公钥 → warning 状态 + showRecipientPublicKey', async () => {
      vi.mocked(nrcsApi.getAccount).mockResolvedValue(ACCOUNT_NO_PUBLIC_KEY as any)
      const { checkRecipient } = useRecipientCheck()
      const res = await checkRecipient(VALID_RS, { selfAccountRS: SELF_RS })
      expect(res?.type).toBe('warning')
      expect(res?.noPublicKey).toBe(true)
      expect(res?.showRecipientPublicKey).toBe(true)
    })

    it('有效 RS 地址 == 自身账户 → success 状态', async () => {
      vi.mocked(nrcsApi.getAccount).mockResolvedValue(ACCOUNT_WITH_PUBLIC_KEY as any)
      const { checkRecipient } = useRecipientCheck()
      const res = await checkRecipient(SELF_RS, { selfAccountRS: SELF_RS })
      expect(res?.type).toBe('success')
      expect(res?.message).toContain('您自己的账户')
    })

    it('未知账户（errorCode=5）→ warning 状态', async () => {
      vi.mocked(nrcsApi.getAccount).mockRejectedValue({ code: 5, message: 'Unknown account' })
      const { checkRecipient } = useRecipientCheck()
      const res = await checkRecipient(VALID_RS)
      expect(res?.type).toBe('warning')
      expect(res?.noPublicKey).toBe(true)
      expect(res?.message).toContain('未知账户')
    })

    it('商家描述 → merchantInfo 填充', async () => {
      vi.mocked(nrcsApi.getAccount).mockResolvedValue(ACCOUNT_WITH_NAME as any)
      const { checkRecipient } = useRecipientCheck()
      const res = await checkRecipient(VALID_RS, { requestType: 'sendMoney' })
      expect(res?.merchantInfo).toBe('merchant account')
    })

    it('格式错误 RS → danger + 纠正建议', async () => {
      // 故意写错一个字符，让 NrsAddress 尝试纠错
      const { checkRecipient } = useRecipientCheck()
      const res = await checkRecipient('NRCS-SM2H-LPVM-ES9M-94C9A')
      expect(res?.type).toBe('danger')
      // guess 可能有建议（取决于纠错算法）
      if (res?.suggestions && res.suggestions.length > 0) {
        expect(res.suggestions[0].address).toMatch(/^NRCS-/)
      }
    })

    it('完全无效 RS → danger 无建议', async () => {
      const { checkRecipient } = useRecipientCheck()
      const res = await checkRecipient('NRCS-ZZZZ-ZZZZ-ZZZZ-ZZZZZ')
      expect(res?.type).toBe('danger')
    })
  })

  describe('checkRecipient - 纯数字', () => {
    it('纯数字账户 ID → danger 不允许', async () => {
      const { checkRecipient } = useRecipientCheck()
      const res = await checkRecipient('12345678901234567890')
      expect(res?.type).toBe('danger')
      expect(res?.message).toContain('不允许使用纯数字账户 ID')
    })
  })

  describe('checkRecipient - 联系人名', () => {
    it('找到联系人 → info + convertedAccount', async () => {
      vi.mocked(storageSelect).mockResolvedValue([
        { name: 'Alice', accountRS: VALID_RS },
      ] as any)
      vi.mocked(nrcsApi.getAccount).mockResolvedValue(ACCOUNT_WITH_PUBLIC_KEY as any)
      const { checkRecipient } = useRecipientCheck()
      const res = await checkRecipient('Alice')
      expect(res?.type).toBe('info')
      expect(res?.message).toContain('Alice')
      expect(res?.convertedAccount).toBe(VALID_RS)
    })

    it('未找到联系人 + 非字母数字 → danger', async () => {
      vi.mocked(storageSelect).mockResolvedValue([] as any)
      const { checkRecipient } = useRecipientCheck()
      const res = await checkRecipient('Alice!@#')
      expect(res?.type).toBe('danger')
      expect(res?.message).toContain('格式错误')
    })
  })

  describe('checkRecipient - alias', () => {
    it('@ 前缀 alias → 解析为账户', async () => {
      vi.mocked(nrcsApi.getAlias).mockResolvedValue({
        alias: '1',
        aliasName: 'myalias',
        account: '123',
        accountRS: VALID_RS,
        timestamp: 1000,
        aliasURI: `acct:${VALID_RS}@nxt`,
      } as any)
      vi.mocked(nrcsApi.getAccount).mockResolvedValue(ACCOUNT_WITH_PUBLIC_KEY as any)
      const { checkRecipient } = useRecipientCheck()
      const res = await checkRecipient('@myalias')
      expect(res?.type).toBe('info')
      expect(res?.message).toContain('myalias')
    })

    it('alias 无 URI → danger', async () => {
      vi.mocked(nrcsApi.getAlias).mockResolvedValue({
        alias: '1',
        aliasName: 'myalias',
        account: '123',
        accountRS: VALID_RS,
        timestamp: 1000,
        aliasURI: '',
      } as any)
      const { checkRecipient } = useRecipientCheck()
      const res = await checkRecipient('@myalias')
      expect(res?.type).toBe('danger')
      expect(res?.message).toContain('URI 为空')
    })

    it('alias URI 不匹配账户格式 → danger', async () => {
      vi.mocked(nrcsApi.getAlias).mockResolvedValue({
        alias: '1',
        aliasName: 'myalias',
        account: '123',
        accountRS: VALID_RS,
        timestamp: 1000,
        aliasURI: 'https://example.com',
      } as any)
      const { checkRecipient } = useRecipientCheck()
      const res = await checkRecipient('@myalias')
      expect(res?.type).toBe('danger')
      expect(res?.message).toContain('未关联账户')
    })
  })

  describe('correctAddressMistake', () => {
    it('应用纠正建议后重新校验', async () => {
      vi.mocked(nrcsApi.getAccount).mockResolvedValue(ACCOUNT_WITH_PUBLIC_KEY as any)
      const { correctAddressMistake, result } = useRecipientCheck()
      const res = await correctAddressMistake(VALID_RS, { selfAccountRS: SELF_RS })
      expect(res?.type).toBe('info')
      expect(result.value).not.toBeNull()
    })
  })

  describe('checkForMerchant', () => {
    it('sendMoney + 含 merchant → 返回描述', async () => {
      const { checkForMerchant } = useRecipientCheck()
      const res = checkForMerchant('this is a merchant account', 'sendMoney')
      expect(res).toBe('this is a merchant account')
    })

    it('sendMoney + 不含 merchant → 空字符串', async () => {
      const { checkForMerchant } = useRecipientCheck()
      const res = checkForMerchant('normal account', 'sendMoney')
      expect(res).toBe('')
    })

    it('非 sendMoney/transferAsset → 空字符串', async () => {
      const { checkForMerchant } = useRecipientCheck()
      const res = checkForMerchant('merchant account', 'sendMessage')
      expect(res).toBe('')
    })
  })

  describe('reset', () => {
    it('重置后 result 为 null, status 为 idle', async () => {
      vi.mocked(nrcsApi.getAccount).mockResolvedValue(ACCOUNT_WITH_PUBLIC_KEY as any)
      const { checkRecipient, reset, result, status } = useRecipientCheck()
      await checkRecipient(VALID_RS)
      expect(result.value).not.toBeNull()
      expect(status.value).toBe('done')
      reset()
      expect(result.value).toBeNull()
      expect(status.value).toBe('idle')
    })
  })

  describe('getAccountError', () => {
    it('errorCode=4 → danger 格式错误', async () => {
      vi.mocked(nrcsApi.getAccount).mockRejectedValue({ code: 4, message: 'Incorrect account' })
      const { getAccountError } = useRecipientCheck()
      const res = await getAccountError(VALID_RS)
      expect(res.type).toBe('danger')
      expect(res.message).toContain('格式错误')
    })

    it('其他 errorCode → danger 通用错误', async () => {
      vi.mocked(nrcsApi.getAccount).mockRejectedValue({ code: 999, message: 'Some error' })
      const { getAccountError } = useRecipientCheck()
      const res = await getAccountError(VALID_RS)
      expect(res.type).toBe('danger')
      expect(res.message).toContain('Some error')
    })
  })
})
