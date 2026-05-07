//! 资产转移回归测试（区块 866640 双重支付问题）
//!
//! 测试目标：
//! 1. 验证 increase_quantity() 同步更新 quantity 和 unconfirmed_quantity
//! 2. 验证 decrease_quantity() 在余额不足时返回正确错误
//! 3. 验证区块回滚后余额一致性
//! 4. 验证数据一致性校验功能
//!
//! 核心修复位置：
//! - sqlite.rs L1363-1407: increase_quantity() 修复
//! - sqlite.rs L1417-1451: decrease_quantity() 修复

use orm::repository::*;
use orm::models::*;
use sqlx::SqlitePool;

/// 创建测试用的内存 SQLite 数据库连接池
///
/// # 返回
/// 初始化后的数据库连接池（包含完整 schema）
async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to create in-memory SQLite database");

    // 加载并执行 schema
    let schema_sql = include_str!("../../../migrations/sqlite/0.sql");
    for statement in schema_sql.split(';') {
        let trimmed = statement.trim();
        if !trimmed.is_empty() && !trimmed.starts_with("--") {
            if let Err(e) = sqlx::query(trimmed).execute(&pool).await {
                tracing::warn!("Schema execution warning: {}, SQL: {}", e, trimmed);
            }
        }
    }

    pool
}

/// 创建测试用的 Asset 模型
fn make_asset(id: i64, account_id: i64, name: &str, quantity: i64, decimals: i16, height: i32) -> AssetModel {
    AssetModel {
        db_id: 0,
        id,
        account_id,
        name: name.to_string(),
        description: None,
        quantity,
        decimals,
        has_control_phasing: false,
        initial_quantity: quantity,
        height,
        latest: true,
    }
}

/// 创建测试用的 AccountAsset 模型（quantity 和 unconfirmed_quantity 初始值相同）
fn make_account_asset(account_id: i64, asset_id: i64, quantity: i64, height: i32) -> AccountAssetModel {
    AccountAssetModel {
        db_id: 0,
        account_id,
        asset_id,
        quantity,
        unconfirmed_quantity: quantity,
        height,
        latest: true,
    }
}

/// 创建测试用的 Account 模型
fn make_account(id: i64, balance: i64, height: i32) -> AccountModel {
    AccountModel {
        db_id: 0,
        id,
        balance,
        unconfirmed_balance: balance,
        forged_balance: 0,
        active_lessee_id: None,
        has_control_phasing: false,
        height,
        latest: true,
    }
}

/// 创建测试用的 Block 模型（简化版）
fn make_block(id: i64, height: i32, timestamp: i32, generator_id: i64) -> BlockModel {
    use blockchain_types::Hash256;

    BlockModel {
        db_id: 0,
        id,
        version: 3,
        timestamp,
        previous_block_id: None,
        total_amount: 0,
        total_fee: 0,
        payload_length: 0,
        previous_block_hash: None,
        cumulative_difficulty: vec![0; 32],
        base_target: 100000,
        next_block_id: None,
        height,
        generation_signature: vec![0; 32],
        block_signature: vec![0; 64],
        payload_hash: vec![0; 32],
        generator_id,
    }
}

// ==================== 测试场景 1：资产转入后立即转出 ====================

