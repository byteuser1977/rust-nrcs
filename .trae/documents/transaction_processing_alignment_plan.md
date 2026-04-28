# NRCS Java ↔ Rust 交易处理完整性对齐计划

## 📊 当前状态分析

### ✅ 已完成的部分（Rust实现）
1. **Payment (Type 0, Subtype 0)** - ORDINARY_PAYMENT 完整实现
   - ACCOUNT表余额操作 ✓
   - ACCOUNT_LEDGER记录 ✓
   - ACCOUNT_GUARANTEED_BALANCE更新 ✓

2. **ColoredCoins/Asset (Type 2)** - 部分实现
   - Subtype 0: ASSET_ISSUANCE ✓ (INSERT ASSET + UPDATE ACCOUNT_ASSET)
   - Subtype 1: ASSET_TRANSFER ✓ (UPDATE ACCOUNT_ASSET + INSERT ASSET_TRANSFER)
   - Subtype 2-12: ❌ 未实现（ASK/BID_ORDER, DIVIDEND等）

3. **MonetarySystem/Currency (Type 4)** - 部分实现
   - Subtype 3: CURRENCY_TRANSFER ✓ (UPDATE ACCOUNT_CURRENCY + INSERT CURRENCY_TRANSFER)
   - Subtype 0,1,2,4-8: ❌ 未实现

### ❌ 未实现的交易类型（Stub/No-op）

#### 🔴 高优先级（用户明确提到）
| Type | Subtype | 名称 | 涉及表 | Java参考方法 |
|------|---------|------|--------|-------------|
| Messaging (1) | 5 | ALIAS_ASSIGNMENT | ALIAS | Alias.addOrUpdateAlias() |
| Messaging (1) | 6 | ALIAS_SELL | ALIAS_OFFER | Alias.sellAlias() |
| Messaging (1) | 7 | ALIAS_BUY | ALIAS | Alias.changeOwner() |
| Messaging (1) | 8 | ALIAS_DELETE | ALIAS | Alias.deleteAlias() |
| Messaging (1) | 2 | POLL_CREATION | POLL | Poll.addPoll() |
| Messaging (1) | 3 | VOTE_CASTING | VOTE, POLL_RESULT | Vote.addVote() |
| Messaging (1) | 9 | PHASING_VOTE_CASTING | PHASING_VOTE | PhasingVote.addVote() |
| Messaging (1) | 10 | ACCOUNT_PROPERTY | ACCOUNT_PROPERTY | recipientAccount.setProperty() |
| Messaging (1) | 11 | ACCOUNT_PROPERTY_DELETE | ACCOUNT_PROPERTY | senderAccount.deleteProperty() |
| Messaging (1) | 12 | ACCOUNT_LONG_VALUE_PROPERTY | ACCOUNT_PROPERTY | recipientAccount.setProperty() |
| AccountControl (3) | 0 | EFFECTIVE_BALANCE_LEASING | ACCOUNT_LEASE | leaseEffectiveBalance() |
| AccountControl (3) | 1 | PHASING_ONLY | ACCOUNT_CONTROL_PHASING | AccountPhasingOnly.set() |
| Data (6) | 0 | TAGGED_DATA_UPLOAD | TAGGED_DATA, TAG | TaggedData.add() |
| Data (6) | 1 | TAGGED_DATA_EXTEND | TAGGED_DATA_EXTEND | TaggedData.extend() |
| LightContract (11) | 0 | CONTRACT_REFERENCE_SET | CONTRACT_REFERENCE | ContractReference.setContractReference() |
| LightContract (11) | 1 | CONTRACT_REFERENCE_DELETE | CONTRACT_REFERENCE | ContractReference.deleteContractReference() |

