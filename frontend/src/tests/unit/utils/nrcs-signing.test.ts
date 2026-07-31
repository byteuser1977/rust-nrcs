/******************************************************************************
 * NRCS 本地交易签名链路单元测试
 *
 * 验证 nrcs-signing.ts 的核心功能：
 *   - verifyTransactionBytes：字节级字段校验
 *   - verifyAndSignTransactionBytes：验证 + 本地签名
 *   - broadcastTransactionBytes：广播（mock）
 *   - getECBlock：EC 区块获取
 *
 * 测试向量来自 nrcs_signature_implementation_plan.md：
 *   Passphrase:  concern entire frozen witch away creak dot drink need season clutch truly
 *   Public Key:  2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c
 *   Signature:   eab9a9fd3d73950a372e17a76a38fb206b875a84f9bc4fa80b18307ea683f204...
 *   Unsigned Tx: 001037b138053c00...
 *
 * 测试向量结构（小端序解析）：
 *   type=0/subtype=0/version=1（sendMoney）
 *   deadline=60, amountNQT=500000000(5 NXT), feeNQT=100000000(1 NXT)
 *   flags=32（bit 5 = 可修剪普通消息）
 *   附件：attachmentVersion=1 + 32字节 SHA256 哈希
 *
 * 注意：测试向量含可修剪普通消息（flags bit 5），但原始消息内容不在本地数据库中，
 * 无法重构 message/messageIsText 字段进行哈希校验。
 * 因此核心签名测试使用 isVerifyOptionalAttachments=false 跳过可选附件校验，
 * 专注于验证：核心字段校验 + 签名生成与参考向量一致性。
 * 另有独立测试验证 isVerifyOptionalAttachments=true（默认）时会正确拒绝。
 ******************************************************************************/

import { describe, it, expect, vi, beforeEach } from 'vitest';

// 测试向量（来自 nrcs_signature_implementation_plan.md）
// 注意：UNSIGNED_TX_BYTES 属于交易 2，其对应签名为 EXPECTED_SIGNATURE（交易 2 的签名）
const TEST_PASSPHRASE = 'concern entire frozen witch away creak dot drink need season clutch truly';
const TEST_PUBLIC_KEY = '2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c';
const EXPECTED_SIGNATURE = 'eab9a9fd3d73950a372e17a76a38fb206b875a84f9bc4fa80b18307ea683f204fcffc4413d187302a84ccdf89d796130a6e9d161afc30a45fc41e4257af4f0c5';
const UNSIGNED_TX_BYTES = '001037b138053c002d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c5fe6cbd7bfb374290065cd1d0000000000e1f505000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000007cb1400e278bbd91da7aae30163fc7d52fb0c3d2cd86b77a1a4f32b233735c3108a68d6a525909d4efe887dde';

import {
  hexStringToByteArray,
  byteArrayToSignedInt32,
  byteArrayToSignedShort,
  byteArrayToBigInteger,
  byteArrayToHexString,
} from '@/utils/converters';

import {
  getECBlock,
  verifyTransactionBytes,
  verifyAndSignTransactionBytes,
  broadcastTransactionBytes,
  signAndBroadcastTransaction,
  type TransactionFormData,
  type VerifyOptions,
} from '@/utils/nrcs-signing';

import { signBytes, verifySignature, getPublicKey } from '@/utils/nrcs-crypto';

// ── 从未签名字节中解析表单数据（用于构造 verifyTransactionBytes 的入参） ────────

/** 从测试向量的未签名交易字节中解析出表单数据 */
function buildFormDataFromBytes(): TransactionFormData {
  const bytes = hexStringToByteArray(UNSIGNED_TX_BYTES);

  // deadline 在 [6, 8)
  const deadline = String(byteArrayToSignedShort(bytes, 6));
  // recipient 在 [40, 48)
  const recipient = String(byteArrayToBigInteger(bytes, 40));
  // amountNQT 在 [48, 56)
  const amountNQT = String(byteArrayToBigInteger(bytes, 48));
  // feeNQT 在 [56, 64)
  const feeNQT = String(byteArrayToBigInteger(bytes, 56));
  // publicKey 在 [8, 40)
  const publicKey = byteArrayToHexString(bytes.slice(8, 40));

  return {
    requestType: 'sendMoney',
    deadline,
    recipient,
    amountNQT,
    feeNQT,
    publicKey,
  };
}

