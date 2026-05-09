//! Log masking configuration and utilities
//!
//! Provides configurable sensitive data masking for log output.
//! Supports custom keywords, patterns, and multiple masking strategies.
//!
//! # Example
//!
//! ```rust
//! use common::{LogMasker, MaskStrategy};
//!
//! let masker = LogMasker::default()
//!     .with_keyword("password")
//!     .with_keyword("secret")
//!     .with_strategy(MaskStrategy::Full);
//!
//! let masked = masker.mask("password=admin123&token=abc");
//! assert_eq!(masked, "password=********&token=***");
//! ```

use std::collections::HashSet;
use regex::Regex;

/// Log masking configuration and utilities
///
/// Provides configurable sensitive data masking for log output.
/// Supports custom keywords, patterns, and multiple masking strategies.
#[derive(Debug, Clone)]
pub struct LogMasker {
    keywords: HashSet<String>,
    patterns: Vec<Regex>,
    strategy: MaskStrategy,
    mask_char: char,
}

impl Default for LogMasker {
    fn default() -> Self {
        let mut keywords = HashSet::new();
        keywords.insert("password".to_string());
        keywords.insert("passwd".to_string());
        keywords.insert("pwd".to_string());
        keywords.insert("secret".to_string());
        keywords.insert("token".to_string());
        keywords.insert("key".to_string());
        keywords.insert("credential".to_string());
        keywords.insert("auth".to_string());
        keywords.insert("api_key".to_string());
        keywords.insert("apikey".to_string());
        keywords.insert("access_token".to_string());
        keywords.insert("private".to_string());

        Self {
            keywords,
            patterns: Vec::new(),
            strategy: MaskStrategy::default(),
            mask_char: '*',
        }
    }
}

impl LogMasker {
    /// Create a new LogMasker with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a custom keyword to mask
    pub fn with_keyword(mut self, keyword: impl Into<String>) -> Self {
        self.keywords.insert(keyword.into());
        self
    }

    /// Add a custom regex pattern to match sensitive data
    pub fn with_pattern(mut self, pattern: Regex) -> Self {
        self.patterns.push(pattern);
        self
    }

