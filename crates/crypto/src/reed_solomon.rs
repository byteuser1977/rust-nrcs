//! Reed-Solomon 编码/解码实现
//!
//! 移植自 NRCS Java 实现，用于 Account ID 编码/解码
//! 参考: /mnt/d/workspace/git/nrcs/nrcs-common/src/main/java/com/bytechain/nrcs/common/utils/ReedSolomon.java

#![allow(clippy::manual_memcpy)]
#![allow(clippy::needless_range_loop)]

const INITIAL_CODEWORD: [i32; 17] = [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
const GEXP: [i32; 32] = [1, 2, 4, 8, 16, 5, 10, 20, 13, 26, 17, 7, 14, 28, 29, 31, 27, 19, 3, 6, 12, 24, 21, 15, 30, 25, 23, 11, 22, 9, 18, 1];
const GLOG: [i32; 32] = [0, 0, 1, 18, 2, 5, 19, 11, 3, 29, 6, 27, 20, 8, 12, 23, 4, 10, 30, 17, 7, 22, 28, 26, 21, 25, 9, 16, 13, 14, 24, 15];
const CODEWORD_MAP: [usize; 17] = [3, 2, 1, 0, 7, 6, 5, 4, 13, 14, 15, 16, 12, 8, 9, 10, 11];
const ALPHABET: &str = "23456789ABCDEFGHJKLMNPQRSTUVWXYZ";

const BASE_32_LENGTH: usize = 13;
const BASE_10_LENGTH: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    CodewordTooLong,
    CodewordInvalid,
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodeError::CodewordTooLong => write!(f, "Codeword too long"),
            DecodeError::CodewordInvalid => write!(f, "Codeword invalid"),
        }
    }
}

impl std::error::Error for DecodeError {}

pub fn encode(plain: u64) -> String {
    let plain_string = plain.to_string();
    let length = plain_string.len();
    let mut plain_string_10 = [0i32; BASE_10_LENGTH];
    for (i, c) in plain_string.chars().enumerate() {
        plain_string_10[i] = (c as i32) - ('0' as i32);
    }

    let mut codeword_length = 0;
    let mut codeword = [0i32; 17];
    let mut length = length;

    while length > 0 {
        let mut new_length = 0;
        let mut digit_32 = 0;
        for i in 0..length {
            digit_32 = digit_32 * 10 + plain_string_10[i];
            if digit_32 >= 32 {
                plain_string_10[new_length] = digit_32 >> 5;
                digit_32 &= 31;
                new_length += 1;
            } else if new_length > 0 {
                plain_string_10[new_length] = 0;
                new_length += 1;
            }
        }
        length = new_length;
        codeword[codeword_length] = digit_32;
        codeword_length += 1;
    }

    let mut p = [0i32; 4];
    for i in (0..BASE_32_LENGTH).rev() {
        let fb = codeword[i] ^ p[3];
        p[3] = p[2] ^ gmult(30, fb);
        p[2] = p[1] ^ gmult(6, fb);
        p[1] = p[0] ^ gmult(9, fb);
        p[0] = gmult(17, fb);
    }

    for i in 0..4 {
        codeword[BASE_32_LENGTH + i] = p[i];
    }

    let mut result = String::new();
    for i in 0..17 {
        let codework_index = CODEWORD_MAP[i];
        let alphabet_index = codeword[codework_index] as usize;
        result.push(ALPHABET.chars().nth(alphabet_index).unwrap());

        if (i & 3) == 3 && i < 13 {
            result.push('-');
        }
    }
    result
}

pub fn decode(cypher_string: &str) -> Result<u64, DecodeError> {
    let mut codeword = INITIAL_CODEWORD;

    let mut codeword_length = 0;
    for c in cypher_string.chars() {
        let position_in_alphabet = ALPHABET.find(c);

        match position_in_alphabet {
            Some(pos) if pos <= ALPHABET.len() => {
                if codeword_length > 16 {
                    return Err(DecodeError::CodewordTooLong);
                }
                let codework_index = CODEWORD_MAP[codeword_length];
                codeword[codework_index] = pos as i32;
                codeword_length += 1;
            }
            _ => {
                continue;
            }
        }
    }

    if codeword_length == 17 && !is_codeword_valid(&codeword) || codeword_length != 17 {
        return Err(DecodeError::CodewordInvalid);
    }

    let length = BASE_32_LENGTH;
    let mut cypher_string_32 = [0i32; BASE_32_LENGTH];
    for i in 0..length {
        cypher_string_32[i] = codeword[length - i - 1];
    }

    let mut plain_string = String::new();
    let mut length = length;
    while length > 0 {
        let mut new_length = 0;
        let mut digit_10 = 0;

        for i in 0..length {
            digit_10 = digit_10 * 32 + cypher_string_32[i];

            if digit_10 >= 10 {
                cypher_string_32[new_length] = digit_10 / 10;
                digit_10 %= 10;
                new_length += 1;
            } else if new_length > 0 {
                cypher_string_32[new_length] = 0;
                new_length += 1;
            }
        }
        length = new_length;
        plain_string.push((digit_10 as u8 + b'0') as char);
    }

    let reversed: String = plain_string.chars().rev().collect();
    u64::from_str_radix(&reversed, 10).map_err(|_| DecodeError::CodewordInvalid)
}

fn gmult(a: i32, b: i32) -> i32 {
    if a == 0 || b == 0 {
        return 0;
    }

    let a_idx = a as usize;
    let b_idx = b as usize;
    let idx = (GLOG[a_idx] + GLOG[b_idx]) as usize % 31;

    GEXP[idx]
}

fn is_codeword_valid(codeword: &[i32; 17]) -> bool {
    let mut sum = 0;

    for i in 1..5 {
        let mut t = 0;

        for j in 0..31 {
            if j > 12 && j < 27 {
                continue;
            }

            let pos = if j > 26 { j - 14 } else { j };

            t ^= gmult(codeword[pos], GEXP[(i * j) % 31]);
        }

        sum |= t;
    }

    sum == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode() {
        let result = encode(1234567890123456789);
        println!("Encoded: {}", result);
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let ids = [0u64, 1, 12345, 123456789, 1234567890123456789, u64::MAX];
        for id in ids {
            let encoded = encode(id);
            let decoded = decode(&encoded).unwrap();
            assert_eq!(id, decoded, "Roundtrip failed for id={}", id);
        }
    }

    #[test]
    fn test_decode_invalid() {
        let result = decode("INVALID");
        assert!(result.is_err());
    }

    #[test]
    fn test_encode_format() {
        let encoded = encode(12345);
        assert!(encoded.contains('-'));
    }
}
