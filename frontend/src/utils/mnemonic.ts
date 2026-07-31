/**
 * NRCS 助记词（Passphrase）工具
 *
 * 完整移植自 NRCS 参考实现：
 *   - js/crypto/passphrasegenerator.js（NRS 官方前端，1626 词表 + 生成算法）
 *   - crates/crypto/src/passphrase/mod.rs（后端 Rust，WORD_COUNT=1626）
 *   - nrs.login.js（密码强度规则、注册验证流程）
 *
 * 安全模型：
 *   - 助记词是账户的唯一凭证，生成在客户端本地，绝不外发
 *   - 账户派生（accountId/publicKey）复用 nrcs-crypto.ts 的本地算法，
 *     passphrase → SHA256 → Curve25519 keygen，与字节序无关
 *
 * 算法说明（与 passphrasegenerator.js 一致）：
 *   1. crypto.getRandomValues 生成 128 位随机数（4 个 Uint32）
 *   2. 每个随机值 x 生成 3 个词：w1 = x%n, w2 = ((x/n)+w1)%n, w3 = (((x/n)/n)+w2)%n
 *   3. 共 4 个随机值 → 12 个词
 */

import { NRCS_WORDS, WORD_COUNT } from './mnemonic-words';
import { getAccountId, getPublicKey } from './nrcs-crypto';
import { convertNumericToRSAccountFormat } from './converters';

/** 助记词生成结果 */
export interface GeneratedMnemonic {
  /** 12 词空格连接的助记词字符串 */
  passphrase: string;
  /** 12 个词的数组 */
  words: string[];
  /** 词数（恒为 12） */
  wordCount: number;
}

/** 助记词验证结果 */
export interface MnemonicValidation {
  valid: boolean;
  errors: string[];
  wordCount: number;
}

/** 掩码确认结果 */
export interface MaskedMnemonic {
  /** 掩码后的词数组（隐藏位置为占位符） */
  masked: string[];
  /** 需要用户填写的词索引列表 */
  hiddenIndices: number[];
  /** 已展示的词索引列表 */
  revealedIndices: number[];
  /** 原始助记词 */
  fullMnemonic: string;
}

/** 密码强度评估 */
export interface PassphraseStrength {
  /** 0-100 分数 */
  score: number;
  /** 强度等级 */
  level: 'very_weak' | 'weak' | 'medium' | 'strong';
  /** 改进建议 */
  suggestions: string[];
}

/** 助记词派生的账户信息 */
export interface MnemonicAccount {
  /** 数字账户 ID */
  accountId: string;
  /** RS 地址（NRCS-XXXX-XXXX-XXXX-XXXXX） */
  accountRS: string;
  /** 十六进制公钥（64 字符） */
  publicKey: string;
}

/** 词表长度（与后端 WORD_COUNT 一致，n = 1626） */
const N = WORD_COUNT;

/** 预构建词表索引集合，用于 O(1) 校验单词是否合法 */
const WORD_SET: ReadonlySet<string> = new Set(NRCS_WORDS);

/**
 * 生成新的 12 词助记词
 *
 * 对标 passphrasegenerator.js.generatePassPhrase 与后端 generate_passphrase。
 * 使用 Web Crypto API 的 crypto.getRandomValues 采集 128 位真随机数，
 * 然后按 x%n / ((x/n)+w1)%n / (((x/n)/n)+w2)%n 算法映射为 12 个词。
 *
 * @returns 包含 passphrase、words、wordCount 的生成结果
 * @throws 当浏览器不支持 crypto.getRandomValues 时抛出
 */
export function generateMnemonic(): GeneratedMnemonic {
  const cryptoObj =
    typeof globalThis !== 'undefined' && globalThis.crypto
      ? globalThis.crypto
      : typeof window !== 'undefined'
        ? window.crypto || (window as unknown as { msCrypto: Crypto }).msCrypto
        : undefined;

  if (!cryptoObj || typeof cryptoObj.getRandomValues !== 'function') {
    throw new Error('Browser does not support secure random number generation');
  }

  // 128 位随机数 = 4 个 Uint32（与参考 bits=128 一致）
  const random = new Uint32Array(4);
  cryptoObj.getRandomValues(random);

  const words: string[] = [];
  for (let i = 0; i < random.length; i++) {
    const x = random[i];
    // 参考 passphrasegenerator.js：x 为无符号 32 位整数
    // (x / n) >> 0 等价于 Math.floor(x / n) 对正数成立（x 始终非负）
    const w1 = x % N;
    const w2 = (((x / N) >> 0) + w1) % N;
    const w3 = (((((x / N) >> 0) / N) >> 0) + w2) % N;

    words.push(NRCS_WORDS[w1]);
    words.push(NRCS_WORDS[w2]);
    words.push(NRCS_WORDS[w3]);
  }

  // 再次采集随机数覆盖 random 数组，降低内存残留熵泄露风险
  cryptoObj.getRandomValues(random);

  return {
    passphrase: words.join(' '),
    words,
    wordCount: words.length,
  };
}

