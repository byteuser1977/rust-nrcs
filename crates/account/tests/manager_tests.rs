use account::*;
use blockchain_types::*;
use orm::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Mock Repository for testing
#[derive(Clone, Default)]
struct MockAccountRepository {
    accounts: Arc<Mutex<HashMap<AccountId, Account>>>,
}

#[async_trait]
impl AccountRepository for MockAccountRepository {
    async fn find_by_id(&self, id: AccountId) -> RepositoryResult<Option<Account>> {
        Ok(self.accounts.lock().unwrap().get(&id).cloned())
    }

    async fn save(&self, account: &Account) -> RepositoryResult<()> {
        self.accounts.lock().unwrap().insert(account.id, account.clone());
        Ok(())
    }

    async fn delete(&self, id: AccountId) -> RepositoryResult<()> {
        self.accounts.lock().unwrap().remove(&id);
        Ok(())
    }

    async fn exists(&self, id: AccountId) -> RepositoryResult<bool> {
        Ok(self.accounts.lock().unwrap().contains_key(&id))
    }

    async fn count(&self) -> RepositoryResult<usize> {
        Ok(self.accounts.lock().unwrap().len())
    }
}

#[derive(Clone, Default)]
struct MockNonceRepository {
    nonces: Arc<Mutex<HashMap<AccountId, u64>>>,
}

#[async_trait]
impl NonceRepository for MockNonceRepository {
    async fn get_nonce(&self, account_id: AccountId) -> RepositoryResult<u64> {
        Ok(self.nonces.lock().unwrap().get(&account_id).copied().unwrap_or(0))
    }

    async fn increment_nonce(&self, account_id: AccountId) -> RepositoryResult<u64> {
        let mut nonces = self.nonces.lock().unwrap();
        let current = nonces.get(&account_id).copied().unwrap_or(0);
        let next = current + 1;
        nonces.insert(account_id, next);
        Ok(next)
    }

    async fn set_nonce(&self, account_id: AccountId, nonce: u64) -> RepositoryResult<()> {
        self.nonces.lock().unwrap().insert(account_id, nonce);
        Ok(())
    }
}

// Mock AccountManager implementation for testing
#[derive(Clone)]
struct MockAccountManager {
    accounts: Arc<Mutex<HashMap<AccountId, Account>>>,
    nonces: Arc<Mutex<HashMap<AccountId, u64>>>,
    config: AccountConfig,
}

impl MockAccountManager {
    fn new(config: AccountConfig) -> Self {
        Self {
            accounts: Arc::new(Mutex::new(HashMap::new())),
            nonces: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }
}

#[async_trait]
impl AccountManager for MockAccountManager {
    async fn create_account(&self, initial_balance: Option<Amount>) -> AccountResult<(Keypair, AccountId, String)> {
        use account::crypto::{generate_keypair, AddressGenerator};

        let kp = generate_keypair();
        let account_id = derive_account_id(&kp.public);
        let address = if self.config.enable_address {
            derive_address(account_id)
        } else {
            format!("{}", account_id)
        };

        let balance = initial_balance.unwrap_or(self.config.initial_balance);
        let mut account = Account::new(account_id, balance);
        account.set_address(address.clone());

        // Check for duplicate
        if self.accounts.lock().unwrap().contains_key(&account_id) {
            return Err(AccountError::DuplicateAccount(account_id));
        }

        self.accounts.lock().unwrap().insert(account_id, account);

        Ok((kp, account_id, address))
    }

    async fn register_account(&self, account_id: AccountId, public_key: Vec<u8>) -> AccountResult<()> {
        if self.accounts.lock().unwrap().contains_key(&account_id) {
            return Err(AccountError::DuplicateAccount(account_id));
        }

        let mut account = Account::new(account_id, self.config.initial_balance);
        if self.config.enable_address {
            account.set_address(derive_address(account_id));
        }

        self.accounts.lock().unwrap().insert(account_id, account);
        Ok(())
    }

