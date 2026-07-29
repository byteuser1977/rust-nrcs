/**
 * NRCS Address class — Reed-Solomon GF(32) encoder/decoder for NRCS account addresses.
 *
 * Address format: NRCS-XXXX-XXXX-XXXX-XXXXX
 * - 17-symbol codeword: 13 data symbols + 4 parity symbols.
 * - Alphabet: "23456789ABCDEFGHJKLMNPQRSTRVWXYZ" (32 human-friendly symbols, no 0/1/I/O).
 * - GF(32) with primitive polynomial x^5 + x^2 + 1 (x^5 = x^2 + 1).
 * - Error correction: up to 2 symbol errors (Berlekamp-Massey algorithm).
 *
 * Ported from nxtaddress.js (Public Domain, original coder: NxtChg).
 */

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/** GF(32) exponentiation table: gexp[i] = alpha^i */
const GEXP: number[] = [
  1, 2, 4, 8, 16, 5, 10, 20, 13, 26, 17, 7, 14, 28, 29, 31,
  27, 19, 3, 6, 12, 24, 21, 15, 30, 25, 23, 11, 22, 9, 18, 1,
];

/** GF(32) logarithm table: glog[alpha^i] = i */
const GLOG: number[] = [
  0, 0, 1, 18, 2, 5, 19, 11, 3, 29, 6, 27, 20, 8, 12, 23,
  4, 10, 30, 17, 7, 22, 28, 26, 21, 25, 9, 16, 13, 14, 24, 15,
];

/**
 * Codeword permutation map.
 * Maps displayed symbol positions → internal codeword positions.
 * Positions 0-3, 4-7, 8-11 are data groups; 12-16 are parity.
 */
const CWMAP: number[] = [3, 2, 1, 0, 7, 6, 5, 4, 13, 14, 15, 16, 12, 8, 9, 10, 11];

/** Display alphabet (32 symbols). */
const ALPHABET = '23456789ABCDEFGHJKLMNPQRSTUVWXYZ';

// ---------------------------------------------------------------------------
// GF(32) arithmetic helpers
// ---------------------------------------------------------------------------

/** Multiplicative inverse in GF(32). */
function ginv(a: number): number {
  return GEXP[31 - GLOG[a]];
}

/** Multiplication in GF(32). */
function gmult(a: number, b: number): number {
  if (a === 0 || b === 0) return 0;
  const idx = (GLOG[a] + GLOG[b]) % 31;
  return GEXP[idx];
}

// ---------------------------------------------------------------------------
// NrsAddress class
// ---------------------------------------------------------------------------