/**
 * 校验助记词是否有效
 *
 * 对标后端 validate_passphrase：必须正好 12 词，且每个词都在词表中（小写）。
 *
 * @param mnemonic 助记词字符串（空格分隔）
 * @returns 验证结果，含 valid、errors、wordCount
 */
export function validateMnemonic(mnemonic: string): MnemonicValidation {
  if (!mnemonic || typeof mnemonic !== 'string') {
    return { valid: false, errors: ['Mnemonic is required'], wordCount: 0 };
  }

  const words = mnemonic.trim().split(/\s+/).filter((w) => w.length > 0);
  const errors: string[] = [];

  if (words.length !== 12) {
    errors.push(`Must contain exactly 12 words, got ${words.length}`);
  }

  // 检查每个词是否在词表中（词表全小写，输入需归一化为小写）
  for (let i = 0; i < words.length; i++) {
    const word = words[i].toLowerCase();
    if (!WORD_SET.has(word)) {
      errors.push(`Invalid word at position ${i + 1}: "${words[i]}"`);
    }
  }

  // 重复词检测（技术允许但安全性低，作为警告）
  const uniqueWords = new Set(words.map((w) => w.toLowerCase()));
  if (uniqueWords.size < words.length) {
    errors.push('Warning: Duplicate words detected (reduces security)');
  }

  // 警告类（以 "Warning:" 开头）不计入 valid 判定
  const hardErrors = errors.filter((e) => !e.startsWith('Warning'));

  return {
    valid: hardErrors.length === 0,
    errors,
    wordCount: words.length,
  };
}

/**
 * 判断词数组是否为有效的 12 词助记词
 *
 * 对标后端 is_12_words_secret：长度 12 且全部在词表中。
 *
 * @param words 词数组
 * @returns 是否有效
 */
export function is12WordsSecret(words: string[]): boolean {
  return (
    words.length === 12 &&
    words.every((w) => WORD_SET.has(w.toLowerCase()))
  );
}

/**
 * 掩码助记词用于确认步骤
 *
 * 随机隐藏部分词，让用户填写以确认已备份。对标 mnemonic.js.maskMnemonicForConfirmation。
 *
 * @param mnemonic 完整助记词
 * @param revealCount 要展示的词数量（默认 6，即 6 显 6 隐）
 * @returns 掩码结果，含 masked 数组、hiddenIndices、revealedIndices
 */
export function maskMnemonicForConfirmation(
  mnemonic: string,
  revealCount = 6,
): MaskedMnemonic {
  const words = mnemonic.trim().split(/\s+/).filter((w) => w.length > 0);
  const total = words.length;

  // Fisher-Yates 随机选择 revealCount 个位置展示
  const indices = Array.from({ length: total }, (_, i) => i);
  for (let i = indices.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [indices[i], indices[j]] = [indices[j], indices[i]];
  }
  const revealedIndices = indices.slice(0, revealCount).sort((a, b) => a - b);
  const revealedSet = new Set(revealedIndices);

  const masked = words.map((word, index) =>
    revealedSet.has(index) ? word : '_____',
  );
  const hiddenIndices = words
    .map((_, index) => index)
    .filter((index) => !revealedSet.has(index));

  return {
    masked,
    hiddenIndices,
    revealedIndices,
    fullMnemonic: mnemonic,
  };
}

/**
 * 校验用户输入的助记词是否与原始助记词匹配
 *
 * 对标 nrs.login.js.verifyGeneratedPassphrase：注册第 3 步用户完整输入助记词，
 * 与生成时记录的 PassPhraseGenerator.passPhrase 比对。
 *
 * @param input 用户输入的助记词
 * @param original 原始助记词
 * @returns 是否匹配（忽略首尾空格与多余空格）
 */
export function verifyPassphraseMatch(
  input: string,
  original: string,
): boolean {
  const normalize = (s: string) =>
    s
      .trim()
      .split(/\s+/)
      .filter((w) => w.length > 0)
      .map((w) => w.toLowerCase())
      .join(' ');
  return normalize(input) === normalize(original);
}

/**
 * 检查密码短语强度
 *
 * 对标 nrs.login.js 的强度规则：
 *   - 长度 < 35：不安全（error_passphrase_length）
 *   - 长度 < 50 且不含大写字母或数字：不安全（error_passphrase_strength）
 *   - 助记词（12 词）天然满足长度要求
 *
 * @param passphrase 密码短语或助记词
 * @returns 强度评估，含 score、level、suggestions
 */
