//! Account Service
//!
//! 对照 Java NRCS Account 类实现完整的账户服务抽象。
//! 提供双余额模型、资产/货币余额操作、担保余额机制、
//! 账户租赁、事件通知、分类账记录等完整功能。
//!
//! 参考: /mnt/d/workspace/git/nrcs/nrcs-main/src/main/java/com/bytechain/nrcs/service/account/Account.java

use async_trait::async_trait;
use std::sync::Arc;
use thiserror::Error;

use blockchain_types::*;
use blockchain_types::prelude::Account;
use blockchain_types::account_ext::{AccountInfo, AccountProperty, AccountControlPhasing};
use orm::{
    AccountRepository, AccountAssetRepository, AccountCurrencyRepository,
    AccountGuaranteedBalanceRepository, AccountLedgerRepository, AccountInfoRepository,
    AccountLeaseRepository, AccountPropertyRepository, AccountControlPhasingRepository,
    PublicKeyRepository, RepositoryError,
};

#[derive(Debug, Error)]
pub enum AccountServiceError {
    #[error("account not found: {0}")]
    NotFound(AccountId),

    #[error("insufficient balance: have {have}, need {need}")]
    InsufficientBalance { have: Amount, need: Amount },

    #[error("insufficient asset balance: have {have}, need {need}")]
    InsufficientAssetBalance { have: Amount, need: Amount },

    #[error("insufficient currency balance: have {have}, need {need}")]
    InsufficientCurrencyBalance { have: Amount, need: Amount },

    #[error("double spending detected: account {account_id}, confirmed {confirmed}, unconfirmed {unconfirmed}")]
    DoubleSpending { account_id: AccountId, confirmed: i64, unconfirmed: i64 },

    #[error("public key mismatch for account {0}")]
    PublicKeyMismatch(AccountId),

    #[error("invalid operation: {0}")]
    InvalidOperation(String),

