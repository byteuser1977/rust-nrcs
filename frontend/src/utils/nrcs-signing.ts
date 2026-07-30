/******************************************************************************
 * Copyright (c) 2013-2016 The Nxt Core Developers.                             *
 * Copyright (c) 2016-2017 Jelurida IP B.V.                                     *
 * Ported to TypeScript for NRCS blockchain platform.                           *
 *                                                                            *
 * See the LICENSE.txt file at the top-level directory of this distribution   *
 * for licensing information.                                                 *
 *                                                                            *
 * Unless otherwise agreed in a custom licensing agreement with Jelurida B.V.,*
 * no part of the Nxt software, including this file, may be copied, modified, *
 * propagated, or distributed except according to the terms contained in the  *
 * LICENSE.txt file.                                                          *
 *                                                                            *
 * Removal or modification of this copyright notice is prohibited.            *
 *                                                                            *
 ******************************************************************************/

/**
 * NRCS 本地交易签名链路
 *
 * 完整移植 nrs.server.js 中的本地签名相关函数：
 *   - verifyTransactionBytes     字节级字段校验
 *   - verifyTransactionTypes     交易类型附件字节校验
 *   - validateCommonPhasingData  phasing 通用字段校验
 *   - verifyAndSignTransactionBytes  验证 → 本地签名 → 注入签名
 *   - broadcastTransactionBytes  广播已签名交易
 *   - getECBlock                 获取 EC 区块（静态回退 + API 拉取）
 *
 * 安全模型：secretPhrase 仅在客户端本地使用，绝不随请求外发。
 * 服务器仅返回 unsignedTransactionBytes，由客户端完成签名后再广播。
 */

import CryptoJS from 'crypto-js';
import { nrcsPost, nrcsGet } from '@/api/nrcs-client';
import { nrcsApi } from '@/api/modules/nrcs.api';
import {
  byteArrayToHexString,
  hexStringToByteArray,
  stringToByteArray,
  byteArrayToString,
  byteArrayToSignedShort,
  byteArrayToSignedInt32,
  byteArrayToBigInteger,
  byteArrayToWordArrayEx,
  wordArrayToByteArrayEx,
} from './converters';
import { signBytes } from './nrcs-crypto';

// ============================================================================
// 常量定义
// ============================================================================

/** Java 有符号 int 最大值（用于负长度 → 正长度的位运算回退） */
const MAX_INT_JAVA = 2147483647;

/** 调度请求前缀（与 nrs.constants.js SCHEDULE_PREFIX 一致） */
const SCHEDULE_PREFIX = 'schedule';

/**
 * 静态 EC 区块回退常量（与 nrs.constants.js LAST_KNOWN_BLOCK 一致）
 * 当无法连接服务器获取实时 EC 区块时使用。
 */
const LAST_KNOWN_BLOCK: EcBlock = {
  id: '3488276486778630462',
  height: '0',
};

/** 测试网静态 EC 区块回退常量 */
const LAST_KNOWN_TESTNET_BLOCK: EcBlock = {
  id: '3488276486778630462',
  height: '0',
};

// ============================================================================
// 类型定义
// ============================================================================

/** EC 区块结构 */
export interface EcBlock {
  id: string;
  height: string;
}

/**
 * 交易表单数据。对应 nrs.server.js 中传入 verifyTransactionBytes 的 `data` 对象。
 * 字段大多为字符串（来自表单输入），附件相关字段视交易类型而定。
 */
export interface TransactionFormData {
  /** 请求类型，如 sendMoney / sendMessage / setAlias 等 */
  requestType?: string;
  /** 截止时间（分钟） */
  deadline?: string;
  /** 接收方账户 ID（数字字符串） */
  recipient?: string;
  /** 金额（NQT，最小单位字符串） */
  amountNQT?: string;
  /** 手续费（NQT，最小单位字符串） */
  feeNQT?: string;
  /** 发送方公钥（hex） */
  publicKey?: string;
  /** 引用交易全哈希 */
  referencedTransactionFullHash?: string;
  /** 是否广播 */
  broadcast?: string;
  /** 消息内容（普通消息） */
  message?: string;
  /** 消息是否为文本 */
  messageIsText?: string;
  /** 消息是否为可修剪 */
  messageIsPrunable?: string;
  /** 加密消息数据（hex） */
  encryptedMessageData?: string;
  /** 加密消息 nonce（hex） */
  encryptedMessageNonce?: string;
  /** 加密消息是否为文本 */
  messageToEncryptIsText?: string;
  /** 加密消息是否为可修剪 */
  encryptedMessageIsPrunable?: string;
  /** 接收方公钥（用于加密） */
  recipientPublicKey?: string;
  /** 加密给自己的消息数据 */
  encryptToSelfMessageData?: string;
  /** 加密给自己的消息 nonce */
  encryptToSelfMessageNonce?: string;
  /** 加密给自己的消息是否为文本 */
  messageToEncryptToSelfIsText?: string;
  /** phasing 完成高度 */
  phasingFinishHeight?: string;
  /** phasing 投票模型 */
  phasingVotingModel?: string | number;
  /** phasing 法定人数 */
  phasingQuorum?: string;
  /** phasing 最低余额 */
  phasingMinBalance?: string;
  /** phasing 白名单 */
  phasingWhitelisted?: string[];
  /** phasing 持有物 ID */
  phasingHolding?: string;
  /** phasing 最低余额模型 */
  phasingMinBalanceModel?: string | number;
  /** phasing 关联交易全哈希列表 */
  phasingLinkedFullHash?: string[];
  /** phasing 哈希密钥（hex） */
  phasingHashedSecret?: string;
  /** phasing 哈希算法 */
  phasingHashedSecretAlgorithm?: string;
  /** 文件字节（上传场景） */
  filebytes?: ArrayBuffer | number[];
  /** 附件对象（来自服务端 transactionJSON.attachment） */
  [key: string]: any;
}

/** verifyTransactionBytes 校验选项 */
export interface VerifyOptions {
  /** 当前账户公钥（对应 NRS.accountInfo.publicKey） */
  accountPublicKey: string;
  /** 创世账户 ID（对应 NRS.constants.GENESIS），默认 '0' */
  genesis?: string;
  /** 是否校验 EC 区块（对应 isVerifyECBlock），默认 false */
  isVerifyECBlock?: boolean;
  /** 是否测试网，默认 false */
  isTestNet?: boolean;
}

/** verifyAndSignTransactionBytes 执行结果 */
export interface SignResult {
  /** 是否成功 */
  ok: boolean;
  /** 错误码（失败时） */
  errorCode?: number;
  /** 错误描述（失败时） */
  errorDescription?: string;
  /** 已签名交易字节（hex，成功时） */
  payload?: string;
  /** 签名（hex，成功时） */
  signature?: string;
}

/** broadcastTransactionBytes 执行结果 */
export interface BroadcastResult {
  /** 是否已广播 */
  broadcasted: boolean;
  /** 错误码（失败时） */
  errorCode?: number;
  /** 错误描述（失败时） */
  errorDescription?: string;
  /** 交易 ID（成功时） */
  transaction?: string;
  /** 交易全哈希（成功时） */
  fullHash?: string;
}

