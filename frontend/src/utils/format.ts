/******************************************************************************
 * Copyright (c) 2013-2016 The Nxt Core Developers.                             *
 * Copyright (c) 2016-2017 Jelurida IP B.V.                                     *
 * Ported to TypeScript for NRCS blockchain platform.                           *
 *                                                                            *
 * NRCS formatting utilities — ported from nrs.util.js                          *
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

import Big from 'big.js'
import { NrsAddress } from './nrs-address'
import { convertNumericToRSAccountFormat } from './converters'

// ============================================================================
// Constants
// ============================================================================

/** 1 NRC = 10^8 NQT (NRCS Quantum Token) — smallest unit */
export const NQT_PER_NRC = 100_000_000

/**
 * NRCS epoch beginning: 2013-11-24 12:00:00 UTC (milliseconds).
 * Matches the Java-side {@code Constants.EPOCH_BEGINNING}.
 */
export const NRCS_EPOCH = 1387468800000

/** Padding zeros used by the core format() function. */
const ZEROS = '00000000000000000000'

// ============================================================================
// NQT / NRC Conversion
// ============================================================================

/**
 * Convert NQT (smallest unit) to NRC (display unit).
 *
 * Uses integer-division semantics matching BigInteger.divide(10^8)
 * and BigInteger.mod(10^8) from the reference Java/NRS code.
 *
 * @param amount - NQT quantity as a string or bigint
 * @param returnAsObject - if true, returns { negative, amount, mantissa }
 */
export function nqtToNxt(
  amount: string | bigint,
  returnAsObject?: false,
): string
export function nqtToNxt(
  amount: string | bigint,
  returnAsObject: true,
): { negative: string; amount: string; mantissa: string }
export function nqtToNxt(
  amount: string | bigint,
  returnAsObject = false,
): string | { negative: string; amount: string; mantissa: string } {
  const value = typeof amount === 'bigint' ? amount.toString() : String(amount)

  let negative = ''
  let nqtStr = value

  if (nqtStr.startsWith('-')) {
    negative = '-'
    nqtStr = nqtStr.substring(1)
  }

  let whole: string
  let fractionalPart: string

  if (nqtStr.length <= 8) {
    whole = '0'
    fractionalPart = nqtStr.padStart(8, '0')
  } else {
    whole = nqtStr.slice(0, -8)
    fractionalPart = nqtStr.slice(-8)
  }

  let mantissa = ''
  if (fractionalPart && fractionalPart !== '0') {
    mantissa = '.'
    // pad fractional part to at least 8 digits (leading zeros)
    for (let i = fractionalPart.length; i < 8; i++) {
      mantissa += '0'
    }
    mantissa += fractionalPart.replace(/0+$/, '')
  }

  if (returnAsObject) {
    return { negative, amount: whole, mantissa }
  }

  return negative + whole + mantissa
}

/**
 * Convert NRC (display unit) to NQT (smallest unit).
 *
 * Matches NRS.convertToNQT behavior:
 * splits on ".", pads fraction to 8 decimal places, strips leading zeros.
 *
 * @param currency - NRC value string, may include decimal point
 * @returns NQT integer string
 */
export function nxtToNqt(currency: string): string {
  currency = String(currency)
  const parts = currency.split('.')

  let amount = parts[0]
  let fraction: string

  if (parts.length === 1) {
    fraction = '00000000'
  } else if (parts.length === 2) {
    if (parts[1].length <= 8) {
      fraction = parts[1]
    } else {
      fraction = parts[1].substring(0, 8)
    }
  } else {
    throw new Error('Invalid NRC amount format')
  }

  for (let i = fraction.length; i < 8; i++) {
    fraction += '0'
  }

  let result = amount + fraction

  // Validate: only digits allowed at this point
  if (!/^\d+$/.test(result)) {
    throw new Error('Invalid NRC amount: non-digit characters found')
  }

  // Remove leading zeros
  result = result.replace(/^0+/, '')
  if (result === '') {
    result = '0'
  }

  return result
}

// ============================================================================
// QNT / QNTf Conversion (Asset quantities with arbitrary decimals)
// ============================================================================