    async fn get_balance(&self, account_id: AccountId) -> AccountResult<Amount> {
        let accounts = self.accounts.lock().unwrap();
        let account = accounts.get(&account_id)
            .ok_or_else(|| AccountError::NotFound(account_id))?;
        Ok(account.balance)
    }

    async fn get_account_info(&self, account_id: AccountId) -> AccountResult<Account> {
        let accounts = self.accounts.lock().unwrap();
        let account = accounts.get(&account_id)
            .ok_or_else(|| AccountError::NotFound(account_id))?
            .clone();
        Ok(account)
    }

    async fn transfer(&self, from: AccountId, to: AccountId, amount: Amount) -> AccountResult<()> {
        let mut accounts = self.accounts.lock().unwrap();

        let sender = accounts.get_mut(&from)
            .ok_or_else(|| AccountError::NotFound(from))?;
        if sender.balance < amount {
            return Err(AccountError::InsufficientBalance {
                have: sender.balance,
                need: amount,
            });
        }
        sender.balance -= amount;

        let recipient = accounts.get_mut(&to)
            .ok_or_else(|| AccountError::NotFound(to))?;
        recipient.balance += amount;

        Ok(())
    }

    async fn credit(&self, account_id: AccountId, amount: Amount) -> AccountResult<()> {
        let mut accounts = self.accounts.lock().unwrap();
        let account = accounts.get_mut(&account_id)
            .ok_or_else(|| AccountError::NotFound(account_id))?;
        account.balance += amount;
        Ok(())
    }

    async fn debit(&self, account_id: AccountId, amount: Amount) -> AccountResult<()> {
        let mut accounts = self.accounts.lock().unwrap();
        let account = accounts.get_mut(&account_id)
            .ok_or_else(|| AccountError::NotFound(account_id))?;
        if account.balance < amount {
            return Err(AccountError::InsufficientBalance {
                have: account.balance,
                need: amount,
            });
        }
        account.balance -= amount;
        Ok(())
    }

    async fn get_and_increment_nonce(&self, sender_id: AccountId) -> AccountResult<u64> {
        let mut nonces = self.nonces.lock().unwrap();
        let current = nonces.get(&sender_id).copied().unwrap_or(0);
        let next = current + 1;
        nonces.insert(sender_id, next);
        Ok(next)
    }

    async fn current_nonce(&self, account_id: AccountId) -> AccountResult<u64> {
        Ok(self.nonces.lock().unwrap().get(&account_id).copied().unwrap_or(0))
    }

    async fn mint_asset(&self, asset_id: AssetId, to: AccountId, amount: Amount) -> AccountResult<()> {
        if let Some(admin_id) = self.config.admin_account_id {
            let accounts = self.accounts.lock().unwrap();
            let admin = accounts.get(&admin_id)
                .ok_or_else(|| AccountError::NotFound(admin_id))?;
            // Simple check: admin should exist (full impl would check admin rights)
        }

        let mut accounts = self.accounts.lock().unwrap();
        let to_account = accounts.get_mut(&to)
            .ok_or_else(|| AccountError::NotFound(to))?;
        *to_account.assets.entry(asset_id).or_insert(0) += amount;
        Ok(())
    }