    #[error("repository error: {0}")]
    Repository(#[from] RepositoryError),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type AccountServiceResult<T> = std::result::Result<T, AccountServiceError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountEvent {
    Balance,
    UnconfirmedBalance,
    AssetBalance,
    UnconfirmedAssetBalance,
    CurrencyBalance,
    UnconfirmedCurrencyBalance,
    LeaseScheduled,
    LeaseStarted,
    LeaseEnded,
    SetProperty,
    DeleteProperty,
}

#[derive(Debug, Clone)]
pub struct AccountBalanceChange {
    pub account_id: AccountId,
    pub event: AccountEvent,
    pub balance: Amount,
    pub unconfirmed_balance: Amount,
}

pub trait AccountEventListener: Send + Sync {
    fn on_balance_change(&self, change: &AccountBalanceChange);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedgerHolding {
    UnconfirmedNrcsBalance = 1,
    NrcsBalance = 2,
    UnconfirmedAssetBalance = 3,
    AssetBalance = 4,
    UnconfirmedCurrencyBalance = 5,
    CurrencyBalance = 6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedgerEvent {
    BlockGenerated = 0,
    TransactionFee = 1,
    Payment = 2,
    Messaging = 3,
    AssetIssuance = 4,
    AssetTransfer = 5,
    AssetOrder = 6,
    DigitalGoods = 7,
    AccountControl = 8,
    MonetarySystem = 9,
    Data = 10,
    Shuffling = 11,
    Aliases = 12,
    Voting = 13,
    AccountProperty = 14,
    CoinExchange = 15,
    LightContract = 16,
}

#[derive(Debug, Clone)]
pub struct LedgerEntry {
    pub account_id: AccountId,
    pub event_type: LedgerEvent,
    pub event_id: i64,
    pub holding_type: LedgerHolding,
    pub holding_id: i64,
    pub change: i64,
    pub balance: i64,
    pub height: i32,
    pub timestamp: i32,
}

#[derive(Debug, Clone)]
pub struct AccountLeaseInfo {
    pub lessor_id: AccountId,
    pub current_lessee_id: Option<AccountId>,
    pub current_leasing_height_from: Option<i32>,
    pub current_leasing_height_to: Option<i32>,
    pub next_lessee_id: Option<AccountId>,
    pub next_leasing_height_from: Option<i32>,
    pub next_leasing_height_to: Option<i32>,
}

#[async_trait]
pub trait AccountService: Send + Sync {
    // ==================== 账户查询 ====================

    async fn get_account(&self, account_id: AccountId) -> AccountServiceResult<Account>;

    async fn get_account_by_public_key(&self, public_key: &[u8; 32]) -> AccountServiceResult<Option<Account>>;

    async fn get_or_create_account(&self, account_id: AccountId, public_key: &[u8; 32], height: i32) -> AccountServiceResult<Account>;

    async fn get_account_count(&self) -> AccountServiceResult<i64>;

    // ==================== NRCS 余额操作 ====================

    async fn get_balance(&self, account_id: AccountId) -> AccountServiceResult<Amount>;

    async fn get_unconfirmed_balance(&self, account_id: AccountId) -> AccountServiceResult<Amount>;

    async fn get_forged_balance(&self, account_id: AccountId) -> AccountServiceResult<Amount>;

    async fn add_to_balance(&self, account_id: AccountId, event: LedgerEvent, event_id: i64, amount: Amount, height: i32) -> AccountServiceResult<()>;

    async fn add_to_unconfirmed_balance(&self, account_id: AccountId, event: LedgerEvent, event_id: i64, amount: Amount, height: i32) -> AccountServiceResult<()>;

    async fn add_to_balance_and_unconfirmed(&self, account_id: AccountId, event: LedgerEvent, event_id: i64, amount: Amount, height: i32) -> AccountServiceResult<()>;

    async fn add_to_forged_balance(&self, account_id: AccountId, amount: Amount, height: i32) -> AccountServiceResult<()>;

    // ==================== 资产余额操作 ====================

    async fn get_asset_balance(&self, account_id: AccountId, asset_id: AssetId) -> AccountServiceResult<Amount>;

    async fn get_unconfirmed_asset_balance(&self, account_id: AccountId, asset_id: AssetId) -> AccountServiceResult<Amount>;

    async fn add_to_asset_balance(&self, account_id: AccountId, asset_id: AssetId, event: LedgerEvent, event_id: i64, quantity: Amount) -> AccountServiceResult<()>;

    async fn add_to_unconfirmed_asset_balance(&self, account_id: AccountId, asset_id: AssetId, event: LedgerEvent, event_id: i64, quantity: Amount) -> AccountServiceResult<()>;

    async fn add_to_asset_and_unconfirmed(&self, account_id: AccountId, asset_id: AssetId, event: LedgerEvent, event_id: i64, quantity: Amount) -> AccountServiceResult<()>;

    // ==================== 货币余额操作 ====================

    async fn get_currency_units(&self, account_id: AccountId, currency_id: CurrencyId) -> AccountServiceResult<Amount>;

    async fn get_unconfirmed_currency_units(&self, account_id: AccountId, currency_id: CurrencyId) -> AccountServiceResult<Amount>;

    async fn add_to_currency_units(&self, account_id: AccountId, currency_id: CurrencyId, event: LedgerEvent, event_id: i64, units: Amount) -> AccountServiceResult<()>;

    async fn add_to_unconfirmed_currency_units(&self, account_id: AccountId, currency_id: CurrencyId, event: LedgerEvent, event_id: i64, units: Amount) -> AccountServiceResult<()>;

    async fn add_to_currency_and_unconfirmed(&self, account_id: AccountId, currency_id: CurrencyId, event: LedgerEvent, event_id: i64, units: Amount) -> AccountServiceResult<()>;

    // ==================== 担保余额 ====================

    async fn get_guaranteed_balance(&self, account_id: AccountId, height: i32) -> AccountServiceResult<Amount>;

    async fn update_guaranteed_balance(&self, account_id: AccountId, height: i32, additions: Amount) -> AccountServiceResult<()>;

    // ==================== 有效余额 ====================

    async fn get_effective_balance(&self, account_id: AccountId, height: i32) -> AccountServiceResult<Amount>;

    // ==================== 公钥管理 ====================

    async fn get_public_key(&self, account_id: AccountId) -> AccountServiceResult<Option<[u8; 32]>>;

    async fn set_or_verify_public_key(&self, account_id: AccountId, public_key: &[u8; 32], height: i32) -> AccountServiceResult<()>;

    async fn apply_public_key(&self, account_id: AccountId, public_key: &[u8; 32], height: i32) -> AccountServiceResult<()>;

    // ==================== 账户信息 ====================

    async fn get_account_info(&self, account_id: AccountId) -> AccountServiceResult<Option<AccountInfo>>;

    async fn set_account_info(&self, account_id: AccountId, name: &str, description: &str, height: i32) -> AccountServiceResult<()>;

    // ==================== 账户属性 ====================

    async fn get_account_property(&self, property_id: i64) -> AccountServiceResult<Option<AccountProperty>>;

    async fn get_account_properties(&self, account_id: AccountId) -> AccountServiceResult<Vec<AccountProperty>>;

    async fn set_account_property(&self, property_id: i64, recipient_id: AccountId, setter_id: AccountId, property: &str, value: &str, height: i32) -> AccountServiceResult<()>;

    async fn delete_account_property(&self, property_id: i64) -> AccountServiceResult<()>;

    // ==================== 账户租赁 ====================

    async fn get_account_lease(&self, lessor_id: AccountId) -> AccountServiceResult<Option<AccountLeaseInfo>>;

    async fn set_account_lease(&self, lessor_id: AccountId, lessee_id: AccountId, start_height: i32, end_height: i32, height: i32) -> AccountServiceResult<()>;

    // ==================== 账户控制 ====================

    async fn get_account_control_phasing(&self, account_id: AccountId) -> AccountServiceResult<Option<AccountControlPhasing>>;

    async fn set_account_control_phasing(&self, model: &AccountControlPhasing) -> AccountServiceResult<()>;

    // ==================== 分类账 ====================

    async fn log_ledger_entry(&self, entry: &LedgerEntry) -> AccountServiceResult<()>;

    async fn get_ledger_entries(&self, account_id: AccountId, limit: i64) -> AccountServiceResult<Vec<orm::AccountLedgerModel>>;

    // ==================== 事件监听 ====================

    fn add_listener(&self, listener: Arc<dyn AccountEventListener>, event: AccountEvent);

    fn remove_listener(&self, listener: &Arc<dyn AccountEventListener>, event: AccountEvent);
}

pub struct DatabaseAccountService {
    account_repo: Arc<dyn AccountRepository>,
    account_asset_repo: Arc<dyn AccountAssetRepository>,
    account_currency_repo: Arc<dyn AccountCurrencyRepository>,
    guaranteed_balance_repo: Arc<dyn AccountGuaranteedBalanceRepository>,
    ledger_repo: Arc<dyn AccountLedgerRepository>,
    account_info_repo: Arc<dyn AccountInfoRepository>,
    account_lease_repo: Arc<dyn AccountLeaseRepository>,
    account_property_repo: Arc<dyn AccountPropertyRepository>,
    account_control_phasing_repo: Arc<dyn AccountControlPhasingRepository>,
    public_key_repo: Arc<dyn PublicKeyRepository>,
    listeners: std::sync::Mutex<Vec<(AccountEvent, Arc<dyn AccountEventListener>)>>,
}

impl DatabaseAccountService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        account_repo: Arc<dyn AccountRepository>,
        account_asset_repo: Arc<dyn AccountAssetRepository>,
        account_currency_repo: Arc<dyn AccountCurrencyRepository>,
        guaranteed_balance_repo: Arc<dyn AccountGuaranteedBalanceRepository>,
        ledger_repo: Arc<dyn AccountLedgerRepository>,
        account_info_repo: Arc<dyn AccountInfoRepository>,
        account_lease_repo: Arc<dyn AccountLeaseRepository>,
        account_property_repo: Arc<dyn AccountPropertyRepository>,
        account_control_phasing_repo: Arc<dyn AccountControlPhasingRepository>,
        public_key_repo: Arc<dyn PublicKeyRepository>,
    ) -> Self {
        Self {
            account_repo,
            account_asset_repo,
            account_currency_repo,
            guaranteed_balance_repo,
            ledger_repo,
            account_info_repo,
            account_lease_repo,
            account_property_repo,
            account_control_phasing_repo,
            public_key_repo,
            listeners: std::sync::Mutex::new(Vec::new()),
        }
    }

