/******************************************************************************
 * 交易详情链接工具单元测试
 *
 * 验证 transaction-links.ts 的核心功能：
 *   - getTransactionLink：交易链接构造
 *   - getAccountLink：账户链接构造（字符串/对象/字段路径）
 *   - getBlockLink / getAssetLink / getCurrencyLink：其他链接构造
 *   - getAccountTitle：账户展示标题
 *   - getAccountForDecryption：解密账户决策
 *   - convertNumericToRSAccount：数字 ID → RS 转换
 *
 * 对标参考：nrs.util.js / nrs.js 中的链接辅助函数
 ******************************************************************************/
import { describe, it, expect } from 'vitest'
import {
  getTransactionLink,
  getAccountLink,
  getBlockLink,
  getAssetLink,
  getCurrencyLink,
  getAccountTitle,
  getAccountForDecryption,
  convertNumericToRSAccount,
} from '@/utils/transaction-links'

describe('transaction-links', () => {
  // ==========================================================================
  // getTransactionLink
  // ==========================================================================
  describe('getTransactionLink', () => {
    it('使用字符串 ID 构造链接', () => {
      const link = getTransactionLink('123456789')
      expect(link).toEqual({
        type: 'transaction',
        value: '123456789',
        text: '123456789',
      })
    })

    it('使用数字 ID 构造链接', () => {
      const link = getTransactionLink(12345)
      expect(link.value).toBe('12345')
      expect(link.type).toBe('transaction')
    })

    it('使用自定义显示文本', () => {
      const link = getTransactionLink('123', '交易 #123')
      expect(link.text).toBe('交易 #123')
      expect(link.value).toBe('123')
    })
  })

  // ==========================================================================
  // getAccountLink
  // ==========================================================================
  describe('getAccountLink', () => {
    it('undefined 输入返回占位符 "-"', () => {
      expect(getAccountLink(undefined)).toBe('-')
    })

    it('空字符串输入返回占位符 "-"', () => {
      expect(getAccountLink('')).toBe('-')
    })

    it('RS 地址字符串直接构造链接', () => {
      const rs = 'NRCS-SM2H-LPVM-ES9M-94C92'
      const link = getAccountLink(rs)
      expect(link).toEqual({
        type: 'account',
        value: rs,
        text: rs,
      })
    })

    it('数字 ID 字符串转换为 RS 地址', () => {
      const link = getAccountLink('123456789')
      expect(link).not.toBe('-')
      expect(typeof link).toBe('object')
      if (typeof link === 'object') {
        expect(link.type).toBe('account')
        // 转换后的 RS 地址应包含连字符
        expect(link.value.includes('-')).toBe(true)
        // NRCS 主网前缀
        expect(link.value.startsWith('NRCS-')).toBe(true)
      }
    })

    it('对象输入 + field=sender 取 sender/senderRS 字段', () => {
      const tx = {
        sender: '111',
        senderRS: 'NRCS-AAAA-AAAA-AAAA-AAAAA',
        recipient: '222',
        recipientRS: 'NRCS-BBBB-BBBB-BBBB-BBBBB',
      }
      const link = getAccountLink(tx, 'sender')
      expect(link).toEqual({
        type: 'account',
        value: 'NRCS-AAAA-AAAA-AAAA-AAAAA',
        text: 'NRCS-AAAA-AAAA-AAAA-AAAAA',
      })
    })

    it('对象输入 + field=recipient 取 recipient/recipientRS 字段', () => {
      const tx = {
        sender: '111',
        senderRS: 'NRCS-AAAA-AAAA-AAAA-AAAAA',
        recipient: '222',
        recipientRS: 'NRCS-BBBB-BBBB-BBBB-BBBBB',
      }
      const link = getAccountLink(tx, 'recipient')
      expect(link).toEqual({
        type: 'account',
        value: 'NRCS-BBBB-BBBB-BBBB-BBBBB',
        text: 'NRCS-BBBB-BBBB-BBBB-BBBBB',
      })
    })

    it('对象输入 + field=account 取 account/accountRS 字段', () => {
      const accountObj = {
        account: '333',
        accountRS: 'NRCS-CCCC-CCCC-CCCC-CCCCC',
      }
      const link = getAccountLink(accountObj, 'account')
      expect(link).toEqual({
        type: 'account',
        value: 'NRCS-CCCC-CCCC-CCCC-CCCCC',
        text: 'NRCS-CCCC-CCCC-CCCC-CCCCC',
      })
    })

    it('对象输入 + undefined field 默认取 account/accountRS', () => {
      const accountObj = {
        account: '333',
        accountRS: 'NRCS-CCCC-CCCC-CCCC-CCCCC',
      }
      const link = getAccountLink(accountObj)
      expect(link).toEqual({
        type: 'account',
        value: 'NRCS-CCCC-CCCC-CCCC-CCCCC',
        text: 'NRCS-CCCC-CCCC-CCCC-CCCCC',
      })
    })

    it('对象输入无相关字段返回 "-"', () => {
      expect(getAccountLink({}, 'sender')).toBe('-')
    })

    it('对象输入仅有数字 ID 时转换为 RS', () => {
      const link = getAccountLink({ sender: '123456789' }, 'sender')
      expect(typeof link).toBe('object')
      if (typeof link === 'object') {
        expect(link.value.startsWith('NRCS-')).toBe(true)
      }
    })

    it('优先使用 RS 地址作为显示文本', () => {
      const link = getAccountLink(
        { sender: '111', senderRS: 'NRCS-XXXX-XXXX-XXXX-XXXXX' },
        'sender',
      )
      expect(typeof link).toBe('object')
      if (typeof link === 'object') {
        expect(link.value).toBe('NRCS-XXXX-XXXX-XXXX-XXXXX')
        expect(link.text).toBe('NRCS-XXXX-XXXX-XXXX-XXXXX')
      }
    })
  })

  // ==========================================================================
  // getBlockLink / getAssetLink / getCurrencyLink
  // ==========================================================================
  describe('getBlockLink', () => {
    it('使用数字高度构造链接', () => {
      const link = getBlockLink(12345)
      expect(link).toEqual({
        type: 'block',
        value: '12345',
        text: '12345',
      })
    })

    it('使用字符串高度构造链接', () => {
      const link = getBlockLink('99999')
      expect(link.value).toBe('99999')
      expect(link.type).toBe('block')
    })

    it('自定义显示文本', () => {
      const link = getBlockLink(100, '区块 100')
      expect(link.text).toBe('区块 100')
    })
  })

  describe('getAssetLink', () => {
    it('使用资产 ID 构造链接', () => {
      const link = getAssetLink('123')
      expect(link).toEqual({
        type: 'asset',
        value: '123',
        text: '123',
      })
    })

    it('使用资产名称作为显示文本', () => {
      const link = getAssetLink('123', 'MYASSET')
      expect(link.text).toBe('MYASSET')
      expect(link.value).toBe('123')
    })
  })

  describe('getCurrencyLink', () => {
    it('使用货币 ID 构造链接', () => {
      const link = getCurrencyLink('456')
      expect(link).toEqual({
        type: 'currency',
        value: '456',
        text: '456',
      })
    })

    it('使用货币代码作为显示文本', () => {
      const link = getCurrencyLink('456', 'USD')
      expect(link.text).toBe('USD')
    })
  })

  // ==========================================================================
  // getAccountTitle
  // ==========================================================================
  describe('getAccountTitle', () => {
    it('返回 RS 地址字符串', () => {
      const rs = 'NRCS-AAAA-AAAA-AAAA-AAAAA'
      expect(getAccountTitle(rs)).toBe(rs)
    })

    it('undefined 输入返回 "-"', () => {
      expect(getAccountTitle(undefined)).toBe('-')
    })

    it('对象输入返回 RS 地址', () => {
      const tx = { sender: '1', senderRS: 'NRCS-AAAA-AAAA-AAAA-AAAAA' }
      expect(getAccountTitle(tx, 'sender')).toBe('NRCS-AAAA-AAAA-AAAA-AAAAA')
    })
  })

  // ==========================================================================
  // getAccountForDecryption
  // ==========================================================================
  describe('getAccountForDecryption', () => {
    it('当前账户是发送方 → 返回接收方账户', () => {
      const result = getAccountForDecryption(
        { sender: '111', recipient: '222' },
        '111',
      )
      expect(result).toBe('222')
    })

    it('当前账户是接收方 → 返回发送方账户', () => {
      const result = getAccountForDecryption(
        { sender: '111', recipient: '222' },
        '222',
      )
      expect(result).toBe('111')
    })

    it('当前账户既非发送方也非接收方 → 返回发送方', () => {
      const result = getAccountForDecryption(
        { sender: '111', recipient: '222' },
        '333',
      )
      expect(result).toBe('111')
    })

    it('发送方缺失时返回 "0"', () => {
      const result = getAccountForDecryption(
        { sender: undefined, recipient: '222' },
        '333',
      )
      expect(result).toBe('0')
    })

    it('当前账户是发送方但接收方缺失 → 回退到发送方', () => {
      const result = getAccountForDecryption(
        { sender: '111', recipient: undefined },
        '111',
      )
      expect(result).toBe('111')
    })
  })

  // ==========================================================================
  // convertNumericToRSAccount
  // ==========================================================================
  describe('convertNumericToRSAccount', () => {
    it('已是 RS 格式则原样返回', () => {
      const rs = 'NRCS-AAAA-AAAA-AAAA-AAAAA'
      expect(convertNumericToRSAccount(rs)).toBe(rs)
    })

    it('数字字符串转换为 RS 地址', () => {
      const rs = convertNumericToRSAccount('123456789')
      expect(rs.startsWith('NRCS-')).toBe(true)
      expect(rs.includes('-')).toBe(true)
    })

    it('数字类型转换为 RS 地址', () => {
      const rs = convertNumericToRSAccount(123456789)
      expect(rs.startsWith('NRCS-')).toBe(true)
    })
  })
})