/**
 * 构造校验选项。
 *
 * 使用 isVerifyOptionalAttachments=false 跳过可选附件校验，
 * 因为测试向量含可修剪普通消息（flags bit 5）但缺少原始消息内容，
 * 无法计算 SHA256 哈希进行比对。
 * 此设置仅影响可选附件校验，核心字段校验（type/version/deadline/publicKey/
 * recipient/amountNQT/feeNQT/referencedTransactionFullHash）仍正常执行。
 */
function buildVerifyOptions(): VerifyOptions {
  return {
    accountPublicKey: TEST_PUBLIC_KEY,
    isVerifyECBlock: false,
    isVerifyOptionalAttachments: false,
  };
}

// ============================================================================
// 测试用例
// ============================================================================

describe('nrcs-signing: getECBlock', () => {
  it('主网返回 LAST_KNOWN_BLOCK 静态常量', () => {
    const block = getECBlock(false);
    expect(block.id).toBe('3488276486778630462');
    expect(block.height).toBe('0');
  });

  it('测试网返回 LAST_KNOWN_TESTNET_BLOCK 静态常量', () => {
    const block = getECBlock(true);
    expect(block.id).toBe('3488276486778630462');
    expect(block.height).toBe('0');
  });
});

describe('nrcs-signing: verifyTransactionBytes', () => {
  it('匹配的表单数据 + 跳过可选附件校验时通过', () => {
    // buildVerifyOptions() 使用 isVerifyOptionalAttachments=false，
    // 核心字段（type/version/deadline/publicKey/recipient/amountNQT/feeNQT）均匹配，应通过。
    const bytes = hexStringToByteArray(UNSIGNED_TX_BYTES);
    const data = buildFormDataFromBytes();
    const ok = verifyTransactionBytes(bytes, 'sendMoney', data, {}, buildVerifyOptions());
    expect(ok).toBe(true);
  });

  it('deadline 不匹配时校验失败', () => {
    const bytes = hexStringToByteArray(UNSIGNED_TX_BYTES);
    const data = { ...buildFormDataFromBytes(), deadline: '999' };
    const ok = verifyTransactionBytes(bytes, 'sendMoney', data, {}, buildVerifyOptions());
    expect(ok).toBe(false);
  });

  it('recipient 不匹配时校验失败', () => {
    const bytes = hexStringToByteArray(UNSIGNED_TX_BYTES);
    const data = { ...buildFormDataFromBytes(), recipient: '123456789' };
    const ok = verifyTransactionBytes(bytes, 'sendMoney', data, {}, buildVerifyOptions());
    expect(ok).toBe(false);
  });

  it('amountNQT 不匹配时校验失败', () => {
    const bytes = hexStringToByteArray(UNSIGNED_TX_BYTES);
    const data = { ...buildFormDataFromBytes(), amountNQT: '1' };
    const ok = verifyTransactionBytes(bytes, 'sendMoney', data, {}, buildVerifyOptions());
    expect(ok).toBe(false);
  });

  it('feeNQT 不匹配时不影响校验（参考 nrs.server.js 不校验 feeNQT）', () => {
    // 参考 nrs.server.js verifyTransactionBytes 仅校验 amountNQT，不校验 feeNQT。
    // 原因：手续费由服务端确定（可根据网络状况调整），客户端不严格验证。
    // 因此即使 feeNQT 不匹配，校验仍应通过。
    const bytes = hexStringToByteArray(UNSIGNED_TX_BYTES);
    const data = { ...buildFormDataFromBytes(), feeNQT: '1' };
    const ok = verifyTransactionBytes(bytes, 'sendMoney', data, {}, buildVerifyOptions());
    expect(ok).toBe(true);
  });

  it('publicKey 与账户公钥不匹配时校验失败', () => {
    const bytes = hexStringToByteArray(UNSIGNED_TX_BYTES);
    const data = buildFormDataFromBytes();
    const options: VerifyOptions = {
      accountPublicKey: '0000000000000000000000000000000000000000000000000000000000000000',
      isVerifyECBlock: false,
    };
    const ok = verifyTransactionBytes(bytes, 'sendMoney', data, {}, options);
    expect(ok).toBe(false);
  });

  it('requestType 与 type/subtype 不匹配时校验失败', () => {
    const bytes = hexStringToByteArray(UNSIGNED_TX_BYTES);
    const data = buildFormDataFromBytes();
    // sendMoney 要求 type=0, subtype=0；此处用 sendMessage（type=1）应失败
    const ok = verifyTransactionBytes(bytes, 'sendMessage', data, {}, buildVerifyOptions());
    expect(ok).toBe(false);
  });

  it('recipient 为空且交易 recipient 为 0 时允许通过（genesis 回退）', () => {
    const bytes = hexStringToByteArray(UNSIGNED_TX_BYTES);
    const data = { ...buildFormDataFromBytes(), recipient: '' };
    // 此处交易字节中 recipient 非 0，所以应该失败
    const ok = verifyTransactionBytes(bytes, 'sendMoney', data, {}, buildVerifyOptions());
    expect(ok).toBe(false);
  });

  it('isVerifyOptionalAttachments=true（默认）时因缺少消息内容校验失败', () => {
    // 测试向量 flags=32（bit 5 = 可修剪普通消息），但表单数据未提供 message/messageIsText，
    // SHA256 哈希比对必然失败，verifyTransactionBytes 应正确拒绝。
    const bytes = hexStringToByteArray(UNSIGNED_TX_BYTES);
    const data = buildFormDataFromBytes();
    const options: VerifyOptions = {
      accountPublicKey: TEST_PUBLIC_KEY,
      isVerifyECBlock: false,
      // isVerifyOptionalAttachments 默认 true，与参考 nrs.server.js 行为一致
    };
    const ok = verifyTransactionBytes(bytes, 'sendMoney', data, {}, options);
    expect(ok).toBe(false);
  });
});