    fn notify_listeners(&self, change: &AccountBalanceChange) {
        if let Ok(listeners) = self.listeners.lock() {
            for (event, listener) in listeners.iter() {
                if *event == change.event {
                    listener.on_balance_change(change);
                }
            }
        }
    }

    #[allow(dead_code)]
    fn check_balance(&self, account_id: AccountId, confirmed: i64, unconfirmed: i64) -> AccountServiceResult<()> {
        if confirmed < 0 {
            return Err(AccountServiceError::DoubleSpending {
                account_id,
                confirmed,
                unconfirmed,
            });
        }
        if unconfirmed < 0 {
            return Err(AccountServiceError::DoubleSpending {
                account_id,
                confirmed,
                unconfirmed,
            });
        }
        if unconfirmed > confirmed {
            return Err(AccountServiceError::DoubleSpending {
                account_id,
                confirmed,
                unconfirmed,
            });
        }
        Ok(())
    }
}

#[async_trait]
impl AccountService for DatabaseAccountService {
    async fn get_account(&self, account_id: AccountId) -> AccountServiceResult<Account> {
        let model = self.account_repo.find_by_account_id(account_id as i64).await?
            .ok_or_else(|| AccountServiceError::NotFound(account_id))?;
        let account = model.to_domain()
            .map_err(|e| AccountServiceError::Repository(RepositoryError::Blockchain(e)))?;
        Ok(account)
    }

