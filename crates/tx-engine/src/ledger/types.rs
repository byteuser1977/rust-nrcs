//! Ledger Types
//!
//! 对应 Java: LedgerEvent.java, LedgerHolding.java

use std::collections::HashMap;
use std::sync::OnceLock;

/// Ledger Event Types
///
/// 对应 Java: LedgerEvent.java
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i16)]
pub enum LedgerEvent {
    BlockGenerated = 1,
    RejectPhasedTransaction = 2,
    TransactionFee = 50,
    OrdinaryPayment = 3,
    AccountInfo = 4,
    AliasAssignment = 5,
    AliasBuy = 6,
    AliasDelete = 7,
    AliasSell = 8,
    ArbitraryMessage = 9,
    HubAnnouncement = 10,
    PhasingVoteCasting = 11,
    PollCreation = 12,
    VoteCasting = 13,
    AccountProperty = 56,
    AccountPropertyDelete = 57,
    AccountPropertySet = 656,
    AccountLongValuePropertySet = 68,
    AssetAskOrderCancellation = 14,
    AssetAskOrderPlacement = 15,
    AssetBidOrderCancellation = 16,
    AssetBidOrderPlacement = 17,
    AssetDividendPayment = 18,
    AssetIssuance = 19,
    AssetTrade = 20,
    AssetTransfer = 21,
    AssetDelete = 49,
    AssetPropertySet = 65,
    AssetPropertyDelete = 66,
    AssetIncrease = 61,
    AssetSetPhasingControl = 62,
    AssetLongValuePropertySet = 67,
    DigitalGoodsDelisted = 22,
    DigitalGoodsDelisting = 23,
    DigitalGoodsDelivery = 24,
    DigitalGoodsFeedback = 25,
    DigitalGoodsListing = 26,
    DigitalGoodsPriceChange = 27,
    DigitalGoodsPurchase = 28,
    DigitalGoodsPurchaseExpired = 29,
    DigitalGoodsQuantityChange = 30,
    DigitalGoodsRefund = 31,
    AccountControlEffectiveBalanceLeasing = 32,
    AccountControlPhasingOnly = 55,
    CurrencyDeletion = 33,
    CurrencyDistribution = 34,
    CurrencyExchange = 35,
    CurrencyExchangeBuy = 36,
    CurrencyExchangeSell = 37,
    CurrencyIssuance = 38,
    CurrencyMinting = 39,
    CurrencyOfferExpired = 40,
    CurrencyOfferReplaced = 41,
    CurrencyPublishExchangeOffer = 42,
    CurrencyReserveClaim = 43,
    CurrencyReserveIncrease = 44,
    CurrencyTransfer = 45,
    CurrencyUndoCrowdfunding = 46,
    TaggedDataUpload = 47,
    TaggedDataExtend = 48,
    ShufflingRegistration = 51,
    ShufflingProcessing = 52,
    ShufflingCancellation = 53,
    ShufflingDistribution = 54,
    CoinExchangeOrderIssue = 58,
    CoinExchangeOrderCancel = 59,
    CoinExchangeTrade = 60,
    ContractReferenceSet = 63,
    ContractReferenceDelete = 64,
}

impl LedgerEvent {
    /// Get event from code
    pub fn from_code(code: i16) -> Option<Self> {
        get_event_map().get(&code).copied()
    }
    
    /// Get the event code
    pub fn code(&self) -> i16 {
        *self as i16
    }
    
    /// Check if the event identifier is a transaction
    pub fn is_transaction(&self) -> bool {
        !matches!(self, LedgerEvent::BlockGenerated)
    }
}

static EVENT_MAP: OnceLock<HashMap<i16, LedgerEvent>> = OnceLock::new();