describe('nrcs-signing: verifyAndSignTransactionBytes（核心签名测试）', () => {
  it('本地签名结果与预期签名向量一致', () => {
    const data = buildFormDataFromBytes();
    const response = { transactionJSON: { attachment: {} } };

    const result = verifyAndSignTransactionBytes(
      UNSIGNED_TX_BYTES,
      'sendMoney',
      data,
      response,
      TEST_PASSPHRASE,
      buildVerifyOptions(),
    );

    expect(result.ok).toBe(true);
    expect(result.signature).toBe(EXPECTED_SIGNATURE);
  });

  it('签名注入后 payload 的 [96, 160) 字节区间为签名', () => {
    const data = buildFormDataFromBytes();
    const response = { transactionJSON: { attachment: {} } };

    const result = verifyAndSignTransactionBytes(
      UNSIGNED_TX_BYTES,
      'sendMoney',
      data,
      response,
      TEST_PASSPHRASE,
      buildVerifyOptions(),
    );

    expect(result.ok).toBe(true);
    expect(result.payload).toBeDefined();
    // 签名注入位置：字节 [96, 160) 即 hex 字符 [192, 320)
    // verifyAndSignTransactionBytes 通过 substring(0,192) + signature + substring(320) 拼接
    const injectedSignature = result.payload!.substring(192, 320);
    expect(injectedSignature).toBe(EXPECTED_SIGNATURE);
  });

  it('payload 签名前后的字节与未签名字节一致', () => {
    const data = buildFormDataFromBytes();
    const response = { transactionJSON: { attachment: {} } };

    const result = verifyAndSignTransactionBytes(
      UNSIGNED_TX_BYTES,
      'sendMoney',
      data,
      response,
      TEST_PASSPHRASE,
      buildVerifyOptions(),
    );

    expect(result.ok).toBe(true);
    // 签名前的 96 字节（hex [0, 192)）与未签名字节一致
    const prefix = result.payload!.substring(0, 192);
    const originalPrefix = UNSIGNED_TX_BYTES.substring(0, 192);
    expect(prefix).toBe(originalPrefix);
    // 签名后的字节（hex [320, ...)）与未签名字节一致
    const suffix = result.payload!.substring(320);
    const originalSuffix = UNSIGNED_TX_BYTES.substring(320);
    expect(suffix).toBe(originalSuffix);
  });

  it('校验失败时返回 ok=false', () => {
    const data = { ...buildFormDataFromBytes(), deadline: '999' };
    const response = { transactionJSON: { attachment: {} } };

    const result = verifyAndSignTransactionBytes(
      UNSIGNED_TX_BYTES,
      'sendMoney',
      data,
      response,
      TEST_PASSPHRASE,
      buildVerifyOptions(),
    );

    expect(result.ok).toBe(false);
    expect(result.errorCode).toBe(1);
    expect(result.errorDescription).toBeDefined();
  });

  it('签名可通过 verifySignature 验证', () => {
    const data = buildFormDataFromBytes();
    const response = { transactionJSON: { attachment: {} } };

    const result = verifyAndSignTransactionBytes(
      UNSIGNED_TX_BYTES,
      'sendMoney',
      data,
      response,
      TEST_PASSPHRASE,
      buildVerifyOptions(),
    );

    expect(result.ok).toBe(true);
    // 用 verifySignature 验证签名
    const valid = verifySignature(result.signature!, UNSIGNED_TX_BYTES, TEST_PUBLIC_KEY);
    expect(valid).toBe(true);
  });
});