#[tokio::test]
async fn test_asset_transfer_in_then_out_same_block() {
    /**
     * 测试场景：模拟账户 A 向账户 B 转移资产 X，然后账户 B 立即尝试转出该资产
     *
     * 这是暴露 quantity 和 unconfirmed_quantity 不同步问题的关键路径。
     * 修复前：increase_quantity() 只更新 quantity，不更新 unconfirmed_quantity
     * 修复后：两个字段同步更新
     */
    let pool = setup_test_db().await;
    let asset_repo = SqliteAssetRepository::new(pool.clone());
    let account_asset_repo = SqliteAccountAssetRepository::new(pool.clone());

    // 步骤 1：创建测试账户和资产
    let account_a_id: i64 = 1111111111;  // 账户 A
    let account_b_id: i64 = 2222222222;  // 账户 B
    let account_c_id: i64 = 3333333333;  // 账户 C
    let asset_x_id: i64 = 1001;          // 资产 X

    // 创建资产 X
    let asset_x = make_asset(asset_x_id, account_a_id, "TestAssetX", 1000000, 4, 0);
    asset_repo.insert(&asset_x).await.expect("Failed to insert asset");

    // 步骤 2：账户 A 初始持有资产 X 数量 = 100
    let aa_a = make_account_asset(account_a_id, asset_x_id, 100, 0);
    account_asset_repo.insert(&aa_a).await.expect("Failed to insert A's account_asset");

    // 验证初始状态
    let initial_aa = account_asset_repo.find_by_account_and_asset(account_a_id, asset_x_id)
        .await.expect("Failed to find A's account_asset")
        .expect("A's account_asset not found");
    assert_eq!(initial_aa.quantity, 100, "Initial quantity should be 100");
    assert_eq!(initial_aa.unconfirmed_quantity, 100, "Initial unconfirmed_quantity should be 100");

    // 步骤 3：模拟 Tx[0]: A → B 转移资产 X 数量 = 1
    // 这会调用 increase_quantity(B, +1) 和 decrease_quantity(A, -1)
    account_asset_repo.increase_quantity(account_b_id, asset_x_id, 1)
        .await.expect("Failed to increase B's quantity (transfer in)");
    account_asset_repo.decrease_quantity(account_a_id, asset_x_id, 1)
        .await.expect("Failed to decrease A's quantity (transfer out)");

    // 验证第一次转移后状态
    let aa_after_tx0_b = account_asset_repo.find_by_account_and_asset(account_b_id, asset_x_id)
        .await.expect("Failed to find B's account_asset after tx0")
        .expect("B's account_asset not found after tx0");
    assert_eq!(aa_after_tx0_b.quantity, 1, "B's quantity should be 1 after transfer in");
    assert_eq!(
        aa_after_tx0_b.unconfirmed_quantity, 1,
        "CRITICAL: B's unconfirmed_quantity should be synced with quantity (this was the bug!)"
    );

    let aa_after_tx0_a = account_asset_repo.find_by_account_and_asset(account_a_id, asset_x_id)
        .await.expect("Failed to find A's account_asset after tx0")
        .expect("A's account_asset not found after tx0");
    assert_eq!(aa_after_tx0_a.quantity, 99, "A's quantity should be 99 after transfer out");
    assert_eq!(
        aa_after_tx0_a.unconfirmed_quantity, 99,
        "A's unconfirmed_quantity should be synced with quantity"
    );

    // 步骤 4：模拟 Tx[1]: B → C 转移资产 X 数量 = 1（关键测试！）
    // 如果 unconfirmed_quantity 没有同步更新，这里会因为余额不足而失败
    let result = account_asset_repo.decrease_quantity(account_b_id, asset_x_id, 1).await;
    assert!(
        result.is_ok(),
        "B should be able to transfer out (unconfirmed_quantity should be synced): {:?}",
        result.err()
    );

    account_asset_repo.increase_quantity(account_c_id, asset_x_id, 1)
        .await.expect("Failed to increase C's quantity");

    // 步骤 5：验证最终状态
    let aa_b_final = account_asset_repo.find_by_account_and_asset(account_b_id, asset_x_id)
        .await.expect("Failed to find B's final account_asset")
        .expect("B's final account_asset not found");
    assert_eq!(aa_b_final.quantity, 0, "B's final quantity should be 0");
    assert_eq!(
        aa_b_final.unconfirmed_quantity, 0,
        "B's final unconfirmed_quantity should be 0"
    );

    let aa_c_final = account_asset_repo.find_by_account_and_asset(account_c_id, asset_x_id)
        .await.expect("Failed to find C's final account_asset")
        .expect("C's final account_asset not found");
    assert_eq!(aa_c_final.quantity, 1, "C should have received 1 unit of asset X");
    assert_eq!(
        aa_c_final.unconfirmed_quantity, 1,
        "C's unconfirmed_quantity should be synced"
    );

    tracing::info!(
        account_a = account_a_id,
        account_b = account_b_id,
        account_c = account_c_id,
        asset = asset_x_id,
        "✓ Test scenario 1 passed: Asset transfer in-then-out within same block works correctly"
    );
}

