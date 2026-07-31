/******************************************************************************
 * 交易消息解析工具单元测试
 *
 * 验证 transaction-message.ts 的核心功能：
 *   - isTextMessage：消息类型判断
 *   - parsePublicMessage：公开消息解析（legacy hex / 新版本文本 / 二进制）
 *   - getDecryptionFields：待解密字段识别
 *   - getMessageHash：消息哈希获取
 *
 * 对标参考：nrs.messages.js / nrs.modals.transaction.js:256-321
 ******************************************************************************/
import { describe, it, expect } from 'vitest'
import {
  isTextMessage,
  parsePublicMessage,
  getDecryptionFields,
  getMessageHash,
} from '@/utils/transaction-message'
import type { NrcsTransaction } from '@/api/modules/nrcs.api'

// 测试辅助：构造交易对象
function makeTx(overrides: Partial<NrcsTransaction> = {}): NrcsTransaction {
  return {
    transaction: '123',
    type: 1,
    subtype: 0,
    sender: '111',
    senderRS: 'NRCS-AAAA-AAAA-AAAA-AAAAA',
    recipient: '222',
    recipientRS: 'NRCS-BBBB-BBBB-BBBB-BBBBB',
    amountNQT: '0',
    feeNQT: '1000000',
    timestamp: 1000,
    height: 100,
    confirmations: 10,
    attachment: {},
    ...overrides,
  }
}

