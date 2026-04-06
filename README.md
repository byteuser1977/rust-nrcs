# NRCS - Neo Rapid BlockChain EcoSystem

重构自 Java 的高性能区块链平台，采用 Rust + Vue 3 技术栈

[![CI/CD](https://github.com/bytechain/nrcs/workflows/Continuous%20Integration/badge.svg)](https://github.com/bytechain/nrcs/actions)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange)](https://www.rust-lang.org)
[![Vue](https://img.shields.io/badge/vue-3.4+-brightgreen)](https://vuejs.org)
[![Postgres](https://img.shields.io/badge/postgres-15+-blue)](https://www.postgresql.org)
[![WSL2](https://img.shields.io/badge/dev_env-WSL2-9cf)](https://learn.microsoft.com/zh-cn/windows/wsl/install)

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

**推荐环境：WSL2 (Windows Subsystem for Linux 2)**

本项目的编译和运行环境配置在 **WSL2** 下进行，已在以下环境验证：
- **OS**: Ubuntu 22.04 LTS (WSL2)
- **Rust**: 1.75+ (stable)
- **PostgreSQL**: 15+ (已安装并运行)
- **Redis**: 7+ (可选，已安装)
- **Node.js**: 20+ (前端编译)

> **开发环境说明**：所有 Rust 后端编译、数据库迁移、集成测试均在 WSL2 环境中执行。Windows 本地环境仅用于编辑器/IDE（如 VS Code）并通过文件系统访问 WSL 工作区。

**原生 Docker 部署**（适合生产环境）:
- Docker & Docker Compose
- Linux (Ubuntu 22.04, Debian 12, CentOS 8+)
- macOS 11+
- Windows 10+ (WSL2 backend)

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

### 方案二：WSL2 本地开发（推荐用于开发调试）

#### 1. 进入 WSL2 环境

```powershell
# 从 PowerShell / CMD 进入 WSL2
wsl
# 或直接启动Ubuntu终端
```

#### 2. 后端编译与启动

```bash
# 进入项目工作区（假设代码位于 Windows D: 盘）
cd /mnt/d/workspace/git/rust-nrcs

# 检查 Rust 环境
rustc --version  # 应为 1.75+
cargo --version

# 更新依赖
cargo fetch

# 配置文件
cp config/default.toml config/local.toml
# 编辑 config/local.toml，设置数据库连接（如 postgres://nrcs:password@localhost:5432/nrcs_db）

# 初始化数据库（如果尚未初始化）
sudo -u postgres psql -f crates/orm/migrations/001_initial.sql
# 或在 WSL 中已安装的 PostgreSQL 环境下执行：
psql -U postgres -d nrcs_db -f crates/orm/migrations/001_initial.sql

# 编译（Debug 模式，便于调试）
cargo build

# 或编译 Release 版本
cargo build --release

# 运行（带数据库迁移）
cargo run --bin node -- --migrate
# 或直接运行（配置文件会自动迁移）
cargo run --bin node -- --config config/local.toml
```

后端 API: http://localhost:17976 (WSL 网络中 localhost 可直接访问)

#### 3. 前端启动

```bash
# 在 WSL2 中（确保 Node.js 20+ 已安装）
cd /mnt/d/workspace/git/rust-nrcs/frontend
npm ci
npm run dev

# 或使用 Windows 本机 Node.js（通过文件路径 \\wsl$\Ubuntu\... 访问）
# 但建议前端在 WSL 内统一管理依赖
```

前端: http://localhost:5173

#### 4. 跨环境访问说明

- **WSL2 网络**：`localhost` 在 Windows 和 WSL 间互通（WSL2 默认使用 NAT，端口自动转发）
- **文件系统**：Windows 路径 `D:\...` 映射为 WSL 路径 `/mnt/d/...`
- **数据库**：PostgreSQL 运行在 WSL 内，连接地址 `localhost:5432` 即可

### 方案三：Docker 一键启动（生产环境）

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

| 文档 | 说明 |
|------|------|
| `architecture.md` | 系统架构与技术选型 |
| `database-schema.md` | 数据库设计 (55+ 表) |
| `frontend-architecture.md` | Vue 3 项目架构 |
| `api-integration-design.md` | API 集成层设计 |
| `deployment.md` | 生产环境部署指南（Docker） |
| `migration-guide.md` | 从 Java NRCs 迁移到 Rust |
| `quickstart_wsl.md` | **WSL2 开发环境快速指南**（新） |
| `orm-ice-diagnosis.md` | ORM ICE 问题诊断报告 |

API 文档在运行时可访问：
- Swagger UI: http://localhost:17976/api/docs
- ReDoc: http://localhost:17976/api/redoc

## 许可证

Apache License 2.0 - 详见 [LICENSE](LICENSE) 文件

---

开发状态: 早期开发中 (v0.1.0-alpha)