#### 🟡 中优先级（功能重要）
| Type | Subtype | 名称 | 涉及表 | Java参考方法 |
|------|---------|------|--------|-------------|
| ColoredCoins (2) | 2 | ASK_ORDER_PLACEMENT | ASK_ORDER, ACCOUNT_ASSET | OrderAsk.addOrder() |
| ColoredCoins (2) | 3 | BID_ORDER_PLACEMENT | BID_ORDER | OrderBid.addOrder() |
| ColoredCoins (2) | 4 | ASK_ORDER_CANCELLATION | ASK_ORDER, ACCOUNT_ASSET | OrderAsk.removeOrder() |
| ColoredCoins (2) | 5 | BID_ORDER_CANCELLATION | BID_ORDER | OrderBid.removeOrder() |
| ColoredCoins (2) | 6 | DIVIDEND_PAYMENT | ACCOUNT_LEDGER (批量) | senderAccount.payDividends() |
| ColoredCoins (2) | 7 | ASSET_DELETE | ASSET, ACCOUNT_ASSET | Asset.deleteAsset() |
| ColoredCoins (2) | 10 | ASSET_PROPERTY_SET | ASSET_PROPERTY | Asset.setProperty() |
| ColoredCoins (2) | 11 | ASSET_PROPERTY_DELETE | ASSET_PROPERTY | Asset.deleteProperty() |
| ColoredCoins (2) | 12 | ASSET_LONG_VALUE_PROPERTY_SET | ASSET_PROPERTY | Asset.setProperty() |
| ColoredCoins (2) | 9 | ASSET_INCREASE | ASSET, ACCOUNT_ASSET | Asset.increaseAsset() |
| MonetarySystem (4) | 0 | CURRENCY_ISSUANCE | CURRENCY, ACCOUNT_CURRENCY | Currency.addCurrency() |
| MonetarySystem (4) | 1 | RESERVE_INCREASE | CURRENCY (reserveSupply) | Currency.increaseReserve() |
| MonetarySystem (4) | 2 | RESERVE_CLAIM | ACCOUNT_CURRENCY, ACCOUNT | Currency.claimReserve() |
| MonetarySystem (4) | 4 | PUBLISH_EXCHANGE_OFFER | CURRENCY (exchangeOffer) | CurrencyExchangeOffer.publishOffer() |
| MonetarySystem (4) | 5 | EXCHANGE_BUY | EXCHANGE_REQUEST, ACCOUNT_CURRENCY | ExchangeRequest + exchangeNRCSForCurrency() |
| MonetarySystem (4) | 6 | EXCHANGE_SELL | EXCHANGE_REQUEST, ACCOUNT_CURRENCY | ExchangeRequest + exchangeCurrencyForNRCS() |
| MonetarySystem (4) | 7 | CURRENCY_MINTING | CURRENCY_MINT, ACCOUNT_CURRENCY | CurrencyMint.mintCurrency() |
| MonetarySystem (4) | 8 | CURRENCY_DELETION | CURRENCY, ACCOUNT_CURRENCY | currency.delete() |

#### 🟢 低优先级（高级功能）
| Type | Subtype | 名称 | 涉及表 |
|------|---------|------|--------|
| DigitalGoods (5) | 0-7 | DGS_* | GOODS, PURCHASE, *FEEDBACK |
| Shuffling (7) | 0-4 | SHUFFLING_* | SHUFFLING, SHUFFLING_DATA, PARTICIPANT |
| Voting (8) | 0-2 | *_POLL_CREATION, *_VOTE_CASTING | PHASING_POLL, PHASING_VOTE, etc. |
| CoinExchange (10) | 0-1 | COIN_EXCHANGE_ORDER | COIN_ORDER_FXT, COIN_TRADE_FXT |

---

## 🎯 实施计划

### Phase 1: 核心交易类型（P0 - 立即实施）
**目标**: 完成用户明确提到的所有交易类型

#### 1.1 Messaging交易类型（Type 1）
```rust
// 文件: crates/tx-engine/src/processor.rs
// 方法: apply_messaging_attachment()

match tx.subtype {
    5 => { // ALIAS_ASSIGNMENT
        // INSERT/UPDATE ALIAS table
        self.alias_repo.upsert(&alias_model).await?;
    }
    6 => { // ALIAS_SELL
        // INSERT/UPDATE ALIAS_OFFER table
        self.alias_offer_repo.insert(&offer_model).await?;
    }
    7 => { // ALIAS_BUY
        // UPDATE ALIAS.owner_id
        // DELETE ALIAS_OFFER
        self.alias_repo.update_owner(alias_id, tx.sender_id).await?;
        self.alias_offer_repo.delete_by_alias(alias_id).await?;
    }
    8 => { // ALIAS_DELETE
        // DELETE ALIAS
        self.alias_repo.delete_by_name(&alias_name).await?;
    }
    2 => { // POLL_CREATION
        // INSERT POLL
        self.poll_repo.insert(&poll_model).await?;
    }
    3 => { // VOTE_CASTING
        // INSERT VOTE
        // UPDATE POLL.voters_count / POLL_RESULT
        self.vote_repo.insert(&vote_model).await?;
        self.poll_repo.update_voters_count(poll_id, new_count).await?;
    }
    9 => { // PHASING_VOTE_CASTING
        // INSERT PHASING_VOTE
        self.phasing_vote_repo.insert(&vote).await?;
    }
    10 => { // ACCOUNT_PROPERTY
        // INSERT ACCOUNT_PROPERTY
        self.account_property_repo.insert(&prop).await?;
    }
    11 => { // ACCOUNT_PROPERTY_DELETE
        // DELETE ACCOUNT_PROPERTY
        self.account_property_repo.delete(property_id).await?;
    }
    12 => { // ACCOUNT_LONG_VALUE_PROPERTY
        // INSERT ACCOUNT_PROPERTY (long value)
        self.account_property_repo.insert(&long_prop).await?;
    }
}
```