    async fn burn_asset(&self, asset_id: AssetId, from: AccountId, amount: Amount) -> AccountResult<()> {
        let mut accounts = self.accounts.lock().unwrap();
        let from_account = accounts.get_mut(&from)
            .ok_or_else(|| AccountError::NotFound(from))?;

        let current = from_account.assets.get(&asset_id).copied().unwrap_or(0);
        if current < amount {
            return Err(AccountError::InvalidOperation(format!("insufficient asset balance: have {}, need {}", current, amount)));
        }

        if current == amount {
            from_account.assets.remove(&asset_id);
        } else {
            *from_account.assets.get_mut(&asset_id).unwrap() = current - amount;
        }

        Ok(())
    }
}

#[tokio::test]
async fn test_create_account() {
    let config = AccountConfig::default();
    let manager = MockAccountManager::new(config);

    let (kp, id, address) = manager.create_account(Some(1000)).await.unwrap();

    assert!(kp.secret.as_bytes().len() == 64);
    assert!(kp.public.as_bytes().len() == 32);
    assert!(id > 0);
    assert!(!address.is_empty());

    let info = manager.get_account_info(id).await.unwrap();
    assert_eq!(info.balance, 1000);
    assert_eq!(info.id, id);
}

#[tokio::test]
async fn test_create_account_default_balance() {
    let config = AccountConfig { initial_balance: 500, ..Default::default() };
    let manager = MockAccountManager::new(config);

    let (_, id, _) = manager.create_account(None).await.unwrap();

    let balance = manager.get_balance(id).await.unwrap();
    assert_eq!(balance, 500);
}

#[tokio::test]
async fn test_get_account_info_not_found() {
    let manager = MockAccountManager::default();

    let result = manager.get_account_info(999999).await;
    assert!(matches!(result, Err(AccountError::NotFound(_))));
}

#[tokio::test]
async fn test_transfer_success() {
    let manager = MockAccountManager::default();

    let (_, id1, _) = manager.create_account(Some(1000)).await.unwrap();
    let (_, id2, _) = manager.create_account(Some(500)).await.unwrap();

    manager.transfer(id1, id2, 300).await.unwrap();

    let balance1 = manager.get_balance(id1).await.unwrap();
    let balance2 = manager.get_balance(id2).await.unwrap();

    assert_eq!(balance1, 700);
    assert_eq!(balance2, 800);
}

#[tokio::test]
async fn test_transfer_insufficient_balance() {
    let manager = MockAccountManager::default();

    let (_, id1, _) = manager.create_account(Some(100)).await.unwrap();
    let (_, id2, _) = manager.create_account(Some(500)).await.unwrap();

    let result = manager.transfer(id1, id2, 200).await;
    assert!(matches!(result, Err(AccountError::InsufficientBalance { .. })));

    // Balances should be unchanged
    assert_eq!(manager.get_balance(id1).await.unwrap(), 100);
    assert_eq!(manager.get_balance(id2).await.unwrap(), 500);
}

#[tokio::test]
async fn test_transfer_non_existent_sender() {
    let manager = MockAccountManager::default();

    let (_, id2, _) = manager.create_account(Some(500)).await.unwrap();

    let result = manager.transfer(999999, id2, 100).await;
    assert!(matches!(result, Err(AccountError::NotFound(_))));
}

#[tokio::test]
async fn test_credit() {
    let manager = MockAccountManager::default();
    let (_, id, _) = manager.create_account(Some(100)).await.unwrap();

    manager.credit(id, 200).await.unwrap();
    assert_eq!(manager.get_balance(id).await.unwrap(), 300);

    manager.credit(id, 500).await.unwrap();
    assert_eq!(manager.get_balance(id).await.unwrap(), 800);
}

#[tokio::test]
async fn test_debit() {
    let manager = MockAccountManager::default();
    let (_, id, _) = manager.create_account(Some(1000)).await.unwrap();

    manager.debit(id, 300).await.unwrap();
    assert_eq!(manager.get_balance(id).await.unwrap(), 700);
}

#[tokio::test]
async fn test_debit_insufficient() {
    let manager = MockAccountManager::default();
    let (_, id, _) = manager.create_account(Some(100)).await.unwrap();

    let result = manager.debit(id, 200).await;
    assert!(matches!(result, Err(AccountError::InsufficientBalance { .. })));
    assert_eq!(manager.get_balance(id).await.unwrap(), 100);
}

#[tokio::test]
async fn test_nonce_increment() {
    let manager = MockAccountManager::default();
    let (_, id, _) = manager.create_account(Some(0)).await.unwrap();

    let nonce1 = manager.get_and_increment_nonce(id).await.unwrap();
    assert_eq!(nonce1, 1);

    let nonce2 = manager.get_and_increment_nonce(id).await.unwrap();
    assert_eq!(nonce2, 2);

    let nonce3 = manager.get_and_increment_nonce(id).await.unwrap();
    assert_eq!(nonce3, 3);
}

#[tokio::test]
async fn test_nonce_current() {
    let manager = MockAccountManager::default();
    let (_, id, _) = manager.create_account(Some(0)).await.unwrap();

    assert_eq!(manager.current_nonce(id).await.unwrap(), 0);

    manager.get_and_increment_nonce(id).await.unwrap();
    assert_eq!(manager.current_nonce(id).await.unwrap(), 1);
}

#[tokio::test]
async fn test_nonce_multiple_accounts() {
    let manager = MockAccountManager::default();
    let (_, id1, _) = manager.create_account(Some(0)).await.unwrap();
    let (_, id2, _) = manager.create_account(Some(0)).await.unwrap();

    manager.get_and_increment_nonce(id1).await.unwrap();
    manager.get_and_increment_nonce(id1).await.unwrap();
    manager.get_and_increment_nonce(id2).await.unwrap();

    assert_eq!(manager.current_nonce(id1).await.unwrap(), 2);
    assert_eq!(manager.current_nonce(id2).await.unwrap(), 1);
}

#[tokio::test]
async fn test_mint_asset() {
    let config = AccountConfig {
        admin_account_id: Some(1),
        ..Default::default()
    };
    let manager = MockAccountManager::new(config);

    // Create admin account manually
    manager.register_account(1, vec![0u8; 32]).await.unwrap();
    let (_, user_id, _) = manager.create_account(Some(0)).await.unwrap();

    manager.mint_asset(100, user_id, 1000).await.unwrap();

    let info = manager.get_account_info(user_id).await.unwrap();
    assert_eq!(info.asset_quantity(100), 1000);
}

#[tokio::test]
async fn test_burn_asset() {
    let manager = MockAccountManager::default();
    let (_, user_id, _) = manager.create_account(Some(0)).await.unwrap();

    // First mint some assets
    manager.credit(user_id, 0).await.unwrap(); // ensure account exists
    // Directly manipulate assets for test
    {
        let mut accounts = manager.accounts.lock().unwrap();
        let account = accounts.get_mut(&user_id).unwrap();
        account.add_asset(100, 1000);
    }

    manager.burn_asset(100, user_id, 400).await.unwrap();

    let info = manager.get_account_info(user_id).await.unwrap();
    assert_eq!(info.asset_quantity(100), 600);
}

#[tokio::test]
async fn test_burn_asset_all() {
    let manager = MockAccountManager::default();
    let (_, user_id, _) = manager.create_account(Some(0)).await.unwrap();

    {
        let mut accounts = manager.accounts.lock().unwrap();
        let account = accounts.get_mut(&user_id).unwrap();
        account.add_asset(100, 500);
    }

    manager.burn_asset(100, user_id, 500).await.unwrap();

    let info = manager.get_account_info(user_id).await.unwrap();
    assert_eq!(info.asset_quantity(100), 0);
    assert!(!info.assets.contains_key(&100));
}

#[tokio::test]
async fn test_create_duplicate_account() {
    let manager = MockAccountManager::default();

    let (kp1, id1, _) = manager.create_account(Some(100)).await.unwrap();
    // Same keypair yields same ID
    let result = manager.create_account_with_keypair(kp1).await;
    assert!(matches!(result, Err(AccountError::DuplicateAccount(_))));
}

// Helper method to create account with existing keypair (if needed)
#[async_trait]
impl AccountManager for MockAccountManager {
    async fn create_account_with_keypair(&self, keypair: Keypair) -> AccountResult<(Keypair, AccountId, String)> {
        let account_id = derive_account_id(&keypair.public);
        if self.accounts.lock().unwrap().contains_key(&account_id) {
            return Err(AccountError::DuplicateAccount(account_id));
        }
        let address = if self.config.enable_address {
            derive_address(account_id)
        } else {
            format!("{}", account_id)
        };
        let mut account = Account::new(account_id, self.config.initial_balance);
        account.set_address(address.clone());
        self.accounts.lock().unwrap().insert(account_id, account);
        Ok((keypair, account_id, address))
    }
}
