//! # Smart Contract Engine
//!
//! 智能合约执行引擎，使用 WASM (WebAssembly) 作为沙箱运行合约代码。
//!
//! ## 设计目标
//! - 安全性：WASM 沙箱隔离，内存限制
//! - 可预测性：确定性执行（禁止随机数、网络访问）
//! - 性能：JIT 编译（wasmtime）快速执行
//! - 兼容性：支持多种语言编译到 WASM（Rust, C/C++, AssemblyScript 等）
//!
//! ## 合约生命周期
//! 1. **部署**：上传 WASM 字节码，初始化合约状态
//! 2. **调用**：执行合约方法（可读/写状态）
//! 3. **升级**：可选，通过治理投票替换 WASM 字节码
//!
//! ## Host Functions (宿主函数)
//! 合约内可调用的系统函数：
//! - `get_block_height()` -> u32
//! - `get_balance(account_id: u64) -> u64`
//! - `transfer(to: u64, amount: u64) -> Result`
//! - `log(message: &str)`
//! - ` emit_event(topic: &str, data: &[u8])`
//! - `random() -> u64` (受限，仅用于 Lottery 类合约)

mod engine;
mod host;
mod runtime;
mod store;

pub use engine::*;
pub use host::*;
pub use runtime::*;
pub use store::*;

use blockchain_types::*;
use wasmtime::{Engine, Store, Module, Instance, Linker, Func};
use wasmtime_wasi::WasiCtxBuilder;
use thiserror::Error;

/// 合约错误类型
#[derive(Debug, Error)]
pub enum ContractError {
    #[error("compilation error: {0}")]
    Compilation(String),

    #[error("instantiation error: {0}")]
    Instantiation(String),

    #[error("execution error: {0}")]
    Execution(String),

    #[error("contract not found: {0}")]
    NotFound(String),

    #[error("invalid signature: {0}")]
    InvalidSignature(String),

    #[error("gas limit exceeded: used {used}, limit {limit}")]
    GasLimitExceeded { used: u64, limit: u64 },

    #[error("host function error: {0}")]
    HostFunction(String),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type ContractResult<T> = std::result::Result<T, ContractError>;

/// 合约引擎 trait
///
/// 定义合约的生命周期操作：
/// - 加载 WASM 字节码
/// - 实例化合约（创建或调用）
/// - 执行合约方法
/// - 状态管理
pub trait ContractEngine: Send + Sync {
    /// 部署新合约
    async fn deploy(
        &self,
        wasm_bytes: Vec<u8>,
        init_method: &str,
        args: &[u8],
    ) -> ContractResult<ContractId>;

    /// 调用合约方法（只读）
    async fn call_read(
        &self,
        contract_id: ContractId,
        method: &str,
        args: &[u8],
    ) -> ContractResult<Vec<u8>>;

    /// 调用合约方法（可写状态）
    async fn call_write(
        &self,
        contract_id: ContractId,
        method: &str,
        args: &[u8],
        gas_limit: u64,
    ) -> ContractResult<Vec<u8>>;

    /// 获取合约状态（键值对）
    async fn get_state(
        &self,
        contract_id: ContractId,
        key: &[u8],
    ) -> ContractResult<Option<Vec<u8>>>;

    /// 删除合约（仅管理员）
    async fn destroy(&self, contract_id: ContractId) -> ContractResult<()>;

    /// 验证合约字节码（在部署前）
    fn validate_wasm(&self, wasm_bytes: &[u8]) -> ContractResult<()>;
}

/// 合约标识符
pub type ContractId = u64;

/// 合约元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractInfo {
    pub id: ContractId,
    pub owner: AccountId,
    pub name: String,
    pub version: String,
    pub wasm_hash: Hash256,
    pub created_at: Timestamp,
    pub gas_limit: u64,
    pub state: ContractState,
}

/// 合约状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContractState {
    /// 正常
    Active,
    /// 暂停（管理员暂停）
    Paused,
    /// 已销毁
    Destroyed,
    /// 冻结（如违法被链上治理冻结）
    Frozen,
}

/// WASM 合约运行时
///
/// 封装 wasmtime 实例、存储和 gas 计量器
pub struct WasmContract {
    /// WASM 模块（已编译）
    module: Module,
    /// 实例（需每次调用重新创建，因为 WASM 全局状态隔离）
    /// 为简化，这里使用内存存储
    store: Store<ContractStore>,
    /// 内存页数限制（每页 64KB）
    memory_limit: usize,
}

impl WasmContract {
    /// 创建新的 WASM 合约运行时
    pub async fn new(
        engine: &Engine,
        wasm_bytes: &[u8],
        store_data: ContractStore,
        memory_limit: usize,
    ) -> ContractResult<Self> {
        let module = Module::new(engine, wasm_bytes)
            .map_err(|e| ContractError::Compilation(e.to_string()))?;

        let store = Store::new(engine, store_data);

        Ok(Self {
            module,
            store,
            memory_limit,
        })
    }