/** 解析后的交易字段（verifyTransactionBytes 内部使用） */
interface ParsedTransaction {
  // 注意：issueCurrency 分支会用 String(byteArray[pos]) 覆盖 type 字段（货币类型），
  // 因此 type 允许 number | string，与 nrs.server.js 的动态赋值行为一致。
  type: number | string;
  version: number;
  subtype: number;
  timestamp: string;
  deadline: string;
  publicKey: string;
  recipient: string;
  amountNQT: string;
  feeNQT: string;
  referencedTransactionFullHash: string;
  flags: number;
  ecBlockHeight?: string;
  ecBlockId?: string;
  [key: string]: any;
}

/**
 * 将 Uint8Array 转为 CryptoJS 兼容的 WordArray（用于 SHA256 哈希计算）。
 * 内部封装 byteArrayToWordArrayEx + 类型断言，简化调用方代码。
 */
function toWA(bytes: Uint8Array): CryptoJS.lib.WordArray {
  return byteArrayToWordArrayEx(bytes) as unknown as CryptoJS.lib.WordArray;
}

// ============================================================================
// EC 区块获取
// ============================================================================

/**
 * 获取静态 EC 区块（回退方案）。
 *
 * 参考：nrs.constants.js NRS.getECBlock(isTestNet)
 * 当无法连接服务器时，使用内置的 LAST_KNOWN_BLOCK 常量。
 *
 * @param isTestNet 是否测试网
 * @returns EC 区块 { id, height }
 */
export function getECBlock(isTestNet = false): EcBlock {
  return isTestNet ? LAST_KNOWN_TESTNET_BLOCK : LAST_KNOWN_BLOCK;
}

/**
 * 从服务端获取实时 EC 区块。
 *
 * 通过 getECBlock API 拉取当前时间的 EC 区块，比静态回退更准确。
 * 失败时回退到静态常量。
 *
 * @param timestamp 时间戳（秒，NRCS epoch 起算）
 * @param isTestNet 是否测试网（仅用于失败回退）
 * @returns EC 区块 { id, height }
 */
export async function getECBlockFromApi(
  timestamp: number,
  isTestNet = false,
): Promise<EcBlock> {
  try {
    const resp = await nrcsApi.getECBlock(timestamp);
    if (resp && resp.ecBlockId && resp.ecBlockHeight !== undefined) {
      return {
        id: String(resp.ecBlockId),
        height: String(resp.ecBlockHeight),
      };
    }
  } catch (err) {
    console.warn('[nrcs-signing] getECBlock API failed, fallback to static:', err);
  }
  return getECBlock(isTestNet);
}

// ============================================================================
// Phasing 通用字段校验
// ============================================================================

/**
 * 校验 phasing 通用字段（votingModel/quorum/minBalance/whitelist/holding/minBalanceModel）。
 *
 * 参考：nrs.server.js validateCommonPhasingData(byteArray, pos, data, prefix)
 * 用于 setPhasingOnlyControl（prefix='control'）和普通 phasing（prefix='phasing'）。
 *
 * @param byteArray 交易字节数组
 * @param pos 当前读取位置
 * @param data 表单数据
 * @param prefix 字段前缀（'control' 或 'phasing'）
 * @returns 校验通过时返回新的位置；失败返回 -1
 */
function validateCommonPhasingData(
  byteArray: Uint8Array,
  pos: number,
  data: TransactionFormData,
  prefix: string,
): number {
  // votingModel
  const expectedVotingModel = parseInt(data[prefix + 'VotingModel'], 10) & 0xff;
  if (byteArray[pos] !== expectedVotingModel) {
    return -1;
  }
  pos++;

  // quorum（0 时跳过校验）
  const quorum = String(byteArrayToBigInteger(byteArray, pos));
  if (quorum !== '0' && quorum !== String(data[prefix + 'Quorum'])) {
    return -1;
  }
  pos += 8;

  // minBalance（0 时跳过校验）
  const minBalance = String(byteArrayToBigInteger(byteArray, pos));
  if (minBalance !== '0' && minBalance !== data[prefix + 'MinBalance']) {
    return -1;
  }
  pos += 8;

  // whitelist
  const whiteListLength = byteArray[pos];
  pos++;
  for (let i = 0; i < whiteListLength; i++) {
    const accountId = byteArrayToBigInteger(byteArray, pos);
    const accountRS = ''; // RS 校验在调用方处理；此处仅做数字 ID 比对
    pos += 8;
    const expected = data[prefix + 'Whitelisted']?.[i];
    if (String(accountId) !== expected && String(accountRS) !== expected) {
      return -1;
    }
  }

  // holding（0 时跳过校验）
  const holdingId = String(byteArrayToBigInteger(byteArray, pos));
  if (holdingId !== '0' && holdingId !== data[prefix + 'Holding']) {
    return -1;
  }
  pos += 8;

  // minBalanceModel（0 时跳过校验）
  const minBalanceModel = String(byteArray[pos]);
  if (
    minBalanceModel !== '0' &&
    minBalanceModel !== String(data[prefix + 'MinBalanceModel'])
  ) {
    return -1;
  }
  pos++;

  return pos;
}

// ============================================================================
// 交易字节通用校验
// ============================================================================

/**
 * 校验未签名交易字节的通用字段（type/version/timestamp/deadline/publicKey/recipient/amountNQT/feeNQT 等）。
 *
 * 参考：nrs.server.js NRS.verifyTransactionBytes(byteArray, requestType, data, attachment, isVerifyECBlock)
 * 解析前 176/177 字节的通用字段，并与表单数据逐字段比对。
 *
 * @param byteArray 未签名交易字节数组
 * @param requestType 请求类型
 * @param data 表单数据
 * @param attachment 服务端返回的 transactionJSON.attachment
 * @param options 校验选项（accountPublicKey / genesis / isVerifyECBlock / isTestNet）
 * @returns 校验通过返回 true，否则 false
 */
