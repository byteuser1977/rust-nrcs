# NRCS 集成测试套件

本目录包含 NRCS Rust 实现的集成测试套件，用于验证与 Java NRCS 的兼容性。

## 目录结构

```
tests/
├── integration/           # 集成测试
│   ├── p2p/              # P2P 网络测试
│   │   ├── test_node_discovery.sh
│   │   └── test_block_sync.sh
│   ├── api/              # HTTP API 测试
│   │   └── test_api_compatibility.sh
│   ├── consensus/        # 共识算法测试
│   │   └── test_consensus.sh
│   └── run_all.sh        # 运行所有测试
├── crypto/               # 加密算法测试
│   └── test_crypto_compatibility.sh
├── performance/          # 性能测试
│   └── tps_test.js       # k6 性能测试脚本
├── data/                 # 测试数据
│   ├── test_vectors.json
│   ├── sample_blocks.json
│   └── sample_transactions.json
├── scripts/              # 工具脚本
│   ├── start_test_env.sh
│   └── stop_test_env.sh
└── reports/              # 测试报告输出目录
```

## 快速开始

### 1. 环境准备

确保以下依赖已安装：
- Rust 1.70+
- PostgreSQL 14+
- curl
- jq (用于 JSON 处理)
- k6 (可选，用于性能测试)

### 2. 配置环境变量

```bash
# Java NRCS 节点配置
export JAVA_NRCS_HOST="192.168.2.164"
export JAVA_NRCS_PORT="17976"
export JAVA_PEER_PORT="17974"

# Rust NRCS 节点配置
export RUST_NRCS_HOST="localhost"
export RUST_NRCS_PORT="17976"
export RUST_PEER_PORT="17974"

# 数据库配置
export DB_HOST="localhost"
export DB_PORT="5432"
export DB_NAME="nrcs_test"
export DB_USER="nrcs"
export DB_PASSWORD="password"
```

### 3. 启动测试环境

```bash
# 启动测试环境（包括数据库、Rust 节点等）
./tests/scripts/start_test_env.sh
```

### 4. 运行测试

```bash
# 运行所有集成测试
./tests/integration/run_all.sh

# 或单独运行某个测试套件
./tests/crypto/test_crypto_compatibility.sh
./tests/integration/p2p/test_node_discovery.sh
./tests/integration/api/test_api_compatibility.sh
```

### 5. 运行性能测试

```bash
# 使用 k6 运行性能测试
k6 run tests/performance/tps_test.js
```

### 6. 停止测试环境

```bash
# 停止测试环境
./tests/scripts/stop_test_env.sh
```

## 测试套件说明

### 1. 加密算法兼容性测试 (P0)

**位置**: `tests/crypto/test_crypto_compatibility.sh`

**测试内容**:
- Ed25519 签名验证
- NRCS 签名格式兼容性
- X25519 密钥交换
- SM2/SM3/SM4 国密算法

**运行**:
```bash
./tests/crypto/test_crypto_compatibility.sh
```

### 2. P2P 网络互通测试 (P0)

**位置**: `tests/integration/p2p/`

**测试内容**:
- 节点发现与握手
- 区块同步
- 交易传播

**运行**:
```bash
# 节点发现测试
./tests/integration/p2p/test_node_discovery.sh

# 区块同步测试
./tests/integration/p2p/test_block_sync.sh
```

### 3. HTTP API 兼容性测试 (P0)

**位置**: `tests/integration/api/test_api_compatibility.sh`

**测试内容**:
- 基础 API 端点
- 账户相关 API
- 区块相关 API
- 交易相关 API
- 工具 API

**运行**:
```bash
./tests/integration/api/test_api_compatibility.sh
```

### 4. 共识算法一致性测试 (P1)

**位置**: `tests/integration/consensus/test_consensus.sh`

**测试内容**:
- 出块者选择算法
- 难度计算
- 共识流程

**运行**:
```bash
./tests/integration/consensus/test_consensus.sh
```

### 5. 性能对比测试 (P2)

**位置**: `tests/performance/tps_test.js`

**测试内容**:
- TPS (Transactions Per Second)
- 延迟对比
- 资源占用

**运行**:
```bash
k6 run tests/performance/tps_test.js
```

## 测试报告

测试报告会自动生成在 `tests/reports/` 目录下：

- `test_report_YYYYMMDD_HHMMSS.md`: 测试报告
- `test_output_YYYYMMDD_HHMMSS.log`: 详细日志
- `rust_node.log`: Rust 节点日志
- `performance_report.json`: 性能测试报告

## 测试数据

测试数据位于 `tests/data/` 目录：

- `test_vectors.json`: 加密算法测试向量
- `sample_blocks.json`: 示例区块数据
- `sample_transactions.json`: 示例交易数据

## 故障排查

### 1. 无法连接到 Java 节点

**症状**: 测试提示无法连接到 Java NRCS 节点

**解决方案**:
- 检查 Java 节点是否正常运行
- 验证 `JAVA_NRCS_HOST` 和 `JAVA_NRCS_PORT` 环境变量
- 检查网络连接和防火墙设置

### 2. 数据库连接失败

**症状**: 测试提示数据库连接失败

**解决方案**:
- 检查 PostgreSQL 是否运行
- 验证数据库配置（DB_HOST, DB_PORT, DB_NAME, DB_USER, DB_PASSWORD）
- 确保数据库用户有创建数据库的权限

### 3. Rust 节点启动失败

**症状**: Rust 节点无法启动

**解决方案**:
- 检查 `tests/reports/rust_node.log` 日志文件
- 确保端口未被占用
- 验证配置文件 `config/local.toml`

### 4. 测试超时

**症状**: 区块同步测试超时

**解决方案**:
- 增加 `test_block_sync.sh` 中的等待时间
- 检查网络延迟
- 确保 Java 节点有足够的区块数据

## 持续集成

可以将测试集成到 CI/CD 流程中：

```yaml
# .github/workflows/integration-test.yml
name: Integration Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    
    services:
      postgres:
        image: postgres:14
        env:
          POSTGRES_USER: nrcs
          POSTGRES_PASSWORD: password
          POSTGRES_DB: nrcs_test
        ports:
          - 5432:5432
    
    steps:
      - uses: actions/checkout@v2
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Run integration tests
        run: |
          ./tests/scripts/start_test_env.sh
          ./tests/integration/run_all.sh
          ./tests/scripts/stop_test_env.sh
```

## 贡献指南

添加新的测试用例：

1. 在相应的测试目录下创建测试脚本
2. 遵循现有测试脚本的命名规范：`test_*.sh`
3. 确保测试脚本返回正确的退出码（0 表示成功，非 0 表示失败）
4. 更新本文档

## 联系方式

如有问题，请联系：
- 测试负责人: [待指定]
- 开发负责人: [待指定]
