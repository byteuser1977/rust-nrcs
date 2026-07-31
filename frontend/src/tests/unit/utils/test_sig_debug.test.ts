import { describe, it } from 'vitest';
import { hexStringToByteArray, byteArrayToHexString } from '@/utils/converters';
import { simpleHash } from '@/utils/nrcs-crypto';
import CryptoJS from 'crypto-js';

const M_HEX = '05c2ccf1a00a14c570c251af408eb997a79f1b3a233a8df6b388d5e08f2deefc';
const S_HEX = 'bcb21065db9c22039ec89acda2b4c4ac81273f40e323404c6b0a151c38f4cd09';
const X_EXPECTED = '58ffc35c1091b7daef024ee63d5ffb576856a7d0fb49dc3c0aa7a994b81cea6b';

// Replicate the JS converters.byteArrayToWordArray EXACTLY
function jsByteArrayToWordArray(byteArray: number[]) {
  let i = 0, offset = 0, word = 0, len = byteArray.length;
  const words = new Uint32Array(((len / 4) | 0) + (len % 4 == 0 ? 0 : 1));
  while (i < (len - (len % 4))) {
    words[offset++] = (byteArray[i++] << 24) | (byteArray[i++] << 16) | (byteArray[i++] << 8) | (byteArray[i++]);
  }
  if (len % 4 != 0) {
    word = byteArray[i++] << 24;
    if (len % 4 > 1) { word = word | byteArray[i++] << 16; }
    if (len % 4 > 2) { word = word | byteArray[i++] << 8; }
    words[offset] = word;
  }
  return { sigBytes: len, words };
}

function jsWordArrayToByteArray(wordArray: any) {
  const words = wordArray.words;
  const sigBytes = wordArray.sigBytes;
  const u8: number[] = [];
  for (let i = 0; i < sigBytes; i++) {
    u8.push((words[i >>> 2] >>> (24 - (i % 4) * 8)) & 0xff);
  }
  return u8;
}

describe('simpleHash debug', () => {
  it('compares JS-style vs TS-style WordArray', () => {
    const m = Array.from(hexStringToByteArray(M_HEX));
    const s = Array.from(hexStringToByteArray(S_HEX));

    // JS-style
    const mWA = jsByteArrayToWordArray(m);
    const sWA = jsByteArrayToWordArray(s);
    const sha1 = CryptoJS.algo.SHA256.create();
    sha1.update(mWA as any);
    sha1.update(sWA as any);
    const hash1 = sha1.finalize();
    const x1 = jsWordArrayToByteArray(hash1).map(b => b.toString(16).padStart(2, '0')).join('');
    console.log('x (JS-style):  ', x1);
    console.log('x (expected):  ', X_EXPECTED);
    console.log('JS match:', x1 === X_EXPECTED);

    // TS-style (current byteArrayToWordArrayEx)
    const x2 = simpleHash(new Uint8Array(m), new Uint8Array(s));
    console.log('x (TS simpleHash):', byteArrayToHexString(x2));
  });
});