/**
 * Convert QNT (base / integer units) to QNTf (display / fractional units).
 *
 * Matches NRS.convertToQNTf: pads with leading zeros to at least `decimals`
 * length, splits off the last `decimals` digits as mantissa, strips trailing
 * zeros from mantissa.
 *
 * @param quantity - base-unit quantity string
 * @param decimals - number of fractional digits the asset uses
 * @param returnAsObject - if true, returns { amount, mantissa }
 */
export function qntToQntf(
  quantity: string,
  decimals: number,
  returnAsObject?: false,
): string
export function qntToQntf(
  quantity: string,
  decimals: number,
  returnAsObject: true,
): { amount: string; mantissa: string }
export function qntToQntf(
  quantity: string,
  decimals: number,
  returnAsObject = false,
): string | { amount: string; mantissa: string } {
  quantity = String(quantity)

  // Pad with leading zeros so we have at least `decimals` digits
  if (quantity.length < decimals) {
    quantity = quantity.padStart(decimals, '0')
  }

  let mantissa = ''
  let whole = quantity

  if (decimals > 0) {
    mantissa = '.' + quantity.substring(quantity.length - decimals)
    whole = quantity.substring(0, quantity.length - decimals)
    if (!whole) {
      whole = '0'
    }
    // Strip trailing zeros from mantissa
    mantissa = mantissa.replace(/0+$/, '')
    if (mantissa === '.') {
      mantissa = ''
    }
  }

  if (returnAsObject) {
    return { amount: whole, mantissa }
  }
  return whole + mantissa
}

/**
 * Convert QNTf (display / fractional units) to QNT (base / integer units).
 *
 * Matches NRS.convertToQNT: splits on ".", pads fraction to exactly
 * `decimals` digits (throwing if the user-supplied fraction is too long),
 * concatenates, strips leading zeros.
 *
 * @param quantity - display-unit quantity string (may include decimal)
 * @param decimals - number of fractional digits the asset uses
 * @returns base-unit integer string
 */
export function qntfToQnt(quantity: string, decimals: number): string {
  quantity = String(quantity)
  const parts = quantity.split('.')

  let qnt = parts[0]

  if (parts.length === 1) {
    if (decimals > 0) {
      for (let i = 0; i < decimals; i++) {
        qnt += '0'
      }
    }
    // Remove leading zeros
    const stripped = qnt.replace(/^0+/, '')
    return stripped === '' ? '0' : stripped
  } else if (parts.length === 2) {
    let fraction = parts[1]
    if (fraction.length > decimals) {
      throw new Error(
        `Fractional part exceeds ${decimals} decimal places`,
      )
    } else if (fraction.length < decimals) {
      for (let i = fraction.length; i < decimals; i++) {
        fraction += '0'
      }
    }
    qnt += fraction
  } else {
    throw new Error('Invalid quantity format')
  }

  // Validate only digits
  if (!/^\d+$/.test(qnt)) {
    throw new Error('Invalid quantity: non-digit characters found')
  }

  // Remove leading zeros
  const stripped = qnt.replace(/^0+/, '')
  return stripped === '' ? '0' : stripped
}

// ============================================================================
// Core Formatting Engine
// ============================================================================

/**
 * Internal type for the params bag passed to the core format() function.
 */
export interface FormatParams {
  negative?: string
  amount: string
  mantissa: string
}

/**
 * Core formatting function — adds thousands separators, handles zero-padding,
 * and optionally performs HTML escaping.
 *
 * In the reference this is `NRS.format(params, no_escaping, zeroPad)`.
 *
 * @param params  Either a pre-built { negative, amount, mantissa } object
 *                or a plain numeric string (e.g. "1234.567" or "-0.500").
 * @param noEscaping  When true, skip HTML escaping of the output.
 * @param zeroPad  Force the fractional part to contain exactly this many digits.
 */