    async fn get_account_by_public_key(&self, public_key: &[u8; 32]) -> AccountServiceResult<Option<Account>> {
        let account_id = account_id_from_public_key(public_key);
        match self.account_repo.find_by_account_id(account_id as i64).await? {
            Some(model) => {
                let account = model.to_domain()
                    .map_err(|e| AccountServiceError::Repository(RepositoryError::Blockchain(e)))?;
                Ok(Some(account))
            }
            None => Ok(None),
        }
    }

    async fn get_or_create_account(&self, account_id: AccountId, public_key: &[u8; 32], height: i32) -> AccountServiceResult<Account> {
        let model = self.account_repo.get_or_create(account_id as i64).await?;
        let pk_opt = self.public_key_repo.find_latest_by_account_id(account_id as i64).await?;
        if pk_opt.is_none() {
            let pk_model = blockchain_types::account_ext::AccountPublicKey {
                account_id,
                public_key: *public_key,
                height: height as u32,
            };
            self.public_key_repo.insert(&pk_model).await?;
        }
        let account = model.to_domain()
            .map_err(|e| AccountServiceError::Repository(RepositoryError::Blockchain(e)))?;
        Ok(account)
    }

    async fn get_account_count(&self) -> AccountServiceResult<i64> {
        Ok(self.account_repo.get_account_count().await?)
    }

    async fn get_balance(&self, account_id: AccountId) -> AccountServiceResult<Amount> {
        let account = self.get_account(account_id).await?;
        Ok(account.balance)
    }

    async fn get_unconfirmed_balance(&self, account_id: AccountId) -> AccountServiceResult<Amount> {
        let account = self.get_account(account_id).await?;
        Ok(account.unconfirmed_balance)
    }

    async fn get_forged_balance(&self, account_id: AccountId) -> AccountServiceResult<Amount> {
        let account = self.get_account(account_id).await?;
        Ok(account.forged_balance)
    }

    async fn add_to_balance(&self, account_id: AccountId, _event: LedgerEvent, _event_id: i64, amount: Amount, height: i32) -> AccountServiceResult<()> {
        if amount == 0 {
            return Ok(());
        }
        self.account_repo.add_to_balance(account_id as i64, amount as i64, height).await?;
        self.notify_listeners(&AccountBalanceChange {
            account_id,
            event: AccountEvent::Balance,
            balance: amount,
            unconfirmed_balance: 0,
        });
        Ok(())
    }