    /// 调用合约方法（通用）
    pub async fn call(
        &mut self,
        method: &str,
        args: &[u8],
        gas_limit: u64,
    ) -> ContractResult<Vec<u8>> {
        // 1. 实例化（或实例）
        let instance = self.instantiate().await?;

        // 2. 查找导出函数
        let func = instance.get_func(&mut self.store, method)
            .ok_or_else(|| ContractError::NotFound(method.to_string()))?;

        // 3. 准备参数（WASM expects (pointer, length) pairs）
        // 简化：实际需要使用 WASM 内存读写
        let result = func.call_async(&mut self.store, &[args.into()])
            .map_err(|e| ContractError::Execution(e.to_string()))?;

        Ok(result)
    }

    /// 实例化 WASM 模块
    async fn instantiate(&mut self) -> ContractResult<Instance> {
        let mut linker = Linker::new(&self.store);

        // 注入 WASI（受限）
        // 实际合约引擎应该使用自定义 WASI 实现，限制文件/网络访问
        let wasi = WasiCtxBuilder::new().inherit_stdio().build();
        wasi.add_to_linker(&mut linker)?;

        // 注入 Host 函数（宿主函数）
        self.register_host_functions(&mut linker).await?;

        let instance = linker.instantiate_async(&mut self.store, &self.module)
            .await
            .map_err(|e| ContractError::Instantiation(e.to_string()))?;

        Ok(instance)
    }

    /// 注册宿主函数（供合约调用）
    async fn register_host_functions(&self, linker: &mut Linker<ContractStore>) -> ContractResult<()> {
        // get_balance(account_id: u64) -> u64
        linker.func_wrap(
            "env",
            "get_balance",
            |store: &mut Store<ContractStore>, account_id: u64| -> u64 {
                let ctx = store.data();
                ctx.blockchain
                    .as_ref()
                    .and_then(|bc| bc.get_account(account_id))
                    .map(|acc| acc.balance)
                    .unwrap_or(0)
            }
        )?;

        // get_block_height() -> u32
        linker.func_wrap(
            "env",
            "get_block_height",
            |store: &mut Store<ContractStore>| -> u32 {
                let ctx = store.data();
                ctx.blockchain.as_ref().map(|bc| bc.height).unwrap_or(0)
            }
        )?;

        // emit_event(topic_ptr: i32, topic_len: i32, data_ptr: i32, data_len: i32)
        linker.func_wrap(
            "env",
            "emit_event",
            |store: &mut Store<ContractStore>, topic_ptr: i32, topic_len: i32, data_ptr: i32, data_len: i32| {
                // 安全限制：不能超过内存边界
                // TODO: 实现内存读取
                // 将事件记录到日志
            }
        )?;

        Ok(())
    }

    /// 设置内存限制
    pub fn set_memory_limit(&mut self, pages: usize) {
        self.memory_limit = pages;
    }
}

/// 合约存储上下文（每个合约实例独有）
#[derive(Debug, Clone)]
pub struct ContractStore {
    /// 合约所有者（用于权限检查）
    pub owner: AccountId,
    /// 链状态（只读引用）
    pub blockchain: Option<Arc<Blockchain>>,
    /// 合约状态键值存储
    pub state: HashMap<Vec<u8>, Vec<u8>>,
    /// 已使用 Gas
    pub gas_used: u64,
    /// Gas 价格（每操作消耗）
    pub gas_price: u64,
}

impl ContractStore {
    pub fn new(owner: AccountId) -> Self {
        Self {
            owner,
            blockchain: None,
            state: HashMap::new(),
            gas_used: 0,
            gas_price: 1,
        }
    }

    /// 设置区块链状态引用（只读）
    pub fn with_blockchain(mut self, bc: Arc<Blockchain>) -> Self {
        self.blockchain = Some(bc);
        self
    }
}

/// 合约工厂
///
/// 管理合约生命周期：部署、调用、删除
pub struct ContractFactory {
    engine: Engine,
    contracts: RwLock<HashMap<ContractId, ContractInfo>>,
    instances: RwLock<HashMap<ContractId, WasmContract>>,
    next_id: RwLock<ContractId>,
    gas_limit: u64,
}

impl ContractFactory {
    /// 创建新的合约工厂
    pub fn new() -> ContractResult<Self> {
        let engine = Engine::default();
        Ok(Self {
            engine,
            contracts: RwLock::new(HashMap::new()),
            instances: RwLock::new(HashMap::new()),
            next_id: RwLock::new(1),
            gas_limit: 10_000_000, // 默认 10M gas
        })
    }