export function format(
  params: FormatParams | string | number,
  noEscaping?: boolean,
  zeroPad?: number,
): string {
  let negative: string
  let amount: string
  let mantissa: string

  if (typeof params !== 'object') {
    // Parse a plain string/number into the params bag
    amount = String(params)
    if (amount.indexOf('.') !== -1) {
      mantissa = amount.substring(amount.indexOf('.'))
      amount = amount.replace(mantissa, '')
    } else {
      mantissa = ''
    }
    negative = amount.charAt(0) === '-' ? '-' : ''
    if (negative) {
      amount = amount.substring(1)
    }
  } else {
    negative = params.negative || ''
    amount = String(params.amount)
    mantissa = params.mantissa || ''
  }

  // ---------- thousands separators ----------
  const digits = amount.split('').reverse()
  let formattedAmount = ''
  for (let i = 0; i < digits.length; i++) {
    if (i > 0 && i % 3 === 0) {
      formattedAmount = "'" + formattedAmount
    }
    formattedAmount = digits[i] + formattedAmount
  }

  // ---------- zeroPad ----------
  let formattedMantissa = mantissa.replace('.', '.')
  if (zeroPad !== undefined && zeroPad !== null) {
    const mantissaLen = formattedMantissa.length
    if (mantissaLen > 0) {
      formattedMantissa += ZEROS.substring(0, zeroPad - mantissaLen + 1)
    } else {
      formattedMantissa += ZEROS.substring(0, zeroPad)
      if (zeroPad !== 0) {
        formattedMantissa = '.' + formattedMantissa
      }
    }
  }

  let output = (negative ? negative : '') + formattedAmount + formattedMantissa

  if (!noEscaping) {
    output = escapeHTML(output)
  }

  return output
}

/**
 * Simple HTML-entity escaping (mirrors nrs.util.js escapeHTML behaviour).
 * Does NOT double-encode already-escaped entities.
 */
function escapeHTML(str: string): string {
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;')
}

// ============================================================================
// Amount / Quantity Formatting
// ============================================================================

/**
 * Format an amount for display.
 *
 * - `bigint`          treated as raw NQT; converted via {@link nqtToNxt} first.
 * - `string` (digits) treated as raw NQT; converted via {@link nqtToNxt}.
 * - `string` (with ".") used directly (assumed already in NRC units).
 * - `number`          optionally rounded to 2 decimal places (when `round` is true).
 *
 * @param amount   Raw value to format.
 * @param round    When true, round `number` inputs to 2 decimal places.
 * @param zeroPad  Force the fractional part to this many digits.
 * @param noEscaping  When true, skip HTML escaping.
 */
export function formatAmount(
  amount: string | bigint | number | undefined | null,
  round?: boolean,
  zeroPad?: number,
  noEscaping?: boolean,
): string {
  if (amount === undefined || amount === null) {
    return '0'
  }

  let negative = ''
  let whole = ''
  let mantissa = ''

  // bigint: assumed to be raw NQT
  if (typeof amount === 'bigint') {
    const params = nqtToNxt(amount, true)
    negative = params.negative
    whole = params.amount
    mantissa = params.mantissa
  } else if (typeof amount === 'string' && /^-?\d+$/.test(amount)) {
    // All-digit string (possibly with leading minus): treat as raw NQT
    const params = nqtToNxt(amount, true)
    negative = params.negative
    whole = params.amount
    mantissa = params.mantissa
  } else if (typeof amount === 'number') {
    // Number type — apply optional rounding
    let val = amount
    if (round) {
      val = Math.round(val * 100) / 100
    }
    if (val < 0) {
      negative = '-'
      val = Math.abs(val)
    }
    const str = String(val)
    if (str.indexOf('.') !== -1) {
      mantissa = str.substring(str.indexOf('.'))
      whole = str.replace(mantissa, '')
    } else {
      whole = str
      mantissa = ''
    }
  } else {
    // String with possible decimal
    const str = String(amount)
    if (str.startsWith('-')) {
      negative = '-'
      const absStr = str.substring(1)
      const dotIndex = absStr.indexOf('.')
      if (dotIndex !== -1) {
        whole = absStr.substring(0, dotIndex)
        mantissa = absStr.substring(dotIndex)
      } else {
        whole = absStr
        mantissa = ''
      }
    } else {
      const dotIndex = str.indexOf('.')
      if (dotIndex !== -1) {
        whole = str.substring(0, dotIndex)
        mantissa = str.substring(dotIndex)
      } else {
        whole = str
        mantissa = ''
      }
    }
  }

  return format({ negative, amount: whole, mantissa }, noEscaping, zeroPad)
}

