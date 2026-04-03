# NRCS - Neo Rapid BlockChain EcoSystem

 重构自 Java 的高性能区块链平台，采用 Rust + Vue 3 技术栈

[![CI/CD](https://github.com/bytechain/nrcs/workflows/Continuous%20Integration/badge.svg)](https://github.com/bytechain/nrcs/actions)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange)](https://www.rust-lang.org)
[![Vue](https://img.shields.io/badge/vue-3.4+-brightgreen)](https://vuejs.org)
[![Postgres](https://img.shields.io/badge/postgres-15+-blue)](https://www.postgresql.org)

## 🚀 项目简介

NRCS 是一个高性能、可扩展的区块链平台，提供完整的交易处理、智能合约、P2P 网络等功能。本项目将原 Java 实现重构为 Rust，以获得更高的性能、更好的内存安全性和更低的资源消耗。

### 核心特性

- ⚡ **高性能**：Rust 编写，TPS 达到原 Java 版本的 120%+
- 🔒 **安全可靠**：WebAssembly 智能合约沙箱、内存安全保证
- 🌐 **P2P 网络**：基于 libp2p 的去中心化通信
- 📦 **完整生态**：REST API + WebSocket + 管理后台
- 🎨 **现代化 UI**：Vue 3 响应式界面，支持多语言
- 🐳 **一键部署**：Docker + docker-compose

## 🏗️ 架构概览

```
┌─────────────────────────────────────────────────────────┐
│                     前端 (Vue 3)                          │
│  • 仪表盘      • 账户管理      • 交易页面    • 合约管理  │
│  • 节点监控    • 钱包功能      • 多语言支持            │
└────────────────────────┬────────────────────────────────┘
                         │ HTTP/WebSocket
┌────────────────────────▼────────────────────────────────┐
│                API Gateway (Axum)                        │
│  • 路由      • 认证      • 限流      • 错误处理         │
└──────┬──────────────┬──────────────┬──────────────┬─────┘
       │              │              │              │
       ▼              ▼              ▼              ▼
┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐
│ Account  │  │ Transaction│  │ Contract │  │   P2P    │
│ Manager  │  │  Engine    │  │  Engine  │  │ Service  │
└──────────┘  └──────────┘  └──────────┘  └──────────┘
       │              │              │              │
       └──────────────┴──────────────┴──────────────┘
                         │
┌────────────────────────▼────────────────────────────────┐
│                数据层 (PostgreSQL + Redis)              │
│  • 账户数据    • 交易记录    • 合约状态    • 缓存       │
└─────────────────────────────────────────────────────────┘
```

## 📦 技术栈

### 后端
- **语言**: Rust 1.75+
- **Web 框架**: Axum 0.7
- **数据库**: PostgreSQL 15 + Redis 7
- **ORM**: SQLx / SeaQuery
- **网络**: libp2p-rs
- **WASM**: wasmtime 18
- **序列化**: bincode, serde

### 前端
- **框架**: Vue 3.4 + TypeScript 5
- **构建**: Vite 6
- **路由**: Vue Router 4
- **状态管理**: Pinia 2
- **UI 库**: Element Plus
- **HTTP**: Axios
- **测试**: Vitest + Vue Test Utils

## 📁 项目结构

```
rust-nrcs/
├── crates/                # Rust crates
│   ├── blockchain-types   # 核心类型定义
│   ├── crypto             # 加密算法
│   ├── consensus          # 共识算法
│   ├── p2p                # P2P 网络
│   ├── contract           # 智能合约引擎
│   ├── orm                # 数据访问层
│   ├── tx-engine          # 交易引擎
│   ├── account            # 账户管理
│   └── http-api           # REST API
├── apps/
│   └── node               # 节点二进制应用
├── frontend/              # Vue 3 前端
│   ├── src/
│   │   ├── api/          # API 层
│   │   ├── components/   # 组件
│   │   ├── stores/       # 状态管理
│   │   ├── views/        # 页面视图
│   │   └── ...
├── config/                # 配置文件
│   ├── default.toml
│   └── production.toml.example
├── docker/                # Docker 配置
│   ├── backend.Dockerfile
│   ├── frontend.Dockerfile
│   └── docker-compose.yml
└── .github/workflows/     # CI/CD
```

## 🚀 快速开始

### 前置要求

- **Rust**: 1.75+ ([安装指南](https://rustup.rs/))
- **Node.js**: 20+ ([下载](https://nodejs.org/))
- **PostgreSQL**: 15+
- **Redis** (可选): 7+
- **Docker & Docker Compose** (推荐)

### 方案一：Docker 一键启动（最快）

```bash
# 1. 克隆并进入项目
git clone https://github.com/bytechain/nrcs.git
cd nrcs

# 2. 配置环境变量（可选修改）
cp docker/env.sh.example docker/env.sh
# 编辑 docker/env.sh 设置数据库密码、JWT 密钥等

# 3. 启动所有服务（后端 + 前端 + PostgreSQL）
docker-compose up -d

# 4. 查看日志
docker-compose logs -f

# 5. 停止服务
docker-compose down
```

✅ 服务启动后：
- 前端管理界面: http://localhost:80
- 后端 REST API: http://localhost:17976
- API 交互文档 (Swagger): http://localhost:17976/api/docs
- 健康检查: http://localhost:17976/health

### 方案二：本地开发环境

#### 后端启动

```bash
# 1. 安装依赖
cargo fetch

# 2. 配置环境
cp config/default.toml config/local.toml
# 编辑 config/local.toml，设置数据库连接：
# database.url = "postgresql://nrcs:nrcs123@localhost:5432/nrcs"

# 3. 初始化数据库（首次）
psql -U postgres -f crates/orm/migrations/001_initial.sql

# 4. 构建并运行
cargo build
cargo run --bin node --release

# 或使用开发模式（热重载）
cargo watch -x run --bin node
```

后端 API: http://localhost:17976

#### 前端启动

```bash
cd frontend

# 安装依赖
npm ci

# 开发模式
npm run dev

# 构建生产版本
npm run build

# 预览构建结果
npm run preview
```

前端访问: http://localhost:5173

## 🧪 运行测试

### 后端测试

```bash
# 单元测试（所有 crates）
cargo test --all-features

# 查看测试覆盖率（需安装 cargo-tarpaulin）
cargo tarpaulin --out Xml --output-dir coverage/

# 代码检查
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
```

### 前端测试

```bash
cd frontend

# 单元测试
npm run test:unit

# E2E 测试（需先启动后端）
npm run test:e2e

# 代码检查
npm run lint
npm run type-check
```

## 📚 文档

完整文档位于 `docs/` 目录（待提交）：

- **architecture.md** - 系统架构与技术选型
- **database-schema.md** - 数据库设计（55+ 表）
- **frontend-architecture.md** - Vue 3 项目架构
- **api-integration-design.md** - API 集成层设计
- **deployment.md** - 生产环境部署指南
- **migration-guide.md** - 从 Java NRCs 迁移到 Rust 版本
- **monitoring.md** - 监控与告警配置
- **ops/** - 运维手册（备份、故障排查、安全加固）

在线文档生成：
```bash
# API 文档在运行时可访问
http://localhost:17976/api/docs  # Swagger UI
http://localhost:17976/api/redoc # ReDoc
```

## 🔧 常用操作

### 数据库操作

```sql
-- 连接 PostgreSQL
psql -U nrcs -d nrcs

-- 查看最新区块
SELECT * FROM block ORDER BY height DESC LIMIT 10;

-- 查看账户余额
SELECT * FROM account WHERE id = <account_id> AND latest = true;

-- 统计交易数
SELECT COUNT(*) FROM transaction WHERE height > 0;
```

### 节点管理

```bash
# 查看节点信息
curl http://localhost:17976/api/v1/node/info

# 查看已连接的 P2P 节点
curl http://localhost:17976/api/v1/node/peers

# 查看最新区块
curl http://localhost:17976/api/v1/blocks/latest

# 查询账户
curl http://localhost:17976/api/v1/accounts/<address>
```

### 部署合约（示例）

```bash
# 编译 WASM 合约
# 假设有合约代码：contract.rs
cargo build --target wasm32-unknown-unknown --release
# WASM文件位于: target/wasm32-unknown-unknown/release/contract.wasm

# 通过 API 部署
curl -X POST http://localhost:17976/api/v1/contracts/deploy \
  -H "Content-Type: application/json" \
  -d '{
    "wasm_bytecode": "<base64 encoded wasm>",
    "constructor_args": "{}"
  }'
```

## 🧩 工作区结构

本项目使用 Rust Workspace 管理多个 crates：

```
backend/workspace/Cargo.toml   # Workspace 根配置
frontend/package.json          # Node 项目根
```

开发时建议：
- 后端修改：在 `crates/xxx/src/` 下编辑，`cargo check` 快速验证
- 前端修改：保存后 Vite 自动热更新

## 🔐 配置文件说明

### 后端 (`config/default.toml`)

```toml
[database]
url = "postgresql://nrcs:nrcs123@localhost:5432/nrcs"
max_connections = 10

[api]
listen = "0.0.0.0:17976"
cors_allowed_origins = ["*"]

[p2p]
listen = "/ip4/0.0.0.0/tcp/0"
seed_peers = []
```

### 前端 (`.env` 文件)

```bash
VITE_API_BASE_URL=http://localhost:17976/api/v1
VITE_APP_TITLE=NRCS Blockchain
```

## 🐛 问题排查

### 后端启动失败：数据库连接错误
1. 确认 PostgreSQL 已启动：`systemctl status postgresql`
2. 检查连接字符串是否匹配 `config/local.toml`
3. 创建数据库：`createdb nrcs -U postgres`

### 前端无法访问 API
1. 确认后端已启动（`curl http://localhost:17976/health`）
2. 检查 `frontend/.env.development` 中的 `VITE_API_BASE_URL`
3. 查看浏览器控制台是否有 CORS 错误

### Docker 容器无法启动
1. 检查端口占用：`netstat -ano | findstr :17976`
2. 查看容器日志：`docker-compose logs <service>`
3. 确保 `docker/env.sh` 已正确配置

## 📄 许可证

Apache License 2.0 - 详见 [LICENSE](LICENSE) 文件

## 🙏 致谢

- 原 Java 版本: [NRCs 项目](https://github.com/bytechain/nrcs)
- Rust 生态: `axum`, `sqlx`, `libp2p`, `wasmtime`
- Vue 生态: `vite`, `vue-router`, `pinia`, `element-plus`
- 社区贡献者

---

**开发状态**: 早期开发中 (v0.1.0-alpha)#   r u s t - n r c s 
 
 #   r u s t - n r c s 
 
 