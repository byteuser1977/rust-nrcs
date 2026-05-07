use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/**
 * 账户变更事件类型枚举（对应 Java NRCS: AccountEvent）
 *
 * 完全兼容 NRCS Java 的 AccountEvent 枚举定义
 */
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AccountEventType {
    /// 余额变更（已确认）
    /// 对应 Java: BALANCE
    Balance,
    /// 未确认余额变更
    /// 对应 Java: UNCONFIRMED_BALANCE
    UnconfirmedBalance,
    /// 资产数量变更（已确认）
    /// 对应 Java: ASSET_BALANCE
    AssetBalance,
    /// 资产数量变更（未确认）
    /// 对应 Java: UNCONFIRMED_ASSET_BALANCE
    UnconfirmedAssetBalance,
    /// 货币单位变更（已确认）
    /// 对应 Java: CURRENCY_BALANCE
    CurrencyBalance,
    /// 货币单位变更（未确认）
    /// 对应 Java: UNCONFIRMED_CURRENCY_BALANCE
    UnconfirmedCurrencyBalance,
    /// 租赁计划已安排
    /// 对应 Java: LEASE_SCHEDULED
    LeaseScheduled,
    /// 租赁已开始
    /// 对应 Java: LEASE_STARTED
    LeaseStarted,
    /// 租赁已结束
    /// 对应 Java: LEASE_ENDED
    LeaseEnded,
    /// 属性已设置
    /// 对应 Java: SET_PROPERTY
    SetProperty,
    /// 属性已删除
    /// 对应 Java: DELETE_PROPERTY
    DeleteProperty,
}

impl std::fmt::Display for AccountEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountEventType::Balance => write!(f, "BALANCE"),
            AccountEventType::UnconfirmedBalance => write!(f, "UNCONFIRMED_BALANCE"),
            AccountEventType::AssetBalance => write!(f, "ASSET_BALANCE"),
            AccountEventType::UnconfirmedAssetBalance => write!(f, "UNCONFIRMED_ASSET_BALANCE"),
            AccountEventType::CurrencyBalance => write!(f, "CURRENCY_BALANCE"),
            AccountEventType::UnconfirmedCurrencyBalance => write!(f, "UNCONFIRMED_CURRENCY_BALANCE"),
            AccountEventType::LeaseScheduled => write!(f, "LEASE_SCHEDULED"),
            AccountEventType::LeaseStarted => write!(f, "LEASE_STARTED"),
            AccountEventType::LeaseEnded => write!(f, "LEASE_ENDED"),
            AccountEventType::SetProperty => write!(f, "SET_PROPERTY"),
            AccountEventType::DeleteProperty => write!(f, "DELETE_PROPERTY"),
        }
    }
}

/**
 * 账户账本持有类型（对应 Java NRCS: LedgerHolding）
 *
 * 用于标识账户账本条目中记录的是哪种类型的余额/资产
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedgerHolding {
    /// 未确认 NRCS 余额 (code=1)
    UnconfirmedNrcsBalance = 1,
    /// 已确认 NRCS 余额 (code=2)
    NrcsBalance = 2,
    /// 未确认资产余额 (code=3)
    UnconfirmedAssetBalance = 3,
    /// 已确认资产余额 (code=4)
    AssetBalance = 4,
    /// 未确认货币余额 (code=5)
    UnconfirmedCurrencyBalance = 5,
    /// 已确认货币余额 (code=6)
    CurrencyBalance = 6,
}

impl LedgerHolding {
    /**
     * 从代码值创建 LedgerHolding
     */
    pub fn from_code(code: i32) -> Option<Self> {
        match code {
            1 => Some(LedgerHolding::UnconfirmedNrcsBalance),
            2 => Some(LedgerHolding::NrcsBalance),
            3 => Some(LedgerHolding::UnconfirmedAssetBalance),
            4 => Some(LedgerHolding::AssetBalance),
            5 => Some(LedgerHolding::UnconfirmedCurrencyBalance),
            6 => Some(LedgerHolding::CurrencyBalance),
            _ => None,
        }
    }

    /**
     * 获取代码值
     */
    pub fn code(&self) -> i32 {
        *self as i32
    }

    /**
     * 检查是否为未确认类型
     */
    pub fn is_unconfirmed(&self) -> bool {
        matches!(
            self,
            LedgerHolding::UnconfirmedNrcsBalance
                | LedgerHolding::UnconfirmedAssetBalance
                | LedgerHolding::UnconfirmedCurrencyBalance
        )
    }
}

impl std::fmt::Display for LedgerHolding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LedgerHolding::UnconfirmedNrcsBalance => write!(f, "UNCONFIRMED_NRCS_BALANCE"),
            LedgerHolding::NrcsBalance => write!(f, "NRCS_BALANCE"),
            LedgerHolding::UnconfirmedAssetBalance => write!(f, "UNCONFIRMED_ASSET_BALANCE"),
            LedgerHolding::AssetBalance => write!(f, "ASSET_BALANCE"),
            LedgerHolding::UnconfirmedCurrencyBalance => write!(f, "UNCONFIRMED_CURRENCY_BALANCE"),
            LedgerHolding::CurrencyBalance => write!(f, "CURRENCY_BALANCE"),
        }
    }
}

/**
 * 账户变更事件
 *
 * 当 Account 对象的任何字段发生变更时产生此事件，
 * 用于通知所有注册的监听器进行联动更新。
 *
 * 对应 Java NRCS: Account 类中的 listeners.notify(this, AccountEvent.XXX)
 */
#[derive(Debug, Clone)]
pub struct AccountEvent {
    /// 账户 ID
    pub account_id: i64,

    /// 事件类型
    pub event_type: AccountEventType,

    /// 变更详情（字段名 -> 变化值）
    /// 例如: {"balance": 100, "unconfirmed_balance": 100}
    pub changes: HashMap<String, i64>,

    /// 关联的区块高度
    pub height: i32,

    /// 时间戳（Unix 时间戳）
    pub timestamp: i64,

    /// 触发来源（可选）
    /// 例如: "transaction_apply", "block_reward", "genesis_init"
    pub source: Option<String>,

    /// 关联的交易 ID（如果有）
    pub transaction_id: Option<i64>,

    /// 关联的资产/货币 ID（对于资产或货币事件）
    pub holding_id: Option<i64>,
}

impl AccountEvent {
    /**
     * 创建新的账户事件
     */
    pub fn new(account_id: i64, event_type: AccountEventType) -> Self {
        Self {
            account_id,
            event_type,
            changes: HashMap::new(),
            height: 0,
            timestamp: chrono::Utc::now().timestamp(),
            source: None,
            transaction_id: None,
            holding_id: None,
        }
    }

    /**
     * 添加变更字段
     */
    pub fn with_change(mut self, field: &str, value: i64) -> Self {
        self.changes.insert(field.to_string(), value);
        self
    }

    /**
     * 设置区块高度
     */
    pub fn with_height(mut self, height: i32) -> Self {
        self.height = height;
        self
    }

    /**
     * 设置触发来源
     */
    pub fn with_source(mut self, source: &str) -> Self {
        self.source = Some(source.to_string());
        self
    }

    /**
     * 设置关联交易 ID
     */
    pub fn with_transaction(mut self, tx_id: i64) -> Self {
        self.transaction_id = Some(tx_id);
        self
    }

    /**
     * 设置关联的资产/货币 ID
     */
    pub fn with_holding_id(mut self, id: i64) -> Self {
        self.holding_id = Some(id);
        self
    }

    /**
     * 获取指定字段的变更值
     */
    pub fn get_change(&self, field: &str) -> Option<i64> {
        self.changes.get(field).copied()
    }
}

/**
 * 事件处理器类型定义
 */
pub type EventHandler = Box<dyn Fn(&AccountEvent) + Send + Sync>;

/**
 * 资产事件处理器类型（对应 Java: Listener<AccountAsset>）
 */