export function verifyTransactionBytes(
  byteArray: Uint8Array,
  requestType: string,
  data: TransactionFormData,
  attachment: any,
  options: VerifyOptions,
): boolean {
  const { accountPublicKey, isVerifyECBlock = false, isTestNet = false } = options;
  const genesis = options.genesis ?? '0';

  const transaction: ParsedTransaction = {
    type: byteArray[0],
    version: (byteArray[1] & 0xf0) >> 4,
    subtype: byteArray[1] & 0x0f,
    timestamp: String(byteArrayToSignedInt32(byteArray, 2)),
    deadline: String(byteArrayToSignedShort(byteArray, 6)),
    publicKey: byteArrayToHexString(byteArray.slice(8, 40)),
    recipient: String(byteArrayToBigInteger(byteArray, 40)),
    amountNQT: String(byteArrayToBigInteger(byteArray, 48)),
    feeNQT: String(byteArrayToBigInteger(byteArray, 56)),
    referencedTransactionFullHash: '',
    flags: 0,
  };

  // referencedTransactionFullHash
  const refHash = byteArray.slice(64, 96);
  transaction.referencedTransactionFullHash = byteArrayToHexString(refHash);
  if (
    transaction.referencedTransactionFullHash ===
    '0000000000000000000000000000000000000000000000000000000000000000'
  ) {
    transaction.referencedTransactionFullHash = '';
  }

  // version > 0 时读取 flags / ecBlockHeight / ecBlockId
  if (transaction.version > 0) {
    transaction.flags = byteArrayToSignedInt32(byteArray, 160);
    transaction.ecBlockHeight = String(byteArrayToSignedInt32(byteArray, 164));
    transaction.ecBlockId = String(byteArrayToBigInteger(byteArray, 168));

    if (isVerifyECBlock) {
      const ecBlock = getECBlock(isTestNet);
      if (transaction.ecBlockHeight !== ecBlock.height) {
        return false;
      }
      if (transaction.ecBlockId !== ecBlock.id) {
        return false;
      }
    }
  }

  // 公钥校验：必须匹配当前账户公钥或表单中的公钥
  if (
    transaction.publicKey !== accountPublicKey &&
    transaction.publicKey !== data.publicKey
  ) {
    return false;
  }

  // deadline 校验
  if (transaction.deadline !== data.deadline) {
    return false;
  }

  // recipient 校验（允许 genesis / 空字符串 → 0 的回退）
  if (transaction.recipient !== data.recipient) {
    if (
      (data.recipient === genesis || data.recipient === '') &&
      transaction.recipient === '0'
    ) {
      // ok
    } else {
      return false;
    }
  }

  // amountNQT 校验
  if (transaction.amountNQT !== data.amountNQT) {
    return false;
  }

  // referencedTransactionFullHash 校验
  if ('referencedTransactionFullHash' in data) {
    if (transaction.referencedTransactionFullHash !== data.referencedTransactionFullHash) {
      return false;
    }
  } else if (transaction.referencedTransactionFullHash !== '') {
    return false;
  }

  // 计算附件起始位置
  let pos: number;
  if (transaction.version > 0) {
    // sendMoney / sendMessage 没有附件版本字节
    if (requestType === 'sendMoney' || requestType === 'sendMessage') {
      pos = 176;
    } else {
      pos = 177;
    }
  } else {
    pos = 160;
  }

  return verifyTransactionTypes(byteArray, transaction, requestType, data, pos, attachment);
}

// ============================================================================
// 交易类型附件校验
// ============================================================================

/**
 * 校验交易类型附件字节。
 *
 * 参考：nrs.server.js NRS.verifyTransactionTypes(byteArray, transaction, requestType, data, pos, attachment)
 * 按 requestType 分支解析附件字节，并与表单数据逐字段比对。
 * 随后统一校验 flags 控制的可选附件（普通消息/加密消息/接收方公钥/加密给自己/phasing/可修剪消息）。
 *
 * @param byteArray 交易字节数组
 * @param transaction 已解析的交易字段
 * @param requestType 请求类型
 * @param data 表单数据
 * @param pos 附件起始位置
 * @param attachment 服务端返回的 attachment 对象
 * @returns 校验通过返回 true，否则 false
 */