    async fn add_to_unconfirmed_balance(&self, account_id: AccountId, _event: LedgerEvent, _event_id: i64, amount: Amount, height: i32) -> AccountServiceResult<()> {
        if amount == 0 {
            return Ok(());
        }
        self.account_repo.add_to_unconfirmed_balance(account_id as i64, amount as i64, height).await?;
        self.notify_listeners(&AccountBalanceChange {
            account_id,
            event: AccountEvent::UnconfirmedBalance,
            balance: 0,
            unconfirmed_balance: amount,
        });
        Ok(())
    }

    async fn add_to_balance_and_unconfirmed(&self, account_id: AccountId, _event: LedgerEvent, _event_id: i64, amount: Amount, height: i32) -> AccountServiceResult<()> {
        if amount == 0 {
            return Ok(());
        }
        self.account_repo.add_to_balance_and_unconfirmed(account_id as i64, amount as i64, height).await?;
        self.notify_listeners(&AccountBalanceChange {
            account_id,
            event: AccountEvent::Balance,
            balance: amount,
            unconfirmed_balance: amount,
        });
        Ok(())
    }

    async fn add_to_forged_balance(&self, account_id: AccountId, amount: Amount, height: i32) -> AccountServiceResult<()> {
        if amount == 0 {
            return Ok(());
        }
        self.account_repo.add_to_forged_balance(account_id as i64, amount as i64, height).await?;
        Ok(())
    }

    async fn get_asset_balance(&self, account_id: AccountId, asset_id: AssetId) -> AccountServiceResult<Amount> {
        match self.account_asset_repo.find_by_account_and_asset(account_id as i64, asset_id as i64).await? {
            Some(model) => Ok(model.quantity as Amount),
            None => Ok(0),
        }
    }

    async fn get_unconfirmed_asset_balance(&self, account_id: AccountId, asset_id: AssetId) -> AccountServiceResult<Amount> {
        match self.account_asset_repo.find_by_account_and_asset(account_id as i64, asset_id as i64).await? {
            Some(model) => Ok(model.unconfirmed_quantity as Amount),
            None => Ok(0),
        }
    }

    async fn add_to_asset_balance(&self, account_id: AccountId, asset_id: AssetId, _event: LedgerEvent, _event_id: i64, quantity: Amount) -> AccountServiceResult<()> {
        if quantity == 0 {
            return Ok(());
        }
        self.account_asset_repo.increase_quantity(account_id as i64, asset_id as i64, quantity as i64).await?;
        self.notify_listeners(&AccountBalanceChange {
            account_id,
            event: AccountEvent::AssetBalance,
            balance: quantity,
            unconfirmed_balance: 0,
        });
        Ok(())
    }

    async fn add_to_unconfirmed_asset_balance(&self, account_id: AccountId, asset_id: AssetId, _event: LedgerEvent, _event_id: i64, quantity: Amount) -> AccountServiceResult<()> {
        if quantity == 0 {
            return Ok(());
        }
        if quantity > 0 {
            self.account_asset_repo.add_to_unconfirmed_quantity(account_id as i64, asset_id as i64, quantity as i64).await?;
        } else {
            self.account_asset_repo.decrease_quantity(account_id as i64, asset_id as i64, (quantity as i64).abs()).await?;
        }
        self.notify_listeners(&AccountBalanceChange {
            account_id,
            event: AccountEvent::UnconfirmedAssetBalance,
            balance: 0,
            unconfirmed_balance: quantity,
        });
        Ok(())
    }

    async fn add_to_asset_and_unconfirmed(&self, account_id: AccountId, asset_id: AssetId, event: LedgerEvent, event_id: i64, quantity: Amount) -> AccountServiceResult<()> {
        self.add_to_asset_balance(account_id, asset_id, event, event_id, quantity).await?;
        self.add_to_unconfirmed_asset_balance(account_id, asset_id, event, event_id, quantity).await?;
        Ok(())
    }

    async fn get_currency_units(&self, account_id: AccountId, currency_id: CurrencyId) -> AccountServiceResult<Amount> {
        match self.account_currency_repo.find_by_account_and_currency(account_id as i64, currency_id as i64).await? {
            Some(model) => Ok(model.units as Amount),
            None => Ok(0),
        }
    }

