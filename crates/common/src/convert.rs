//! 核心数据转换工具模块
//!
//! 对照 Java NRCS Convert.java 实现，提供统一的数据转换方法。
//! 所有数值转换、编解码、时间转换集中在此模块，确保与 Java 版本完全一致。
//!
//! 参考: /mnt/d/workspace/git/nrcs/nrcs-common/src/main/java/com/bytechain/nrcs/common/utils/Convert.java

use std::collections::HashSet;

pub const TWO_64: u128 = 0x1_0000_0000_0000_0000;

const MULTIPLIERS: [u64; 9] = [1, 10, 100, 1000, 10_000, 100_000, 1_000_000, 10_000_000, 100_000_000];

const EPOCH_BEGINNING_MS: i64 = 1_578_356_400_000;

pub fn parse_hex_string(hex: &str) -> Option<Vec<u8>> {
    if hex.is_empty() {
        return None;
    }
    hex::decode(hex).ok()
}

pub fn to_hex_string(bytes: &[u8]) -> Option<String> {
    if bytes.is_empty() {
        return None;
    }
    Some(hex::encode(bytes))
}

pub fn parse_unsigned_long(number: &str) -> u64 {
    if number.is_empty() {
        return 0;
    }
    number.parse::<u64>().unwrap_or(0)
}

pub fn full_hash_to_id(hash: &[u8]) -> u64 {
    if hash.len() < 8 {
        panic!("Invalid hash: length {} < 8", hash.len());
    }
    let bytes: [u8; 8] = [hash[7], hash[6], hash[5], hash[4], hash[3], hash[2], hash[1], hash[0]];
    u64::from_be_bytes(bytes)
}

pub fn parse_account_id(account: &str) -> u64 {
    let account = account.trim();
    if account.is_empty() {
        return 0;
    }
    let account_upper = account.to_uppercase();
    if let Some(rs_part) = account_upper.strip_prefix("NRCS-") {
        rs_decode(rs_part)
    } else {
        let prefix_end = account_upper.find('-');
        match prefix_end {
            Some(idx) if idx > 0 => {
                let rs_part = &account_upper[idx + 1..];
                rs_decode(rs_part)
            }
            Some(_) => account.parse::<u64>().unwrap_or(0),
            None => account.parse::<u64>().unwrap_or(0),
        }
    }
}

pub fn rs_encode(id: u64) -> String {
    crypto::reed_solomon::encode(id)
}

pub fn rs_decode(rs_string: &str) -> u64 {
    let rs_upper = rs_string.to_uppercase();
    crypto::reed_solomon::decode(&rs_upper)
        .unwrap_or_else(|e| panic!("Reed-Solomon decoding failed for {}: {}", rs_string, e))
}

pub fn rs_account(account_id: u64) -> String {
    format!("NRCS-{}", rs_encode(account_id))
}

pub fn from_epoch_time(epoch_time: i32) -> i64 {
    epoch_time as i64 * 1000 + EPOCH_BEGINNING_MS - 500
}

pub fn to_epoch_time(current_time_ms: i64) -> i32 {
    ((current_time_ms - EPOCH_BEGINNING_MS + 500) / 1000) as i32
}

pub fn empty_to_null_string(s: &str) -> Option<&str> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

pub fn null_to_empty_string(s: Option<&str>) -> &str {
    s.unwrap_or("")
}

pub fn empty_to_null_bytes(bytes: &[u8]) -> Option<&[u8]> {
    if bytes.iter().all(|&b| b == 0) {
        None
    } else {
        Some(bytes)
    }
}

pub fn null_to_zero(val: Option<u64>) -> u64 {
    val.unwrap_or(0)
}

pub fn to_set(array: &[u64]) -> HashSet<u64> {
    array.iter().copied().collect()
}

pub fn to_bytes_from_str(s: &str) -> Vec<u8> {
    s.as_bytes().to_vec()
}