export function verifyTransactionTypes(
  byteArray: Uint8Array,
  transaction: ParsedTransaction,
  requestType: string,
  data: TransactionFormData,
  pos: number,
  attachment: any,
): boolean {
  let length = 0;
  let i = 0;

  switch (requestType) {
    case 'sendMoney':
      if (transaction.type !== 0 || transaction.subtype !== 0) {
        return false;
      }
      break;

    case 'sendMessage':
      if (transaction.type !== 1 || transaction.subtype !== 0) {
        return false;
      }
      break;

    case 'setAlias':
      if (transaction.type !== 1 || transaction.subtype !== 1) {
        return false;
      }
      length = parseInt(String(byteArray[pos]), 10);
      pos++;
      transaction.aliasName = byteArrayToString(byteArray, pos, length);
      pos += length;
      length = byteArrayToSignedShort(byteArray, pos);
      pos += 2;
      transaction.aliasURI = byteArrayToString(byteArray, pos, length);
      pos += length;
      if (transaction.aliasName !== data.aliasName || transaction.aliasURI !== data.aliasURI) {
        return false;
      }
      break;

    case 'createPoll':
      if (transaction.type !== 1 || transaction.subtype !== 2) {
        return false;
      }
      length = byteArrayToSignedShort(byteArray, pos);
      pos += 2;
      transaction.name = byteArrayToString(byteArray, pos, length);
      pos += length;
      length = byteArrayToSignedShort(byteArray, pos);
      pos += 2;
      transaction.description = byteArrayToString(byteArray, pos, length);
      pos += length;
      transaction.finishHeight = byteArrayToSignedInt32(byteArray, pos);
      pos += 4;
      const nr_options = byteArray[pos];
      pos++;

      for (i = 0; i < nr_options; i++) {
        const optionLength = byteArrayToSignedShort(byteArray, pos);
        pos += 2;
        transaction['option' + (i < 10 ? '0' + i : i)] = byteArrayToString(
          byteArray,
          pos,
          optionLength,
        );
        pos += optionLength;
      }
      transaction.votingModel = String(byteArray[pos]);
      pos++;
      transaction.minNumberOfOptions = String(byteArray[pos]);
      pos++;
      transaction.maxNumberOfOptions = String(byteArray[pos]);
      pos++;
      transaction.minRangeValue = String(byteArray[pos]);
      pos++;
      transaction.maxRangeValue = String(byteArray[pos]);
      pos++;
      transaction.minBalance = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      transaction.minBalanceModel = String(byteArray[pos]);
      pos++;
      transaction.holding = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;

      if (
        transaction.name !== data.name ||
        transaction.description !== data.description ||
        transaction.minNumberOfOptions !== data.minNumberOfOptions ||
        transaction.maxNumberOfOptions !== data.maxNumberOfOptions
      ) {
        return false;
      }

      for (i = 0; i < nr_options; i++) {
        if (transaction['option' + (i < 10 ? '0' + i : i)] !== data['option' + (i < 10 ? '0' + i : i)]) {
          return false;
        }
      }

      if ('option' + (i < 10 ? '0' + i : i) in data) {
        return false;
      }
      break;

    case 'castVote':
      if (transaction.type !== 1 || transaction.subtype !== 3) {
        return false;
      }
      transaction.poll = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      const voteLength = byteArray[pos];
      pos++;
      transaction.votes = [];

      for (i = 0; i < voteLength; i++) {
        transaction['vote' + (i < 10 ? '0' + i : i)] = byteArray[pos];
        pos++;
      }
      if (transaction.poll !== data.poll) {
        return false;
      }
      break;

    case 'hubAnnouncement':
      // 参考实现直接返回 false（功能未启用）
      return false;

    case 'setAccountInfo':
      if (transaction.type !== 1 || transaction.subtype !== 5) {
        return false;
      }
      length = parseInt(String(byteArray[pos]), 10);
      pos++;
      transaction.name = byteArrayToString(byteArray, pos, length);
      pos += length;
      length = byteArrayToSignedShort(byteArray, pos);
      pos += 2;
      transaction.description = byteArrayToString(byteArray, pos, length);
      pos += length;
      if (transaction.name !== data.name || transaction.description !== data.description) {
        return false;
      }
      break;

    case 'sellAlias':
      if (transaction.type !== 1 || transaction.subtype !== 6) {
        return false;
      }
      length = parseInt(String(byteArray[pos]), 10);
      pos++;
      transaction.alias = byteArrayToString(byteArray, pos, length);
      pos += length;
      transaction.priceNQT = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      if (transaction.alias !== data.aliasName || transaction.priceNQT !== data.priceNQT) {
        return false;
      }
      break;

    case 'buyAlias':
      if (transaction.type !== 1 || transaction.subtype !== 7) {
        return false;
      }
      length = parseInt(String(byteArray[pos]), 10);
      pos++;
      transaction.alias = byteArrayToString(byteArray, pos, length);
      pos += length;
      if (transaction.alias !== data.aliasName) {
        return false;
      }
      break;

    case 'deleteAlias':
      if (transaction.type !== 1 || transaction.subtype !== 8) {
        return false;
      }
      length = parseInt(String(byteArray[pos]), 10);
      pos++;
      transaction.alias = byteArrayToString(byteArray, pos, length);
      pos += length;
      if (transaction.alias !== data.aliasName) {
        return false;
      }
      break;

    case 'approveTransaction':
      if (transaction.type !== 1 || transaction.subtype !== 9) {
        return false;
      }
      const fullHashesLength = byteArray[pos];
      if (fullHashesLength !== 1) {
        return false;
      }
      pos++;
      transaction.transactionFullHash = byteArrayToHexString(byteArray.slice(pos, pos + 32));
      pos += 32;
      if (transaction.transactionFullHash !== data.transactionFullHash) {
        return false;
      }
      transaction.revealedSecretLength = byteArrayToSignedInt32(byteArray, pos);
      pos += 4;
      if (transaction.revealedSecretLength > 0) {
        transaction.revealedSecret = byteArrayToHexString(
          byteArray.slice(pos, pos + transaction.revealedSecretLength),
        );
        pos += transaction.revealedSecretLength;
      }
      if (
        transaction.revealedSecret !== data.revealedSecret &&
        transaction.revealedSecret !== byteArrayToHexString(stringToByteArray(data.revealedSecretText || ''))
      ) {
        return false;
      }
      break;

    case 'setAccountProperty':
      if (transaction.type !== 1 || transaction.subtype !== 10) {
        return false;
      }
      length = byteArray[pos];
      pos++;
      if (byteArrayToString(byteArray, pos, length) !== data.property) {
        return false;
      }
      pos += length;
      length = byteArray[pos];
      pos++;
      if (byteArrayToString(byteArray, pos, length) !== data.value) {
        return false;
      }
      pos += length;
      break;

    case 'deleteAccountProperty':
      if (transaction.type !== 1 || transaction.subtype !== 11) {
        return false;
      }
      // property id 无法直接校验，仅跳过
      String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      break;

    case 'issueAsset':
      if (transaction.type !== 2 || transaction.subtype !== 0) {
        return false;
      }
      length = byteArray[pos];
      pos++;
      transaction.name = byteArrayToString(byteArray, pos, length);
      pos += length;
      length = byteArrayToSignedShort(byteArray, pos);
      pos += 2;
      transaction.description = byteArrayToString(byteArray, pos, length);
      pos += length;
      transaction.quantityQNT = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      transaction.decimals = String(byteArray[pos]);
      pos++;
      if (
        transaction.name !== data.name ||
        transaction.description !== data.description ||
        transaction.quantityQNT !== data.quantityQNT ||
        transaction.decimals !== data.decimals
      ) {
        return false;
      }
      break;

    case 'transferAsset':
      if (transaction.type !== 2 || transaction.subtype !== 1) {
        return false;
      }
      transaction.asset = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      transaction.quantityQNT = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      if (transaction.asset !== data.asset || transaction.quantityQNT !== data.quantityQNT) {
        return false;
      }
      break;

    case 'placeAskOrder':
    case 'placeBidOrder':
      if (transaction.type !== 2) {
        return false;
      } else if (requestType === 'placeAskOrder' && transaction.subtype !== 2) {
        return false;
      } else if (requestType === 'placeBidOrder' && transaction.subtype !== 3) {
        return false;
      }
      transaction.asset = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      transaction.quantityQNT = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      transaction.priceNQT = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      if (
        transaction.asset !== data.asset ||
        transaction.quantityQNT !== data.quantityQNT ||
        transaction.priceNQT !== data.priceNQT
      ) {
        return false;
      }
      break;

    case 'cancelAskOrder':
    case 'cancelBidOrder':
      if (transaction.type !== 2) {
        return false;
      } else if (requestType === 'cancelAskOrder' && transaction.subtype !== 4) {
        return false;
      } else if (requestType === 'cancelBidOrder' && transaction.subtype !== 5) {
        return false;
      }
      transaction.order = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      if (transaction.order !== data.order) {
        return false;
      }
      break;

    case 'deleteAssetShares':
      if (transaction.type !== 2 || transaction.subtype !== 7) {
        return false;
      }
      transaction.asset = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      transaction.quantityQNT = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      if (transaction.asset !== data.asset || transaction.quantityQNT !== data.quantityQNT) {
        return false;
      }
      break;

    case 'dividendPayment':
      if (transaction.type !== 2 || transaction.subtype !== 6) {
        return false;
      }
      transaction.asset = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      transaction.height = String(byteArrayToSignedInt32(byteArray, pos));
      pos += 4;
      transaction.amountNQTPerQNT = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      if (
        transaction.asset !== data.asset ||
        transaction.height !== data.height ||
        transaction.amountNQTPerQNT !== data.amountNQTPerQNT
      ) {
        return false;
      }
      break;

    case 'dgsListing':
      if (transaction.type !== 3 || transaction.subtype !== 0) {
        return false;
      }
      length = byteArrayToSignedShort(byteArray, pos);
      pos += 2;
      transaction.name = byteArrayToString(byteArray, pos, length);
      pos += length;
      length = byteArrayToSignedShort(byteArray, pos);
      pos += 2;
      transaction.description = byteArrayToString(byteArray, pos, length);
      pos += length;
      length = byteArrayToSignedShort(byteArray, pos);
      pos += 2;
      transaction.tags = byteArrayToString(byteArray, pos, length);
      pos += length;
      transaction.quantity = String(byteArrayToSignedInt32(byteArray, pos));
      pos += 4;
      transaction.priceNQT = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      if (
        transaction.name !== data.name ||
        transaction.description !== data.description ||
        transaction.tags !== data.tags ||
        transaction.quantity !== data.quantity ||
        transaction.priceNQT !== data.priceNQT
      ) {
        return false;
      }
      break;

    case 'dgsDelisting':
      if (transaction.type !== 3 || transaction.subtype !== 1) {
        return false;
      }
      transaction.goods = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      if (transaction.goods !== data.goods) {
        return false;
      }
      break;

    case 'dgsPriceChange':
      if (transaction.type !== 3 || transaction.subtype !== 2) {
        return false;
      }
      transaction.goods = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      transaction.priceNQT = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      if (transaction.goods !== data.goods || transaction.priceNQT !== data.priceNQT) {
        return false;
      }
      break;

    case 'dgsQuantityChange':
      if (transaction.type !== 3 || transaction.subtype !== 3) {
        return false;
      }
      transaction.goods = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      transaction.deltaQuantity = String(byteArrayToSignedInt32(byteArray, pos));
      pos += 4;
      if (transaction.goods !== data.goods || transaction.deltaQuantity !== data.deltaQuantity) {
        return false;
      }
      break;

    case 'dgsPurchase':
      if (transaction.type !== 3 || transaction.subtype !== 4) {
        return false;
      }
      transaction.goods = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      transaction.quantity = String(byteArrayToSignedInt32(byteArray, pos));
      pos += 4;
      transaction.priceNQT = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      transaction.deliveryDeadlineTimestamp = String(byteArrayToSignedInt32(byteArray, pos));
      pos += 4;
      if (
        transaction.goods !== data.goods ||
        transaction.quantity !== data.quantity ||
        transaction.priceNQT !== data.priceNQT ||
        transaction.deliveryDeadlineTimestamp !== data.deliveryDeadlineTimestamp
      ) {
        return false;
      }
      break;

    case 'dgsDelivery':
      if (transaction.type !== 3 || transaction.subtype !== 5) {
        return false;
      }
      transaction.purchase = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      const encryptedGoodsLength = byteArrayToSignedShort(byteArray, pos);
      const goodsLength = byteArrayToSignedInt32(byteArray, pos);
      transaction.goodsIsText = goodsLength < 0; // ugly hack??
      if (goodsLength < 0) {
        // 与 Java MAX_INT_JAVA 按位与还原正长度
      }
      pos += 4;
      transaction.goodsData = byteArrayToHexString(
        byteArray.slice(pos, pos + encryptedGoodsLength),
      );
      pos += encryptedGoodsLength;
      transaction.goodsNonce = byteArrayToHexString(byteArray.slice(pos, pos + 32));
      pos += 32;
      transaction.discountNQT = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      const goodsIsText = transaction.goodsIsText ? 'true' : 'false';
      if (goodsIsText !== data.goodsIsText) {
        return false;
      }
      if (
        transaction.purchase !== data.purchase ||
        transaction.goodsData !== data.goodsData ||
        transaction.goodsNonce !== data.goodsNonce ||
        transaction.discountNQT !== data.discountNQT
      ) {
        return false;
      }
      break;

    case 'dgsFeedback':
      if (transaction.type !== 3 || transaction.subtype !== 6) {
        return false;
      }
      transaction.purchase = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      if (transaction.purchase !== data.purchase) {
        return false;
      }
      break;

    case 'dgsRefund':
      if (transaction.type !== 3 || transaction.subtype !== 7) {
        return false;
      }
      transaction.purchase = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      transaction.refundNQT = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      if (transaction.purchase !== data.purchase || transaction.refundNQT !== data.refundNQT) {
        return false;
      }
      break;

    case 'leaseBalance':
      if (transaction.type !== 4 || transaction.subtype !== 0) {
        return false;
      }
      transaction.period = String(byteArrayToSignedShort(byteArray, pos));
      pos += 2;
      if (transaction.period !== data.period) {
        return false;
      }
      break;

    case 'setPhasingOnlyControl':
      if (transaction.type !== 4 || transaction.subtype !== 1) {
        return false;
      }
      return validateCommonPhasingData(byteArray, pos, data, 'control') !== -1;

    case 'issueCurrency':
      if (transaction.type !== 5 || transaction.subtype !== 0) {
        return false;
      }
      length = byteArray[pos];
      pos++;
      transaction.name = byteArrayToString(byteArray, pos, length);
      pos += length;
      const codeLength = byteArray[pos];
      pos++;
      transaction.code = byteArrayToString(byteArray, pos, codeLength);
      pos += codeLength;
      length = byteArrayToSignedShort(byteArray, pos);
      pos += 2;
      transaction.description = byteArrayToString(byteArray, pos, length);
      pos += length;
      transaction.type = String(byteArray[pos]);
      pos++;
      transaction.initialSupply = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      transaction.reserveSupply = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      transaction.maxSupply = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      transaction.issuanceHeight = String(byteArrayToSignedInt32(byteArray, pos));
      pos += 4;
      transaction.minReservePerUnitNQT = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      transaction.minDifficulty = String(byteArray[pos]);
      pos++;
      transaction.maxDifficulty = String(byteArray[pos]);
      pos++;
      transaction.ruleset = String(byteArray[pos]);
      pos++;
      transaction.algorithm = String(byteArray[pos]);
      pos++;
      transaction.decimals = String(byteArray[pos]);
      pos++;
      if (
        transaction.name !== data.name ||
        transaction.code !== data.code ||
        transaction.description !== data.description ||
        transaction.type != data.type ||
        transaction.initialSupply !== data.initialSupply ||
        transaction.reserveSupply !== data.reserveSupply ||
        transaction.maxSupply !== data.maxSupply ||
        transaction.issuanceHeight !== data.issuanceHeight ||
        transaction.ruleset !== data.ruleset ||
        transaction.algorithm !== data.algorithm ||
        transaction.decimals !== data.decimals
      ) {
        return false;
      }
      if (
        transaction.minReservePerUnitNQT !== '0' &&
        transaction.minReservePerUnitNQT !== data.minReservePerUnitNQT
      ) {
        return false;
      }
      if (transaction.minDifficulty !== '0' && transaction.minDifficulty !== data.minDifficulty) {
        return false;
      }
      if (transaction.maxDifficulty !== '0' && transaction.maxDifficulty !== data.maxDifficulty) {
        return false;
      }
      break;

    case 'currencyReserveIncrease':
      if (transaction.type !== 5 || transaction.subtype !== 1) {
        return false;
      }
      transaction.currency = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      transaction.amountPerUnitNQT = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      if (
        transaction.currency !== data.currency ||
        transaction.amountPerUnitNQT !== data.amountPerUnitNQT
      ) {
        return false;
      }
      break;

    case 'currencyReserveClaim':
      if (transaction.type !== 5 || transaction.subtype !== 2) {
        return false;
      }
      transaction.currency = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      transaction.units = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      if (transaction.currency !== data.currency || transaction.units !== data.units) {
        return false;
      }
      break;

    case 'transferCurrency':
      if (transaction.type !== 5 || transaction.subtype !== 3) {
        return false;
      }
      transaction.currency = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      transaction.units = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      if (transaction.currency !== data.currency || transaction.units !== data.units) {
        return false;
      }
      break;

    case 'publishExchangeOffer':
      if (transaction.type !== 5 || transaction.subtype !== 4) {
        return false;
      }
      transaction.currency = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      transaction.buyRateNQT = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      transaction.sellRateNQT = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      transaction.totalBuyLimit = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      transaction.totalSellLimit = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      transaction.initialBuySupply = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      transaction.initialSellSupply = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      transaction.expirationHeight = String(byteArrayToSignedInt32(byteArray, pos));
      pos += 4;
      if (
        transaction.currency !== data.currency ||
        transaction.buyRateNQT !== data.buyRateNQT ||
        transaction.sellRateNQT !== data.sellRateNQT ||
        transaction.totalBuyLimit !== data.totalBuyLimit ||
        transaction.totalSellLimit !== data.totalSellLimit ||
        transaction.initialBuySupply !== data.initialBuySupply ||
        transaction.initialSellSupply !== data.initialSellSupply ||
        transaction.expirationHeight !== data.expirationHeight
      ) {
        return false;
      }
      break;

    case 'currencyBuy':
      if (transaction.type !== 5 || transaction.subtype !== 5) {
        return false;
      }
      transaction.currency = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      transaction.rateNQT = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      transaction.units = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      if (
        transaction.currency !== data.currency ||
        transaction.rateNQT !== data.rateNQT ||
        transaction.units !== data.units
      ) {
        return false;
      }
      break;

    case 'currencySell':
      if (transaction.type !== 5 || transaction.subtype !== 6) {
        return false;
      }
      transaction.currency = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      transaction.rateNQT = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      transaction.units = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      if (
        transaction.currency !== data.currency ||
        transaction.rateNQT !== data.rateNQT ||
        transaction.units !== data.units
      ) {
        return false;
      }
      break;

    case 'currencyMint':
      if (transaction.type !== 5 || transaction.subtype !== 7) {
        return false;
      }
      transaction.currency = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      transaction.nonce = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      transaction.units = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      transaction.counter = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      if (
        transaction.currency !== data.currency ||
        transaction.nonce !== data.nonce ||
        transaction.units !== data.units ||
        transaction.counter !== data.counter
      ) {
        return false;
      }
      break;

    case 'deleteCurrency':
      if (transaction.type !== 5 || transaction.subtype !== 8) {
        return false;
      }
      transaction.currency = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      if (transaction.currency !== data.currency) {
        return false;
      }
      break;

    case 'uploadTaggedData':
      if (transaction.type !== 6 || transaction.subtype !== 0) {
        return false;
      }
      if (!verifyTaggedDataHash(byteArray, pos, data, attachment)) {
        return false;
      }
      break;

    case 'extendTaggedData':
      if (transaction.type !== 6 || transaction.subtype !== 1) {
        return false;
      }
      transaction.taggedDataId = String(byteArrayToBigInteger(byteArray, pos));
      pos += 8;
      if (transaction.taggedDataId !== data.transaction) {
        return false;
      }
      break;

    case 'shufflingCreate':
      if (transaction.type !== 7 || transaction.subtype !== 0) {
        return false;
      }
      const holding = String(byteArrayToBigInteger(byteArray, pos));
      if (
        (holding !== '0' && holding !== data.holding) ||
        (holding === '0' &&
          data.holding !== undefined &&
          data.holding !== '' &&
          data.holding !== '0')
      ) {
        return false;
      }
      pos += 8;
      const holdingType = String(byteArray[pos]);
      if (
        (holdingType !== '0' && holdingType !== data.holdingType) ||
        (holdingType === '0' &&
          data.holdingType !== undefined &&
          data.holdingType !== '' &&
          data.holdingType !== '0')
      ) {
        return false;
      }
      pos++;
      const amount = String(byteArrayToBigInteger(byteArray, pos));
      if (amount !== data.amount) {
        return false;
      }
      pos += 8;
      const participantCount = String(byteArray[pos]);
      if (participantCount !== data.participantCount) {
        return false;
      }
      pos++;
      const registrationPeriod = byteArrayToSignedShort(byteArray, pos);
      if (registrationPeriod !== Number(data.registrationPeriod)) {
        return false;
      }
      pos += 2;
      break;

    default:
      // 未知 requestType
      return false;
  }

  return verifyOptionalAttachments(byteArray, transaction, requestType, data, attachment, pos);
}

