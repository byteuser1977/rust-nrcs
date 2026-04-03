# NRCS - Neo Rapid BlockChain EcoSystem

重构自 Java 的高性能区块链平台，采用 Rust + Vue 3 技术栈

[![CI/CD](https://github.com/bytechain/nrcs/workflows/Continuous%20Integration/badge.svg)](https://github.com/bytechain/nrcs/actions)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange)](https://www.rust-lang.org)
[![Vue](https://img.shields.io/badge/vue-3.4+-brightgreen)](https://vuejs.org)
[![Postgres](https://img.shields.io/badge/postgres-15+-blue)](https://www.postgresql.org)

## 项目简介

NRCS 是一个高性能、可扩展的区块链平台，提供完整的交易处理、智能合约、P2P 网络等功能。本项目将原 Java 实现重构为 Rust，以获得更高的性能、更好的内存安全性和更低的资源消耗。

### 核心特性

- **高性能**：Rust 编写，TPS 达到原 Java 版本的 120%+
- **安全可靠**：WebAssembly 智能合约沙箱、内存安全保证
- **P2P 网络**：基于 libp2p 的去中心化通信
- **完整生态**：REST API + WebSocket + 管理后台
- **现代化 UI**：Vue 3 响应式界面，支持多语言
- **一键部署**：Docker + docker-compose

## 架构概览

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

## 技术栈

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

## 快速开始

### 前置要求

- Rust 1.75+
- Node.js 20+
- PostgreSQL 15+
- Redis 7+ (可选)
- Docker & Docker Compose (推荐)

### 方案一：Docker 一键启动

```bash
git clone https://github.com/byteuser1977/rust-nrcs.git
cd nrcs

cp docker/env.sh.example docker/env.sh
# 编辑 docker/env.sh 设置数据库密码、JWT 密钥等

docker-compose up -d
docker-compose logs -f

# 停止服务
docker-compose down
```

服务地址：
- 前端: http://localhost:80
- 后端 API: http://localhost:17976
- API 文档: http://localhost:17976/api/docs
- 健康检查: http://localhost:17976/health

### 方案二：本地开发

#### 后端启动

```bash
cargo fetch
cp config/default.toml config/local.toml
# 编辑 config/local.toml，设置数据库连接

psql -U postgres -f crates/orm/migrations/001_initial.sql

cargo build --release
cargo run --bin node --release
```

后端 API: http://localhost:17976

#### 前端启动

```bash
cd frontend
npm ci
npm run dev
```

前端: http://localhost:5173

## 测试

### 后端

```bash
cargo test --all-features
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
```

### 前端

```bash
cd frontend
npm run test:unit
npm run lint
npm run type-check
```

## 文档

完整文档位于 `docs/` 目录：
- architecture.md - 系统架构与技术选型
- database-schema.md - 数据库设计 (55+ 表)
- frontend-architecture.md - Vue 3 项目架构
- api-integration-design.md - API 集成层设计
- deployment.md - 生产环境部署指南
- migration-guide.md - 从 Java NRCs 迁移到 Rust

API 文档在运行时可访问：
- Swagger UI: http://localhost:17976/api/docs
- ReDoc: http://localhost:17976/api/redoc

## 许可证

Apache License 2.0 - 详见 [LICENSE](LICENSE) 文件

---

开发状态: 早期开发中 (v0.1.0-alpha)