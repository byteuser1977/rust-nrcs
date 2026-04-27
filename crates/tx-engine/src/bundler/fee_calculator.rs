//! Fee Calculator
//!
//! 对应 Java: MinFeeCalculator.java, ProportionalFeeCalculator.java

use async_trait::async_trait;

/// Fee Calculator Trait
///
/// 对应 Java: IFeeCalculator
#[async_trait]
pub trait FeeCalculator: Send + Sync {
    fn name(&self) -> &str;
    
    fn calculate_fee(&self, transaction_size: usize, base_fee: i64) -> i64;
    
    fn validate_rule(&self, _rule: &super::rule::BundlerRule) -> Result<(), String> {
        Ok(())
    }
}

/// Minimum Fee Calculator
///
/// 对应 Java: MinFeeCalculator
pub struct MinFeeCalculator;

impl MinFeeCalculator {
    pub const NAME: &'static str = "MIN_FEE";
    
    pub fn new() -> Self {
        Self
    }
}

impl Default for MinFeeCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FeeCalculator for MinFeeCalculator {
    fn name(&self) -> &str {
        Self::NAME
    }
    
    fn calculate_fee(&self, _transaction_size: usize, base_fee: i64) -> i64 {
        base_fee
    }
}

/// Proportional Fee Calculator
///
/// 对应 Java: ProportionalFeeCalculator
pub struct ProportionalFeeCalculator {
    pub fee_per_byte: i64,
}

impl ProportionalFeeCalculator {
    pub const NAME: &'static str = "PROPORTIONAL_FEE";
    
    pub fn new(fee_per_byte: i64) -> Self {
        Self { fee_per_byte }
    }
    
    pub fn default_fee_per_byte() -> i64 {
        100
    }
}

impl Default for ProportionalFeeCalculator {
    fn default() -> Self {
        Self::new(Self::default_fee_per_byte())
    }
}

#[async_trait]
impl FeeCalculator for ProportionalFeeCalculator {
    fn name(&self) -> &str {
        Self::NAME
    }
    
    fn calculate_fee(&self, transaction_size: usize, base_fee: i64) -> i64 {
        let proportional_fee = self.fee_per_byte * transaction_size as i64;
        base_fee.max(proportional_fee)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_min_fee_calculator() {
        let calc = MinFeeCalculator::new();
        assert_eq!(calc.name(), "MIN_FEE");
        assert_eq!(calc.calculate_fee(100, 1000), 1000);
    }

    #[test]
    fn test_proportional_fee_calculator() {
        let calc = ProportionalFeeCalculator::new(10);
        assert_eq!(calc.name(), "PROPORTIONAL_FEE");
        
        assert_eq!(calc.calculate_fee(100, 500), 1000);
        
        assert_eq!(calc.calculate_fee(50, 2000), 2000);
    }

    #[test]
    fn test_proportional_fee_default() {
        let calc = ProportionalFeeCalculator::default();
        assert_eq!(calc.fee_per_byte, 100);
    }
}