/**
 * 校验 uploadTaggedData 的哈希。
 *
 * 参考：nrs.server.js verifyTransactionTypes 中 uploadTaggedData 分支
 * 服务端在字节中写入 name/description/tags/type/channel/isText/filename/data 的 SHA256 哈希，
 * 客户端用同样输入重算并比对。
 *
 * @param byteArray 交易字节数组
 * @param pos 当前位置（指向哈希起始）
 * @param data 表单数据
 * @param attachment 服务端 attachment 对象
 * @returns 校验通过返回 true
 */
function verifyTaggedDataHash(
  byteArray: Uint8Array,
  pos: number,
  data: TransactionFormData,
  attachment: any,
): boolean {
  const serverHash = byteArrayToHexString(byteArray.slice(pos, pos + 32));
  const sha256 = CryptoJS.algo.SHA256.create();
  sha256.update(toWA(stringToByteArray(data.name || '')));
  sha256.update(toWA(stringToByteArray(data.description || '')));
  sha256.update(toWA(stringToByteArray(data.tags || '')));
  sha256.update(toWA(stringToByteArray(attachment?.type || '')));
  sha256.update(toWA(stringToByteArray(data.channel || '')));
  const isText: number[] = [attachment?.isText ? 1 : 0];
  sha256.update(toWA(new Uint8Array(isText)));
  sha256.update(toWA(stringToByteArray(data.filename || '')));
  const dataBytes = new Int8Array(data.filebytes || []);
  sha256.update(toWA(new Uint8Array(dataBytes)));
  const hashWords = sha256.finalize();
  const calculatedHash = byteArrayToHexString(wordArrayToByteArrayEx(hashWords));
  return serverHash === calculatedHash;
}

