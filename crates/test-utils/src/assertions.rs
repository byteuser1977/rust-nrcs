//! 自定义断言宏和辅助函数
//!
//! 参照 Java NRCS 的 JSONAssert 类，提供 JSON 响应断言、余额断言等。

use serde_json::Value;

pub fn assert_json_success(json: &Value) {
    assert!(
        json.get("errorCode").is_none() || json["errorCode"].is_null(),
        "Expected success but got error: {:?}",
        json.get("errorDescription").unwrap_or(&Value::Null)
    );
    assert!(
        json.get("error").is_none() || json["error"].is_null(),
        "Expected success but got error: {:?}",
        json.get("error").unwrap_or(&Value::Null)
    );
}

pub fn assert_json_error(json: &Value) {
    let has_error_code = json.get("errorCode").is_some_and(|v| !v.is_null());
    let has_error = json.get("error").is_some_and(|v| !v.is_null());
    let has_error_desc = json.get("errorDescription").is_some_and(|v| !v.is_null());
    assert!(
        has_error_code || has_error || has_error_desc,
        "Expected error but response appears successful: {:?}",
        json
    );
}

pub fn assert_json_error_code(json: &Value, expected_code: i64) {
    let actual = json.get("errorCode")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    assert_eq!(
        actual, expected_code,
        "Expected errorCode {} but got {}: {:?}",
        expected_code, actual, json
    );
}

pub fn assert_json_str(json: &Value, key: &str, expected: &str) {
    let actual = json.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert_eq!(
        actual, expected,
        "Expected {}='{}' but got '{}'",
        key, expected, actual
    );
}

pub fn assert_json_i64(json: &Value, key: &str, expected: i64) {
    let actual = json.get(key)
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    assert_eq!(
        actual, expected,
        "Expected {}={} but got {}",
        key, expected, actual
    );
}

pub fn assert_json_has_key(json: &Value, key: &str) {
    assert!(
        json.get(key).is_some(),
        "Expected key '{}' in JSON: {:?}",
        key, json
    );
}

pub fn assert_json_not_has_key(json: &Value, key: &str) {
    assert!(
        json.get(key).is_none() || json[key].is_null(),
        "Did not expect key '{}' in JSON, but found: {:?}",
        key, json.get(key)
    );
}

pub fn assert_balance_diff(expected: i64, initial: i64, current: i64, account_name: &str) {
    let diff = current - initial;
    assert_eq!(
        diff, expected,
        "{} balance diff: expected {}, got {} (initial={}, current={})",
        account_name, expected, diff, initial, current
    );
}

pub fn get_json_str<'a>(json: &'a Value, key: &str) -> Option<&'a str> {
    json.get(key).and_then(|v| v.as_str())
}

pub fn get_json_i64(json: &Value, key: &str) -> Option<i64> {
    json.get(key).and_then(|v| v.as_i64())
}

pub fn get_json_u64(json: &Value, key: &str) -> Option<u64> {
    json.get(key).and_then(|v| v.as_str()).and_then(|s| s.parse::<u64>().ok())
        .or_else(|| json.get(key).and_then(|v| v.as_u64()))
}

#[macro_export]
macro_rules! assert_approx_eq {
    ($left:expr, $right:expr, $tolerance:expr) => {
        let diff = if $left > $right { $left - $right } else { $right - $left };
        assert!(
            diff <= $tolerance,
            "assertion failed: {:?} ≈ {:?} (tolerance={:?})",
            $left, $right, $tolerance
        );
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_assert_json_success_ok() {
        let json = json!({"fullHash": "abc123"});
        assert_json_success(&json);
    }

    #[test]
    #[should_panic]
    fn test_assert_json_success_with_error() {
        let json = json!({"errorCode": 6, "errorDescription": "insufficient balance"});
        assert_json_success(&json);
    }

    #[test]
    fn test_assert_json_error_ok() {
        let json = json!({"errorCode": 6, "errorDescription": "insufficient balance"});
        assert_json_error(&json);
    }

    #[test]
    fn test_assert_json_error_code() {
        let json = json!({"errorCode": 6});
        assert_json_error_code(&json, 6);
    }

    #[test]
    fn test_assert_json_str() {
        let json = json!({"name": "test"});
        assert_json_str(&json, "name", "test");
    }

    #[test]
    fn test_assert_json_i64() {
        let json = json!({"amount": 1000});
        assert_json_i64(&json, "amount", 1000);
    }

    #[test]
    fn test_assert_json_has_key() {
        let json = json!({"fullHash": "abc"});
        assert_json_has_key(&json, "fullHash");
    }

    #[test]
    fn test_assert_balance_diff() {
        assert_balance_diff(100, 1000, 1100, "ALICE");
        assert_balance_diff(-50, 1000, 950, "BOB");
    }

    #[test]
    fn test_get_json_u64() {
        let json = json!({"balanceNQT": "100000000000"});
        assert_eq!(get_json_u64(&json, "balanceNQT"), Some(100000000000u64));
    }
}