    async fn get_unconfirmed_currency_units(&self, account_id: AccountId, currency_id: CurrencyId) -> AccountServiceResult<Amount> {
        match self.account_currency_repo.find_by_account_and_currency(account_id as i64, currency_id as i64).await? {
            Some(model) => Ok(model.unconfirmed_units as Amount),
            None => Ok(0),
        }
    }

    async fn add_to_currency_units(&self, account_id: AccountId, currency_id: CurrencyId, _event: LedgerEvent, _event_id: i64, units: Amount) -> AccountServiceResult<()> {
        if units == 0 {
            return Ok(());
        }
        self.account_currency_repo.update_units(account_id as i64, currency_id as i64, units as i64).await?;
        self.notify_listeners(&AccountBalanceChange {
            account_id,
            event: AccountEvent::CurrencyBalance,
            balance: units,
            unconfirmed_balance: 0,
        });
        Ok(())
    }

    async fn add_to_unconfirmed_currency_units(&self, account_id: AccountId, currency_id: CurrencyId, _event: LedgerEvent, _event_id: i64, units: Amount) -> AccountServiceResult<()> {
        if units == 0 {
            return Ok(());
        }
        self.account_currency_repo.add_to_unconfirmed_units(account_id as i64, currency_id as i64, units as i64).await?;
        self.notify_listeners(&AccountBalanceChange {
            account_id,
            event: AccountEvent::UnconfirmedCurrencyBalance,
            balance: 0,
            unconfirmed_balance: units,
        });
        Ok(())
    }

    async fn add_to_currency_and_unconfirmed(&self, account_id: AccountId, currency_id: CurrencyId, event: LedgerEvent, event_id: i64, units: Amount) -> AccountServiceResult<()> {
        self.add_to_currency_units(account_id, currency_id, event, event_id, units).await?;
        self.add_to_unconfirmed_currency_units(account_id, currency_id, event, event_id, units).await?;
        Ok(())
    }

    async fn get_guaranteed_balance(&self, account_id: AccountId, height: i32) -> AccountServiceResult<Amount> {
        let confirmations = GUARANTEED_BALANCE_CONFIRMATIONS as i32;
        let since_height = height.saturating_sub(confirmations);
        let additions = self.guaranteed_balance_repo
            .get_total_additions_since(account_id as i64, since_height, height)
            .await
            .unwrap_or(0);

        let account = self.get_account(account_id).await?;
        let guaranteed = (account.balance as i64).saturating_sub(additions);
        Ok(guaranteed.max(0) as Amount)
    }

    async fn update_guaranteed_balance(&self, account_id: AccountId, height: i32, additions: Amount) -> AccountServiceResult<()> {
        self.guaranteed_balance_repo.upsert_additions(account_id as i64, height, additions as i64).await?;
        Ok(())
    }

    async fn get_effective_balance(&self, account_id: AccountId, height: i32) -> AccountServiceResult<Amount> {
        let pk_opt = self.public_key_repo.find_latest_by_account_id(account_id as i64).await?;
        let pk = match pk_opt {
            Some(pk) => pk,
            None => return Ok(0),
        };

        if pk.public_key == [0u8; 32] {
            return Ok(0);
        }

        let pk_height = pk.height as i32;
        if height - pk_height < GUARANTEED_BALANCE_CONFIRMATIONS as i32 {
            return Ok(0);
        }

        let guaranteed = self.get_guaranteed_balance(account_id, height).await?;

        let effective = guaranteed / ONE_NRCS;
        if effective < MIN_FORGING_BALANCE_NQT / ONE_NRCS {
            return Ok(0);
        }

        Ok(effective * ONE_NRCS)
    }

    async fn get_public_key(&self, account_id: AccountId) -> AccountServiceResult<Option<[u8; 32]>> {
        let pk_opt = self.public_key_repo.find_latest_by_account_id(account_id as i64).await?;
        Ok(pk_opt.map(|pk| pk.public_key))
    }

