# NRCS Rust 重构技术栈选型报告

## 1. Web 框架：Axum vs Actix Web

### 1.1 候选方案

| 特性 | Axum | Actix Web |
|------|------|-----------|
| 开发状态 | 活跃维护（Tokio 官方） | 活跃维护 |
| 性能基准 | ~380k req/s (hyper) | ~320k req/s (actix-http) |
| 学习曲线 | 中等（基于 tower） | 较陡（Actor 模型概念） |
| 中间件生态 | tower 生态丰富 | Actix-middleware |
| 错误处理 | 类型安全（IntoResponse） | 灵活但需手动 |
| 文档质量 | 优秀（官方 guide） | 良好 |

### 1.2 对比分析

**Axum 优势：**
1. **零成本抽象**：基于 `hyper` + `tower`，无额外开销
2. **类型安全**：Extractor 机制自动提取请求参数，编译时检查
3. **Tokio 深度集成**：无缝使用异步运行时特性
4. **现代 API**：`Router::new().route(...)` 清晰直观
5. **更强的可测试性**：纯函数式处理程序

**Actix Web 优势：**
1. **成熟度高**：生产环境验证多年
2. **功能丰富**：WebSocket、SSE 开箱即用
3. **Actor 可选**：需要状态共享时可用 `web::Data<struct>`

### 1.3 结论

**选择 Axum**

理由：
- NRCS 区块链不需要复杂的 Actor 状态共享，`Arc<Mutex<>>` 足够
- Axum 性能略高，且与 Tokio 生态更融合
- 类型安全的 API 设计减少运行时错误

参考实现（Axum）：

```rust
use axum::{
    routing::post,
    Router, Json, extract::State,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct GetBlockRequest {
    height: u32,
}

#[derive(Serialize)]
struct GetBlockResponse {
    version: u32,
    hash: String,
    // ...
}

async fn get_block(
    State(state): State<AppState>,
    Json(req): Json<GetBlockRequest>,
) -> Result<Json<GetBlockResponse>, AppError> {
    let block = state.blockchain.get_block(req.height).await?;
    Ok(Json(response))
}
```

---

## 2. P2P 网络库：libp2p-rs vs 自研

### 2.1 候选方案

| 特性 | libp2p-rs | 自研（基于 TCP + WebSocket） |
|------|-----------|---------------------------|
| 开发成本 | 低（开箱即用） | 高（数百小时） |
| 功能完整性 | 完整（DHT、Gossipsub、Relay） | 仅基础功能 |
| 性能 | 中等（抽象有开销） | 高（无抽象） |
| 协议灵活性 | 高（多路复用、Yamux） | 需自研 |
| 维护成本 | 低（依赖上游） | 高（自解 bug） |

### 2.2 分析

**libp2p-rs：**
- 成熟度：已经用于 Polkadot、Eth2 等大型项目
- 模块化：可选 `tcp`, `websocket`, `dns`, `identify`, `ping`, `kad` 等
- 缺点：包体积增大（~2MB），抽象层带来少许性能损失（可接受）

**自研：**
- 完全控制协议，性能最优
- 但需要自己实现节点发现、连接管理、消息广播等
- NRCS 已有 Java 实现，可直接移植逻辑，减少风险

### 2.3 结论

**第一阶段：使用 libp2p-rs**

理由：
1. 快速验证架构，避免陷入网络协议细节
2. libp2p 经过大规模网络测试，稳定性有保障
3. 便于后续扩展（如跨公网 NAT 穿透）

实现路线：

```rust
use libp2p::{
    Multiaddr, PeerId,
    identify::{Identify, IdentifyConfig},
    ping::{Ping, PingConfig},
    swarm::{Swarm, SwarmEvent},
    yamux::YamuxConfig,
    tcp::TokioTcpConfig,
    Secio,
};
use futures::stream::StreamExt;

struct P2PService {
    swarm: Swarm<Behaviour>,
    local_peer_id: PeerId,
    // NRCS specific
    blockchain: Arc<Blockchain>,
    tx_pool: Arc<TxPool>,
}

impl P2PService {
    pub async fn start(bind_addr: Multiaddr) -> Result<()> {
        let key = ed25519::SecretKey::generate();
        let peer_id = PeerId::from(key.public());
        let transport = build_transport(key);
        let behaviour = build_behaviour();
        let swarm = Swarm::new(transport, behaviour, peer_id);
        Swarm::listen_on(swarm, bind_addr)?;
        // ...
    }
}

// 自定义协议：区块/交易广播
#[derive(NetworkBehaviour)]
struct Behaviour {
    identify: Identify,
    ping: Ping,
    nr_block_announce: custom_protocol::Behaviour<NrBlockAnnounce>,
    nr_tx_broadcast: custom_protocol::Behaviour<NrTxBroadcast>,
}
```

---