/**
 * Format an asset quantity for display.
 *
 * Converts QNT (base units) to QNTf (display units with `decimals` fractional
 * digits), then passes through {@link format}.
 *
 * @param quantity  Base-unit quantity string.
 * @param decimals  Number of fractional digits for this asset.
 * @param zeroPad   Force the fractional part to this many digits.
 */
export function formatQuantity(
  quantity: string,
  decimals: number,
  zeroPad?: number,
): string {
  return format(qntToQntf(quantity, decimals, true), false, zeroPad)
}

/**
 * Format a styled amount for HTML display.
 *
 * Splits the formatted amount at the decimal point and wraps the fractional
 * part in a smaller-font span.
 *
 * @param strAmount  Raw amount (NQT string) to format.
 * @param round      Passed through to {@link formatAmount}.
 */
export function formatStyledAmount(
  strAmount: string,
  round?: boolean,
): string {
  const formatted = formatAmount(strAmount, round, undefined, true /* noEscaping */)
  const parts = formatted.split('.')
  if (parts.length === 2) {
    return `${parts[0]}<span style='font-size:12px'>.${parts[1]}</span>`
  }
  return parts[0]
}

// ============================================================================
// Price / Total / Percentage Calculations
// ============================================================================

/**
 * Calculate the price per whole QNT (decimals-adjusted) and return the raw
 * NXT string.
 *
 * In the reference: NRS.calculateOrderPricePerWholeQNT(price, decimals, true)
 * multiplies price (NQT) by 10^decimals and converts the result to NXT.
 *
 * @param price     Price in NQT per one QNT.
 * @param decimals  Number of fractional digits for the asset.
 * @returns Price per whole QNT in NXT units (raw string with mantissa).
 */
function calculateOrderPricePerWholeQNT_raw(
  price: string,
  decimals: number,
): string {
  const priceBig = new Big(String(price))
  const multiplier = new Big(10).pow(decimals)
  const result = priceBig.times(multiplier)
  return nqtToNxt(result.toFixed(0))
}

/**
 * Format the price per whole QNT for display.
 *
 * - Multiplies the per-QNT price (in NQT) by 10^decimals to get the
 *   price for one full unit, converts to NXT, then formats with thousands
 *   separators.
 * - Pass `zeroPad` to force a fixed number of fractional digits.
 *
 * @param price     Price in NQT per one QNT.
 * @param decimals  Number of fractional digits for the asset.
 * @param zeroPad   Force the fractional part to this many digits.
 */
export function formatOrderPricePerWholeQNT(
  price: string,
  decimals: number,
  zeroPad?: number,
): string {
  const converted = calculateOrderPricePerWholeQNT_raw(price, decimals)
  return format(converted, false, zeroPad)
}

/**
 * Calculate order total in NQT (quantityQNT * priceNQT).
 *
 * Uses Big.js to support arbitrary-precision multiplication.
 *
 * @param quantityQNT  Quantity in base units (QNT).
 * @param priceNQT     Price per QNT in NQT.
 * @returns Total in NQT as a string.
 */
export function calculateOrderTotalNQT(
  quantityQNT: string,
  priceNQT: string,
): string {
  const qty = new Big(String(quantityQNT))
  const price = new Big(String(priceNQT))
  return qty.times(price).toFixed(0, 0) // truncate to integer
}

/**
 * Calculate order total converted to NRC (display units).
 *
 * Computes `quantityQNT * priceNQT`, converts from NQT to NXT,
 * and passes the result through {@link format}.
 *
 * @param quantityQNT  Quantity in base units (QNT).
 * @param priceNQT     Price per QNT in NQT.
 * @returns Formatted total in NRC.
 */
export function calculateOrderTotal(
  quantityQNT: string,
  priceNQT: string,
): string {
  const qty = new Big(String(quantityQNT))
  const price = new Big(String(priceNQT))
  const totalNQT = qty.times(price).toFixed(0, 0)
  return format(nqtToNxt(totalNQT))
}

/**
 * Calculate percentage (a / b * 100%).
 *
 * Uses Big.js and formats the result with thousands separators.
 *
 * @param a  Numerator.
 * @param b  Denominator.
 * @returns Formatted percentage string (e.g. "12.34").
 */
