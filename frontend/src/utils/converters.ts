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

import CryptoJS from 'crypto-js';
import { NrsAddress } from './nrs-address';

// ── Character-to-nibble lookup tables ────────────────────────────────────────

const charToNibble: Record<string, number> = {};
const nibbleToChar: string[] = [];

for (let i = 0; i <= 9; ++i) {
  const character = i.toString();
  charToNibble[character] = i;
  nibbleToChar.push(character);
}

for (let i = 10; i <= 15; ++i) {
  const lowerChar = String.fromCharCode('a'.charCodeAt(0) + i - 10);
  const upperChar = String.fromCharCode('A'.charCodeAt(0) + i - 10);

  charToNibble[lowerChar] = i;
  charToNibble[upperChar] = i;
  nibbleToChar.push(lowerChar);
}

// ── WordArray type (compatible with CryptoJS.lib.WordArray) ──────────────────

export interface WordArray {
  words: number[] | Uint32Array;
  sigBytes: number;
}

// ── Internal helpers ─────────────────────────────────────────────────────────

/**
 * Validates that a byte array has enough bytes from a given start index
 * to read an integer of `numBytes` length. Returns the resolved start index.
 */
function checkBytesToIntInput(
  bytes: Uint8Array,
  numBytes: number,
  opt_startIndex?: number,
): number {
  const startIndex = opt_startIndex ?? 0;
  if (startIndex < 0) {
    throw new Error('Start index should not be negative');
  }

  if (bytes.length < startIndex + numBytes) {
    throw new Error('Need at least ' + numBytes + ' bytes to convert to an integer');
  }
  return startIndex;
}

/**
 * Produces an array of the specified number of bytes to represent the integer
 * value. Default output encodes ints in little endian format. Handles signed
 * as well as unsigned integers. Due to limitations in JavaScript's number
 * format, x cannot be a true 64 bit integer (8 bytes).
 */
function intToBytes_(
  x: number,
  numBytes: number,
  unsignedMax: number,
  opt_bigEndian?: boolean,
): number[] {
  const signedMax = Math.floor(unsignedMax / 2);
  const negativeMax = (signedMax + 1) * -1;
  if (x !== Math.floor(x) || x < negativeMax || x > unsignedMax) {
    throw new Error(x + ' is not a ' + numBytes * 8 + ' bit integer');
  }
  const bytes: number[] = [];
  // Number type 0 is in the positive int range, 1 is larger than signed int,
  // and 2 is negative int.
  let numberType: 0 | 1 | 2;

  if (x >= 0 && x <= signedMax) {
    numberType = 0;
  } else if (x > signedMax && x <= unsignedMax) {
    numberType = 1;
  } else {
    numberType = 2;
  }

  if (numberType === 2) {
    x = x * -1 - 1;
  }

  let current: number;
  for (let i = 0; i < numBytes; i++) {
    if (numberType === 2) {
      current = 255 - (x % 256);
    } else {
      current = x % 256;
    }

    if (opt_bigEndian) {
      bytes.unshift(current);
    } else {
      bytes.push(current);
    }

    if (numberType === 1) {
      x = Math.floor(x / 256);
    } else {
      x = x >> 8;
    }
  }
  return bytes;
}

/**
 * Simple HTML escaping for display of account identifiers.
 */
function escapeHTML(str: string): string {
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
}

// ── Hexadecimal / byte-array conversions ─────────────────────────────────────

/**
 * Converts a Uint8Array to a hexadecimal string.
 */
export function byteArrayToHexString(bytes: Uint8Array): string {
  let str = '';
  for (let i = 0; i < bytes.length; ++i) {
    str += nibbleToChar[bytes[i] >> 4] + nibbleToChar[bytes[i] & 0x0f];
  }
  return str;
}

/**
 * Converts a hexadecimal string to a Uint8Array.
 * Handles odd-length hex strings by prepending a zero nibble.
 */
export function hexStringToByteArray(str: string): Uint8Array {
  const bytes: number[] = [];
  let i = 0;
  if (str.length % 2 !== 0) {
    bytes.push(charToNibble[str.charAt(0)]);
    ++i;
  }

  for (; i < str.length - 1; i += 2) {
    bytes.push((charToNibble[str.charAt(i)] << 4) + charToNibble[str.charAt(i + 1)]);
  }

  return new Uint8Array(bytes);
}

// ── String / byte-array conversions (UTF-8 via TextEncoder / TextDecoder) ────

const textEncoder = new TextEncoder();
const textDecoder = new TextDecoder();

/**
 * Encodes a string to a UTF-8 Uint8Array.
 */