    async fn set_or_verify_public_key(&self, account_id: AccountId, public_key: &[u8; 32], height: i32) -> AccountServiceResult<()> {
        let pk_opt = self.public_key_repo.find_latest_by_account_id(account_id as i64).await?;
        match pk_opt {
            Some(existing) => {
                if existing.public_key == [0u8; 32] {
                    let pk_model = blockchain_types::account_ext::AccountPublicKey {
                        account_id,
                        public_key: *public_key,
                        height: height as u32,
                    };
                    self.public_key_repo.insert(&pk_model).await?;
                } else if existing.public_key != *public_key {
                    return Err(AccountServiceError::PublicKeyMismatch(account_id));
                }
            }
            None => {
                let pk_model = blockchain_types::account_ext::AccountPublicKey {
                    account_id,
                    public_key: *public_key,
                    height: height as u32,
                };
                self.public_key_repo.insert(&pk_model).await?;
            }
        }
        Ok(())
    }

    async fn apply_public_key(&self, account_id: AccountId, public_key: &[u8; 32], height: i32) -> AccountServiceResult<()> {
        self.set_or_verify_public_key(account_id, public_key, height).await
    }

    async fn get_account_info(&self, account_id: AccountId) -> AccountServiceResult<Option<AccountInfo>> {
        let info_opt = self.account_info_repo.find_by_account(account_id as i64).await?;
        Ok(info_opt.map(|m| AccountInfo {
            account_id,
            name: m.name,
            description: m.description,
            height: m.height as u32,
        }))
    }

    async fn set_account_info(&self, account_id: AccountId, name: &str, description: &str, height: i32) -> AccountServiceResult<()> {
        let model = orm::AccountInfoModel {
            db_id: 0,
            account_id: account_id as i64,
            name: Some(name.to_string()),
            description: Some(description.to_string()),
            height,
            latest: true,
        };
        self.account_info_repo.upsert(&model).await?;
        Ok(())
    }

    async fn get_account_property(&self, property_id: i64) -> AccountServiceResult<Option<AccountProperty>> {
        let prop = self.account_property_repo.find_by_id(property_id).await?;
        Ok(prop.map(|p| AccountProperty {
            id: p.id as u64,
            recipient_id: p.recipient_id as AccountId,
            setter_id: p.setter_id.unwrap_or(0) as AccountId,
            property: p.property,
            value: p.value,
            height: p.height as u32,
        }))
    }

    async fn get_account_properties(&self, account_id: AccountId) -> AccountServiceResult<Vec<AccountProperty>> {
        let props = self.account_property_repo.find_by_account(account_id as i64).await?;
        Ok(props.into_iter().map(|p| AccountProperty {
            id: p.id as u64,
            recipient_id: p.recipient_id as AccountId,
            setter_id: p.setter_id.unwrap_or(0) as AccountId,
            property: p.property,
            value: p.value,
            height: p.height as u32,
        }).collect())
    }

    async fn set_account_property(&self, property_id: i64, recipient_id: AccountId, setter_id: AccountId, property: &str, value: &str, height: i32) -> AccountServiceResult<()> {
        let model = orm::AccountPropertyModel {
            db_id: 0,
            id: property_id,
            recipient_id: recipient_id as i64,
            setter_id: Some(setter_id as i64),
            property: property.to_string(),
            value: Some(value.to_string()),
            height,
            latest: true,
        };
        self.account_property_repo.upsert(&model).await?;
        self.notify_listeners(&AccountBalanceChange {
            account_id: recipient_id,
            event: AccountEvent::SetProperty,
            balance: 0,
            unconfirmed_balance: 0,
        });
        Ok(())
    }

    async fn delete_account_property(&self, property_id: i64) -> AccountServiceResult<()> {
        self.account_property_repo.delete_by_id(property_id).await?;
        Ok(())
    }