export function calculatePercentage(a: string, b: string): string {
  if (String(b) === '0' || String(b) === '') {
    return '0'
  }
  const bigA = new Big(String(a))
  const bigB = new Big(String(b))
  const result = bigA.div(bigB).times(new Big('100')).toFixed(2)
  return format(result.toString(), undefined)
}

/**
 * Truncate an amount string to at most `decimals` fractional digits.
 *
 * Strips trailing zeros from the fractional part and enforces the maximum
 * number of decimal places.
 *
 * @param amount    Amount string (e.g. "123.456789").
 * @param decimals  Maximum allowed fractional digits.
 * @returns Truncated amount string.
 */
export function amountToPrecision(amount: string, decimals: number): string {
  amount = String(amount)
  const parts = amount.split('.')

  if (parts.length === 1) {
    return parts[0]
  } else if (parts.length === 2) {
    let fraction = parts[1].replace(/0+$/, '')
    if (fraction.length > decimals) {
      fraction = fraction.substring(0, decimals)
    }
    return parts[0] + '.' + fraction
  } else {
    throw new Error('Invalid amount format')
  }
}

// ============================================================================
// Timestamp (NRCS Epoch: 2013-11-24 12:00:00 UTC)
// ============================================================================

/**
 * Convert an NRCS epoch timestamp to a JavaScript millisecond timestamp.
 *
 * Matching NRS.fromEpochTime: {@code epochTime * 1000 + EPOCH_BEGINNING - 500}
 *
 * @param epochTime  Seconds since the NRCS epoch.
 * @returns JavaScript milliseconds since Unix epoch.
 */
export function fromEpochTime(epochTime: number): number {
  return epochTime * 1000 + NRCS_EPOCH - 500
}

/**
 * Convert a JavaScript Date (or current time) to an NRCS epoch timestamp.
 *
 * Matching NRS.toEpochTime:
 * {@code Math.floor((currentTime - EPOCH_BEGINNING) / 1000)}
 *
 * @param currentTime  Optional Date object (defaults to `new Date()`).
 * @returns Seconds since the NRCS epoch.
 */
export function toEpochTime(currentTime?: Date): number {
  const ct = currentTime ? currentTime.getTime() : Date.now()
  return Math.floor((ct - NRCS_EPOCH) / 1000)
}

/**
 * Format an NRCS epoch timestamp (or absolute JS timestamp) into a display
 * string.
 *
 * @param timestamp     NRCS epoch-seconds, Date object, or absolute JS ms.
 * @param dateOnly      When true, omit the time portion.
 * @param isAbsolute    When true, `timestamp` is treated as an absolute JS
 *                      millisecond timestamp instead of NRCS epoch-seconds.
 * @returns Formatted date / time string.
 */
export function formatTimestamp(
  timestamp: number | Date,
  dateOnly = false,
  isAbsolute = false,
): string {
  let date: Date

  if (typeof timestamp === 'object') {
    date = timestamp
  } else if (isAbsolute) {
    date = new Date(timestamp)
  } else {
    date = new Date(fromEpochTime(timestamp))
  }

  if (isNaN(date.getTime())) {
    return String(timestamp)
  }

  const d = date.getDate()
  const dd = d < 10 ? '0' + d : String(d)
  const M = date.getMonth() + 1
  const MM = M < 10 ? '0' + M : String(M)
  const yyyy = date.getFullYear()

  let result = `${yyyy}/${MM}/${dd}`

  if (!dateOnly) {
    const hours = date.getHours()
    const minutes = date.getMinutes()
    const seconds = date.getSeconds()

    const hh = hours < 10 ? '0' + hours : String(hours)
    const mm = minutes < 10 ? '0' + minutes : String(minutes)
    const ss = seconds < 10 ? '0' + seconds : String(seconds)

    result += ` ${hh}:${mm}:${ss}`
  }

  return result
}

/**
 * Convenience helper — format a block's `timestamp` field for display.
 *
 * @param block     Object with a `timestamp` property (in NRCS epoch-seconds).
 * @param dateOnly  When true, omit the time portion.
 * @returns Formatted date / time string.
 */
export function formatBlockTimestamp(
  block: { timestamp: number },
  dateOnly = false,
): string {
  return formatTimestamp(block.timestamp, dateOnly)
}