export function stringToByteArray(str: string): Uint8Array {
  return textEncoder.encode(str);
}

/**
 * Decodes a Uint8Array to a UTF-8 string.
 * Optionally accepts a start index and length to slice the byte array.
 */
export function byteArrayToString(
  bytes: Uint8Array,
  opt_startIndex?: number,
  length?: number,
): string {
  if (length === 0) {
    return '';
  }

  if (opt_startIndex !== undefined && length !== undefined) {
    checkBytesToIntInput(bytes, length, opt_startIndex);
    bytes = bytes.slice(opt_startIndex, opt_startIndex + length);
  }

  return textDecoder.decode(bytes);
}

/**
 * Converts a string to a hexadecimal string.
 */
export function stringToHexString(str: string): string {
  return byteArrayToHexString(stringToByteArray(str));
}

/**
 * Converts a hexadecimal string back to a UTF-8 string.
 */
export function hexStringToString(hex: string): string {
  return byteArrayToString(hexStringToByteArray(hex));
}

// ── Integer decoding (little-endian) ─────────────────────────────────────────

/**
 * Reads a signed 16-bit integer (little-endian) from a byte array,
 * optionally starting at `opt_startIndex`.
 */
export function byteArrayToSignedShort(
  bytes: Uint8Array,
  opt_startIndex?: number,
): number {
  const index = checkBytesToIntInput(bytes, 2, opt_startIndex);
  let value = bytes[index];
  value += bytes[index + 1] << 8;
  return value;
}

/**
 * Reads a signed 32-bit integer (little-endian) from a byte array,
 * optionally starting at `opt_startIndex`.
 */
export function byteArrayToSignedInt32(
  bytes: Uint8Array,
  opt_startIndex?: number,
): number {
  const index = checkBytesToIntInput(bytes, 4, opt_startIndex);
  let value = bytes[index];
  value += bytes[index + 1] << 8;
  value += bytes[index + 2] << 16;
  value += bytes[index + 3] << 24;
  return value;
}

/**
 * Reads an 8-byte little-endian unsigned integer from a byte array
 * and returns it as a native BigInt.
 */
export function byteArrayToBigInteger(
  bytes: Uint8Array,
  opt_startIndex?: number,
): bigint {
  const index = checkBytesToIntInput(bytes, 8, opt_startIndex);

  let value = 0n;
  for (let i = 7; i >= 0; i--) {
    value = (value << 8n) | BigInt(bytes[index + i]);
  }
  return value;
}

// ── Integer encoding ─────────────────────────────────────────────────────────

/**
 * Converts a signed 32-bit integer to a Uint8Array (little-endian by default).
 * Pass `opt_bigEndian = true` for big-endian output.
 */
export function int32ToBytes(x: number, opt_bigEndian?: boolean): Uint8Array {
  return new Uint8Array(intToBytes_(x, 4, 4294967295, opt_bigEndian));
}

// ── CryptoJS Big-Endian WordArray conversions ────────────────────────────────

/**
 * Creates a Big-Endian WordArray from a Uint8Array.
 */
export function byteArrayToWordArray(byteArray: Uint8Array): WordArray {
  let i = 0;
  let offset = 0;
  let word = 0;
  const len = byteArray.length;
  const words = new Uint32Array(((len / 4) | 0) + (len % 4 === 0 ? 0 : 1));

  while (i < len - (len % 4)) {
    words[offset++] =
      (byteArray[i++] << 24) |
      (byteArray[i++] << 16) |
      (byteArray[i++] << 8) |
      byteArray[i++];
  }
  if (len % 4 !== 0) {
    word = byteArray[i++] << 24;
    if (len % 4 > 1) {
      word = word | (byteArray[i++] << 16);
    }
    if (len % 4 > 2) {
      word = word | (byteArray[i++] << 8);
    }
    words[offset] = word;
  }

  return {
    words,
    sigBytes: len,
  };
}

/**
 * Internal implementation for converting Big-Endian WordArray to byte array.
 * @param isFirstByteHasSign When true, the first byte is treated as signed
 *   (arithmetic shift); when false, it is masked to 8 bits.
 */