    async fn get_account_lease(&self, lessor_id: AccountId) -> AccountServiceResult<Option<AccountLeaseInfo>> {
        let lease_opt = self.account_lease_repo.find_by_account(lessor_id as i64).await?;
        Ok(lease_opt.map(|l| AccountLeaseInfo {
            lessor_id,
            current_lessee_id: l.current_lessee_id.map(|id| id as AccountId),
            current_leasing_height_from: l.current_leasing_height_from,
            current_leasing_height_to: l.current_leasing_height_to,
            next_lessee_id: l.next_lessee_id.map(|id| id as AccountId),
            next_leasing_height_from: l.next_leasing_height_from,
            next_leasing_height_to: l.next_leasing_height_to,
        }))
    }

    async fn set_account_lease(&self, lessor_id: AccountId, lessee_id: AccountId, start_height: i32, end_height: i32, height: i32) -> AccountServiceResult<()> {
        let model = orm::AccountLeaseModel {
            db_id: 0,
            lessor_id: lessor_id as i64,
            current_leasing_height_from: None,
            current_leasing_height_to: None,
            current_lessee_id: None,
            next_leasing_height_from: Some(start_height),
            next_leasing_height_to: Some(end_height),
            next_lessee_id: Some(lessee_id as i64),
            height,
            latest: true,
        };
        self.account_lease_repo.upsert(&model).await?;
        self.notify_listeners(&AccountBalanceChange {
            account_id: lessor_id,
            event: AccountEvent::LeaseScheduled,
            balance: 0,
            unconfirmed_balance: 0,
        });
        Ok(())
    }

    async fn get_account_control_phasing(&self, account_id: AccountId) -> AccountServiceResult<Option<AccountControlPhasing>> {
        let control_opt = self.account_control_phasing_repo.find_by_account(account_id as i64).await?;
        Ok(control_opt.map(|c| AccountControlPhasing {
            account_id,
            voting_model: c.voting_model as u8,
            quorum: c.quorum.unwrap_or(0),
            min_balance: c.min_balance.unwrap_or(0),
            min_balance_model: c.min_balance_model.unwrap_or(0) as u8,
            holding_id: c.holding_id.map(|id| id as u64),
            whitelist: vec![],
            height: c.height as u32,
        }))
    }

    async fn set_account_control_phasing(&self, model: &AccountControlPhasing) -> AccountServiceResult<()> {
        let db_model = orm::AccountControlPhasingModel {
            db_id: 0,
            account_id: model.account_id as i64,
            whitelist: None,
            voting_model: model.voting_model as i16,
            quorum: Some(model.quorum),
            min_balance: Some(model.min_balance),
            holding_id: model.holding_id.map(|id| id as i64),
            min_balance_model: Some(model.min_balance_model as i16),
            max_fees: None,
            min_duration: None,
            max_duration: None,
            height: model.height as i32,
            latest: true,
        };
        self.account_control_phasing_repo.upsert(&db_model).await?;
        Ok(())
    }

    async fn log_ledger_entry(&self, entry: &LedgerEntry) -> AccountServiceResult<()> {
        let model = orm::AccountLedgerModel {
            db_id: 0,
            account_id: entry.account_id as i64,
            event_type: entry.event_type as i16,
            event_id: entry.event_id,
            holding_type: entry.holding_type as i16,
            holding_id: Some(entry.holding_id),
            change: entry.change,
            balance: entry.balance,
            block_id: 0,
            height: entry.height,
            timestamp: entry.timestamp,
        };
        self.ledger_repo.insert(&model).await?;
        Ok(())
    }

    async fn get_ledger_entries(&self, account_id: AccountId, limit: i64) -> AccountServiceResult<Vec<orm::AccountLedgerModel>> {
        Ok(self.ledger_repo.find_by_account(account_id as i64, limit).await?)
    }

    fn add_listener(&self, listener: Arc<dyn AccountEventListener>, event: AccountEvent) {
        if let Ok(mut listeners) = self.listeners.lock() {
            listeners.push((event, listener));
        }
    }

    fn remove_listener(&self, listener: &Arc<dyn AccountEventListener>, event: AccountEvent) {
        if let Ok(mut listeners) = self.listeners.lock() {
            listeners.retain(|(e, l)| !(*e == event && Arc::ptr_eq(l, listener)));
        }
    }
}

fn account_id_from_public_key(public_key: &[u8; 32]) -> AccountId {
    blockchain_types::block::account_id_from_public_key(public_key)
}