pub type AssetEventHandler = Box<dyn Fn(i64, i64, i64, i64) + Send + Sync>;

/**
 * 货币事件处理器类型（对应 Java: Listener<AccountCurrency>）
 */
pub type CurrencyEventHandler = Box<dyn Fn(i64, i64, i64, i64) + Send + Sync>;

/**
 * 账户事件分发器（对应 Java NRCS: Listeners<Account, AccountEvent>）
 *
 * 负责将 Account 变更事件分发给所有注册的处理器。
 * 这是实现"观察者模式"的核心组件。
 *
 * # 设计原则（参考 Java NRCS）
 * - 线程安全（使用 RwLock 保护处理器列表）
 * - 松耦合（Account 不需要知道有哪些监听器）
 * - 可扩展（运行时动态注册/注销处理器）
 * - 多组监听器支持（账户、资产、货币等独立管理）
 *
 * # 与 Java NRCS 的对应关系
 * - `listeners` → Account 主监听器列表
 * - `asset_listeners` → 资产监听器列表
 * - `currency_listeners` → 货币监听器列表
 */
pub struct EventDispatcher {
    /// 账户主事件处理器列表
    account_handlers: RwLock<Vec<EventHandler>>,

    /// 资产事件处理器列表
    asset_handlers: RwLock<Vec<AssetEventHandler>>,

    /// 货币事件处理器列表
    currency_handlers: RwLock<Vec<CurrencyEventHandler>>,
}

impl EventDispatcher {
    /**
     * 创建新的事件分发器实例
     */
    pub fn new() -> Self {
        Self {
            account_handlers: RwLock::new(Vec::new()),
            asset_handlers: RwLock::new(Vec::new()),
            currency_handlers: RwLock::new(Vec::new()),
        }
    }

    // ==================== 账户事件处理 ====================

    /**
     * 注册账户事件处理器（对应 Java: Account.addListener()）
     *
     * # 参数
     * - `handler`: 处理函数，接收 AccountEvent 引用
     *
     * # 示例
     * ```rust
     * dispatcher.on_account_event(|event| {
     *     if event.event_type == AccountEventType::Balance {
     *         println!("Balance changed for account {}", event.account_id);
     *     }
     * });
     * ```
     */
    pub async fn on_account_event<F>(&self, handler: F)
    where
        F: Fn(&AccountEvent) + Send + Sync + 'static,
    {
        let mut handlers = self.account_handlers.write().await;
        handlers.push(Box::new(handler));

        debug!(
            handler_count = handlers.len(),
            "Registered new account event handler"
        );
    }