/**
 * 校验 flags 控制的可选附件（普通消息/加密消息/接收方公钥/加密给自己/phasing/可修剪消息）。
 *
 * 参考：nrs.server.js verifyTransactionTypes 末尾的统一 flags 校验段。
 * 按位依次检查 flags 的每一位（position 从 1 开始左移）。
 *
 * @param byteArray 交易字节数组
 * @param transaction 已解析的交易字段
 * @param requestType 请求类型
 * @param data 表单数据
 * @param attachment 服务端 attachment
 * @param pos 当前位置（附件主体之后）
 * @returns 校验通过返回 true
 */
function verifyOptionalAttachments(
  byteArray: Uint8Array,
  transaction: ParsedTransaction,
  requestType: string,
  data: TransactionFormData,
  attachment: any,
  pos: number,
): boolean {
  let position = 1;
  let attachmentVersion: number;

  // ── 普通消息（flag bit 0） ──────────────────────────────────────────────
  if (
    (transaction.flags & position) !== 0 ||
    (requestType === 'sendMessage' && data.message && !(data.messageIsPrunable === 'true'))
  ) {
    attachmentVersion = byteArray[pos];
    if (attachmentVersion < 0 || attachmentVersion > 2) {
      return false;
    }
    pos++;
    let messageLength = byteArrayToSignedInt32(byteArray, pos);
    transaction.messageIsText = messageLength < 0; // ugly hack??
    if (messageLength < 0) {
      messageLength &= MAX_INT_JAVA;
    }
    pos += 4;
    if (transaction.messageIsText) {
      transaction.message = byteArrayToString(byteArray, pos, messageLength);
    } else {
      const slice = byteArray.slice(pos, pos + messageLength);
      transaction.message = byteArrayToHexString(slice);
    }
    pos += messageLength;
    const messageIsText = transaction.messageIsText ? 'true' : 'false';
    if (messageIsText !== data.messageIsText) {
      return false;
    }
    if (transaction.message !== data.message) {
      return false;
    }
  } else if (data.message && !(data.messageIsPrunable === 'true')) {
    return false;
  }

  position <<= 1;

  // ── 加密消息（flag bit 1） ─────────────────────────────────────────────
  if ((transaction.flags & position) !== 0) {
    attachmentVersion = byteArray[pos];
    if (attachmentVersion < 0 || attachmentVersion > 2) {
      return false;
    }
    pos++;
    let encryptedMessageLength = byteArrayToSignedInt32(byteArray, pos);
    transaction.messageToEncryptIsText = encryptedMessageLength < 0;
    if (encryptedMessageLength < 0) {
      encryptedMessageLength &= MAX_INT_JAVA;
    }
    pos += 4;
    transaction.encryptedMessageData = byteArrayToHexString(
      byteArray.slice(pos, pos + encryptedMessageLength),
    );
    pos += encryptedMessageLength;
    transaction.encryptedMessageNonce = byteArrayToHexString(byteArray.slice(pos, pos + 32));
    pos += 32;
    const messageToEncryptIsText = transaction.messageToEncryptIsText ? 'true' : 'false';
    if (messageToEncryptIsText !== data.messageToEncryptIsText) {
      return false;
    }
    if (
      transaction.encryptedMessageData !== data.encryptedMessageData ||
      transaction.encryptedMessageNonce !== data.encryptedMessageNonce
    ) {
      return false;
    }
  } else if (data.encryptedMessageData && !(data.encryptedMessageIsPrunable === 'true')) {
    return false;
  }

  position <<= 1;

  // ── 接收方公钥（flag bit 2） ───────────────────────────────────────────
  if ((transaction.flags & position) !== 0) {
    attachmentVersion = byteArray[pos];
    if (attachmentVersion < 0 || attachmentVersion > 2) {
      return false;
    }
    pos++;
    const recipientPublicKey = byteArrayToHexString(byteArray.slice(pos, pos + 32));
    if (recipientPublicKey !== data.recipientPublicKey) {
      return false;
    }
    pos += 32;
  } else if (data.recipientPublicKey) {
    return false;
  }

  position <<= 1;

  // ── 加密给自己的消息（flag bit 3） ────────────────────────────────────
  if ((transaction.flags & position) !== 0) {
    attachmentVersion = byteArray[pos];
    if (attachmentVersion < 0 || attachmentVersion > 2) {
      return false;
    }
    pos++;
    let encryptedToSelfMessageLength = byteArrayToSignedInt32(byteArray, pos);
    transaction.messageToEncryptToSelfIsText = encryptedToSelfMessageLength < 0;
    if (encryptedToSelfMessageLength < 0) {
      encryptedToSelfMessageLength &= MAX_INT_JAVA;
    }
    pos += 4;
    transaction.encryptToSelfMessageData = byteArrayToHexString(
      byteArray.slice(pos, pos + encryptedToSelfMessageLength),
    );
    pos += encryptedToSelfMessageLength;
    transaction.encryptToSelfMessageNonce = byteArrayToHexString(byteArray.slice(pos, pos + 32));
    pos += 32;
    const messageToEncryptToSelfIsText = transaction.messageToEncryptToSelfIsText
      ? 'true'
      : 'false';
    if (messageToEncryptToSelfIsText !== data.messageToEncryptToSelfIsText) {
      return false;
    }
    if (
      transaction.encryptToSelfMessageData !== data.encryptToSelfMessageData ||
      transaction.encryptToSelfMessageNonce !== data.encryptToSelfMessageNonce
    ) {
      return false;
    }
  } else if (data.encryptToSelfMessageData) {
    return false;
  }

  position <<= 1;

  // ── phasing（flag bit 4） ──────────────────────────────────────────────
  if ((transaction.flags & position) !== 0) {
    attachmentVersion = byteArray[pos];
    if (attachmentVersion < 0 || attachmentVersion > 2) {
      return false;
    }
    pos++;
    if (String(byteArrayToSignedInt32(byteArray, pos)) !== data.phasingFinishHeight) {
      return false;
    }
    pos += 4;
    pos = validateCommonPhasingData(byteArray, pos, data, 'phasing');
    if (pos === -1) {
      return false;
    }
    const linkedFullHashesLength = byteArray[pos];
    pos++;
    for (let i = 0; i < linkedFullHashesLength; i++) {
      const fullHash = byteArrayToHexString(byteArray.slice(pos, pos + 32));
      pos += 32;
      if (fullHash !== data.phasingLinkedFullHash?.[i]) {
        return false;
      }
    }
    const hashedSecretLength = byteArray[pos];
    pos++;
    if (
      hashedSecretLength > 0 &&
      byteArrayToHexString(byteArray.slice(pos, pos + hashedSecretLength)) !==
        data.phasingHashedSecret
    ) {
      return false;
    }
    pos += hashedSecretLength;
    const algorithm = String(byteArray[pos]);
    if (algorithm !== '0' && algorithm !== data.phasingHashedSecretAlgorithm) {
      return false;
    }
    pos++;
  }

  position <<= 1;

  // ── 可修剪普通消息（flag bit 5） ───────────────────────────────────────
  if ((transaction.flags & position) !== 0) {
    attachmentVersion = byteArray[pos];
    if (attachmentVersion < 0 || attachmentVersion > 2) {
      return false;
    }
    pos++;
    const serverHash = byteArrayToHexString(byteArray.slice(pos, pos + 32));
    pos += 32;
    const sha256 = CryptoJS.algo.SHA256.create();
    const isText: number[] = [data.messageIsText === 'true' ? 1 : 0];
    sha256.update(toWA(new Uint8Array(isText)));
    let utfBytes: Uint8Array;
    if (data.filebytes) {
      utfBytes = new Uint8Array(new Int8Array(data.filebytes));
    } else {
      utfBytes = stringToByteArray(data.message || '');
    }
    sha256.update(toWA(utfBytes));
    const hashWords = sha256.finalize();
    const calculatedHash = byteArrayToHexString(wordArrayToByteArrayEx(hashWords));
    if (serverHash !== calculatedHash) {
      return false;
    }
  }
  position <<= 1;

  // ── 可修剪加密消息（flag bit 6） ───────────────────────────────────────
  if ((transaction.flags & position) !== 0) {
    attachmentVersion = byteArray[pos];
    if (attachmentVersion < 0 || attachmentVersion > 2) {
      return false;
    }
    pos++;
    const serverHash = byteArrayToHexString(byteArray.slice(pos, pos + 32));
    const sha256 = CryptoJS.algo.SHA256.create();
    if (data.messageToEncryptIsText === 'true') {
      sha256.update(toWA(new Uint8Array([1])));
    } else {
      sha256.update(toWA(new Uint8Array([0])));
    }
    sha256.update(toWA(new Uint8Array([1]))); // compression
    let utfBytes: Uint8Array;
    if (data.filebytes) {
      utfBytes = new Uint8Array(new Int8Array(data.filebytes));
    } else {
      utfBytes = hexStringToByteArray(data.encryptedMessageData || '');
    }
    sha256.update(toWA(utfBytes));
    sha256.update(
      toWA(hexStringToByteArray(data.encryptedMessageNonce || '')),
    );
    const hashWords = sha256.finalize();
    const calculatedHash = byteArrayToHexString(wordArrayToByteArrayEx(hashWords));
    if (serverHash !== calculatedHash) {
      return false;
    }
  }

  return true;
}