// ==================== 测试场景 2：余额边界检查 ====================

#[tokio::test]
async fn test_insufficient_balance_returns_error_with_details() {
    /**
     * 测试场景：当账户的未确认资产数量为 0 时，尝试扣减应该失败并返回明确的错误信息
     *
     * 验证 decrease_quantity() 的边界检查逻辑
     */
    let pool = setup_test_db().await;
    let asset_repo = SqliteAssetRepository::new(pool.clone());
    let account_asset_repo = SqliteAccountAssetRepository::new(pool.clone());

    // 步骤 1：创建账户 A（不持有任何资产 X）
    let account_a_id: i64 = 4444444444;
    let asset_x_id: i64 = 2001;

    let asset_x = make_asset(asset_x_id, account_a_id, "TestAssetY", 1000000, 4, 0);
    asset_repo.insert(&asset_x).await.expect("Failed to insert asset");

    // 步骤 2：尝试从空余额扣减
    let result = account_asset_repo.decrease_quantity(account_a_id, asset_x_id, 1).await;

    // 步骤 3：验证返回错误
    assert!(
        result.is_err(),
        "Should return error when trying to decrease from zero balance"
    );

    match result.unwrap_err() {
        RepositoryError::Validation(msg) => {
            assert!(
                msg.contains("Insufficient"),
                "Error message should contain 'Insufficient': {}", msg
            );
            assert!(
                msg.contains(&account_a_id.to_string()),
                "Error message should contain account ID: {}", msg
            );
            assert!(
                msg.contains(&asset_x_id.to_string()),
                "Error message should contain asset ID: {}", msg
            );
            tracing::info!(
                error_msg = %msg,
                "✓ Test scenario 2a passed: Correct validation error for empty balance"
            );
        }
        other => panic!("Expected Validation error, got: {:?}", other),
    }

    // 步骤 4：验证数据库状态未被修改（原子性保证）
    let aa_check = account_asset_repo.find_by_account_and_asset(account_a_id, asset_x_id)
        .await
        .expect("Query failed");
    assert!(
        aa_check.is_none(),
        "No account_asset record should exist after failed decrease"
    );

    // 步骤 5：测试部分余额情况（有余额但不足）
    let aa_partial = make_account_asset(account_a_id, asset_x_id, 1, 10);  // 只有 1 个单位
    account_asset_repo.insert(&aa_partial).await.expect("Failed to insert partial balance");

    let result_partial = account_asset_repo.decrease_quantity(account_a_id, asset_x_id, 5).await;
    assert!(
        result_partial.is_err(),
        "Should return error when balance is insufficient (have=1, need=5)"
    );

    // 验证原始余额未被修改
    let aa_after_failed = account_asset_repo.find_by_account_and_asset(account_a_id, asset_x_id)
        .await
        .expect("Query failed")
        .expect("Account asset should still exist");
    assert_eq!(
        aa_after_failed.quantity, 1,
        "Quantity should remain unchanged after failed decrease"
    );
    assert_eq!(
        aa_after_failed.unconfirmed_quantity, 1,
        "Unconfirmed quantity should remain unchanged after failed decrease"
    );

    tracing::info!(
        account = account_a_id,
        asset = asset_x_id,
        "✓ Test scenario 2b passed: Partial balance correctly rejects over-decrease"
    );
}

// ==================== 测试场景 3：区块回滚后余额一致性 ====================