    /**
     * 分发账户事件给所有注册的处理器（对应 Java: listeners.notify()）
     *
     * # 参数
     * - `event`: 要分发的事件
     *
     * # 注意
     * 所有处理器会按注册顺序依次调用。
     * 如果某个处理器 panic，会记录错误但不会影响其他处理器。
     */
    pub async fn dispatch_account_event(&self, event: &AccountEvent) {
        let handlers = self.account_handlers.read().await;

        debug!(
            account = event.account_id,
            event_type = %event.event_type,
            change_count = event.changes.len(),
            handler_count = handlers.len(),
            "Dispatching account event"
        );

        for handler in handlers.iter() {
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| handler(event))) {
                Ok(()) => {}
                Err(e) => {
                    let message = if let Some(s) = e.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = e.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "Unknown error".to_string()
                    };

                    warn!(
                        account = event.account_id,
                        error = message,
                        "Account event handler panicked"
                    );
                }
            }
        }
    }

    // ==================== 资产事件处理 ====================

    /**
     * 注册资产事件处理器（对应 Java: Account.addAssetListener()）
     *
     * # 参数
     * - `handler`: 处理函数，参数为 (account_id, asset_id, quantity, unconfirmed_quantity)
     */
    pub async fn on_asset_event<F>(&self, handler: F)
    where
        F: Fn(i64, i64, i64, i64) + Send + Sync + 'static,
    {
        let mut handlers = self.asset_handlers.write().await;
        handlers.push(Box::new(handler));

        debug!(
            handler_count = handlers.len(),
            "Registered new asset event handler"
        );
    }

    /**
     * 分发资产事件给所有注册的处理器（对应 Java: assetListeners.notify()）
     *
     * # 参数
     * - `account_id`: 账户 ID
     * - `asset_id`: 资产 ID
     * - `quantity`: 已确认数量
     * - `unconfirmed_quantity`: 未确认数量
     */
    pub async fn dispatch_asset_event(
        &self,
        account_id: i64,
        asset_id: i64,
        quantity: i64,
        unconfirmed_quantity: i64,
    ) {
        let handlers = self.asset_handlers.read().await;

        debug!(
            account = account_id,
            asset = asset_id,
            quantity,
            unconfirmed_quantity,
            handler_count = handlers.len(),
            "Dispatching asset event"
        );

        for handler in handlers.iter() {
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                handler(account_id, asset_id, quantity, unconfirmed_quantity)
            })) {
                Ok(()) => {}
                Err(e) => {
                    let message = if let Some(s) = e.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = e.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "Unknown error".to_string()
                    };

                    warn!(
                        account = account_id,
                        asset = asset_id,
                        error = message,
                        "Asset event handler panicked"
                    );
                }
            }
        }
    }

    // ==================== 货币事件处理 ====================

    /**
     * 注册货币事件处理器（对应 Java: Account.addCurrencyListener()）
     *
     * # 参数
     * - `handler`: 处理函数，参数为 (account_id, currency_id, units, unconfirmed_units)
     */
    pub async fn on_currency_event<F>(&self, handler: F)
    where
        F: Fn(i64, i64, i64, i64) + Send + Sync + 'static,
    {
        let mut handlers = self.currency_handlers.write().await;
        handlers.push(Box::new(handler));

        debug!(
            handler_count = handlers.len(),
            "Registered new currency event handler"
        );
    }

    /**
     * 分发货币事件给所有注册的处理器（对应 Java: currencyListeners.notify()）
     *
     * # 参数
     * - `account_id`: 账户 ID
     * - `currency_id`: 货币 ID
     * - `units`: 已确认单位数
     * - `unconfirmed_units`: 未确认单位数
     */
    pub async fn dispatch_currency_event(
        &self,
        account_id: i64,
        currency_id: i64,
        units: i64,
        unconfirmed_units: i64,
    ) {
        let handlers = self.currency_handlers.read().await;

        debug!(
            account = account_id,
            currency = currency_id,
            units,
            unconfirmed_units,
            handler_count = handlers.len(),
            "Dispatching currency event"
        );

        for handler in handlers.iter() {
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                handler(account_id, currency_id, units, unconfirmed_units)
            })) {
                Ok(()) => {}
                Err(e) => {
                    let message = if let Some(s) = e.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = e.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "Unknown error".to_string()
                    };

                    warn!(
                        account = account_id,
                        currency = currency_id,
                        error = message,
                        "Currency event handler panicked"
                    );
                }
            }
        }
    }

    // ==================== 通用方法 ====================

    /**
     * 清除所有注册的处理器
     *
     * 通常在测试或重启时使用
     */
    pub async fn clear_all_handlers(&self) {
        let mut account_handlers = self.account_handlers.write().await;
        let mut asset_handlers = self.asset_handlers.write().await;
        let mut currency_handlers = self.currency_handlers.write().await;

        let account_count = account_handlers.len();
        let asset_count = asset_handlers.len();
        let currency_count = currency_handlers.len();

        account_handlers.clear();
        asset_handlers.clear();
        currency_handlers.clear();

        debug!(
            removed_accounts = account_count,
            removed_assets = asset_count,
            removed_currencies = currency_count,
            "Cleared all event handlers"
        );
    }

    /**
     * 获取当前注册的处理器数量
     */
    pub async fn handler_counts(&self) -> (usize, usize, usize) {
        let account = self.account_handlers.read().await.len();
        let asset = self.asset_handlers.read().await.len();
        let currency = self.currency_handlers.read().await.len();
        (account, asset, currency)
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

/**
 * 账户事件监听器 Trait（用于自定义监听器实现）
 *
 * 对应 Java NRCS: Listener<T> 接口
 */
#[async_trait::async_trait]
pub trait AccountEventListener: Send + Sync {
    /**
     * 处理账户事件
     *
     * # 参数
     * - `event`: 账户变更事件
     */
    async fn handle(&self, event: &AccountEvent);
}

/**
 * 资产事件监听器 Trait（用于自定义资产监听器）
 *
 * 对应 Java NRCS: Listener<AccountAsset>
 */
#[async_trait::async_trait]
pub trait AssetEventListener: Send + Sync {
    /**
     * 处理资产事件
     *
     * # 参数
     * - `account_id`: 账户 ID
     * - `asset_id`: 资产 ID
     * - `quantity`: 新的已确认数量
     * - `unconfirmed_quantity`: 新的未确认数量
     */
    async fn handle(&self, account_id: i64, asset_id: i64, quantity: i64, unconfirmed_quantity: i64);
}

/**
 * 货币事件监听器 Trait（用于自定义货币监听器）
 *
 * 对应 Java NRCS: Listener<AccountCurrency>
 */
#[async_trait::async_trait]
pub trait CurrencyEventListener: Send + Sync {
    /**
     * 处理货币事件
     *
     * # 参数
     * - `account_id`: 账户 ID
     * - `currency_id`: 货币 ID
     * - `units`: 新的已确认单位数
     * - `unconfirmed_units`: 新的未确认单位数
     */
    async fn handle(&self, account_id: i64, currency_id: i64, units: i64, unconfirmed_units: i64);
}

/**
 * 默认账户日志监听器（参考 Java NRCS: 基础日志功能）
 *
 * 记录所有账户变更事件到日志系统
 */
pub struct DefaultLoggingListener;

#[async_trait::async_trait]
impl AccountEventListener for DefaultLoggingListener {
    async fn handle(&self, event: &AccountEvent) {
        info!(
            account = event.account_id,
            event_type = %event.event_type,
            changes = ?event.changes,
            height = event.height,
            tx_id = ?event.transaction_id,
            source = ?event.source,
            holding_id = ?event.holding_id,
            "Account event detected (multi-table sync ready)"
        );
    }
}

/**
 * 默认资产日志监听器（参考 Java NRCS: AssetEventHandler）
 *
 * 记录所有资产变更事件到日志系统
 */
pub struct DefaultAssetLoggingListener;

#[async_trait::async_trait]
impl AssetEventListener for DefaultAssetLoggingListener {
    async fn handle(&self, account_id: i64, asset_id: i64, quantity: i64, unconfirmed_quantity: i64) {
        info!(
            account = account_id,
            asset = asset_id,
            quantity,
            unconfirmed_quantity,
            "Asset balance changed (multi-table sync ready)"
        );
    }
}

/**
 * 默认货币日志监听器（参考 Java NRCS: CurrencyEventHandler）
 *
 * 记录所有货币变更事件到日志系统
 */
pub struct DefaultCurrencyLoggingListener;

#[async_trait::async_trait]
impl CurrencyEventListener for DefaultCurrencyLoggingListener {
    async fn handle(&self, account_id: i64, currency_id: i64, units: i64, unconfirmed_units: i64) {
        info!(
            account = account_id,
            currency = currency_id,
            units,
            unconfirmed_units,
            "Currency balance changed (multi-table sync ready)"
        );
    }
}

/**
 * 便捷方法：设置默认监听器集合（参考 Java NRCS: FundingMonitor.init()）
 *
 * 自动注册所有默认的事件监听器：
 * - 账户余额变更日志
 * - 资产余额变更日志
 * - 货币余额变更日志
 *
 * # 示例
 * ```rust
 * use orm::events::{EventDispatcher, setup_default_listeners};
 *
 * let dispatcher = Arc::new(EventDispatcher::new());
 * setup_default_listeners(&dispatcher);
 * ```
 */
pub async fn setup_default_listeners(dispatcher: &Arc<EventDispatcher>) {
    // 注册账户事件日志监听器（对应 Java: Account.addListener(new AccountEventHandler(), AccountEvent.BALANCE)）
    dispatcher.on_account_event(move |event| {

        // 使用 tokio spawn 处理异步操作（简化版：直接调用同步日志）
        // 注意：生产环境应该使用异步 runtime
        if event.event_type == AccountEventType::Balance
            || event.event_type == AccountEventType::UnconfirmedBalance
        {
            info!(
                account = event.account_id,
                balance_change = ?event.changes.get("balance"),
                unconfirmed_balance_change = ?event.changes.get("unconfirmed_balance"),
                height = event.height,
                source = ?event.source,
                "[DefaultListener] Account balance change detected"
            );
        }
    }).await;

    // 注册资产事件日志监听器（对应 Java: Account.addAssetListener(new AssetEventHandler(), AccountEvent.ASSET_BALANCE)）
    dispatcher.on_asset_event(|account_id, asset_id, quantity, unconfirmed_quantity| {
        info!(
            account = account_id,
            asset = asset_id,
            quantity,
            unconfirmed_quantity,
            "[DefaultListener] Asset balance change detected"
        );
    }).await;

    // 注册货币事件日志监听器（对应 Java: Account.addCurrencyListener(new CurrencyEventHandler(), AccountEvent.CURRENCY_BALANCE)）
    dispatcher.on_currency_event(|account_id, currency_id, units, unconfirmed_units| {
        info!(
            account = account_id,
            currency = currency_id,
            units,
            unconfirmed_units,
            "[DefaultListener] Currency balance change detected"
        );
    }).await;

    info!("Default event listeners registered successfully");
}

// ==================== 账本系统（对应 Java NRCS: AccountLedger） ====================

/**
 * 账本事件类型枚举（对应 Java NRCS: LedgerEvent）
 *
 * 完整定义所有 NRCS 系统中的账本事件类型，
 * 用于记录账户余额变更的历史轨迹。
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LedgerEvent {
    // 区块和交易事件
    BlockGenerated = 1,
    RejectPhasedTransaction = 2,
    TransactionFee = 50,

    // TYPE_PAYMENT
    OrdinaryPayment = 3,

    // TYPE_MESSAGING
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

    // TYPE_COLORED_COINS
    AssetAskOrderCancellation = 14,
    AssetAskOrderPlacement = 15,
    AssetBidOrderCancellation = 16,
    AssetBidOrderPlacement = 17,
    AssetDividendPayment = 18,
    AssetIssuance = 19,
    AssetTrade = 20,
    AssetTransfer = 21,
    AssetDelete = 49,
    // V2.0 新增
    AssetPropertySet = 65,
    AssetPropertyDelete = 66,
    AssetIncrease = 61,
    AssetSetPhasingControl = 62,
    AssetLongValuePropertySet = 67,

    // TYPE_DIGITAL_GOODS
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

    // TYPE_ACCOUNT_CONTROL
    AccountControlEffectiveBalanceLeasing = 32,
    AccountControlPhasingOnly = 55,

    // TYPE_CURRENCY
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

    // TYPE_DATA
    TaggedDataUpload = 47,
    TaggedDataExtend = 48,

    // TYPE_SHUFFLING
    ShufflingRegistration = 51,
    ShufflingProcessing = 52,
    ShufflingCancellation = 53,
    ShufflingDistribution = 54,

    // TYPE_COIN_EXCHANGE
    CoinExchangeOrderIssue = 58,
    CoinExchangeOrderCancel = 59,
    CoinExchangeTrade = 60,

    // TYPE_LIGHT_CONTRACT
    ContractReferenceSet = 63,
    ContractReferenceDelete = 64,
}

impl LedgerEvent {
    /**
     * 从代码值创建 LedgerEvent（对应 Java: fromCode()）
     */
    pub fn from_code(code: i16) -> Option<Self> {
        match code {
            1 => Some(LedgerEvent::BlockGenerated),
            2 => Some(LedgerEvent::RejectPhasedTransaction),
            3 => Some(LedgerEvent::OrdinaryPayment),
            4 => Some(LedgerEvent::AccountInfo),
            5 => Some(LedgerEvent::AliasAssignment),
            6 => Some(LedgerEvent::AliasBuy),
            7 => Some(LedgerEvent::AliasDelete),
            8 => Some(LedgerEvent::AliasSell),
            9 => Some(LedgerEvent::ArbitraryMessage),
            10 => Some(LedgerEvent::HubAnnouncement),
            11 => Some(LedgerEvent::PhasingVoteCasting),
            12 => Some(LedgerEvent::PollCreation),
            13 => Some(LedgerEvent::VoteCasting),
            14 => Some(LedgerEvent::AssetAskOrderCancellation),
            15 => Some(LedgerEvent::AssetAskOrderPlacement),
            16 => Some(LedgerEvent::AssetBidOrderCancellation),
            17 => Some(LedgerEvent::AssetBidOrderPlacement),
            18 => Some(LedgerEvent::AssetDividendPayment),
            19 => Some(LedgerEvent::AssetIssuance),
            20 => Some(LedgerEvent::AssetTrade),
            21 => Some(LedgerEvent::AssetTransfer),
            22 => Some(LedgerEvent::DigitalGoodsDelisted),
            23 => Some(LedgerEvent::DigitalGoodsDelisting),
            24 => Some(LedgerEvent::DigitalGoodsDelivery),
            25 => Some(LedgerEvent::DigitalGoodsFeedback),
            26 => Some(LedgerEvent::DigitalGoodsListing),
            27 => Some(LedgerEvent::DigitalGoodsPriceChange),
            28 => Some(LedgerEvent::DigitalGoodsPurchase),
            29 => Some(LedgerEvent::DigitalGoodsPurchaseExpired),
            30 => Some(LedgerEvent::DigitalGoodsQuantityChange),
            31 => Some(LedgerEvent::DigitalGoodsRefund),
            32 => Some(LedgerEvent::AccountControlEffectiveBalanceLeasing),
            33 => Some(LedgerEvent::CurrencyDeletion),
            34 => Some(LedgerEvent::CurrencyDistribution),
            35 => Some(LedgerEvent::CurrencyExchange),
            36 => Some(LedgerEvent::CurrencyExchangeBuy),
            37 => Some(LedgerEvent::CurrencyExchangeSell),
            38 => Some(LedgerEvent::CurrencyIssuance),
            39 => Some(LedgerEvent::CurrencyMinting),
            40 => Some(LedgerEvent::CurrencyOfferExpired),
            41 => Some(LedgerEvent::CurrencyOfferReplaced),
            42 => Some(LedgerEvent::CurrencyPublishExchangeOffer),
            43 => Some(LedgerEvent::CurrencyReserveClaim),
            44 => Some(LedgerEvent::CurrencyReserveIncrease),
            45 => Some(LedgerEvent::CurrencyTransfer),
            46 => Some(LedgerEvent::CurrencyUndoCrowdfunding),
            47 => Some(LedgerEvent::TaggedDataUpload),
            48 => Some(LedgerEvent::TaggedDataExtend),
            49 => Some(LedgerEvent::AssetDelete),
            50 => Some(LedgerEvent::TransactionFee),
            51 => Some(LedgerEvent::ShufflingRegistration),
            52 => Some(LedgerEvent::ShufflingProcessing),
            53 => Some(LedgerEvent::ShufflingCancellation),
            54 => Some(LedgerEvent::ShufflingDistribution),
            55 => Some(LedgerEvent::AccountControlPhasingOnly),
            56 => Some(LedgerEvent::AccountProperty),
            57 => Some(LedgerEvent::AccountPropertyDelete),
            58 => Some(LedgerEvent::CoinExchangeOrderIssue),
            59 => Some(LedgerEvent::CoinExchangeOrderCancel),
            60 => Some(LedgerEvent::CoinExchangeTrade),
            61 => Some(LedgerEvent::AssetIncrease),
            62 => Some(LedgerEvent::AssetSetPhasingControl),
            63 => Some(LedgerEvent::ContractReferenceSet),
            64 => Some(LedgerEvent::ContractReferenceDelete),
            65 => Some(LedgerEvent::AssetPropertySet),
            66 => Some(LedgerEvent::AssetPropertyDelete),
            67 => Some(LedgerEvent::AssetLongValuePropertySet),
            68 => Some(LedgerEvent::AccountLongValuePropertySet),
            656 => Some(LedgerEvent::AccountPropertySet),
            _ => None,
        }
    }

    /**
     * 获取事件代码（对应 Java: getCode()）
     */
    pub fn code(&self) -> i16 {
        *self as i16
    }

    /**
     * 检查是否为交易事件（对应 Java: isTransaction()）
     */
    pub fn is_transaction(&self) -> bool {
        !matches!(self, LedgerEvent::BlockGenerated)
    }
}