    /// Set the masking strategy
    pub fn with_strategy(mut self, strategy: MaskStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Set the character used for masking (default: *)
    pub fn with_mask_char(mut self, ch: char) -> Self {
        self.mask_char = ch;
        self
    }

    /// Remove a keyword from the mask list
    pub fn without_keyword(mut self, keyword: &str) -> Self {
        self.keywords.remove(keyword);
        self
    }

    /// Mask sensitive information in a log string
    ///
    /// Detects patterns like `keyword=value` and masks the value portion.
    /// Supports both space-separated and delimiter-separated formats.
    pub fn mask(&self, input: &str) -> String {
        if input.is_empty() {
            return input.to_string();
        }

        let mut result = input.to_string();

        for keyword in &self.keywords {
            result = self.mask_keyword(&result, keyword);
        }

        for pattern in &self.patterns {
            result = pattern.replace_all(&result, |caps: &regex::Captures| {
                let matched = caps.get(0).map(|m| m.as_str()).unwrap_or("");
                self.apply_mask(matched)
            }).to_string();
        }

        result
    }

    /// Mask a specific keyword=value pattern
    fn mask_keyword(&self, input: &str, keyword: &str) -> String {
        let mut result = input.to_string();

        let patterns_to_check = vec![
            format!("{}=", keyword),
            format!("{}:", keyword),
            format!("{} =", keyword),
            format!("{} :", keyword),
        ];

        for pattern in &patterns_to_check {
            if let Some(pos) = result.find(pattern.as_str()) {
                let value_start = pos + pattern.len();
                if value_start < result.len() {
                    let value_end = self.find_value_end(&result, value_start);
                    let value = &result[value_start..value_end];
                    let masked_value = self.apply_mask(value);
                    result = format!(
                        "{}{}{}",
                        &result[..value_start],
                        masked_value,
                        &result[value_end..]
                    );
                }
            }
        }

        result
    }

    /// Find the end position of a value (stops at delimiter or end of string)
    fn find_value_end(&self, input: &str, start: usize) -> usize {
        let delimiters = [' ', '&', ';', ',', '\n', '\r', '"', '\''];
        
        for (i, ch) in input[start..].char_indices() {
            if delimiters.contains(&ch) {
                return start + i;
            }
        }
        
        input.len()
    }

    /// Apply masking strategy to a value string
    fn apply_mask(&self, value: &str) -> String {
        let masked: String = std::iter::repeat_n(self.mask_char, value.len()).collect();
        
        match self.strategy {
            MaskStrategy::Full => masked,
            MaskStrategy::Partial => {
                if value.len() <= 2 {
                    masked
                } else {
                    format!(
                        "{}{}",
                        &value[..1],
                        std::iter::repeat_n(self.mask_char, value.len() - 1).collect::<String>()
                    )
                }
            },
            MaskStrategy::Fixed(len) => {
                if value.len() <= len {
                    masked
                } else {
                    format!(
                        "{}{}{}",
                        &value[..2],
                        std::iter::repeat_n(self.mask_char, len).collect::<String>(),
                        &value[value.len()-2..]
                    )
                }
            },
        }
    }

    /// Check if a string contains any sensitive information
    pub fn contains_sensitive(&self, input: &str) -> bool {
        for keyword in &self.keywords {
            let patterns = vec![
                format!("{}=", keyword),
                format!("{}:", keyword),
            ];
            
            for pattern in &patterns {
                if input.contains(pattern.as_str()) {
                    return true;
                }
            }
        }

        for pattern in &self.patterns {
            if pattern.is_match(input) {
                return true;
            }
        }

        false
    }

    /// Get list of configured keywords (for debugging)
    pub fn keywords(&self) -> &HashSet<String> {
        &self.keywords
    }
}

/// Masking strategies for sensitive data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MaskStrategy {
    /// Replace entire value with mask characters (e.g., `****`)
    #[default]
    Full,
    /// Show first character only (e.g., `a***`)
    Partial,
    /// Show first and last 2 characters with fixed mask length (e.g., `ab****xy`)
    Fixed(usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_keywords() {
        let masker = LogMasker::default();
        assert!(masker.keywords().contains(&"password".to_string()));
        assert!(masker.keywords().contains(&"secret".to_string()));
        assert!(masker.keywords().contains(&"token".to_string()));
    }

    #[test]
    fn test_mask_password_full_strategy() {
        let masker = LogMasker::default();
        
        assert_eq!(
            masker.mask("password=admin123"),
            "password=********"
        );
    }

    #[test]
    fn test_mask_multiple_sensitive_fields() {
        let masker = LogMasker::default();
        
        let input = "password=admin&secret=mysecret&token=abc123";
        let result = masker.mask(input);
        
        assert_eq!(
            result,
            "password=*****&secret=********&token=******"
        );
    }

    #[test]
    fn test_mask_partial_strategy() {
        let masker = LogMasker::default()
            .with_strategy(MaskStrategy::Partial);
        
        assert_eq!(
            masker.mask("password=admin123"),
            "password=a*******"
        );
    }

    #[test]
    fn test_mask_fixed_strategy() {
        let masker = LogMasker::default()
            .with_strategy(MaskStrategy::Fixed(4));
        
        assert_eq!(
            masker.mask("password=admin123456"),
            "password=ad****56"
        );
    }

    #[test]
    fn test_custom_mask_char() {
        let masker = LogMasker::default()
            .with_mask_char('#');
        
        assert_eq!(
            masker.mask("password=admin"),
            "password=#####"
        );
    }

    #[test]
    fn test_add_custom_keyword() {
        let masker = LogMasker::default()
            .with_keyword("custom_field");
        
        assert_eq!(
            masker.mask("custom_field=sensitive_data"),
            "custom_field=**************"
        );
    }

    #[test]
    fn test_remove_keyword() {
        let masker = LogMasker::default()
            .without_keyword("password");
        
        assert_eq!(
            masker.mask("password=admin123"),
            "password=admin123"
        );
    }

    #[test]
    fn test_contains_sensitive_true() {
        let masker = LogMasker::default();
        
        assert!(masker.contains_sensitive("password=admin"));
        assert!(masker.contains_sensitive("token=xyz"));
    }

    #[test]
    fn test_contains_sensitive_false() {
        let masker = LogMasker::default();
        
        assert!(!masker.contains_sensitive("username=admin"));
        assert!(!masker.contains_sensitive("host=localhost"));
    }

    #[test]
    fn test_mask_database_url_with_credentials() {
        let masker = LogMasker::default();
        
        let url = "postgres://user=admin&password=secret123@db.example.com:5432/production";
        let result = masker.mask(url);
        
        assert!(!result.contains("secret123"));
    }

    #[test]
    fn test_mask_query_parameters() {
        let masker = LogMasker::default();
        
        let query = "password=test AND api_key=key123";
        let result = masker.mask(query);
        
        assert!(result.contains("password=****"));
        assert!(result.contains("api_key=*****"));
    }

    #[test]
    fn test_mask_simple_format() {
        let masker = LogMasker::default();
        
        let simple_str = "password=admin token=abc";
        let result = masker.mask(simple_str);
        
        assert!(!result.contains("password=admin"));
        assert!(result.contains("password="));
        assert!(!result.contains("token=abc"));
    }

    #[test]
    fn test_empty_input() {
        let masker = LogMasker::default();
        
        assert_eq!(masker.mask(""), "");
    }

    #[test]
    fn test_no_sensitive_data() {
        let masker = LogMasker::default();
        
        let safe_str = "username=admin host=localhost port=5432";
        assert_eq!(masker.mask(safe_str), safe_str);
    }

    #[test]
    fn test_delimiter_aware_masking() {
        let masker = LogMasker::default();
        
        let input = "password=admin&username=test;token=xyz,api_key=key";
        let result = masker.mask(input);
        
        assert_eq!(
            result,
            "password=*****&username=test;token=***,api_key=***"
        );
    }

    #[test]
    fn test_short_value_masking() {
        let masker = LogMasker::default()
            .with_strategy(MaskStrategy::Partial);
        
        assert_eq!(
            masker.mask("password=a"),
            "password=*"
        );
        
        assert_eq!(
            masker.mask("password=ab"),
            "password=**"
        );
    }
}