describe('nrcs-signing: 签名原语一致性', () => {
  it('getPublicKey(passphrase) 与预期公钥一致', () => {
    const pubKey = getPublicKey(TEST_PASSPHRASE);
    expect(pubKey).toBe(TEST_PUBLIC_KEY);
  });

  it('signBytes(unsignedBytes, passphrase) 与预期签名一致', () => {
    const signature = signBytes(UNSIGNED_TX_BYTES, TEST_PASSPHRASE);
    expect(signature).toBe(EXPECTED_SIGNATURE);
  });

  it('verifySignature(expectedSignature, unsignedBytes, publicKey) 通过', () => {
    const valid = verifySignature(EXPECTED_SIGNATURE, UNSIGNED_TX_BYTES, TEST_PUBLIC_KEY);
    expect(valid).toBe(true);
  });
});

describe('nrcs-signing: broadcastTransactionBytes', () => {
  beforeEach(() => {
    vi.resetModules();
  });

  it('广播成功时返回 transaction 和 fullHash', async () => {
    // mock nrcsPost 返回成功
    vi.doMock('@/api/nrcs-client', () => ({
      nrcsPost: vi.fn().mockResolvedValue({
        transaction: '1234567890',
        fullHash: 'abcdef0123456789',
      }),
      nrcsGet: vi.fn(),
    }));

    const { broadcastTransactionBytes } = await import('@/utils/nrcs-signing');
    // 构造已签名 payload：签名注入在字节 [96, 160) 即 hex [192, 320)
    const payload = UNSIGNED_TX_BYTES.substring(0, 192) + EXPECTED_SIGNATURE + UNSIGNED_TX_BYTES.substring(320);
    const response = { transactionJSON: { attachment: {} } };

    const result = await broadcastTransactionBytes(payload, response, {}, {});

    expect(result.broadcasted).toBe(true);
    expect(result.transaction).toBe('1234567890');
    expect(result.fullHash).toBe('abcdef0123456789');
  });

  it('服务端返回 errorCode 时视为失败', async () => {
    vi.doMock('@/api/nrcs-client', () => ({
      nrcsPost: vi.fn().mockResolvedValue({
        errorCode: 5,
        errorDescription: 'invalid signature',
      }),
      nrcsGet: vi.fn(),
    }));

    const { broadcastTransactionBytes } = await import('@/utils/nrcs-signing');
    const result = await broadcastTransactionBytes('00', { transactionJSON: { attachment: {} } }, {}, {});

    expect(result.broadcasted).toBe(false);
    expect(result.errorCode).toBe(5);
    expect(result.errorDescription).toBe('invalid signature');
  });

  it('网络异常时返回 errorCode=-1', async () => {
    vi.doMock('@/api/nrcs-client', () => ({
      nrcsPost: vi.fn().mockRejectedValue(new Error('network timeout')),
      nrcsGet: vi.fn(),
    }));

    const { broadcastTransactionBytes } = await import('@/utils/nrcs-signing');
    const result = await broadcastTransactionBytes('00', { transactionJSON: { attachment: {} } }, {}, {});

    expect(result.broadcasted).toBe(false);
    expect(result.errorCode).toBe(-1);
    expect(result.errorDescription).toBe('network timeout');
  });
});