impl std::fmt::Display for LedgerEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LedgerEvent::BlockGenerated => write!(f, "BLOCK_GENERATED"),
            LedgerEvent::OrdinaryPayment => write!(f, "ORDINARY_PAYMENT"),
            LedgerEvent::TransactionFee => write!(f, "TRANSACTION_FEE"),
            LedgerEvent::AssetTransfer => write!(f, "ASSET_TRANSFER"),
            LedgerEvent::CurrencyTransfer => write!(f, "CURRENCY_TRANSFER"),
            _ => write!(f, "LEDGER_EVENT_{}", self.code()),
        }
    }
}

/**
 * 账本条目结构体（对应 Java NRCS: LedgerEntry）
 *
 * 记录账户余额变更的完整历史信息，
 * 用于审计追踪和数据恢复。
 */
#[derive(Debug, Clone)]
pub struct LedgerEntry {
    /// 数据库 ID（自增主键）
    pub db_id: i64,

    /// 账户 ID
    pub account_id: i64,

    /// 事件类型（对应 LedgerEvent.code()）
    pub event_type: i16,

    /// 事件 ID（通常是交易 ID 或区块 ID）
    pub event_id: i64,

    /// 持有类型（对应 LedgerHolding.code()）
    pub holding_type: Option<i32>,

    /// 持有 ID（资产/货币 ID）
    pub holding_id: Option<i64>,

    /// 变更金额
    pub change: i64,

    /// 变更后余额
    pub balance: i64,

    /// 关联区块 ID
    pub block_id: i64,

    /// 区块高度
    pub height: i32,

    /// 时间戳
    pub timestamp: i32,
}

impl LedgerEntry {
    /**
     * 创建新的账本条目（对应 Java: new LedgerEntry(event, eventId, accountId, holding, holdingId, change, balance)）
     */
    pub fn new(
        event: LedgerEvent,
        event_id: i64,
        account_id: i64,
        holding: Option<LedgerHolding>,
        holding_id: Option<i64>,
        change: i64,
        balance: i64,
        block_id: i64,
        height: i32,
        timestamp: i32,
    ) -> Self {
        Self {
            db_id: 0,
            account_id,
            event_type: event.code(),
            event_id,
            holding_type: holding.map(|h| h.code()),
            holding_id,
            change,
            balance,
            block_id,
            height,
            timestamp,
        }
    }

    /**
     * 创建简化版账本条目（无持有信息）
     */
    pub fn new_simple(
        event: LedgerEvent,
        event_id: i64,
        account_id: i64,
        change: i64,
        balance: i64,
        block_id: i64,
        height: i32,
        timestamp: i32,
    ) -> Self {
        Self::new(event, event_id, account_id, None, None, change, balance, block_id, height, timestamp)
    }

    /**
     * 更新变更金额（对应 Java: updateChange()）
     */
    pub fn update_change(&mut self, amount: i64) {
        self.change += amount;
    }