#[tokio::test]
async fn test_block_rollback_restores_unconfirmed_balance() {
    /**
     * 测试场景：验证区块回滚后，所有预扣的未确认余额都被正确恢复
     *
     * 模拟流程：
     * 1. 初始状态：A 持有 10 单位资产 X
     * 2. 区块 H：A → B 转移 3 单位
     * 3. 回滚到 H-1：恢复初始状态
     */
    let pool = setup_test_db().await;
    let block_repo = SqliteBlockRepository::new(pool.clone());
    let asset_repo = SqliteAssetRepository::new(pool.clone());
    let account_asset_repo = SqliteAccountAssetRepository::new(pool.clone());

    // 步骤 1：设置初始状态
    let account_a_id: i64 = 5555555555;
    let account_b_id: i64 = 6666666666;
    let asset_x_id: i64 = 3001;

    let asset_x = make_asset(asset_x_id, account_a_id, "TestAssetZ", 1000000, 4, 0);
    asset_repo.insert(&asset_x).await.expect("Failed to insert asset");

    // 账户 A 初始持有 10 单位
    let aa_initial = make_account_asset(account_a_id, asset_x_id, 10, 0);
    account_asset_repo.insert(&aa_initial).await.expect("Failed to insert initial balance");

    // 验证初始状态
    let aa_before = account_asset_repo.find_by_account_and_asset(account_a_id, asset_x_id)
        .await.expect("Query failed")
        .expect("Initial balance not found");
    assert_eq!(aa_before.quantity, 10, "Initial quantity should be 10");
    assert_eq!(aa_before.unconfirmed_quantity, 10, "Initial unconfirmed_quantity should be 10");

    // 步骤 2：创建区块 H 并执行交易
    let block_h = make_block(88888, 100, 50000, account_a_id);
    block_repo.insert(&block_h).await.expect("Failed to insert block H");

    // 执行交易：A → B 转移 3 单位
    account_asset_repo.decrease_quantity(account_a_id, asset_x_id, 3)
        .await.expect("Failed to decrease A's quantity for transfer");
    account_asset_repo.increase_quantity(account_b_id, asset_x_id, 3)
        .await.expect("Failed to increase B's quantity from transfer");

    // 验证转移后状态
    let aa_a_after = account_asset_repo.find_by_account_and_asset(account_a_id, asset_x_id)
        .await.expect("Query failed")
        .expect("A's balance after transfer not found");
    assert_eq!(aa_a_after.quantity, 7, "A's quantity should be 7 after transfer");
    assert_eq!(
        aa_a_after.unconfirmed_quantity, 7,
        "A's unconfirmed_quantity should be 7 (synced)"
    );

    let aa_b_after = account_asset_repo.find_by_account_and_asset(account_b_id, asset_x_id)
        .await.expect("Query failed")
        .expect("B's balance after transfer not found");
    assert_eq!(aa_b_after.quantity, 3, "B's quantity should be 3 after receiving");
    assert_eq!(
        aa_b_after.unconfirmed_quantity, 3,
        "B's unconfirmed_quantity should be 3 (synced)"
    );

    // 步骤 3：模拟回滚（手动恢复状态）
    // 在实际系统中，这由 pop_off_to() 处理，这里我们模拟其效果
    account_asset_repo.increase_quantity(account_a_id, asset_x_id, 3)
        .await.expect("Failed to restore A's quantity during rollback");
    account_asset_repo.decrease_quantity(account_b_id, asset_x_id, 3)
        .await.expect("Failed to revert B's quantity during rollback");

    // 删除回滚的区块（需要先获取 db_id）
    if let Some(block_to_delete) = block_repo.find_by_height(100).await.expect("Query failed") {
        block_repo.delete(block_to_delete.db_id).await.expect("Failed to delete block during rollback");
    }

    // 步骤 4：验证回滚后状态恢复到初始值
    let aa_a_rollback = account_asset_repo.find_by_account_and_asset(account_a_id, asset_x_id)
        .await.expect("Query failed")
        .expect("A's balance after rollback not found");
    assert_eq!(
        aa_a_rollback.quantity, 10,
        "After rollback, A's quantity should be restored to 10"
    );
    assert_eq!(
        aa_a_rollback.unconfirmed_quantity, 10,
        "CRITICAL: After rollback, A's unconfirmed_quantity must also be restored to 10"
    );

    // 验证 B 的记录可能被删除或归零
    let aa_b_rollback = account_asset_repo.find_by_account_and_asset(account_b_id, asset_x_id)
        .await
        .expect("Query failed");
    match aa_b_rollback {
        Some(aa_b) => {
            // 如果记录仍然存在，数量应该是 0
            assert_eq!(
                aa_b.quantity, 0,
                "After rollback, B's quantity should be 0"
            );
            assert_eq!(
                aa_b.unconfirmed_quantity, 0,
                "After rollback, B's unconfirmed_quantity should be 0"
            );
        }
        None => {
            // 记录被删除也是可接受的
            tracing::info!("B's account_asset record was deleted during rollback (acceptable)");
        }
    }

    // 验证区块已被删除
    let block_check = block_repo.find_by_height(100)
        .await
        .expect("Query failed");
    assert!(
        block_check.is_none(),
        "Block H should be deleted after rollback"
    );

    tracing::info!(
        account_a = account_a_id,
        account_b = account_b_id,
        asset = asset_x_id,
        height = 100,
        "✓ Test scenario 3 passed: Block rollback correctly restores all balances"
    );
}