describe('nrcs-signing: signAndBroadcastTransaction（三步流程编排）', () => {
  beforeEach(() => {
    vi.resetModules();
  });

  it('完整签名+广播流程成功', async () => {
    vi.doMock('@/api/nrcs-client', () => ({
      nrcsPost: vi.fn().mockResolvedValue({
        transaction: '9876543210',
        fullHash: 'fedcba9876543210',
      }),
      nrcsGet: vi.fn(),
    }));

    const { signAndBroadcastTransaction } = await import('@/utils/nrcs-signing');
    const data = buildFormDataFromBytes();
    const response = { transactionJSON: { attachment: {} } };

    const result = await signAndBroadcastTransaction(
      UNSIGNED_TX_BYTES,
      'sendMoney',
      data,
      response,
      TEST_PASSPHRASE,
      { ...buildVerifyOptions(), broadcast: true },
    );

    expect(result.broadcasted).toBe(true);
    expect(result.transaction).toBe('9876543210');
    expect(result.fullHash).toBe('fedcba9876543210');
    expect(result.signature).toBe(EXPECTED_SIGNATURE);
  });

  it('broadcast=false 时不广播，仅返回签名', async () => {
    vi.doMock('@/api/nrcs-client', () => ({
      nrcsPost: vi.fn(),
      nrcsGet: vi.fn(),
    }));

    const { signAndBroadcastTransaction } = await import('@/utils/nrcs-signing');
    const data = buildFormDataFromBytes();
    const response = { transactionJSON: { attachment: {} } };

    const result = await signAndBroadcastTransaction(
      UNSIGNED_TX_BYTES,
      'sendMoney',
      data,
      response,
      TEST_PASSPHRASE,
      { ...buildVerifyOptions(), broadcast: false },
    );

    expect(result.broadcasted).toBe(false);
    expect(result.signature).toBe(EXPECTED_SIGNATURE);
    expect(result.payload).toBeDefined();
  });

  it('校验失败时不进行签名和广播', async () => {
    vi.doMock('@/api/nrcs-client', () => ({
      nrcsPost: vi.fn(),
      nrcsGet: vi.fn(),
    }));

    const { signAndBroadcastTransaction } = await import('@/utils/nrcs-signing');
    const data = { ...buildFormDataFromBytes(), deadline: '999' };
    const response = { transactionJSON: { attachment: {} } };

    const result = await signAndBroadcastTransaction(
      UNSIGNED_TX_BYTES,
      'sendMoney',
      data,
      response,
      TEST_PASSPHRASE,
      { ...buildVerifyOptions(), broadcast: true },
    );

    expect(result.broadcasted).toBe(false);
    expect(result.errorCode).toBe(1);
    expect(result.signature).toBeUndefined();
  });
});

describe('nrcs-signing: 字节解析一致性', () => {
  it('测试向量的 type=0, subtype=0, version=1（sendMoney）', () => {
    const bytes = hexStringToByteArray(UNSIGNED_TX_BYTES);
    expect(bytes[0]).toBe(0); // type
    expect((bytes[1] & 0xf0) >> 4).toBe(1); // version
    expect(bytes[1] & 0x0f).toBe(0); // subtype
  });

  it('测试向量的 deadline=60', () => {
    const bytes = hexStringToByteArray(UNSIGNED_TX_BYTES);
    const deadline = byteArrayToSignedShort(bytes, 6);
    expect(deadline).toBe(60);
  });

  it('测试向量的 publicKey 与预期一致', () => {
    const bytes = hexStringToByteArray(UNSIGNED_TX_BYTES);
    const pubKey = byteArrayToHexString(bytes.slice(8, 40));
    expect(pubKey).toBe(TEST_PUBLIC_KEY);
  });
});