    /**
     * 获取事件类型
     */
    pub fn get_event(&self) -> Option<LedgerEvent> {
        LedgerEvent::from_code(self.event_type)
    }

    /**
     * 获取持有类型
     */
    pub fn get_holding(&self) -> Option<LedgerHolding> {
        self.holding_type.and_then(|t| LedgerHolding::from_code(t))
    }
}

// ==================== FundingMonitor（对应 Java NRCS: FundingMonitor） ====================

/**
 * 监控账户持有类型（对应 Java NRCS: HoldingType）
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoldingType {
    Nrcs,
    Asset,
    Currency,
}

impl std::fmt::Display for HoldingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HoldingType::Nrcs => write!(f, "NRCS"),
            HoldingType::Asset => write!(f, "ASSET"),
            HoldingType::Currency => write!(f, "CURRENCY"),
        }
    }
}

/**
 * 监控账户配置（对应 Java NRCS: MonitoredAccount）
 */
#[derive(Debug, Clone)]
pub struct MonitoredAccountConfig {
    /// 被监控的账户 ID
    pub account_id: i64,

    /// 持有类型
    pub holding_type: HoldingType,

    /// 持有 ID（资产/货币 ID）
    pub holding_id: Option<i64>,

    /// 触发阈值
    pub threshold: i64,

    /// 充值金额
    pub funding_amount: i64,

    /// 充值源账户 ID
    pub funding_account_id: i64,
}

impl MonitoredAccountConfig {
    /**
     * 创建新的监控配置
     */
    pub fn new(
        account_id: i64,
        holding_type: HoldingType,
        holding_id: Option<i64>,
        threshold: i64,
        funding_amount: i64,
        funding_account_id: i64,
    ) -> Self {
        Self {
            account_id,
            holding_type,
            holding_id,
            threshold,
            funding_amount,
            funding_account_id,
        }
    }
}

/**
 * 账户监控服务（对应 Java NRCS: FundingMonitor）
 *
 * 监控指定账户的余额变化，当余额低于阈值时自动发起充值交易。
 * 这是 NRCS 系统中用于保证关键账户持续运行的核心组件。
 */
pub struct FundingMonitor {
    /// 是否已停止
    stopped: RwLock<bool>,

    /// 是否已启动
    started: RwLock<bool>,

    /// 监控账户列表
    monitored_accounts: RwLock<Vec<MonitoredAccountConfig>>,

    /// 待处理的事件队列
    pending_events: RwLock<Vec<MonitoredAccountConfig>>,

    /// 事件分发器引用
    dispatcher: Arc<EventDispatcher>,
}

impl FundingMonitor {
    /**
     * 创建新的监控服务实例
     */
    pub fn new(dispatcher: Arc<EventDispatcher>) -> Arc<Self> {
        Arc::new(Self {
            stopped: RwLock::new(false),
            started: RwLock::new(false),
            monitored_accounts: RwLock::new(Vec::new()),
            pending_events: RwLock::new(Vec::new()),
            dispatcher,
        })
    }

    /**
     * 初始化监控服务（对应 Java: FundingMonitor.init()）
     *
     * 注册以下监听器：
     * - AccountEventHandler → 监控 NRCS 余额变更
     * - AssetEventHandler → 监控资产余额变更
     * - CurrencyEventHandler → 监控货币余额变更
     */
    pub async fn init(self: &Arc<Self>) {
        if *self.stopped.read().await {
            panic!("Funding monitor processing has been stopped");
        }
        if *self.started.read().await {
            return;
        }

        // 注册账户余额监控（对应 Java: Account.addListener(new AccountEventHandler(), AccountEvent.BALANCE)）
        let monitor = Arc::clone(self);
        self.dispatcher.on_account_event(move |event| {
            let monitor_inner = Arc::clone(&monitor);
            if event.event_type == AccountEventType::Balance {
                let account_id = event.account_id;
                let balance_change = event.get_change("balance").unwrap_or(0);
                tokio::spawn(async move {
                    monitor_inner.check_balance_event(
                        account_id,
                        HoldingType::Nrcs,
                        None,
                        balance_change,
                    ).await;
                });
            }
        }).await;

        // 注册资产余额监控（对应 Java: Account.addAssetListener(new AssetEventHandler(), AccountEvent.ASSET_BALANCE)）
        let monitor = Arc::clone(self);
        self.dispatcher.on_asset_event(move |account_id, asset_id, quantity, _| {
            let monitor_inner = Arc::clone(&monitor);
            tokio::spawn(async move {
                monitor_inner.check_balance_event(account_id, HoldingType::Asset, Some(asset_id), quantity).await;
            });
        }).await;

        // 注册货币余额监控（对应 Java: Account.addCurrencyListener(new CurrencyEventHandler(), AccountEvent.CURRENCY_BALANCE)）
        let monitor = Arc::clone(self);
        self.dispatcher.on_currency_event(move |account_id, currency_id, units, _| {
            let monitor_inner = Arc::clone(&monitor);
            tokio::spawn(async move {
                monitor_inner.check_balance_event(account_id, HoldingType::Currency, Some(currency_id), units).await;
            });
        }).await;

        *self.started.write().await = true;

        info!(
            "[FundingMonitor] Initialization completed (compatible with Java NRCS)"
        );
    }

    /**
     * 添加监控账户（对应 Java: FundingMonitor.addMonitoredAccount()）
     */
    pub async fn add_monitored_account(&self, config: MonitoredAccountConfig) {
        let mut accounts = self.monitored_accounts.write().await;

        // 检查是否已存在
        if !accounts.iter().any(|a| a.account_id == config.account_id && a.holding_type == config.holding_type) {
            info!(
                account = config.account_id,
                holding_type = %config.holding_type,
                threshold = config.threshold,
                "[FundingMonitor] Added monitored account"
            );
            accounts.push(config);
        }
    }

    /**
     * 移除监控账户
     */
    pub async fn remove_monitored_account(&self, account_id: i64, holding_type: HoldingType) {
        let mut accounts = self.monitored_accounts.write().await;
        accounts.retain(|a| !(a.account_id == account_id && a.holding_type == holding_type));
    }

    /**
     * 检查余额变更事件（核心逻辑）
     */
    async fn check_balance_event(
        &self,
        target_account_id: i64,
        holding_type: HoldingType,
        holding_id: Option<i64>,
        current_balance: i64,
    ) {
        if *self.stopped.read().await {
            return;
        }

        let accounts = self.monitored_accounts.read().await;

        // 查找匹配的监控账户
        for config in accounts.iter() {
            if config.account_id == target_account_id
                && config.holding_type == holding_type
                && config.holding_id == holding_id
                && current_balance < config.threshold
            {
                // 添加到待处理队列
                let mut pending = self.pending_events.write().await;
                if !pending.iter().any(|c| c.account_id == config.account_id) {
                    pending.push(config.clone());
                    warn!(
                        account = config.account_id,
                        balance = current_balance,
                        threshold = config.threshold,
                        "[FundingMonitor] ⚠️ Balance below threshold! Queued for funding"
                    );
                }
            }
        }
    }

    /**
     * 处理待充值事件（对应 Java: FundingMonitor.processBcesEvent()）
     *
     * 注意：当前版本只记录日志，不实际发起交易。
     * 生产环境需要集成 TransactionProcessor 来构建和广播交易。
     */
    pub async fn process_pending_events(&self) {
        if *self.stopped.read().await {
            return;
        }

        let mut pending = self.pending_events.write().await;
        let events: Vec<MonitoredAccountConfig> = pending.drain(..).collect();

        for config in events.iter() {
            info!(
                account = config.account_id,
                holding_type = %config.holding_type,
                amount = config.funding_amount,
                source = config.funding_account_id,
                "[FundingMonitor] Processing funding request (logging mode)"
            );

            // TODO: 生产环境实现
            // 1. 使用 TransactionProcessor 构建充值交易
            // 2. 验证充值源账户余额充足
            // 3. 广播交易到网络
            // 4. 更新 config.height 为当前高度

            warn!(
                "[FundingMonitor] ⚠️ Auto-funding not implemented in current version (log-only mode)"
            );
        }

        if !events.is_empty() {
            info!(
                count = events.len(),
                "[FundingMonitor] Processed {} funding requests",
                events.len()
            );
        }
    }