## 3. 数据库 ORM：SQLx vs Diesel

### 3.1 候选方案对比

| 特性 | SQLx | Diesel |
|------|------|--------|
| 查询方式 | 原生 SQL | DSL（类型安全） |
| 编译时检查 | ✅（SQL 语法） | ✅（类型匹配） |
| 异步支持 | ✅（ tokio-postgres） | ⚠️（ async 分支在 v2.0 才稳定） |
| 迁移工具 | sqlx-cli (内置) | diesel-cli |
| 数据库支持 | PostgreSQL, MySQL, SQLite | PostgreSQL, MySQL, SQLite |
| 学习成本 | 低（SQL 基础） | 中（需学 DSL） |
| 复杂查询 | 直接写 SQL | 链式调用 |
| 性能 | 直接查询，无额外开销 | 中等（DSL 转换） |

### 3.2 分析

**SQLx 优势：**
1. **编译时 SQL 校验**：`sqlx::query!()` 在编译时检查表/列是否存在
2. **原生 SQL**：区块链复杂查询（如区块范围、账户交易历史）直接写 SQL 更直观
3. **成熟异步支持**：基于 `tokio-postgres` 和 `mysql_async`
4. **迁移工具**：`sqlx migrate` 开箱即用

**Diesel 优势：**
1. **类型安全 DSL**：`users.filter(name.eq("Alice"))` 编译时安全
2. **Rust 惯用 API**：链式调用更符合 Rust 风格
3. **关联查询**：`has_many`, `belongs_to` 简化多表关联

### 3.3 结论

**选择 SQLx**

理由：
- 区块链查询复杂（多表 JOIN、范围查询），原生 SQL 更便利
- Diesel v2.0 前异步支持不稳定（生产环境风险）
- SQLx 的 `query!` 宏提供编译时检查，兼顾安全与灵活性

示例代码（SQLx + MySQL）：

```rust
#[derive(Debug, Clone, FromRow)]
struct BlockRow {
    id: u64,
    height: u32,
    timestamp: u32,
    previous_block_hash: Vec<u8>,
    payload_hash: Vec<u8>,
    // ...
}

async fn get_block_by_height(pool: &PgPool, height: u32) -> Result<Option<Block>> {
    let row = sqlx::query!(
        "SELECT * FROM block WHERE height = ? LIMIT 1",
        height as i64
    )
    .fetch_optional(pool)
    .await?;

    row.map(|r| Ok(Block {
        id: r.id as u64,
        height: r.height as u32,
        // ...
    })).transpose()
}
```

---

## 4. 智能合约运行时：wasmtime vs wasmer

### 4.1 候选对比

| 特性 | wasmtime | wasmer |
|------|----------|--------|
| 性能 | ⭐⭐⭐⭐⭐ (V8 集成优化) | ⭐⭐⭐⭐ |
| 安全性 | 沙箱成熟（Fastly 生产） | 沙箱成熟 |
| WASI 支持 | ✅ (wasi 0.2) | ✅ |
| 内存限制 | ✅ (实例内存上限) | ✅ |
| AOT 编译 | ⚠️ (实验性) | ✅ (singlepass, cranelift) |
| 包大小 | ~1.5MB (核心) | ~2MB |
| 社区活跃度 | 高（Cloudflare, Fastly） | 中 |
| License | Apache-2.0 | MIT |

### 4.2 分析

**wasmtime 优势：**
1. **性能优异**：基于 Cranelift 编译器后端，JIT 性能高
2. **安全性强**：Fastly 大规模使用，沙箱机制严格
3. **WASI 0.2**：支持最新的 WASI 预览2（线程、引用类型）
4. **RFC 兼容**：紧跟 WASM 标准

**wasmer 优势：**
1. **单文件部署**：`wasmer pack` 可将合约打包为单一文件
2. **更多编译器**：支持 Native（AOT）编译，启动快

### 4.3 结论

**选择 wasmtime**

理由：
- NRCS 合约不需要极端启动速度（JIT 足够快）
- wasmtime 的 WASI 0.2 支持更好，便于合约访问系统调用（文件、网络需限制）
- 社区更活跃，安全更新及时

合约执行示例：

```rust
use wasmtime::{Engine, Store, Module, Instance, Linker, Func};
use wasmtime_wasi::preview1::WasiP1Ctx;

struct ContractContext {
    blockchain: Arc<Blockchain>,
    tx_ctx: TransactionContext, // 交易上下文
}

async fn execute_contract(
    engine: &Engine,
    wasm_bytes: &[u8],
    ctx: ContractContext,
    func_name: &str,
    args: &[u8],
) -> Result<Vec<u8>> {
    let mut store = Store::new(engine, ctx);
    let module = Module::new(engine, wasm_bytes)?;
    let linker = Linker::new(engine);

    // 注入 WASI 和自定义 host 函数
    let wasi = WasiP1Ctx::new();
    wasi.add_to_linker(&linker)?;

    // 自定义 host 函数：查询余额、获取区块高度等
    linker.func_wrap(
        "env",
        "get_block_height",
        |_store: &mut Store<ContractContext>| {
            let ctx = store.data();
            ctx.blockchain.height() as i32
        }
    )?;

    let instance = linker.instantiate_async(&mut store, &module).await?;
    let func = instance.get_func(&mut store, func_name)
        .ok_or_else(|| anyhow!("function not found"))?;

    let result = func.call_async(&mut store, args)?;
    Ok(result)
}
```

