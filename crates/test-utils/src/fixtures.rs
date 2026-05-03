//! 测试账户和常量定义
//!
//! 参照 Java NRCS 的 BlockchainTest.java 中定义的 6 个标准测试账户。
//! 所有密钥短语与 Java 版本完全一致，确保测试数据兼容性。

use crypto::{derive_public_key, account_id_from_public_key};
use blockchain_types::constants::ONE_NRCS;

pub const FUND_AMOUNT_NQT: i64 = 100_000 * ONE_NRCS as i64;

#[derive(Debug, Clone)]
pub struct TestAccount {
    pub name: &'static str,
    pub passphrase: &'static str,
    pub public_key: [u8; 32],
    pub account_id: u64,
}

impl TestAccount {
    pub fn account_id_signed(&self) -> i64 {
        self.account_id as i64
    }

    pub fn public_key_hex(&self) -> String {
        hex::encode(self.public_key)
    }
}

fn derive_account(passphrase: &'static str, name: &'static str) -> TestAccount {
    let pk_vec = derive_public_key(passphrase).expect("invalid passphrase");
    let mut pk = [0u8; 32];
    pk.copy_from_slice(&pk_vec[..32]);
    let account_id = account_id_from_public_key(&pk);
    TestAccount {
        name,
        passphrase,
        public_key: pk,
        account_id,
    }
}

pub fn forgy() -> TestAccount {
    derive_account(
        "concern entire frozen witch away creak dot drink need season clutch truly",
        "FORGY",
    )
}

pub fn alice() -> TestAccount {
    derive_account(
        "trickle pierce warm early gentle another thorn gotta illuminate everywhere glare determine",
        "ALICE",
    )
}

pub fn bob() -> TestAccount {
    derive_account(
        "shape dress gas less embarrass somewhere mostly fly greet minute date harsh",
        "BOB",
    )
}

pub fn chuck() -> TestAccount {
    derive_account(
        "bless prefer bone whatever heard realize game teeth stun childhood jeans somebody",
        "CHUCK",
    )
}

pub fn dave() -> TestAccount {
    derive_account(
        "meant ourselves measure bump devil soothe baby torture born childhood skill park",
        "DAVE",
    )
}

pub fn riker() -> TestAccount {
    derive_account(
        "meant ourselves measure bump devil soothe baby torture born childhood skill park",
        "RIKER",
    )
}

pub fn all_test_accounts() -> Vec<TestAccount> {
    vec![forgy(), alice(), bob(), chuck(), dave(), riker()]
}

#[derive(Debug, Clone)]
pub struct Tester {
    pub account: TestAccount,
    pub initial_balance: i64,
    pub initial_unconfirmed_balance: i64,
}

impl Tester {
    pub fn new(account: TestAccount, balance: i64) -> Self {
        Self {
            initial_balance: balance,
            initial_unconfirmed_balance: balance,
            account,
        }
    }

    pub fn balance_diff(&self, current_balance: i64) -> i64 {
        current_balance - self.initial_balance
    }

    pub fn unconfirmed_balance_diff(&self, current_unconfirmed: i64) -> i64 {
        current_unconfirmed - self.initial_unconfirmed_balance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forgy_account_derivation() {
        let forgy = forgy();
        assert_eq!(forgy.name, "FORGY");
        assert_ne!(forgy.account_id, 0);
    }

    #[test]
    fn test_alice_account_derivation() {
        let alice = alice();
        assert_eq!(alice.name, "ALICE");
        assert_ne!(alice.account_id, 0);
    }

    #[test]
    fn test_dave_and_riker_same_key() {
        let dave = dave();
        let riker = riker();
        assert_eq!(dave.public_key, riker.public_key);
        assert_eq!(dave.account_id, riker.account_id);
    }

    #[test]
    fn test_all_accounts_unique() {
        let accounts = all_test_accounts();
        assert_eq!(accounts.len(), 6);
        let mut ids: Vec<u64> = accounts.iter().map(|a| a.account_id).collect();
        ids.dedup();
        assert!(ids.len() >= 5, "Should have at least 5 unique accounts (DAVE=RIKER)");
    }

    #[test]
    fn test_tester_balance_diff() {
        let forgy = forgy();
        let tester = Tester::new(forgy, 1000);
        assert_eq!(tester.balance_diff(1100), 100);
        assert_eq!(tester.balance_diff(900), -100);
        assert_eq!(tester.unconfirmed_balance_diff(800), -200);
    }
}