    /**
     * 停止监控服务（对应 Java: shutdown()）
     */
    pub async fn shutdown(&self) {
        if *self.started.read().await && !*self.stopped.read().await {
            *self.stopped.write().await = true;

            // 清空待处理队列
            let mut pending = self.pending_events.write().await;
            pending.clear();

            info!("[FundingMonitor] Service shutdown complete");
        }
    }

    /**
     * 获取当前监控的账户数量
     */
    pub async fn monitored_count(&self) -> usize {
        self.monitored_accounts.read().await.len()
    }

    /**
     * 获取待处理事件数量
     */
    pub async fn pending_count(&self) -> usize {
        self.pending_events.read().await.len()
    }
}

// ==================== 数据一致性检查器 ====================

/**
 * 数据一致性检查结果
 */
#[derive(Debug, Clone)]
pub struct ConsistencyCheckResult {
    /// 是否一致
    pub is_consistent: bool,

    /// 账户 ID
    pub account_id: i64,

    /// 检查项列表
    pub checks: Vec<ConsistencyCheckItem>,
}

/**
 * 单个检查项
 */
#[derive(Debug, Clone)]
pub struct ConsistencyCheckItem {
    /// 检查名称
    pub name: String,

    /// 是否通过
    pub passed: bool,

    /// 详细信息
    pub detail: String,
}

/**
 * 数据一致性检查器
 *
 * 用于验证跨表数据的一致性，确保：
 * - account 表与 public_key 表一致
 * - account 表与 ACCOUNT_ASSET 表一致
 * - 已确认余额与未确认余额的关系正确
 */
pub struct DataConsistencyChecker {
    /// 事件分发器引用
    dispatcher: Arc<EventDispatcher>,
}

impl DataConsistencyChecker {
    /**
     * 创建新的检查器实例
     */
    pub fn new(dispatcher: Arc<EventDispatcher>) -> Self {
        Self { dispatcher }
    }

    /**
     * 执行完整的一致性检查
     *
     * 对应 Java: Account.checkBalance() 的扩展实现
     */
    pub async fn check_account_consistency(
        &self,
        account_id: i64,
        confirmed_balance: i64,
        unconfirmed_balance: i64,
    ) -> ConsistencyCheckResult {
        let mut checks = Vec::new();

        // 检查 1：确认余额非负
        checks.push(ConsistencyCheckItem {
            name: "confirmed_balance_non_negative".to_string(),
            passed: confirmed_balance >= 0,
            detail: format!("confirmed_balance={}", confirmed_balance),
        });

        // 检查 2：未确认余额非负
        checks.push(ConsistencyCheckItem {
            name: "unconfirmed_balance_non_negative".to_string(),
            passed: unconfirmed_balance >= 0,
            detail: format!("unconfirmed_balance={}", unconfirmed_balance),
        });

        // 检查 3：未确认余额 <= 确认余额
        checks.push(ConsistencyCheckItem {
            name: "unconfirms_not_exceeds_confirmed".to_string(),
            passed: unconfirmed_balance <= confirmed_balance,
            detail: format!(
                "unconfirmed={} <= confirmed={} ? {}",
                unconfirmed_balance,
                confirmed_balance,
                unconfirmed_balance <= confirmed_balance
            ),
        });

        let all_passed = checks.iter().all(|c| c.passed);

        if !all_passed {
            warn!(
                account = account_id,
                confirmed = confirmed_balance,
                unconfirmed = unconfirmed_balance,
                checks = ?checks,
                "[ConsistencyChecker] ❌ Data inconsistency detected!"
            );

            // 分发不一致事件
            let inconsistency_event = AccountEvent::new(account_id, AccountEventType::Balance)
                .with_change("confirmed_balance", confirmed_balance)
                .with_change("unconfirmed_balance", unconfirmed_balance)
                .with_source("consistency_check")
                .with_height(0); // 将在调用处设置实际高度

            self.dispatcher.dispatch_account_event(&inconsistency_event).await;
        } else {
            debug!(
                account = account_id,
                "[ConsistencyChecker] ✅ All consistency checks passed"
            );
        }

        ConsistencyCheckResult {
            is_consistent: all_passed,
            account_id,
            checks,
        }
    }