---

## 5. 加密库：ed25519-dalek vs ring

### 5.1 对比

| 特性 | ed25519-dalek | ring |
|------|---------------|------|
| 算法支持 | Ed25519 签名 | Ed25519, RSA, ECDSA, AES, CHACHA... |
| API 易用性 | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ (复杂) |
| 性能 | ~50μs (签名) | ~60μs |
| 安全审计 | 经过审计 | 经过审计 |
| 内存安全 | ✅ (纯 Rust) | ✅ (部分 C 绑定) |
| 大小 | ~300KB | ~1.5MB |

### 5.2 结论

**选择 ed25519-dalek**

理由：
- NRCS 需要的是高效的数字签名，Ed25519 足够（现代、安全、快速）
- API 简洁，易于使用
- ring 过于庞大，且 RSA/ECDSA 目前不需要（未来可按需引入）

示例：

```rust
use ed25519_dalek::{Keypair, Signer, Verifier};

#[derive(Clone)]
pub struct CryptoEngine {
    keypair: Keypair,
}

impl CryptoEngine {
    pub fn new(seed: &[u8; 32]) -> Self {
        let secret = ed25519_dalek::SecretKey::from_bytes(seed)
            .expect("seed length 32");
        let public = (&secret).into();
        Self {
            keypair: Keypair { secret, public },
        }
    }

    pub fn sign(&self, msg: &[u8]) -> [u8; 64] {
        *self.keypair.sign(msg)
    }

    pub fn verify(public_key: &[u8], msg: &[u8], sig: &[u8]) -> Result<()> {
        let pk = ed25519_dalek::PublicKey::from_bytes(public_key)?;
        pk.verify(msg, sig)?;
        Ok(())
    }
}
```

---

## 6. 共识算法：自定义 PoS 实现

### 6.1 选型

不依赖现有 PoS 库，而是**自研**，因为：

1. **业务逻辑耦合**：NRCS 的 PoS 依赖 `generation_signature`、`base_target`、`cumulative_difficulty`
2. **性能敏感**：挖矿选择（sorting）需 O(log n) 时间复杂度，需自定义
3. **灵活性**：未来可能支持多链不同共识

### 6.2 实现思路

```rust
/// 权益证明（PoS）算法入口
pub fn select_forger(
    candidates: &[&BlockchainState],
    timestamp: u32,
) -> Result<Option<ForgerSelection>> {
    // 1. 计算每个账户的有效余额（考虑租赁、保留余额）
    // 2. 计算整体难度
    // 3. 根据 generation_signature + timestamp 生成随机数
    // 4. 扫描链上区块（扫描窗口内有效出块者）
    // 5. 返回出块者 + 计算 deadline
}
```

---

## 7. 日志框架：tracing + tower-log

```rust
use tracing::{info, warn, error, debug};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn init_logging() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "nrcs=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}
```

---

## 8. 配置管理

```rust
#[derive(Debug, Deserialize)]
struct Config {
    pub database: DatabaseConfig,
    pub p2p: P2PConfig,
    pub http: HttpConfig,
    pub blockchain: BlockchainConfig,
}

#[derive(Debug, Deserialize)]
struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub pool_max_size: u32,
}
```

使用 `config` crate 支持多格式（YAML/JSON/TOML）：

```rust
let settings = config::Config::try_from(&config::Environment::default())?
    .merge(config::File::with_name("config/default"))?
    .merge(config::Environment::with_prefix("NRCS"))?;
let config: Config = settings.try_deserialize()?;
```

---

## 9. 总结表格

| 组件 | 选型 | 理由 |
|-----|------|------|
| Web 框架 | Axum | 类型安全、性能高、Tokio 生态 |
| P2P | libp2p-rs | 功能完整、大幅降低开发成本 |
| ORM | SQLx | 原生 SQL + 异步 + 编译检查 |
| 合约引擎 | wasmtime | 成熟、安全、性能好 |
| 加密 | ed25519-dalek | 简洁、高效、安全 |
| 日志 | tracing | 结构化日志、与 tower 集成 |
| 配置 | config + dotenv | 灵活、多格式支持 |

---

**报告版本**：v1.0
**创建日期**：2026-04-03
**作者**：Rust 后端架构师