function wordArrayToByteArrayImpl(
  wordArray: WordArray,
  isFirstByteHasSign: boolean,
): number[] {
  const len = wordArray.words.length;
  if (len === 0) {
    return [];
  }
  const byteArray: number[] = new Array(wordArray.sigBytes);
  let offset = 0;
  let word: number;
  let i: number;
  for (i = 0; i < len - 1; i++) {
    word = wordArray.words[i];
    byteArray[offset++] = isFirstByteHasSign ? word >> 24 : (word >> 24) & 0xff;
    byteArray[offset++] = (word >> 16) & 0xff;
    byteArray[offset++] = (word >> 8) & 0xff;
    byteArray[offset++] = word & 0xff;
  }
  word = wordArray.words[len - 1];
  byteArray[offset++] = isFirstByteHasSign ? word >> 24 : (word >> 24) & 0xff;
  if (wordArray.sigBytes % 4 === 0) {
    byteArray[offset++] = (word >> 16) & 0xff;
    byteArray[offset++] = (word >> 8) & 0xff;
    byteArray[offset++] = word & 0xff;
  }
  if (wordArray.sigBytes % 4 > 1) {
    byteArray[offset++] = (word >> 16) & 0xff;
  }
  if (wordArray.sigBytes % 4 > 2) {
    byteArray[offset++] = (word >> 8) & 0xff;
  }
  return byteArray;
}

/**
 * Converts a Big-Endian WordArray back to a Uint8Array.
 */
export function wordArrayToByteArray(wordArray: WordArray): Uint8Array {
  return new Uint8Array(wordArrayToByteArrayImpl(wordArray, true));
}

// ── Short-array conversions (Curve25519) ─────────────────────────────────────

/**
 * Converts a 32-byte Uint8Array to a 16-element short array (little-endian).
 */
export function byteArrayToShortArray(byteArray: Uint8Array): number[] {
  const shortArray: number[] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
  for (let i = 0; i < 16; i++) {
    shortArray[i] = byteArray[i * 2] | (byteArray[i * 2 + 1] << 8);
  }
  return shortArray;
}

/**
 * Converts a 16-element short array to a 32-byte Uint8Array (little-endian).
 */
export function shortArrayToByteArray(shortArray: number[]): Uint8Array {
  const byteArray = new Uint8Array(32);
  for (let i = 0; i < 16; i++) {
    byteArray[2 * i] = shortArray[i] & 0xff;
    byteArray[2 * i + 1] = shortArray[i] >> 8;
  }
  return byteArray;
}

/**
 * Converts a short array to a hexadecimal string.
 */
export function shortArrayToHexString(ary: number[]): string {
  let res = '';
  for (let i = 0; i < ary.length; i++) {
    res +=
      nibbleToChar[(ary[i] >> 4) & 0x0f] +
      nibbleToChar[ary[i] & 0x0f] +
      nibbleToChar[(ary[i] >> 12) & 0x0f] +
      nibbleToChar[(ary[i] >> 8) & 0x0f];
  }
  return res;
}

// ── Uint8Array <-> CryptoJS WordArray (WordArrayEx) ──────────────────────────

/**
 * Converts a CryptoJS WordArray to a Uint8Array.
 */
export function wordArrayToByteArrayEx(wordArray: WordArray): Uint8Array {
  const words = wordArray.words;
  const sigBytes = wordArray.sigBytes;

  const u8 = new Uint8Array(sigBytes);
  for (let i = 0; i < sigBytes; i++) {
    const byte = (words[i >>> 2] >>> (24 - (i % 4) * 8)) & 0xff;
    u8[i] = byte;
  }

  return u8;
}

/**
 * Converts a Uint8Array to a CryptoJS WordArray.
 */
export function byteArrayToWordArrayEx(u8arr: Uint8Array): WordArray {
  const len = u8arr.length;

  const words: number[] = [];
  for (let i = 0; i < len; i++) {
    const wordIndex = i >>> 2;
    const wordShift = 24 - (i % 4) * 8;
    // >>> 0 converts to unsigned 32-bit, matching the JS converters.byteArrayToWordArray
    // which uses Uint32Array. Without this, bytes > 127 produce negative signed
    // values via << 24, causing CryptoJS SHA-256 to compute incorrect hashes.
    words[wordIndex] = ((words[wordIndex] || 0) | ((u8arr[i] & 0xff) << wordShift)) >>> 0;
  }

  return CryptoJS.lib.WordArray.create(words, len) as unknown as WordArray;
}

// ── NRCS address formatting ─────────────────────────────────────────────────

/**
 * Converts a numeric account ID to NRCS-Reed-Solomon address format.
 * If the input is already in NRCS-… format, it is HTML-escaped and returned.
 * Otherwise the NrsAddress class is used to convert and format the account.
 */
export function convertNumericToRSAccountFormat(account: string): string {
  if (/^NRCS\-/i.test(account)) {
    return escapeHTML(String(account));
  } else {
    const address = new NrsAddress();

    if (address.set(account)) {
      return escapeHTML(address.toString());
    } else {
      return '';
    }
  }
}