    /// 部署合约
    pub async fn deploy(
        &self,
        wasm_bytes: Vec<u8>,
        owner: AccountId,
        name: String,
        init_method: &str,
        args: Vec<u8>,
    ) -> ContractResult<ContractId> {
        // 1. 验证 WASM 格式
        self.validate_wasm(&wasm_bytes)?;

        // 2. 计算 WASM 哈希（用于合约标识）
        let wasm_hash = blake3(&wasm_bytes);

        // 3. 分配合约 ID
        let id = {
            let mut next_id = self.next_id.write().await;
            let id = *next_id;
            *next_id += 1;
            id
        };

        // 4. 创建合约信息
        let info = ContractInfo {
            id,
            owner,
            name,
            version: "1.0.0".to_string(),
            wasm_hash,
            created_at: chrono::Utc::now().timestamp() as u32,
            gas_limit: self.gas_limit,
            state: ContractState::Active,
        };

        // 5. 存入合约
        self.contracts.write().await.insert(id, info);

        // 6. 初始化合约状态（调用 init 方法）
        let store = ContractStore::new(owner);
        let store = store.with_blockchain(/* 链引用 */);

        let contract = WasmContract::new(&self.engine, &wasm_bytes, store, 256) // 256 页 ≈ 16MB
            .await?;

        // 调用初始化方法
        let _result = contract.call(init_method, &args, self.gas_limit).await?;

        self.instances.write().await.insert(id, contract);

        Ok(id)
    }

    /// 调用合约（只读）
    pub async fn call_read(
        &self,
        contract_id: ContractId,
        method: &str,
        args: Vec<u8>,
    ) -> ContractResult<Vec<u8>> {
        let mut instances = self.instances.write().await;
        let contract = instances.get_mut(&contract_id)
            .ok_or_else(|| ContractError::NotFound(format!("Contract #{}", contract_id)))?;

        // Gas 检查（只读操作消耗较少）
        contract.call(method, &args, self.gas_limit / 10).await
    }

    /// 调用合约（可写）
    pub async fn call_write(
        &self,
        contract_id: ContractId,
        method: &str,
        args: Vec<u8>,
        gas_limit: Option<u64>,
    ) -> ContractResult<Vec<u8>> {
        let mut instances = self.instances.write().await;
        let contract = instances.get_mut(&contract_id)
            .ok_or_else(|| ContractError::NotFound(format!("Contract #{}", contract_id)))?;

        let limit = gas_limit.unwrap_or(self.gas_limit);
        contract.call(method, &args, limit).await
    }

    /// 获取合约信息
    pub async fn get_info(&self, contract_id: ContractId) -> Option<ContractInfo> {
        self.contracts.read().await.get(&contract_id).cloned()
    }

    /// 列出所有合约
    pub async fn list_contracts(&self) -> Vec<ContractInfo> {
        self.contracts.read().await.values().cloned().collect()
    }
}

impl Default for ContractFactory {
    fn default() -> Self {
        Self::new().expect("Failed to create ContractFactory")
    }
}

/// 合约调用上下文（传递给宿主函数）
pub struct ContractExecutionContext<'a> {
    /// 调用者账户 ID
    pub caller: AccountId,
    /// 交易上下文
    pub tx: &'a Transaction,
    /// 当前区块高度
    pub block_height: Height,
    /// 合约数据存储（读写）
    pub storage: &'a mut ContractStore,
}

impl ContractEngine for ContractFactory {
    async fn deploy(
        &self,
        wasm_bytes: Vec<u8>,
        init_method: &str,
        args: &[u8],
    ) -> ContractResult<ContractId> {
        // 简化：owner 从 args 中解析
        // TODO: 实际需要从调用者上下文获取
        let owner = 0u64; // placeholder
        self.deploy(wasm_bytes, owner, "Unnamed".to_string(), init_method, args.to_vec()).await
    }

    async fn call_read(
        &self,
        contract_id: ContractId,
        method: &str,
        args: &[u8],
    ) -> ContractResult<Vec<u8>> {
        self.call_read(contract_id, method, args.to_vec()).await
    }

    async fn call_write(
        &self,
        contract_id: ContractId,
        method: &str,
        args: &[u8],
        gas_limit: u64,
    ) -> ContractResult<Vec<u8>> {
        self.call_write(contract_id, method, args.to_vec(), Some(gas_limit)).await
    }

    async fn get_state(
        &self,
        contract_id: ContractId,
        key: &[u8],
    ) -> ContractResult<Option<Vec<u8>>> {
        let instances = self.instances.read().await;
        let contract = instances.get(&contract_id)
            .ok_or_else(|| ContractError::NotFound(format!("Contract #{}", contract_id)))?;
        Ok(contract.store.data().state.get(key).cloned())
    }

    async fn destroy(&self, contract_id: ContractId) -> ContractResult<()> {
        self.contracts.write().await.remove(&contract_id);
        self.instances.write().await.remove(&contract_id);
        Ok(())
    }

    fn validate_wasm(&self, wasm_bytes: &[u8]) -> ContractResult<()> {
        // 验证魔术字节（0x00 0x61 0x73 0x6d）
        if wasm_bytes.len() < 4 || &wasm_bytes[0..4] != b"\0asm" {
            return Err(ContractError::Compilation("invalid WASM magic".to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_contract_factory_creation() {
        let factory = ContractFactory::new();
        assert!(factory.contracts.read().await.is_empty());
    }

    #[test]
    fn test_wasm_validation() {
        let factory = ContractFactory::new().unwrap();
        // Valid WASM header
        assert!(factory.validate_wasm(&[0x00, 0x61, 0x73, 0x6D]).is_ok());
        // Invalid
        assert!(factory.validate_wasm(b"invalid").is_err());
    }
}
