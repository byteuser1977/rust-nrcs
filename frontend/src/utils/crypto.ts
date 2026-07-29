/**
 * NRCS Cryptocurrency Utility Functions
 *
 * Re-exports common cryptographic operations from the NRCS crypto modules.
 * This file replaces the previous Ethereum-focused crypto.ts.
 */

// Re-export from converters (low-level byte/hex operations)
export {
  byteArrayToHexString,
  hexStringToByteArray,
  stringToByteArray,
  byteArrayToString,
  stringToHexString,
  hexStringToString,
  byteArrayToBigInteger,
} from './converters'

// Re-export RS address encoding
export {
  convertNumericToRSAccountFormat,
  NrsAddress,
} from './nrs-address'

// Re-export NRCS crypto operations
export {
  getPublicKey,
  getPrivateKey,
  getAccountId,
  getAccountIdFromPublicKey,
  signBytes,
  verifySignature,
  getSharedKey,
  getSharedSecret,
  sharedSecretToSharedKey,
  encryptNote,
  decryptNote,
  encryptData,
  decryptData,
  aesEncrypt,
  aesDecrypt,
  generateToken,
  getEncryptionKeys,
  getRandomBytes,
  simpleHash,
} from './nrcs-crypto'
