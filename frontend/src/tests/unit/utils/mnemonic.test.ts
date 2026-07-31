/**
 * mnemonic.ts 单元测试
 *
 * 验证助记词生成、校验、掩码、强度检测、账户派生与已知测试向量一致性。
 */

import { describe, it, expect } from 'vitest';
import {
  generateMnemonic,
  validateMnemonic,
  is12WordsSecret,
  maskMnemonicForConfirmation,
  verifyPassphraseMatch,
  checkPassphraseStrength,
  passphraseToAccount,
  formatMnemonicForDisplay,
} from '@/utils/mnemonic';
import { NRCS_WORDS, WORD_COUNT } from '@/utils/mnemonic-words';
import { getPublicKey, getAccountId } from '@/utils/nrcs-crypto';

// 已知测试向量（与 nrcs-signing.test.ts 一致，后端已验证）
const TEST_PASSPHRASE = 'concern entire frozen witch away creak dot drink need season clutch truly';
const TEST_PUBLIC_KEY = '2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c';

// 后端测试向量（crates/crypto/src/passphrase/mod.rs tests）
const BACKEND_TEST_1 = 'like just love know never want time out there make look eye';
const BACKEND_TEST_2 = 'confusion flirt teeth story crawl dear shove screw decay flood cover warrior';

describe('mnemonic-words 词表', () => {
  it('词表长度与后端 WORD_COUNT 一致（1626）', () => {
    expect(WORD_COUNT).toBe(1626);
    expect(NRCS_WORDS.length).toBe(1626);
  });

  it('词表无重复', () => {
    const set = new Set(NRCS_WORDS);
    expect(set.size).toBe(NRCS_WORDS.length);
  });

  it('首词为 like，末词为 weary（与后端/参考一致）', () => {
    expect(NRCS_WORDS[0]).toBe('like');
    expect(NRCS_WORDS[1625]).toBe('weary');
  });
});

describe('generateMnemonic 生成助记词', () => {
  it('生成正好 12 个词', () => {
    const result = generateMnemonic();
    expect(result.wordCount).toBe(12);
    expect(result.words.length).toBe(12);
    expect(result.passphrase.split(' ').length).toBe(12);
  });

  it('所有词都在词表中', () => {
    const result = generateMnemonic();
    for (const word of result.words) {
      expect(NRCS_WORDS).toContain(word);
    }
  });

  it('两次生成结果不同（随机性）', () => {
    const a = generateMnemonic();
    const b = generateMnemonic();
    expect(a.passphrase).not.toBe(b.passphrase);
  });

  it('生成的助记词能通过 validateMnemonic', () => {
    const result = generateMnemonic();
    const validation = validateMnemonic(result.passphrase);
    expect(validation.valid).toBe(true);
    expect(validation.errors).toHaveLength(0);
  });
});

describe('validateMnemonic 校验助记词', () => {
  it('接受已知测试向量', () => {
    expect(validateMnemonic(TEST_PASSPHRASE).valid).toBe(true);
    expect(validateMnemonic(BACKEND_TEST_1).valid).toBe(true);
    expect(validateMnemonic(BACKEND_TEST_2).valid).toBe(true);
  });

  it('拒绝非 12 词', () => {
    const r1 = validateMnemonic('like just love');
    expect(r1.valid).toBe(false);
    expect(r1.wordCount).toBe(3);

    const r2 = validateMnemonic('like just love know never want time out there make look eye extra');
    expect(r2.valid).toBe(false);
    expect(r2.wordCount).toBe(13);
  });

  it('拒绝不在词表中的词', () => {
    const r = validateMnemonic('like just love know never want time out there make look invalidword');
    expect(r.valid).toBe(false);
    expect(r.errors.some((e) => e.includes('invalidword'))).toBe(true);
  });

  it('大小写不敏感（词表全小写）', () => {
    expect(validateMnemonic('Like Just Love Know Never Want Time Out There Make Look Eye').valid).toBe(true);
  });

  it('拒绝空字符串', () => {
    const r = validateMnemonic('');
    expect(r.valid).toBe(false);
    expect(r.wordCount).toBe(0);
  });

  it('拒绝 null/undefined', () => {
    expect(validateMnemonic(null as unknown as string).valid).toBe(false);
    expect(validateMnemonic(undefined as unknown as string).valid).toBe(false);
  });
});

describe('is12WordsSecret', () => {
  it('有效 12 词返回 true', () => {
    expect(is12WordsSecret(TEST_PASSPHRASE.split(' '))).toBe(true);
  });

  it('非 12 词返回 false', () => {
    expect(is12WordsSecret(['like', 'just', 'love'])).toBe(false);
  });

  it('含无效词返回 false', () => {
    const words = TEST_PASSPHRASE.split(' ');
    words[0] = 'invalidword';
    expect(is12WordsSecret(words)).toBe(false);
  });
});