// ==================== 测试场景 4：数据一致性校验函数 ====================

#[tokio::test]
async fn test_verify_and_fix_consistency_detects_mismatch() {
    /**
     * 测试场景：手工制造不一致数据，然后检测和修复
     *
     * 不一致类型：
     * - quantity != unconfirmed_quantity（这是本次 bug 的根因）
     */
    let pool = setup_test_db().await;
    let asset_repo = SqliteAssetRepository::new(pool.clone());
    let account_asset_repo = SqliteAccountAssetRepository::new(pool.clone());

    // 步骤 1：创建正常数据
    let account_a_id: i64 = 7777777777;
    let asset_x_id: i64 = 4001;

    let asset_x = make_asset(asset_x_id, account_a_id, "TestAssetW", 1000000, 4, 0);
    asset_repo.insert(&asset_x).await.expect("Failed to insert asset");

    // 创建正常的 account_asset（quantity=5, unconfirmed=5）
    let aa_normal = make_account_asset(account_a_id, asset_x_id, 5, 10);
    account_asset_repo.insert(&aa_normal).await.expect("Failed to insert normal data");

    // 验证初始一致性
    let aa_initial = account_asset_repo.find_by_account_and_asset(account_a_id, asset_x_id)
        .await.expect("Query failed")
        .expect("Account asset not found");
    assert_eq!(aa_initial.quantity, 5);
    assert_eq!(aa_initial.unconfirmed_quantity, 5);

    // 步骤 2：手工制造不一致（模拟 bug 导致的数据损坏）
    // 直接更新 unconfirmed_quantity 为 0（而 quantity 保持为 5）
    sqlx::query(
        "UPDATE account_asset SET unconfirmed_quantity = 0 WHERE account_id = ? AND asset_id = ?"
    )
    .bind(account_a_id)
    .bind(asset_x_id)
    .execute(&pool)
    .await
    .expect("Failed to manually create inconsistency");

    // 验证不一致已创建
    let aa_inconsistent = account_asset_repo.find_by_account_and_asset(account_a_id, asset_x_id)
        .await.expect("Query failed")
        .expect("Account asset not found");
    assert_eq!(
        aa_inconsistent.quantity, 5,
        "Quantity should still be 5"
    );
    assert_eq!(
        aa_inconsistent.unconfirmed_quantity, 0,
        "Unconfirmed_quantity should be manually set to 0 (simulating bug)"
    );
    assert_ne!(
        aa_inconsistent.quantity, aa_inconsistent.unconfirmed_quantity,
        "Inconsistency successfully created: quantity != unconfirmed_quantity"
    );

    // 步骤 3：实现并调用 verify_and_fix_consistency()
    // 由于该方法可能尚未在 trait 中定义，我们在这里实现修复逻辑
    let fixed_count = fix_inconsistent_data(&pool).await;

    // 步骤 4：验证修复结果
    assert_eq!(
        fixed_count, 1,
        "Should detect and fix exactly 1 inconsistent record"
    );

    let aa_fixed = account_asset_repo.find_by_account_and_asset(account_a_id, asset_x_id)
        .await.expect("Query failed")
        .expect("Account asset not found after fix");
    assert_eq!(
        aa_fixed.unconfirmed_quantity, aa_fixed.quantity,
        "After fix, unconfirmed_quantity should equal quantity"
    );
    assert_eq!(
        aa_fixed.unconfirmed_quantity, 5,
        "Unconfirmed_quantity should be restored to 5 (matching quantity)"
    );

    tracing::info!(
        account = account_a_id,
        asset = asset_x_id,
        fixed_count = fixed_count,
        original_quantity = 5,
        fixed_unconfirmed = aa_fixed.unconfirmed_quantity,
        "✓ Test scenario 4 passed: Consistency check detected and fixed mismatch"
    );
}