export class NrsAddress {
  /** Internal codeword (17 symbols in GF(32)). */
  private codeword: number[] = [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

  /** Syndrome values (indices 1..4); syndrome[0] is unused. */
  private syndrome: number[] = [0, 0, 0, 0, 0];

  /** Guessed addresses when error correction produces multiple candidates. */
  public guess: string[] = [];

  // ---- Public API --------------------------------------------------------

  /**
   * Set / parse an address string.
   *
   * @param adr  Raw address input (may be numeric account ID or RS-formatted address).
   * @param allowAccounts  If true, pure numeric account IDs (1-20 digits) are accepted
   *                       directly without requiring RS format.
   * @returns true if the address is valid (or successfully corrected).
   */
  set(adr: string, allowAccounts: boolean = true): boolean {
    let len = 0;
    this.guess = [];
    this.reset();

    adr = String(adr).trim().toUpperCase();

    // Strip "NRCS-" prefix if present.
    if (adr.startsWith('NRCS-')) {
      adr = adr.substring(4);
    }

    // --- Pure numeric account ID ---
    if (/^\d{1,20}$/.test(adr)) {
      if (allowAccounts) {
        return this.fromAcc(adr);
      }
      return false;
    }

    // --- RS-formatted address ---
    const clean: number[] = [];
    for (let i = 0; i < adr.length; i++) {
      const pos = ALPHABET.indexOf(adr[i]);
      if (pos >= 0) {
        clean[len++] = pos;
        if (len > 18) return false;
      }
    }

    if (len === 16) {
      // Guess a single deletion — try inserting every possible symbol at
      // every position.
      for (let i = 16; i >= 0; i--) {
        for (let j = 0; j < 32; j++) {
          clean[i] = j;
          this.setCodeword(clean);
          if (this.ok()) this.addGuess();
        }
        if (i > 0) {
          const t = clean[i - 1];
          clean[i - 1] = clean[i];
          clean[i] = t;
        }
      }
    }

    if (len === 18) {
      // Guess a single insertion — try dropping each symbol.
      for (let i = 0; i < 18; i++) {
        this.setCodeword(clean, 18, i);
        if (this.ok()) this.addGuess();
      }
    }

    if (len === 17) {
      this.setCodeword(clean);

      if (this.ok()) return true;

      // Try error correction (up to 2 symbol errors).
      if (this.guessErrors() && this.ok()) {
        this.addGuess();
      }
    }

    this.reset();
    return false;
  }

  /**
   * Return the RS-formatted address string (e.g. "NRCS-XXXX-XXXX-XXXX-XXXXX").
   */
  toString(): string {
    let out = 'NRCS-';
    for (let i = 0; i < 17; i++) {
      out += ALPHABET[this.codeword[CWMAP[i]]];
      if ((i & 3) === 3 && i < 13) out += '-';
    }
    return out;
  }

  /**
   * Return the numeric account ID decoded from the codeword.
   */
  accountId(): string {
    const inp: number[] = [];
    let len = 13;

    // Reverse the data portion.
    for (let i = 0; i < 13; i++) {
      inp[i] = this.codeword[12 - i];
    }

    let out = '';
    let newlen = 0;

    // Base 32 → base 10 conversion.
    do {
      let divide = 0;
      newlen = 0;

      for (let i = 0; i < len; i++) {
        divide = divide * 32 + inp[i];

        if (divide >= 10) {
          inp[newlen++] = Math.floor(divide / 10);
          divide %= 10;
        } else if (newlen > 0) {
          inp[newlen++] = 0;
        }
      }

      len = newlen;
      out += String.fromCharCode(divide + 48); // 48 = '0'.charCodeAt(0)
    } while (newlen);

    return out.split('').reverse().join('');
  }

  /**
   * Compute syndromes and return true if the codeword is valid (syndrome sum == 0).
   */
  ok(): boolean {
    let sum = 0;

    for (let i = 1; i < 5; i++) {
      let t = 0;

      for (let j = 0; j < 31; j++) {
        if (j > 12 && j < 27) continue;

        let pos = j;
        if (j > 26) pos -= 14;

        t ^= gmult(this.codeword[pos], GEXP[(i * j) % 31]);
      }

      sum |= t;
      this.syndrome[i] = t;
    }

    return sum === 0;
  }

  /**
   * Add the current address string to the guess list (deduplicated, max 3).
   */
  addGuess(): void {
    const s = this.toString();
    const len = this.guess.length;

    if (len > 2) return;

    for (let i = 0; i < len; i++) {
      if (this.guess[i] === s) return;
    }

    this.guess[len] = s;
  }

  // ---- Encode / decode ---------------------------------------------------

  /**
   * Encode the data portion of the codeword by computing 4 Reed-Solomon
   * parity symbols and writing them into positions 13..16.
   */
  private encode(): void {
    const p = [0, 0, 0, 0];

    for (let i = 12; i >= 0; i--) {
      const fb = this.codeword[i] ^ p[3];

      p[3] = p[2] ^ gmult(30, fb);
      p[2] = p[1] ^ gmult(6, fb);
      p[1] = p[0] ^ gmult(9, fb);
      p[0] = gmult(17, fb);
    }

    this.codeword[13] = p[0];
    this.codeword[14] = p[1];
    this.codeword[15] = p[2];
    this.codeword[16] = p[3];
  }

  /** Reset the codeword to all-1s (sentinel). */
  private reset(): void {
    for (let i = 0; i < 17; i++) {
      this.codeword[i] = 1;
    }
  }

  /**
   * Populate the codeword from a numeric account ID string.
   * Converts base-10 → base-32, then encodes parity.
   */
  private fromAcc(acc: string): boolean {
    const inp: number[] = [];
    const out: number[] = [];
    let pos = 0;
    let len = acc.length;

    // 20-digit account IDs must start with '1'.
    if (len === 20 && acc.charAt(0) !== '1') return false;

    for (let i = 0; i < len; i++) {
      inp[i] = acc.charCodeAt(i) - 48; // '0'.charCodeAt(0)
    }

    // Base 10 → base 32 conversion.
    let newlen = 0;
    do {
      let divide = 0;
      newlen = 0;

      for (let i = 0; i < len; i++) {
        divide = divide * 10 + inp[i];

        if (divide >= 32) {
          inp[newlen++] = divide >> 5; // Math.floor(divide / 32)
          divide &= 31;                // divide % 32
        } else if (newlen > 0) {
          inp[newlen++] = 0;
        }
      }

      len = newlen;
      out[pos++] = divide;
    } while (newlen);

    // Copy to codeword in reverse, padding with zeros.
    for (let i = 0; i < 13; i++) {
      this.codeword[i] = --pos >= 0 ? out[i] : 0;
    }

    this.encode();
    return true;
  }

  /**
   * Set the codeword from an array of symbol values, applying the CWMAP
   * permutation.
   *
   * @param cw   Input symbol array.
   * @param len  Number of symbols to read (default 17).
   * @param skip Index to skip, or -1 for none (for insertion guessing).
   */
  private setCodeword(cw: number[], len: number = 17, skip: number = -1): void {
    for (let i = 0, j = 0; i < len; i++) {
      if (i !== skip) {
        this.codeword[CWMAP[j++]] = cw[i];
      }
    }
  }

  // ---- Error correction (Berlekamp-Massey) -------------------------------

  /**
   * Run the Berlekamp-Massey decoder to find and correct up to 2 symbol
   * errors in the codeword.
   *
   * @returns true if errors were successfully corrected.
   */
  private guessErrors(): boolean {
    const b: number[] = [0, 0, 0, 0, 0];
    const t: number[] = [];

    let degLambda = 0;
    let lambda: number[] = [1, 0, 0, 0, 0]; // error+erasure locator polynomial
    let el = 0;

    // Berlekamp-Massey — determine error locator polynomial.
    for (let r = 0; r < 4; r++) {
      const discr = this.calcDiscrepancy(lambda, r + 1);

      if (discr !== 0) {
        degLambda = 0;

        for (let i = 0; i < 5; i++) {
          t[i] = lambda[i] ^ gmult(discr, b[i]);
          if (t[i]) degLambda = i;
        }

        if (2 * el <= r) {
          el = r + 1 - el;

          for (let i = 0; i < 5; i++) {
            b[i] = gmult(lambda[i], ginv(discr));
          }
        }

        lambda = t.slice(); // copy
      }

      b.unshift(0); // shift => multiply by x
    }

    // Find roots of the locator polynomial.
    const errloc = this.findErrors(lambda);
    const errors = errloc.length;

    if (errors < 1 || errors > 2) return false;
    if (degLambda !== errors) return false; // deg(lambda) != #roots => uncorrectable

    // Compute error evaluator polynomial omega(x) = s(x)*lambda(x) (mod x^4).
    const omega: number[] = [0, 0, 0, 0, 0];

    for (let i = 0; i < 4; i++) {
      let val = 0;

      for (let j = 0; j < i; j++) {
        val ^= gmult(this.syndrome[i + 1 - j], lambda[j]);
      }

      omega[i] = val;
    }

    // Compute error values.
    for (let r = 0; r < errors; r++) {
      let val = 0;
      let pos = errloc[r];
      const root = 31 - pos;

      // Evaluate omega at alpha^(-i).
      for (let i = 0; i < 4; i++) {
        val ^= gmult(omega[i], GEXP[(root * i) % 31]);
      }

      if (val) {
        // Evaluate Lambda' (derivative) at alpha^(-i); odd powers vanish.
        const denom =
          gmult(lambda[1], 1) ^ gmult(lambda[3], GEXP[(root * 2) % 31]);

        if (denom === 0) return false;

        if (pos > 12) pos -= 14;

        this.codeword[pos] ^= gmult(val, ginv(denom));
      }
    }

    return true;
  }

  /**
   * Find error locations (roots of the error locator polynomial).
   *
   * @returns Array of error positions, or empty array if locations are invalid.
   */
  private findErrors(lambda: number[]): number[] {
    const errloc: number[] = [];

    for (let i = 1; i <= 31; i++) {
      let sum = 0;

      for (let j = 0; j < 5; j++) {
        sum ^= gmult(GEXP[(j * i) % 31], lambda[j]);
      }

      if (sum === 0) {
        const pos = 31 - i;
        if (pos > 12 && pos < 27) return [];
        errloc.push(pos);
      }
    }

    return errloc;
  }

  /**
   * Compute the r-th discrepancy term during Berlekamp-Massey decoding.
   */
  private calcDiscrepancy(lambda: number[], r: number): number {
    let discr = 0;

    for (let i = 0; i < r; i++) {
      discr ^= gmult(lambda[i], this.syndrome[r - i]);
    }

    return discr;
  }

  /**
   * Format a guessed address string for display, wrapping mismatched
   * characters in bold red HTML tags.
   *
   * @param s   The corrected/guessed string.
   * @param org The original (mistyped) string.
   * @returns HTML string with errors highlighted.
   */
  formatGuess(s: string, org: string): string {
    let d = '';
    const list: Array<{ s: number; e: number }> = [];

    s = s.toUpperCase();
    org = org.toUpperCase();

    // Find contiguous matching substrings between the original and guess.
    for (let i = 0; i < s.length; ) {
      let m = 0;

      for (let j = 1; j < s.length; j++) {
        const pos = org.indexOf(s.substring(i, j));

        if (pos !== -1) {
          if (Math.abs(pos - i) < 3) m = j;
        } else {
          break;
        }
      }

      if (m) {
        list.push({ s: i, e: i + m });
        i += m;
      } else {
        i++;
      }
    }

    if (list.length === 0) return s;

    for (let i = 0, j = 0; i < s.length; i++) {
      if (i >= list[j].e) {
        let start: number;

        while (j < list.length - 1) {
          start = list[j++].s;
          if (i < list[j].e || list[j].s >= start) break;
        }
      }

      if (i >= list[j].s && i < list[j].e) {
        d += s.charAt(i);
      } else {
        d += '<b style="color:red">' + s.charAt(i) + '</b>';
      }
    }

    return d;
  }
}

// ---------------------------------------------------------------------------
// Convenience functions
// ---------------------------------------------------------------------------

/**
 * Convert a numeric account ID (1-20 digits) to the NRCS Reed-Solomon address
 * format (e.g. "NRCS-XXXX-XXXX-XXXX-XXXXX").
 *
 * @param account  Numeric account ID string.
 * @returns RS-formatted address string.
 * @throws If the account ID is invalid.
 */
export function convertNumericToRSAccountFormat(account: string): string {
  const addr = new NrsAddress();
  if (!addr.set(account, true)) {
    throw new Error('Invalid numeric account ID: ' + account);
  }
  return addr.toString();
}

/**
 * Convert an NRCS Reed-Solomon address (e.g. "NRCS-XXXX-XXXX-XXXX-XXXXX")
 * back to the numeric account ID.
 *
 * Handles error correction for addresses with up to 2 symbol errors.
 *
 * @param rsAddress  RS-formatted address string.
 * @returns Numeric account ID string.
 * @throws If the address is invalid or uncorrectable.
 */
export function convertRSToNumericAccount(rsAddress: string): string {
  const addr = new NrsAddress();
  if (!addr.set(rsAddress, false)) {
    throw new Error('Invalid or uncorrectable RS address: ' + rsAddress);
  }
  return addr.accountId();
}