// ============================================================================
// Validation
// ============================================================================

/**
 * Validate decimal input during keyboard events.
 *
 * Ported from NRS.validateDecimals. Used as a key-press / key-down handler.
 *
 * @param maxFractionLength  Maximum allowed digits after the decimal separator
 *                           (0 = no fractional part allowed).
 * @param charCode           The `keyCode` or `charCode` from the keyboard event.
 * @param val                Current input element value (before the new character).
 * @param e                  (Optional) The keyboard event — `preventDefault()`
 *                           is called when the character is rejected.
 * @returns true if the character should be accepted.
 */
export function validateDecimals(
  maxFractionLength: number,
  charCode: number,
  val: string,
  e?: KeyboardEvent,
): boolean {
  // Control keys: always allow
  if (charCode < 32 || charCode === 10 || charCode === 13) {
    return true
  }

  // Period / decimal separator
  if (maxFractionLength > 0) {
    if (charCode === 110 || charCode === 190) {
      if (val.indexOf('.') !== -1) {
        e?.preventDefault()
        return false
      }
      return true
    }
  } else {
    // No fractions allowed — disallow period and comma
    if (charCode === 110 || charCode === 190 || charCode === 188) {
      e?.preventDefault()
      return false
    }
  }

  // Normalize numpad digits to standard ASCII
  let normalizedCharCode = charCode
  if (charCode >= 96 && charCode <= 105) {
    normalizedCharCode = charCode + 48 - 96
  }

  // Disallow comma (thousands separator)
  if (charCode === 188) {
    e?.preventDefault()
    return false
  }

  // Check fraction length
  if (maxFractionLength > 0) {
    const input = val + String.fromCharCode(normalizedCharCode)
    const mantissaMatch = input.match(/\.(\d*)$/)
    if (mantissaMatch && mantissaMatch[1].length > maxFractionLength) {
      e?.preventDefault()
      return false
    }
  }

  // Allow: backspace(8), left(37), right(39), delete(46), digits(48-57)
  if (
    charCode === 8 ||
    charCode === 37 ||
    charCode === 39 ||
    charCode === 46 ||
    (normalizedCharCode >= 48 && normalizedCharCode <= 57 &&
      !isNaN(Number(String.fromCharCode(normalizedCharCode))))
  ) {
    return true
  }

  e?.preventDefault()
  return false
}

/**
 * Validate an NRCS account input (numeric account ID or Reed-Solomon address).
 *
 * @param input  Raw user input — may be a numeric account ID or an RS-formatted
 *               address with or without the "NRCS-" prefix.
 * @returns An object with:
 *   - `isValid`:    true if the input resolves to a valid NRCS account.
 *   - `accountRS`:  The canonical Reed-Solomon formatted address string,
 *                   or undefined if invalid.
 *   - `accountId`:  The numeric account ID string, or undefined if invalid.
 */
export function validateNRSAccount(input: string): {
  accountRS?: string
  accountId?: string
  isValid: boolean
} {
  if (!input || String(input).trim() === '') {
    return { isValid: false }
  }

  const address = new NrsAddress()
  const valid = address.set(String(input).trim())

  if (!valid) {
    return { isValid: false }
  }

  return {
    isValid: true,
    accountRS: address.toString(),
    accountId: address.accountId(),
  }
}

// ============================================================================
// Display Helpers
// ============================================================================

/**
 * Format a byte count into a human-readable volume string.
 *
 * Ported from NRS.formatVolume. Uses base-1024 with sizes B / KB / MB / GB / TB.
 *
 * @param volume  Number of bytes.
 * @returns Formatted string with thousands separators (e.g. "1'500 KB").
 */
export function formatVolume(volume: number): string {
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  if (volume === 0) return '0 B'
  const i = Math.floor(Math.log(volume) / Math.log(1024))

  volume = Math.round(volume / Math.pow(1024, i))
  const size = sizes[i]

  const digits: number[] = []
  let vol = volume
  do {
    digits.push(vol % 10)
    vol = Math.floor(vol / 10)
  } while (vol > 0)

  let formattedVolume = ''
  for (let j = 0; j < digits.length; j++) {
    if (j > 0 && j % 3 === 0) {
      formattedVolume = "'" + formattedVolume
    }
    formattedVolume = digits[j] + formattedVolume
  }

  return formattedVolume + ' ' + size
}