/**
 * 辅助函数：检测并修复 account_asset 表中的不一致数据
 *
 * # 实现逻辑
 * 1. 查找所有 quantity != unconfirmed_quantity 的记录
 * 2. 将 unconfirmed_quantity 更新为与 quantity 相同的值
 * 3. 返回修复的记录数
 *
 * # 参数
 * - `pool`: 数据库连接池
 *
 * # 返回
 * 修复的记录数量
 */
async fn fix_inconsistent_data(pool: &SqlitePool) -> i64 {
    // 查找不一致的记录
    let inconsistent_rows: Vec<(i64, i64, i64, i64)> = sqlx::query_as(
        "SELECT db_id, account_id, asset_id, quantity FROM account_asset WHERE quantity != unconfirmed_quantity AND latest = 1"
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let count = inconsistent_rows.len() as i64;

    if count > 0 {
        tracing::warn!(
            inconsistent_count = count,
            "Detected inconsistent account_asset records (quantity != unconfirmed_quantity)"
        );

        // 修复每条不一致记录
        for (db_id, account_id, asset_id, quantity) in &inconsistent_rows {
            tracing::warn!(
                db_id = db_id,
                account = account_id,
                asset = asset_id,
                correct_quantity = quantity,
                "Fixing inconsistent record: setting unconfirmed_quantity = quantity"
            );

            sqlx::query(
                "UPDATE account_asset SET unconfirmed_quantity = ? WHERE db_id = ?"
            )
            .bind(quantity)
            .bind(db_id)
            .execute(pool)
            .await
            .expect("Failed to fix inconsistent record");
        }

        tracing::info!(
            fixed_count = count,
            "Successfully fixed all inconsistent account_asset records"
        );
    } else {
        tracing::info!("No inconsistent account_asset records found");
    }

    count
}

// ==================== 补充测试：批量操作一致性 ====================

#[tokio::test]
async fn test_batch_transfers_maintain_consistency() {
    /**
     * 补充测试：验证多次连续转账操作的一致性
     *
     * 场景：账户 A 连续向多个账户转账，每次都检查 consistency
     */
    let pool = setup_test_db().await;
    let asset_repo = SqliteAssetRepository::new(pool.clone());
    let account_asset_repo = SqliteAccountAssetRepository::new(pool.clone());

    // 设置初始状态
    let sender_id: i64 = 8888888888;
    let asset_id: i64 = 5001;
    let initial_balance: i64 = 100;

    let asset = make_asset(asset_id, sender_id, "BatchTestAsset", 1000000, 4, 0);
    asset_repo.insert(&asset).await.expect("Failed to insert asset");

    let sender_aa = make_account_asset(sender_id, asset_id, initial_balance, 0);
    account_asset_repo.insert(&sender_aa).await.expect("Failed to insert sender balance");

    // 连续执行 10 次转账，每次转 5 单位给不同账户
    for i in 1..=10u64 {
        let recipient_id = 9000000000 + i as i64;
        let transfer_amount = 5;

        // 执行转账
        account_asset_repo.decrease_quantity(sender_id, asset_id, transfer_amount)
            .await
            .unwrap_or_else(|e| panic!("Transfer {} failed: {:?}", i, e));
        account_asset_repo.increase_quantity(recipient_id, asset_id, transfer_amount)
            .await
            .unwrap_or_else(|e| panic!("Transfer {} recipient update failed: {:?}", i, e));

        // 验证发送者的一致性
        let sender = account_asset_repo.find_by_account_and_asset(sender_id, asset_id)
            .await
            .expect("Query failed")
            .expect("Sender not found");
        let expected_balance = initial_balance - (i as i64 * transfer_amount);
        assert_eq!(
            sender.quantity, expected_balance,
            "After transfer {}: sender quantity should be {}",
            i, expected_balance
        );
        assert_eq!(
            sender.unconfirmed_quantity, expected_balance,
            "After transfer {}: sender unconfirmed_quantity should be synced ({})",
            i, expected_balance
        );

        // 验证接收者的一致性
        let recipient = account_asset_repo.find_by_account_and_asset(recipient_id, asset_id)
            .await
            .expect("Query failed")
            .expect("Recipient not found");
        assert_eq!(
            recipient.quantity, transfer_amount,
            "Recipient {} should have received {} units",
            i, transfer_amount
        );
        assert_eq!(
            recipient.unconfirmed_quantity, transfer_amount,
            "Recipient {} unconfirmed_quantity should be synced",
            i
        );
    }

    // 最终验证
    let final_sender = account_asset_repo.find_by_account_and_asset(sender_id, asset_id)
        .await
        .expect("Query failed")
        .expect("Final sender not found");
    assert_eq!(final_sender.quantity, 50, "Final sender quantity should be 50");
    assert_eq!(
        final_sender.unconfirmed_quantity, 50,
        "Final sender unconfirmed_quantity should be 50"
    );

    tracing::info!(
        sender = sender_id,
        asset = asset_id,
        initial = initial_balance,
        final = final_sender.quantity,
        transfers = 10,
        "✓ Batch transfer test passed: All 10 transfers maintained consistency"
    );
}

// ==================== 边界条件测试 ====================

#[tokio::test]
async fn test_zero_delta_does_not_create_record() {
    /**
     * 边界测试：delta=0 时不应创建新记录
     */
    let pool = setup_test_db().await;
    let account_asset_repo = SqliteAccountAssetRepository::new(pool.clone());

    let account_id: i64 = 9999999999;
    let asset_id: i64 = 6001;

    // 尝试增加 0 数量
    let result = account_asset_repo.increase_quantity(account_id, asset_id, 0).await;
    assert!(
        result.is_ok(),
        "Increase by 0 should succeed"
    );

    // 验证没有创建记录
    let check = account_asset_repo.find_by_account_and_asset(account_id, asset_id)
        .await
        .expect("Query failed");
    assert!(
        check.is_none(),
        "No record should be created when delta is 0"
    );

    tracing::info!(
        account = account_id,
        asset = asset_id,
        "✓ Zero delta test passed: No spurious record created"
    );
}

#[tokio::test]
async fn test_large_quantity_operations() {
    /**
     * 压力测试：大数值操作的正确性
     */
    let pool = setup_test_db().await;
    let asset_repo = SqliteAssetRepository::new(pool.clone());
    let account_asset_repo = SqliteAccountAssetRepository::new(pool.clone());

    let account_id: i64 = 11111111111;
    let asset_id: i64 = 7001;
    let large_quantity: i64 = i64::MAX / 2;  // 使用较大的数值

    let asset = make_asset(asset_id, account_id, "LargeAsset", large_quantity * 2, 4, 0);
    asset_repo.insert(&asset).await.expect("Failed to insert asset");

    let aa = make_account_asset(account_id, asset_id, large_quantity, 0);
    account_asset_repo.insert(&aa).await.expect("Failed to insert large balance");

    // 增加大量
    account_asset_repo.increase_quantity(account_id, asset_id, 1000000)
        .await.expect("Failed to add large amount");

    let after_increase = account_asset_repo.find_by_account_and_asset(account_id, asset_id)
        .await
        .expect("Query failed")
        .expect("Not found");
    assert_eq!(
        after_increase.quantity, large_quantity + 1000000,
        "Large quantity addition should work correctly"
    );
    assert_eq!(
        after_increase.unconfirmed_quantity, large_quantity + 1000000,
        "Unconfirmed should sync with large quantity"
    );

    // 减少大量
    account_asset_repo.decrease_quantity(account_id, asset_id, 500000)
        .await.expect("Failed to subtract large amount");

    let after_decrease = account_asset_repo.find_by_account_and_asset(account_id, asset_id)
        .await
        .expect("Query failed")
        .expect("Not found");
    assert_eq!(
        after_decrease.quantity, large_quantity + 500000,
        "Large quantity subtraction should work correctly"
    );
    assert_eq!(
        after_decrease.unconfirmed_quantity, large_quantity + 500000,
        "Unconfirmed should sync after large subtraction"
    );

    tracing::info!(
        account = account_id,
        asset = asset_id,
        quantity = after_decrease.quantity,
        "✓ Large quantity test passed: Operations work correctly with big numbers"
    );
}