    /**
     * 检查资产余额一致性
     */
    pub async fn check_asset_consistency(
        &self,
        account_id: i64,
        asset_id: i64,
        confirmed_quantity: i64,
        unconfirmed_quantity: i64,
    ) -> ConsistencyCheckResult {
        let mut checks = Vec::new();

        checks.push(ConsistencyCheckItem {
            name: "asset_quantity_non_negative".to_string(),
            passed: confirmed_quantity >= 0,
            detail: format!(
                "asset={}, quantity={}",
                asset_id, confirmed_quantity
            ),
        });

        checks.push(ConsistencyCheckItem {
            name: "asset_unconfirmed_non_negative".to_string(),
            passed: unconfirmed_quantity >= 0,
            detail: format!(
                "asset={}, unconfirmed={}",
                asset_id, unconfirmed_quantity
            ),
        });

        checks.push(ConsistencyCheckItem {
            name: "asset_unconfirms_within_confirmed".to_string(),
            passed: unconfirmed_quantity <= confirmed_quantity,
            detail: format!(
                "asset={}, unconfirmed={} <= confirmed={} ? {}",
                asset_id,
                unconfirmed_quantity,
                confirmed_quantity,
                unconfirmed_quantity <= confirmed_quantity
            ),
        });

        let all_passed = checks.iter().all(|c| c.passed);

        if !all_passed {
            warn!(
                account = account_id,
                asset = asset_id,
                confirmed = confirmed_quantity,
                unconfirmed = unconfirmed_quantity,
                "[ConsistencyChecker] ❌ Asset data inconsistency detected!"
            );
        }

        ConsistencyCheckResult {
            is_consistent: all_passed,
            account_id,
            checks,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicI64, Ordering};

    #[tokio::test]
    async fn test_account_event_creation() {
        let event = AccountEvent::new(12345, AccountEventType::Balance)
            .with_change("balance", 1000)
            .with_change("unconfirmed_balance", 1000)
            .with_height(866640)
            .with_source("test_transaction")
            .with_transaction(987654321)
            .with_holding_id(1046280091729872666);

        assert_eq!(event.account_id, 12345);
        assert_eq!(event.event_type, AccountEventType::Balance);
        assert_eq!(event.get_change("balance"), Some(1000));
        assert_eq!(event.height, 866640);
        assert_eq!(event.transaction_id, Some(987654321));
        assert_eq!(event.holding_id, Some(1046280091729872666));
    }

    #[tokio::test]
    async fn test_ledger_holding_conversion() {
        assert_eq!(LedgerHolding::from_code(1), Some(LedgerHolding::UnconfirmedNrcsBalance));
        assert_eq!(LedgerHolding::from_code(2), Some(LedgerHolding::NrcsBalance));
        assert_eq!(LedgerHolding::from_code(99), None);

        assert!(LedgerHolding::UnconfirmedNrcsBalance.is_unconfirmed());
        assert!(!LedgerHolding::NrcsBalance.is_unconfirmed());

        assert_eq!(LedgerHolding::NrcsBalance.code(), 2);
    }

    #[tokio::test]
    async fn test_event_dispatcher_basic() {
        let dispatcher = EventDispatcher::new();
        let call_count = Arc::new(AtomicI64::new(0));
        let count_clone = Arc::clone(&call_count);

        dispatcher.on_account_event(move |_event| {
            count_clone.fetch_add(1, Ordering::SeqCst);
        }).await;

        let event = AccountEvent::new(100, AccountEventType::Balance).with_change("balance", 500);
        dispatcher.dispatch_account_event(&event).await;

        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_asset_event_dispatch() {
        use std::sync::atomic::{AtomicI64, Ordering};

        let dispatcher = EventDispatcher::new();
        let last_account_id = Arc::new(AtomicI64::new(0));
        let last_asset_id = Arc::new(AtomicI64::new(0));
        let last_quantity = Arc::new(AtomicI64::new(0));
        let last_unconfirmed = Arc::new(AtomicI64::new(0));

        let acc_id = Arc::clone(&last_account_id);
        let ast_id = Arc::clone(&last_asset_id);
        let qty = Arc::clone(&last_quantity);
        let unconf = Arc::clone(&last_unconfirmed);

        dispatcher.on_asset_event(move |account_id, asset_id, quantity, unconfirmed_qty| {
            acc_id.store(account_id, Ordering::SeqCst);
            ast_id.store(asset_id, Ordering::SeqCst);
            qty.store(quantity, Ordering::SeqCst);
            unconf.store(unconfirmed_qty, Ordering::SeqCst);
        }).await;

        dispatcher
            .dispatch_asset_event(200, 300, 1000, 950)
            .await;

        assert_eq!(last_account_id.load(Ordering::SeqCst), 200);
        assert_eq!(last_asset_id.load(Ordering::SeqCst), 300);
        assert_eq!(last_quantity.load(Ordering::SeqCst), 1000);
        assert_eq!(last_unconfirmed.load(Ordering::SeqCst), 950);
    }

    #[tokio::test]
    async fn test_currency_event_dispatch() {
        use std::sync::atomic::{AtomicI64, Ordering};

        let dispatcher = EventDispatcher::new();
        let last_account_id = Arc::new(AtomicI64::new(0));
        let last_currency_id = Arc::new(AtomicI64::new(0));
        let last_units = Arc::new(AtomicI64::new(0));
        let last_unconfirmed = Arc::new(AtomicI64::new(0));

        let acc_id = Arc::clone(&last_account_id);
        let cur_id = Arc::clone(&last_currency_id);
        let units = Arc::clone(&last_units);
        let unconf = Arc::clone(&last_unconfirmed);

        dispatcher.on_currency_event(move |account_id, currency_id, units_val, unconfirmed_units| {
            acc_id.store(account_id, Ordering::SeqCst);
            cur_id.store(currency_id, Ordering::SeqCst);
            units.store(units_val, Ordering::SeqCst);
            unconf.store(unconfirmed_units, Ordering::SeqCst);
        }).await;

        dispatcher
            .dispatch_currency_event(400, 500, 2000, 1900)
            .await;

        assert_eq!(last_account_id.load(Ordering::SeqCst), 400);
        assert_eq!(last_currency_id.load(Ordering::SeqCst), 500);
        assert_eq!(last_units.load(Ordering::SeqCst), 2000);
        assert_eq!(last_unconfirmed.load(Ordering::SeqCst), 1900);
    }

    #[tokio::test]
    async fn test_multiple_handlers() {
        let dispatcher = EventDispatcher::new();
        let count1 = Arc::new(AtomicI64::new(0));
        let count2 = Arc::new(AtomicI64::new(0));

        let c1 = Arc::clone(&count1);
        let c2 = Arc::clone(&count2);

        dispatcher.on_account_event(move |_event| {
            c1.fetch_add(1, Ordering::SeqCst);
        }).await;

        dispatcher.on_account_event(move |_event| {
            c2.fetch_add(1, Ordering::SeqCst);
        }).await;

        let event = AccountEvent::new(999, AccountEventType::UnconfirmedBalance);
        dispatcher.dispatch_account_event(&event).await;

        assert_eq!(count1.load(Ordering::SeqCst), 1);
        assert_eq!(count2.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_handler_counts() {
        let dispatcher = EventDispatcher::new();

        assert_eq!(dispatcher.handler_counts().await, (0, 0, 0));

        dispatcher.on_account_event(|_| {}).await;
        dispatcher.on_asset_event(|_, _, _, _| {}).await;
        dispatcher.on_currency_event(|_, _, _, _| {}).await;

        assert_eq!(dispatcher.handler_counts().await, (1, 1, 1));
    }

    #[tokio::test]
    async fn test_clear_all_handlers() {
        let dispatcher = EventDispatcher::new();

        dispatcher.on_account_event(|_| {}).await;
        dispatcher.on_asset_event(|_, _, _, _| {}).await;

        assert_eq!(dispatcher.handler_counts().await, (1, 1, 0));

        dispatcher.clear_all_handlers().await;

        assert_eq!(dispatcher.handler_counts().await, (0, 0, 0));
    }

    #[tokio::test]
    async fn test_setup_default_listeners() {
        let dispatcher = Arc::new(EventDispatcher::new());

        setup_default_listeners(&dispatcher).await;

        let counts = dispatcher.handler_counts().await;
        assert!(counts.0 >= 1); // 至少有账户监听器
        assert!(counts.1 >= 1); // 至少有资产监听器
        assert!(counts.2 >= 1); // 至少有货币监听器
    }

    #[tokio::test]
    async fn test_event_type_display() {
        assert_eq!(format!("{}", AccountEventType::Balance), "BALANCE");
        assert_eq!(format!("{}", AccountEventType::UnconfirmedBalance), "UNCONFIRMED_BALANCE");
        assert_eq!(format!("{}", AccountEventType::AssetBalance), "ASSET_BALANCE");
        assert_eq!(format!("{}", AccountEventType::LeaseStarted), "LEASE_STARTED");
    }

    #[tokio::test]
    async fn test_ledger_holding_display() {
        assert_eq!(
            format!("{}", LedgerHolding::NrcsBalance),
            "NRCS_BALANCE"
        );
        assert_eq!(
            format!("{}", LedgerHolding::UnconfirmedAssetBalance),
            "UNCONFIRMED_ASSET_BALANCE"
        );
    }

    // ==================== LedgerEvent 测试 ====================

    #[test]
    fn test_ledger_event_from_code() {
        assert_eq!(LedgerEvent::from_code(1), Some(LedgerEvent::BlockGenerated));
        assert_eq!(LedgerEvent::from_code(3), Some(LedgerEvent::OrdinaryPayment));
        assert_eq!(LedgerEvent::from_code(21), Some(LedgerEvent::AssetTransfer));
        assert_eq!(LedgerEvent::from_code(50), Some(LedgerEvent::TransactionFee));
        assert_eq!(LedgerEvent::from_code(9999), None);
    }

    #[test]
    fn test_ledger_event_code() {
        assert_eq!(LedgerEvent::BlockGenerated.code(), 1);
        assert_eq!(LedgerEvent::OrdinaryPayment.code(), 3);
        assert_eq!(LedgerEvent::TransactionFee.code(), 50);
    }

    #[test]
    fn test_ledger_event_is_transaction() {
        assert!(!LedgerEvent::BlockGenerated.is_transaction());
        assert!(LedgerEvent::OrdinaryPayment.is_transaction());
        assert!(LedgerEvent::AssetTransfer.is_transaction());
    }

    #[test]
    fn test_ledger_event_display() {
        assert_eq!(format!("{}", LedgerEvent::BlockGenerated), "BLOCK_GENERATED");
        assert_eq!(format!("{}", LedgerEvent::OrdinaryPayment), "ORDINARY_PAYMENT");
        assert_eq!(format!("{}", LedgerEvent::TransactionFee), "TRANSACTION_FEE");
    }

    // ==================== LedgerEntry 测试 ====================

    #[test]
    fn test_ledger_entry_creation() {
        let entry = LedgerEntry::new(
            LedgerEvent::OrdinaryPayment,
            123456789,
            9876543210,
            Some(LedgerHolding::NrcsBalance),
            None,
            -10000000,
            900000000,
            111222333,
            866640,
            40674,
        );

        assert_eq!(entry.db_id, 0);
        assert_eq!(entry.account_id, 9876543210);
        assert_eq!(entry.event_type, 3); // OrdinaryPayment.code()
        assert_eq!(entry.event_id, 123456789);
        assert_eq!(entry.holding_type, Some(2)); // NrcsBalance.code()
        assert_eq!(entry.holding_id, None);
        assert_eq!(entry.change, -10000000);
        assert_eq!(entry.balance, 900000000);
        assert_eq!(entry.block_id, 111222333);
        assert_eq!(entry.height, 866640);
        assert_eq!(entry.timestamp, 40674);
    }

    #[test]
    fn test_ledger_entry_simple() {
        let entry = LedgerEntry::new_simple(
            LedgerEvent::AssetTransfer,
            999888777,
            1234567890,
            500,
            1500,
            444555666,
            100000,
            50506,
        );

        assert_eq!(entry.event_type, 21); // AssetTransfer.code()
        assert_eq!(entry.holding_type, None);
        assert_eq!(entry.change, 500);
        assert_eq!(entry.balance, 1500);
    }

    #[test]
    fn test_ledger_entry_update_change() {
        let mut entry = LedgerEntry::new_simple(
            LedgerEvent::TransactionFee,
            111222333,
            9999999999,
            -1000,
            99999,
            777888999,
            200000,
            60707,
        );

        entry.update_change(-500);
        assert_eq!(entry.change, -1500);

        entry.update_change(2000);
        assert_eq!(entry.change, 500);
    }

    #[test]
    fn test_ledger_entry_getters() {
        let entry = LedgerEntry::new(
            LedgerEvent::CurrencyTransfer,
            123,
            456,
            Some(LedgerHolding::CurrencyBalance),
            Some(789),
            100,
            200,
            321,
            400,
            500,
        );

        assert_eq!(entry.get_event(), Some(LedgerEvent::CurrencyTransfer));
        assert_eq!(entry.get_holding(), Some(LedgerHolding::CurrencyBalance));

        // Test with invalid code
        let mut invalid_entry = LedgerEntry::new(
            LedgerEvent::BlockGenerated,
            0,
            0,
            None,
            None,
            0,
            0,
            0,
            0,
            0,
        );
        invalid_entry.holding_type = Some(99);
        assert_eq!(invalid_entry.get_holding(), None);
    }

    // ==================== FundingMonitor 测试 ====================

    #[tokio::test]
    async fn test_funding_monitor_creation() {
        let dispatcher = Arc::new(EventDispatcher::new());
        let monitor = FundingMonitor::new(Arc::clone(&dispatcher));

        assert_eq!(monitor.monitored_count().await, 0);
        assert_eq!(monitor.pending_count().await, 0);
    }

    #[tokio::test]
    async fn test_funding_monitor_add_account() {
        let dispatcher = Arc::new(EventDispatcher::new());
        let monitor = FundingMonitor::new(Arc::clone(&dispatcher));

        let config = MonitoredAccountConfig::new(
            1234567890,
            HoldingType::Nrcs,
            None,
            100000000, // 阈值: 1 NRCS
            1000000000, // 充值: 10 NRCS
            9876543210, // 充值源
        );

        monitor.add_monitored_account(config.clone()).await;
        assert_eq!(monitor.monitored_count().await, 1);

        // 添加重复账户应该被忽略
        monitor.add_monitored_account(config.clone()).await;
        assert_eq!(monitor.monitored_count().await, 1);
    }

    #[tokio::test]
    async fn test_funding_monitor_remove_account() {
        let dispatcher = Arc::new(EventDispatcher::new());
        let monitor = FundingMonitor::new(Arc::clone(&dispatcher));

        let config = MonitoredAccountConfig::new(
            1234567890,
            HoldingType::Nrcs,
            None,
            100000000,
            1000000000,
            9876543210,
        );

        monitor.add_monitored_account(config).await;
        assert_eq!(monitor.monitored_count().await, 1);

        monitor.remove_monitored_account(1234567890, HoldingType::Nrcs).await;
        assert_eq!(monitor.monitored_count().await, 0);
    }

    #[tokio::test]
    async fn test_funding_monitor_shutdown() {
        let dispatcher = Arc::new(EventDispatcher::new());
        let monitor = FundingMonitor::new(Arc::clone(&dispatcher));

        monitor.shutdown().await;

        // 停止后应该无法添加监控账户（因为 started=false）
        // 注意：shutdown 只在 started=true 时才设置 stopped=true
        // 所以这里测试的是正常流程
        assert!(true); // 简单验证 shutdown 不 panic
    }

    // ==================== DataConsistencyChecker 测试 ====================

    #[tokio::test]
    async fn test_consistency_checker_normal_case() {
        let dispatcher = Arc::new(EventDispatcher::new());
        let checker = DataConsistencyChecker::new(Arc::clone(&dispatcher));

        let result = checker.check_account_consistency(12345, 1000000000, 800000000).await;

        assert!(result.is_consistent);
        assert_eq!(result.account_id, 12345);
        assert_eq!(result.checks.len(), 3);
        assert!(result.checks.iter().all(|c| c.passed));
    }

    #[tokio::test]
    async fn test_consistency_checker_negative_balance() {
        let dispatcher = Arc::new(EventDispatcher::new());
        let checker = DataConsistencyChecker::new(Arc::clone(&dispatcher));

        let result = checker.check_account_consistency(12345, -100, 0).await;

        assert!(!result.is_consistent);
        assert!(!result.checks[0].passed); // confirmed_balance_non_negative 失败
    }

    #[tokio::test]
    async fn test_consistency_checker_unconfirmed_exceeds_confirmed() {
        let dispatcher = Arc::new(EventDispatcher::new());
        let checker = DataConsistencyChecker::new(Arc::clone(&dispatcher));

        let result = checker.check_account_consistency(12345, 100, 200).await;

        assert!(!result.is_consistent);
        assert!(!result.checks[2].passed); // unconfirms_not_exceeds_confirmed 失败
    }

    #[tokio::test]
    async fn test_consistency_checker_asset_normal() {
        let dispatcher = Arc::new(EventDispatcher::new());
        let checker = DataConsistencyChecker::new(Arc::clone(&dispatcher));

        let result = checker.check_asset_consistency(12345, 1046280091729872666, 1000, 800).await;

        assert!(result.is_consistent);
        assert_eq!(result.checks.len(), 3);
    }

    #[tokio::test]
    async fn test_consistency_checker_asset_negative() {
        let dispatcher = Arc::new(EventDispatcher::new());
        let checker = DataConsistencyChecker::new(Arc::clone(&dispatcher));

        let result = checker.check_asset_consistency(12345, 1046280091729872666, -5, 0).await;

        assert!(!result.is_consistent);
        assert!(!result.checks[0].passed);
    }

    // ==================== HoldingType 和 MonitoredAccountConfig 测试 ====================

    #[test]
    fn test_holding_type_display() {
        assert_eq!(format!("{}", HoldingType::Nrcs), "NRCS");
        assert_eq!(format!("{}", HoldingType::Asset), "ASSET");
        assert_eq!(format!("{}", HoldingType::Currency), "CURRENCY");
    }

    #[test]
    fn test_monitored_account_config_creation() {
        let config = MonitoredAccountConfig::new(
            1234567890,
            HoldingType::Asset,
            Some(9876543210),
            100,
            1000,
            1122334455,
        );

        assert_eq!(config.account_id, 1234567890);
        assert_eq!(config.holding_type, HoldingType::Asset);
        assert_eq!(config.holding_id, Some(9876543210));
        assert_eq!(config.threshold, 100);
        assert_eq!(config.funding_amount, 1000);
        assert_eq!(config.funding_account_id, 1122334455);
    }
}