fn get_event_map() -> &'static HashMap<i16, LedgerEvent> {
    EVENT_MAP.get_or_init(|| {
        let mut map = HashMap::new();
        for event in [
            LedgerEvent::BlockGenerated,
            LedgerEvent::RejectPhasedTransaction,
            LedgerEvent::TransactionFee,
            LedgerEvent::OrdinaryPayment,
            LedgerEvent::AccountInfo,
            LedgerEvent::AliasAssignment,
            LedgerEvent::AliasBuy,
            LedgerEvent::AliasDelete,
            LedgerEvent::AliasSell,
            LedgerEvent::ArbitraryMessage,
            LedgerEvent::HubAnnouncement,
            LedgerEvent::PhasingVoteCasting,
            LedgerEvent::PollCreation,
            LedgerEvent::VoteCasting,
            LedgerEvent::AccountProperty,
            LedgerEvent::AccountPropertyDelete,
            LedgerEvent::AccountPropertySet,
            LedgerEvent::AccountLongValuePropertySet,
            LedgerEvent::AssetAskOrderCancellation,
            LedgerEvent::AssetAskOrderPlacement,
            LedgerEvent::AssetBidOrderCancellation,
            LedgerEvent::AssetBidOrderPlacement,
            LedgerEvent::AssetDividendPayment,
            LedgerEvent::AssetIssuance,
            LedgerEvent::AssetTrade,
            LedgerEvent::AssetTransfer,
            LedgerEvent::AssetDelete,
            LedgerEvent::AssetPropertySet,
            LedgerEvent::AssetPropertyDelete,
            LedgerEvent::AssetIncrease,
            LedgerEvent::AssetSetPhasingControl,
            LedgerEvent::AssetLongValuePropertySet,
            LedgerEvent::DigitalGoodsDelisted,
            LedgerEvent::DigitalGoodsDelisting,
            LedgerEvent::DigitalGoodsDelivery,
            LedgerEvent::DigitalGoodsFeedback,
            LedgerEvent::DigitalGoodsListing,
            LedgerEvent::DigitalGoodsPriceChange,
            LedgerEvent::DigitalGoodsPurchase,
            LedgerEvent::DigitalGoodsPurchaseExpired,
            LedgerEvent::DigitalGoodsQuantityChange,
            LedgerEvent::DigitalGoodsRefund,
            LedgerEvent::AccountControlEffectiveBalanceLeasing,
            LedgerEvent::AccountControlPhasingOnly,
            LedgerEvent::CurrencyDeletion,
            LedgerEvent::CurrencyDistribution,
            LedgerEvent::CurrencyExchange,
            LedgerEvent::CurrencyExchangeBuy,
            LedgerEvent::CurrencyExchangeSell,
            LedgerEvent::CurrencyIssuance,
            LedgerEvent::CurrencyMinting,
            LedgerEvent::CurrencyOfferExpired,
            LedgerEvent::CurrencyOfferReplaced,
            LedgerEvent::CurrencyPublishExchangeOffer,
            LedgerEvent::CurrencyReserveClaim,
            LedgerEvent::CurrencyReserveIncrease,
            LedgerEvent::CurrencyTransfer,
            LedgerEvent::CurrencyUndoCrowdfunding,
            LedgerEvent::TaggedDataUpload,
            LedgerEvent::TaggedDataExtend,
            LedgerEvent::ShufflingRegistration,
            LedgerEvent::ShufflingProcessing,
            LedgerEvent::ShufflingCancellation,
            LedgerEvent::ShufflingDistribution,
            LedgerEvent::CoinExchangeOrderIssue,
            LedgerEvent::CoinExchangeOrderCancel,
            LedgerEvent::CoinExchangeTrade,
            LedgerEvent::ContractReferenceSet,
            LedgerEvent::ContractReferenceDelete,
        ] {
            map.insert(event.code(), event);
        }
        map
    })
}

/// Ledger Holding Types
///
/// 对应 Java: LedgerHolding.java
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i16)]
pub enum LedgerHolding {
    UnconfirmedNrcsBalance = 1,
    NrcsBalance = 2,
    UnconfirmedAssetBalance = 3,
    AssetBalance = 4,
    UnconfirmedCurrencyBalance = 5,
    CurrencyBalance = 6,
}

impl LedgerHolding {
    /// Get holding from code
    pub fn from_code(code: i16) -> Option<Self> {
        get_holding_map().get(&code).copied()
    }
    
    /// Get the holding code
    pub fn code(&self) -> i16 {
        *self as i16
    }
    
    /// Check if the holding is unconfirmed
    pub fn is_unconfirmed(&self) -> bool {
        matches!(
            self,
            LedgerHolding::UnconfirmedNrcsBalance
                | LedgerHolding::UnconfirmedAssetBalance
                | LedgerHolding::UnconfirmedCurrencyBalance
        )
    }
}

static HOLDING_MAP: OnceLock<HashMap<i16, LedgerHolding>> = OnceLock::new();

fn get_holding_map() -> &'static HashMap<i16, LedgerHolding> {
    HOLDING_MAP.get_or_init(|| {
        let mut map = HashMap::new();
        for holding in [
            LedgerHolding::UnconfirmedNrcsBalance,
            LedgerHolding::NrcsBalance,
            LedgerHolding::UnconfirmedAssetBalance,
            LedgerHolding::AssetBalance,
            LedgerHolding::UnconfirmedCurrencyBalance,
            LedgerHolding::CurrencyBalance,
        ] {
            map.insert(holding.code(), holding);
        }
        map
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ledger_event_from_code() {
        assert_eq!(LedgerEvent::from_code(1), Some(LedgerEvent::BlockGenerated));
        assert_eq!(LedgerEvent::from_code(3), Some(LedgerEvent::OrdinaryPayment));
        assert_eq!(LedgerEvent::from_code(999), None);
    }

    #[test]
    fn test_ledger_event_is_transaction() {
        assert!(!LedgerEvent::BlockGenerated.is_transaction());
        assert!(LedgerEvent::OrdinaryPayment.is_transaction());
    }

    #[test]
    fn test_ledger_holding_from_code() {
        assert_eq!(
            LedgerHolding::from_code(1),
            Some(LedgerHolding::UnconfirmedNrcsBalance)
        );
        assert_eq!(LedgerHolding::from_code(2), Some(LedgerHolding::NrcsBalance));
        assert_eq!(LedgerHolding::from_code(999), None);
    }

    #[test]
    fn test_ledger_holding_is_unconfirmed() {
        assert!(LedgerHolding::UnconfirmedNrcsBalance.is_unconfirmed());
        assert!(!LedgerHolding::NrcsBalance.is_unconfirmed());
    }
}
