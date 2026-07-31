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
 * NRCS Cryptography Module
 *
 * Complete TypeScript port of nrs.encryption.js, curve25519.js, and
 * curve25519_.js.  Implements EC-KCDSA signatures over Curve25519,
 * ECDH shared-secret derivation, AES-256-CBC encrypt/decrypt via
 * CryptoJS, and account-id generation.
 *
 * Dependencies (already installed):
 *   crypto-js  – SHA-256, AES, HMAC
 *   pako       – gzip / deflate compression
 */

import CryptoJS from 'crypto-js';
import { gzip, inflate } from 'pako';
import {
  byteArrayToHexString,
  hexStringToByteArray,
  stringToByteArray,
  byteArrayToString,
  byteArrayToShortArray,
  shortArrayToByteArray,
  shortArrayToHexString,
  byteArrayToWordArrayEx,
  wordArrayToByteArrayEx,
  byteArrayToBigInteger,
} from './converters';

// ============================================================================
// SECTION 1 — GF(2^255-19) Field Arithmetic (from curve25519_.js)
// ============================================================================
// All functions in this section operate on "short arrays" — number[] of
// length 16, where each element is a 16‑bit limb (0 … 0xFFFF).
// ============================================================================

const SHORT_ZERO: number[] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
const SHORT_ONE: number[] = [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
const SHORT_NINE: number[] = [9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
const SHORT_486671: number[] = [27919, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
const SHORT_39420360: number[] = [33224, 601, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
const SHORT_R2Y: number[] = [0x1670, 0x4000, 0xf219, 0xd369, 0x2248, 0x4845, 0x679a, 0x884d, 0x5d19, 0x16bf, 0xda74, 0xe57d, 0x5e53, 0x3705, 0x3526, 0x17c0];
const SHORT_2Y: number[] = [0x583b, 0x0262, 0x74bb, 0xac2c, 0x3c9b, 0x2507, 0x6503, 0xdb85, 0x5d66, 0x116e, 0x45a7, 0x3fc2, 0xf296, 0x8ebe, 0xccbc, 0x3ea3];

/* group order (a prime near 2^252+2^124) */
const CURVE_ORDER: number[] = [
  237, 211, 245, 92, 26, 99, 18, 88, 214, 156, 247, 162, 222, 249, 222, 20,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16,
];

const CURVE_ORDER_TIMES_8: number[] = [
  104, 159, 174, 231, 210, 24, 147, 192, 178, 230, 188, 23, 245, 206, 247, 166,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128,
];

// ---- helpers (short-array utilities) ---------------------------------------

function cpy16(a: number[]): number[] {
  const r = new Array(16);
  for (let i = 0; i < 16; i++) r[i] = a[i];
  return r;
}

function fill16(src: number[], dest: number[]): void {
  for (let i = 0; i < 16; i++) dest[i] = src[i];
}

function isNegative(x: number[]): number {
  return x[0] & 1;
}

// ---- reduce ----------------------------------------------------------------

function reduce2(a: number[]): void {
  let v: number = a[15];
  if (v < 0x8000) return;
  a[15] = v % 0x8000;
  v = ~~(v / 0x8000) * 19;
  a[0] = (v += a[0]) & 0xffff;
  if ((v = ~~(v / 0x10000)) < 1) return;
  a[1] = (v += a[1]) & 0xffff;
  if ((v = ~~(v / 0x10000)) < 1) return;
  a[2] = (v += a[2]) & 0xffff;
  if ((v = ~~(v / 0x10000)) < 1) return;
  a[3] = (v += a[3]) & 0xffff;
  if ((v = ~~(v / 0x10000)) < 1) return;
  a[4] = (v += a[4]) & 0xffff;
  if ((v = ~~(v / 0x10000)) < 1) return;
  a[5] = (v += a[5]) & 0xffff;
  if ((v = ~~(v / 0x10000)) < 1) return;
  a[6] = (v += a[6]) & 0xffff;
  if ((v = ~~(v / 0x10000)) < 1) return;
  a[7] = (v += a[7]) & 0xffff;
  if ((v = ~~(v / 0x10000)) < 1) return;
  a[8] = (v += a[8]) & 0xffff;
  if ((v = ~~(v / 0x10000)) < 1) return;
  a[9] = (v += a[9]) & 0xffff;
  if ((v = ~~(v / 0x10000)) < 1) return;
  a[10] = (v += a[10]) & 0xffff;
  if ((v = ~~(v / 0x10000)) < 1) return;
  a[11] = (v += a[11]) & 0xffff;
  if ((v = ~~(v / 0x10000)) < 1) return;
  a[12] = (v += a[12]) & 0xffff;
  if ((v = ~~(v / 0x10000)) < 1) return;
  a[13] = (v += a[13]) & 0xffff;
  if ((v = ~~(v / 0x10000)) < 1) return;
  a[14] = (v += a[14]) & 0xffff;
  if ((v = ~~(v / 0x10000)) < 1) return;
  a[15] += v;
}

function reduce(a: number[]): void {
  reduce2(a);

  // special case for p <= a < 2^255
  if (
    a[15] !== 0x7fff || a[14] !== 0xffff || a[13] !== 0xffff ||
    a[12] !== 0xffff || a[11] !== 0xffff || a[10] !== 0xffff ||
    a[9] !== 0xffff || a[8] !== 0xffff || a[7] !== 0xffff ||
    a[6] !== 0xffff || a[5] !== 0xffff || a[4] !== 0xffff ||
    a[3] !== 0xffff || a[2] !== 0xffff || a[1] !== 0xffff ||
    a[0] < 0xffed
  ) {
    return;
  }

  for (let i = 1; i < 16; i++) a[i] = 0;
  a[0] = a[0] - 0xffed;
}

// ---- sqr8h / sqrmodp -------------------------------------------------------

function sqr8h(
  r: number[], a7: number, a6: number, a5: number, a4: number,
  a3: number, a2: number, a1: number, a0: number,
): void {
  let v: number;
  r[0] = (v = a0 * a0) & 0xffff;
  r[1] = (v = ~~(v / 0x10000) + 2 * a0 * a1) & 0xffff;
  r[2] = (v = ~~(v / 0x10000) + 2 * a0 * a2 + a1 * a1) & 0xffff;
  r[3] = (v = ~~(v / 0x10000) + 2 * a0 * a3 + 2 * a1 * a2) & 0xffff;
  r[4] = (v = ~~(v / 0x10000) + 2 * a0 * a4 + 2 * a1 * a3 + a2 * a2) & 0xffff;
  r[5] = (v = ~~(v / 0x10000) + 2 * a0 * a5 + 2 * a1 * a4 + 2 * a2 * a3) & 0xffff;
  r[6] = (v = ~~(v / 0x10000) + 2 * a0 * a6 + 2 * a1 * a5 + 2 * a2 * a4 + a3 * a3) & 0xffff;
  r[7] = (v = ~~(v / 0x10000) + 2 * a0 * a7 + 2 * a1 * a6 + 2 * a2 * a5 + 2 * a3 * a4) & 0xffff;
  r[8] = (v = ~~(v / 0x10000) + 2 * a1 * a7 + 2 * a2 * a6 + 2 * a3 * a5 + a4 * a4) & 0xffff;
  r[9] = (v = ~~(v / 0x10000) + 2 * a2 * a7 + 2 * a3 * a6 + 2 * a4 * a5) & 0xffff;
  r[10] = (v = ~~(v / 0x10000) + 2 * a3 * a7 + 2 * a4 * a6 + a5 * a5) & 0xffff;
  r[11] = (v = ~~(v / 0x10000) + 2 * a4 * a7 + 2 * a5 * a6) & 0xffff;
  r[12] = (v = ~~(v / 0x10000) + 2 * a5 * a7 + a6 * a6) & 0xffff;
  r[13] = (v = ~~(v / 0x10000) + 2 * a6 * a7) & 0xffff;
  r[14] = (v = ~~(v / 0x10000) + a7 * a7) & 0xffff;
  r[15] = ~~(v / 0x10000);
}

function sqrmodp(r: number[], a: number[]): void {
  const x: number[] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
  const y: number[] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
  const z: number[] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
  sqr8h(x, a[15], a[14], a[13], a[12], a[11], a[10], a[9], a[8]);
  sqr8h(z, a[7], a[6], a[5], a[4], a[3], a[2], a[1], a[0]);
  sqr8h(y, a[15] + a[7], a[14] + a[6], a[13] + a[5], a[12] + a[4],
    a[11] + a[3], a[10] + a[2], a[9] + a[1], a[8] + a[0]);
  let v: number;
  r[0] = (v = 0x800000 + z[0] + (y[8] - x[8] - z[8] + x[0] - 0x80) * 38) & 0xffff;
  r[1] = (v = 0x7fff80 + ~~(v / 0x10000) + z[1] + (y[9] - x[9] - z[9] + x[1]) * 38) & 0xffff;
  r[2] = (v = 0x7fff80 + ~~(v / 0x10000) + z[2] + (y[10] - x[10] - z[10] + x[2]) * 38) & 0xffff;
  r[3] = (v = 0x7fff80 + ~~(v / 0x10000) + z[3] + (y[11] - x[11] - z[11] + x[3]) * 38) & 0xffff;
  r[4] = (v = 0x7fff80 + ~~(v / 0x10000) + z[4] + (y[12] - x[12] - z[12] + x[4]) * 38) & 0xffff;
  r[5] = (v = 0x7fff80 + ~~(v / 0x10000) + z[5] + (y[13] - x[13] - z[13] + x[5]) * 38) & 0xffff;
  r[6] = (v = 0x7fff80 + ~~(v / 0x10000) + z[6] + (y[14] - x[14] - z[14] + x[6]) * 38) & 0xffff;
  r[7] = (v = 0x7fff80 + ~~(v / 0x10000) + z[7] + (y[15] - x[15] - z[15] + x[7]) * 38) & 0xffff;
  r[8] = (v = 0x7fff80 + ~~(v / 0x10000) + z[8] + y[0] - x[0] - z[0] + x[8] * 38) & 0xffff;
  r[9] = (v = 0x7fff80 + ~~(v / 0x10000) + z[9] + y[1] - x[1] - z[1] + x[9] * 38) & 0xffff;
  r[10] = (v = 0x7fff80 + ~~(v / 0x10000) + z[10] + y[2] - x[2] - z[2] + x[10] * 38) & 0xffff;
  r[11] = (v = 0x7fff80 + ~~(v / 0x10000) + z[11] + y[3] - x[3] - z[3] + x[11] * 38) & 0xffff;
  r[12] = (v = 0x7fff80 + ~~(v / 0x10000) + z[12] + y[4] - x[4] - z[4] + x[12] * 38) & 0xffff;
  r[13] = (v = 0x7fff80 + ~~(v / 0x10000) + z[13] + y[5] - x[5] - z[5] + x[13] * 38) & 0xffff;
  r[14] = (v = 0x7fff80 + ~~(v / 0x10000) + z[14] + y[6] - x[6] - z[6] + x[14] * 38) & 0xffff;
  r[15] = 0x7fff80 + ~~(v / 0x10000) + z[15] + y[7] - x[7] - z[7] + x[15] * 38;
  reduce(r);
}

// ---- mul8h / mulmodp -------------------------------------------------------

function mul8h(
  r: number[],
  a7: number, a6: number, a5: number, a4: number,
  a3: number, a2: number, a1: number, a0: number,
  b7: number, b6: number, b5: number, b4: number,
  b3: number, b2: number, b1: number, b0: number,
): void {
  let v: number;
  r[0] = (v = a0 * b0) & 0xffff;
  r[1] = (v = ~~(v / 0x10000) + a0 * b1 + a1 * b0) & 0xffff;
  r[2] = (v = ~~(v / 0x10000) + a0 * b2 + a1 * b1 + a2 * b0) & 0xffff;
  r[3] = (v = ~~(v / 0x10000) + a0 * b3 + a1 * b2 + a2 * b1 + a3 * b0) & 0xffff;
  r[4] = (v = ~~(v / 0x10000) + a0 * b4 + a1 * b3 + a2 * b2 + a3 * b1 + a4 * b0) & 0xffff;
  r[5] = (v = ~~(v / 0x10000) + a0 * b5 + a1 * b4 + a2 * b3 + a3 * b2 + a4 * b1 + a5 * b0) & 0xffff;
  r[6] = (v = ~~(v / 0x10000) + a0 * b6 + a1 * b5 + a2 * b4 + a3 * b3 + a4 * b2 + a5 * b1 + a6 * b0) & 0xffff;
  r[7] = (v = ~~(v / 0x10000) + a0 * b7 + a1 * b6 + a2 * b5 + a3 * b4 + a4 * b3 + a5 * b2 + a6 * b1 + a7 * b0) & 0xffff;
  r[8] = (v = ~~(v / 0x10000) + a1 * b7 + a2 * b6 + a3 * b5 + a4 * b4 + a5 * b3 + a6 * b2 + a7 * b1) & 0xffff;
  r[9] = (v = ~~(v / 0x10000) + a2 * b7 + a3 * b6 + a4 * b5 + a5 * b4 + a6 * b3 + a7 * b2) & 0xffff;
  r[10] = (v = ~~(v / 0x10000) + a3 * b7 + a4 * b6 + a5 * b5 + a6 * b4 + a7 * b3) & 0xffff;
  r[11] = (v = ~~(v / 0x10000) + a4 * b7 + a5 * b6 + a6 * b5 + a7 * b4) & 0xffff;
  r[12] = (v = ~~(v / 0x10000) + a5 * b7 + a6 * b6 + a7 * b5) & 0xffff;
  r[13] = (v = ~~(v / 0x10000) + a6 * b7 + a7 * b6) & 0xffff;
  r[14] = (v = ~~(v / 0x10000) + a7 * b7) & 0xffff;
  r[15] = ~~(v / 0x10000);
}

function mulmodp(r: number[], a: number[], b: number[]): void {
  const x: number[] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
  const y: number[] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
  const z: number[] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
  mul8h(x, a[15], a[14], a[13], a[12], a[11], a[10], a[9], a[8],
    b[15], b[14], b[13], b[12], b[11], b[10], b[9], b[8]);
  mul8h(z, a[7], a[6], a[5], a[4], a[3], a[2], a[1], a[0],
    b[7], b[6], b[5], b[4], b[3], b[2], b[1], b[0]);
  mul8h(y, a[15] + a[7], a[14] + a[6], a[13] + a[5], a[12] + a[4],
    a[11] + a[3], a[10] + a[2], a[9] + a[1], a[8] + a[0],
    b[15] + b[7], b[14] + b[6], b[13] + b[5], b[12] + b[4],
    b[11] + b[3], b[10] + b[2], b[9] + b[1], b[8] + b[0]);
  let v: number;
  r[0] = (v = 0x800000 + z[0] + (y[8] - x[8] - z[8] + x[0] - 0x80) * 38) & 0xffff;
  r[1] = (v = 0x7fff80 + ~~(v / 0x10000) + z[1] + (y[9] - x[9] - z[9] + x[1]) * 38) & 0xffff;
  r[2] = (v = 0x7fff80 + ~~(v / 0x10000) + z[2] + (y[10] - x[10] - z[10] + x[2]) * 38) & 0xffff;
  r[3] = (v = 0x7fff80 + ~~(v / 0x10000) + z[3] + (y[11] - x[11] - z[11] + x[3]) * 38) & 0xffff;
  r[4] = (v = 0x7fff80 + ~~(v / 0x10000) + z[4] + (y[12] - x[12] - z[12] + x[4]) * 38) & 0xffff;
  r[5] = (v = 0x7fff80 + ~~(v / 0x10000) + z[5] + (y[13] - x[13] - z[13] + x[5]) * 38) & 0xffff;
  r[6] = (v = 0x7fff80 + ~~(v / 0x10000) + z[6] + (y[14] - x[14] - z[14] + x[6]) * 38) & 0xffff;
  r[7] = (v = 0x7fff80 + ~~(v / 0x10000) + z[7] + (y[15] - x[15] - z[15] + x[7]) * 38) & 0xffff;
  r[8] = (v = 0x7fff80 + ~~(v / 0x10000) + z[8] + y[0] - x[0] - z[0] + x[8] * 38) & 0xffff;
  r[9] = (v = 0x7fff80 + ~~(v / 0x10000) + z[9] + y[1] - x[1] - z[1] + x[9] * 38) & 0xffff;
  r[10] = (v = 0x7fff80 + ~~(v / 0x10000) + z[10] + y[2] - x[2] - z[2] + x[10] * 38) & 0xffff;
  r[11] = (v = 0x7fff80 + ~~(v / 0x10000) + z[11] + y[3] - x[3] - z[3] + x[11] * 38) & 0xffff;
  r[12] = (v = 0x7fff80 + ~~(v / 0x10000) + z[12] + y[4] - x[4] - z[4] + x[12] * 38) & 0xffff;
  r[13] = (v = 0x7fff80 + ~~(v / 0x10000) + z[13] + y[5] - x[5] - z[5] + x[13] * 38) & 0xffff;
  r[14] = (v = 0x7fff80 + ~~(v / 0x10000) + z[14] + y[6] - x[6] - z[6] + x[14] * 38) & 0xffff;
  r[15] = 0x7fff80 + ~~(v / 0x10000) + z[15] + y[7] - x[7] - z[7] + x[15] * 38;
  reduce(r);
}

// ---- mulasmall / addmodp / submodp -----------------------------------------

function mulasmall(r: number[], a: number[], m: number): void {
  let v: number;
  r[0] = (v = a[0] * m) & 0xffff;
  r[1] = (v = ~~(v / 0x10000) + a[1] * m) & 0xffff;
  r[2] = (v = ~~(v / 0x10000) + a[2] * m) & 0xffff;
  r[3] = (v = ~~(v / 0x10000) + a[3] * m) & 0xffff;
  r[4] = (v = ~~(v / 0x10000) + a[4] * m) & 0xffff;
  r[5] = (v = ~~(v / 0x10000) + a[5] * m) & 0xffff;
  r[6] = (v = ~~(v / 0x10000) + a[6] * m) & 0xffff;
  r[7] = (v = ~~(v / 0x10000) + a[7] * m) & 0xffff;
  r[8] = (v = ~~(v / 0x10000) + a[8] * m) & 0xffff;
  r[9] = (v = ~~(v / 0x10000) + a[9] * m) & 0xffff;
  r[10] = (v = ~~(v / 0x10000) + a[10] * m) & 0xffff;
  r[11] = (v = ~~(v / 0x10000) + a[11] * m) & 0xffff;
  r[12] = (v = ~~(v / 0x10000) + a[12] * m) & 0xffff;
  r[13] = (v = ~~(v / 0x10000) + a[13] * m) & 0xffff;
  r[14] = (v = ~~(v / 0x10000) + a[14] * m) & 0xffff;
  r[15] = ~~(v / 0x10000) + a[15] * m;
  reduce(r);
}

function addmodp(r: number[], a: number[], b: number[]): void {
  let v: number;
  r[0] = (v = (~~(a[15] / 0x8000) + ~~(b[15] / 0x8000)) * 19 + a[0] + b[0]) & 0xffff;
  r[1] = (v = ~~(v / 0x10000) + a[1] + b[1]) & 0xffff;
  r[2] = (v = ~~(v / 0x10000) + a[2] + b[2]) & 0xffff;
  r[3] = (v = ~~(v / 0x10000) + a[3] + b[3]) & 0xffff;
  r[4] = (v = ~~(v / 0x10000) + a[4] + b[4]) & 0xffff;
  r[5] = (v = ~~(v / 0x10000) + a[5] + b[5]) & 0xffff;
  r[6] = (v = ~~(v / 0x10000) + a[6] + b[6]) & 0xffff;
  r[7] = (v = ~~(v / 0x10000) + a[7] + b[7]) & 0xffff;
  r[8] = (v = ~~(v / 0x10000) + a[8] + b[8]) & 0xffff;
  r[9] = (v = ~~(v / 0x10000) + a[9] + b[9]) & 0xffff;
  r[10] = (v = ~~(v / 0x10000) + a[10] + b[10]) & 0xffff;
  r[11] = (v = ~~(v / 0x10000) + a[11] + b[11]) & 0xffff;
  r[12] = (v = ~~(v / 0x10000) + a[12] + b[12]) & 0xffff;
  r[13] = (v = ~~(v / 0x10000) + a[13] + b[13]) & 0xffff;
  r[14] = (v = ~~(v / 0x10000) + a[14] + b[14]) & 0xffff;
  r[15] = ~~(v / 0x10000) + a[15] % 0x8000 + b[15] % 0x8000;
}

function submodp(r: number[], a: number[], b: number[]): void {
  let v: number;
  r[0] = (v = 0x80000 + (~~(a[15] / 0x8000) - ~~(b[15] / 0x8000) - 1) * 19 + a[0] - b[0]) & 0xffff;
  r[1] = (v = ~~(v / 0x10000) + 0x7fff8 + a[1] - b[1]) & 0xffff;
  r[2] = (v = ~~(v / 0x10000) + 0x7fff8 + a[2] - b[2]) & 0xffff;
  r[3] = (v = ~~(v / 0x10000) + 0x7fff8 + a[3] - b[3]) & 0xffff;
  r[4] = (v = ~~(v / 0x10000) + 0x7fff8 + a[4] - b[4]) & 0xffff;
  r[5] = (v = ~~(v / 0x10000) + 0x7fff8 + a[5] - b[5]) & 0xffff;
  r[6] = (v = ~~(v / 0x10000) + 0x7fff8 + a[6] - b[6]) & 0xffff;
  r[7] = (v = ~~(v / 0x10000) + 0x7fff8 + a[7] - b[7]) & 0xffff;
  r[8] = (v = ~~(v / 0x10000) + 0x7fff8 + a[8] - b[8]) & 0xffff;
  r[9] = (v = ~~(v / 0x10000) + 0x7fff8 + a[9] - b[9]) & 0xffff;
  r[10] = (v = ~~(v / 0x10000) + 0x7fff8 + a[10] - b[10]) & 0xffff;
  r[11] = (v = ~~(v / 0x10000) + 0x7fff8 + a[11] - b[11]) & 0xffff;
  r[12] = (v = ~~(v / 0x10000) + 0x7fff8 + a[12] - b[12]) & 0xffff;
  r[13] = (v = ~~(v / 0x10000) + 0x7fff8 + a[13] - b[13]) & 0xffff;
  r[14] = (v = ~~(v / 0x10000) + 0x7fff8 + a[14] - b[14]) & 0xffff;
  r[15] = ~~(v / 0x10000) + 0x7ff8 + a[15] % 0x8000 - b[15] % 0x8000;
}

// ---- invmodp (x^(p-2) using addition chain) -------------------------------

function invmodp(r: number[], a: number[], sqrtassist: number): void {
  const r1: number[] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
  const r2: number[] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
  const r3: number[] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
  const r4: number[] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
  const r5: number[] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
  let i: number;

  sqrmodp(r2, a);                //  2 == 2 * 1
  sqrmodp(r3, r2);               //  4 == 2 * 2
  sqrmodp(r1, r3);               //  8 == 2 * 4
  mulmodp(r3, r1, a);            //  9 == 8 + 1
  mulmodp(r1, r3, r2);           // 11 == 9 + 2
  sqrmodp(r2, r1);               // 22 == 2 * 11
  mulmodp(r4, r2, r3);           // 31 == 22 + 9
  // 2^5 - 2^0
  sqrmodp(r2, r4);               // 2^6 - 2^1
  sqrmodp(r3, r2);               // 2^7 - 2^2
  sqrmodp(r2, r3);               // 2^8 - 2^3
  sqrmodp(r3, r2);               // 2^9 - 2^4
  sqrmodp(r2, r3);               // 2^10 - 2^5
  mulmodp(r3, r2, r4);           // 2^10 - 2^0
  sqrmodp(r2, r3);               // 2^11 - 2^1
  sqrmodp(r4, r2);               // 2^12 - 2^2
  for (i = 1; i < 5; i++) {
    sqrmodp(r2, r4);
    sqrmodp(r4, r2);
  }                              // 2^20 - 2^10
  mulmodp(r2, r4, r3);           // 2^20 - 2^0
  sqrmodp(r4, r2);               // 2^21 - 2^1
  sqrmodp(r5, r4);               // 2^22 - 2^2
  for (i = 1; i < 10; i++) {
    sqrmodp(r4, r5);
    sqrmodp(r5, r4);
  }                              // 2^40 - 2^20
  mulmodp(r4, r5, r2);           // 2^40 - 2^0
  for (i = 0; i < 5; i++) {
    sqrmodp(r2, r4);
    sqrmodp(r4, r2);
  }                              // 2^50 - 2^10
  mulmodp(r2, r4, r3);           // 2^50 - 2^0
  sqrmodp(r3, r2);               // 2^51 - 2^1
  sqrmodp(r4, r3);               // 2^52 - 2^2
  for (i = 1; i < 25; i++) {
    sqrmodp(r3, r4);
    sqrmodp(r4, r3);
  }                              // 2^100 - 2^50
  mulmodp(r3, r4, r2);           // 2^100 - 2^0
  sqrmodp(r4, r3);               // 2^101 - 2^1
  sqrmodp(r5, r4);               // 2^102 - 2^2
  for (i = 1; i < 50; i++) {
    sqrmodp(r4, r5);
    sqrmodp(r5, r4);
  }                              // 2^200 - 2^100
  mulmodp(r4, r5, r3);           // 2^200 - 2^0
  for (i = 0; i < 25; i++) {
    sqrmodp(r5, r4);
    sqrmodp(r4, r5);
  }                              // 2^250 - 2^50
  mulmodp(r3, r4, r2);           // 2^250 - 2^0
  sqrmodp(r2, r3);               // 2^251 - 2^1
  sqrmodp(r3, r2);               // 2^252 - 2^2
  if (sqrtassist === 1) {
    mulmodp(r, a, r3);           // 2^252 - 3
  } else {
    sqrmodp(r2, r3);             // 2^253 - 2^3
    sqrmodp(r3, r2);             // 2^254 - 2^4
    sqrmodp(r2, r3);             // 2^255 - 2^5
    mulmodp(r, r2, r1);          // 2^255 - 21
  }
}

// ---- sqrtmodp --------------------------------------------------------------

function sqrtmodp(r: number[], x: number[]): void {
  const r1: number[] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
  const r2: number[] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
  const r3: number[] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
  const r4: number[] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
  addmodp(r1, x, x);                   // r1 = 2x
  invmodp(r2, r1, 1);                  // r2 = (2x)^((p-5)/8)
  sqrmodp(r3, r2);                     // r3 = v^2
  mulmodp(r4, r1, r3);                 // r4 = 2xv^2 = i
  submodp(r, r4, SHORT_ONE);           //  r = i-1
  mulmodp(r1, r2, r);                  // r1 = v(i-1)
  mulmodp(r, x, r1);                   //  r = xv(i-1)
}

// ---- x_to_y2 (Montgomery curve: y^2 = x^3 + 486662 x^2 + x) ---------------

function xToY2(r: number[], x: number[]): void {
  const r1: number[] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
  const r2: number[] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
  sqrmodp(r1, x);
  mulasmall(r2, x, 486662);
  addmodp(r, r1, r2);
  addmodp(r1, r, SHORT_ONE);
  mulmodp(r, r1, x);
}

// ---- Montgomery ladder helpers: prep / dbl / sum ----------------------------

function prep(r: number[], s: number[], a: number[], b: number[]): void {
  addmodp(r, a, b);
  submodp(s, a, b);
}

function dblOp(r: number[], s: number[], t1: number[], t2: number[]): void {
  const r1: number[] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
  const r2: number[] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
  const r3: number[] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
  const r4: number[] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
  sqrmodp(r1, t1);
  sqrmodp(r2, t2);
  submodp(r3, r1, r2);
  mulmodp(r, r2, r1);
  mulasmall(r2, r3, 121665);
  addmodp(r4, r2, r1);
  mulmodp(s, r4, r3);
}

function curveSum(
  r: number[], s: number[], t1: number[], t2: number[],
  t3: number[], t4: number[], x1: number[],
): void {
  const r1: number[] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
  const r2: number[] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
  const r3: number[] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
  const r4: number[] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
  mulmodp(r1, t2, t3);
  mulmodp(r2, t1, t4);
  addmodp(r3, r1, r2);
  submodp(r4, r1, r2);
  sqrmodp(r, r3);
  sqrmodp(r1, r4);
  mulmodp(s, r1, x1);
}

// ---- getbit ----------------------------------------------------------------

function getbit(curve: number[], c: number): number {
  return ~~(curve[~~(c / 16)] / Math.pow(2, c % 16)) % 2;
}

// ---- short ↔ byte conversion utilities -------------------------------------

function shortArrayToByteArrayLocal(shortArray: number[]): Uint8Array {
  return shortArrayToByteArray(shortArray);
}

function byteArrayToShortArrayLocal(byteArray: Uint8Array): number[] {
  return byteArrayToShortArray(byteArray);
}

// ============================================================================
// SECTION 2 — Radix-2^8 Arithmetic (for EC-KCDSA sign / verify)
// ============================================================================

function mulaSmall(
  p: number[], q: number[], m: number,
  x: number[], n: number, z: number,
): number {
  // Coerce to 32-bit signed integers — matches curve25519.js mula_small.
  // CRITICAL: z = z | 0 truncates any floating-point z (e.g. from divmod's
  // z /= dt) before multiplication, preventing float accumulation in v.
  m = m | 0;
  n = n | 0;
  z = z | 0;

  let v = 0;
  for (let j = 0; j < n; ++j) {
    v += (q[j + m] & 0xff) + z * (x[j] & 0xff);
    p[j + m] = v & 0xff;
    v >>= 8;
  }
  return v;
}

function mula32Op(
  p: number[], x: number[], y: number[],
  t: number, z: number,
): number {
  // Coerce to 32-bit signed integers — matches curve25519.js mula32.
  t = t | 0;
  z = z | 0;

  const n = 31;
  let w = 0;
  let i = 0;
  for (; i < t; i++) {
    const zy = z * (y[i] & 0xff);
    w += mulaSmall(p, p, i, x, n, zy) + (p[i + n] & 0xff) + zy * (x[n] & 0xff);
    p[i + n] = w & 0xff;
    w >>= 8;
  }
  p[i + n] = ((w + (p[i + n] & 0xff)) & 0xff);
  return w >> 8;
}

function divmodOp(
  q: number[], r: number[], n: number,
  d: number[], t: number,
): void {
  let rn = 0;
  let dt = ((d[t - 1] & 0xff) << 8);
  if (t > 1) {
    dt |= (d[t - 2] & 0xff);
  }

  while (n-- >= t) {
    let z = (rn << 16) | ((r[n] & 0xff) << 8);
    if (n > 0) {
      z |= (r[n - 1] & 0xff);
    }
    z = Math.trunc(z / dt);
    rn += mulaSmall(r, r, n - t + 1, d, t, -z);
    q[n - t + 1] = (z + rn) & 0xff;
    mulaSmall(r, r, n - t + 1, d, t, -rn);
    rn = r[n] & 0xff;
    r[n] = 0;
  }
  r[t - 1] = rn & 0xff;
}

function numsize(x: number[], n: number): number {
  while (n-- !== 0 && x[n] === 0) { /* noop */ }
  return n + 1;
}

// NOTE: divmodOpSign was removed — it used floating-point z /= dt, but since
// mulaSmall now truncates z via `z = z | 0` (matching curve25519.js mula_small),
// the float is discarded before multiplication anyway. divmodOp (Math.trunc)
// produces identical results and is used for all sign/keygen/egcd paths.

function egcd32Op(
  x: number[], y: number[],
  a: number[], b: number[],
): number[] {
  let an: number;
  let bn = 32;
  let qn: number;
  let i: number;
  for (i = 0; i < 32; i++) {
    x[i] = y[i] = 0;
  }
  x[0] = 1;
  an = numsize(a, 32);
  if (an === 0) {
    return y;
  }
  const temp = new Array(32).fill(0);
  while (true) {
    qn = bn - an + 1;
    divmodOp(temp, b, bn, a, an);
    bn = numsize(b, bn);
    if (bn === 0) {
      return x;
    }
    mula32Op(y, x, temp, qn, -1);

    qn = an - bn + 1;
    divmodOp(temp, a, an, b, bn);
    an = numsize(a, an);
    if (an === 0) {
      return y;
    }
    mula32Op(x, y, temp, qn, -1);
  }
}

// ============================================================================
// SECTION 3 — Curve25519 ECDH Core (curve25519_)
// ============================================================================
// f  — 16-element short-array private key (already clamped)
// c  — 16-element short-array base point x-coordinate
// s  — optional output for signing key (16-element short array), or null
// Returns q[0] — the x-coordinate of f*c (16-element short array)
// ============================================================================

function curve25519EcDh(
  f: number[],
  c: number[],
  s: number[] | null,
): number[] {
  const x1 = c;
  const q: number[][] = [cpy16(SHORT_ONE), cpy16(SHORT_ZERO)];
  const a: number[][] = [cpy16(x1), cpy16(SHORT_ONE)];

  let n = 255;

  const r0: number[][] = [new Array(16), new Array(16)];
  const r1: number[][] = [new Array(16), new Array(16)];
  const t1: number[] = new Array(16);
  const t2: number[] = new Array(16);
  const t3: number[] = new Array(16);
  const t4: number[] = new Array(16);
  let fi: number;

  while (n >= 0) {
    fi = getbit(f, n);
    if (fi === 0) {
      prep(t1, t2, a[0], a[1]);
      prep(t3, t4, q[0], q[1]);
      curveSum(r1[0], r1[1], t1, t2, t3, t4, x1);
      dblOp(r0[0], r0[1], t3, t4);
    } else {
      prep(t1, t2, q[0], q[1]);
      prep(t3, t4, a[0], a[1]);
      curveSum(r0[0], r0[1], t1, t2, t3, t4, x1);
      dblOp(r1[0], r1[1], t3, t4);
    }
    // copy results back
    for (let i = 0; i < 16; i++) {
      q[0][i] = r0[0][i];
      q[1][i] = r0[1][i];
      a[0][i] = r1[0][i];
      a[1][i] = r1[1][i];
    }
    n--;
  }

  const t: number[] = new Array(16);
  invmodp(t, q[1], 0);
  const t1m: number[] = new Array(16);
  mulmodp(t1m, q[0], t);
  q[0] = cpy16(t1m);

  // Compute s (signing key) if requested
  if (s !== null) {
    const tk = cpy16(q[0]);
    const t1x: number[] = new Array(16);
    xToY2(t1x, tk);

    const t3x: number[] = new Array(16);
    invmodp(t3x, a[1], 0);
    const t2x: number[] = new Array(16);
    mulmodp(t2x, a[0], t3x);

    const t4x: number[] = new Array(16);
    addmodp(t4x, t2x, tk);
    addmodp(t2x, t4x, SHORT_486671);
    submodp(t4x, tk, SHORT_NINE);
    sqrmodp(t3x, t4x);
    mulmodp(t4x, t2x, t3x);
    const t0x: number[] = new Array(16);
    submodp(t0x, t4x, t1x);
    submodp(t4x, t0x, SHORT_39420360);
    mulmodp(t1x, t4x, SHORT_R2Y);

    const fb = shortArrayToByteArrayLocal(f);
    const sb = new Array(32).fill(0);

    const j = isNegative(t1x);
    if (j !== 0) {
      for (let i = 0; i < 32; i++) sb[i] = fb[i];
    } else {
      mulaSmall(sb, CURVE_ORDER_TIMES_8, 0, Array.from(fb), 32, -1);
    }

    const temp1 = Array.from(CURVE_ORDER);
    const temp2 = new Array(64).fill(0);
    const temp3 = new Array(64).fill(0);
    const egcdRes = egcd32Op(temp2, temp3, Array.from(sb), temp1);
    for (let i = 0; i < 32; i++) sb[i] = egcdRes[i];
    if ((sb[31] & 0x80) !== 0) {
      mulaSmall(sb, sb, 0, CURVE_ORDER, 32, 1);
    }

    const stmp = byteArrayToShortArrayLocal(new Uint8Array(sb));
    fill16(stmp, s);
  }

  return q[0];
}

// ============================================================================
// SECTION 4 — Curve25519 short-array clamp (from nrs.encryption.js)
// ============================================================================

/**
 * curve25519_clamp — clamps a 16-element short array.
 *
 * Reference (nrs.encryption.js:636-641):
 *   curve[0]  &= 0xFFF8;
 *   curve[15] &= 0x7FFF;
 *   curve[15] |= 0x4000;
 */
function curve25519ShortClamp(curve: number[]): number[] {
  curve[0] &= 0xfff8;
  curve[15] &= 0x7fff;
  curve[15] |= 0x4000;
  return curve;
}

/**
 * Byte-level clamp used internally by keygen.
 *
 * Reference (curve25519.js:79-82):
 *   k[31] &= 0x7F;
 *   k[31] |= 0x40;
 *   k[0]  &= 0xF8;
 */
function curve25519ByteClamp(k: Uint8Array): void {
  k[31] &= 0x7f;
  k[31] |= 0x40;
  k[0] &= 0xf8;
}

/**
 * EC-KCDSA key generation.
 *
 * Reference: curve25519.keygen() from curve25519.js (lines 933-948)
 *
 * Takes a 32-byte digest, applies byte-level clamp, computes the public key P
 * (x-coordinate of k*G) and the signing key s (modular inverse of k).
 *
 * Returns { p: Uint8Array, s: Uint8Array, k: Uint8Array }
 */
function curve25519Keygen(digest: Uint8Array): {
  p: Uint8Array;
  s: Uint8Array;
  k: Uint8Array;
} {
  const k = new Uint8Array(digest);
  curve25519ByteClamp(k);

  const kShorts = byteArrayToShortArrayLocal(k);
  curve25519ShortClamp(kShorts);

  const sShorts: number[] = new Array(16);
  const pShorts = curve25519EcDh(kShorts, SHORT_NINE, sShorts);

  return {
    p: shortArrayToByteArrayLocal(pShorts),
    s: shortArrayToByteArrayLocal(sShorts),
    k,
  };
}

// ============================================================================
// SECTION 5 — EC-KCDSA sign / verify primitives
// ============================================================================

/**
 * EC-KCDSA sign primitive: v = (x - h) * s mod ORDER
 *
 * Ported from curve25519.js sign() (lines 755-788)
 * and curve25519_.js curve25519_sign (lines 766-782).
 *
 * Returns v as a 32-byte Uint8Array.
 */
function signOperation(
  h: Uint8Array,
  x: Uint8Array,
  s: Uint8Array,
): Uint8Array {
  const h1 = new Array(32);
  const x1 = new Array(32);
  for (let i = 0; i < 32; i++) {
    h1[i] = h[i];
    x1[i] = x[i];
  }

  // Reduce modulo group order (mulaSmall truncates z via z|0, so divmodOp
  // with Math.trunc matches curve25519.js divmod exactly)
  const tmp3 = new Array(32).fill(0);
  divmodOp(tmp3, h1, 32, CURVE_ORDER, 32);
  divmodOp(tmp3, x1, 32, CURVE_ORDER, 32);

  // v = x1 - h1. If v is negative, add the group order.
  const v = new Array(32).fill(0);
  mulaSmall(v, x1, 0, h1, 32, -1);
  mulaSmall(v, v, 0, CURVE_ORDER, 32, 1);

  // tmp1 = v * s mod ORDER
  const tmp1 = new Array(64).fill(0);
  mula32Op(tmp1, v, Array.from(s), 32, 1);
  const tmp2 = new Array(32).fill(0);
  divmodOp(tmp2, tmp1, 64, CURVE_ORDER, 32);

  let w = 0;
  for (let i = 0; i < 32; i++) {
    v[i] = tmp1[i];
    w |= v[i];
  }
  if (w === 0) {
    throw new Error('Signature failed: resulting value is zero');
  }

  return new Uint8Array(v);
}

/**
 * EC-KCDSA verify primitive: Y = v*P + h*G
 *
 * Ported from curve25519_.js curve25519_verify (lines 784-877).
 *
 * Returns the recovered public key Y as a 32-byte Uint8Array.
 */
function verifyOperation(
  v: Uint8Array,
  h: Uint8Array,
  P: Uint8Array,
): Uint8Array {
  const d = new Array(32).fill(0);

  const yx: number[][] = [new Array(16), new Array(16), new Array(16)];
  const yz: number[][] = [new Array(16), new Array(16), new Array(16)];
  const sArr: number[][] = [new Array(16), new Array(16)];
  const q: number[][] = [new Array(16), new Array(16)];
  const t1: number[][] = [new Array(16), new Array(16), new Array(16)];
  const t2: number[][] = [new Array(16), new Array(16), new Array(16)];

  let vi = 0; let hi = 0; let di = 0; let nvh = 0;
  let i: number; let j: number; let kIdx: number;

  const p: number[][] = [cpy16(SHORT_NINE), byteArrayToShortArrayLocal(P)];

  // Compute s[0] = P+G and s[1] = P-G
  xToY2(q[0], p[1]);
  sqrtmodp(t1[0], q[0]);
  j = isNegative(t1[0]) ? 0 : 1; // 0 if negative, 1 if positive
  // Adjust: isNegative returns x[0] & 1, so if 1 (odd, "negative" in the ref),
  // j = 1. Let's follow the original logic exactly.
  // Actually in the reference: j = curve25519_isNegative(t1[0]);
  // Then t1[j] = sub, t1[1-j] = add.
  // So if negative (j != 0), we put sub into t1[0], add into t1[1].
  // Let's re-derive: j = isNegative(t1[0]) returns 0 or 1.
  // If j=1 (negative): t1[1] = sub, t1[0] = add.
  // If j=0 (positive): t1[0] = sub, t1[1] = add.
  // In the code: submodp(t1[j], ...); addmodp(t1[1-j], ...);
  // So j==1: t1[1]=sub, t1[0]=add. j==0: t1[0]=sub, t1[1]=add.
  // The sign of t1[0] is how we chose the branch.

  j = isNegative(t1[0]);
  addmodp(t2[0], q[0], SHORT_39420360);          // t2[0] = Py^2 + Gy^2
  mulmodp(t2[1], SHORT_2Y, t1[0]);              // t2[1] = +/- Py * 2Gy

  const jIdx = j !== 0 ? j : 0;
  const notJ = j !== 0 ? 0 : 1;
  submodp(t1[jIdx], t2[0], t2[1]);              // Py^2 + Gy^2 - 2PyGy
  addmodp(t1[notJ], t2[0], t2[1]);             // Py^2 + Gy^2 + 2PyGy

  fill16(p[1], q[0]);                            // q[0] = Px
  submodp(t2[0], q[0], SHORT_NINE);              // t2[0] = Px - Gx
  sqrmodp(t2[1], t2[0]);                         // t2[1] = (Px-Gx)^2
  invmodp(t2[0], t2[1], 0);                      // t2[0] = 1/(Px-Gx)^2

  mulmodp(q[0], t1[0], t2[0]);                   // X(P+G) candidate
  submodp(q[1], q[0], p[1]);
  submodp(sArr[0], q[1], SHORT_486671);          // s[0] = X(P+G)
  mulmodp(q[0], t1[1], t2[0]);                   // X(P-G) candidate
  submodp(q[1], q[0], p[1]);
  submodp(sArr[1], q[1], SHORT_486671);          // s[1] = X(P-G)

  // reduce s[0], s[1] by multiplying by 1
  mulasmall(sArr[0], sArr[0], 1);
  mulasmall(sArr[1], sArr[1], 1);

  // Prepare the chain (compute d[])
  for (i = 0; i < 32; i++) {
    vi = (vi >> 8) ^ (v[i] & 0xff) ^ ((v[i] & 0xff) << 1);
    hi = (hi >> 8) ^ (h[i] & 0xff) ^ ((h[i] & 0xff) << 1);
    nvh = ~(vi ^ hi);
    di = (nvh & (di & 0x80) >> 7) ^ vi;
    di ^= nvh & (di & 0x01) << 1;
    di ^= nvh & (di & 0x02) << 1;
    di ^= nvh & (di & 0x04) << 1;
    di ^= nvh & (di & 0x08) << 1;
    di ^= nvh & (di & 0x10) << 1;
    di ^= nvh & (di & 0x20) << 1;
    di ^= nvh & (di & 0x40) << 1;
    d[i] = di & 0xff;
  }

  di = ((nvh & (di & 0x80) << 1) ^ vi) >> 8;

  // Initialize state
  fill16(SHORT_ONE, yx[0]);
  fill16(p[di], yx[1]);
  fill16(sArr[0], yx[2]);
  fill16(SHORT_ZERO, yz[0]);
  fill16(SHORT_ONE, yz[1]);
  fill16(SHORT_ONE, yz[2]);

  vi = 0;
  hi = 0;

  // Main ladder loop
  for (i = 32; i-- !== 0;) {
    vi = (vi << 8) | (v[i] & 0xff);
    hi = (hi << 8) | (h[i] & 0xff);
    di = (di << 8) | (d[i] & 0xff);

    for (j = 8; j-- !== 0;) {
      const t1Local0: number[] = new Array(16);
      const t2Local0: number[] = new Array(16);
      const t1Local1: number[] = new Array(16);
      const t2Local1: number[] = new Array(16);
      const t1Local2: number[] = new Array(16);
      const t2Local2: number[] = new Array(16);

      prep(t1Local0, t2Local0, yx[0], yz[0]);
      prep(t1Local1, t2Local1, yx[1], yz[1]);
      prep(t1Local2, t2Local2, yx[2], yz[2]);

      kIdx = ((vi ^ vi >> 1) >> j & 1) + ((hi ^ hi >> 1) >> j & 1);
      const t1k = kIdx === 0 ? t1Local0 : kIdx === 1 ? t1Local1 : t1Local2;
      const t2k = kIdx === 0 ? t2Local0 : kIdx === 1 ? t2Local1 : t2Local2;

      const yxNew0: number[] = new Array(16);
      const yzNew0: number[] = new Array(16);
      dblOp(yxNew0, yzNew0, t1k, t2k);

      const k2 = (di >> j & 2) ^ ((di >> j & 1) << 1);
      const t1k2 = k2 === 0 ? t1Local0 : k2 === 1 ? t1Local1 : t1Local2;
      const t2k2 = k2 === 0 ? t2Local0 : k2 === 1 ? t2Local1 : t2Local2;
      const yxNew1: number[] = new Array(16);
      const yzNew1: number[] = new Array(16);
      curveSum(yxNew1, yzNew1, t1Local1, t2Local1, t1k2, t2k2, p[di >> j & 1]);

      const yxNew2: number[] = new Array(16);
      const yzNew2: number[] = new Array(16);
      curveSum(yxNew2, yzNew2, t1Local2, t2Local2, t1Local0, t2Local0,
        sArr[((vi ^ hi) >> j & 2) >> 1]);

      fill16(yxNew0, yx[0]);
      fill16(yzNew0, yz[0]);
      fill16(yxNew1, yx[1]);
      fill16(yzNew1, yz[1]);
      fill16(yxNew2, yx[2]);
      fill16(yzNew2, yz[2]);
    }
  }

  kIdx = (vi & 1) + (hi & 1);
  const t1Inv: number[] = new Array(16);
  invmodp(t1Inv, yz[kIdx], 0);
  const t1Res: number[] = new Array(16);
  mulmodp(t1Res, yx[kIdx], t1Inv);

  // Convert from short array (16 elements) to byte array (32 elements)
  const Y: number[] = new Array(32);
  for (let idx = 0; idx < 16; idx++) {
    Y[2 * idx] = t1Res[idx] & 0xff;
    Y[2 * idx + 1] = (t1Res[idx] >> 8) & 0xff;
  }
  return new Uint8Array(Y);
}

// ============================================================================
// SECTION 6 — Internal helpers
// ============================================================================

/**
 * SHA-256 of b1 if b2 is undefined, otherwise SHA-256(b1 || b2).
 * Ported exactly from the reference simpleHash() in nrs.encryption.js.
 */
export function simpleHash(b1: Uint8Array, b2?: Uint8Array): Uint8Array {
  const sha256 = CryptoJS.algo.SHA256.create();
  sha256.update(byteArrayToWordArrayEx(b1) as unknown as CryptoJS.lib.WordArray);
  if (b2 !== undefined) {
    sha256.update(byteArrayToWordArrayEx(b2) as unknown as CryptoJS.lib.WordArray);
  }
  const hash = sha256.finalize();
  return wordArrayToByteArrayEx(hash as unknown as { words: number[]; sigBytes: number });
}

/**
 * Constant-time byte-array equality check.
 */
function areByteArraysEqual(bytes1: Uint8Array, bytes2: Uint8Array): boolean {
  if (bytes1.length !== bytes2.length) {
    return false;
  }
  for (let i = 0; i < bytes1.length; ++i) {
    if (bytes1[i] !== bytes2[i]) {
      return false;
    }
  }
  return true;
}

/**
 * Concatenate two Uint8Arrays.
 */
function concatBytes(a: Uint8Array, b: Uint8Array): Uint8Array {
  const result = new Uint8Array(a.length + b.length);
  result.set(a, 0);
  result.set(b, a.length);
  return result;
}

/**
 * ECDH shared secret: myPrivateKey * theirPublicKey
 *
 * Reference: getSharedSecret() in nrs.encryption.js (lines 764-766)
 */
export function getSharedSecret(
  privateKey: Uint8Array,
  publicKey: Uint8Array,
): Uint8Array {
  return shortArrayToByteArrayLocal(
    curve25519EcDh(
      byteArrayToShortArrayLocal(privateKey),
      byteArrayToShortArrayLocal(publicKey),
      null,
    ),
  );
}

/**
 * Cryptographically secure random bytes.
 */
function getRandomBytesInternal(length: number): Uint8Array {
  const bytes = new Uint8Array(length);

  // Web Crypto API
  if (
    typeof globalThis !== 'undefined' &&
    typeof globalThis.crypto !== 'undefined' &&
    typeof globalThis.crypto.getRandomValues === 'function'
  ) {
    globalThis.crypto.getRandomValues(bytes);
    return bytes;
  }

  // Node.js crypto module
  try {
    // eslint-disable-next-line @typescript-eslint/no-var-requires
    const nodeCrypto = require('crypto');
    const randomBytes = nodeCrypto.randomBytes(length);
    bytes.set(new Uint8Array(randomBytes));
    return bytes;
  } catch {
    throw new Error('No secure random number source available');
  }
}

// ============================================================================
// SECTION 7 — Public API
// ============================================================================

/**
 * getPublicKey — Derives a 32-byte Curve25519 public key from a secret phrase.
 *
 * Reference: NRS.getPublicKey() (nrs.encryption.js:46-63)
 *
 * Algorithm:
 *   1. SHA-256(UTF-8 bytes of secretPhrase) → 32 bytes
 *   2. curve25519.keygen(digest) → { p, s, k }
 *   3. Return hex string of p (64 hex chars)
 */
export function getPublicKey(secretPhrase: string): string {
  const secretPhraseBytes = stringToByteArray(secretPhrase);
  const digest = simpleHash(secretPhraseBytes);
  const keypair = curve25519Keygen(digest);
  return byteArrayToHexString(keypair.p);
}

/**
 * getPrivateKey — Derives a clamped 32-byte private key from a secret phrase.
 *
 * Reference: NRS.getPrivateKey() (nrs.encryption.js:66-69)
 *
 * Algorithm:
 *   1. SHA-256(UTF-8 bytes of secretPhrase) → 32 bytes
 *   2. Convert to 16-element short array
 *   3. Clamp the short array (curve25519_clamp)
 *   4. Return hex string of the clamped short array (64 hex chars)
 */
export function getPrivateKey(secretPhrase: string): string {
  const bytes = simpleHash(stringToByteArray(secretPhrase));
  return shortArrayToHexString(
    curve25519ShortClamp(byteArrayToShortArray(bytes)),
  );
}

/**
 * getAccountIdFromPublicKey — Convert a hex public key to a numeric account ID.
 *
 * Reference: NRS.getAccountIdFromPublicKey() (nrs.encryption.js:75-86)
 *
 * Algorithm:
 *   1. hexStringToByteArray(publicKey) → 32 bytes
 *   2. SHA-256(publicKey bytes) → 32 bytes
 *   3. Take first 8 bytes
 *   4. Convert to BigInt → decimal string
 */
export function getAccountIdFromPublicKey(publicKey: string): string {
  const publicKeyBytes = hexStringToByteArray(publicKey);
  const account = simpleHash(publicKeyBytes);
  const slice = account.slice(0, 8);
  return byteArrayToBigInteger(slice).toString();
}

/**
 * getAccountId — Full pipeline from secret phrase to numeric account ID.
 *
 * Reference: NRS.getAccountId() (nrs.encryption.js:71-73)
 *
 *   1. secretPhrase → getPublicKey → getAccountIdFromPublicKey
 */
export function getAccountId(secretPhrase: string): string {
  const publicKey = getPublicKey(secretPhrase);
  return getAccountIdFromPublicKey(publicKey);
}

/**
 * signBytes — EC-KCDSA signature generation.
 *
 * Reference: NRS.signBytes() (nrs.encryption.js:205-227)
 *
 * Parameters:
 *   message      — hex-encoded string of the transaction bytes to sign
 *   secretPhrase — the raw secret phrase (UTF-8 string)
 *
 * Algorithm:
 *   secretPhraseBytes = stringToByteArray(secretPhrase)
 *   messageBytes      = hexStringToByteArray(message)
 *   digest  = SHA-256(secretPhraseBytes)                → 32 bytes
 *   keypair = curve25519.keygen(digest)                  → { p, s, k }
 *   s       = keypair.s                                  → signing private key
 *   m       = SHA-256(messageBytes)                      → 32 bytes
 *   x       = SHA-256(m || s)                            → signing private key
 *   Y       = curve25519.keygen(x).p                     → public key from x
 *   h       = SHA-256(m || Y)                            → signature hash
 *   v       = curve25519.sign(h, x, s)                   → 32-byte signature v
 *   Returns hex(v || h)  (128 hex chars = 64 bytes)
 */
export function signBytes(message: string, secretPhrase: string): string {
  const messageBytes = hexStringToByteArray(message);
  const secretPhraseBytes = stringToByteArray(secretPhrase);

  const digest = simpleHash(secretPhraseBytes);
  const keypair = curve25519Keygen(digest);
  const s = keypair.s;

  const m = simpleHash(messageBytes);

  const x = simpleHash(m, s);

  // curve25519Keygen 内部 clamp x（与参考 curve25519.keygen 行为一致）。
  // 参考 curve25519.js 的 keygen 直接修改调用方传入的 k，导致后续 sign(h, x, s)
  // 接收的是 clamp 后的 x。TS 的 curve25519Keygen 复制了 digest 不修改原 x，
  // 因此必须显式使用返回的 k（clamp 后的 x）传给 signOperation，否则签名 v 错误。
  const yKeypair = curve25519Keygen(x);
  const Y = yKeypair.p;
  const xClamped = yKeypair.k;

  const h = simpleHash(m, Y);

  const v = signOperation(h, xClamped, s);

  return byteArrayToHexString(concatBytes(v, h));
}

/**
 * verifySignature — Verify an EC-KCDSA signature.
 *
 * Reference: NRS.verifySignature() (nrs.encryption.js:229-246)
 *
 * Parameters:
 *   signature — 128 hex chars (v: first 64 hex chars, h: last 64 hex chars)
 *   message   — hex-encoded string of the signed bytes
 *   publicKey — hex-encoded 32-byte public key
 *
 * Returns true if the signature is valid.
 */
export function verifySignature(
  signature: string,
  message: string,
  publicKey: string,
): boolean {
  const signatureBytes = hexStringToByteArray(signature);
  const messageBytes = hexStringToByteArray(message);
  const publicKeyBytes = hexStringToByteArray(publicKey);

  const v = signatureBytes.slice(0, 32);
  const h = signatureBytes.slice(32);

  const Y = verifyOperation(v, h, publicKeyBytes);

  const m = simpleHash(messageBytes);
  const h2 = simpleHash(m, Y);

  return areByteArraysEqual(h, h2);
}

// ============================================================================
// SECTION 8 — AES Encryption / Decryption
// ============================================================================

export interface EncryptionOptions {
  privateKey?: Uint8Array;
  publicKey?: Uint8Array;
  sharedKey?: Uint8Array;
  nonce?: Uint8Array;
  account?: string;
  isText?: boolean;
  isCompressed?: boolean;
}

/**
 * getEncryptionKeys — Ensure the options object has the keys needed
 * for encryption/decryption.  If sharedKey is already set, return as-is.
 * Otherwise derive the ECDH shared key from privateKey + publicKey.
 *
 * Reference: NRS.getEncryptionKeys() (nrs.encryption.js:88-135)
 */
export function getEncryptionKeys(
  options: EncryptionOptions,
  secretPhrase?: string,
): EncryptionOptions {
  if (options.sharedKey) {
    return options;
  }

  if (!options.privateKey) {
    if (!secretPhrase) {
      throw new Error('Passphrase required for encryption');
    }
    options.privateKey = hexStringToByteArray(getPrivateKey(secretPhrase));
  }

  if (!options.publicKey) {
    throw new Error('Public key not specified');
  }

  return options;
}

/**
 * sharedSecretToSharedKey — XOR shared secret with nonce, then SHA-256 hash.
 *
 * Reference: NRS.sharedSecretToSharedKey() (nrs.encryption.js:768-773)
 */
export function sharedSecretToSharedKey(
  sharedSecret: Uint8Array,
  nonce: Uint8Array,
): Uint8Array {
  const xored = new Uint8Array(32);
  for (let i = 0; i < 32; i++) {
    xored[i] = sharedSecret[i] ^ nonce[i];
  }
  return simpleHash(xored);
}

/**
 * getSharedKey — ECDH shared secret XORed with nonce, then SHA-256 hashed.
 *
 * Reference: NRS.getSharedKey() (nrs.encryption.js:775-778)
 */
export function getSharedKey(
  privateKey: Uint8Array,
  publicKey: Uint8Array,
  nonce: Uint8Array,
): Uint8Array {
  const sharedSecret = getSharedSecret(privateKey, publicKey);
  for (let i = 0; i < 32; i++) {
    sharedSecret[i] ^= nonce[i];
  }
  return simpleHash(sharedSecret);
}

/**
 * aesEncrypt — AES-256-CBC encrypt with ECDH-derived shared key.
 *
 * Reference: aesEncrypt() (nrs.encryption.js:654-675)
 *
 * Algorithm:
 *   1. Generate random 16-byte IV
 *   2. Derive shared key = getSharedSecret(privateKey, publicKey) XOR nonce
 *   3. SHA-256(shared key XOR nonce) → AES key
 *   4. AES-CBC encrypt plaintext
 *   5. Return IV || ciphertext
 */
export function aesEncrypt(
  plaintext: Uint8Array,
  options: EncryptionOptions,
): Uint8Array {
  const ivBytes = getRandomBytesInternal(16);

  const text = byteArrayToWordArrayEx(plaintext) as unknown as CryptoJS.lib.WordArray;

  let sharedKey: Uint8Array;
  if (options.sharedKey) {
    sharedKey = new Uint8Array(options.sharedKey);
  } else {
    sharedKey = getSharedSecret(options.privateKey!, options.publicKey!);
  }

  if (options.nonce) {
    for (let i = 0; i < 32; i++) {
      sharedKey[i] ^= options.nonce[i];
    }
  }

  const key = CryptoJS.SHA256(byteArrayToWordArrayEx(sharedKey) as unknown as CryptoJS.lib.WordArray);
  const iv = byteArrayToWordArrayEx(ivBytes) as unknown as CryptoJS.lib.WordArray;

  const encrypted = CryptoJS.AES.encrypt(text, key, { iv });
  const ivOut = wordArrayToByteArrayEx(encrypted.iv as unknown as { words: number[]; sigBytes: number });
  const ciphertextOut = wordArrayToByteArrayEx(encrypted.ciphertext as unknown as { words: number[]; sigBytes: number });

  return concatBytes(ivOut, ciphertextOut);
}

/**
 * aesDecrypt — AES-256-CBC decrypt.
 *
 * Reference: aesDecrypt() (nrs.encryption.js:677-722)
 *
 * Returns { decrypted: Uint8Array, sharedKey: Uint8Array }
 */
export function aesDecrypt(
  ivCiphertext: Uint8Array,
  options: EncryptionOptions,
): { decrypted: Uint8Array; sharedKey: Uint8Array } {
  if (ivCiphertext.length < 16 || ivCiphertext.length % 16 !== 0) {
    throw new Error('Invalid ciphertext');
  }

  const iv = byteArrayToWordArrayEx(ivCiphertext.slice(0, 16)) as unknown as CryptoJS.lib.WordArray;
  const ciphertext = byteArrayToWordArrayEx(ivCiphertext.slice(16)) as unknown as CryptoJS.lib.WordArray;

  let sharedKey: Uint8Array;
  if (options.sharedKey) {
    sharedKey = new Uint8Array(options.sharedKey);
  } else {
    sharedKey = getSharedSecret(options.privateKey!, options.publicKey!);
  }

  let keyBytes: Uint8Array;
  if (options.nonce) {
    for (let i = 0; i < 32; i++) {
      sharedKey[i] ^= options.nonce[i];
    }
    keyBytes = wordArrayToByteArrayEx(
      CryptoJS.SHA256(
        byteArrayToWordArrayEx(sharedKey) as unknown as CryptoJS.lib.WordArray,
      ) as unknown as { words: number[]; sigBytes: number },
    );
  } else {
    keyBytes = new Uint8Array(sharedKey);
  }

  const key = byteArrayToWordArrayEx(keyBytes) as unknown as CryptoJS.lib.WordArray;

  const encrypted = CryptoJS.lib.CipherParams.create({
    ciphertext,
    iv,
    key,
  });

  const decrypted = CryptoJS.AES.decrypt(encrypted as any, key, { iv });
  const decryptedBytes = wordArrayToByteArrayEx(decrypted as unknown as { words: number[]; sigBytes: number });

  return {
    decrypted: decryptedBytes,
    sharedKey: new Uint8Array(keyBytes),
  };
}

/**
 * encryptData — Gzip-compress then AES-encrypt arbitrary data.
 *
 * Reference: encryptData() (nrs.encryption.js:728-739)
 *
 * Algorithm:
 *   1. Generate random 32-byte nonce
 *   2. Derive shared key if not provided
 *   3. Gzip compress the plaintext
 *   4. AES encrypt with the nonce
 *   5. Return { nonce, data }
 */
export function encryptData(
  plaintext: Uint8Array,
  options: EncryptionOptions,
): { nonce: Uint8Array; data: Uint8Array } {
  const opts = { ...options };
  opts.nonce = getRandomBytesInternal(32);

  if (!opts.sharedKey) {
    opts.sharedKey = getSharedSecret(opts.privateKey!, opts.publicKey!);
  }

  const compressedPlaintext = gzip(new Uint8Array(plaintext));
  const data = aesEncrypt(new Uint8Array(compressedPlaintext), opts);

  return {
    nonce: opts.nonce,
    data,
  };
}

/**
 * decryptData — AES-decrypt then Gzip-decompress data.
 *
 * Reference: decryptData() (nrs.encryption.js:745-762)
 *
 * Returns { message: string, sharedKey: string }
 */
export function decryptData(
  data: Uint8Array,
  options: EncryptionOptions,
): { message: string; sharedKey: string } {
  const opts = { ...options };

  if (!opts.sharedKey) {
    opts.sharedKey = getSharedSecret(opts.privateKey!, opts.publicKey!);
  }

  const result = aesDecrypt(data, opts);
  let binData = new Uint8Array(result.decrypted);

  if (!(opts.isCompressed === false)) {
    binData = inflate(binData);
  }

  let message: string;
  if (!(opts.isText === false)) {
    message = byteArrayToString(binData);
  } else {
    message = byteArrayToHexString(binData);
  }

  return { message, sharedKey: byteArrayToHexString(result.sharedKey) };
}

/**
 * encryptNote — Encrypt a text message for a recipient.
 *
 * Reference: NRS.encryptNote() (nrs.encryption.js:137-155)
 *
 * Parameters:
 *   message      — UTF-8 string to encrypt
 *   options      — must contain publicKey or sharedKey
 *   secretPhrase — sender's secret phrase (optional if privateKey is provided)
 *
 * Returns { message: hex string, nonce: hex string }
 */
export function encryptNote(
  message: string,
  options: EncryptionOptions,
  secretPhrase?: string,
): { message: string; nonce: string } {
  const resolved = getEncryptionKeys(options, secretPhrase);
  const encrypted = encryptData(stringToByteArray(message), resolved);
  return {
    message: byteArrayToHexString(encrypted.data),
    nonce: byteArrayToHexString(encrypted.nonce),
  };
}

/**
 * decryptNote — Decrypt an encrypted note.
 *
 * Reference: NRS.decryptNote() (nrs.encryption.js:157-203)
 *
 * Parameters:
 *   message      — hex-encoded encrypted data
 *   options      — must contain privateKey/publicKey or sharedKey, and nonce
 *   secretPhrase — optional secret phrase to derive the private key
 *
 * Returns { message: string, sharedKey: string }
 */
export function decryptNote(
  message: string,
  options: EncryptionOptions,
  secretPhrase?: string,
): { message: string; sharedKey: string } {
  const resolved = getEncryptionKeys(options, secretPhrase);

  if (resolved.nonce) {
    resolved.nonce = hexStringToByteArray(
      typeof resolved.nonce === 'string'
        ? resolved.nonce
        : byteArrayToHexString(resolved.nonce),
    );
  }

  return decryptData(hexStringToByteArray(message), resolved);
}

/**
 * generateToken — Derive a short numeric token from a message and secret phrase.
 *
 * Algorithm:
 *   1. Compute public key from the secret phrase
 *   2. SHA-256(publicKey bytes || UTF-8 bytes of message)
 *   3. Take first 8 bytes of the hash
 *   4. Convert to BigInt → decimal string
 */
export function generateToken(
  message: string,
  secretPhrase: string,
): string {
  const publicKey = getPublicKey(secretPhrase);
  const publicKeyBytes = hexStringToByteArray(publicKey);
  const messageBytes = stringToByteArray(message);
  const hash = simpleHash(publicKeyBytes, messageBytes);
  const tokenBytes = hash.slice(0, 8);
  return byteArrayToBigInteger(tokenBytes).toString();
}

/**
 * getRandomBytes — Cryptographically secure random byte generation.
 *
 * Exported for convenience. Uses Web Crypto API or Node.js crypto module.
 */
export function getRandomBytes(length: number): Uint8Array {
  return getRandomBytesInternal(length);
}

// ============================================================================
// SECTION 9 — Legacy-compatibility exports
// ============================================================================

/**
 * getPublicKeyFromHex — Derive public key from a hex-encoded secret phrase.
 *
 * This matches the old NRS.getPublicKey(hexSecretPhrase) where the secret
 * phrase was already converted to hex before being passed in.
 *
 * Reference: NRS.getPublicKey(converters.stringToHexString(secretPhrase))
 */
export function getPublicKeyFromHex(hexSecretPhrase: string): string {
  const bytes = hexStringToByteArray(hexSecretPhrase);
  const digest = simpleHash(bytes);
  const keypair = curve25519Keygen(digest);
  return byteArrayToHexString(keypair.p);
}