/**
 * Truncate a hash / long identifier string for compact display.
 *
 * Keeps `prefixLen` characters at the start and end, separated by "...".
 *
 * @param hash       Full hash string.
 * @param prefixLen  Number of characters to preserve at each end (default 8).
 * @returns Truncated string (e.g. "abcd1234...ef567890").
 */
export function truncateHash(hash: string, prefixLen = 8): string {
  if (!hash || hash.length <= prefixLen * 2 + 3) return hash
  return `${hash.slice(0, prefixLen)}...${hash.slice(-prefixLen)}`
}

/**
 * Convenience: convert raw NQT to a formatted NRC display string.
 *
 * Equivalent to `formatAmount(nqt)` — passes a pure-digit string through
 * the NQT-to-NXT conversion and formats with thousands separators.
 *
 * @param nqt  Raw NQT quantity string.
 * @returns Formatted NRC display string.
 */
export function formatNqtToNrc(nqt: string): string {
  return formatAmount(nqt)
}

/**
 * Convert raw NQT to NRC display string (without thousands separators).
 *
 * Alias for {@link nqtToNxt}. Use this for inline display where the
 * raw NQT integer string needs to be converted to a human-readable
 * NRC value.
 *
 * @param nqt  Raw NQT quantity string.
 * @returns NRC display string (e.g. "1.5" for "150000000").
 */
export function formatNrc(nqt: string): string {
  if (!nqt || nqt === '0') return '0'
  return nqtToNxt(nqt)
}

/**
 * Format an account address for compact display.
 *
 * Chain-agnostic convenience wrapper over {@link truncateHash}.
 * Displays the first `chars` and last `chars` characters, separated by "...".
 *
 * @param address  Full account address / identifier string.
 * @param chars    Number of characters to keep at each end (default 6).
 * @returns Truncated address string.
 */
export function formatAddress(address: string, chars = 6): string {
  return truncateHash(address, chars)
}

// ============================================================================
// Chain-Agnostic Utilities (kept from the previous iteration)
// ============================================================================

/**
 * Format a number with locale-aware thousands separators.
 *
 * @param n         The number to format.
 * @param decimals  Number of fractional digits (default 0).
 */
export function formatNumber(n: number, decimals = 0): string {
  return n.toLocaleString('en-US', {
    minimumFractionDigits: decimals,
    maximumFractionDigits: decimals,
  })
}

/**
 * Format a decimal value as a percentage string.
 *
 * @param n         Percentage value.
 * @param decimals  Number of fractional digits (default 2).
 */
export function formatPercent(n: number, decimals = 2): string {
  return `${n.toFixed(decimals)}%`
}

/**
 * Format a byte count into a human-readable file-size string.
 *
 * Uses base-1024 with standard IEC suffixes (B / KB / MB / GB / TB).
 *
 * @param bytes     Number of bytes.
 * @param decimals  Number of fractional digits (default 2).
 */
export function formatFileSize(bytes: number, decimals = 2): string {
  if (bytes === 0) return '0 B'

  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))

  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(decimals))} ${sizes[i]}`
}

/**
 * Format a hash (or any long hex/identifier) for compact display.
 *
 * Renamed from the previous Ethereum `formatAddress` — this is chain-agnostic
 * and simply truncates the middle portion.
 *
 * @param hash   Full hash / identifier string.
 * @param chars  Number of characters to keep at each end (default 8).
 * @returns Truncated string (e.g. "abcd1234...ef567890").
 */
export function formatHash(hash: string, chars = 8): string {
  return truncateHash(hash, chars)
}

/**
 * Format a block number as a styled string (e.g. "#1,234,567").
 *
 * @param n  Block height / number.
 */
export function formatBlockNumber(n: number): string {
  return `#${formatNumber(n)}`
}

/**
 * Format a date/time from various input types into a localized string.
 *
 * - ISO 8601 string, Date object, or numeric Unix-seconds timestamp.
 *
 * @param dateInput  ISO string, Date object, or Unix-seconds timestamp.
 * @param options    Intl.DateTimeFormat overrides.
 */