// ============================================================================
// 验证 + 本地签名
// ============================================================================

/**
 * 验证未签名交易字节并使用 secretPhrase 本地签名。
 *
 * 参考：nrs.server.js NRS.verifyAndSignTransactionBytes(transactionBytes, signature, requestType, data, callback, response, extra, isVerifyECBlock)
 *
 * 流程：
 *   1. 将 hex 字符串转为字节数组
 *   2. 调用 verifyTransactionBytes 校验所有字段
 *   3. 用 secretPhrase 对未签名字节进行 EC-KCDSA 签名
 *   4. 将签名注入到字节 [192, 320) 区间，得到完整交易 payload
 *
 * @param transactionBytes 未签名交易字节（hex）
 * @param requestType 请求类型
 * @param data 表单数据
 * @param response 服务端响应（需含 transactionJSON.attachment）
 * @param secretPhrase 本地密钥短语（仅用于签名，不外发）
 * @param options 校验选项
 * @returns 签名结果 { ok, payload, signature } 或 { ok: false, errorCode, errorDescription }
 */
export function verifyAndSignTransactionBytes(
  transactionBytes: string,
  requestType: string,
  data: TransactionFormData,
  response: { transactionJSON?: { attachment?: any } },
  secretPhrase: string,
  options: VerifyOptions,
): SignResult {
  const byteArray = hexStringToByteArray(transactionBytes);

  // 校验未签名字节
  const attachment = response?.transactionJSON?.attachment;
  if (
    !verifyTransactionBytes(byteArray, requestType, data, attachment, options)
  ) {
    return {
      ok: false,
      errorCode: 1,
      errorDescription: '服务端返回的交易字节与本地数据不一致，已拒绝签名',
    };
  }

  // 本地签名：unsignedTransactionBytes 作为待签名消息
  const signature = signBytes(transactionBytes, secretPhrase);

  // 注入签名到 [192, 320) 区间
  const payload =
    transactionBytes.substring(0, 192) +
    signature +
    transactionBytes.substring(320);

  return {
    ok: true,
    payload,
    signature,
  };
}