describe('maskMnemonicForConfirmation 掩码确认', () => {
  it('隐藏指定数量的词（默认 6 显 6 隐）', () => {
    const result = maskMnemonicForConfirmation(TEST_PASSPHRASE);
    expect(result.masked).toHaveLength(12);
    expect(result.hiddenIndices).toHaveLength(6);
    expect(result.revealedIndices).toHaveLength(6);
  });

  it('隐藏位置为占位符 _____', () => {
    const result = maskMnemonicForConfirmation(TEST_PASSPHRASE, 6);
    const hiddenSet = new Set(result.hiddenIndices);
    for (const idx of result.hiddenIndices) {
      expect(result.masked[idx]).toBe('_____');
    }
    for (let i = 0; i < 12; i++) {
      if (!hiddenSet.has(i)) {
        expect(result.masked[i]).not.toBe('_____');
      }
    }
  });

  it('revealedIndices 与 hiddenIndices 互补', () => {
    const result = maskMnemonicForConfirmation(TEST_PASSPHRASE, 4);
    expect(result.revealedIndices).toHaveLength(4);
    expect(result.hiddenIndices).toHaveLength(8);
    const all = [...result.revealedIndices, ...result.hiddenIndices].sort((a, b) => a - b);
    expect(all).toEqual([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
  });

  it('保留原始助记词', () => {
    const result = maskMnemonicForConfirmation(TEST_PASSPHRASE);
    expect(result.fullMnemonic).toBe(TEST_PASSPHRASE);
  });
});

describe('verifyPassphraseMatch 验证匹配', () => {
  it('完全匹配返回 true', () => {
    expect(verifyPassphraseMatch(TEST_PASSPHRASE, TEST_PASSPHRASE)).toBe(true);
  });

  it('忽略首尾空格与多余空格', () => {
    expect(verifyPassphraseMatch(`  ${TEST_PASSPHRASE}  `, TEST_PASSPHRASE)).toBe(true);
    expect(verifyPassphraseMatch(TEST_PASSPHRASE.split(' ').join('  '), TEST_PASSPHRASE)).toBe(true);
  });

  it('大小写不敏感', () => {
    expect(verifyPassphraseMatch(TEST_PASSPHRASE.toUpperCase(), TEST_PASSPHRASE)).toBe(true);
  });

  it('不匹配返回 false', () => {
    expect(verifyPassphraseMatch(BACKEND_TEST_1, TEST_PASSPHRASE)).toBe(false);
    expect(verifyPassphraseMatch('wrong passphrase', TEST_PASSPHRASE)).toBe(false);
  });
});

describe('checkPassphraseStrength 密码强度', () => {
  it('12 词助记词强度为 strong', () => {
    const s = checkPassphraseStrength(TEST_PASSPHRASE);
    expect(s.score).toBeGreaterThanOrEqual(70);
    expect(s.level).toBe('strong');
  });

  it('空字符串为 very_weak', () => {
    const s = checkPassphraseStrength('');
    expect(s.level).toBe('very_weak');
    expect(s.score).toBe(0);
  });

  it('短密码为 weak/very_weak', () => {
    const s = checkPassphraseStrength('abc');
    expect(s.level).toBe('very_weak');
  });

  it('长度 >= 35 得分高于 < 35', () => {
    const long = checkPassphraseStrength(TEST_PASSPHRASE);
    const short = checkPassphraseStrength('shortpass');
    expect(long.score).toBeGreaterThan(short.score);
  });
});

describe('passphraseToAccount 账户派生', () => {
  it('公钥与已知测试向量一致', () => {
    const account = passphraseToAccount(TEST_PASSPHRASE);
    expect(account.publicKey).toBe(TEST_PUBLIC_KEY);
  });

  it('公钥与 nrcs-crypto.getPublicKey 一致', () => {
    const account = passphraseToAccount(TEST_PASSPHRASE);
    expect(account.publicKey).toBe(getPublicKey(TEST_PASSPHRASE));
  });

  it('accountId 与 nrcs-crypto.getAccountId 一致', () => {
    const account = passphraseToAccount(TEST_PASSPHRASE);
    expect(account.accountId).toBe(getAccountId(TEST_PASSPHRASE));
  });

  it('accountRS 以 NRCS- 前缀且为 20 字符格式', () => {
    const account = passphraseToAccount(TEST_PASSPHRASE);
    expect(account.accountRS).toMatch(/^NRCS-[A-Z0-9]{4}-[A-Z0-9]{4}-[A-Z0-9]{4}-[A-Z0-9]{5}$/);
  });

  it('accountId 为数字字符串', () => {
    const account = passphraseToAccount(TEST_PASSPHRASE);
    expect(/^\d+$/.test(account.accountId)).toBe(true);
  });

  it('不同 passphrase 派生不同账户', () => {
    const a = passphraseToAccount(TEST_PASSPHRASE);
    const b = passphraseToAccount(BACKEND_TEST_1);
    expect(a.accountRS).not.toBe(b.accountRS);
    expect(a.publicKey).not.toBe(b.publicKey);
  });
});

describe('formatMnemonicForDisplay 格式化显示', () => {
  it('每 4 词一行，共 3 行', () => {
    const formatted = formatMnemonicForDisplay(TEST_PASSPHRASE);
    const lines = formatted.split('\n');
    expect(lines).toHaveLength(3);
    for (const line of lines) {
      expect(line.split('  ')).toHaveLength(4);
    }
  });
});

describe('往返测试', () => {
  it('生成 → 校验 → 派生账户链路完整', () => {
    const generated = generateMnemonic();
    const validation = validateMnemonic(generated.passphrase);
    expect(validation.valid).toBe(true);

    const account = passphraseToAccount(generated.passphrase);
    expect(account.publicKey).toHaveLength(64);
    expect(account.accountRS).toMatch(/^NRCS-/);
  });
});