export function formatTime(
  dateInput: string | Date | number,
  options: Intl.DateTimeFormatOptions = {},
): string {
  if (!dateInput) return ''

  const date = typeof dateInput === 'number'
    ? new Date(dateInput * 1000)
    : new Date(dateInput)

  if (isNaN(date.getTime())) return ''

  const defaultOptions: Intl.DateTimeFormatOptions = {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false,
    ...options,
  }

  return new Intl.DateTimeFormat('en-US', defaultOptions).format(date)
}

/**
 * Format a time in relative terms (e.g. "5 minutes ago", "2 days ago").
 *
 * @param dateInput  ISO string, Date object, or Unix-seconds timestamp.
 * @param locale     Locale to use (default 'en-US').
 */
export function formatRelativeTime(
  dateInput: string | Date | number,
  locale = 'en-US',
): string {
  if (!dateInput) return ''

  const date = typeof dateInput === 'number'
    ? new Date(dateInput * 1000)
    : new Date(dateInput)

  if (isNaN(date.getTime())) return ''

  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffSec = Math.floor(diffMs / 1000)
  const diffMin = Math.floor(diffSec / 60)
  const diffHour = Math.floor(diffMin / 60)
  const diffDay = Math.floor(diffHour / 24)
  const diffMonth = Math.floor(diffDay / 30)
  const diffYear = Math.floor(diffDay / 365)

  const rtf = new Intl.RelativeTimeFormat(locale, { numeric: 'auto' })

  if (diffSec < 60) {
    return rtf.format(-diffSec, 'second')
  } else if (diffMin < 60) {
    return rtf.format(-diffMin, 'minute')
  } else if (diffHour < 24) {
    return rtf.format(-diffHour, 'hour')
  } else if (diffDay < 30) {
    return rtf.format(-diffDay, 'day')
  } else if (diffMonth < 12) {
    return rtf.format(-diffMonth, 'month')
  } else {
    return rtf.format(-diffYear, 'year')
  }
}

/**
 * Deep-clone an object or array.
 *
 * @param obj  Any serializable value.
 */
export function deepClone<T>(obj: T): T {
  if (typeof structuredClone === 'function') {
    return structuredClone(obj)
  }
  return JSON.parse(JSON.stringify(obj))
}

/**
 * @deprecated Ethereum 风格的 wei → ETH 格式化函数。
 *
 * 仅为兼容尚未重写为 NRCS 风格的旧模块（如 `transaction.store.ts`）保留，
 * 新代码请使用 NRCS 风格的 `formatAmount` / `nqtToNxt` 等函数。
 *
 * 1 ETH = 10^18 wei。使用 BigInt 精确计算，避免 Number 精度丢失。
 *
 * @param wei  wei 数量的字符串表示（避免 Number 精度丢失）。
 * @returns    ETH 数量的字符串表示（保留 6 位小数）。
 */
export function formatWeiToEth(wei: string | number): string {
  if (!wei && wei !== 0) return '0'
  try {
    const weiBig = BigInt(wei)
    // 整数部分：wei / 10^18
    const ETH_DIVISOR = BigInt(10) ** BigInt(18)
    const intPart = weiBig / ETH_DIVISOR
    // 小数部分：wei % 10^18，补齐 18 位后取前 6 位
    const fracBig = weiBig % ETH_DIVISOR
    const fracStr = fracBig.toString().padStart(18, '0').slice(0, 6)
    return `${intPart}.${fracStr}`
  } catch {
    return '0'
  }
}

/**
 * @deprecated Ethereum 风格的 wei 格式化（带单位）函数。
 *
 * 仅为兼容尚未重写为 NRCS 风格的旧视图（如 `SendTransaction.vue`）保留，
 * 新代码请使用 NRCS 风格的 `formatAmount` / `formatNrc` 等函数。
 *
 * 与 `formatWeiToEth` 的区别：本函数在数值后追加 " ETH" 单位后缀，
 * 适合直接用于 UI 展示（如 Gas 费用总览）。
 *
 * @param wei  wei 数量的字符串表示。
 * @returns    形如 "0.001234 ETH" 的展示字符串。
 */
export function formatWei(wei: string | number): string {
  return `${formatWeiToEth(wei)} ETH`
}