// ============================================================================
// 广播已签名交易
// ============================================================================

/** 广播选项 */
export interface BroadcastOptions {
  /** 是否调度请求（对应 isSchedule） */
  isSchedule?: boolean;
  /** 调度请求的 offerIssuer（货币买入专用） */
  offerIssuer?: string;
  /** 管理员密码（对应 NRS.getAdminPassword()） */
  adminPassword?: string;
  /** 是否通过 API 代理（对应 NRS.state.apiProxy） */
  apiProxy?: boolean;
  /** 原始请求类型（调度请求时用于构造前缀） */
  requestType?: string;
}

/**
 * 广播已签名交易字节到网络。
 *
 * 参考：nrs.server.js NRS.broadcastTransactionBytes(transactionData, callback, originalResponse, originalData, isSchedule, requestType)
 *
 * 请求字段：
 *   - transactionBytes：已签名交易字节（hex）
 *   - prunableAttachmentJSON：可修剪附件的 JSON（来自服务端 transactionJSON.attachment）
 *   - adminPassword：管理员密码（本地节点才需要）
 *
 * requestType 选择：
 *   - 调度请求：SCHEDULE_PREFIX + 首字母大写 + 剩余部分
 *   - API 代理：sendTransaction
 *   - 普通：broadcastTransaction
 *
 * @param transactionData 已签名交易字节（hex）
 * @param originalResponse 服务端原始响应（需含 transactionJSON.attachment）
 * @param originalData 原始表单数据
 * @param options 广播选项
 * @returns 广播结果 { broadcasted, transaction, fullHash } 或错误
 */
export async function broadcastTransactionBytes(
  transactionData: string,
  originalResponse: { transactionJSON?: { attachment?: any } },
  originalData: TransactionFormData,
  options: BroadcastOptions = {},
): Promise<BroadcastResult> {
  const {
    isSchedule = false,
    offerIssuer,
    adminPassword,
    apiProxy = false,
    requestType: origRequestType = '',
  } = options;

  // 构造请求体
  const payload: Record<string, any> = {
    transactionBytes: transactionData,
    prunableAttachmentJSON: JSON.stringify(originalResponse?.transactionJSON?.attachment || {}),
  };
  if (adminPassword) {
    payload.adminPassword = adminPassword;
  }

  // 选择 requestType
  let requestType: string;
  if (isSchedule) {
    requestType =
      SCHEDULE_PREFIX +
      origRequestType.substring(0, 1).toUpperCase() +
      origRequestType.substring(1);
    if (offerIssuer) {
      payload.offerIssuer = offerIssuer;
    }
  } else {
    requestType = apiProxy ? 'sendTransaction' : 'broadcastTransaction';
  }

  try {
    const response = await nrcsPost<any>(requestType, payload);

    if (response.errorCode) {
      return {
        broadcasted: false,
        errorCode: response.errorCode,
        errorDescription: response.errorDescription || response.errorMessage || '未知错误',
      };
    }
    if (response.error) {
      return {
        broadcasted: false,
        errorCode: 1,
        errorDescription: response.error,
      };
    }

    return {
      broadcasted: true,
      transaction: response.transaction,
      fullHash: response.fullHash,
    };
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    return {
      broadcasted: false,
      errorCode: -1,
      errorDescription: message,
    };
  }
}

// ============================================================================
// 三步签名流程编排
// ============================================================================

/**
 * 完整的本地三步签名流程：验证 → 签名 → 广播。
 *
 * 这是一个高层编排函数，封装了 verifyAndSignTransactionBytes + broadcastTransactionBytes，
 * 对应 nrs.server.js processAjaxRequest 中 isVolatile 分支的完整逻辑。
 *
 * 步骤：
 *   1. 对服务端返回的 unsignedTransactionBytes 进行字段校验
 *   2. 用 secretPhrase 本地签名并注入到字节中
 *   3. 根据是否需要广播，调用 broadcastTransactionBytes 或返回原始 payload
 *
 * @param unsignedTransactionBytes 服务端返回的未签名交易字节（hex）
 * @param requestType 请求类型
 * @param data 表单数据
 * @param response 服务端响应（需含 transactionJSON.attachment）
 * @param secretPhrase 本地密钥短语
 * @param options 校验与广播选项
 * @returns 广播结果或签名结果
 */
export async function signAndBroadcastTransaction(
  unsignedTransactionBytes: string,
  requestType: string,
  data: TransactionFormData,
  response: { transactionJSON?: { attachment?: any } },
  secretPhrase: string,
  options: VerifyOptions & BroadcastOptions & { broadcast?: boolean },
): Promise<BroadcastResult & { signature?: string; payload?: string }> {
  // 第一步：验证 + 签名
  const signResult = verifyAndSignTransactionBytes(
    unsignedTransactionBytes,
    requestType,
    data,
    response,
    secretPhrase,
    options,
  );

  if (!signResult.ok) {
    return {
      broadcasted: false,
      errorCode: signResult.errorCode,
      errorDescription: signResult.errorDescription,
    };
  }

  // 若不需要广播，直接返回签名结果（用于"不广播"模式 / 调试）
  if (options.broadcast === false) {
    return {
      broadcasted: false,
      payload: signResult.payload,
      signature: signResult.signature,
    };
  }

  // 第二步：广播
  const broadcastResult = await broadcastTransactionBytes(
    signResult.payload!,
    response,
    data,
    options,
  );

  return {
    ...broadcastResult,
    payload: signResult.payload,
    signature: signResult.signature,
  };
}