#### 1.2 AccountControl交易类型（Type 3）
```rust
// 方法: apply_account_control_attachment()

match tx.subtype {
    0 => { // EFFECTIVE_BALANCE_LEASING
        // INSERT/UPDATE ACCOUNT_LEASE
        self.account_lease_repo.upsert(&lease).await?;
    }
    1 => { // PHASING_ONLY
        // INSERT/UPDATE ACCOUNT_CONTROL_PHASING
        self.account_control_phasing_repo.upsert(&phasing).await?;
    }
}
```

#### 1.3 Data交易类型（Type 6）
```rust
// 方法: apply_data_attachment()

match tx.subtype {
    0 => { // TAGGED_DATA_UPLOAD
        // INSERT TAGGED_DATA
        // INSERT TAG (multiple)
        self.tagged_data_repo.insert(&data).await?;
        for tag in tags {
            self.tagged_data_tag_repo.insert(&tag).await?;
        }
    }
    1 => { // TAGGED_DATA_EXTEND
        // INSERT TAGGED_DATA_EXTEND
        self.tagged_data_extend_repo.insert(&ext).await?;
    }
}
```

#### 1.4 LightContract交易类型（Type 11）
```rust
// 方法: apply_light_contract_attachment()

match tx.subtype {
    0 => { // CONTRACT_REFERENCE_SET
        // INSERT CONTRACT_REFERENCE
        self.contract_ref_repo.insert(&ref_model).await?;
    }
    1 => { // CONTRACT_REFERENCE_DELETE
        // DELETE CONTRACT_REFERENCE
        self.contract_ref_repo.delete_by_account_and_name(account_id, name).await?;
    }
}
```

#### 1.5 补充ColoredCoins缺失subtype（Type 2）
```rust
// 在 apply_colored_coins_attachment() 中补充:

2 => { // ASK_ORDER_PLACEMENT
    // INSERT ASK_ORDER
    // UPDATE ACCOUNT_ASSET (unconfirmed quantity -)
    self.ask_order_repo.insert(&order).await?;
    self.account_asset_repo.decrease_quantity(sender_id, asset_id, qty).await?;
}

3 => { // BID_ORDER_PLACEMENT
    // INSERT BID_ORDER
    // UPDATE ACCOUNT (unconfirmed balance -)
    self.bid_order_repo.insert(&order).await?;
    self.account_repo.add_to_unconfirmed_balance(sender_id, -total_cost).await?;
}

4 => { // ASK_ORDER_CANCELLATION
    // DELETE ASK_ORDER
    // UPDATE ACCOUNT_ASSET (unconfirmed quantity +)
    let order = self.ask_order_repo.find_by_order_id(order_id).await?;
    self.ask_order_repo.delete(order_id).await?;
    if let Some(order) = order {
        self.account_asset_repo.increase_quantity(sender_id, order.asset_id, order.quantity).await?;
    }
}

5 => { // BID_ORDER_CANCELLATION
    // DELETE BID_ORDER
    // UPDATE ACCOUNT (unconfirmed balance +)
    let order = self.bid_order_repo.find_by_order_id(order_id).await?;
    self.bid_order_repo.delete(order_id).await?;
    if let Some(order) = order {
        self.account_repo.add_to_unconfirmed_balance(sender_id, order.quantity * order.price).await?;
    }
}

6 => { // DIVIDEND_PAYMENT
    // 批量更新ACCOUNT (balance + dividend per share)
    // 批量插入ACCOUNT_LEDGER
    self.process_dividend_payment(tx).await?;
}

7 => { // ASSET_DELETE
    // UPDATE ASSET (quantity -= deleted_qty)
    // UPDATE ACCOUNT_ASSET (quantity -= deleted_qty)
    self.asset_repo.decrease_quantity(asset_id, qty).await?;
    self.account_asset_repo.decrease_quantity(sender_id, asset_id, qty).await?;
}

9 => { // ASSET_INCREASE
    // UPDATE ASSET (quantity += increase_qty)
    // UPDATE ACCOUNT_ASSET (quantity += increase_qty)
    self.asset_repo.increase_quantity(asset_id, qty).await?;
    self.account_asset_repo.increase_quantity(sender_id, asset_id, qty).await?;
}

10 => { // ASSET_PROPERTY_SET
    // INSERT ASSET_PROPERTY
    self.asset_prop_repo.insert(&prop).await?;
}

11 => { // ASSET_PROPERTY_DELETE
    // DELETE ASSET_PROPERTY
    self.asset_prop_repo.delete(asset_id, account_id, property).await?;
}

12 => { // ASSET_LONG_VALUE_PROPERTY_SET
    // INSERT ASSET_PROPERTY (long value)
    self.asset_prop_repo.insert(&long_prop).await?;
}
```

