//! Monitored Account
//!
//! 对应 Java: MonitoredAccount.java

use serde::{Deserialize, Serialize};
use blockchain_types::{AccountId, Height};

/// Monitored Account
///
/// 对应 Java: MonitoredAccount
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoredAccount {
    pub account_id: AccountId,
    pub monitor_index: usize,
    pub amount: i64,
    pub threshold: i64,
    pub interval: i32,
    pub height: Height,
}

impl MonitoredAccount {
    pub const MIN_FUND_AMOUNT: i64 = 1;
    pub const MIN_FUND_THRESHOLD: i64 = 1;
    pub const MIN_FUND_INTERVAL: i32 = 10;

    pub fn new(
        account_id: AccountId,
        monitor_index: usize,
        amount: i64,
        threshold: i64,
        interval: i32,
    ) -> Result<Self, String> {
        if amount < Self::MIN_FUND_AMOUNT {
            return Err(format!("Minimum fund amount is {}", Self::MIN_FUND_AMOUNT));
        }
        if threshold < Self::MIN_FUND_THRESHOLD {
            return Err(format!("Minimum fund threshold is {}", Self::MIN_FUND_THRESHOLD));
        }
        if interval < Self::MIN_FUND_INTERVAL {
            return Err(format!("Minimum fund interval is {}", Self::MIN_FUND_INTERVAL));
        }

        Ok(Self {
            account_id,
            monitor_index,
            amount,
            threshold,
            interval,
            height: 0,
        })
    }

    pub fn set_amount(&mut self, amount: i64) -> Result<(), String> {
        if amount < Self::MIN_FUND_AMOUNT {
            return Err(format!("Minimum fund amount is {}", Self::MIN_FUND_AMOUNT));
        }
        self.amount = amount;
        Ok(())
    }

    pub fn set_threshold(&mut self, threshold: i64) -> Result<(), String> {
        if threshold < Self::MIN_FUND_THRESHOLD {
            return Err(format!("Minimum fund threshold is {}", Self::MIN_FUND_THRESHOLD));
        }
        self.threshold = threshold;
        Ok(())
    }

    pub fn set_interval(&mut self, interval: i32) -> Result<(), String> {
        if interval < Self::MIN_FUND_INTERVAL {
            return Err(format!("Minimum fund interval is {}", Self::MIN_FUND_INTERVAL));
        }
        self.interval = interval;
        Ok(())
    }

    pub fn set_height(&mut self, height: Height) {
        self.height = height;
    }

    pub fn needs_funding(&self, current_balance: i64, current_height: Height) -> bool {
        current_balance < self.threshold && current_height >= self.height + self.interval as u32
    }

    pub fn from_property_value(
        account_id: AccountId,
        monitor_index: usize,
        property_value: &str,
        default_amount: i64,
        default_threshold: i64,
        default_interval: i32,
    ) -> Result<Self, String> {
        let (amount, threshold, interval) = if property_value.is_empty() {
            (default_amount, default_threshold, default_interval)
        } else {
            parse_property_json(property_value, default_amount, default_threshold, default_interval)?
        };

        Self::new(account_id, monitor_index, amount, threshold, interval)
    }
}

fn parse_property_json(
    json: &str,
    default_amount: i64,
    default_threshold: i64,
    default_interval: i32,
) -> Result<(i64, i64, i32), String> {
    let parsed: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| format!("Invalid JSON: {}", e))?;

    let obj = parsed.as_object()
        .ok_or("Property value is not a JSON object")?;

    let amount = obj.get("amount")
        .and_then(|v| v.as_i64())
        .unwrap_or(default_amount);

    let threshold = obj.get("threshold")
        .and_then(|v| v.as_i64())
        .unwrap_or(default_threshold);

    let interval = obj.get("interval")
        .and_then(|v| v.as_i64())
        .map(|v| v as i32)
        .unwrap_or(default_interval);

    Ok((amount, threshold, interval))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitored_account_new() {
        let account = MonitoredAccount::new(100, 0, 1000, 100, 10).unwrap();
        assert_eq!(account.account_id, 100);
        assert_eq!(account.amount, 1000);
        assert_eq!(account.threshold, 100);
        assert_eq!(account.interval, 10);
    }

    #[test]
    fn test_monitored_account_validation() {
        assert!(MonitoredAccount::new(100, 0, 0, 100, 10).is_err());
        assert!(MonitoredAccount::new(100, 0, 1000, 0, 10).is_err());
        assert!(MonitoredAccount::new(100, 0, 1000, 100, 0).is_err());
    }

    #[test]
    fn test_monitored_account_needs_funding() {
        let mut account = MonitoredAccount::new(100, 0, 1000, 100, 10).unwrap();
        account.height = 100;

        assert!(account.needs_funding(50, 110));
        assert!(!account.needs_funding(150, 110));
        assert!(!account.needs_funding(50, 105));
    }

    #[test]
    fn test_monitored_account_setters() {
        let mut account = MonitoredAccount::new(100, 0, 1000, 100, 10).unwrap();
        
        account.set_amount(2000).unwrap();
        assert_eq!(account.amount, 2000);
        
        account.set_threshold(200).unwrap();
        assert_eq!(account.threshold, 200);
        
        account.set_interval(20).unwrap();
        assert_eq!(account.interval, 20);
    }

    #[test]
    fn test_from_property_value() {
        let account = MonitoredAccount::from_property_value(
            100,
            0,
            r#"{"amount": 500, "threshold": 50, "interval": 15}"#,
            1000,
            100,
            10,
        ).unwrap();
        
        assert_eq!(account.amount, 500);
        assert_eq!(account.threshold, 50);
        assert_eq!(account.interval, 15);
    }

    #[test]
    fn test_from_property_value_empty() {
        let account = MonitoredAccount::from_property_value(
            100,
            0,
            "",
            1000,
            100,
            10,
        ).unwrap();
        
        assert_eq!(account.amount, 1000);
        assert_eq!(account.threshold, 100);
        assert_eq!(account.interval, 10);
    }
}