export function checkPassphraseStrength(passphrase: string): PassphraseStrength {
  if (!passphrase || passphrase.length === 0) {
    return {
      score: 0,
      level: 'very_weak',
      suggestions: ['Passphrase is required'],
    };
  }

  let score = 0;
  const suggestions: string[] = [];

  // 长度检查（对标 nrs.login.js 的 35/50 阈值）
  if (passphrase.length >= 35) {
    score += 40;
  } else if (passphrase.length >= 20) {
    score += 25;
    suggestions.push('Consider using a longer passphrase for better security');
  } else if (passphrase.length >= 12) {
    score += 15;
    suggestions.push('Passphrase should be at least 35 characters for optimal security');
  } else if (passphrase.length >= 8) {
    score += 5;
    suggestions.push('Password too short (minimum 12 characters recommended)');
  } else {
    suggestions.push('Password is very short and weak');
  }

  // 复杂性检查
  const hasUpperCase = /[A-Z]/.test(passphrase);
  const hasLowerCase = /[a-z]/.test(passphrase);
  const hasNumbers = /[0-9]/.test(passphrase);
  const hasSpecialChars = /[^a-zA-Z0-9\s]/.test(passphrase);

  if (hasUpperCase && hasLowerCase) score += 10;
  if (hasNumbers) score += 10;
  if (hasSpecialChars) score += 10;

  // 多词检查（助记词加分）。
  // NRCS 标准 12 词助记词具有 log2(1626^12) ≈ 128 位熵，属银行级强度，
  // 即使全小写无数字也应判为 strong。此处按词数分级加分。
  const wordCount = passphrase.split(/\s+/).filter((w) => w.length > 0).length;
  if (wordCount >= 12) {
    score += 35; // 标准 12 词助记词
  } else if (wordCount >= 8) {
    score += 20;
  } else if (wordCount >= 4) {
    score += 10;
  }

  let level: PassphraseStrength['level'];
  if (score >= 70) {
    level = 'strong';
  } else if (score >= 45) {
    level = 'medium';
  } else if (score >= 20) {
    level = 'weak';
  } else {
    level = 'very_weak';
  }

  return { score: Math.min(score, 100), level, suggestions };
}

/**
 * 从助记词派生账户信息
 *
 * 复用 nrcs-crypto.ts 的本地派生链路（secretPhrase 不出客户端）：
 *   passphrase → getPublicKey (SHA256 + Curve25519 keygen)
 *             → getAccountId (SHA256(publicKey)[0..8])
 *             → convertNumericToRSAccountFormat (Reed-Solomon 编码)
 *
 * 对标 nrs.login.js 中 `NRS.getPublicKey(converters.stringToHexString(id))`
 * 与 `NRS.getAccountId(id, true)` 的本地派生（注释明确 "Processed locally,
 * not submitted to server"）。
 *
 * @param passphrase 助记词或密码短语
 * @returns 账户信息，含 accountId、accountRS、publicKey
 */
export function passphraseToAccount(passphrase: string): MnemonicAccount {
  const publicKey = getPublicKey(passphrase);
  const accountId = getAccountId(passphrase);
  const accountRS = convertNumericToRSAccountFormat(accountId);

  return {
    accountId,
    accountRS,
    publicKey,
  };
}

/**
 * 格式化助记词为显示格式（每 4 词一行）
 *
 * @param mnemonic 助记词字符串
 * @returns 每行 4 词、用双空格分隔的显示字符串
 */
export function formatMnemonicForDisplay(mnemonic: string): string {
  const words = mnemonic.trim().split(/\s+/).filter((w) => w.length > 0);
  const lines: string[] = [];
  for (let i = 0; i < words.length; i += 4) {
    lines.push(words.slice(i, i + 4).join('  '));
  }
  return lines.join('\n');
}

/**
 * 安全擦除字符串内存（尽力而为）
 *
 * 注意：JS 字符串不可变，此函数无法真正擦除原字符串内存，
 * 仅返回一个随机填充的副本以减少引用残留。敏感数据应尽量
 * 缩短生命周期并避免不必要的复制。
 *
 * @param data 待擦除的字符串
 * @returns 随机填充的字符串（不可用于还原原数据）
 */
export function clearSensitiveData(data: string): string | null {
  if (typeof data !== 'string') return null;
  const cryptoObj =
    typeof globalThis !== 'undefined' && globalThis.crypto
      ? globalThis.crypto
      : undefined;
  if (!cryptoObj) return null;
  const array = new Uint8Array(data.length);
  cryptoObj.getRandomValues(array);
  return String.fromCharCode(...array);
}