pub fn to_string_from_bytes(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

pub fn to_bytes_from_u64(n: u64) -> [u8; 8] {
    n.to_le_bytes()
}

pub fn long_to_bytes(l: u64) -> [u8; 8] {
    l.to_be_bytes()
}

pub fn bytes_to_long(b: &[u8]) -> u64 {
    let mut arr = [0u8; 8];
    arr.copy_from_slice(&b[..8]);
    u64::from_be_bytes(arr)
}

pub fn int_to_bytes(num: i32) -> [u8; 4] {
    num.to_le_bytes()
}

pub fn byte_to_int(bytes: &[u8]) -> i32 {
    let mut arr = [0u8; 4];
    arr.copy_from_slice(&bytes[..4]);
    i32::from_le_bytes(arr)
}

pub fn decimal_multiplier(decimals: u32) -> u64 {
    if (decimals as usize) < MULTIPLIERS.len() {
        MULTIPLIERS[decimals as usize]
    } else {
        10u64.pow(decimals)
    }
}

pub fn parse_nrcs(nrcs: &str) -> u64 {
    parse_string_fraction(nrcs, 8, 1_000_000_000)
}

fn parse_string_fraction(value: &str, decimals: usize, max_value: u64) -> u64 {
    let trimmed = value.trim();
    let parts: Vec<&str> = trimmed.split('.').collect();
    if parts.is_empty() || parts.len() > 2 {
        panic!("Invalid number: {}", value);
    }
    let whole_part: u64 = parts[0].parse().unwrap_or_else(|_| {
        panic!("Invalid whole part: {}", parts[0])
    });
    if whole_part > max_value {
        panic!("Whole part of value exceeds maximum possible");
    }
    if parts.len() == 1 {
        return whole_part * decimal_multiplier(decimals as u32);
    }
    let fractional_str = parts[1];
    let fractional_part: u64 = fractional_str.parse().unwrap_or_else(|_| {
        panic!("Invalid fractional part: {}", fractional_str)
    });
    let mult = decimal_multiplier(decimals as u32);
    if fractional_part >= mult || fractional_str.len() > decimals {
        panic!("Fractional part exceeds maximum allowed divisibility");
    }
    let mut adjusted_fractional = fractional_part;
    for _ in fractional_str.len()..decimals {
        adjusted_fractional *= 10;
    }
    whole_part * mult + adjusted_fractional
}

pub fn compress(bytes: &[u8]) -> Vec<u8> {
    use std::io::Write;
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(bytes).expect("GZIP compression failed");
    encoder.finish().expect("GZIP compression failed")
}

pub fn uncompress(bytes: &[u8]) -> Vec<u8> {
    use std::io::Read;
    let mut decoder = flate2::read::GzDecoder::new(bytes);
    let mut result = Vec::new();
    decoder.read_to_end(&mut result).expect("GZIP decompression failed");
    result
}

pub fn truncate(s: &str, replace_null: &str, limit: usize, dots: bool) -> String {
    if s.is_empty() && !replace_null.is_empty() {
        return replace_null.to_string();
    }
    if s.len() > limit {
        let end = if dots { limit.saturating_sub(3) } else { limit };
        let mut result = s[..end].to_string();
        if dots {
            result.push_str("...");
        }
        result
    } else {
        s.to_string()
    }
}

pub fn unit_rate_to_amount(units_qnt: u64, units_decimals: u32, rate_nqt: u64, rate_decimals: u32) -> u64 {
    let units = units_qnt as f64 / 10u64.pow(units_decimals) as f64;
    let rate = rate_nqt as f64 / 10u64.pow(rate_decimals) as f64;
    let result = units * rate * 10u64.pow(rate_decimals) as f64;
    result as u64
}

pub fn long_array_to_string_array(longs: &[u64]) -> Vec<String> {
    longs.iter().map(|l| l.to_string()).collect()
}

pub fn string_array_to_long_array(strings: &[&str]) -> Vec<u64> {
    strings.iter().map(|s| s.parse::<u64>().unwrap_or(0)).collect()
}

pub fn get_max_string_size(length: usize) -> usize {
    3 * length
}

pub fn byte_to_hex_upper(bytes: &[u8]) -> String {
    hex::encode_upper(bytes)
}

pub fn hex_to_byte(hex: &str) -> Vec<u8> {
    hex::decode(hex).unwrap_or_else(|e| panic!("Invalid hex string: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_hash_to_id() {
        let hash = [0u8; 32];
        let id = full_hash_to_id(&hash);
        assert_eq!(id, 0);
    }

    #[test]
    fn test_to_bytes_from_u64() {
        let bytes = to_bytes_from_u64(1);
        assert_eq!(bytes, [1, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_long_to_bytes() {
        let bytes = long_to_bytes(1);
        assert_eq!(bytes, [0, 0, 0, 0, 0, 0, 0, 1]);
    }

    #[test]
    fn test_bytes_to_long() {
        let bytes = [0, 0, 0, 0, 0, 0, 0, 1];
        let result = bytes_to_long(&bytes);
        assert_eq!(result, 1);
    }

    #[test]
    fn test_int_to_bytes_and_back() {
        let bytes = int_to_bytes(256);
        let result = byte_to_int(&bytes);
        assert_eq!(result, 256);
    }

    #[test]
    fn test_decimal_multiplier() {
        assert_eq!(decimal_multiplier(0), 1);
        assert_eq!(decimal_multiplier(8), 100_000_000);
    }

    #[test]
    fn test_parse_nrcs() {
        assert_eq!(parse_nrcs("1"), 100_000_000);
        assert_eq!(parse_nrcs("1.5"), 150_000_000);
        assert_eq!(parse_nrcs("0.1"), 10_000_000);
    }

    #[test]
    fn test_epoch_time_conversion() {
        let epoch = 1000;
        let ms = from_epoch_time(epoch);
        let back = to_epoch_time(ms);
        assert_eq!(epoch, back);
    }

    #[test]
    fn test_empty_to_null_string() {
        assert_eq!(empty_to_null_string(""), None);
        assert_eq!(empty_to_null_string("hello"), Some("hello"));
    }

    #[test]
    fn test_null_to_empty_string() {
        assert_eq!(null_to_empty_string(None), "");
        assert_eq!(null_to_empty_string(Some("hello")), "hello");
    }

    #[test]
    fn test_empty_to_null_bytes() {
        assert_eq!(empty_to_null_bytes(&[0, 0, 0]), None);
        assert_eq!(empty_to_null_bytes(&[0, 1, 0]), Some(&[0u8, 1, 0][..]));
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello world", "", 5, true), "he...");
        assert_eq!(truncate("hello", "", 10, false), "hello");
    }

    #[test]
    fn test_parse_unsigned_long() {
        assert_eq!(parse_unsigned_long("12345"), 12345);
        assert_eq!(parse_unsigned_long(""), 0);
    }

    #[test]
    fn test_hex_roundtrip() {
        let bytes = vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef];
        let hex = to_hex_string(&bytes).unwrap();
        let decoded = parse_hex_string(&hex).unwrap();
        assert_eq!(bytes, decoded);
    }

    #[test]
    fn test_compress_uncompress() {
        let data = b"Hello, NRCS! This is a test for GZIP compression.";
        let compressed = compress(data);
        let decompressed = uncompress(&compressed);
        assert_eq!(data.to_vec(), decompressed);
    }

    #[test]
    fn test_rs_account() {
        let account = rs_account(12345);
        assert!(account.starts_with("NRCS-"));
    }

    #[test]
    fn test_parse_account_id_numeric() {
        let id = parse_account_id("123456789");
        assert_eq!(id, 123456789);
    }

    #[test]
    fn test_unit_rate_to_amount() {
        let result = unit_rate_to_amount(100, 2, 50, 4);
        assert!(result > 0);
    }
}