#### 1.6 补充MonetarySystem缺失subtype（Type 4）
```rust
// 在 apply_monetary_system_attachment() 中补充:

0 => { // CURRENCY_ISSUANCE
    // INSERT CURRENCY
    // UPDATE ACCOUNT_CURRENCY (units = initialSupply)
    self.currency_repo.insert(&currency).await?;
    self.account_currency_repo.update_units(sender_id, currency_id, initial_supply).await?;
}

1 => { // RESERVE_INCREASE
    // UPDATE CURRENCY (reserveSupply增加)
    // UPDATE ACCOUNT (balance -= cost)
    self.currency_repo.increase_reserve(currency_id, amount_per_unit).await?;
    self.account_repo.add_to_balance(sender_id, -total_cost).await?;
}

2 => { // RESERVE_CLAIM
    // UPDATE ACCOUNT_CURRENCY (units -= claimed_units)
    // UPDATE ACCOUNT (balance += NRCS received)
    self.account_currency_repo.update_units(sender_id, currency_id, -units).await?;
    self.account_repo.add_to_balance(sender_id, nrcs_received).await?;
}

4 => { // PUBLISH_EXCHANGE_OFFER
    // 更新CURRENCY.exchangeOffer字段
    // UPDATE ACCOUNT (balance -= buy supply * buy rate)
    // UPDATE ACCOUNT_CURRENCY (units -= sell supply)
    self.currency_exchange_repo.publish_offer(tx, attachment).await?;
}

5 => { // EXCHANGE_BUY
    // INSERT EXCHANGE_REQUEST
    // UPDATE ACCOUNT (balance -= units * rate)
    // UPDATE ACCOUNT_CURRENCY (units += exchanged)
    self.exchange_request_repo.insert(&request).await?;
    self.currency_exchange_repo.exchange_nrcs_for_currency(...).await?;
}

6 => { // EXCHANGE_SELL
    // INSERT EXCHANGE_REQUEST
    // UPDATE ACCOUNT_CURRENCY (units -= sold)
    // UPDATE ACCOUNT (balance += received NRCS)
    self.exchange_request_repo.insert(&request).await?;
    self.currency_exchange_repo.exchange_currency_for_nrcs(...).await?;
}

7 => { // CURRENCY_MINTING
    // INSERT CURRENCY_MINT
    // UPDATE ACCOUNT_CURRENCY (units += minted)
    self.currency_mint_repo.insert(&mint).await?;
    self.account_currency_repo.update_units(sender_id, currency_id, minted_units).await?;
}

8 => { // CURRENCY_DELETION
    // 标记CURRENCY为deleted
    // 清理相关数据
    self.currency_repo.delete(currency_id).await?;
}
```

---

## 📝 实施步骤

### Step 1: 准备工作
1. ✅ 分析Java源码（已完成）
2. ✅ 确认Rust ORM层完整性（已确认完整）
3. ⏳ 检查tx-engine是否已注入所有需要的Repository

### Step 2: 实现核心交易处理器
按优先级顺序实现以下方法：
1. `apply_messaging_attachment()` - 13个subtype
2. `apply_account_control_attachment()` - 2个subtype
3. `apply_data_attachment()` - 2个subtype
4. `apply_light_contract_attachment()` - 2个subtype
5. 补充`apply_colored_coins_attachment()` - 11个新subtype
6. 补充`apply_monetary_system_attachment()` - 9个新subtype

### Step 3: Ledger记录完善
确保每种交易类型都正确写入ACCOUNT_LEDGER：
- 使用正确的EVENT_TYPE（参考ledger_event.rs常量）
- 使用正确的HOLDING_TYPE（NRCS_BALANCE=2, ASSET_BALANCE=4, CURRENCY_BALANCE=6）
- 设置正确的HOLDING_ID（assetId/currencyId）