describe('transaction-message', () => {
  // ==========================================================================
  // isTextMessage
  // ==========================================================================
  describe('isTextMessage', () => {
    it('无 attachment 返回 false', () => {
      expect(isTextMessage(makeTx({ attachment: undefined }))).toBe(false)
    })

    it('messageIsText=true 返回 true', () => {
      expect(
        isTextMessage(
          makeTx({ attachment: { messageIsText: true, message: 'hello' } }),
        ),
      ).toBe(true)
    })

    it('messageIsText=false 返回 false', () => {
      expect(
        isTextMessage(
          makeTx({ attachment: { messageIsText: false, message: 'abcd' } }),
        ),
      ).toBe(false)
    })

    it('prunable 文本消息（含 version.PrunablePlainMessage）返回 true', () => {
      expect(
        isTextMessage(
          makeTx({
            attachment: {
              messageIsText: true,
              messageIsPrunable: true,
              'version.PrunablePlainMessage': 1,
              message: 'hello',
            },
          }),
        ),
      ).toBe(true)
    })

    it('prunable 二进制消息返回 false', () => {
      expect(
        isTextMessage(
          makeTx({
            attachment: {
              messageIsText: false,
              messageIsPrunable: true,
              'version.PrunablePlainMessage': 1,
              message: 'abcd',
            },
          }),
        ),
      ).toBe(false)
    })

    it('无 messageIsText 标记默认返回 false', () => {
      expect(
        isTextMessage(makeTx({ attachment: { message: 'hello' } })),
      ).toBe(false)
    })
  })

  // ==========================================================================
  // parsePublicMessage
  // ==========================================================================
  describe('parsePublicMessage', () => {
    it('undefined 附件返回 null', () => {
      expect(parsePublicMessage(undefined)).toBeNull()
    })

    it('无 message 字段返回 null', () => {
      expect(parsePublicMessage({})).toBeNull()
    })

    it('新版本文本消息直接返回字符串', () => {
      const result = parsePublicMessage({
        message: 'Hello World',
        messageIsText: true,
        'version.Message': 1,
      })
      expect(result).toEqual({
        text: 'Hello World',
        isBinary: false,
        isLegacyUtf16: false,
      })
    })

    it('新版本二进制消息返回 "binary_data" 占位', () => {
      const result = parsePublicMessage({
        message: 'aabbcc',
        messageIsText: false,
        'version.Message': 1,
      })
      expect(result).toEqual({
        text: 'binary_data',
        isBinary: true,
        isLegacyUtf16: false,
      })
    })

    it('legacy hex 消息尝试 hexStringToString 解码', () => {
      // "Hello" 的 UTF-8 hex 编码
      const hex = '48656c6c6f'
      const result = parsePublicMessage({
        message: hex,
      })
      expect(result).not.toBeNull()
      expect(result?.text).toBe('Hello')
      expect(result?.isBinary).toBe(false)
    })

    it('legacy 消息使用 hexStringToString 解码（不区分 UTF-8/UTF-16，回退到 UTF-8 路径）', () => {
      // hexStringToString 使用 TextDecoder 非致命模式，不会抛出异常
      // 因此 UTF-16 BOM 检测路径在实践中不易触发
      // 这里仅验证 legacy 路径返回结果非空
      const hex = 'feff00480069'
      const result = parsePublicMessage({
        message: hex,
      })
      expect(result).not.toBeNull()
      expect(result?.isBinary).toBe(false)
    })
  })

  // ==========================================================================
  // getDecryptionFields
  // ==========================================================================
  describe('getDecryptionFields', () => {
    it('无 attachment 返回空列表', () => {
      expect(getDecryptionFields(makeTx({ attachment: undefined }), '111')).toEqual([])
    })

    it('有 encryptedMessage 字段添加到列表', () => {
      const tx = makeTx({
        attachment: {
          encryptedMessage: { data: 'abc', nonce: 'def' },
          'version.EncryptedMessage': 1,
        },
      })
      const fields = getDecryptionFields(tx, '222')
      expect(fields).toHaveLength(1)
      expect(fields[0]).toEqual({
        fieldName: 'encryptedMessage',
        label: 'encrypted_message',
      })
    })

    it('有 encryptToSelfMessage 且当前账户是发送方添加到列表', () => {
      const tx = makeTx({
        sender: '111',
        attachment: {
          encryptToSelfMessage: { data: 'abc', nonce: 'def' },
          'version.EncryptToSelfMessage': 1,
        },
      })
      const fields = getDecryptionFields(tx, '111')
      expect(fields).toHaveLength(1)
      expect(fields[0]).toEqual({
        fieldName: 'encryptToSelfMessage',
        label: 'note_to_self',
      })
    })

    it('有 encryptToSelfMessage 但当前账户不是发送方不添加', () => {
      const tx = makeTx({
        sender: '111',
        attachment: {
          encryptToSelfMessage: { data: 'abc', nonce: 'def' },
        },
      })
      const fields = getDecryptionFields(tx, '222')
      expect(fields).toHaveLength(0)
    })

    it('同时有 encryptedMessage 和 encryptToSelfMessage（自己是发送方）返回两个字段', () => {
      const tx = makeTx({
        sender: '111',
        attachment: {
          encryptedMessage: { data: 'abc', nonce: 'def' },
          encryptToSelfMessage: { data: 'xyz', nonce: '123' },
        },
      })
      const fields = getDecryptionFields(tx, '111')
      expect(fields).toHaveLength(2)
      expect(fields[0].fieldName).toBe('encryptedMessage')
      expect(fields[1].fieldName).toBe('encryptToSelfMessage')
    })

    it('无加密消息字段返回空列表', () => {
      const tx = makeTx({
        attachment: { message: 'hello', messageIsText: true },
      })
      expect(getDecryptionFields(tx, '111')).toEqual([])
    })
  })

  // ==========================================================================
  // getMessageHash
  // ==========================================================================
  describe('getMessageHash', () => {
    it('undefined 附件返回 undefined', () => {
      expect(getMessageHash(undefined)).toBeUndefined()
    })

    it('优先返回 messageHash', () => {
      const hash = getMessageHash({
        messageHash: 'aaa',
        encryptedMessageHash: 'bbb',
      })
      expect(hash).toBe('aaa')
    })

    it('无 messageHash 时回退到 encryptedMessageHash', () => {
      const hash = getMessageHash({
        encryptedMessageHash: 'bbb',
      })
      expect(hash).toBe('bbb')
    })

    it('两者都没有返回 undefined', () => {
      const hash = getMessageHash({ message: 'hello' })
      expect(hash).toBeUndefined()
    })
  })
})