### Step 4: 测试验证
1. 编译检查：`cargo build --release`
2. 代码质量：`cargo clippy -- -D warnings`
3. 单元测试：`cargo test --lib`
4. 同步测试：启动节点同步NRCS链，对比数据库

---

## 📌 关键注意事项

### 1. 两阶段提交模式
所有交易必须遵循：
```rust
async fn process_transaction(&self, tx: &Transaction) -> ProcessorResult<()> {
    // Phase 1: Pre-check (apply_unconfirmed)
    self.apply_unconfirmed(tx).await?;  // 预扣unconfirmed余额

    // Phase 2: Execute (apply)
    self.apply(tx).await?;  // 正式执行，扣减confirmed余额

    // Phase 3: Persist
    self.tx_repo.insert(&tx_model).await?;  // 写入TRANSACTION表
}
```

### 2. 余额操作顺序（Java参考）
```java
// Sender:
addToBalance(event, txId, -(amountNQT + feeNQT))  // 确认余额减少
addToUnconfirmedBalance(event, txId, +(amountNQT + feeNQT))  // unconfirmed恢复

// Recipient (if exists):
addToBalanceAndUnconfirmedBalance(event, txId, +amountNQT)  // 同时增加两者
```

### 3. Guaranteed Balance规则
只在余额**增加**时记录：
- 发送方：amount+fee都是负数 → 不记录
- 接收方：amount是正数 → 记录到ACCOUNT_GUARANTEED_BALANCE

### 4. Ledger Entry规则
每笔交易产生多条Ledger记录：
- **发送方Fee**: EVENT=TRANSACTION_FEE(50), HOLDING=NRCS_BALANCE(2), CHANGE=-fee
- **发送方Amount**: EVENT=event_type, HOLDING=NRCS_BALANCE(2), CHANGE=-amount
- **接收方Amount**: EVENT=event_type, HOLDING=NRCS_BALANCE(2), CHANGE=+amount
- **资产变更**: EVENT=event_type, HOLDING=ASSET_BALANCE(4), HOLDING_ID=assetId
- **货币变更**: EVENT=event_type, HOLDING=CURRENCY_BALANCE(6), HOLDING_ID=currencyId

---

## ✅ 验证清单

完成后需要验证的数据库表（共70+个）：

### 必须有数据的表（核心功能）
- [ ] ACCOUNT - 所有账户信息
- [ ] TRANSACTION - 所有交易记录
- [ ] BLOCK - 区块数据
- [ ] ACCOUNT_LEDGER - 所有账本条目
- [ ] ACCOUNT_GUARANTED_BALANCE - 担保余额记录
- [ ] PUBLIC_KEY - 公钥（账户控制交易）

### 资产相关（如果链上有资产交易）
- [ ] ASSET - 资产定义
- [ ] ACCOUNT_ASSET - 账户资产余额
- [ ] ASSET_TRANSFER - 资产转账记录
- [ ] ASK_ORDER - 卖单
- [ ] BID_ORDER - 买单
- [ ] TRADE - 成交记录
- [ ] ASSET_PROPERTY - 资产属性

### 货币相关（如果链上有货币交易）
- [ ] CURRENCY - 货币定义
- [ ] ACCOUNT_CURRENCY - 账户货币余额
- [ ] CURRENCY_TRANSFER - 货币转账记录
- [ ] EXCHANGE / EXCHANGE_REQUEST - 兑换请求

### 其他功能（如果有相关交易）
- [ ] ALIAS / ALIAS_OFFER - 别名
- [ ] POLL / VOTE / POLL_RESULT - 投票
- [ ] PHASING_POLL / PHASING_VOTE - 阶段投票
- [ ] TAGGED_DATA / TAG - 标签数据
- [ ] CONTRACT_REFERENCE - 合约引用
- [ ] ACCOUNT_PROPERTY - 账户属性
- [ ] ACCOUNT_LEASE - 余额租赁
- [ ] ACCOUNT_CONTROL_PHASING - 阶段控制
- [ ] SHUFFLING / SHUFFLING_DATA - 混币
- [ ] GOODS / PURCHASE - 数字商品

---

## 🚀 开始实施

**预计工作量**：
- Phase 1 (Messaging + AccountControl + Data + LightContract): ~400行代码
- Phase 2 (补充ColoredCoins + MonetarySystem): ~600行代码
- Phase 3 (Ledger完善 + 测试): ~200行代码
- **总计**: ~1200行新代码

**时间估计**: 2-3小时（含测试验证）
